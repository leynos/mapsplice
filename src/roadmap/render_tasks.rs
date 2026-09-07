//! Task and sub-task rendering with source-preservation accounting.

use super::{
    checkbox_marker,
    find_sub_task_for_child,
    render_inline,
    render_item_summary,
    render_nested_body,
    trim_preserved_task_source,
};
use crate::{
    error::Result,
    observability::record_task_render_states,
    roadmap::model::{SubTaskEntry, TaskChild, TaskEntry},
};

/// Render a task list after validating every task and child reference.
///
/// A task with preserved source is emitted verbatim, retaining its original
/// continuation indentation and line endings. Tasks without preserved source
/// use canonical rendering, and the returned list omits trailing newlines so
/// callers can place it between Markdown blocks.
///
/// # Errors
///
/// Returns an error if a summary or body cannot be rendered, or if a task
/// child reference does not resolve to a sub-task.
pub(super) fn render_tasks(tasks: &[&TaskEntry]) -> Result<String> {
    super::validation::validate_tasks_for_render(tasks)?;
    let mut rendered = String::new();
    let mut preserved = 0;
    let mut canonical = 0;
    for task in tasks {
        if !rendered.is_empty() && !rendered.ends_with('\n') {
            rendered.push('\n');
        }
        if let Some(source) = task.task_source() {
            rendered.push_str(source);
            preserved += 1;
        } else {
            rendered.push_str(&render_task(task)?);
            canonical += 1;
        }
    }
    record_task_render_states(preserved, canonical, canonical);
    Ok(trim_preserved_task_source(&rendered).to_owned())
}

/// Render one task using canonical Markdown formatting.
///
/// The task summary and body use the two-space continuation convention, and
/// child sub-tasks are rendered in their recorded order. This path is used
/// when a task has no preserved source after insertion or invalidation.
///
/// # Errors
///
/// Returns an error when Markdown content cannot be rendered or a recorded
/// sub-task child reference is missing.
fn render_task(task: &TaskEntry) -> Result<String> {
    let mut parts = vec![format!(
        "- {}{}. {}",
        checkbox_marker(task.checked),
        task.number,
        render_item_summary(&render_inline(task.summary.nodes())?, 2)
    )];
    for child in task.children() {
        match child {
            TaskChild::Body(body) => parts.extend(render_nested_body(body, 2)?),
            TaskChild::SubTask(identity) => {
                let sub_task = find_sub_task_for_child(task, *identity)?;
                parts.push(render_sub_task(sub_task, 2)?);
            }
        }
    }
    Ok(parts.join("\n"))
}

/// Render a sub-task at the supplied list indentation.
///
/// The marker is placed at `indent` spaces and its summary and body use the
/// corresponding canonical continuation column. The caller supplies the
/// nesting level so nested sub-task lists remain structurally aligned.
///
/// # Errors
///
/// Returns an error when the sub-task summary or body cannot be rendered.
fn render_sub_task(sub_task: &SubTaskEntry, indent: usize) -> Result<String> {
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
