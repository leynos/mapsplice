//! Identity primitives for the dependency oracle.
//!
//! Generation counts, [`Level`], [`ItemId`], [`ClauseForm`], the positional
//! helpers that turn a position into an identity, and [`old_anchor`], which
//! gives an identity's spelling in the unmodified generated target.
//!
//! Numbers are *derived* here and nowhere stored: this is what lets the
//! oracle tell "the consumer's clause follows its prerequisite" apart from
//! "the consumer's clause still reads `1.1.1`, which the inserted task now
//! also holds".

/// Number of phases generated into the target roadmap.
pub(crate) const PHASE_COUNT: usize = 2;
/// Number of steps generated beneath each phase.
pub(crate) const STEPS_PER_PHASE: usize = 2;
/// Number of tasks generated beneath each step.
pub(crate) const TASKS_PER_STEP: usize = 3;
/// Number of addendum sub-tasks beneath each step's last task.
pub(crate) const SUB_TASKS_PER_TAIL: usize = 2;
/// Number of tasks generated into the fragment document.
pub(crate) const FRAGMENT_TASKS: usize = 2;

/// Derived target step count, used to size generated selector ranges.
pub(crate) const STEP_COUNT: usize = PHASE_COUNT * STEPS_PER_PHASE;
/// Derived target task count, used to size generated selector ranges.
pub(crate) const TASK_COUNT: usize = STEP_COUNT * TASKS_PER_STEP;

/// Identity indices at or above this base belong to the fragment document.
pub(crate) const FRAGMENT_BASE: usize = 1000;

/// Structural level of a generated roadmap item.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Level {
    /// A `## n.` phase heading.
    Phase,
    /// A `### n.m.` step heading.
    Step,
    /// A `- [ ] n.m.k.` task.
    Task,
    /// A `- [ ] n.m.k.j.` addendum sub-task.
    SubTask,
}

/// Stable identity for one generated roadmap item.
///
/// The identity is a generation-time token, not a number: it is what a
/// dependency edge is *about*, and it must survive an edit that gives every
/// item a different rendered number.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ItemId {
    /// Structural level of the item.
    pub level: Level,
    /// Index within the item's own level, in generated source order.
    pub index: usize,
}

impl ItemId {
    /// Create an identity for one item.
    #[must_use]
    pub const fn new(level: Level, index: usize) -> Self { Self { level, index } }

    /// Create an identity for one fragment task.
    #[must_use]
    pub const fn fragment_task(index: usize) -> Self {
        Self::new(Level::Task, FRAGMENT_BASE + index)
    }

    /// Return whether this identity belongs to the fragment document.
    #[must_use]
    pub const fn is_fragment(self) -> bool { self.index >= FRAGMENT_BASE }
}

/// How one generated clause is spelled in the source text.
///
/// Only representations that `docs/mapsplice-design.md` section 7 recognizes
/// are generated, so every one of them carries the same rewrite and
/// dangling-dependency guarantees. A clause split across a hard line wrap and a
/// numeric range are excluded by the grammar and are deliberately never
/// generated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClauseForm {
    /// The clause shares the numbered summary line.
    Inline,
    /// The clause begins an indented continuation line.
    Continuation,
    /// The clause begins a nested bullet item in the task body.
    NestedBullet,
    /// The clause begins a nested bullet item in an addendum sub-task's body.
    NestedSubTaskBullet,
}

impl ClauseForm {
    /// Convert a generated selector into a clause representation.
    #[must_use]
    pub const fn from_selector(selector: u8) -> Self {
        match selector.rem_euclid(4) {
            0 => Self::Inline,
            1 => Self::Continuation,
            2 => Self::NestedBullet,
            _ => Self::NestedSubTaskBullet,
        }
    }
}

/// Return the step owning a sub-task identity.
pub(super) const fn sub_task_owner(sub_task: usize) -> usize {
    sub_task.div_euclid(SUB_TASKS_PER_TAIL)
}

/// Return the identity of sub-task `position` beneath `step`'s tail task.
pub(super) const fn sub_task_of_step(step: usize, position: usize) -> ItemId {
    ItemId::new(
        Level::SubTask,
        step * SUB_TASKS_PER_TAIL + position.rem_euclid(SUB_TASKS_PER_TAIL),
    )
}

/// Return the identity of task `position` within `step`.
pub(super) const fn task_in_step(step: usize, position: usize) -> ItemId {
    ItemId::new(Level::Task, step * TASKS_PER_STEP + position)
}

/// Return the identity of the step owning task `task`.
pub(super) const fn step_of_task(task: usize) -> usize { task.div_euclid(TASKS_PER_STEP) }

/// Return whether `task` is the last task of its step, and so owns sub-tasks.
pub(super) const fn is_tail_task(task: usize) -> bool {
    task.rem_euclid(TASKS_PER_STEP) == TASKS_PER_STEP - 1
}

/// Return the addendum identities belonging to one task index.
pub(super) fn sub_tasks_of_task(task: usize) -> Vec<ItemId> {
    if !is_tail_task(task) {
        return Vec::new();
    }
    let step = step_of_task(task);
    (0..SUB_TASKS_PER_TAIL)
        .map(|position| sub_task_of_step(step, position))
        .collect()
}

/// The anchor an identity holds in the *unmodified* generated target.
///
/// Fragment identities are numbered from `1` because a fragment is a standalone
/// document whose own numbering the target numbering replaces on splice.
#[must_use]
pub(crate) fn old_anchor(id: ItemId) -> String {
    match id.level {
        Level::Phase => (id.index + 1).to_string(),
        Level::Step => {
            let phase = id.index.div_euclid(STEPS_PER_PHASE);
            let offset = id.index.rem_euclid(STEPS_PER_PHASE);
            format!("{}.{}", phase + 1, offset + 1)
        }
        Level::Task if id.is_fragment() => (id.index - FRAGMENT_BASE + 1).to_string(),
        Level::Task => {
            let position = id.index.rem_euclid(TASKS_PER_STEP);
            let step = old_anchor(ItemId::new(Level::Step, step_of_task(id.index)));
            format!("{step}.{}", position + 1)
        }
        Level::SubTask => {
            let position = id.index.rem_euclid(SUB_TASKS_PER_TAIL);
            let step = id.index.div_euclid(SUB_TASKS_PER_TAIL);
            let task = task_in_step(step, TASKS_PER_STEP - 1);
            format!("{}.{}", old_anchor(task), position + 1)
        }
    }
}
