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

use markdown::mdast::Node;

use super::super::model::{ItemSourceSpan, MarkdownNodes};

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
/// The comparison uses the title's parsed inline nodes rather than its source
/// spelling, so a literal trailing marker such as the `*` in `build*` is never
/// mistaken for closing markup, while formatting around a real suffix is
/// ignored: `**Finish — done**.`, `Finish **done**`, and `**done.**` all match
/// their plain words. A suffix beginning with a word character must start at a
/// word boundary; otherwise `done` would treat a title ending in `undone` as
/// already complete.
pub(super) fn title_has_suffix(title: &MarkdownNodes, suffix: &str) -> bool {
    let plain_suffix = plain_suffix_text(suffix);
    if plain_suffix.is_empty() {
        return false;
    }
    suffix_matches(&plain_title_text(title.nodes()), &plain_suffix)
}

/// Collapse a title's parsed inline nodes into its rendered plain text.
fn plain_title_text(nodes: &[Node]) -> String {
    let mut text = String::new();
    for node in nodes {
        append_plain_text(node, &mut text);
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Append one node's rendered text, descending through inline containers.
fn append_plain_text(node: &Node, text: &mut String) {
    if let Some(value) = node_text(node) {
        text.push_str(value);
    } else if let Some(children) = node.children() {
        for child in children {
            append_plain_text(child, text);
        }
    } else {
        // A leaf without text, such as an image or a break, still separates the
        // words around it, so it can never join two words together.
        text.push(' ');
    }
}

/// Return the literal text one mdast node contributes, when it carries any.
fn node_text(node: &Node) -> Option<&str> {
    match node {
        Node::Text(content) => Some(&content.value),
        Node::InlineCode(content) => Some(&content.value),
        Node::InlineMath(content) => Some(&content.value),
        Node::Math(content) => Some(&content.value),
        _ => None,
    }
}

/// Reduce a configured suffix to the plain words a reader would see.
fn plain_suffix_text(suffix: &str) -> String {
    suffix
        .chars()
        .filter(|character| !INLINE_MARKERS.contains(character))
        .collect::<String>()
        .trim()
        .to_owned()
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
/// suffix carrying its own punctuation replaces the title's run instead. A
/// suffix that ends with an inline marker follows the title's closing markers,
/// and the width of any replaced punctuation is credited back to the line.
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
    patched.insert_str(insert_at, &wrap_suffix(suffix, &wrap));
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
/// The parser reports where the title's inline content ends, before closing
/// emphasis, strong, and strike markers so terminal punctuation remains
/// terminal inside them, as in `**Finish the scope — done.**`, but after
/// literal constructs such as inline code and links. A suffix that itself ends
/// with an inline marker is placed after the closing markers instead, because
/// joining two marker runs would make the result ambiguous.
fn suffix_slot<'source>(
    source: &'source str,
    span: ItemSourceSpan,
    suffix: &str,
) -> Option<SuffixSlot<'source>> {
    let raw = source.get(span.title_start..span.paragraph_end)?;
    let number_end = raw.find(". ").map(|index| span.title_start + index + 2)?;
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
    if suffix_ends_with_marker(suffix) {
        // Placing marker-wrapped words directly before the title's closing
        // markers would merge both runs into one delimiter sequence, so such a
        // suffix follows the closing markers instead. Punctuation it supplies
        // still replaces the title's trailing run.
        let replaced = (suffix_supplies_punctuation(suffix) && punctuation_start < content_end)
            .then_some(punctuation_start..content_end);
        return Some(SuffixSlot {
            insert_at: span.paragraph_end,
            terminal: "",
            replaced,
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

/// Match a suffix at the end of `text`, honouring word boundaries.
fn ends_at_word_boundary(text: &str, suffix: &str) -> bool {
    let Some(prefix) = text.strip_suffix(suffix) else {
        return false;
    };
    let starts_with_word = suffix.chars().next().is_some_and(char::is_alphanumeric);
    !starts_with_word || prefix.chars().next_back().is_none_or(char::is_whitespace)
}

/// Return the byte length of the terminal punctuation ending at `end`.
///
/// Punctuation escaped by a backslash is literal title content, so the scan
/// stops at it and the suffix follows the escaped character.
fn trailing_punctuation_len(source: &str, end: usize) -> usize {
    let Some(text) = source.get(..end) else {
        return 0;
    };
    let mut length = 0;
    for (offset, character) in text.char_indices().rev() {
        // Only ASCII punctuation can be backslash-escaped; the ellipsis is
        // always literal punctuation, so a backslash before it is content.
        let escaped = character.is_ascii() && is_escaped(text, offset);
        if !is_terminal_punctuation(character) || escaped {
            break;
        }
        length += character.len_utf8();
    }
    length
}

/// Report whether the character at `offset` follows an odd run of backslashes.
fn is_escaped(text: &str, offset: usize) -> bool {
    let backslashes = text.get(..offset).map_or(0, |prefix| {
        prefix
            .chars()
            .rev()
            .take_while(|character| *character == '\\')
            .count()
    });
    !backslashes.is_multiple_of(2)
}

/// Report whether a character is sentence-ending punctuation this edit moves.
const fn is_terminal_punctuation(character: char) -> bool {
    matches!(character, '.' | '!' | '?' | '…')
}

/// Report whether a suffix ends with an inline marker character.
///
/// Such a suffix cannot be placed immediately before the title's closing
/// markers: the two marker runs would touch and read as one ambiguous
/// delimiter sequence.
fn suffix_ends_with_marker(suffix: &str) -> bool {
    suffix
        .trim_end()
        .chars()
        .next_back()
        .is_some_and(|character| INLINE_MARKERS.contains(&character))
}

/// Report whether a suffix carries its own terminal punctuation.
///
/// Closing inline markers are ignored so a formatted suffix such as
/// `**done.**` still counts as punctuated.
fn suffix_supplies_punctuation(suffix: &str) -> bool {
    suffix
        .trim_end_matches(INLINE_MARKERS)
        .trim_end()
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
