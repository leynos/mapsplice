//! Generated dependency graph, held separately from the rendered text.
//!
//! An [`Edge`] is a relation between two [`ItemId`]s, never between two
//! spellings, so a clause whose number changes under an edit still names the
//! same prerequisite. [`Model::generate`] builds the graph from the same
//! selectors that drive rendering, but through no shared code path.

use super::oracle_identity::{
    ClauseForm,
    ItemId,
    Level,
    STEP_COUNT,
    SUB_TASKS_PER_TAIL,
    TASKS_PER_STEP,
    step_of_task,
    sub_task_of_step,
    sub_task_owner,
    task_in_step,
};

/// One dependency edge over stable identities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Edge {
    /// The item whose clause names the prerequisite.
    pub consumer: ItemId,
    /// The item the clause requires.
    pub prerequisite: ItemId,
    /// How the clause is spelled in the generated target.
    pub form: ClauseForm,
}

/// A generated dependency graph held separately from the generated text.
#[derive(Clone, Debug, Default)]
pub(crate) struct Model {
    /// Every generated edge, in generation order.
    pub edges: Vec<Edge>,
}

/// Selector bits naming the consumer task within its step.
const CONSUMER_MASK: u8 = 0b0000_0011;
/// Selector bits naming the prerequisite kind.
const PREREQUISITE_MASK: u8 = 0b0000_1100;
/// Selector shift for the prerequisite field.
const PREREQUISITE_SHIFT: u8 = 2;
/// Selector flag making the consumer a sub-task rather than a task.
const SUB_TASK_CONSUMER: u8 = 0b0100_0000;
/// Selector flag suppressing this step's edge entirely.
const OMIT_EDGE: u8 = 0b1000_0000;
/// Selector shift for the clause representation field.
const CLAUSE_SHIFT: u8 = 4;

impl Model {
    /// Build a model from one selector per generated step.
    ///
    /// Each selector contributes at most one edge. A selector whose
    /// prerequisite resolves to the consumer's own identity is dropped: a
    /// self-dependency is not a meaningful roadmap edge, and generating one
    /// would make the expected outcome ambiguous rather than adversarial.
    ///
    /// The consumer is a task of the step, or — with [`SUB_TASK_CONSUMER`] —
    /// one of the step's tail task's addendum sub-tasks. Both are clause-
    /// bearing containers, so both must carry the same rewrite and
    /// dangling-dependency guarantees.
    #[must_use]
    pub fn generate(selectors: &[u8]) -> Self {
        let mut edges = Vec::new();
        for (step, selector) in selectors.iter().enumerate().take(STEP_COUNT) {
            if selector & OMIT_EDGE != 0 {
                continue;
            }
            let position = usize::from(selector & CONSUMER_MASK).rem_euclid(TASKS_PER_STEP);
            let consumer = if selector & SUB_TASK_CONSUMER != 0 {
                sub_task_of_step(step, position.rem_euclid(SUB_TASKS_PER_TAIL))
            } else {
                task_in_step(step, position)
            };
            if let Some(edge) = prerequisite_for(step, *selector, consumer) {
                edges.push(edge);
            }
        }
        Self { edges }
    }

    /// Return the edges whose consumer is `consumer`.
    #[must_use]
    pub fn edges_from(&self, consumer: ItemId) -> Vec<&Edge> {
        self.edges
            .iter()
            .filter(|edge| edge.consumer == consumer)
            .collect()
    }
}

/// Build the prerequisite edge selected by `selector` for `consumer`.
///
/// The four prerequisite kinds deliberately span the anchor depths the
/// resolver must handle: a same-level task, a deeper sub-task, a shallower
/// step, and a task in a different step that only resolves as a unique
/// cross-source anchor.
fn prerequisite_for(step: usize, selector: u8, consumer: ItemId) -> Option<Edge> {
    let kind = (selector & PREREQUISITE_MASK) >> PREREQUISITE_SHIFT;
    let form = ClauseForm::from_selector(selector >> CLAUSE_SHIFT);
    let prerequisite = match kind {
        // The next task in the same step, a same-level anchor.
        0 => task_in_step(
            step,
            (consumer_of(consumer).index + 1).rem_euclid(TASKS_PER_STEP),
        ),
        // This step's tail task's first addendum sub-task, a deeper anchor.
        1 => sub_task_of_step(step, 0),
        // The enclosing step, a shallower two-level anchor.
        2 => ItemId::new(Level::Step, step),
        // The first task of the next step, resolvable only as a unique
        // cross-source anchor.
        _ => task_in_step((step + 1).rem_euclid(STEP_COUNT), 0),
    };
    if prerequisite == consumer {
        None
    } else {
        Some(Edge {
            consumer,
            prerequisite,
            form,
        })
    }
}

/// Return the task a consumer belongs to, whether it is a task or a sub-task.
const fn consumer_of(consumer: ItemId) -> ItemId {
    match consumer.level {
        Level::SubTask => task_in_step(
            step_of_task(sub_task_owner(consumer.index)),
            TASKS_PER_STEP - 1,
        ),
        _ => consumer,
    }
}
