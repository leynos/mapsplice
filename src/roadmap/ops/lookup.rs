//! Anchor lookups shared by the roadmap splice helpers.
//!
//! Immutable finders serve replacement, which reads the retired item before
//! splicing. Mutable parent finders serve insertion and deletion, which need
//! the owning collection and the item's index. Every finder reports a missing
//! anchor rather than treating it as an empty result.

use super::super::{
    PhaseNumber,
    RoadmapDocument,
    StepNumber,
    SubTaskNumber,
    TaskNumber,
    model::{PhaseSection, StepSection, SubTaskEntry, TaskEntry},
};
use crate::error::{MapspliceError, Result};

/// Locate a numbered phase or report its anchor as missing.
pub(super) fn find_phase_index(roadmap: &RoadmapDocument, target: PhaseNumber) -> Result<usize> {
    roadmap
        .phases
        .iter()
        .position(|phase| phase.number == target)
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.into(),
        })
}

/// Borrow a numbered phase, reporting a missing anchor.
pub(super) fn find_phase(roadmap: &RoadmapDocument, target: PhaseNumber) -> Result<&PhaseSection> {
    let index = find_phase_index(roadmap, target)?;
    roadmap
        .phases
        .get(index)
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.into(),
        })
}

/// Iterate over every step in document order.
fn steps(roadmap: &RoadmapDocument) -> impl Iterator<Item = &StepSection> {
    roadmap.phases.iter().flat_map(|phase| &phase.steps)
}

/// Borrow a numbered step, reporting a missing anchor.
pub(super) fn find_step(roadmap: &RoadmapDocument, target: StepNumber) -> Result<&StepSection> {
    steps(roadmap)
        .find(|step| step.number == target)
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.into(),
        })
}

/// Borrow a numbered task, reporting a missing anchor.
pub(super) fn find_task(roadmap: &RoadmapDocument, target: TaskNumber) -> Result<&TaskEntry> {
    steps(roadmap)
        .flat_map(StepSection::tasks)
        .find(|task| task.number() == target)
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.into(),
        })
}

/// Borrow a numbered sub-task, reporting a missing anchor.
pub(super) fn find_sub_task(
    roadmap: &RoadmapDocument,
    target: SubTaskNumber,
) -> Result<&SubTaskEntry> {
    steps(roadmap)
        .flat_map(StepSection::tasks)
        .find_map(|task| {
            task.sub_tasks()
                .iter()
                .find(|sub_task| sub_task.number == target)
        })
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.into(),
        })
}

/// Find the parent phase index and step index of an addressed step.
pub(super) fn find_step_parent_mut(
    roadmap: &mut RoadmapDocument,
    target: StepNumber,
) -> Result<(usize, &mut PhaseSection, usize)> {
    let (phase_index, step_index) = roadmap
        .phases
        .iter()
        .enumerate()
        .find_map(|(phase_index, phase)| {
            phase
                .steps
                .iter()
                .position(|step| step.number == target)
                .map(|step_index| (phase_index, step_index))
        })
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.into(),
        })?;
    let phase = roadmap
        .phases
        .get_mut(phase_index)
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.into(),
        })?;
    Ok((phase_index, phase, step_index))
}

/// Find the phase index, step index, and task index of an addressed task.
pub(super) fn find_task_parent_mut(
    roadmap: &mut RoadmapDocument,
    target: TaskNumber,
) -> Result<(usize, usize, &mut StepSection, usize)> {
    let (phase_index, step_index, task_index) = roadmap
        .phases
        .iter()
        .enumerate()
        .find_map(|(phase_index, phase)| {
            phase
                .steps
                .iter()
                .enumerate()
                .find_map(|(step_index, step)| {
                    step.tasks()
                        .iter()
                        .position(|task| task.number() == target)
                        .map(|task_index| (phase_index, step_index, task_index))
                })
        })
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.into(),
        })?;
    let step = roadmap
        .phases
        .get_mut(phase_index)
        .and_then(|phase| phase.steps.get_mut(step_index))
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.into(),
        })?;
    Ok((phase_index, step_index, step, task_index))
}
