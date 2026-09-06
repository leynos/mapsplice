//! Task and sub-task rendering with source-preservation fallbacks.

use markdown::mdast::Node;

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
    roadmap::model::{ItemIdentity, MarkdownNodes, SubTaskEntry, TaskChild, TaskEntry},
};

/// Render a task from preserved source when stable, otherwise canonically.
///
/// Returns an error when canonical rendering encounters invalid Markdown or
/// an inconsistent structural child sequence.
pub(super) fn render_task(task: &TaskEntry) -> Result<String> {
    if let Some(original) = task.original_source() {
        validate_task_for_render(task)?;
        let rendered =
            render_preserved_task_or_canonical(original, || render_task_canonical(task))?;
        return Ok(preserve_task_separator(task, original, rendered));
    }
    render_task_canonical(task)
}

/// Restore the separator required after a preserved task's final child.
fn preserve_task_separator(task: &TaskEntry, original: &str, rendered: String) -> String {
    if rendered == trim_preserved_task_source(original) && task_requires_trailing_separator(task) {
        format!("{rendered}\n")
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

/// Return whether a task's final child needs a trailing blank separator.
fn task_requires_trailing_separator(task: &TaskEntry) -> bool {
    task.children().last().is_some_and(|child| match child {
        TaskChild::Body(body) => markdown_requires_trailing_separator(body),
        TaskChild::SubTask(identity) => task
            .sub_tasks()
            .iter()
            .find(|sub_task| sub_task.identity == *identity)
            .is_some_and(sub_task_requires_trailing_separator),
    })
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
fn render_task_canonical(task: &TaskEntry) -> Result<String> {
    let mut parts = vec![format!(
        "- {}{}. {}",
        checkbox_marker(task.checked),
        task.number,
        render_item_summary(&render_inline(task.summary.nodes())?, 4)
    )];
    for child in task.children() {
        match child {
            TaskChild::Body(body) => parts.extend(render_nested_body(body, 4)?),
            TaskChild::SubTask(identity) => {
                let sub_task = find_sub_task_for_child(task, *identity)?;
                parts.push(render_sub_task(sub_task, 2)?);
            }
        }
    }
    Ok(parts.join("\n"))
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
fn render_sub_task(sub_task: &SubTaskEntry, indent: usize) -> Result<String> {
    if let Some(original) = sub_task.original_source() {
        validate_sub_task_for_render(sub_task)?;
        let preserved = render_preserved_task_or_canonical(original, || {
            render_sub_task_canonical(sub_task, indent)
        })?;
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
    let body_blocks = render_nested_body(&sub_task.body, indent + 4)?;
    if !body_blocks.is_empty() {
        parts.push(String::new());
        parts.extend(body_blocks);
    }
    Ok(parts.join("\n"))
}
