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

use super::super::model::ItemSourceSpan;

/// Column limit for Markdown paragraphs and bullets in this repository.
const WRAP_COLUMNS: usize = 80;

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
/// spaces. A suffix without its own terminal punctuation is matched against
/// the title with that punctuation removed, because an earlier run placed the
/// suffix before it. A suffix beginning with a word character must also start
/// at a word boundary; otherwise `done` would treat a title ending in `undone`
/// as already complete.
pub(super) fn title_has_suffix(source: &str, span: ItemSourceSpan, suffix: &str) -> bool {
    let Some(raw) = source.get(span.title_start..span.paragraph_end) else {
        return false;
    };
    let normalized = normalize_title(raw);
    if suffix_supplies_punctuation(suffix) {
        ends_at_word_boundary(&normalized, suffix)
    } else {
        let without_terminal = normalized
            .trim_end_matches(is_terminal_punctuation)
            .trim_end();
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
    let raw = source.get(span.title_start..span.paragraph_end)?;
    let number_end = raw.find(". ").map(|index| span.title_start + index + 2)?;
    let trimmed_end = span.title_start + raw.trim_end_matches([' ', '\t']).len();
    let punctuation_start = trimmed_end - trailing_punctuation_len(source, trimmed_end);
    let suffix_owns_punctuation = suffix_supplies_punctuation(suffix);
    let title_is_empty = punctuation_start <= number_end;
    let (insert_at, terminal) = if title_is_empty {
        (span.paragraph_end, "")
    } else if suffix_owns_punctuation {
        (punctuation_start, "")
    } else {
        (
            punctuation_start,
            source.get(punctuation_start..span.paragraph_end)?,
        )
    };
    let replaces_title_punctuation =
        suffix_owns_punctuation && !title_is_empty && punctuation_start < trimmed_end;
    if replaces_title_punctuation {
        if !source.is_char_boundary(trimmed_end) {
            return None;
        }
        patched.replace_range(punctuation_start..trimmed_end, "");
    }

    let line_start = source
        .get(..insert_at)
        .and_then(|prefix| prefix.rfind(['\n', '\r']).map(|index| index + 1))
        .unwrap_or(0);
    let wrap = SuffixWrap {
        width: source
            .get(line_start..insert_at)
            .map_or(0, |line| line.chars().count()),
        leading_space: !source
            .get(..insert_at)
            .is_some_and(|prefix| prefix.ends_with([' ', '\t'])),
        terminal,
        hanging: " ".repeat(title_column(source, span.title_start)),
        line_ending: detect_line_ending(source),
    };
    if !patched.is_char_boundary(insert_at) {
        return None;
    }
    patched.insert_str(insert_at, &wrap_suffix(suffix, &wrap));
    Some(())
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
fn ends_at_word_boundary(text: &str, suffix: &str) -> bool {
    let Some(prefix) = text.strip_suffix(suffix) else {
        return false;
    };
    let starts_with_word = suffix.chars().next().is_some_and(char::is_alphanumeric);
    !starts_with_word || prefix.chars().next_back().is_none_or(char::is_whitespace)
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
