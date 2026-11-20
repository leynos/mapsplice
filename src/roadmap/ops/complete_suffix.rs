//! Done-suffix detection, placement, and wrapping for completion edits.
//!
//! Repository Markdown is wrapped at 80 columns, and `mdtablefix --wrap`
//! reflows an overflowing line using a hanging indent equal to the paragraph
//! content column: six spaces beneath `- [ ] `, eight beneath `  - [ ] `. It
//! leaves every other line alone. A suffix inserted by `mapsplice complete`
//! therefore refills only the item's final title line, greedily, at the same
//! 80-column limit and with the same hanging indent and document line ending.
//! Formatter-clean input then stays formatter-clean without any other line
//! being rewritten.
//!
//! The layout helpers are deliberately total over a validated span: they
//! return `None` rather than construct a diagnostic when an offset does not
//! fit, leaving the caller to report fail-closed completion errors.

use std::ops::Range;

pub(super) use super::complete_text::title_has_suffix;
use super::{
    super::model::ItemSourceSpan,
    complete_text::{plain_suffix_text, suffix_supplies_punctuation, trailing_punctuation_len},
};

/// Column limit for Markdown paragraphs and bullets in this repository.
const WRAP_COLUMNS: usize = 80;

/// Characters that open or close inline emphasis, strong, code, or strike spans.
const INLINE_MARKERS: [char; 4] = ['*', '_', '`', '~'];

/// Collapse the configured suffix into the single-line words that are inserted.
///
/// A suffix that is empty or only whitespace selects the default of no suffix.
/// The CLI rejects embedded line breaks before configuration reaches this
/// module; any whitespace run that remains becomes one separating space, so a
/// suffix can never introduce a paragraph break.
pub(super) fn normalize_suffix(suffix: Option<&str>) -> Option<String> {
    suffix
        .map(|text| text.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|text| !text.is_empty())
}

/// Insert the done suffix into the addressed title in the patched text.
///
/// Returns `None` when the retained span does not fit the source or a pairing
/// character cannot fit, which the caller reports as an unsafe edit. The suffix
/// is placed before the title's terminal sentence punctuation so
/// `… scope — done.` replaces `… scope.`; a suffix carrying its own punctuation
/// replaces the title's run instead. A suffix that ends with an inline marker
/// follows the title's closing markers, and the width of any replaced
/// punctuation is credited back to the line. A backslash that would otherwise
/// escape the punctuation, marker, or line ending after the inserted words is
/// paired so the author's literal backslash survives.
pub(super) fn insert_suffix(
    patched: &mut String,
    source: &str,
    span: ItemSourceSpan,
    suffix: &str,
) -> Option<()> {
    let slot = suffix_slot(source, span, suffix)?;
    // Only punctuation before the insertion point shifts its patched offset and
    // frees width on the last line. Punctuation at the insertion point is removed
    // ahead of the suffix, so both stay put.
    let removed_before = slot.replaced.as_ref().map_or(0, |range| {
        range.end.min(slot.insert_at) - range.start.min(slot.insert_at)
    });
    let removed_width = slot.replaced.as_ref().map_or(0, |range| {
        source
            .get(range.start..range.end.min(slot.insert_at))
            .map_or(0, |removed| removed.chars().count())
    });
    if let Some(replaced) = slot.replaced.clone() {
        if !source.is_char_boundary(replaced.end) {
            return None;
        }
        patched.replace_range(replaced, "");
    }

    let insert_at = slot.insert_at.checked_sub(removed_before)?;
    let line_start = source
        .get(..slot.insert_at)
        .and_then(|prefix| prefix.rfind(['\n', '\r']).map(|index| index + 1))
        .unwrap_or(0);
    let wrap = SuffixWrap {
        width: source
            .get(line_start..slot.insert_at)
            .map_or(0, |line| line.chars().count())
            .saturating_sub(removed_width),
        leading_space: !source
            .get(..slot.insert_at)
            .is_some_and(|prefix| prefix.ends_with([' ', '\t'])),
        terminal: slot.terminal,
        hanging: " ".repeat(title_column(source, span.title_start)),
        line_ending: detect_line_ending(source),
    };
    if !patched.is_char_boundary(insert_at) {
        return None;
    }
    let mut inserted = wrap_suffix(suffix, &wrap)?;
    if inserted.starts_with(wrap.line_ending)
        && ends_with_unpaired_backslash(source.get(..slot.insert_at)?)
    {
        // A backslash immediately before a line break is a CommonMark hard
        // break, so the author's literal backslash would vanish from the
        // rendered title. Pairing it with one inserted backslash keeps the
        // visible text unchanged. A line already at the column limit cannot
        // take that character, so the edit fails closed instead.
        if wrap.width >= WRAP_COLUMNS {
            return None;
        }
        inserted.insert(0, '\\');
    }
    let removed_end = slot
        .replaced
        .as_ref()
        .map_or(slot.insert_at, |range| range.end.max(slot.insert_at));
    let followed_by_escape = source
        .get(removed_end..)
        .and_then(|rest| rest.chars().next())
        .is_some_and(is_escaping_follower);
    if ends_with_unpaired_backslash(&inserted) && followed_by_escape {
        // The suffix's own trailing backslash would escape the punctuation,
        // closing marker, or line ending that follows it, so pair it the same
        // way. A last line already at the column limit cannot take the extra
        // character, so the edit fails closed instead.
        if last_line_width(&inserted, wrap.width) + wrap.terminal_width() >= WRAP_COLUMNS {
            return None;
        }
        inserted.push('\\');
    }
    patched.insert_str(insert_at, &inserted);
    Some(())
}

