//! Behaviour for parsed roadmap step sections.

use super::model::{StepSection, TaskEntry};
use crate::error::Result;

impl StepSection {
    /// Return the step's tasks in their current source order.
    #[must_use]
    pub fn tasks(&self) -> &[TaskEntry] { &self.tasks }

    /// Mutate task membership after invalidating the whole-list source cache.
    ///
    /// Callers changing an individual task must use that task's invalidating
    /// setters so its preserved source cannot outlive the semantic change.
    pub(crate) fn tasks_mut(&mut self) -> &mut Vec<TaskEntry> {
        self.clear_task_list_source();
        &mut self.tasks
    }

    /// Update tasks and discard list source only when the update changes them.
    ///
    /// The callback returns whether it changed a task. This lets traversals
    /// inspect every task without losing whole-list preservation for steps
    /// whose semantic task content remains unchanged.
    pub(crate) fn update_tasks(
        &mut self,
        update: impl FnOnce(&mut [TaskEntry]) -> Result<bool>,
    ) -> Result<()> {
        if update(&mut self.tasks)? {
            self.clear_task_list_source();
        }
        Ok(())
    }

    /// Return the preserved source for an unchanged task list.
    #[must_use]
    pub(crate) fn task_list_source(&self) -> Option<&str> { self.task_list_source.as_deref() }

    /// Preserve the original task list source when parsing an unchanged step.
    pub(crate) fn set_task_list_source(&mut self, s: Option<String>) { self.task_list_source = s; }

    /// Clear preserved task list source after semantic task changes.
    pub(crate) fn clear_task_list_source(&mut self) { self.task_list_source = None; }
}
