//! Markdown-to-roadmap parsing and validation.

mod document;
mod fragment;
mod sub_task_body;
mod sub_task_fragment;
mod task_children;

pub use fragment::parse_fragment;
use markdown::{
    ParseOptions,
    mdast::{Heading, List, ListItem, Node, Paragraph, Root, Text},
    to_mdast,
};
use sub_task_body::parse_sub_task_body;
use sub_task_fragment::parse_sub_task_fragment_list;
use task_children::TaskChildren;

use super::{
    PhaseNumber,
    RoadmapAnchor,
    RoadmapDocument,
    RoadmapItemLevel,
    StepNumber,
    SubTaskNumber,
    TaskNumber,
    model::{ItemIdentity, MarkdownNodes, SourceId, SubTaskEntry, TaskChild, TaskEntry},
};
use crate::error::{MapspliceError, Result};

#[derive(Clone, Copy)]
pub(super) struct ParseContext<'source> {
    source: SourceId,
    source_text: &'source str,
}

/// Parse a target roadmap document.
///
/// # Errors
///
/// Returns an error when Markdown parsing or roadmap grammar validation fails.
#[tracing::instrument(skip_all, fields(bytes = markdown.len()))]
pub fn parse_roadmap(markdown: &str) -> Result<RoadmapDocument> {
    document::parse_document_root(parse_root(markdown)?, SourceId::Target, markdown)
}

pub(super) fn is_phase_heading(heading: &Heading) -> bool {
    heading.depth == 2 && parse_phase_heading(heading).is_ok()
}

pub(super) fn is_step_heading(heading: &Heading) -> bool {
    heading.depth == 3 && parse_step_heading(heading).is_ok()
}

pub(super) fn parse_phase_heading(heading: &Heading) -> Result<(PhaseNumber, Vec<Node>)> {
    let (anchor, title) = strip_heading_prefix(&heading.children, RoadmapItemLevel::Phase)?;
    match anchor {
        RoadmapAnchor::Phase(number) => Ok((number, title)),
        _ => Err(MapspliceError::InvalidRoadmap {
            message: "expected a phase heading".to_owned(),
        }),
    }
}

pub(super) fn parse_step_heading(heading: &Heading) -> Result<(StepNumber, Vec<Node>)> {
    let (anchor, title) = strip_heading_prefix(&heading.children, RoadmapItemLevel::Step)?;
    match anchor {
        RoadmapAnchor::Step(number) => Ok((number, title)),
        _ => Err(MapspliceError::InvalidRoadmap {
            message: "expected a step heading".to_owned(),
        }),
    }
}

fn strip_heading_prefix(
    children: &[Node],
    level: RoadmapItemLevel,
) -> Result<(RoadmapAnchor, Vec<Node>)> {
    let Node::Text(Text { value, .. }) =
        children
            .first()
            .ok_or_else(|| MapspliceError::InvalidRoadmap {
                message: "roadmap headings must start with plain text".to_owned(),
            })?
    else {
        return Err(MapspliceError::InvalidRoadmap {
            message: "roadmap headings must start with plain text".to_owned(),
        });
    };
    let (anchor, remainder) = split_numbered_prefix(value, level)?;
    let mut title = children.to_vec();
    if let Some(Node::Text(text)) = title.first_mut() {
        text.value = remainder;
    }
    Ok((anchor, title))
}

pub(super) fn parse_task_list(
    list: &List,
    source: SourceId,
    source_text: &str,
) -> Result<Vec<TaskEntry>> {
    let context = ParseContext {
        source,
        source_text,
    };
    if list.ordered {
        return Err(MapspliceError::InvalidRoadmap {
            message: "roadmap task lists must be unordered checklist items".to_owned(),
        });
    }

    list.children
        .iter()
        .map(|node| match node {
            Node::ListItem(item) => parse_task_item(item, context),
            _ => Err(MapspliceError::InvalidRoadmap {
                message: "roadmap lists must contain only list items".to_owned(),
            }),
        })
        .collect()
}

