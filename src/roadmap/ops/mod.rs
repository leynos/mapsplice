//! Splice operations and anchor-aware mutation helpers.

mod rewrite;

use rewrite::{renumber_document, rewrite_dependencies};

use super::{
    PhaseNumber,
    RoadmapAnchor,
    RoadmapDocument,
    RoadmapFragment,
    RoadmapItemLevel,
    StepNumber,
    TaskNumber,
    fragment_level,
    model::{PhaseSection, StepSection},
};
use crate::error::{MapspliceError, Result};

/// Library-level roadmap mutation request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoadmapOperation {
    /// Append one or more phases to the end of the roadmap.
    Append,
    /// Insert sibling items before or after the anchor.
    Insert {
        /// Anchor to insert around.
        anchor: RoadmapAnchor,
        /// Insert after the anchor when true.
        after: bool,
    },
    /// Delete the addressed item.
    Delete {
        /// Anchor to delete.
        anchor: RoadmapAnchor,
    },
    /// Replace the addressed item with fragment content.
    Replace {
        /// Anchor to replace.
        anchor: RoadmapAnchor,
    },
}

impl RoadmapOperation {
    /// Return the stable operation name used in logs and diagnostics.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Append => "append",
            Self::Insert { .. } => "insert",
            Self::Delete { .. } => "delete",
            Self::Replace { .. } => "replace",
        }
    }

    /// Return the operation anchor, when the operation addresses one.
    #[must_use]
    pub const fn anchor(self) -> Option<RoadmapAnchor> {
        match self {
            Self::Append => None,
            Self::Insert { anchor, .. } | Self::Delete { anchor } | Self::Replace { anchor } => {
                Some(anchor)
            }
        }
    }
}

/// Apply a roadmap operation to the parsed roadmap.
#[tracing::instrument(
    skip_all,
    fields(operation = operation.name(), anchor = operation.anchor().map(|anchor| anchor.to_string()).as_deref().unwrap_or(""))
)]
pub fn apply_command(
    roadmap: &mut RoadmapDocument,
    operation: RoadmapOperation,
    fragment: Option<&RoadmapFragment>,
) -> Result<()> {
    let mut staged = roadmap.clone();
    match operation {
        RoadmapOperation::Append => append_fragment(&mut staged, fragment)?,
        RoadmapOperation::Insert { anchor, after } => {
            insert_fragment(&mut staged, anchor, after, fragment)?;
        }
        RoadmapOperation::Delete { anchor } => delete_anchor(&mut staged, anchor)?,
        RoadmapOperation::Replace { anchor } => replace_anchor(&mut staged, anchor, fragment)?,
    }

    let plan = renumber_document(&mut staged)?;
    rewrite_dependencies(&mut staged, &plan)?;
    *roadmap = staged;
    Ok(())
}

/// Append a phase-level fragment to the roadmap.
fn append_fragment(
    roadmap: &mut RoadmapDocument,
    fragment: Option<&RoadmapFragment>,
) -> Result<()> {
    let fragment_document = required_fragment("append", fragment)?;
    let RoadmapFragment::Phase(phases) = fragment_document else {
        return Err(MapspliceError::AppendLevelMismatch {
            expected: RoadmapItemLevel::Phase,
            found: fragment_level(fragment_document),
        });
    };
    roadmap.phases.extend(phases.clone());
    Ok(())
}

/// Insert a fragment before or after an existing anchor.
fn insert_fragment(
    roadmap: &mut RoadmapDocument,
    anchor: RoadmapAnchor,
    after: bool,
    fragment: Option<&RoadmapFragment>,
) -> Result<()> {
    let fragment_document = required_fragment("insert", fragment)?;
    validate_fragment_level(anchor, fragment_document.level())?;

    match (anchor, fragment_document) {
        (RoadmapAnchor::Phase(target), RoadmapFragment::Phase(phases)) => {
            insert_phases(roadmap, target, after, phases)?;
        }
        (RoadmapAnchor::Step(target), RoadmapFragment::Step(steps)) => {
            insert_steps(roadmap, target, after, steps)?;
        }
        (RoadmapAnchor::Task(target), RoadmapFragment::Task(tasks)) => {
            insert_tasks(roadmap, target, after, tasks)?;
        }
        _ => {
            return Err(MapspliceError::LevelMismatch {
                anchor,
                expected: anchor.level(),
                found: fragment_level(fragment_document),
            });
        }
    }

    Ok(())
}

/// Insert phase sections around the target phase.
fn insert_phases(
    roadmap: &mut RoadmapDocument,
    target: PhaseNumber,
    after: bool,
    phases: &[PhaseSection],
) -> Result<()> {
    let index = roadmap
        .phases
        .iter()
        .position(|phase| phase.number == target)
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.into(),
        })?;
    let insert_at = insertion_index(index, after);
    roadmap.phases.splice(insert_at..insert_at, phases.to_vec());
    Ok(())
}

