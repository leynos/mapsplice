//! Level-specific delete and replace edits applied by the anchor operations.

use super::{
    continuity::{ContinuityPlan, ItemPosition, Replacement, RetiredSubtree, continuity_for},
    lookup::{
        find_phase,
        find_phase_index,
        find_step,
        find_step_parent_mut,
        find_task,
        find_task_parent_mut,
    },
};
use crate::{
    error::Result,
    roadmap::{
        PhaseNumber,
        RoadmapDocument,
        StepNumber,
        TaskNumber,
        model::{PhaseSection, StepSection, TaskEntry},
    },
};

/// Remove one phase by number.
pub(super) fn delete_phase(roadmap: &mut RoadmapDocument, target: PhaseNumber) -> Result<()> {
    let index = find_phase_index(roadmap, target)?;
    roadmap.phases.remove(index);
    Ok(())
}

/// Remove one step by number.
pub(super) fn delete_step(roadmap: &mut RoadmapDocument, target: StepNumber) -> Result<()> {
    let (_, phase, step_index) = find_step_parent_mut(roadmap, target)?;
    phase.steps.remove(step_index);
    Ok(())
}

/// Remove one task by number and invalidate its parent's preserved list source.
pub(super) fn delete_task(roadmap: &mut RoadmapDocument, target: TaskNumber) -> Result<()> {
    let (_, _, step, task_index) = find_task_parent_mut(roadmap, target)?;
    step.clear_task_list_source();
    step.tasks.remove(task_index);
    Ok(())
}

/// Splice `replacements` over the addressed slot and describe the outcome.
///
/// The position describes the slot before the splice; the returned
/// [`Replacement`] reports how many items now occupy it, which is what
/// descendant pairing needs. The count is read before the splice consumes the
/// vector.
fn splice_replacement<T>(
    collection: &mut Vec<T>,
    index: usize,
    replacements: Vec<T>,
    position: ItemPosition,
) -> Replacement {
    let count = replacements.len();
    collection.splice(index..=index, replacements);
    Replacement::new(position, count)
}

/// Replace one phase with the fragment's phases.
pub(super) fn replace_phase(
    roadmap: &mut RoadmapDocument,
    target: PhaseNumber,
    phases: Vec<PhaseSection>,
) -> Result<Replacement> {
    let index = find_phase_index(roadmap, target)?;
    Ok(splice_replacement(
        &mut roadmap.phases,
        index,
        phases,
        ItemPosition::Phase(index),
    ))
}

/// Replace one step with the fragment's steps.
pub(super) fn replace_step(
    roadmap: &mut RoadmapDocument,
    target: StepNumber,
    steps: Vec<StepSection>,
) -> Result<Replacement> {
    let (phase_index, phase, step_index) = find_step_parent_mut(roadmap, target)?;
    Ok(splice_replacement(
        &mut phase.steps,
        step_index,
        steps,
        ItemPosition::Step {
            phase: phase_index,
            step: step_index,
        },
    ))
}

/// Replace one task with the fragment's tasks and invalidate its parent's list source.
pub(super) fn replace_task(
    roadmap: &mut RoadmapDocument,
    target: TaskNumber,
    tasks: Vec<TaskEntry>,
) -> Result<Replacement> {
    let (phase_index, step_index, step, task_index) = find_task_parent_mut(roadmap, target)?;
    step.clear_task_list_source();
    Ok(splice_replacement(
        &mut step.tasks,
        task_index,
        tasks,
        ItemPosition::Task {
            phase: phase_index,
            step: step_index,
            task: task_index,
        },
    ))
}

/// Replace one phase and continue the identities that stay in place.
pub(super) fn replace_phase_with_continuity(
    roadmap: &mut RoadmapDocument,
    target: PhaseNumber,
    phases: Vec<PhaseSection>,
) -> Result<ContinuityPlan> {
    let retired = RetiredSubtree::phase(find_phase(roadmap, target)?);
    let replacement = replace_phase(roadmap, target, phases)?;
    Ok(continuity_for(roadmap, &retired, replacement))
}

/// Replace one step and continue the identities that stay in place.
pub(super) fn replace_step_with_continuity(
    roadmap: &mut RoadmapDocument,
    target: StepNumber,
    steps: Vec<StepSection>,
) -> Result<ContinuityPlan> {
    let retired = RetiredSubtree::step(find_step(roadmap, target)?);
    let replacement = replace_step(roadmap, target, steps)?;
    Ok(continuity_for(roadmap, &retired, replacement))
}

/// Replace one task and continue the identities that stay in place.
pub(super) fn replace_task_with_continuity(
    roadmap: &mut RoadmapDocument,
    target: TaskNumber,
    tasks: Vec<TaskEntry>,
) -> Result<ContinuityPlan> {
    let retired = RetiredSubtree::task(find_task(roadmap, target)?);
    let replacement = replace_task(roadmap, target, tasks)?;
    Ok(continuity_for(roadmap, &retired, replacement))
}