fn parse_task_item(item: &ListItem, context: ParseContext<'_>) -> Result<TaskEntry> {
    if item.checked.is_none() {
        return Err(MapspliceError::InvalidRoadmap {
            message: "roadmap task lists must be unordered checklist items".to_owned(),
        });
    }
    let first = item
        .children
        .first()
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: "task list items must start with a paragraph".to_owned(),
        })?;
    let Node::Paragraph(paragraph) = first else {
        return Err(MapspliceError::InvalidRoadmap {
            message: "task list items must start with a paragraph".to_owned(),
        });
    };
    let (number, summary) = parse_task_paragraph(paragraph)?;
    let child_body = item.children.get(1..).unwrap_or(&[]);
    let (body, sub_tasks, children) = split_task_children(child_body, number, context)?;
    Ok(TaskEntry {
        identity: ItemIdentity {
            source: context.source,
            anchor: RoadmapAnchor::Task(number),
        },
        number,
        checked: item.checked,
        summary: MarkdownNodes::from_nodes(summary),
        body,
        sub_tasks,
        children,
    })
}

fn split_task_children(
    children: &[Node],
    parent: TaskNumber,
    context: ParseContext<'_>,
) -> Result<(MarkdownNodes, Vec<SubTaskEntry>, Vec<TaskChild>)> {
    let mut task_children = TaskChildren::new();
    for child in children {
        if let Node::List(list) = child {
            if looks_like_sub_task_list(list) {
                task_children.flush_body();
                parse_sub_task_list(list, parent, context, &mut task_children)?;
                continue;
            }
            if looks_like_task_list(list) {
                return Err(MapspliceError::InvalidRoadmap {
                    message: "nested roadmap task lists must use sub-task numbers".to_owned(),
                });
            }
        }
        task_children
            .body
            .push_preserved(child.clone(), context.source_text);
    }
    task_children.flush_body();
    Ok((
        task_children.body,
        task_children.sub_tasks,
        task_children.ordered,
    ))
}

fn parse_sub_task_list(
    list: &List,
    parent: TaskNumber,
    context: ParseContext<'_>,
    task_children: &mut TaskChildren,
) -> Result<()> {
    if list.ordered {
        return Err(MapspliceError::InvalidRoadmap {
            message: "roadmap sub-task lists must be unordered checklist items".to_owned(),
        });
    }
    let expected_start = task_children.sub_tasks.len().saturating_add(1);
    for (offset, node) in list.children.iter().enumerate() {
        let Node::ListItem(item) = node else {
            return Err(MapspliceError::InvalidRoadmap {
                message: "roadmap sub-task lists must contain only list items".to_owned(),
            });
        };
        let expected_ordinal =
            u32::try_from(expected_start + offset).map_err(|_| MapspliceError::InvalidRoadmap {
                message: "sub-task count exceeds supported numbering range".to_owned(),
            })?;
        let sub_task = parse_sub_task_item(item, parent, expected_ordinal, context)?;
        task_children
            .ordered
            .push(TaskChild::SubTask(sub_task.identity));
        task_children.sub_tasks.push(sub_task);
    }
    Ok(())
}

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

pub(super) fn parse_sub_task_item_unchecked(
    item: &ListItem,
    context: ParseContext<'_>,
) -> Result<SubTaskEntry> {
    if item.checked.is_none() {
        return Err(MapspliceError::InvalidRoadmap {
            message: "roadmap sub-task lists must be unordered checklist items".to_owned(),
        });
    }
    let first = item
        .children
        .first()
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: "sub-task list items must start with a paragraph".to_owned(),
        })?;
    let Node::Paragraph(paragraph) = first else {
        return Err(MapspliceError::InvalidRoadmap {
            message: "sub-task list items must start with a paragraph".to_owned(),
        });
    };
    let (number, summary) = parse_sub_task_paragraph(paragraph)?;
    let child_body = item.children.get(1..).unwrap_or(&[]);
    let body = parse_sub_task_body(child_body, context.source_text)?;
    Ok(SubTaskEntry {
        identity: ItemIdentity {
            source: context.source,
            anchor: RoadmapAnchor::SubTask(number),
        },
        number,
        checked: item.checked,
        summary: MarkdownNodes::from_nodes(summary),
        body,
    })
}

