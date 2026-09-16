//! Behaviour and invariants for roadmap task and sub-task entries.

use std::collections::BTreeSet;

use super::{
    MapspliceError,
    Result,
    SubTaskEntry,
    SubTaskNumber,
    SubTaskSplice,
    TaskChild,
    TaskEntry,
    TaskEntryParts,
};
use crate::observability::{PreservationInvalidationReason, record_preserved_source_invalidation};

impl TaskEntry {
    /// Build a parsed task entry from parser-owned parts.
    pub(crate) fn from_parts(parts: TaskEntryParts) -> Result<Self> {
        validate_task_children(&parts)?;
        Ok(Self {
            identity: parts.identity,
            number: parts.number,
            checked: parts.checked,
            summary: parts.summary,
            body: parts.body,
            original_source: parts.original_source,
            sub_tasks: parts.sub_tasks,
            children: parts.children,
        })
    }

    /// Return the structural sub-tasks nested beneath this task.
    #[must_use]
    pub fn sub_tasks(&self) -> &[SubTaskEntry] { &self.sub_tasks }

    /// Return mutable sub-tasks for renumbering and dependency rewriting.
    ///
    /// Callers must preserve the correspondence between this slice and the
    /// structural [`TaskChild`] entries held by the task.
    pub(crate) fn sub_tasks_mut(&mut self) -> &mut [SubTaskEntry] { &mut self.sub_tasks }

    /// Return the original source while this task remains unchanged.
    #[must_use]
    pub(crate) fn original_source(&self) -> Option<&str> { self.original_source.as_deref() }

    /// Clear preserved source after this task or one of its descendants changes.
    ///
    /// Records the supplied cause only when a source span was present, so the
    /// counter measures invalidations rather than mutation attempts.
    pub(crate) fn clear_original_source(&mut self, reason: PreservationInvalidationReason) {
        if self.original_source.take().is_some() {
            record_preserved_source_invalidation(reason);
        }
    }

    /// Return the original ordered task-child sequence.
    #[must_use]
    pub(crate) fn children(&self) -> &[TaskChild] { &self.children }

    /// Find the index of a structural sub-task by rendered number.
    #[must_use]
    pub(crate) fn find_sub_task_index(&self, target: SubTaskNumber) -> Option<usize> {
        self.sub_tasks
            .iter()
            .position(|sub_task| sub_task.number == target)
    }

    /// Insert sub-tasks while keeping the child order vector aligned.
    pub(crate) fn insert_sub_tasks(
        &mut self,
        splice: SubTaskSplice,
        after: bool,
        sub_tasks: Vec<SubTaskEntry>,
    ) {
        self.clear_original_source(PreservationInvalidationReason::ChildMutation);
        let new_children = sub_task_children(&sub_tasks);
        let insert_at = splice.sub_task_index + usize::from(after);
        let child_insert_at = splice.child_index + usize::from(after);
        self.sub_tasks.splice(insert_at..insert_at, sub_tasks);
        self.children
            .splice(child_insert_at..child_insert_at, new_children);
    }

    /// Delete one sub-task while keeping the child order vector aligned.
    pub(crate) fn delete_sub_task(&mut self, splice: SubTaskSplice) {
        self.clear_original_source(PreservationInvalidationReason::ChildMutation);
        self.sub_tasks.remove(splice.sub_task_index);
        self.children.remove(splice.child_index);
    }

    /// Replace one sub-task while keeping the child order vector aligned.
    pub(crate) fn replace_sub_task(&mut self, splice: SubTaskSplice, sub_tasks: Vec<SubTaskEntry>) {
        self.clear_original_source(PreservationInvalidationReason::ChildMutation);
        let new_children = sub_task_children(&sub_tasks);
        self.sub_tasks
            .splice(splice.sub_task_index..=splice.sub_task_index, sub_tasks);
        self.children
            .splice(splice.child_index..=splice.child_index, new_children);
    }

    /// Remove a sub-task without updating structural children for invariant tests.
    #[cfg(test)]
    pub(crate) fn remove_sub_task_without_child_update_for_test(
        &mut self,
        sub_task_index: usize,
    ) -> SubTaskEntry {
        self.sub_tasks.remove(sub_task_index)
    }
}

impl SubTaskEntry {
    /// Return the original source while this sub-task remains unchanged.
    #[must_use]
    pub(crate) fn original_source(&self) -> Option<&str> { self.original_source.as_deref() }

    /// Clear preserved source after this sub-task changes.
    ///
    /// Records the supplied cause only when a source span was present.
    pub(crate) fn clear_original_source(&mut self, reason: PreservationInvalidationReason) {
        if self.original_source.take().is_some() {
            record_preserved_source_invalidation(reason);
        }
    }
}

/// Build child markers for the supplied sub-task sequence in the same order.
fn sub_task_children(sub_tasks: &[SubTaskEntry]) -> Vec<TaskChild> {
    sub_tasks
        .iter()
        .map(|sub_task| TaskChild::SubTask(sub_task.identity))
        .collect()
}

/// Validate that structural children identify exactly the task's sub-tasks.
///
/// Returns [`MapspliceError::InvalidRoadmap`] when either representation is
/// missing an identity or contains an identity not present in the other.
fn validate_task_children(parts: &TaskEntryParts) -> Result<()> {
    let sub_task_identities = parts
        .sub_tasks
        .iter()
        .map(|sub_task| sub_task.identity)
        .collect::<BTreeSet<_>>();
    let child_identities = parts
        .children
        .iter()
        .filter_map(|child| match child {
            TaskChild::Body(_) => None,
            TaskChild::SubTask(identity) => Some(*identity),
        })
        .collect::<BTreeSet<_>>();
    if sub_task_identities == child_identities {
        Ok(())
    } else {
        Err(MapspliceError::InvalidRoadmap {
            message: format!(
                "task `{}` has inconsistent structural sub-task children",
                parts.number
            ),
        })
    }
}
