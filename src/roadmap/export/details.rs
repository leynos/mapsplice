//! Checkbox detail mapping, including numbered details found in mixed lists.

use markdown::mdast::{List, ListItem, Node, Paragraph};

use super::{
    blocks::{
        TaskContentBuilder,
        empty_entry,
        invalid_at,
        invalid_node,
        list_item_entry,
        list_items,
    },
    requires::{from_checkbox, from_sub_task, is_requires_clause},
    schema::{ExportDetail, ExportEntry, ExportState},
    text::{paragraph_entry, render_text},
};
use crate::{
    error::Result,
    roadmap::{RoadmapAnchor, SubTaskEntry, TaskNumber, model::MarkdownNodes},
};

/// Numbered details expected beneath one task, in source order.
pub(super) struct NumberedDetails {
    /// Task number whose numbered details are being validated, when at root.
    parent_task: Option<TaskNumber>,
    /// Next expected structural sub-task ordinal.
    next_ordinal: u32,
}

impl NumberedDetails {
    /// Begin tracking structural sub-task numbering for one task.
    pub(super) const fn for_task(parent_task: TaskNumber) -> Self {
        Self {
            parent_task: Some(parent_task),
            next_ordinal: 1,
        }
    }

    /// Track nested checkbox details that cannot own numbered sub-tasks.
    pub(super) const fn nested() -> Self {
        Self {
            parent_task: None,
            next_ordinal: 1,
        }
    }

    /// Validate and account for a sub-task already recognized by the parser.
    pub(super) fn accept_existing(&mut self, sub_task: &SubTaskEntry) -> Result<()> {
        self.accept_number(
            sub_task.number,
            sub_task.summary.nodes().first().and_then(Node::position),
        )
    }

    /// Validate a numbered checkbox detail and advance its expected ordinal.
    fn accept_number(
        &mut self,
        number: crate::roadmap::SubTaskNumber,
        position: Option<&markdown::unist::Position>,
    ) -> Result<()> {
        let Some(parent_task) = self.parent_task else {
            return Err(invalid_at(
                position,
                format!("numbered detail `{number}` is not valid beneath a checkbox detail"),
            ));
        };
        if number.task_number() != parent_task {
            return Err(invalid_at(
                position,
                format!("numbered detail `{number}` does not belong to task `{parent_task}`"),
            ));
        }
        if number.sub_task() != self.next_ordinal {
            return Err(invalid_at(
                position,
                format!(
                    "numbered detail `{number}` is out of order; expected ordinal {}",
                    self.next_ordinal
                ),
            ));
        }
        self.next_ordinal = self.next_ordinal.checked_add(1).ok_or_else(|| {
            invalid_at(
                position,
                "sub-task count exceeds supported numbering range".to_owned(),
            )
        })?;
        Ok(())
    }
}

/// Convert an unordered task-body list into entries and checkbox details.
pub(super) fn append_task_list(list: &List, content: &mut TaskContentBuilder<'_>) -> Result<()> {
    if list.ordered {
        return Err(invalid_node(
            &Node::List(list.clone()),
            "ordered lists are not supported in task bodies",
        ));
    }
    for item in list_items(list)? {
        if item.checked.is_some() {
            let detail = checkbox_detail(item, content.known_anchors, &mut content.numbering)?;
            content.details.push(detail);
        } else if let Some(entry) = list_item_entry(item)? {
            content.entries.push(entry);
        }
    }
    Ok(())
}

/// Convert a numbered structural sub-task into an export detail.
pub(super) fn structural_detail(
    sub_task: &SubTaskEntry,
    known_anchors: &std::collections::HashSet<String>,
) -> Result<ExportDetail> {
    let state = checkbox_state(
        sub_task.checked,
        "sub-task",
        &sub_task.number.to_string(),
        sub_task.summary.nodes().first().and_then(Node::position),
    )?;
    let title = render_text(sub_task.summary.nodes())?;
    let (entries, details) = detail_body(&sub_task.body, known_anchors)?;
    Ok(ExportDetail {
        number: Some(sub_task.number.to_string()),
        title,
        state,
        requires: from_sub_task(sub_task, known_anchors)?,
        entries,
        details,
    })
}

