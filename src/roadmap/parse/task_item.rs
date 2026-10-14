//! Parsing numbered task and sub-task list items.

use markdown::mdast::{List, ListItem, Node};

use super::{
    ParseContext,
    RoadmapAnchor,
    RoadmapItemLevel,
    SubTaskNumber,
    TaskNumber,
    checklist::{ChecklistKind, parse_checklist_item_head},
    locate,
    parse_sub_task_paragraph,
    parse_task_paragraph,
    split_numbered_prefix,
    sub_task_body::parse_sub_task_body,
    task_children::TaskChildren,
};
use crate::{
    error::{MapspliceError, Result},
    roadmap::{
        model::{
            ItemIdentity,
            MarkdownNodes,
            SourceId,
            SubTaskEntry,
            TaskChild,
            TaskEntry,
            TaskEntryParts,
        },
        source_preservation::original_position_source,
    },
};

/// Parse one task list item, retaining target source for unchanged rendering.
///
/// Returns an error when the item is not a valid task or its descendants have
/// invalid roadmap structure.
pub(super) fn parse_task_item(
    item: &ListItem,
    context: ParseContext<'_>,
    original_source: Option<String>,
) -> Result<TaskEntry> {
    let head = parse_checklist_item_head(item, ChecklistKind::Task)
        .map_err(|error| locate(error, item.position.as_ref()))?;
    let (number, summary) = parse_task_paragraph(head.paragraph)
        .map_err(|error| locate(error, head.paragraph.position.as_ref()))?;
    let (body, sub_tasks, children) = split_task_children(head.child_body, number, context)?;
    TaskEntry::from_parts(TaskEntryParts {
        identity: ItemIdentity {
            source: context.source,
            anchor: RoadmapAnchor::Task(number),
        },
        number,
        checked: head.checked,
        summary: MarkdownNodes::from_nodes(summary),
        body,
        original_source,
        sub_tasks,
        children,
    })
}

/// Separate body Markdown and numbered sub-tasks in their original order.
fn split_task_children(
    children: &[Node],
    parent: TaskNumber,
    context: ParseContext<'_>,
) -> Result<(MarkdownNodes, Vec<SubTaskEntry>, Vec<TaskChild>)> {
    let mut task_children = TaskChildren::new();
    for child in children {
        if let Node::List(list) = child {
            if looks_like_sub_task_list(list) {
                parse_sub_task_list(list, parent, context, &mut task_children)?;
                continue;
            }
            if looks_like_task_list(list) {
                return Err(locate(
                    MapspliceError::InvalidRoadmap {
                        message: "nested roadmap task lists must use sub-task numbers".to_owned(),
                        location: None,
                    },
                    list.position.as_ref(),
                ));
            }
        }
        task_children.push_body_node(child.clone(), context.source_text);
    }
    Ok(task_children.finish())
}

/// Parse a nested checklist and enforce consecutive sub-task ordinals.
fn parse_sub_task_list(
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
    let expected_start = task_children.next_sub_task_ordinal()?;
    for (offset, node) in list.children.iter().enumerate() {
        let Node::ListItem(item) = node else {
            return Err(locate(
                MapspliceError::InvalidRoadmap {
                    message: "roadmap sub-task lists must contain only list items".to_owned(),
                    location: None,
                },
                node.position(),
            ));
        };
        let ordinal_offset = u32::try_from(offset).map_err(|_| MapspliceError::InvalidRoadmap {
            message: "sub-task count exceeds supported numbering range".to_owned(),
            location: None,
        })?;
        let expected_ordinal = expected_start.checked_add(ordinal_offset).ok_or_else(|| {
            MapspliceError::InvalidRoadmap {
                message: "sub-task count exceeds supported numbering range".to_owned(),
                location: None,
            }
        })?;
        let sub_task = parse_sub_task_item(item, parent, expected_ordinal, context)
            .map_err(|error| locate(error, item.position.as_ref()))?;
        task_children.push_sub_task(sub_task);
    }
    Ok(())
}

/// Parse and validate one structural sub-task list item.
///
/// Returns an error when the checklist shape or expected ordinal is invalid.
fn parse_sub_task_item(
    item: &ListItem,
    parent: TaskNumber,
    expected_ordinal: u32,
    context: ParseContext<'_>,
) -> Result<SubTaskEntry> {
    let sub_task = parse_sub_task_item_unchecked(item, context)?;
    validate_sub_task_number(parent, expected_ordinal, sub_task.number)?;
    Ok(sub_task)
}

/// Parse a sub-task item without checking its ordinal against its parent.
///
/// Returns an error when the checklist shape or Markdown body is invalid.
pub(super) fn parse_sub_task_item_unchecked(
    item: &ListItem,
    context: ParseContext<'_>,
) -> Result<SubTaskEntry> {
    let head = parse_checklist_item_head(item, ChecklistKind::SubTask)
        .map_err(|error| locate(error, item.position.as_ref()))?;
    let (number, summary) = parse_sub_task_paragraph(head.paragraph)
        .map_err(|error| locate(error, head.paragraph.position.as_ref()))?;
    let body = parse_sub_task_body(head.child_body, context.source_text)?;
    Ok(SubTaskEntry {
        identity: ItemIdentity {
            source: context.source,
            anchor: RoadmapAnchor::SubTask(number),
        },
        number,
        checked: head.checked,
        summary: MarkdownNodes::from_nodes(summary),
        body,
        original_source: original_item_source(item, context),
    })
}

/// Capture exact target source for a list item, if its parser span is present.
///
/// Fragment items intentionally return `None` because their source is new and
/// must use canonical rendering.
fn original_item_source(item: &ListItem, context: ParseContext<'_>) -> Option<String> {
    (context.source == SourceId::Target)
        .then(|| {
            item.position
                .as_ref()
                .and_then(|position| original_position_source(position, context.source_text))
        })
        .flatten()
}

/// Check a sub-task's parent and expected position in document order.
fn validate_sub_task_number(
    parent: TaskNumber,
    expected_ordinal: u32,
    number: SubTaskNumber,
) -> Result<()> {
    if number.task_number() != parent {
        return Err(MapspliceError::InvalidRoadmap {
            message: format!("sub-task `{number}` does not belong to task `{parent}`"),
            location: None,
        });
    }
    if number.sub_task() != expected_ordinal {
        return Err(MapspliceError::InvalidRoadmap {
            message: format!("sub-task `{number}` is not in document order"),
            location: None,
        });
    }
    Ok(())
}

/// Detect whether a list starts with a numbered task item.
pub(super) fn looks_like_task_list(list: &List) -> bool {
    looks_like_numbered_list(list, RoadmapItemLevel::Task)
}

/// Detect whether a list starts with a numbered sub-task item.
pub(super) fn looks_like_sub_task_list(list: &List) -> bool {
    looks_like_numbered_list(list, RoadmapItemLevel::SubTask)
}

/// Match the first list item's prefix against a roadmap level.
fn looks_like_numbered_list(list: &List, level: RoadmapItemLevel) -> bool {
    let Some(Node::ListItem(item)) = list.children.first() else {
        return false;
    };
    let Some(Node::Paragraph(paragraph)) = item.children.first() else {
        return false;
    };
    let Some(Node::Text(text)) = paragraph.children.first() else {
        return false;
    };
    split_numbered_prefix(&text.value, level).is_ok()
}
