//! Inline Markdown rendering and mdast names for diagnostics.

use markdown::mdast::{Link, Node};

use super::text::escape_markdown;
use crate::{
    error::{MapspliceError, Result},
    roadmap::source_location::source_location,
};

/// Render inline Markdown nodes into a single string.
pub(super) fn render_inline(nodes: &[Node]) -> Result<String> {
    nodes
        .iter()
        .map(|node| render_inline_node(node, false))
        .collect::<Result<Vec<_>>>()
        .map(|parts| parts.concat())
}

/// Render inline nodes for export, replacing hard breaks with whitespace.
pub(crate) fn render_inline_export(nodes: &[Node]) -> Result<String> {
    nodes
        .iter()
        .map(|node| render_inline_node(node, true))
        .collect::<Result<Vec<_>>>()
        .map(|parts| parts.concat())
}

/// Render one supported inline Markdown node.
fn render_inline_node(node: &Node, break_as_space: bool) -> Result<String> {
    match node {
        Node::Text(text) => Ok(escape_markdown(&text.value)),
        Node::Emphasis(emphasis) => render_wrapped("*", &emphasis.children, break_as_space),
        Node::Strong(strong) => render_wrapped("**", &strong.children, break_as_space),
        Node::Delete(delete) => render_wrapped("~~", &delete.children, break_as_space),
        Node::InlineCode(code) => Ok(format!("`{}`", code.value)),
        Node::Break(_) if break_as_space => Ok(" ".to_owned()),
        Node::Break(_) => Ok("\\\n".to_owned()),
        Node::Link(link) => render_link(link, break_as_space),
        other => Err(unsupported_inline_node(other)),
    }
}

/// Render children between matching delimiters, such as the `**` of strong emphasis.
fn render_wrapped(marker: &str, children: &[Node], break_as_space: bool) -> Result<String> {
    let inner = render_inline_children(children, break_as_space)?;
    Ok(format!("{marker}{inner}{marker}"))
}

/// Render an inline link with its optional quoted title.
fn render_link(link: &Link, break_as_space: bool) -> Result<String> {
    let Link {
        children,
        url,
        title,
        ..
    } = link;
    let suffix = title
        .as_ref()
        .map_or_else(String::new, |link_title| format!(" \"{link_title}\""));
    let label = render_inline_children(children, break_as_space)?;
    Ok(format!("[{label}]({url}{suffix})"))
}

/// Build the located diagnostic for an inline node the renderer does not support.
fn unsupported_inline_node(node: &Node) -> MapspliceError {
    MapspliceError::InvalidRoadmap {
        message: format!(
            "unsupported inline node `{}` in rendered roadmap",
            node_name(node)
        ),
        location: node.position().map(source_location),
    }
}

/// Render the children of a supported inline container with the same break policy.
fn render_inline_children(nodes: &[Node], break_as_space: bool) -> Result<String> {
    nodes
        .iter()
        .map(|node| render_inline_node(node, break_as_space))
        .collect::<Result<Vec<_>>>()
        .map(|parts| parts.concat())
}

/// Return a stable mdast node name for renderer diagnostics.
pub(super) const fn node_name(node: &Node) -> &'static str {
    match node {
        Node::Root(_) => "root",
        Node::Blockquote(_) => "blockquote",
        Node::FootnoteDefinition(_) => "footnoteDefinition",
        Node::MdxJsxFlowElement(_) => "mdxJsxFlowElement",
        Node::List(_) => "list",
        Node::MdxjsEsm(_) => "mdxjsEsm",
        Node::Toml(_) => "toml",
        Node::Yaml(_) => "yaml",
        Node::Break(_) => "break",
        Node::InlineCode(_) => "inlineCode",
        Node::InlineMath(_) => "inlineMath",
        Node::Delete(_) => "delete",
        Node::Emphasis(_) => "emphasis",
        Node::MdxTextExpression(_) => "mdxTextExpression",
        Node::FootnoteReference(_) => "footnoteReference",
        Node::Html(_) => "html",
        Node::Image(_) => "image",
        Node::ImageReference(_) => "imageReference",
        Node::MdxJsxTextElement(_) => "mdxJsxTextElement",
        Node::Link(_) => "link",
        Node::LinkReference(_) => "linkReference",
        Node::Strong(_) => "strong",
        Node::Text(_) => "text",
        Node::Code(_) => "code",
        Node::Math(_) => "math",
        Node::MdxFlowExpression(_) => "mdxFlowExpression",
        Node::Heading(_) => "heading",
        Node::Table(_) => "table",
        Node::ThematicBreak(_) => "thematicBreak",
        Node::TableRow(_) => "tableRow",
        Node::TableCell(_) => "tableCell",
        Node::ListItem(_) => "listItem",
        Node::Definition(_) => "definition",
        Node::Paragraph(_) => "paragraph",
    }
}
