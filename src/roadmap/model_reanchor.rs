//! Re-anchoring of retained identities after a committed operation.
//!
//! An operation rewrites dependency references against a renumber plan keyed
//! by the anchors items held when the operation started. Those anchors are
//! stale as soon as the operation commits: the next operation reads the
//! rendered numbers, so its plan must be keyed by them. Resetting each
//! identity to its current number, and to the target source because spliced
//! fragment content is now part of the target document, makes the committed
//! document identical in identity terms to a fresh parse of its own rendering.
//!
//! This is what lets a caller apply successive operations without rendering
//! and re-parsing between them. A continuity alias granted by an in-place
//! replacement is therefore only needed for the operation that granted it; the
//! re-anchored identity carries the continuation forward.

use super::{
    ItemIdentity,
    RoadmapAnchor,
    RoadmapDocument,
    SourceId,
    StepSection,
    TaskChild,
    TaskEntry,
};

impl RoadmapDocument {
    /// Re-anchor every retained identity to the number its item now holds.
    pub(crate) fn reanchor_identities(&mut self) {
        for phase in &mut self.phases {
            phase.identity = identity_at(phase.number.into());
            for step in &mut phase.steps {
                reanchor_step(step);
            }
        }
    }
}

/// Return the target-sourced identity for an item's current anchor.
const fn identity_at(anchor: RoadmapAnchor) -> ItemIdentity {
    ItemIdentity {
        source: SourceId::Target,
        anchor,
    }
}

/// Re-anchor one step, its tasks, and their sub-tasks.
fn reanchor_step(step: &mut StepSection) {
    step.identity = identity_at(step.number.into());
    for task in &mut step.tasks {
        reanchor_task(task);
    }
}

/// Re-anchor a task and its sub-tasks, realigning structural child markers.
///
/// The parser and every splice keep exactly one [`TaskChild::SubTask`] marker
/// per sub-task, in the same order, so the marker sequence is updated by
/// position rather than by the old identity: duplicate fragment anchors are
/// possible before they are re-anchored to distinct numbers.
fn reanchor_task(task: &mut TaskEntry) {
    task.identity = identity_at(RoadmapAnchor::Task(task.number));
    for sub_task in &mut task.sub_tasks {
        sub_task.identity = identity_at(RoadmapAnchor::SubTask(sub_task.number));
    }

    let mut reanchored = task.sub_tasks.iter().map(|sub_task| sub_task.identity);
    for marker in task.children.iter_mut().filter_map(|child| match child {
        TaskChild::SubTask(identity) => Some(identity),
        TaskChild::Body(_) => None,
    }) {
        if let Some(identity) = reanchored.next() {
            *marker = identity;
        }
    }
}
