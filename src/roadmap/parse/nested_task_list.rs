//! Per-item parsing for nested task lists that mix body bullets and addenda.

use markdown::{
    mdast::{List, ListItem, Node},
    unist::Position,
};

use super::{
    ParseContext,
    RoadmapItemLevel,
    TaskNumber,
    locate,
    task_children::TaskChildren,
    task_item::{looks_like_numbered_item, parse_sub_task_item},
};
use crate::error::{MapspliceError, Result};

/// Parse a nested checklist and enforce consecutive sub-task ordinals.
pub(super) fn parse_sub_task_list(
    list: &List,
    parent: TaskNumber,
    context: ParseContext<'_>,
    task_children: &mut TaskChildren,
) -> Result<()> {
    if list.ordered {
        return Err(locate(
            MapspliceError::InvalidRoadmap {
                message: "roadmap sub-task lists must be unordered checklist items".to_owned(),
                location: None,
            },
            list.position.as_ref(),
        ));
    }
    let mut body_run = Vec::new();
    for node in &list.children {
        let Node::ListItem(item) = node else {
            return Err(locate(
                MapspliceError::InvalidRoadmap {
                    message: "roadmap sub-task lists must contain only list items".to_owned(),
                    location: None,
                },
                node.position(),
            ));
        };
        if looks_like_numbered_item(item, RoadmapItemLevel::SubTask) {
            push_body_list_run(list, &mut body_run, context.source_text, task_children);
            let expected_ordinal = task_children.next_sub_task_ordinal()?;
            let sub_task = parse_sub_task_item(item, parent, expected_ordinal, context)
                .map_err(|error| locate(error, item.position.as_ref()))?;
            task_children.push_sub_task(sub_task);
        } else {
            body_run.push(item.clone());
        }
    }
    push_body_list_run(list, &mut body_run, context.source_text, task_children);
    Ok(())
}

/// Preserve one consecutive run of ordinary list items as a body block.
fn push_body_list_run(
    list: &List,
    items: &mut Vec<ListItem>,
    source_text: &str,
    task_children: &mut TaskChildren,
) {
    if items.is_empty() {
        return;
    }
    let position = items
        .first()
        .and_then(|item| item.position.as_ref())
        .zip(items.last().and_then(|item| item.position.as_ref()))
        .map(|(first, last)| Position {
            start: first.start.clone(),
            end: last.end.clone(),
        });
    let mut body_list = list.clone();
    body_list.children = std::mem::take(items)
        .into_iter()
        .map(Node::ListItem)
        .collect();
    body_list.position = position;
    task_children.push_body_node(Node::List(body_list), source_text);
}
