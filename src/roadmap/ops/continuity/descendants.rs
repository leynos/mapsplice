//! Descendant collection and pairing for in-place replacements.
//!
//! A replacement continues a descendant only where exactly one retired item and
//! exactly one replacement item carry the same anchor. Anchoring the pair to the
//! number the consumer actually wrote is what stops an unrelated item from
//! inheriting a reference.

use std::collections::{BTreeMap, BTreeSet};

use super::{RetiredSubtree, position::ItemPosition};
use crate::roadmap::{
    RoadmapAnchor,
    RoadmapDocument,
    model::{ItemIdentity, PhaseSection, StepSection, TaskEntry},
};

/// Parent step indices shared by the descendant-collection helpers.
///
/// Grouping the two indices keeps the recursive helpers within the argument
/// budget and makes the parent of every child position explicit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StepPath {
    /// Parent phase index.
    phase: usize,
    /// Step index within the phase.
    step: usize,
}

impl StepPath {
    /// Build the position of a task beneath this step.
    const fn task(self, task: usize) -> ItemPosition {
        ItemPosition::Task {
            phase: self.phase,
            step: self.step,
            task,
        }
    }

    /// Build the position of a sub-task beneath this step.
    const fn sub_task(self, task: usize, sub_task: usize) -> ItemPosition {
        ItemPosition::SubTask {
            phase: self.phase,
            step: self.step,
            task,
            sub_task,
        }
    }
}

/// Collect the identities and positions beneath one replacement item.
pub(super) fn replacement_descendants(
    roadmap: &RoadmapDocument,
    position: ItemPosition,
) -> Vec<(ItemIdentity, ItemPosition)> {
    let mut descendants = Vec::new();
    match position {
        ItemPosition::Phase(phase_index) => {
            let Some(phase_entry) = roadmap.phases.get(phase_index) else {
                return descendants;
            };
            for (step_index, step) in phase_entry.steps.iter().enumerate() {
                push_step(
                    &mut descendants,
                    StepPath {
                        phase: phase_index,
                        step: step_index,
                    },
                    step,
                );
            }
        }
        ItemPosition::Step { phase, step } => {
            let step_entry = roadmap
                .phases
                .get(phase)
                .and_then(|parent_phase| parent_phase.steps.get(step));
            if let Some(found_step) = step_entry {
                push_step_tasks(&mut descendants, StepPath { phase, step }, found_step);
            }
        }
        ItemPosition::Task { phase, step, task } => {
            let task_entry = roadmap
                .phases
                .get(phase)
                .and_then(|parent_phase| parent_phase.steps.get(step))
                .and_then(|parent_step| parent_step.tasks().get(task));
            if let Some(found_task) = task_entry {
                push_task_sub_tasks(&mut descendants, StepPath { phase, step }, task, found_task);
            }
        }
        ItemPosition::SubTask { .. } => {}
    }
    descendants
}

/// Push one step and everything beneath it.
fn push_step(
    descendants: &mut Vec<(ItemIdentity, ItemPosition)>,
    path: StepPath,
    step: &StepSection,
) {
    descendants.push((
        step.identity,
        ItemPosition::Step {
            phase: path.phase,
            step: path.step,
        },
    ));
    push_step_tasks(descendants, path, step);
}

/// Push every task and sub-task beneath one step.
fn push_step_tasks(
    descendants: &mut Vec<(ItemIdentity, ItemPosition)>,
    path: StepPath,
    step: &StepSection,
) {
    for (task_index, task) in step.tasks().iter().enumerate() {
        push_task(descendants, path, task_index, task);
    }
}

/// Push one task and its sub-tasks.
fn push_task(
    descendants: &mut Vec<(ItemIdentity, ItemPosition)>,
    path: StepPath,
    task_index: usize,
    task: &TaskEntry,
) {
    descendants.push((task.identity, path.task(task_index)));
    push_task_sub_tasks(descendants, path, task_index, task);
}

/// Push every sub-task beneath one task.
fn push_task_sub_tasks(
    descendants: &mut Vec<(ItemIdentity, ItemPosition)>,
    path: StepPath,
    task_index: usize,
    task: &TaskEntry,
) {
    for (sub_task_index, sub_task) in task.sub_tasks().iter().enumerate() {
        descendants.push((sub_task.identity, path.sub_task(task_index, sub_task_index)));
    }
}

/// Push a phase's step identities and their descendants onto `descendants`.
pub(super) fn collect_phase_descendants(phase: &PhaseSection, descendants: &mut Vec<ItemIdentity>) {
    for step in &phase.steps {
        descendants.push(step.identity);
        collect_step_descendants(step, descendants);
    }
}

/// Push a step's tasks and sub-tasks onto `descendants`.
pub(super) fn collect_step_descendants(step: &StepSection, descendants: &mut Vec<ItemIdentity>) {
    for task in step.tasks() {
        collect_task_descendants(task, descendants);
    }
}

/// Push a task's identity and its sub-task identities onto `descendants`.
pub(super) fn collect_task_descendants(task: &TaskEntry, descendants: &mut Vec<ItemIdentity>) {
    descendants.push(task.identity);
    descendants.extend(task.sub_tasks().iter().map(|sub_task| sub_task.identity));
}

/// Pair retired descendants with replacement descendants of the same anchor.
pub(crate) fn pair_descendants(
    retired: &RetiredSubtree,
    replacement: &[(ItemIdentity, ItemPosition)],
) -> Vec<(ItemIdentity, ItemPosition)> {
    let retired_by_anchor = unique_by_anchor(
        retired
            .descendants
            .iter()
            .map(|identity| (identity.anchor, *identity)),
    );
    let replacement_by_anchor = unique_by_anchor(
        replacement
            .iter()
            .map(|(identity, position)| (identity.anchor, *position)),
    );

    retired_by_anchor
        .iter()
        .filter_map(|(anchor, identity)| {
            replacement_by_anchor
                .get(anchor)
                .map(|position| (*identity, *position))
        })
        .collect()
}

/// Index values by anchor, dropping every anchor that occurs more than once.
fn unique_by_anchor<T: Copy>(
    pairs: impl IntoIterator<Item = (RoadmapAnchor, T)>,
) -> BTreeMap<RoadmapAnchor, T> {
    let mut unique = BTreeMap::new();
    let mut duplicated = BTreeSet::new();
    for (anchor, value) in pairs {
        if unique.insert(anchor, value).is_some() {
            duplicated.insert(anchor);
        }
    }
    for anchor in duplicated {
        unique.remove(&anchor);
    }
    unique
}
