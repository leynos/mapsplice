//! Task and sub-task rendering with source-preservation fallbacks.

use markdown::mdast::{List, Node};

use super::{
    checkbox_marker,
    preservation::render_preserved_task_or_canonical,
    render_inline,
    render_item_summary,
    render_nested_body,
    trim_preserved_task_source,
    validate_sub_task_for_render,
    validate_task_for_render,
};
use crate::{
    error::{MapspliceError, Result},
    roadmap::{
        model::{ItemIdentity, MarkdownNodes, SubTaskEntry, TaskChild, TaskEntry},
        preservation_events::PreservationReport,
    },
};

/// Preserve distinct body lists after a structural child is removed.
const BODY_LIST_BOUNDARY_MARKER: &str = "  <!-- mapsplice: preserve body list boundary -->";

/// Render a task from preserved source when stable, otherwise canonically.
///
/// Returns an error when canonical rendering encounters invalid Markdown or
/// an inconsistent structural child sequence.
pub(super) fn render_task(task: &TaskEntry, report: &mut PreservationReport) -> Result<String> {
    if let Some(original) = task.original_source() {
        validate_task_for_render(task)?;
        let (rendered, outcome) =
            render_preserved_task_or_canonical(original, || render_task_canonical(task, report))?;
        report.record_render_outcome(outcome);
        return Ok(preserve_task_separator(original, rendered));
    }
    render_task_canonical(task, report)
}

/// Restore the complete source boundary after a stable preserved task.
///
/// The list renderer only synthesizes a separator when its preceding task did
/// not supply one. Returning the original suffix therefore retains loose-list
/// blank lines and their line-ending convention byte-for-byte.
fn preserve_task_separator(original: &str, rendered: String) -> String {
    if rendered == trim_preserved_task_source(original) {
        original.to_owned()
    } else {
        rendered
    }
}

/// Restore the separator required after a preserved sub-task's final body node.
fn preserve_sub_task_separator(
    sub_task: &SubTaskEntry,
    original: &str,
    rendered: String,
) -> String {
    if rendered == trim_preserved_task_source(original)
        && sub_task_requires_trailing_separator(sub_task)
    {
        format!("{rendered}\n")
    } else {
        rendered
    }
}

/// Return whether a sub-task's final body node needs a trailing separator.
fn sub_task_requires_trailing_separator(sub_task: &SubTaskEntry) -> bool {
    markdown_requires_trailing_separator(&sub_task.body)
}

/// Return whether Markdown content ends in a block requiring separation.
fn markdown_requires_trailing_separator(markdown: &MarkdownNodes) -> bool {
    let paragraph_count = markdown
        .nodes()
        .iter()
        .filter(|node| matches!(node, Node::Paragraph(_)))
        .count();
    paragraph_count > 1
        || markdown
            .nodes()
            .last()
            .is_some_and(|node| matches!(node, Node::Code(_) | Node::List(_) | Node::Table(_)))
}

/// Render a task canonically, including body nodes and ordered sub-tasks.
///
/// Returns an error when the task's structural child sequence references a
/// missing sub-task.
fn render_task_canonical(task: &TaskEntry, report: &mut PreservationReport) -> Result<String> {
    let mut parts = vec![format!(
        "- {}{}. {}",
        checkbox_marker(task.checked),
        task.number,
        render_item_summary(&render_inline(task.summary.nodes())?, 2)
    )];
    for (index, child) in task.children().iter().enumerate() {
        match child {
            TaskChild::Body(body) => push_body_child(&mut parts, task, index, body)?,
            TaskChild::SubTask(identity) => {
                let sub_task = find_sub_task_for_child(task, *identity)?;
                parts.push(render_sub_task(sub_task, 2, report)?);
            }
        }
    }
    Ok(parts.join("\n"))
}

/// Append one rendered body run with its list marker or paragraph separator.
fn push_body_child(
    parts: &mut Vec<String>,
    task: &TaskEntry,
    index: usize,
    body: &MarkdownNodes,
) -> Result<()> {
    let mut body_blocks = render_body_child(task, index, body)?;
    if needs_body_list_boundary_marker(task, index, body) {
        push_body_list_boundary_marker(parts, &body_blocks);
    }
    let has_trailing_separator = parts.last().is_some_and(String::is_empty);
    if has_trailing_separator && body_blocks.first().is_some_and(String::is_empty) {
        // Adjacent body runs share this boundary after a removed sub-task.
        body_blocks.remove(0);
    }
    if !has_trailing_separator && needs_paragraph_body_separator(body, &body_blocks) {
        // Keep this body paragraph outside the preceding child summary.
        parts.push(String::new());
    }
    parts.extend(body_blocks);
    Ok(())
}

