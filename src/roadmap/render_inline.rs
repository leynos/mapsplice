//! Inline Markdown rendering and mdast names for diagnostics.

use markdown::mdast::{Link, Node};

use super::text::escape_markdown;
use crate::error::{MapspliceError, Result};

/// Render inline Markdown nodes into a single string.
pub(super) fn render_inline(nodes: &[Node]) -> Result<String> {
    nodes
        .iter()
        .map(render_inline_node)
        .collect::<Result<Vec<_>>>()
        .map(|parts| parts.concat())
}

/// Render one supported inline Markdown node.
fn render_inline_node(node: &Node) -> Result<String> {
    match node {
        Node::Text(text) => Ok(escape_markdown(&text.value)),
        Node::Emphasis(emphasis) => Ok(format!("*{}*", render_inline(&emphasis.children)?)),
        Node::Strong(strong) => Ok(format!("**{}**", render_inline(&strong.children)?)),
        Node::Delete(delete) => Ok(format!("~~{}~~", render_inline(&delete.children)?)),
        Node::InlineCode(code) => Ok(format!("`{}`", code.value)),
        Node::Break(_) => Ok("\\\n".to_owned()),
        Node::Link(Link {
            children,
            url,
            title,
            ..
        }) => {
            let suffix = title
                .as_ref()
                .map_or_else(String::new, |link_title| format!(" \"{link_title}\""));
            Ok(format!("[{}]({url}{suffix})", render_inline(children)?))
        }
        other => Err(MapspliceError::InvalidRoadmap {
            message: format!(
                "unsupported inline node `{}` in rendered roadmap",
                node_name(other)
            ),
        }),
    }
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
