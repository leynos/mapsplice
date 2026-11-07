//! Strict mapping from roadmap Markdown blocks to ordered JSON entries.

use markdown::mdast::{List, ListItem, Node};

use super::{
    details::{self, NumberedDetails},
    requires::paragraph_entry_or_clause,
    schema::{ExportDetail, ExportEntry},
};
use crate::{
    error::{MapspliceError, Result},
    roadmap::{
        model::{MarkdownNodes, TaskChild, TaskEntry},
        source_location::source_location,
    },
};

/// Convert phase or trailing content into labelled entries.
pub(super) fn entries(nodes: &MarkdownNodes) -> Result<Vec<ExportEntry>> {
    let mut entries = Vec::new();
    let mut question = None;
    append_region_entries(
        nodes.nodes(),
        &mut RegionEntries {
            entries: &mut entries,
            allow_question: false,
            question: &mut question,
        },
    )?;
    Ok(entries)
}

/// Convert step body content into its first question and remaining entries.
pub(super) fn step_content(nodes: &MarkdownNodes) -> Result<(Option<String>, Vec<ExportEntry>)> {
    let mut entries = Vec::new();
    let mut question = None;
    append_region_entries(
        nodes.nodes(),
        &mut RegionEntries {
            entries: &mut entries,
            allow_question: true,
            question: &mut question,
        },
    )?;
    Ok((question, entries))
}

/// Convert one task and its ordered structural and checkbox details.
pub(super) fn task_content(
    task: &TaskEntry,
    known_anchors: &std::collections::HashSet<String>,
) -> Result<(Vec<ExportEntry>, Vec<ExportDetail>)> {
    let mut content = TaskContentBuilder::for_task(task.number(), known_anchors);
    for child in task.children() {
        match child {
            TaskChild::Body(nodes) => append_task_blocks(nodes.nodes(), &mut content)?,
            TaskChild::SubTask(identity) => {
                let Some(sub_task) = task
                    .sub_tasks()
                    .iter()
                    .find(|item| item.identity == *identity)
                else {
                    return Err(invalid_at(
                        None,
                        format!("task `{}` has a detached sub-task child", task.number()),
                    ));
                };
                content.numbering.accept_existing(sub_task)?;
                let detail = details::structural_detail(sub_task, content.known_anchors)?;
                content.details.push(detail);
            }
        }
    }
    Ok((content.entries, content.details))
}

/// Accumulate task-body entries and checkbox details in source order.
pub(super) struct TaskContentBuilder<'a> {
    /// Ordered prose entries already mapped from task body blocks.
    pub(super) entries: Vec<ExportEntry>,
    /// Ordered checkbox details already mapped from task body blocks.
    pub(super) details: Vec<ExportDetail>,
    /// Known anchors used to validate each item's dependencies.
    pub(super) known_anchors: &'a std::collections::HashSet<String>,
    /// Numbering state shared by structural and mixed-list details.
    pub(super) numbering: NumberedDetails,
}

impl<'a> TaskContentBuilder<'a> {
    /// Start a task-level content builder.
    pub(super) const fn for_task(
        task_number: crate::roadmap::TaskNumber,
        known_anchors: &'a std::collections::HashSet<String>,
    ) -> Self {
        Self {
            entries: Vec::new(),
            details: Vec::new(),
            known_anchors,
            numbering: NumberedDetails::for_task(task_number),
        }
    }

    /// Start a nested-detail builder that cannot accept numbered sub-tasks.
    pub(super) const fn nested(known_anchors: &'a std::collections::HashSet<String>) -> Self {
        Self {
            entries: Vec::new(),
            details: Vec::new(),
            known_anchors,
            numbering: NumberedDetails::nested(),
        }
    }
}

/// Accumulate phase or step entries and the first unlabelled question.
struct RegionEntries<'a> {
    /// Ordered entries mapped from the region's blocks.
    entries: &'a mut Vec<ExportEntry>,
    /// Whether the first unlabelled paragraph becomes the step question.
    allow_question: bool,
    /// Question captured from that paragraph.
    question: &'a mut Option<String>,
}

impl RegionEntries<'_> {
    /// Record a paragraph as the step question or an ordinary entry.
    fn push(&mut self, key: Option<String>, value: String) {
        if is_question_candidate(self.allow_question, self.question.is_some(), key.is_some()) {
            *self.question = Some(value);
        } else {
            self.entries.push(empty_entry(key, value));
        }
    }
}

/// Append phase or step prose, attaching lists to an immediately preceding empty label.
fn append_region_entries(nodes: &[Node], region: &mut RegionEntries<'_>) -> Result<()> {
    for node in nodes {
        match node {
            Node::Paragraph(paragraph) => {
                if let Some((key, value)) = paragraph_entry_or_clause(paragraph)? {
                    region.push(key, value);
                }
            }
            Node::List(list) => append_labelled_list(list, region.entries.as_mut_slice())?,
            Node::ThematicBreak(_) => {}
            other => {
                return Err(invalid_node(
                    other,
                    "unsupported block in phase or step content",
                ));
            }
        }
    }
    Ok(())
}

