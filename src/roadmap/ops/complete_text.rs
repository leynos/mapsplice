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

/// Punctuation that keeps a parsed suffix in paragraph context.
///
/// A suffix is inline title content, but parsing it as a standalone document
/// would read a leading `#`, `>`, or list marker as block syntax and compare
/// only the block's text. Parsing behind this prefix keeps the suffix in one
/// paragraph, and the prefix is removed from the rendered comparison text.
const PARAGRAPH_PREFIX: char = '(';

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
/// as `done.`. The suffix is parsed as inline title content, so a leading `#`
/// stays literal text rather than becoming a heading marker that would hide
/// it from the comparison. Text the parser cannot handle falls back to its
/// trimmed source.
pub(super) fn plain_suffix_text(suffix: &str) -> String {
    fragment_plain_text(suffix).unwrap_or_else(|| collapse_whitespace(suffix))
}

/// Render one Markdown fragment as the plain inline text a reader sees.
///
/// The fragment is parsed behind [`PARAGRAPH_PREFIX`], so block syntax stays
/// literal and remains part of the comparison text. Returns `None` when the
/// parser rejects the fragment.
pub(super) fn fragment_plain_text(fragment: &str) -> Option<String> {
    let document = format!("{PARAGRAPH_PREFIX}{fragment}");
    markdown::to_mdast(&document, &markdown::ParseOptions::gfm())
        .ok()?
        .children()
        .map(|children| strip_paragraph_prefix(&plain_title_text(children)))
}

/// Remove the prefix that pinned the suffix to inline parsing.
fn strip_paragraph_prefix(text: &str) -> String {
    text.strip_prefix(PARAGRAPH_PREFIX)
        .map_or_else(|| text.to_owned(), |rest| rest.trim_start().to_owned())
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
    } else if renders_word_break(node) {
        // A leaf without text still separates the words around it when it
        // renders a break, such as an image, a line break, or a `<br>` tag. An
        // inline HTML tag that renders nothing must not invent a boundary a
        // reader never sees: `un<span></span>done` still renders `undone`.
        text.push(' ');
    }
}

/// Report whether a textless leaf node separates the words around it.
fn renders_word_break(node: &Node) -> bool {
    match node {
        Node::Html(html) => html_renders_line_break(&html.value),
        _ => true,
    }
}

/// Report whether an inline HTML tag renders a line break.
///
/// Only break elements such as `<br>`, `<br/>`, and `<br class="gap">` render a
/// visible break between words. Other inline tags, including closing tags and
/// empty containers, render nothing and so join the surrounding text.
fn html_renders_line_break(value: &str) -> bool {
    let Some(tag) = value.trim().strip_prefix('<') else {
        return false;
    };
    let name = tag
        .trim_start_matches('/')
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .collect::<String>();
    name.eq_ignore_ascii_case("br")
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