/// Emit the marker that stops `CommonMark` merging adjacent same-kind lists.
fn push_body_list_boundary_marker(parts: &mut Vec<String>, body_blocks: &[String]) {
    if !parts.last().is_some_and(String::is_empty) {
        parts.push(String::new());
    }
    parts.push(BODY_LIST_BOUNDARY_MARKER.to_owned());
    if !body_blocks.first().is_some_and(String::is_empty) {
        parts.push(String::new());
    }
}

/// Keep adjacent same-kind body lists distinct when a child is removed.
fn needs_body_list_boundary_marker(task: &TaskEntry, index: usize, body: &MarkdownNodes) -> bool {
    index
        .checked_sub(1)
        .and_then(|previous_index| task.children().get(previous_index))
        .and_then(|previous_child| match previous_child {
            TaskChild::Body(previous_body) => Some((previous_body, body)),
            TaskChild::SubTask(_) => None,
        })
        .is_some_and(|(previous_body, right_body)| {
            body_runs_need_list_boundary_marker(previous_body, right_body)
        })
}

/// Return whether a body run begins with a separate paragraph block.
fn body_starts_with_paragraph(body: &MarkdownNodes) -> bool {
    matches!(body.nodes().first(), Some(Node::Paragraph(_)))
}

/// Return whether a paragraph-first body run needs a separate boundary.
fn needs_paragraph_body_separator(body: &MarkdownNodes, body_blocks: &[String]) -> bool {
    body_starts_with_paragraph(body) && !body_blocks.first().is_some_and(String::is_empty)
}

/// Render one body run and trim separators at compatible tight-list boundaries.
fn render_body_child(task: &TaskEntry, index: usize, body: &MarkdownNodes) -> Result<Vec<String>> {
    let mut body_blocks = render_nested_body(body, 2)?;
    let previous_child = index
        .checked_sub(1)
        .and_then(|previous_index| task.children().get(previous_index));
    let next_child = task.children().get(index + 1);
    trim_leading_tight_list_separator(
        &mut body_blocks,
        is_tight_interrupting_list(body.nodes().first()),
        joins_previous_child(previous_child, next_child, body),
    );
    trim_trailing_tight_list_separator(
        &mut body_blocks,
        is_tight_list(body.nodes().last()),
        joins_next_child(body, next_child),
    );
    Ok(body_blocks)
}

/// Return whether a Markdown node is a tight list.
const fn is_tight_list(node: Option<&Node>) -> bool {
    matches!(node, Some(Node::List(list)) if !list.spread)
}

/// Return whether a tight list can interrupt the preceding paragraph.
fn is_tight_interrupting_list(node: Option<&Node>) -> bool {
    matches!(node, Some(Node::List(list)) if is_tight_and_interrupting(list))
}

/// Return whether a list is tight and can interrupt its preceding paragraph.
fn is_tight_and_interrupting(list: &List) -> bool {
    !list.spread && list_can_interrupt_paragraph(list)
}

/// Return whether `CommonMark` allows a list to interrupt its preceding paragraph.
fn list_can_interrupt_paragraph(list: &List) -> bool { !list.ordered || list.start == Some(1) }

/// Return whether a body run joins the child before it without a blank.
fn joins_previous_child(
    previous_child: Option<&TaskChild>,
    next_child: Option<&TaskChild>,
    body: &MarkdownNodes,
) -> bool {
    match previous_child {
        Some(TaskChild::SubTask(_)) => true,
        Some(TaskChild::Body(previous_body)) => {
            body_runs_have_managed_list_boundary(previous_body, body)
        }
        None => matches!(next_child, Some(TaskChild::SubTask(_))),
    }
}

/// Return whether a body run joins the child after it without a blank.
fn joins_next_child(body: &MarkdownNodes, next_child: Option<&TaskChild>) -> bool {
    match next_child {
        Some(TaskChild::SubTask(_)) => true,
        Some(TaskChild::Body(next_body)) => body_runs_have_managed_list_boundary(body, next_body),
        None => false,
    }
}

/// Return whether adjacent tight bullet-list runs intentionally join.
fn body_runs_share_tight_bullet_list_boundary(left: &MarkdownNodes, right: &MarkdownNodes) -> bool {
    let (Some(Node::List(left_list)), Some(Node::List(right_list))) =
        (left.nodes().last(), right.nodes().first())
    else {
        return false;
    };
    !left_list.spread && !right_list.spread && !left_list.ordered && !right_list.ordered
}

