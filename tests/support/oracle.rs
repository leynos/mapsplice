//! Independent identity-preserving oracle for generated dependency edits.
//!
//! The oracle is built from generation parameters, not from a parse of the
//! generated text, and never calls the production clause recognizer
//! (`src/roadmap/ops/dependency_text.rs`). A defect shared between the two
//! therefore cannot cancel out.
//!
//! The model is split three ways, because conflating the parts is the defect
//! class this oracle exists to catch:
//!
//! - **Identity** — [`oracle_identity`] holds the generation-time tokens and the positional helpers
//!   that derive numbers from them. Numbers are never the identity. This is what lets the oracle
//!   tell "the consumer's clause follows its prerequisite" apart from "the consumer's clause still
//!   reads `1.1.1`, which the inserted task now also holds".
//! - **Dependency graph** — [`oracle_model`] holds [`Edge`] values over identities, built from the
//!   same selectors that drive generation, independent of clause spelling. [`ClauseForm`] varies
//!   only the bytes of the generated Markdown, never the expected outcome.
//! - **Post-edit structure** — [`oracle_tree`] applies the edit to the identity tree, numbers it by
//!   position, and reports which identities the edit removed.
//!
//! Expected outcomes are derived from the post-edit identity tree alone:
//! which identities the edit removes, which survive, and which edges still
//! point at a removed identity afterwards.

#[path = "oracle_identity.rs"]
mod oracle_identity;
#[path = "oracle_model.rs"]
mod oracle_model;
#[path = "oracle_tree.rs"]
mod oracle_tree;

use std::collections::{BTreeMap, BTreeSet};

pub(crate) use oracle_identity::{
    ClauseForm,
    FRAGMENT_BASE,
    FRAGMENT_TASKS,
    ItemId,
    Level,
    PHASE_COUNT,
    STEP_COUNT,
    STEPS_PER_PHASE,
    SUB_TASKS_PER_TAIL,
    TASK_COUNT,
    TASKS_PER_STEP,
    old_anchor,
};
pub(crate) use oracle_model::Model;
use oracle_tree::{number_tree, post_edit_tree};

/// A structural edit applied to the generated model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Edit {
    /// Insert the fragment tasks before the addressed target task.
    InsertTasksBefore {
        /// Target task index, in generated source order.
        task: usize,
    },
    /// Delete the addressed target task and its addendum sub-tasks.
    DeleteTask {
        /// Target task index, in generated source order.
        task: usize,
    },
    /// Replace the addressed target task with the fragment tasks.
    ///
    /// The generated fragment holds two tasks, so the addressed identity is
    /// retired even though the second fragment task may reuse its number.
    ReplaceTask {
        /// Target task index, in generated source order.
        task: usize,
    },
    /// Replace the addressed target task with one task carrying its own anchor.
    ///
    /// The generated fragment reproduces the addressed task exactly, sub-tasks
    /// and clauses included. The splice therefore edits the item in place: the
    /// identity never goes away, so no clause may dangle.
    ReplaceTaskInPlace {
        /// Target task index, in generated source order.
        task: usize,
    },
    /// Delete the addressed step and every item beneath it.
    DeleteStep {
        /// Target step index, in generated source order.
        step: usize,
    },
    /// Delete the addressed phase and every item beneath it.
    DeletePhase {
        /// Target phase index, in generated source order.
        phase: usize,
    },
}

impl Edit {
    /// Convert a generated selector into a structural edit over the model.
    #[must_use]
    pub fn from_selector(selector: u8) -> Self {
        let index = usize::from(selector);
        match selector.rem_euclid(6) {
            0 => Self::InsertTasksBefore {
                task: index.rem_euclid(TASK_COUNT),
            },
            1 => Self::DeleteTask {
                task: index.rem_euclid(TASK_COUNT),
            },
            2 => Self::ReplaceTask {
                task: index.rem_euclid(TASK_COUNT),
            },
            3 => Self::DeleteStep {
                step: index.rem_euclid(STEP_COUNT),
            },
            4 => Self::DeletePhase {
                phase: index.rem_euclid(PHASE_COUNT),
            },
            _ => Self::ReplaceTaskInPlace {
                task: index.rem_euclid(TASK_COUNT),
            },
        }
    }

    /// Return the command-line anchor this edit addresses, in pre-edit numbering.
    #[must_use]
    pub fn anchor(self) -> String {
        match self {
            Self::InsertTasksBefore { task }
            | Self::DeleteTask { task }
            | Self::ReplaceTask { task }
            | Self::ReplaceTaskInPlace { task } => old_anchor(ItemId::new(Level::Task, task)),
            Self::DeleteStep { step } => old_anchor(ItemId::new(Level::Step, step)),
            Self::DeletePhase { phase } => old_anchor(ItemId::new(Level::Phase, phase)),
        }
    }
}

