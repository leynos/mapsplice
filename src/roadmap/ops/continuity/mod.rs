//! Identity continuity for in-place replacements.
//!
//! A replacement normally retires the addressed item: the item leaves the tree
//! and the fragment's item takes its position. When the fragment supplies
//! exactly one item at the same level, numbered as the addressed item was, the
//! operation is instead an edit of that item — it never left. This module
//! collects the retired identities and the structural positions that continue
//! them, so the renumber pass can hand the plan an alias from the old identity
//! to the new item's final number.
//!
//! Positions, not fragment anchors, identify the replacements. A fragment's
//! anchors are unvalidated and several items may share one, whereas collection
//! indices are exact and renumbering never reorders items. Descendants are
//! paired only where one retired item and one replacement item carry the same
//! anchor: anchoring the rule to the number the consumer actually wrote is what
//! stops an unrelated item from inheriting a reference.

mod descendants;
mod position;

pub(crate) use descendants::pair_descendants;
use descendants::{
    collect_phase_descendants,
    collect_step_descendants,
    collect_task_descendants,
    replacement_descendants,
};
pub(crate) use position::{ItemPosition, Replacement};

use crate::roadmap::{
    RoadmapDocument,
    model::{ItemIdentity, PhaseSection, RenumberPlan, StepSection, SubTaskEntry, TaskEntry},
};

#[cfg(test)]
mod tests;

/// The addressed identity and every identity its removal retires.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetiredSubtree {
    /// Identity of the item the operation addressed.
    addressed: ItemIdentity,
    /// Identities of every descendant the removal also takes away.
    descendants: Vec<ItemIdentity>,
}

impl RetiredSubtree {
    /// Collect a phase, its steps, tasks, and sub-tasks.
    #[must_use]
    pub(crate) fn phase(phase: &PhaseSection) -> Self {
        let mut descendants = Vec::new();
        collect_phase_descendants(phase, &mut descendants);
        Self {
            addressed: phase.identity,
            descendants,
        }
    }

    /// Collect a step, its tasks, and their sub-tasks.
    #[must_use]
    pub(crate) fn step(step: &StepSection) -> Self {
        let mut descendants = Vec::new();
        collect_step_descendants(step, &mut descendants);
        Self {
            addressed: step.identity,
            descendants,
        }
    }

    /// Collect a task and its sub-tasks.
    #[must_use]
    pub(crate) fn task(task: &TaskEntry) -> Self {
        let mut descendants = Vec::new();
        collect_task_descendants(task, &mut descendants);
        Self {
            addressed: task.identity,
            descendants,
        }
    }

    /// Collect one sub-task, which has no descendants.
    #[must_use]
    pub(crate) const fn sub_task(sub_task: &SubTaskEntry) -> Self {
        Self {
            addressed: sub_task.identity,
            descendants: Vec::new(),
        }
    }
}

/// Retired identities paired with the positions that continue them.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ContinuityPlan {
    /// One pair per identity the replacement continues.
    pairs: Vec<(ItemIdentity, ItemPosition)>,
}

impl ContinuityPlan {
    /// Record each pair's final anchor as a continuity alias.
    ///
    /// Returns how many aliases the renumber plan accepted. A pair whose item
    /// is missing, or whose identity already has a mapping, is skipped: the
    /// surviving tree, not the replacement, owns that identity.
    pub(crate) fn apply(&self, roadmap: &RoadmapDocument, plan: &mut RenumberPlan) -> usize {
        let mut recorded = 0;
        for (identity, position) in &self.pairs {
            let Some(anchor) = position.anchor(roadmap) else {
                continue;
            };
            if plan.record_continuity(*identity, anchor) {
                recorded += 1;
            }
        }
        recorded
    }
}

/// Build the continuity plan for an in-place replacement.
///
/// Continuity is granted only when the fragment supplied exactly one item and
/// that item carries the addressed anchor. Anything else — a changed number, or
/// several items — is an ordinary replacement that retires every identity.
#[must_use]
pub(crate) fn continuity_for(
    roadmap: &RoadmapDocument,
    retired: &RetiredSubtree,
    replacement: Replacement,
) -> ContinuityPlan {
    if !continues_in_place(roadmap, retired, replacement) {
        return ContinuityPlan::default();
    }
    let mut pairs = vec![(retired.addressed, replacement.position)];
    pairs.extend(pair_descendants(
        retired,
        &replacement_descendants(roadmap, replacement.position),
    ));
    ContinuityPlan { pairs }
}

/// Return whether the single spliced item carries the addressed anchor.
fn continues_in_place(
    roadmap: &RoadmapDocument,
    retired: &RetiredSubtree,
    replacement: Replacement,
) -> bool {
    replacement.count == 1
        && replacement
            .position
            .identity(roadmap)
            .is_some_and(|identity| identity.anchor == retired.addressed.anchor)
}
