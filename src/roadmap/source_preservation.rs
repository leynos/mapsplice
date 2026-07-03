//! Source-span helpers for preserving unchanged roadmap Markdown.

use markdown::mdast::Node;

/// Copy source for an unchanged node, preserving leading indentation.
///
/// The Markdown parser positions some nodes after indentation that belongs to
/// their rendered block. This helper widens the span back to the start of the
/// source line so preserved nested list blocks keep their original indent.
///
/// # Examples
///
/// ```rust,ignore
/// let source = "  - [ ] 1.1.1. Nested task.\n";
/// let node = parse_first_list_node(source)?;
/// assert_eq!(original_node_source(&node, source), Some(source.to_owned()));
/// ```
pub(crate) fn original_node_source(node: &Node, source: &str) -> Option<String> {
    let position = node.position()?;
    let prefix = source.get(..position.start.offset)?;
    let start = prefix.rfind('\n').map_or(0, |index| index + 1);
    source.get(start..position.end.offset).map(str::to_owned)
}
