//! Done-suffix detection and placement for completion edits.
//!
//! The suffix is placed from the parser's retained title span: before the
//! title's terminal sentence punctuation, inside any closing emphasis markers,
//! and after the title text so nested bullets and numbering keep their place.
//! Word wrapping, hanging indents, and backslash pairing live in
//! [`super::complete_wrap`]; this module decides where the inserted words go
//! and which existing title bytes they displace.
//!
//! The placement helpers are deliberately total over a validated span: they
//! return `None` rather than construct a diagnostic when an offset does not
//! fit, leaving the caller to report fail-closed completion errors.

use std::ops::Range;

pub(super) use super::complete_text::title_has_suffix;
use super::{
    super::model::ItemSourceSpan,
    complete_text::{plain_suffix_text, suffix_supplies_punctuation, trailing_punctuation_len},
    complete_wrap::{
        SuffixWrap,
        WRAP_COLUMNS,
        ends_with_unpaired_backslash,
        last_line_width,
        wrap_suffix,
    },
};

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
    let layout = SuffixLayout::new(source, span, &slot);
    remove_replaced(patched, source, slot.replaced.as_ref())?;
    let insert_at = slot.insert_at.checked_sub(layout.removed_before)?;
    if !patched.is_char_boundary(insert_at) {
        return None;
    }
    let mut inserted = wrap_suffix(suffix, &layout.wrap)?;
    pair_leading_backslash(&mut inserted, source, &slot, &layout.wrap)?;
    pair_trailing_backslash(&mut inserted, source, &slot, &layout.wrap)?;
    patched.insert_str(insert_at, &inserted);
    Some(())
}

/// Computed layout state for one suffix insertion.
struct SuffixLayout<'source> {
    /// Bytes removed before the insertion point, which shift its patched offset.
    removed_before: usize,
    /// Wrapping inputs for the inserted words.
    wrap: SuffixWrap<'source>,
}

impl<'source> SuffixLayout<'source> {
    /// Measure the title line and the bytes the chosen slot displaces.
    fn new(source: &'source str, span: ItemSourceSpan, slot: &SuffixSlot<'source>) -> Self {
        // Only punctuation before the insertion point shifts its patched
        // offset and frees width on the last line. Punctuation at the
        // insertion point is removed ahead of the suffix, so both stay put.
        let removed_before = slot.replaced.as_ref().map_or(0, |range| {
            range.end.min(slot.insert_at) - range.start.min(slot.insert_at)
        });
        let removed_width = slot.replaced.as_ref().map_or(0, |range| {
            source
                .get(range.start..range.end.min(slot.insert_at))
                .map_or(0, |removed| removed.chars().count())
        });
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
        Self {
            removed_before,
            wrap,
        }
    }
}

/// Delete the title bytes the suffix displaces from the patched text.
fn remove_replaced(
    patched: &mut String,
    source: &str,
    replaced: Option<&Range<usize>>,
) -> Option<()> {
    let Some(range) = replaced else {
        return Some(());
    };
    if !source.is_char_boundary(range.end) {
        return None;
    }
    patched.replace_range(range.clone(), "");
    Some(())
}

/// Pair a backslash that would otherwise escape the inserted line break.
///
/// A backslash immediately before a line break is a CommonMark hard break, so
/// the author's literal backslash would vanish from the rendered title.
/// Pairing it with one inserted backslash keeps the visible text unchanged. A
/// line already at the column limit cannot take that character, so the edit
/// fails closed instead.
fn pair_leading_backslash(
    inserted: &mut String,
    source: &str,
    slot: &SuffixSlot<'_>,
    wrap: &SuffixWrap<'_>,
) -> Option<()> {
    let starts_with_break = inserted.starts_with(wrap.line_ending);
    let ends_with_escape = ends_with_unpaired_backslash(source.get(..slot.insert_at)?);
    if !starts_with_break || !ends_with_escape {
        return Some(());
    }
    if wrap.width >= WRAP_COLUMNS {
        return None;
    }
    inserted.insert(0, '\\');
    Some(())
}

/// Pair a suffix backslash that would escape the character after it.
///
/// The suffix's own trailing backslash would escape the punctuation, closing
/// marker, or line ending that follows it. A last line already at the column
/// limit cannot take the extra character, so the edit fails closed instead.
fn pair_trailing_backslash(
    inserted: &mut String,
    source: &str,
    slot: &SuffixSlot<'_>,
    wrap: &SuffixWrap<'_>,
) -> Option<()> {
    let followed_by_escape = following_character(source, slot).is_some_and(is_escaping_follower);
    if !ends_with_unpaired_backslash(inserted) || !followed_by_escape {
        return Some(());
    }
    if last_line_width(inserted, wrap.width) + wrap.terminal_width() >= WRAP_COLUMNS {
        return None;
    }
    inserted.push('\\');
    Some(())
}

/// Return the source character that follows the inserted suffix, if any.
fn following_character(source: &str, slot: &SuffixSlot<'_>) -> Option<char> {
    let removed_end = slot
        .replaced
        .as_ref()
        .map_or(slot.insert_at, |range| range.end.max(slot.insert_at));
    source
        .get(removed_end..)
        .and_then(|rest| rest.chars().next())
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
