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
    TaskNumber,
    model::{ItemIdentity, SourceId, TaskEntry},
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
    Ok(TaskEntry {
        identity: ItemIdentity {
            source,
            anchor: RoadmapAnchor::Task(number),
        },
        number,
        checked: item.checked,
        summary: summary.into(),
        body: item
            .children
            .iter()
            .skip(1)
            .cloned()
            .collect::<Vec<_>>()
            .into(),
    })
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
    let Some(Node::ListItem(item)) = list.children.first() else {
        return false;
    };
    let Some(Node::Paragraph(paragraph)) = item.children.first() else {
        return false;
    };
    let Some(Node::Text(text)) = paragraph.children.first() else {
        return false;
    };
    split_numbered_prefix(&text.value, RoadmapItemLevel::Task).is_ok()
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
