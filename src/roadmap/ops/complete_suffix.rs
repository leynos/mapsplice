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

/// Closing markers that may follow punctuation the title already supplies.
///
/// A backtick is excluded because stripping punctuation inside a code span
/// would change the construct's literal content.
const STRIPPABLE_MARKERS: [char; 3] = ['*', '_', '~'];

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
/// Returns `None` when the retained span does not fit the source or the suffix
/// cannot be placed without rewriting the title's punctuation, which the caller
/// reports as an unsafe edit. The title's terminal sentence punctuation is
/// authoritative: a suffix whose own punctuation can be removed is inserted
/// before that run, so `… scope.` becomes `… scope — done.` and
/// `Is it ready?` becomes `Is it ready — done?`. A suffix keeps its own
/// punctuation when the title has none. A suffix that ends with an inline
/// marker follows the title's closing markers, and a backslash that would
/// otherwise escape the punctuation, marker, or line ending after the inserted
/// words is paired so the author's literal backslash survives.
pub(super) fn insert_suffix(
    patched: &mut String,
    source: &str,
    span: ItemSourceSpan,
    suffix: &str,
) -> Option<()> {
    let placement = place_suffix(source, span, suffix)?;
    let wrap = suffix_wrap(source, span, &placement.slot);
    let insert_at = placement.slot.insert_at;
    if !patched.is_char_boundary(insert_at) {
        return None;
    }
    let mut inserted = wrap_suffix(&placement.text.text, &wrap)?;
    pair_leading_backslash(&mut inserted, source, &placement.slot, &wrap)?;
    pair_trailing_backslash(&mut inserted, source, &placement.slot, &wrap)?;
    patched.insert_str(insert_at, &inserted);
    Some(())
}

/// Build the wrapping inputs for the chosen suffix placement.
fn suffix_wrap<'source>(
    source: &'source str,
    span: ItemSourceSpan,
    slot: &SuffixSlot<'source>,
) -> SuffixWrap<'source> {
    let line_start = source
        .get(..slot.insert_at)
        .and_then(|prefix| prefix.rfind(['\n', '\r']).map(|index| index + 1))
        .unwrap_or(0);
    SuffixWrap {
        width: source
            .get(line_start..slot.insert_at)
            .map_or(0, |line| line.chars().count()),
        leading_space: !source
            .get(..slot.insert_at)
            .is_some_and(|prefix| prefix.ends_with([' ', '\t'])),
        terminal: slot.terminal,
        hanging: " ".repeat(title_column(source, span.title_start)),
        line_ending: detect_line_ending(source),
    }
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
    source
        .get(slot.insert_at..)
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

/// Text actually inserted for the configured suffix.
struct SuffixText {
    /// Suffix words with punctuation the title already supplies removed.
    text: String,
    /// Whether a terminal punctuation run was removed.
    stripped: bool,
}

/// Return the suffix text to insert.
///
/// With `strip` set, the suffix's terminal punctuation is removed when it
/// precedes only closing emphasis, strong, or strike markers, so `**done.**`
/// becomes `**done**` and a plain `— done.` becomes `— done`. A suffix whose
/// punctuation is not at the end (a link label, for example) or that ends in a
/// code span keeps its bytes, because removing punctuation there would change
/// the construct's literal content. Without `strip`, the suffix keeps its
/// punctuation because the title supplies none.
fn suffix_text(suffix: &str, strip: bool) -> SuffixText {
    let trimmed = suffix.trim_end();
    if !strip {
        return SuffixText {
            text: trimmed.to_owned(),
            stripped: false,
        };
    }
    let mut marker_start = trimmed.len();
    while let Some(character) = trimmed
        .get(..marker_start)
        .and_then(|text| text.chars().next_back())
    {
        if !STRIPPABLE_MARKERS.contains(&character) {
            break;
        }
        marker_start -= character.len_utf8();
    }
    let punctuation = trailing_punctuation_len(trimmed, marker_start);
    if punctuation == 0 {
        return SuffixText {
            text: trimmed.to_owned(),
            stripped: false,
        };
    }
    let core = trimmed
        .get(..marker_start - punctuation)
        .zip(trimmed.get(marker_start..));
    let Some((head, tail)) = core else {
        return SuffixText {
            text: trimmed.to_owned(),
            stripped: false,
        };
    };
    SuffixText {
        text: format!("{head}{tail}"),
        stripped: true,
    }
}