/// The expected outcome of applying an [`Edit`] to a [`Model`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Expectation {
    /// The edit must succeed and produce the supplied anchors per identity.
    Accept {
        /// New anchor for every surviving identity, by identity.
        anchors: BTreeMap<ItemId, String>,
        /// For each surviving edge, the consumer, its prerequisite, and the
        /// anchor the consumer's clause must carry: the prerequisite
        /// identity's **own** new anchor.
        rewritten: Vec<(ItemId, ItemId, String)>,
        /// Every anchor a removed identity used to hold. A rewritten clause
        /// must never carry one of these.
        retired: BTreeSet<String>,
    },
    /// The edit must be rejected because a surviving edge still requires an
    /// item the edit removes.
    Reject {
        /// Anchors of removed prerequisites that surviving consumers still
        /// require. The operation must report one of these.
        stalled: BTreeSet<String>,
    },
}

impl Expectation {
    /// Return the anchors per identity when the edit must succeed.
    #[must_use]
    pub const fn anchors(&self) -> Option<&BTreeMap<ItemId, String>> {
        match self {
            Self::Accept { anchors, .. } => Some(anchors),
            Self::Reject { .. } => None,
        }
    }

    /// Return the expected rewritten clauses when the edit must succeed.
    #[must_use]
    pub fn rewritten(&self) -> &[(ItemId, ItemId, String)] {
        match self {
            Self::Accept { rewritten, .. } => rewritten,
            Self::Reject { .. } => &[],
        }
    }

    /// Return the anchors of removed identities, when the edit must succeed.
    #[must_use]
    pub const fn retired(&self) -> Option<&BTreeSet<String>> {
        match self {
            Self::Accept { retired, .. } => Some(retired),
            Self::Reject { .. } => None,
        }
    }
}

/// Compute the expected outcome for one generated edit.
///
/// The expectation is derived from the model's structure alone, so the
/// production pipeline and this oracle can disagree. The rejection set is a
/// set rather than a single anchor because *which* stalled anchor the
/// diagnostic names depends on traversal order, an implementation detail; the
/// contract is that the rejected anchor is one of them.
#[must_use]
pub(crate) fn expectation_for(model: &Model, edit: Edit) -> Expectation {
    let tree = post_edit_tree(edit);
    let anchors = number_tree(&tree);
    let removed = tree.removed.clone();
    let retired = removed.iter().map(|id| old_anchor(*id)).collect();
    let stalled = model
        .edges
        .iter()
        .filter(|edge| !removed.contains(&edge.consumer) && removed.contains(&edge.prerequisite))
        .map(|edge| old_anchor(edge.prerequisite))
        .collect::<BTreeSet<_>>();

    if !stalled.is_empty() {
        return Expectation::Reject { stalled };
    }

    let rewritten = model
        .edges
        .iter()
        .filter(|edge| anchors.contains_key(&edge.consumer))
        .filter_map(|edge| {
            anchors
                .get(&edge.prerequisite)
                .map(|anchor| (edge.consumer, edge.prerequisite, anchor.clone()))
        })
        .collect();
    Expectation::Accept {
        anchors,
        rewritten,
        retired,
    }
}

#[cfg(test)]
mod tests {
    //! Tests for the selector vocabulary the property suite draws from.

    use super::Edit;

    /// Every variant is reachable, including both replacement forms.
    ///
    /// The property draws its edit selector from an arbitrary byte, so a
    /// variant the selector space cannot reach would make its half of the
    /// contract unreachable rather than merely rare.
    #[test]
    fn the_selector_space_reaches_every_edit_variant() {
        let edits = (0..6).map(Edit::from_selector).collect::<Vec<_>>();

        for (selector, expected) in [
            (0, Edit::InsertTasksBefore { task: 0 }),
            (1, Edit::DeleteTask { task: 1 }),
            (2, Edit::ReplaceTask { task: 2 }),
            (3, Edit::DeleteStep { step: 3 }),
            (4, Edit::DeletePhase { phase: 0 }),
            (5, Edit::ReplaceTaskInPlace { task: 5 }),
        ] {
            assert!(
                edits.contains(&expected),
                "selector {selector} should reach {expected:?}, got {edits:?}"
            );
        }
    }

    /// The in-place edit addresses the same pre-edit anchor as a retirement.
    #[test]
    fn the_in_place_edit_addresses_the_old_anchor() {
        let edit = Edit::ReplaceTaskInPlace { task: 5 };

        assert_eq!(
            edit.anchor(),
            "1.2.3",
            "task 5 is the third task of step 1.2, and that is the number the fragment must carry"
        );
    }
}
