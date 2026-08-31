//! Per-task source preservation accessors.

use super::{TaskEntry, TaskEntryParts, validate_task_children};
use crate::error::Result;

impl TaskEntry {
    /// Build a parsed task entry from parser-owned parts.
    pub(crate) fn from_parts(parts: TaskEntryParts) -> Result<Self> {
        validate_task_children(&parts)?;
        let mut task = Self {
            identity: parts.identity,
            number: parts.number,
            checked: parts.checked,
            summary: parts.summary,
            body: parts.body,
            task_source: None,
            sub_tasks: parts.sub_tasks,
            children: parts.children,
        };
        task.set_task_source(parts.task_source);
        Ok(task)
    }

    /// Return verbatim source captured for this task during parsing.
    #[must_use]
    pub(crate) fn task_source(&self) -> Option<&str> { self.task_source.as_deref() }

    /// Set verbatim source captured for this task during parsing.
    pub(crate) fn set_task_source(&mut self, source: Option<String>) { self.task_source = source; }

    /// Clear verbatim source after a semantic task edit.
    pub(crate) fn clear_task_source(&mut self) { self.task_source = None; }
}