/// Return whether same-kind lists merge unless a structural boundary separates them.
fn body_runs_need_list_boundary_marker(left: &MarkdownNodes, right: &MarkdownNodes) -> bool {
    let (Some(Node::List(left_list)), Some(Node::List(right_list))) =
        (left.nodes().last(), right.nodes().first())
    else {
        return false;
    };
    left_list.ordered == right_list.ordered
        && !body_runs_share_tight_bullet_list_boundary(left, right)
}

/// Return whether adjacent body lists need a separator or intentionally join.
fn body_runs_have_managed_list_boundary(left: &MarkdownNodes, right: &MarkdownNodes) -> bool {
    body_runs_share_tight_bullet_list_boundary(left, right)
        || body_runs_need_list_boundary_marker(left, right)
}

/// Remove the generated leading blank when a body list joins a task child.
fn trim_leading_tight_list_separator(
    body_blocks: &mut Vec<String>,
    begins_with_tight_list: bool,
    joins_neighbouring_child: bool,
) {
    if should_trim_leading_tight_list_separator(
        body_blocks,
        begins_with_tight_list,
        joins_neighbouring_child,
    ) {
        body_blocks.remove(0);
    }
}

/// Identify a generated blank before a tight list joining a task child.
fn should_trim_leading_tight_list_separator(
    body_blocks: &[String],
    begins_with_tight_list: bool,
    joins_neighbouring_child: bool,
) -> bool {
    let list_joins_child = begins_with_tight_list && joins_neighbouring_child;
    let has_leading_separator = body_blocks.first().is_some_and(String::is_empty);
    list_joins_child && has_leading_separator
}

/// Remove the generated trailing blank when a tight list joins the next child.
fn trim_trailing_tight_list_separator(
    body_blocks: &mut Vec<String>,
    ends_with_tight_list: bool,
    joins_next_child: bool,
) {
    if should_trim_trailing_tight_list_separator(
        body_blocks,
        ends_with_tight_list,
        joins_next_child,
    ) {
        drop(body_blocks.pop());
    }
}

/// Identify a generated blank after a tight list joining the next child.
fn should_trim_trailing_tight_list_separator(
    body_blocks: &[String],
    ends_with_tight_list: bool,
    joins_next_child: bool,
) -> bool {
    let list_joins_child = ends_with_tight_list && joins_next_child;
    let has_trailing_separator = body_blocks.last().is_some_and(String::is_empty);
    list_joins_child && has_trailing_separator
}

/// Resolve a structural child identity to its corresponding sub-task.
///
/// Returns [`MapspliceError::InvalidRoadmap`] when the child sequence refers
/// to an absent sub-task.
pub(super) fn find_sub_task_for_child(
    task: &TaskEntry,
    identity: ItemIdentity,
) -> Result<&SubTaskEntry> {
    task.sub_tasks()
        .iter()
        .find(|sub_task| sub_task.identity == identity)
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: format!(
                "task `{}` child ordering references missing sub-task `{}`",
                task.number, identity.anchor
            ),
        })
}

/// Render a sub-task from preserved source when stable, otherwise canonically.
///
/// Returns an error when canonical rendering encounters invalid Markdown.
fn render_sub_task(
    sub_task: &SubTaskEntry,
    indent: usize,
    report: &mut PreservationReport,
) -> Result<String> {
    if let Some(original) = sub_task.original_source() {
        validate_sub_task_for_render(sub_task)?;
        let (preserved, outcome) = render_preserved_task_or_canonical(original, || {
            render_sub_task_canonical(sub_task, indent)
        })?;
        report.record_render_outcome(outcome);
        return Ok(preserve_sub_task_separator(sub_task, original, preserved));
    }
    render_sub_task_canonical(sub_task, indent)
}

/// Render a sub-task with the requested indentation and canonical formatting.
fn render_sub_task_canonical(sub_task: &SubTaskEntry, indent: usize) -> Result<String> {
    let prefix = " ".repeat(indent);
    let mut parts = vec![format!(
        "{prefix}- {}{}. {}",
        checkbox_marker(sub_task.checked),
        sub_task.number,
        render_item_summary(&render_inline(sub_task.summary.nodes())?, indent + 2)
    )];
    let body_blocks = render_nested_body(&sub_task.body, indent + 2)?;
    if !body_blocks.is_empty() {
        parts.push(String::new());
        parts.extend(body_blocks);
    }
    Ok(parts.join("\n"))
}