/// Report whether a preceding unpaired backslash would consume `character`.
///
/// CommonMark turns a backslash before a line ending into a hard break and
/// removes it, and a backslash before ASCII punctuation into a literal
/// character. In both cases the author's backslash would vanish unless it is
/// paired.
const fn is_escaping_follower(character: char) -> bool {
    character.is_ascii_punctuation() || matches!(character, '\n' | '\r')
}

/// Return the display width of the last line in `rendered`.
///
/// The first line follows the title text already on it, so its width adds
/// `first_line_width`; continuation lines carry their own hanging indent.
fn last_line_width(rendered: &str, first_line_width: usize) -> usize {
    rendered.rfind(['\n', '\r']).map_or_else(
        || first_line_width + rendered.chars().count(),
        |index| {
            rendered
                .get(index + 1..)
                .map_or(0, |line| line.chars().count())
        },
    )
}

/// Where the suffix goes and which title bytes it displaces.
struct SuffixSlot<'source> {
    /// Byte offset at which the suffix words begin.
    insert_at: usize,
    /// Title bytes that follow the suffix and count toward the last width.
    terminal: &'source str,
    /// Existing title punctuation the suffix replaces, if any.
    replaced: Option<Range<usize>>,
}

/// Choose the insertion point relative to the title's final content.
///
/// The parser reports where the title's inline content ends, before closing
/// emphasis, strong, and strike markers so terminal punctuation remains
/// terminal inside them, as in `**Finish the scope — done.**`, but after
/// literal constructs such as inline code and links. When the title ends with
/// closing markers and the suffix itself ends with an inline marker, the suffix
/// is placed after those markers instead, because joining two marker runs would
/// make the result ambiguous. A title without closing markers keeps the usual
/// placement, so terminal punctuation stays last.
///
/// The numbered prefix contains no whitespace, so the first whitespace in the
/// source paragraph ends it whatever the separator's spelling. A separator
/// written as an entity such as `&#46;` has no literal `. ` to search for.
fn suffix_slot<'source>(
    source: &'source str,
    span: ItemSourceSpan,
    suffix: &str,
) -> Option<SuffixSlot<'source>> {
    let raw = source.get(span.title_start..span.paragraph_end)?;
    let number_end = raw
        .find([' ', '\t'])
        .map_or(span.paragraph_end, |index| span.title_start + index + 1);
    let trimmed_end = span.title_start + raw.trim_end_matches([' ', '\t']).len();
    let content_end = span.title_content_end.min(trimmed_end);
    let punctuation_start = content_end - trailing_punctuation_len(source, content_end);
    if punctuation_start <= number_end {
        return Some(SuffixSlot {
            insert_at: span.paragraph_end,
            terminal: "",
            replaced: None,
        });
    }
    // Decorations such as links end with syntax characters, so punctuation is
    // classified from the text a reader sees, not from the suffix's spelling.
    let supplies_punctuation = suffix_supplies_punctuation(&plain_suffix_text(suffix));
    if suffix_ends_with_marker(suffix) && content_end < trimmed_end {
        // Placing marker-wrapped words directly before the title's closing
        // markers would merge both runs into one delimiter sequence, so such a
        // suffix follows the closing markers instead. Punctuation it supplies
        // still replaces the title's trailing run. A title whose content runs
        // to its end has no closing run, so the marker suffix keeps the usual
        // placement before terminal punctuation.
        let replaced = (supplies_punctuation && punctuation_start < content_end)
            .then_some(punctuation_start..content_end);
        return Some(SuffixSlot {
            insert_at: span.paragraph_end,
            terminal: "",
            replaced,
        });
    }
    if supplies_punctuation {
        let replaced = (punctuation_start < content_end).then_some(punctuation_start..content_end);
        return Some(SuffixSlot {
            insert_at: punctuation_start,
            // Any closing inline markers survive the punctuation removal and
            // still count toward the last line's width.
            terminal: source.get(content_end..span.paragraph_end)?,
            replaced,
        });
    }
    Some(SuffixSlot {
        insert_at: punctuation_start,
        terminal: source.get(punctuation_start..span.paragraph_end)?,
        replaced: None,
    })
}

