//! Splice operations and anchor-aware mutation helpers.

mod complete;

mod complete_suffix;

mod complete_text;

mod complete_wrap;

mod continuity;

mod dependency_text;

mod dependency_visitor;

mod level_edit;

mod lookup;

mod remap_kernel;

mod rewrite;

mod sub_task;

pub(crate) use complete::complete_item;
use continuity::ContinuityPlan;
pub(crate) use dependency_text::dependency_candidates;
pub(crate) use dependency_visitor::collect_dependencies;
use level_edit::{
    delete_phase,
    delete_step,
    delete_task,
    replace_phase_with_continuity,
    replace_step_with_continuity,
    replace_task_with_continuity,
};
use lookup::{find_phase_index, find_step_parent_mut, find_task_parent_mut};
pub(crate) use remap_kernel::select_resolution;
use rewrite::{renumber_document, rewrite_dependencies};
use sub_task::{
    SubTaskInsertion,
    delete_sub_task,
    insert_sub_tasks,
    replace_sub_task_with_continuity,
};

use super::{
    PhaseNumber,
    RoadmapAnchor,
    RoadmapDocument,
    RoadmapFragment,
    RoadmapItemLevel,
    StepNumber,
    TaskNumber,
    model::{PhaseSection, StepSection, TaskEntry},
    preservation_events::PreservationReport,
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

/// Validated content and placement for one structural insertion.
///
/// The fragment remains optional until this request reaches the level-specific
/// insertion helper, where [`required_fragment`] reports a missing fragment.
struct InsertionRequest {
    /// Existing item used as the insertion boundary.
    anchor: RoadmapAnchor,
    /// Place new siblings after the anchor when true.
    after: bool,
    /// Content to insert, validated by the level-specific helper.
    fragment: Option<RoadmapFragment>,
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
    apply_command_with_report(roadmap, operation, fragment)
        .map(|(dependency_rewrites, _)| dependency_rewrites)
}

/// Apply a roadmap operation and return local source-preservation outcomes.
///
/// The returned report describes only successful staged mutations. Callers can
/// defer process-wide observability until rendering and output writing also
/// succeed. Errors preserve the original roadmap because mutation occurs on a
/// cloned staging document.
pub(crate) fn apply_command_with_report(
    roadmap: &mut RoadmapDocument,
    operation: RoadmapOperation,
    fragment: Option<RoadmapFragment>,
) -> Result<(u64, PreservationReport)> {
    let mut staged = roadmap.clone();
    let mut report = PreservationReport::default();
    let continuity = match operation {
        RoadmapOperation::Append => {
            append_fragment(&mut staged, fragment)?;
            ContinuityPlan::default()
        }
        RoadmapOperation::Insert { anchor, after } => {
            insert_fragment(
                &mut staged,
                InsertionRequest {
                    anchor,
                    after,
                    fragment,
                },
                &mut report,
            )?;
            ContinuityPlan::default()
        }
        RoadmapOperation::Delete { anchor } => {
            delete_anchor(&mut staged, anchor, &mut report)?;
            ContinuityPlan::default()
        }
        RoadmapOperation::Replace { anchor } => {
            replace_anchor(&mut staged, anchor, fragment, &mut report)?
        }
    };

    let mut plan = renumber_document(&mut staged, &mut report)?;
    continuity.apply(&staged, &mut plan);
    let dependency_rewrites = rewrite_dependencies(&mut staged, &plan, &mut report)?;
    *roadmap = staged;
    Ok((dependency_rewrites, report))
}

/// Append phase-level fragment content to the roadmap.
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

/// Insert a validated fragment around an anchor and report sub-task invalidation.
///
/// Returns an error when the fragment level differs from the anchor or the
/// anchor is absent. Only sub-task insertion can invalidate a preserved parent
/// source before the common renumbering pass.
fn insert_fragment(
    roadmap: &mut RoadmapDocument,
    request: InsertionRequest,
    report: &mut PreservationReport,
) -> Result<()> {
    let InsertionRequest {
        anchor,
        after,
        fragment,
    } = request;
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
        (RoadmapAnchor::SubTask(target), RoadmapFragment::SubTask(sub_tasks)) => insert_sub_tasks(
            roadmap,
            target,
            SubTaskInsertion { after, sub_tasks },
            report,
        ),
        _ => Err(MapspliceError::LevelMismatch {
            anchor,
            expected: anchor.level(),
            found,
        }),
    }
}