/// Convert a checkbox item and its nested children, retaining its optional number.
fn checkbox_detail(
    item: &ListItem,
    known_anchors: &std::collections::HashSet<String>,
    numbering: &mut NumberedDetails,
) -> Result<ExportDetail> {
    let Some(Node::Paragraph(title_paragraph)) = item.children.first() else {
        return Err(invalid_node(
            &Node::ListItem(item.clone()),
            "checkbox details must begin with a paragraph",
        ));
    };
    let (number, title_nodes) = numbered_title(title_paragraph, numbering)?;
    let title = render_text(&title_nodes)?;
    let (entries, details) =
        detail_blocks(item.children.get(1..).unwrap_or_default(), known_anchors)?;
    let state = checkbox_state(item.checked, "detail", &title, item.position.as_ref())?;
    Ok(ExportDetail {
        number,
        title,
        state,
        requires: from_checkbox(item, known_anchors)?,
        entries,
        details,
    })
}

/// Remove and validate an anchor-like prefix from a mixed-list checkbox item.
fn numbered_title(
    paragraph: &Paragraph,
    numbering: &mut NumberedDetails,
) -> Result<(Option<String>, Vec<Node>)> {
    let mut title_nodes = paragraph.children.clone();
    let Some(Node::Text(text)) = title_nodes.first_mut() else {
        return Ok((None, title_nodes));
    };
    let Some((prefix, remainder)) = text.value.split_once(". ") else {
        return Ok((None, title_nodes));
    };
    if !is_anchor_like(prefix) {
        return Ok((None, title_nodes));
    }
    let position = paragraph.position.as_ref();
    let anchor = super::super::parse_anchor(prefix).map_err(|_| {
        invalid_at(
            position,
            format!("malformed numbered checkbox detail prefix `{prefix}`"),
        )
    })?;
    let RoadmapAnchor::SubTask(number) = anchor else {
        return Err(invalid_at(
            position,
            format!("numbered checkbox detail prefix `{prefix}` must name a sub-task"),
        ));
    };
    numbering.accept_number(number, position)?;
    text.value = remainder.to_owned();
    Ok((Some(number.to_string()), title_nodes))
}

/// Return whether a prefix is composed only of dotted numeric anchor characters.
fn is_anchor_like(prefix: &str) -> bool {
    prefix.contains('.')
        && prefix
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.')
}

/// Convert nested task-body content owned by a structural sub-task.
fn detail_body(
    body: &MarkdownNodes,
    known_anchors: &std::collections::HashSet<String>,
) -> Result<(Vec<ExportEntry>, Vec<ExportDetail>)> {
    detail_blocks(body.nodes(), known_anchors)
}

/// Convert detail content while retaining nested checkbox source order.
fn detail_blocks(
    nodes: &[Node],
    known_anchors: &std::collections::HashSet<String>,
) -> Result<(Vec<ExportEntry>, Vec<ExportDetail>)> {
    let mut content = TaskContentBuilder::nested(known_anchors);
    for node in nodes {
        match node {
            Node::Paragraph(paragraph) if is_requires_clause(&paragraph.children)? => {}
            Node::Paragraph(paragraph) => {
                let (key, value) = paragraph_entry(paragraph)?;
                content.entries.push(empty_entry(key, value));
            }
            Node::List(list) => append_task_list(list, &mut content)?,
            Node::ThematicBreak(_) => {}
            other => return Err(invalid_node(other, "unsupported block in checkbox detail")),
        }
    }
    Ok((content.entries, content.details))
}

/// Convert a parsed checkbox marker to the stable JSON state vocabulary.
fn checkbox_state(
    checked: Option<bool>,
    kind: &str,
    item: &str,
    position: Option<&markdown::unist::Position>,
) -> Result<ExportState> {
    checked.map_or_else(
        || {
            Err(invalid_at(
                position,
                format!("{kind} `{item}` has no checkbox state"),
            ))
        },
        |is_done| {
            Ok(if is_done {
                ExportState::Done
            } else {
                ExportState::Open
            })
        },
    )
}