/// Append the suffix words greedily, wrapping at [`WRAP_COLUMNS`].
///
/// Returns `None` when a generated break cannot be made safe, so the caller
/// fails closed rather than changing the rendered title.
fn wrap_suffix(suffix: &str, wrap: &SuffixWrap<'_>) -> Option<String> {
    let terminal_width = wrap.terminal_width();
    let hanging_width = wrap.hanging.chars().count();
    let words = suffix.split_whitespace().collect::<Vec<_>>();
    let mut rendered = String::new();
    let mut width = wrap.width;
    for (index, word) in words.iter().enumerate() {
        let is_last = index + 1 == words.len();
        let tail_width = if is_last { terminal_width } else { 0 };
        let word_width = word.chars().count();
        let separator = if index == 0 {
            usize::from(wrap.leading_space)
        } else {
            1
        };
        if width + separator + word_width + tail_width <= WRAP_COLUMNS {
            push_word(&mut rendered, word, separator);
            width += separator + word_width + tail_width;
        } else {
            start_continuation_line(&mut rendered, width, wrap, word)?;
            width = hanging_width + word_width + tail_width;
        }
    }
    Some(rendered)
}

/// Append a word to the current line, preceded by `separator` spaces.
fn push_word(rendered: &mut String, word: &str, separator: usize) {
    if separator == 1 {
        rendered.push(' ');
    }
    rendered.push_str(word);
}

/// Start a continuation line at the hanging indent with `word` on it.
///
/// A line that would end with an unpaired backslash gets one more backslash so
/// the literal character survives the break. Returns `None` when the line is
/// already at the column limit and cannot take that character.
fn start_continuation_line(
    rendered: &mut String,
    width: usize,
    wrap: &SuffixWrap<'_>,
    word: &str,
) -> Option<()> {
    if !rendered.is_empty() && ends_with_unpaired_backslash(rendered) {
        if width >= WRAP_COLUMNS {
            return None;
        }
        rendered.push('\\');
    }
    rendered.push_str(wrap.line_ending);
    rendered.push_str(&wrap.hanging);
    rendered.push_str(word);
    Some(())
}

/// Layout inputs for wrapping an inserted suffix.
struct SuffixWrap<'source> {
    /// Display width of the title line before the insertion point.
    width: usize,
    /// Whether a space must separate the title from the first suffix word.
    leading_space: bool,
    /// Title punctuation that stays in the source after the inserted words.
    ///
    /// It contributes to the last line's width but is not re-emitted, because
    /// the original bytes already follow the insertion point.
    terminal: &'source str,
    /// Indentation for continuation lines.
    hanging: String,
    /// Line ending used by the document.
    line_ending: &'source str,
}

impl SuffixWrap<'_> {
    /// Display width of the title punctuation that follows the suffix.
    fn terminal_width(&self) -> usize {
        self.terminal.trim_end_matches([' ', '\t']).chars().count()
    }
}

/// Report whether `text` ends with a backslash that is not itself escaped.
fn ends_with_unpaired_backslash(text: &str) -> bool {
    let backslashes = text
        .chars()
        .rev()
        .take_while(|character| *character == '\\')
        .count();
    !backslashes.is_multiple_of(2)
}

/// Report whether a suffix ends with an inline marker character.
///
/// Such a suffix cannot be placed immediately before a closing marker run: the
/// two runs would touch and read as one ambiguous delimiter sequence.
fn suffix_ends_with_marker(suffix: &str) -> bool {
    suffix
        .trim_end()
        .chars()
        .next_back()
        .is_some_and(|character| INLINE_MARKERS.contains(&character))
}

/// Return the character column at which the numbered title begins.
fn title_column(source: &str, title_start: usize) -> usize {
    let line_start = source
        .get(..title_start)
        .and_then(|prefix| prefix.rfind(['\n', '\r']).map(|index| index + 1))
        .unwrap_or(0);
    source
        .get(line_start..title_start)
        .map_or(0, |indent| indent.chars().count())
}

/// Return the line ending used by the document, defaulting to a newline.
fn detect_line_ending(source: &str) -> &'static str {
    let Some(index) = source.find(['\n', '\r']) else {
        return "\n";
    };
    match source.as_bytes().get(index) {
        Some(b'\r') if source.as_bytes().get(index + 1) == Some(&b'\n') => "\r\n",
        Some(b'\r') => "\r",
        _ => "\n",
    }
}
