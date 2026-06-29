//! Markdown-to-roadmap parsing and validation.

mod document;
mod fragment;

pub use fragment::parse_fragment;
use markdown::{
    ParseOptions,
    mdast::{Heading, List, ListItem, Node, Paragraph, Root, Text},
    to_mdast,
};

use super::{
    PhaseNumber,
    RoadmapAnchor,
    RoadmapDocument,
    RoadmapItemLevel,
    StepNumber,
    SubTaskNumber,
    TaskNumber,
    model::{ItemIdentity, MarkdownNodes, SourceId, SubTaskEntry, TaskEntry},
};
use crate::error::{MapspliceError, Result};

/// Parse a target roadmap document.
///
/// # Errors
///
/// Returns an error when the Markdown cannot be parsed or when the document
/// does not match the supported roadmap structure.
#[tracing::instrument(skip_all, fields(bytes = markdown.len()))]
pub fn parse_roadmap(markdown: &str) -> Result<RoadmapDocument> {
    let root = parse_root(markdown)?;
    document::parse_document_root(root, SourceId::Target)
}

/// Return whether a heading is a supported phase heading.
pub(super) fn is_phase_heading(heading: &Heading) -> bool {
    heading.depth == 2 && parse_phase_heading(heading).is_ok()
}

/// Return whether a heading is a supported step heading.
pub(super) fn is_step_heading(heading: &Heading) -> bool {
    heading.depth == 3 && parse_step_heading(heading).is_ok()
}

/// Parse a phase heading into its number and title nodes.
pub(super) fn parse_phase_heading(heading: &Heading) -> Result<(PhaseNumber, Vec<Node>)> {
    let (anchor, title) = strip_heading_prefix(&heading.children, RoadmapItemLevel::Phase)?;
    match anchor {
        RoadmapAnchor::Phase(number) => Ok((number, title)),
        _ => Err(MapspliceError::InvalidRoadmap {
            message: "expected a phase heading".to_owned(),
        }),
    }
}

/// Parse a step heading into its number and title nodes.
pub(super) fn parse_step_heading(heading: &Heading) -> Result<(StepNumber, Vec<Node>)> {
    let (anchor, title) = strip_heading_prefix(&heading.children, RoadmapItemLevel::Step)?;
    match anchor {
        RoadmapAnchor::Step(number) => Ok((number, title)),
        _ => Err(MapspliceError::InvalidRoadmap {
            message: "expected a step heading".to_owned(),
        }),
    }
}

/// Split a roadmap number prefix from heading inline nodes.
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

/// Parse an unordered checklist into roadmap task entries.
pub(super) fn parse_task_list(list: &List, source: SourceId) -> Result<Vec<TaskEntry>> {
    if list.ordered {
        return Err(MapspliceError::InvalidRoadmap {
            message: "roadmap task lists must be unordered checklist items".to_owned(),
        });
    }

    list.children
        .iter()
        .map(|node| match node {
            Node::ListItem(item) => parse_task_item(item, source),
            _ => Err(MapspliceError::InvalidRoadmap {
                message: "roadmap lists must contain only list items".to_owned(),
            }),
        })
        .collect()
}

/// Parse one checklist list item into a task entry.
fn parse_task_item(item: &ListItem, source: SourceId) -> Result<TaskEntry> {
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
    let (body, sub_tasks) = split_task_children(child_body, number, source)?;
    Ok(TaskEntry {
        identity: ItemIdentity {
            source,
            anchor: RoadmapAnchor::Task(number),
        },
        number,
        checked: item.checked,
        summary: MarkdownNodes::from_nodes(summary),
        body,
        sub_tasks,
    })
}

/// Split nested task content into body blocks and first-class sub-tasks.
fn split_task_children(
    children: &[Node],
    parent: TaskNumber,
    source: SourceId,
) -> Result<(MarkdownNodes, Vec<SubTaskEntry>)> {
    let mut body = MarkdownNodes::new();
    let mut sub_tasks = Vec::new();
    for child in children {
        if let Node::List(list) = child {
            if looks_like_sub_task_list(list) {
                parse_sub_task_list(list, parent, source, &mut sub_tasks)?;
                continue;
            }
            if looks_like_task_list(list) {
                return Err(MapspliceError::InvalidRoadmap {
                    message: "nested roadmap task lists must use sub-task numbers".to_owned(),
                });
            }
        }
        body.push(child.clone());
    }
    Ok((body, sub_tasks))
}

/// Parse one nested checklist list into ordered sub-tasks.
fn parse_sub_task_list(
    list: &List,
    parent: TaskNumber,
    source: SourceId,
    sub_tasks: &mut Vec<SubTaskEntry>,
) -> Result<()> {
    if list.ordered {
        return Err(MapspliceError::InvalidRoadmap {
            message: "roadmap sub-task lists must be unordered checklist items".to_owned(),
        });
    }
    let expected_start = sub_tasks.len().saturating_add(1);
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
        sub_tasks.push(parse_sub_task_item(item, parent, expected_ordinal, source)?);
    }
    Ok(())
}

/// Parse one nested checklist list item into a sub-task entry.
fn parse_sub_task_item(
    item: &ListItem,
    parent: TaskNumber,
    expected_ordinal: u32,
    source: SourceId,
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
    validate_sub_task_number(parent, expected_ordinal, number)?;
    let child_body = item.children.get(1..).unwrap_or(&[]);
    let body = parse_sub_task_body(child_body)?;
    Ok(SubTaskEntry {
        identity: ItemIdentity {
            source,
            anchor: RoadmapAnchor::SubTask(number),
        },
        number,
        checked: item.checked,
        summary: MarkdownNodes::from_nodes(summary),
        body,
    })
}

/// Parse sub-task body blocks while rejecting deeper roadmap numbering.
fn parse_sub_task_body(children: &[Node]) -> Result<MarkdownNodes> {
    let mut body = MarkdownNodes::new();
    for child in children {
        if let Node::List(list) = child
            && looks_like_sub_task_list(list)
        {
            return Err(MapspliceError::InvalidRoadmap {
                message: "sub-tasks cannot contain nested roadmap sub-tasks".to_owned(),
            });
        }
        body.push(child.clone());
    }
    Ok(body)
}

/// Validate that a sub-task belongs to its parent and appears in order.
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

/// Parse the numbered prefix and summary from a task paragraph.
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

/// Parse the numbered prefix and summary from a sub-task paragraph.
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

/// Split and validate a numbered roadmap prefix from plain text.
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

/// Return whether a list begins with a roadmap task number.
pub(super) fn looks_like_task_list(list: &List) -> bool {
    looks_like_numbered_list(list, RoadmapItemLevel::Task)
}

/// Return whether a list begins with a roadmap sub-task number.
pub(super) fn looks_like_sub_task_list(list: &List) -> bool {
    looks_like_numbered_list(list, RoadmapItemLevel::SubTask)
}

/// Return whether a list begins with the requested roadmap item level.
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

/// Parse Markdown into an mdast root node.
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
