//! Inline Markdown rendering and mdast names for diagnostics.

use markdown::mdast::{Link, Node};

use super::text::escape_markdown;
use crate::{
    error::{MapspliceError, Result},
    roadmap::source_location::source_location,
};

/// Rendering context that decides how plain text and breaks are emitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InlineMode {
    /// Re-render canonical Markdown for a rewritten roadmap.
    Markdown,
    /// Emit literal content for the machine-readable export.
    Export,
}

/// Render inline Markdown nodes into a single canonical Markdown string.
pub(super) fn render_inline(nodes: &[Node]) -> Result<String> {
    render_inline_nodes(nodes, InlineMode::Markdown)
}

/// Render inline nodes for export, replacing hard breaks with whitespace.
///
/// Plain text is emitted literally. The export is machine-readable data, not
/// Markdown source, so it must not carry the backslash escapes that
/// [`render_inline`] needs to keep a rewritten roadmap reparsable.
pub(crate) fn render_inline_export(nodes: &[Node]) -> Result<String> {
    render_inline_nodes(nodes, InlineMode::Export)
}

/// Render a sequence of inline nodes in one mode.
fn render_inline_nodes(nodes: &[Node], mode: InlineMode) -> Result<String> {
    nodes
        .iter()
        .map(|node| render_inline_node(node, mode))
        .collect::<Result<Vec<_>>>()
        .map(|parts| parts.concat())
}

/// Render one supported inline Markdown node.
fn render_inline_node(node: &Node, mode: InlineMode) -> Result<String> {
    match node {
        Node::Text(text) => Ok(match mode {
            InlineMode::Markdown => escape_markdown(&text.value),
            InlineMode::Export => text.value.clone(),
        }),
        Node::Emphasis(emphasis) => render_wrapped("*", &emphasis.children, mode),
        Node::Strong(strong) => render_wrapped("**", &strong.children, mode),
        Node::Delete(delete) => render_wrapped("~~", &delete.children, mode),
        Node::InlineCode(code) => Ok(format!("`{}`", code.value)),
        Node::Break(_) if mode == InlineMode::Export => Ok(" ".to_owned()),
        Node::Break(_) => Ok("\\\n".to_owned()),
        Node::Link(link) => render_link(link, mode),
        other => Err(unsupported_inline_node(other)),
    }
}

/// Render children between matching delimiters, such as the `**` of strong emphasis.
fn render_wrapped(marker: &str, children: &[Node], mode: InlineMode) -> Result<String> {
    let inner = render_inline_children(children, mode)?;
    Ok(format!("{marker}{inner}{marker}"))
}

/// Render an inline link with its optional quoted title.
fn render_link(link: &Link, mode: InlineMode) -> Result<String> {
    let Link {
        children,
        url,
        title,
        ..
    } = link;
    let suffix = title
        .as_ref()
        .map_or_else(String::new, |link_title| format!(" \"{link_title}\""));
    let label = render_inline_children(children, mode)?;
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

/// Render the children of a supported inline container in the same mode.
fn render_inline_children(nodes: &[Node], mode: InlineMode) -> Result<String> {
    render_inline_nodes(nodes, mode)
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
