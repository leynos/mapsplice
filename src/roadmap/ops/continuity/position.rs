//! Structural positions of items in a staged roadmap.
//!
//! A position is a collection index path captured after a splice and read after
//! renumbering. It is never stored as an anchor, because the anchor can change
//! while the position cannot.

use crate::roadmap::{RoadmapAnchor, RoadmapDocument, model::ItemIdentity};

/// Location of an item in the staged roadmap, as collection indices.
///
/// The position is captured after a splice and read after renumbering. It is
/// never converted to an anchor for storage, because the anchored value can
/// change while the position cannot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ItemPosition {
    /// Position in the document's phase vector.
    Phase(usize),
    /// Phase and step indices.
    Step {
        /// Parent phase index.
        phase: usize,
        /// Step index within the phase.
        step: usize,
    },
    /// Phase, step, and task indices.
    Task {
        /// Parent phase index.
        phase: usize,
        /// Parent step index.
        step: usize,
        /// Task index within the step.
        task: usize,
    },
    /// Phase, step, task, and sub-task indices.
    SubTask {
        /// Parent phase index.
        phase: usize,
        /// Parent step index.
        step: usize,
        /// Parent task index.
        task: usize,
        /// Sub-task index within the task.
        sub_task: usize,
    },
}

impl ItemPosition {
    /// Return the identity recorded at this position.
    ///
    /// Used to decide whether the spliced item is the addressed one: its
    /// identity still carries the anchor the fragment wrote.
    #[must_use]
    pub(crate) fn identity(self, roadmap: &RoadmapDocument) -> Option<ItemIdentity> {
        match self {
            Self::Phase(phase) => roadmap.phases.get(phase).map(|entry| entry.identity),
            Self::Step { phase, step } => roadmap
                .phases
                .get(phase)?
                .steps
                .get(step)
                .map(|entry| entry.identity),
            Self::Task { phase, step, task } => roadmap
                .phases
                .get(phase)?
                .steps
                .get(step)?
                .tasks()
                .get(task)
                .map(|entry| entry.identity),
            Self::SubTask {
                phase,
                step,
                task,
                sub_task,
            } => roadmap
                .phases
                .get(phase)?
                .steps
                .get(step)?
                .tasks()
                .get(task)?
                .sub_tasks()
                .get(sub_task)
                .map(|entry| entry.identity),
        }
    }

    /// Return the item's current number at this position.
    ///
    /// This reads the rendered number rather than the identity, so it must be
    /// called after renumbering to observe where the item actually landed.
    #[must_use]
    pub(crate) fn anchor(self, roadmap: &RoadmapDocument) -> Option<RoadmapAnchor> {
        match self {
            Self::Phase(phase) => roadmap.phases.get(phase).map(|entry| entry.number.into()),
            Self::Step { phase, step } => roadmap
                .phases
                .get(phase)?
                .steps
                .get(step)
                .map(|entry| entry.number.into()),
            Self::Task { phase, step, task } => roadmap
                .phases
                .get(phase)?
                .steps
                .get(step)?
                .tasks()
                .get(task)
                .map(|entry| entry.number().into()),
            Self::SubTask {
                phase,
                step,
                task,
                sub_task,
            } => roadmap
                .phases
                .get(phase)?
                .steps
                .get(step)?
                .tasks()
                .get(task)?
                .sub_tasks()
                .get(sub_task)
                .map(|entry| entry.number.into()),
        }
    }
}

/// One fragment splice and the number of items it supplied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Replacement {
    /// Position of the first item the fragment supplied.
    pub(crate) position: ItemPosition,
    /// Number of items the fragment supplied at the addressed level.
    pub(crate) count: usize,
}

impl Replacement {
    /// Pair a splice position with the fragment's top-level item count.
    #[must_use]
    pub(crate) const fn new(position: ItemPosition, count: usize) -> Self {
        Self { position, count }
    }
}
