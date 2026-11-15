//! Level-specific delete and replace edits applied by the anchor operations.

use super::{find_phase_index, find_step_parent_mut, find_task_parent_mut};
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
    let (phase, step_index) = find_step_parent_mut(roadmap, target)?;
    phase.steps.remove(step_index);
    Ok(())
}

/// Remove one task by number and invalidate its parent's preserved list source.
pub(super) fn delete_task(roadmap: &mut RoadmapDocument, target: TaskNumber) -> Result<()> {
    let (step, task_index) = find_task_parent_mut(roadmap, target)?;
    step.clear_task_list_source();
    step.tasks.remove(task_index);
    Ok(())
}

/// Replace one phase with the fragment's phases.
pub(super) fn replace_phase(
    roadmap: &mut RoadmapDocument,
    target: PhaseNumber,
    phases: Vec<PhaseSection>,
) -> Result<()> {
    let index = find_phase_index(roadmap, target)?;
    roadmap.phases.splice(index..=index, phases);
    Ok(())
}

/// Replace one step with the fragment's steps.
pub(super) fn replace_step(
    roadmap: &mut RoadmapDocument,
    target: StepNumber,
    steps: Vec<StepSection>,
) -> Result<()> {
    let (phase, step_index) = find_step_parent_mut(roadmap, target)?;
    phase.steps.splice(step_index..=step_index, steps);
    Ok(())
}

/// Replace one task with the fragment's tasks and invalidate its parent's list source.
pub(super) fn replace_task(
    roadmap: &mut RoadmapDocument,
    target: TaskNumber,
    tasks: Vec<TaskEntry>,
) -> Result<()> {
    let (step, task_index) = find_task_parent_mut(roadmap, target)?;
    step.clear_task_list_source();
    step.tasks.splice(task_index..=task_index, tasks);
    Ok(())
}
