//! Post-edit identity tree and the numbering derived from it.
//!
//! Concrete `Requires` clause rewriting is not observed here. The tree is
//! built by applying the edit to the generation-time identities, then
//! numbered by position, which recomputes the C2 renumber contract from the
//! model rather than from the pipeline's output.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    Edit,
    oracle_identity::{
        FRAGMENT_TASKS,
        ItemId,
        Level,
        PHASE_COUNT,
        STEPS_PER_PHASE,
        TASKS_PER_STEP,
        sub_tasks_of_task,
    },
};

/// One task node in the post-edit identity tree.
#[derive(Clone, Debug)]
struct TaskNode {
    id: ItemId,
    sub_tasks: Vec<ItemId>,
}

/// One step node in the post-edit identity tree.
#[derive(Clone, Debug)]
struct StepNode {
    id: ItemId,
    tasks: Vec<TaskNode>,
}

/// One phase node in the post-edit identity tree.
#[derive(Clone, Debug)]
struct PhaseNode {
    id: ItemId,
    steps: Vec<StepNode>,
}

/// Post-edit identity structure plus the identities the edit removed.
#[derive(Clone, Debug)]
pub(super) struct EditTree {
    phases: Vec<PhaseNode>,
    pub(super) removed: BTreeSet<ItemId>,
}

/// Apply `edit` to the generated identity structure without using numbers.
pub(super) fn post_edit_tree(edit: Edit) -> EditTree {
    let removed = removed_items(edit);
    let splice = splice_task(edit);
    let mut phases = Vec::new();
    for phase in 0..PHASE_COUNT {
        let mut steps = Vec::new();
        for offset in 0..STEPS_PER_PHASE {
            let step = phase * STEPS_PER_PHASE + offset;
            if removed.contains(&ItemId::new(Level::Step, step)) {
                continue;
            }
            steps.push(StepNode {
                id: ItemId::new(Level::Step, step),
                tasks: surviving_tasks(step, &removed, splice),
            });
        }
        if removed.contains(&ItemId::new(Level::Phase, phase)) {
            continue;
        }
        phases.push(PhaseNode {
            id: ItemId::new(Level::Phase, phase),
            steps,
        });
    }
    EditTree { phases, removed }
}

/// Return one step's tasks after the edit, with the fragment spliced in.
///
/// `splice` names the addressed task, the only position the fragment lands at:
/// before it for an insert, in its place for a replace. Extracting the loop
/// keeps the nesting shallow enough for the workspace's threshold.
fn surviving_tasks(
    step: usize,
    removed: &BTreeSet<ItemId>,
    splice: Option<(usize, Splice)>,
) -> Vec<TaskNode> {
    let mut tasks = Vec::new();
    for position in 0..TASKS_PER_STEP {
        let task_index = step * TASKS_PER_STEP + position;
        let task = ItemId::new(Level::Task, task_index);
        if splice == Some((task_index, Splice::Before)) {
            append_fragment_tasks(&mut tasks);
        }
        if !removed.contains(&task) {
            tasks.push(TaskNode {
                id: task,
                sub_tasks: sub_tasks_of_task(task_index),
            });
            continue;
        }
        if splice == Some((task_index, Splice::InPlace)) {
            append_fragment_tasks(&mut tasks);
        }
    }
    tasks
}

/// Where an edit puts the fragment tasks relative to its addressed task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Splice {
    /// The fragment tasks precede the addressed task, which survives.
    Before,
    /// The fragment tasks take the addressed task's position, which is removed.
    InPlace,
}

/// Return the addressed task and splice position of a task-splicing edit.
///
/// Delete and step- or phase-level edits splice nothing and return [`None`].
const fn splice_task(edit: Edit) -> Option<(usize, Splice)> {
    match edit {
        Edit::InsertTasksBefore { task } => Some((task, Splice::Before)),
        Edit::ReplaceTask { task } => Some((task, Splice::InPlace)),
        Edit::DeleteTask { .. } | Edit::DeleteStep { .. } | Edit::DeletePhase { .. } => None,
    }
}

/// Push the fragment's task identities onto the post-edit task sequence.
///
/// Fragment tasks never carry addendum sub-tasks: a fragment task owns a
/// sub-task only where the fragment itself nests one, and the generated
/// fragment is a flat checklist.
fn append_fragment_tasks(tasks: &mut Vec<TaskNode>) {
    for index in 0..FRAGMENT_TASKS {
        tasks.push(TaskNode {
            id: ItemId::fragment_task(index),
            sub_tasks: Vec::new(),
        });
    }
}

/// Assign an anchor to every identity in the post-edit tree, by position.
///
/// This is the C2 renumber contract recomputed from the model rather than
/// observed from the pipeline: numbers are contiguous from 1, in document
/// order, at every level. Fragment identities renumbered into the target are
/// ordinary tasks at their spliced position, so the same rule applies to them.
pub(super) fn number_tree(tree: &EditTree) -> BTreeMap<ItemId, String> {
    let mut anchors = BTreeMap::new();
    for (phase_position, phase) in tree.phases.iter().enumerate() {
        let phase_number = phase_position + 1;
        anchors.insert(phase.id, phase_number.to_string());
        for (step_position, step) in phase.steps.iter().enumerate() {
            let step_anchor = format!("{phase_number}.{}", step_position + 1);
            anchors.insert(step.id, step_anchor.clone());
            number_tasks(&step.tasks, &step_anchor, &mut anchors);
        }
    }
    anchors
}

/// Number one step's tasks and their addendum sub-tasks, appending to `anchors`.
fn number_tasks(tasks: &[TaskNode], step_anchor: &str, anchors: &mut BTreeMap<ItemId, String>) {
    for (task_position, task) in tasks.iter().enumerate() {
        let task_anchor = format!("{step_anchor}.{}", task_position + 1);
        anchors.insert(task.id, task_anchor.clone());
        for (sub_position, sub_task) in task.sub_tasks.iter().enumerate() {
            anchors.insert(*sub_task, format!("{task_anchor}.{}", sub_position + 1));
        }
    }
}

/// Return the identities one edit removes.
fn removed_items(edit: Edit) -> BTreeSet<ItemId> {
    let mut removed = BTreeSet::new();
    match edit {
        Edit::InsertTasksBefore { .. } => {}
        Edit::DeleteTask { task } | Edit::ReplaceTask { task } => {
            removed.insert(ItemId::new(Level::Task, task));
            removed.extend(sub_tasks_of_task(task));
        }
        Edit::DeleteStep { step } => {
            removed.insert(ItemId::new(Level::Step, step));
            removed.extend(step_descendants(step));
        }
        Edit::DeletePhase { phase } => {
            removed.insert(ItemId::new(Level::Phase, phase));
            for offset in 0..STEPS_PER_PHASE {
                let step = phase * STEPS_PER_PHASE + offset;
                removed.insert(ItemId::new(Level::Step, step));
                removed.extend(step_descendants(step));
            }
        }
    }
    removed
}

/// Return every task and sub-task identity beneath one step.
fn step_descendants(step: usize) -> Vec<ItemId> {
    let mut descendants = Vec::new();
    for position in 0..TASKS_PER_STEP {
        let task = step * TASKS_PER_STEP + position;
        descendants.push(ItemId::new(Level::Task, task));
        descendants.extend(sub_tasks_of_task(task));
    }
    descendants
}
