//! Markdown-to-roadmap parsing and validation.

use std::sync::Arc;

mod checklist;
mod document;
mod fragment;
mod headings;
mod nested_task_list;
mod step_accumulator;
mod sub_task_body;
mod sub_task_fragment;
mod task_children;
mod task_item;
#[cfg(test)]
#[path = "tests.rs"]
mod tests;

pub use fragment::parse_fragment;
use headings::{is_phase_heading, is_step_heading, parse_phase_heading, parse_step_heading};
use markdown::{
    ParseOptions,
    mdast::{List, Node, Paragraph, Root, Text},
    to_mdast,
    unist::Position,
};
use sub_task_fragment::parse_sub_task_fragment_list;
use task_item::{looks_like_sub_task_list, looks_like_task_list, parse_sub_task_item_unchecked};

use super::{
    RoadmapAnchor,
    RoadmapDocument,
    RoadmapItemLevel,
    StepNumber,
    SubTaskNumber,
    TaskNumber,
    model::{SourceId, TaskEntry},
    source_preservation::task_item_sources,
};
use crate::{
    error::{MapspliceError, Result},
    roadmap::source_location::source_location,
};

/// Add a source location to a grammar error when the parser has one available.
pub(super) fn locate(error: MapspliceError, position: Option<&Position>) -> MapspliceError {
    let location = position.map(source_location);
    match error {
        MapspliceError::InvalidRoadmap {
            message,
            location: existing,
        } => MapspliceError::InvalidRoadmap {
            message,
            location: existing.or(location),
        },
        MapspliceError::DanglingDependency {
            anchor,
            location: existing,
        } => MapspliceError::DanglingDependency {
            anchor,
            location: existing.or(location),
        },
        other => other,
    }
}

/// Shared handle to the document a parse pass is reading.
///
/// The parser builds one [`super::model::MarkdownNodes`] per title, summary and
/// body. Handing each of them a clone of this handle keeps a single allocation
/// per document, so retaining the source for diagnostics costs a pointer bump
/// rather than another copy of the whole file.
pub(super) type SourceText = Arc<str>;

/// Source identity and text needed while parsing one roadmap item.
#[derive(Clone, Copy)]
pub(super) struct ParseContext<'source> {
    /// Distinguish target content from inserted fragment content.
    source: SourceId,
    /// Retain the original Markdown for source-preserving nodes.
    source_text: &'source SourceText,
}

/// Parse a target roadmap document.
///
/// # Examples
///
/// ```rust
/// # fn main() -> mapsplice::Result<()> {
/// let roadmap = mapsplice::parse_roadmap(
///     "## 1. Initial phase\n\n### 1.1. First step\n\n- [ ] 1.1.1. Ship the first task\n",
/// )?;
///
/// let task = &roadmap.phases[0].steps[0].tasks()[0];
/// assert_eq!(task.number().to_string(), "1.1.1");
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error when Markdown parsing or roadmap grammar validation fails.
#[tracing::instrument(skip_all, fields(bytes = markdown.len()))]
pub fn parse_roadmap(markdown: &str) -> Result<RoadmapDocument> {
    let source_text: SourceText = Arc::from(markdown);
    document::parse_document_root(parse_root(markdown)?, SourceId::Target, &source_text)
}

/// Parse the checklist items in a task list, preserving target source spans.
pub(super) fn parse_task_list(
    list: &List,
    source: SourceId,
    source_text: &SourceText,
) -> Result<Vec<TaskEntry>> {
    let context = ParseContext {
        source,
        source_text,
    };
    if list.ordered {
        return Err(locate(
            MapspliceError::InvalidRoadmap {
                message: "roadmap task lists must be unordered checklist items".to_owned(),
                location: None,
            },
            list.position.as_ref(),
        ));
    }

    let items = list
        .children
        .iter()
        .map(|node| match node {
            Node::ListItem(item) => Ok(item),
            _ => Err(locate(
                MapspliceError::InvalidRoadmap {
                    message: "roadmap lists must contain only list items".to_owned(),
                    location: None,
                },
                node.position(),
            )),
        })
        .collect::<Result<Vec<_>>>()?;
    let sources = if source == SourceId::Target {
        task_item_sources(list, &items, source_text)
    } else {
        vec![None; items.len()]
    };

    items
        .into_iter()
        .zip(sources)
        .map(|(item, original_source)| task_item::parse_task_item(item, context, original_source))
        .collect()
}