/// Splice phase siblings before or after the addressed phase.
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

/// Splice step siblings within the addressed step's parent phase.
fn insert_steps(
    roadmap: &mut RoadmapDocument,
    target: StepNumber,
    after: bool,
    steps: Vec<StepSection>,
) -> Result<()> {
    let (_, phase, step_index) = find_step_parent_mut(roadmap, target)?;
    phase.steps.splice(
        step_index + usize::from(after)..step_index + usize::from(after),
        steps,
    );
    Ok(())
}

/// Splice task siblings and invalidate the parent's preserved list source.
fn insert_tasks(
    roadmap: &mut RoadmapDocument,
    target: TaskNumber,
    after: bool,
    tasks: Vec<TaskEntry>,
) -> Result<()> {
    let (_, _, step, task_index) = find_task_parent_mut(roadmap, target)?;
    step.clear_task_list_source();
    step.tasks.splice(
        task_index + usize::from(after)..task_index + usize::from(after),
        tasks,
    );
    Ok(())
}

/// Delete one addressed item and report a preserved parent source invalidation.
///
/// Returns an error when the anchor is absent. Only sub-task deletion clears a
/// retained parent source before the common renumbering pass.
fn delete_anchor(
    roadmap: &mut RoadmapDocument,
    anchor: RoadmapAnchor,
    report: &mut PreservationReport,
) -> Result<()> {
    match anchor {
        RoadmapAnchor::Phase(target) => delete_phase(roadmap, target),
        RoadmapAnchor::Step(target) => delete_step(roadmap, target),
        RoadmapAnchor::Task(target) => delete_task(roadmap, target),
        RoadmapAnchor::SubTask(target) => delete_sub_task(roadmap, target, report),
    }
}

/// Replace one addressed item and report a preserved parent source invalidation.
///
/// Returns an error for a missing fragment, an anchor-level mismatch, or an
/// absent anchor. Only sub-task replacement clears a retained parent source
/// before the common renumbering pass.
fn replace_anchor(
    roadmap: &mut RoadmapDocument,
    anchor: RoadmapAnchor,
    fragment: Option<RoadmapFragment>,
    report: &mut PreservationReport,
) -> Result<ContinuityPlan> {
    let fragment_document = required_fragment("replace", fragment)?;
    let found = fragment_document.level();
    validate_fragment_level(anchor, found)?;

    match (anchor, fragment_document) {
        (RoadmapAnchor::Phase(target), RoadmapFragment::Phase(phases)) => {
            replace_phase_with_continuity(roadmap, target, phases)
        }
        (RoadmapAnchor::Step(target), RoadmapFragment::Step(steps)) => {
            replace_step_with_continuity(roadmap, target, steps)
        }
        (RoadmapAnchor::Task(target), RoadmapFragment::Task(tasks)) => {
            replace_task_with_continuity(roadmap, target, tasks)
        }
        (RoadmapAnchor::SubTask(target), RoadmapFragment::SubTask(sub_tasks)) => {
            replace_sub_task_with_continuity(roadmap, target, sub_tasks, report)
        }
        _ => Err(MapspliceError::LevelMismatch {
            anchor,
            expected: anchor.level(),
            found,
        }),
    }
}

/// Return required fragment content or report the missing operation input.
fn required_fragment(
    command: &'static str,
    fragment: Option<RoadmapFragment>,
) -> Result<RoadmapFragment> {
    fragment.ok_or(MapspliceError::MissingFragment { command })
}

/// Reject fragment content at a different roadmap level from the anchor.
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
// weave: run 'weave explain src/roadmap/ops/mod.rs' for per-hunk detail, 'weave check' to verify your resolution
