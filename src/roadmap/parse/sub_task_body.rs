//! Sub-task body parsing for roadmap Markdown.

use markdown::mdast::Node;

use super::{SourceText, locate};
use crate::{
    error::{MapspliceError, Result},
    roadmap::{model::MarkdownNodes, numbered_detail::contains_numbered_checkbox},
};

/// Parse sub-task body blocks while rejecting deeper roadmap numbering.
pub(super) fn parse_sub_task_body(
    children: &[Node],
    source_text: &SourceText,
) -> Result<MarkdownNodes> {
    let mut body = MarkdownNodes::new();
    for child in children {
        if is_nested_roadmap_list(child) {
            return Err(locate(
                MapspliceError::InvalidRoadmap {
                    message: "sub-tasks cannot contain nested roadmap lists".to_owned(),
                    location: None,
                },
                child.position(),
            ));
        }
        body.push_preserved(child.clone(), source_text);
    }
    Ok(body)
}

/// Detect a numbered checkbox anywhere in a list beneath a sub-task.
///
/// The export adapter rejects numbered checkbox details at any depth below a
/// sub-task body, so the parser refuses the same shape here. Using the shared
/// predicate keeps both boundaries in step.
fn is_nested_roadmap_list(node: &Node) -> bool {
    matches!(node, Node::List(list) if contains_numbered_checkbox(list))
}
