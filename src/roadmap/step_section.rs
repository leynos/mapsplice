//! Behaviour for parsed roadmap step sections.

use super::model::StepSection;

impl StepSection {
    /// Return the preserved source for an unchanged task list.
    #[must_use]
    pub(crate) fn task_list_source(&self) -> Option<&str> { self.task_list_source.as_deref() }

    /// Preserve the original task list source when parsing an unchanged step.
    pub(crate) fn set_task_list_source(&mut self, s: Option<String>) { self.task_list_source = s; }

    /// Clear preserved task list source after semantic task changes.
    pub(crate) fn clear_task_list_source(&mut self) { self.task_list_source = None; }
}
