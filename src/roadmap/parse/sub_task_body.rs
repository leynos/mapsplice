//! Sub-task body parsing for roadmap Markdown.

use markdown::mdast::Node;

use super::{looks_like_sub_task_list, looks_like_task_list};
use crate::{
    error::{MapspliceError, Result},
    roadmap::model::MarkdownNodes,
};

/// Parse sub-task body blocks while rejecting deeper roadmap numbering.
pub(super) fn parse_sub_task_body(children: &[Node], source_text: &str) -> Result<MarkdownNodes> {
    let mut body = MarkdownNodes::new();
    for child in children {
        if is_nested_roadmap_list(child) {
            return Err(MapspliceError::InvalidRoadmap {
                message: "sub-tasks cannot contain nested roadmap lists".to_owned(),
            });
        }
        body.push_preserved(child.clone(), source_text);
    }
    Ok(body)
}

fn is_nested_roadmap_list(node: &Node) -> bool {
    matches!(
        node,
        Node::List(list) if looks_like_task_list(list) || looks_like_sub_task_list(list)
    )
}