/// Reject tasks whose number names a different parent step.
pub(super) fn validate_tasks_belong_to_step(
    step_number: StepNumber,
    tasks: &[TaskEntry],
    list: &List,
) -> Result<()> {
    for (index, task) in tasks.iter().enumerate() {
        if task.number().step_number() != step_number {
            return Err(locate(
                MapspliceError::InvalidRoadmap {
                    message: format!(
                        "task `{}` does not belong to step `{}`",
                        task.number(),
                        step_number
                    ),
                    location: None,
                },
                list.children.get(index).and_then(Node::position),
            ));
        }
    }
    Ok(())
}

/// Extract a task number and summary nodes from its opening paragraph.
fn parse_task_paragraph(paragraph: &Paragraph) -> Result<(TaskNumber, Vec<Node>)> {
    let (anchor, summary) = parse_numbered_paragraph(paragraph, RoadmapItemLevel::Task, "task")?;
    let RoadmapAnchor::Task(number) = anchor else {
        return Err(MapspliceError::InvalidRoadmap {
            message: "expected a task number".to_owned(),
            location: None,
        });
    };
    Ok((number, summary))
}

/// Extract a sub-task number and summary nodes from its opening paragraph.
fn parse_sub_task_paragraph(paragraph: &Paragraph) -> Result<(SubTaskNumber, Vec<Node>)> {
    let (anchor, summary) =
        parse_numbered_paragraph(paragraph, RoadmapItemLevel::SubTask, "sub-task")?;
    let RoadmapAnchor::SubTask(number) = anchor else {
        return Err(MapspliceError::InvalidRoadmap {
            message: "expected a sub-task number".to_owned(),
            location: None,
        });
    };
    Ok((number, summary))
}

/// Replace a paragraph's numbered prefix with its remaining summary text.
fn parse_numbered_paragraph(
    paragraph: &Paragraph,
    level: RoadmapItemLevel,
    item_name: &str,
) -> Result<(RoadmapAnchor, Vec<Node>)> {
    let Some(Node::Text(Text { value, .. })) = paragraph.children.first() else {
        return Err(MapspliceError::InvalidRoadmap {
            message: format!("{item_name} paragraphs must start with plain text"),
            location: None,
        });
    };
    let (anchor, remainder) = split_numbered_prefix(value, level)?;
    let removed = value
        .get(..value.len().saturating_sub(remainder.len()))
        .unwrap_or_default();
    let mut summary = paragraph.children.clone();
    if let Some(Node::Text(text)) = summary.first_mut() {
        text.value = remainder;
        advance_text_start(text, removed);
    }
    Ok((anchor, summary))
}

/// Move a text node's start past a prefix removed from its value.
///
/// The node keeps the span it had before the numbered prefix was replaced by
/// the summary. Dependency diagnostics derive coordinates from offsets inside
/// the value, so the start must move with it or every token would be reported
/// before its real column. Numbered prefixes never contain a line break, so
/// only the column changes.
fn advance_text_start(text: &mut Text, removed: &str) {
    let Some(position) = text.position.as_mut() else {
        return;
    };
    position.start.offset += removed.len();
    position.start.column += removed.chars().count();
}

/// Split one paragraph prefix into a typed anchor and its trimmed remainder.
///
/// Returns an error when the prefix is missing, malformed, or has a different
/// roadmap level than `level`.
pub(super) fn split_numbered_prefix(
    value: &str,
    level: RoadmapItemLevel,
) -> Result<(RoadmapAnchor, String)> {
    let (digits, remainder) =
        value
            .split_once(". ")
            .ok_or_else(|| MapspliceError::InvalidRoadmap {
                message: format!("expected a numbered {level} prefix in `{value}`"),
                location: None,
            })?;
    let anchor = super::parse_anchor(digits)?;
    if anchor.level() != level {
        return Err(MapspliceError::InvalidRoadmap {
            message: format!("expected a {level} prefix in `{value}`"),
            location: None,
        });
    }
    Ok((anchor, remainder.to_owned()))
}

/// Parse GFM Markdown and require an mdast root node.
pub(super) fn parse_root(markdown: &str) -> Result<Root> {
    match to_mdast(markdown, &ParseOptions::gfm()).map_err(|error| MapspliceError::Markdown {
        message: error.to_string(),
    })? {
        Node::Root(root) => Ok(root),
        _ => Err(MapspliceError::InvalidRoadmap {
            message: "markdown parser did not return a root node".to_owned(),
            location: None,
        }),
    }
}