/// Identify the first unlabelled paragraph in a step body.
const fn is_question_candidate(allow_question: bool, has_question: bool, has_label: bool) -> bool {
    allow_question && !has_question && !has_label
}

/// Append task-body blocks, separating checkbox items into recursive details.
fn append_task_blocks(nodes: &[Node], content: &mut TaskContentBuilder<'_>) -> Result<()> {
    for node in nodes {
        match node {
            Node::Paragraph(paragraph) => {
                if let Some((key, value)) = paragraph_entry_or_clause(paragraph)? {
                    content.entries.push(empty_entry(key, value));
                }
            }
            Node::List(list) => details::append_task_list(list, content)?,
            Node::ThematicBreak(_) => {}
            other => return Err(invalid_node(other, "unsupported block in task body")),
        }
    }
    Ok(())
}

/// Append a following list to the preceding empty labelled entry.
fn append_labelled_list(list: &List, output: &mut [ExportEntry]) -> Result<()> {
    let Some(parent) = output
        .last_mut()
        .filter(|entry| entry.key.is_some() && entry.value.is_empty())
    else {
        return Err(invalid_node(
            &Node::List(list.clone()),
            "a phase or step list must follow an empty labelled paragraph",
        ));
    };
    for item in list_items(list)? {
        if item.checked.is_some() {
            return Err(invalid_node(
                &Node::ListItem(item.clone()),
                "checkbox details are only valid in task bodies",
            ));
        }
        parent.entries.extend(list_item_entries(item)?);
    }
    Ok(())
}

/// Convert one non-checkbox list item and its nested non-checkbox bullets.
///
/// A dependency bullet contributes no entry of its own, but its children are
/// still inspected: supported blocks become entries and unsupported nesting is
/// rejected with its source location, so no content is silently discarded.
/// Prose after the clause is ordinary content and becomes an entry, so
/// `Requires 1.1.1. Success: keep this.` retains its `Success` entry.
pub(super) fn list_item_entries(item: &ListItem) -> Result<Vec<ExportEntry>> {
    let Some(Node::Paragraph(first)) = item.children.first() else {
        return Err(invalid_node(
            &Node::ListItem(item.clone()),
            "list items must begin with a paragraph",
        ));
    };
    let rest = item.children.get(1..).unwrap_or_default();
    match paragraph_entry_or_clause(first)? {
        None => {
            let mut entries = Vec::new();
            append_entry_children(rest, &mut entries)?;
            Ok(entries)
        }
        Some((key, value)) => {
            let mut entry = empty_entry(key, value);
            append_entry_children(rest, &mut entry.entries)?;
            Ok(vec![entry])
        }
    }
}

/// Map nested blocks beneath an entry, rejecting checkboxes that cannot be represented there.
pub(super) fn append_entry_children(nodes: &[Node], entries: &mut Vec<ExportEntry>) -> Result<()> {
    for node in nodes {
        match node {
            Node::Paragraph(paragraph) => {
                if let Some((key, value)) = paragraph_entry_or_clause(paragraph)? {
                    entries.push(empty_entry(key, value));
                }
            }
            Node::List(list) => {
                if list.ordered {
                    return Err(invalid_node(
                        &Node::List(list.clone()),
                        "ordered lists are not supported in task entries",
                    ));
                }
                for item in list_items(list)? {
                    append_non_checkbox_entry(item, entries)?;
                }
            }
            Node::ThematicBreak(_) => {}
            other => return Err(invalid_node(other, "unsupported nested task entry block")),
        }
    }
    Ok(())
}

/// Append one non-checkbox entry or reject a nested checkbox detail.
fn append_non_checkbox_entry(item: &ListItem, entries: &mut Vec<ExportEntry>) -> Result<()> {
    if item.checked.is_some() {
        return Err(invalid_node(
            &Node::ListItem(item.clone()),
            "checkbox detail cannot be nested beneath a non-checkbox entry",
        ));
    }
    entries.extend(list_item_entries(item)?);
    Ok(())
}

/// Convert list children to typed list-item references.
pub(super) fn list_items(list: &List) -> Result<Vec<&ListItem>> {
    if list.ordered {
        return Err(invalid_node(
            &Node::List(list.clone()),
            "ordered lists are not supported in roadmap entries",
        ));
    }
    list.children
        .iter()
        .map(|node| match node {
            Node::ListItem(item) => Ok(item),
            other => Err(invalid_node(other, "list must contain only list items")),
        })
        .collect()
}

/// Construct a plain export entry.
pub(super) const fn empty_entry(key: Option<String>, value: String) -> ExportEntry {
    ExportEntry {
        key,
        value,
        entries: Vec::new(),
    }
}

/// Build a located diagnostic for an unsupported block.
pub(super) fn invalid_node(node: &Node, message: &str) -> MapspliceError {
    invalid_at(node.position(), message.to_owned())
}

/// Build a located invalid-roadmap diagnostic.
pub(super) fn invalid_at(
    position: Option<&markdown::unist::Position>,
    message: String,
) -> MapspliceError {
    MapspliceError::InvalidRoadmap {
        message,
        location: position.map(source_location),
    }
}