/// Where the suffix goes.
struct SuffixSlot<'source> {
    /// Byte offset at which the suffix words begin.
    insert_at: usize,
    /// Title bytes that follow the suffix and count toward the last width.
    terminal: &'source str,
}

/// Chosen insertion point and the text to insert there.
struct SuffixPlacement<'source> {
    /// Placement within the title.
    slot: SuffixSlot<'source>,
    /// Text to insert.
    text: SuffixText,
}

/// Choose the insertion point relative to the title's final content.
///
/// The parser reports where the title's inline content ends, before closing
/// emphasis, strong, and strike markers so terminal punctuation remains
/// terminal inside them, as in `**Finish the scope — done.**`, but after
/// literal constructs such as inline code and links. A suffix that ends with an
/// inline marker follows a title's closing markers, because joining two marker
/// runs would make the result ambiguous.
///
/// When the title has terminal punctuation, that run stays in place. A suffix
/// whose own punctuation cannot be removed, or whose marker run cannot be
/// separated from the title's closing run without displacing the punctuation,
/// has no safe placement and fails closed. The numbered prefix contains no
/// whitespace, so the first whitespace in the source paragraph ends it whatever
/// the separator's spelling; a separator written as an entity such as `&#46;`
/// has no literal `. ` to search for.
fn place_suffix<'source>(
    source: &'source str,
    span: ItemSourceSpan,
    suffix: &str,
) -> Option<SuffixPlacement<'source>> {
    let raw = source.get(span.title_start..span.paragraph_end)?;
    let number_end = raw
        .find([' ', '\t'])
        .map_or(span.paragraph_end, |index| span.title_start + index + 1);
    let trimmed_end = span.title_start + raw.trim_end_matches([' ', '\t']).len();
    let content_end = span.title_content_end.min(trimmed_end);
    let punctuation_start = content_end - trailing_punctuation_len(source, content_end);
    let has_title_punctuation = punctuation_start < content_end;
    let text = suffix_text(suffix, has_title_punctuation);
    if punctuation_start <= number_end {
        return Some(SuffixPlacement {
            slot: SuffixSlot {
                insert_at: span.paragraph_end,
                terminal: "",
            },
            text,
        });
    }
    // Decorations such as links end with syntax characters, so punctuation is
    // classified from the text a reader sees, not from the suffix's spelling.
    let supplies_punctuation = suffix_supplies_punctuation(&plain_suffix_text(suffix));
    if has_title_punctuation {
        // The title's bytes stay in place, so a suffix whose own punctuation
        // cannot be removed has no safe placement: replacing the title's
        // terminal character would rewrite the author's sentence.
        if punctuation_is_fixed(supplies_punctuation, &text) {
            return None;
        }
        // A marker-ending suffix cannot touch the title's closing markers, and
        // following them would leave the title's punctuation before the suffix.
        if follows_closing_markers(&text, content_end, trimmed_end) {
            return None;
        }
        return Some(SuffixPlacement {
            slot: SuffixSlot {
                insert_at: punctuation_start,
                terminal: source.get(punctuation_start..span.paragraph_end)?,
            },
            text,
        });
    }
    if follows_closing_markers(&text, content_end, trimmed_end) {
        // Placing marker-wrapped words directly before the title's closing
        // markers would merge both runs into one delimiter sequence, so such a
        // suffix follows the closing markers instead.
        return Some(SuffixPlacement {
            slot: SuffixSlot {
                insert_at: span.paragraph_end,
                terminal: "",
            },
            text,
        });
    }
    Some(SuffixPlacement {
        slot: SuffixSlot {
            insert_at: content_end,
            terminal: source.get(content_end..span.paragraph_end)?,
        },
        text,
    })
}

/// Report whether the suffix must follow the title's closing marker run.
fn follows_closing_markers(text: &SuffixText, content_end: usize, trimmed_end: usize) -> bool {
    content_end < trimmed_end && suffix_ends_with_marker(&text.text)
}

/// Report whether the suffix's punctuation cannot be separated from its construct.
const fn punctuation_is_fixed(supplies_punctuation: bool, text: &SuffixText) -> bool {
    supplies_punctuation && !text.stripped
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
