//! Rendered plain text and word matching for completion suffixes.
//!
//! `mapsplice complete` decides whether a title already carries its configured
//! done suffix by comparing the text a reader sees, not the source spelling.
//! Both the title and the suffix are parsed as Markdown and flattened, so
//! formatting is ignored while literal trailing markers survive. Suffix
//! placement shares [`plain_suffix_text`] and [`suffix_supplies_punctuation`],
//! so it classifies punctuation from rendered text as well.

use markdown::mdast::Node;

use super::super::model::MarkdownNodes;

/// Report whether the item's title already ends with the configured suffix.
///
/// The comparison uses the title's parsed inline nodes rather than its source
/// spelling, so a literal trailing marker such as the `*` in `build*` is never
/// mistaken for closing markup, while formatting around a real suffix is
/// ignored: `**Finish — done**.`, `Finish **done**`, and `**done.**` all match
/// their plain words. The configured suffix is rendered the same way, so a
/// suffix carrying its own literal marker, such as `build*`, still matches
/// itself. A suffix beginning with a word character must start at a word
/// boundary, where punctuation separates words as well as whitespace does;
/// otherwise `done` would treat a title ending in `undone` as already
/// complete.
pub(super) fn title_has_suffix(title: &MarkdownNodes, suffix: &str) -> bool {
    let plain_suffix = plain_suffix_text(suffix);
    if plain_suffix.is_empty() {
        return false;
    }
    suffix_matches(&plain_title_text(title.nodes()), &plain_suffix)
}

/// Reduce a configured suffix to the plain words a reader would see.
///
/// Parsing the suffix removes only delimiters that really pair up, so a
/// configured `build*` keeps its literal asterisk while `**done.**` compares
/// as `done.`. Text the parser cannot handle falls back to its trimmed source.
pub(super) fn plain_suffix_text(suffix: &str) -> String {
    let Ok(document) = markdown::to_mdast(suffix, &markdown::ParseOptions::gfm()) else {
        return collapse_whitespace(suffix);
    };
    document.children().map_or_else(
        || collapse_whitespace(suffix),
        |children| plain_title_text(children),
    )
}

/// Report whether rendered suffix text carries its own terminal punctuation.
///
/// The caller passes the suffix's rendered plain text, so closing syntax such
/// as the `)` of a link never hides punctuation or stands in for it.
pub(super) fn suffix_supplies_punctuation(plain: &str) -> bool {
    plain
        .trim_end()
        .chars()
        .next_back()
        .is_some_and(is_terminal_punctuation)
}

/// Collapse a title's parsed inline nodes into its rendered plain text.
fn plain_title_text(nodes: &[Node]) -> String {
    let mut text = String::new();
    for node in nodes {
        append_plain_text(node, &mut text);
    }
    collapse_whitespace(&text)
}

/// Collapse every whitespace run in `text` to one space and trim the ends.
fn collapse_whitespace(text: &str) -> String {
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

/// Match `suffix` at the end of `text`, ignoring terminal punctuation it precedes.
fn suffix_matches(text: &str, suffix: &str) -> bool {
    if suffix_supplies_punctuation(suffix) {
        ends_at_word_boundary(text, suffix)
    } else {
        let without_terminal = text.trim_end_matches(is_terminal_punctuation).trim_end();
        ends_at_word_boundary(without_terminal, suffix)
    }
}

/// Match a suffix at the end of `text`, honouring word boundaries.
fn ends_at_word_boundary(text: &str, suffix: &str) -> bool {
    let Some(prefix) = text.strip_suffix(suffix) else {
        return false;
    };
    let starts_with_word = suffix.chars().next().is_some_and(char::is_alphanumeric);
    !starts_with_word || prefix.chars().next_back().is_none_or(is_word_separator)
}

/// Report whether a character separates words rather than continuing one.
///
/// Punctuation separates a suffix from the preceding title, so `Status:done`
/// already ends with `done`. An underscore binds a compound word, so
/// `foo_done` does not.
fn is_word_separator(character: char) -> bool { !character.is_alphanumeric() && character != '_' }

/// Report whether a character is sentence-ending punctuation this edit moves.
pub(super) const fn is_terminal_punctuation(character: char) -> bool {
    matches!(character, '.' | '!' | '?' | '…')
}