fn validate_sub_task_number(
    parent: TaskNumber,
    expected_ordinal: u32,
    number: SubTaskNumber,
) -> Result<()> {
    if number.task_number() != parent {
        return Err(MapspliceError::InvalidRoadmap {
            message: format!("sub-task `{number}` does not belong to task `{parent}`"),
        });
    }
    if number.sub_task() != expected_ordinal {
        return Err(MapspliceError::InvalidRoadmap {
            message: format!("sub-task `{number}` is not in document order"),
        });
    }
    Ok(())
}

fn parse_task_paragraph(paragraph: &Paragraph) -> Result<(TaskNumber, Vec<Node>)> {
    let Node::Text(Text { value, .. }) =
        paragraph
            .children
            .first()
            .ok_or_else(|| MapspliceError::InvalidRoadmap {
                message: "task paragraphs must start with plain text".to_owned(),
            })?
    else {
        return Err(MapspliceError::InvalidRoadmap {
            message: "task paragraphs must start with plain text".to_owned(),
        });
    };
    let (anchor, remainder) = split_numbered_prefix(value, RoadmapItemLevel::Task)?;
    let RoadmapAnchor::Task(number) = anchor else {
        return Err(MapspliceError::InvalidRoadmap {
            message: "expected a task number".to_owned(),
        });
    };
    let mut summary = paragraph.children.clone();
    if let Some(Node::Text(text)) = summary.first_mut() {
        text.value = remainder;
    }
    Ok((number, summary))
}

fn parse_sub_task_paragraph(paragraph: &Paragraph) -> Result<(SubTaskNumber, Vec<Node>)> {
    let Node::Text(Text { value, .. }) =
        paragraph
            .children
            .first()
            .ok_or_else(|| MapspliceError::InvalidRoadmap {
                message: "sub-task paragraphs must start with plain text".to_owned(),
            })?
    else {
        return Err(MapspliceError::InvalidRoadmap {
            message: "sub-task paragraphs must start with plain text".to_owned(),
        });
    };
    let (anchor, remainder) = split_numbered_prefix(value, RoadmapItemLevel::SubTask)?;
    let RoadmapAnchor::SubTask(number) = anchor else {
        return Err(MapspliceError::InvalidRoadmap {
            message: "expected a sub-task number".to_owned(),
        });
    };
    let mut summary = paragraph.children.clone();
    if let Some(Node::Text(text)) = summary.first_mut() {
        text.value = remainder;
    }
    Ok((number, summary))
}

fn split_numbered_prefix(value: &str, level: RoadmapItemLevel) -> Result<(RoadmapAnchor, String)> {
    let (digits, remainder) =
        value
            .split_once(". ")
            .ok_or_else(|| MapspliceError::InvalidRoadmap {
                message: format!("expected a numbered {level} prefix in `{value}`"),
            })?;
    let anchor = super::parse_anchor(digits)?;
    if anchor.level() != level {
        return Err(MapspliceError::InvalidRoadmap {
            message: format!("expected a {level} prefix in `{value}`"),
        });
    }
    Ok((anchor, remainder.to_owned()))
}

pub(super) fn looks_like_task_list(list: &List) -> bool {
    looks_like_numbered_list(list, RoadmapItemLevel::Task)
}

pub(super) fn looks_like_sub_task_list(list: &List) -> bool {
    looks_like_numbered_list(list, RoadmapItemLevel::SubTask)
}

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

pub(super) fn parse_root(markdown: &str) -> Result<Root> {
    match to_mdast(markdown, &ParseOptions::gfm()).map_err(|error| MapspliceError::Markdown {
        message: error.to_string(),
    })? {
        Node::Root(root) => Ok(root),
        _ => Err(MapspliceError::InvalidRoadmap {
            message: "markdown parser did not return a root node".to_owned(),
        }),
    }
}