/// Insert step sections around the target step.
fn insert_steps(
    roadmap: &mut RoadmapDocument,
    target: StepNumber,
    after: bool,
    steps: &[StepSection],
) -> Result<()> {
    let (phase, step_index) = find_step_parent_mut(roadmap, target)?;
    let insert_at = insertion_index(step_index, after);
    phase.steps.splice(insert_at..insert_at, steps.to_vec());
    Ok(())
}

/// Insert task entries around the target task.
fn insert_tasks(
    roadmap: &mut RoadmapDocument,
    target: TaskNumber,
    after: bool,
    tasks: &[super::model::TaskEntry],
) -> Result<()> {
    let (step, task_index) = find_task_parent_mut(roadmap, target)?;
    let insert_at = insertion_index(task_index, after);
    step.tasks.splice(insert_at..insert_at, tasks.to_vec());
    Ok(())
}

/// Return the splice index for before-or-after insertion.
const fn insertion_index(index: usize, after: bool) -> usize {
    if after { index + 1 } else { index }
}

/// Delete the roadmap item addressed by the anchor.
fn delete_anchor(roadmap: &mut RoadmapDocument, anchor: RoadmapAnchor) -> Result<()> {
    match anchor {
        RoadmapAnchor::Phase(target) => {
            let index = roadmap
                .phases
                .iter()
                .position(|phase| phase.number == target)
                .ok_or(MapspliceError::AnchorNotFound { anchor })?;
            roadmap.phases.remove(index);
        }
        RoadmapAnchor::Step(target) => {
            let (phase, step_index) = find_step_parent_mut(roadmap, target)?;
            phase.steps.remove(step_index);
        }
        RoadmapAnchor::Task(target) => {
            let (step, task_index) = find_task_parent_mut(roadmap, target)?;
            step.tasks.remove(task_index);
        }
    }
    Ok(())
}

/// Replace the addressed roadmap item with sibling fragment items.
fn replace_anchor(
    roadmap: &mut RoadmapDocument,
    anchor: RoadmapAnchor,
    fragment: Option<&RoadmapFragment>,
) -> Result<()> {
    let fragment_document = required_fragment("replace", fragment)?;
    validate_fragment_level(anchor, fragment_document.level())?;

    match (anchor, fragment_document) {
        (RoadmapAnchor::Phase(target), RoadmapFragment::Phase(phases)) => {
            let index = roadmap
                .phases
                .iter()
                .position(|phase| phase.number == target)
                .ok_or(MapspliceError::AnchorNotFound { anchor })?;
            roadmap.phases.splice(index..=index, phases.clone());
        }
        (RoadmapAnchor::Step(target), RoadmapFragment::Step(steps)) => {
            let (phase, step_index) = find_step_parent_mut(roadmap, target)?;
            phase.steps.splice(step_index..=step_index, steps.clone());
        }
        (RoadmapAnchor::Task(target), RoadmapFragment::Task(tasks)) => {
            let (step, task_index) = find_task_parent_mut(roadmap, target)?;
            step.tasks.splice(task_index..=task_index, tasks.clone());
        }
        _ => {
            return Err(MapspliceError::LevelMismatch {
                anchor,
                expected: anchor.level(),
                found: fragment_level(fragment_document),
            });
        }
    }

    Ok(())
}

/// Return the fragment required by a fragment-consuming command.
fn required_fragment<'a>(
    command: &'static str,
    fragment: Option<&'a RoadmapFragment>,
) -> Result<&'a RoadmapFragment> {
    fragment.ok_or(MapspliceError::MissingFragment { command })
}

/// Ensure a fragment level matches the target anchor level.
fn validate_fragment_level(anchor: RoadmapAnchor, found: RoadmapItemLevel) -> Result<()> {
    let expected = anchor.level();
    if expected == found {
        Ok(())
    } else {
        Err(MapspliceError::LevelMismatch {
            anchor,
            expected,
            found,
        })
    }
}

/// Locate the mutable parent phase and index for a step anchor.
fn find_step_parent_mut(
    roadmap: &mut RoadmapDocument,
    target: StepNumber,
) -> Result<(&mut PhaseSection, usize)> {
    roadmap
        .phases
        .iter_mut()
        .find_map(|phase| {
            phase
                .steps
                .iter()
                .position(|step| step.number == target)
                .map(|step_index| (phase, step_index))
        })
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.into(),
        })
}

/// Locate the mutable parent step and index for a task anchor.
fn find_task_parent_mut(
    roadmap: &mut RoadmapDocument,
    target: TaskNumber,
) -> Result<(&mut StepSection, usize)> {
    for phase in &mut roadmap.phases {
        if let Some((step, task_index)) = phase.steps.iter_mut().find_map(|step| {
            step.tasks
                .iter()
                .position(|task| task.number == target)
                .map(|task_index| (step, task_index))
        }) {
            return Ok((step, task_index));
        }
    }

    Err(MapspliceError::AnchorNotFound {
        anchor: target.into(),
    })
}
