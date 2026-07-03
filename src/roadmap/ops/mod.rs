//! Splice operations and anchor-aware mutation helpers.

mod dependency_text;
mod rewrite;
mod sub_task;

use rewrite::{renumber_document, rewrite_dependencies};
use sub_task::{delete_sub_task, insert_sub_tasks, replace_sub_task};

use super::{
    PhaseNumber,
    RoadmapAnchor,
    RoadmapDocument,
    RoadmapFragment,
    RoadmapItemLevel,
    StepNumber,
    TaskNumber,
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
///
/// # Errors
///
/// Returns an error for missing or mismatched fragments, absent anchors, or
/// unresolved dependency references.
#[tracing::instrument(
    skip_all,
    fields(operation = operation.name(), anchor = operation.anchor().map(|anchor| anchor.to_string()).as_deref().unwrap_or(""))
)]
pub fn apply_command(
    roadmap: &mut RoadmapDocument,
    operation: RoadmapOperation,
    fragment: Option<RoadmapFragment>,
) -> Result<u64> {
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
    let dependency_rewrites = rewrite_dependencies(&mut staged, &plan)?;
    *roadmap = staged;
    Ok(dependency_rewrites)
}

fn append_fragment(roadmap: &mut RoadmapDocument, fragment: Option<RoadmapFragment>) -> Result<()> {
    let fragment_document = required_fragment("append", fragment)?;
    let found = fragment_document.level();
    let RoadmapFragment::Phase(phases) = fragment_document else {
        return Err(MapspliceError::AppendLevelMismatch {
            expected: RoadmapItemLevel::Phase,
            found,
        });
    };
    roadmap.phases.extend(phases);
    Ok(())
}

fn insert_fragment(
    roadmap: &mut RoadmapDocument,
    anchor: RoadmapAnchor,
    after: bool,
    fragment: Option<RoadmapFragment>,
) -> Result<()> {
    let fragment_document = required_fragment("insert", fragment)?;
    let found = fragment_document.level();
    validate_fragment_level(anchor, found)?;

    match (anchor, fragment_document) {
        (RoadmapAnchor::Phase(target), RoadmapFragment::Phase(phases)) => {
            insert_phases(roadmap, target, after, phases)
        }
        (RoadmapAnchor::Step(target), RoadmapFragment::Step(steps)) => {
            insert_steps(roadmap, target, after, steps)
        }
        (RoadmapAnchor::Task(target), RoadmapFragment::Task(tasks)) => {
            insert_tasks(roadmap, target, after, tasks)
        }
        (RoadmapAnchor::SubTask(target), RoadmapFragment::SubTask(sub_tasks)) => {
            insert_sub_tasks(roadmap, target, after, sub_tasks)
        }
        _ => Err(MapspliceError::LevelMismatch {
            anchor,
            expected: anchor.level(),
            found,
        }),
    }
}

fn insert_phases(
    roadmap: &mut RoadmapDocument,
    target: PhaseNumber,
    after: bool,
    phases: Vec<PhaseSection>,
) -> Result<()> {
    let index = find_phase_index(roadmap, target)?;
    roadmap.phases.splice(
        index + usize::from(after)..index + usize::from(after),
        phases,
    );
    Ok(())
}

fn insert_steps(
    roadmap: &mut RoadmapDocument,
    target: StepNumber,
    after: bool,
    steps: Vec<StepSection>,
) -> Result<()> {
    let (phase, step_index) = find_step_parent_mut(roadmap, target)?;
    phase.steps.splice(
        step_index + usize::from(after)..step_index + usize::from(after),
        steps,
    );
    Ok(())
}

fn insert_tasks(
    roadmap: &mut RoadmapDocument,
    target: TaskNumber,
    after: bool,
    tasks: Vec<super::model::TaskEntry>,
) -> Result<()> {
    let (step, task_index) = find_task_parent_mut(roadmap, target)?;
    step.tasks.splice(
        task_index + usize::from(after)..task_index + usize::from(after),
        tasks,
    );
    Ok(())
}

fn delete_anchor(roadmap: &mut RoadmapDocument, anchor: RoadmapAnchor) -> Result<()> {
    match anchor {
        RoadmapAnchor::Phase(target) => {
            let index = find_phase_index(roadmap, target)?;
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
        RoadmapAnchor::SubTask(target) => {
            delete_sub_task(roadmap, target)?;
        }
    }
    Ok(())
}

fn replace_anchor(
    roadmap: &mut RoadmapDocument,
    anchor: RoadmapAnchor,
    fragment: Option<RoadmapFragment>,
) -> Result<()> {
    let fragment_document = required_fragment("replace", fragment)?;
    let found = fragment_document.level();
    validate_fragment_level(anchor, found)?;

    match (anchor, fragment_document) {
        (RoadmapAnchor::Phase(target), RoadmapFragment::Phase(phases)) => {
            let index = find_phase_index(roadmap, target)?;
            roadmap.phases.splice(index..=index, phases);
            Ok(())
        }
        (RoadmapAnchor::Step(target), RoadmapFragment::Step(steps)) => {
            let (phase, step_index) = find_step_parent_mut(roadmap, target)?;
            phase.steps.splice(step_index..=step_index, steps);
            Ok(())
        }
        (RoadmapAnchor::Task(target), RoadmapFragment::Task(tasks)) => {
            let (step, task_index) = find_task_parent_mut(roadmap, target)?;
            step.tasks.splice(task_index..=task_index, tasks);
            Ok(())
        }
        (RoadmapAnchor::SubTask(target), RoadmapFragment::SubTask(sub_tasks)) => {
            replace_sub_task(roadmap, target, sub_tasks)
        }
        _ => Err(MapspliceError::LevelMismatch {
            anchor,
            expected: anchor.level(),
            found,
        }),
    }
}

fn required_fragment(
    command: &'static str,
    fragment: Option<RoadmapFragment>,
) -> Result<RoadmapFragment> {
    fragment.ok_or(MapspliceError::MissingFragment { command })
}

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

fn find_phase_index(roadmap: &RoadmapDocument, target: PhaseNumber) -> Result<usize> {
    roadmap
        .phases
        .iter()
        .position(|phase| phase.number == target)
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.into(),
        })
}

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
