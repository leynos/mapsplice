//! Validation for task structures before roadmap rendering.

use super::{find_sub_task_for_child, render_inline, render_nested_body};
use crate::{
    error::Result,
    roadmap::model::{SubTaskEntry, TaskChild, TaskEntry},
};

/// Validate all tasks before rendering a task list.
///
/// This checks summaries, bodies, and ordered child references without
/// mutating the tasks or clearing preserved source. It returns success only
/// when every task can be rendered by the canonical or preservation-aware
/// renderer.
///
/// # Errors
///
/// Returns the first rendering or missing-child error reported by a task.
pub(super) fn validate_tasks_for_render(tasks: &[&TaskEntry]) -> Result<()> {
    tasks
        .iter()
        .try_for_each(|task| validate_task_for_render(task))
}

/// Validate one task's summary, body, and ordered children.
///
/// The body is checked using the canonical top-level continuation indent.
/// Preserved task source is not changed; validation only confirms that the
/// semantic model remains renderable and that each child identity resolves.
///
/// # Errors
///
/// Returns an error when Markdown content cannot be rendered or a child
/// reference points to a missing sub-task.
fn validate_task_for_render(task: &TaskEntry) -> Result<()> {
    render_inline(task.summary.nodes())?;
    task.children().iter().try_for_each(|child| match child {
        TaskChild::Body(body) => render_nested_body(body, 2).map(drop),
        TaskChild::SubTask(identity) => {
            validate_sub_task_for_render(find_sub_task_for_child(task, *identity)?)
        }
    })
}

/// Validate a sub-task's summary and body at the canonical nested indent.
///
/// This validates the four-space content column used by a sub-task nested
/// below a top-level task and leaves any preserved source untouched.
///
/// # Errors
///
/// Returns an error when the summary or body contains unsupported Markdown.
fn validate_sub_task_for_render(sub_task: &SubTaskEntry) -> Result<()> {
    render_inline(sub_task.summary.nodes())?;
    render_nested_body(&sub_task.body, 4)?;
    Ok(())
}
