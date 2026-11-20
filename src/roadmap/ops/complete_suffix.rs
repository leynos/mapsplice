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

use super::super::model::ItemSourceSpan;

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

/// Report whether the item's title already ends with the configured suffix.
///
/// The comparison collapses the title's source wrapping back into single
/// spaces and then peels trailing markup. Closing inline markers and terminal
/// punctuation may wrap one another, as in `**Finish — done**.`, so each layer
/// is removed and compared in turn; a formatted suffix such as `**done**` and a
/// suffix placed before the title's full stop both match. A suffix beginning
/// with a word character must start at a word or inline-marker boundary;
/// otherwise `done` would treat a title ending in `undone` as already complete.
pub(super) fn title_has_suffix(source: &str, span: ItemSourceSpan, suffix: &str) -> bool {
    let Some(raw) = source.get(span.title_start..span.paragraph_end) else {
        return false;
    };
    ends_with_suffix(&normalize_title(raw), suffix)
}

/// Match `suffix` at the end of `text`, ignoring trailing markup layers.
fn ends_with_suffix(text: &str, suffix: &str) -> bool {
    let mut candidate = text.trim_end();
    loop {
        if suffix_matches(candidate, suffix) {
            return true;
        }
        let Some(shorter) = peel_title_ending(candidate) else {
            return false;
        };
        candidate = shorter;
    }
}

/// Remove one trailing layer of inline markers or terminal punctuation.
///
/// Only the outermost layer goes, so repeated calls reach the plain title
/// however markers and terminal punctuation interleave.
fn peel_title_ending(text: &str) -> Option<&str> {
    let trimmed = text.trim_end();
    let without_markers = trimmed.trim_end_matches(INLINE_MARKERS);
    if without_markers.len() != trimmed.len() {
        return Some(without_markers.trim_end());
    }
    let without_punctuation = trimmed.trim_end_matches(is_terminal_punctuation);
    (without_punctuation.len() != trimmed.len()).then(|| without_punctuation.trim_end())
}

/// Match `suffix` at the end of `text`, ignoring terminal punctuation it precedes.
fn suffix_matches(text: &str, suffix: &str) -> bool {
    if suffix_supplies_punctuation(suffix) {
        ends_at_word_boundary(text, suffix)
    } else {
        let without_terminal = text.trim_end_matches(is_terminal_punctuation).trim_end();
        ends_at_word_boundary(without_terminal, suffix)
    }
}

/// Insert the done suffix into the addressed title in the patched text.
///
/// Returns `None` when the retained span does not fit the source, which the
/// caller reports as an unsafe edit. The suffix is placed before the title's
/// terminal sentence punctuation so `… scope — done.` replaces `… scope.`; a
/// suffix carrying its own punctuation replaces the title's run instead.
pub(super) fn insert_suffix(
    patched: &mut String,
    source: &str,
    span: ItemSourceSpan,
    suffix: &str,
) -> Option<()> {
    let slot = suffix_slot(source, span, suffix)?;
    if let Some(replaced) = slot.replaced.clone() {
        if !source.is_char_boundary(replaced.end) {
            return None;
        }
        patched.replace_range(replaced, "");
    }

    let line_start = source
        .get(..slot.insert_at)
        .and_then(|prefix| prefix.rfind(['\n', '\r']).map(|index| index + 1))
        .unwrap_or(0);
    let wrap = SuffixWrap {
        width: source
            .get(line_start..slot.insert_at)
            .map_or(0, |line| line.chars().count()),
        leading_space: !source
            .get(..slot.insert_at)
            .is_some_and(|prefix| prefix.ends_with([' ', '\t'])),
        terminal: slot.terminal,
        hanging: " ".repeat(title_column(source, span.title_start)),
        line_ending: detect_line_ending(source),
    };
    if !patched.is_char_boundary(slot.insert_at) {
        return None;
    }
    patched.insert_str(slot.insert_at, &wrap_suffix(suffix, &wrap));
    Some(())
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
/// Closing inline markers stay after the suffix so terminal punctuation
/// remains terminal inside emphasis, as in `**Finish the scope — done.**`.
fn suffix_slot<'source>(
    source: &'source str,
    span: ItemSourceSpan,
    suffix: &str,
) -> Option<SuffixSlot<'source>> {
    let raw = source.get(span.title_start..span.paragraph_end)?;
    let number_end = raw.find(". ").map(|index| span.title_start + index + 2)?;
    let trimmed_end = span.title_start + raw.trim_end_matches([' ', '\t']).len();
    let content_end = trimmed_end - trailing_closer_len(source, trimmed_end);
    let punctuation_start = content_end - trailing_punctuation_len(source, content_end);
    if punctuation_start <= number_end {
        return Some(SuffixSlot {
            insert_at: span.paragraph_end,
            terminal: "",
            replaced: None,
        });
    }
    if suffix_supplies_punctuation(suffix) {
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

/// Return the byte length of closing inline markers ending at `end`.
///
/// The scan is limited to marker characters so ordinary title words are never
/// treated as delimiters.
fn trailing_closer_len(source: &str, end: usize) -> usize {
    source.get(..end).map_or(0, |prefix| {
        prefix
            .chars()
            .rev()
            .take_while(|character| INLINE_MARKERS.contains(character))
            .map(char::len_utf8)
            .sum()
    })
}

/// Append the suffix words greedily, wrapping at [`WRAP_COLUMNS`].
fn wrap_suffix(suffix: &str, wrap: &SuffixWrap<'_>) -> String {
    let terminal_width = wrap.terminal.trim_end_matches([' ', '\t']).chars().count();
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
            if separator == 1 {
                rendered.push(' ');
            }
            rendered.push_str(word);
            width += separator + word_width + tail_width;
        } else {
            rendered.push_str(wrap.line_ending);
            rendered.push_str(&wrap.hanging);
            rendered.push_str(word);
            width = hanging_width + word_width + tail_width;
        }
    }
    rendered
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

/// Join a title's source lines into one single-spaced sentence.
fn normalize_title(raw: &str) -> String { raw.split_whitespace().collect::<Vec<_>>().join(" ") }

/// Match a suffix at the end of `text`, honouring word boundaries.
///
/// Inline marker characters count as a boundary because the suffix may sit
/// just inside emphasis, as in `Finish **done**` once the closing markers are
/// peeled.
fn ends_at_word_boundary(text: &str, suffix: &str) -> bool {
    let Some(prefix) = text.strip_suffix(suffix) else {
        return false;
    };
    let starts_with_word = suffix.chars().next().is_some_and(char::is_alphanumeric);
    !starts_with_word
        || prefix.chars().next_back().is_none_or(|character| {
            character.is_whitespace() || INLINE_MARKERS.contains(&character)
        })
}

/// Return the byte length of the terminal punctuation ending at `end`.
fn trailing_punctuation_len(source: &str, end: usize) -> usize {
    source.get(..end).map_or(0, |prefix| {
        prefix
            .chars()
            .rev()
            .take_while(|character| is_terminal_punctuation(*character))
            .map(char::len_utf8)
            .sum()
    })
}

/// Report whether a character is sentence-ending punctuation this edit moves.
const fn is_terminal_punctuation(character: char) -> bool {
    matches!(character, '.' | '!' | '?' | '…')
}

/// Report whether a suffix already ends with its own terminal punctuation.
fn suffix_supplies_punctuation(suffix: &str) -> bool {
    suffix
        .chars()
        .next_back()
        .is_some_and(is_terminal_punctuation)
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
