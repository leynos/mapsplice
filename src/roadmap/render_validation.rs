//! Validation for task structures before roadmap rendering.

use super::{find_sub_task_for_child, render_inline, render_nested_body};
use crate::{
    error::Result,
    roadmap::model::{SubTaskEntry, TaskChild, TaskEntry},
};

pub(super) fn validate_tasks_for_render(tasks: &[&TaskEntry]) -> Result<()> {
    tasks
        .iter()
        .try_for_each(|task| validate_task_for_render(task))
}

fn validate_task_for_render(task: &TaskEntry) -> Result<()> {
    render_inline(task.summary.nodes())?;
    task.children().iter().try_for_each(|child| match child {
        TaskChild::Body(body) => render_nested_body(body, 2).map(drop),
        TaskChild::SubTask(identity) => {
            validate_sub_task_for_render(find_sub_task_for_child(task, *identity)?)
        }
    })
}

fn validate_sub_task_for_render(sub_task: &SubTaskEntry) -> Result<()> {
    render_inline(sub_task.summary.nodes())?;
    render_nested_body(&sub_task.body, 4)?;
    Ok(())
}
