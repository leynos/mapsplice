//! In-memory representation of a supported roadmap document.

use std::collections::BTreeMap;

use markdown::mdast::Node;

use super::{RoadmapAnchor, RoadmapItemLevel, StepNumber, TaskNumber};

/// A parsed roadmap document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoadmapDocument {
    /// Preamble blocks before the first phase.
    pub preamble: Vec<Node>,
    /// Ordered phase sections.
    pub phases: Vec<PhaseSection>,
}

/// A fragment file parsed at a single structural level.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RoadmapFragment {
    /// Phase-level fragment.
    Phase(Vec<PhaseSection>),
    /// Step-level fragment.
    Step(Vec<StepSection>),
    /// Task-level fragment.
    Task(Vec<TaskEntry>),
}

/// A phase section headed by `## <n>.`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhaseSection {
    /// Original identity for renumber tracking.
    pub identity: ItemIdentity,
    /// Current rendered phase number.
    pub number: super::PhaseNumber,
    /// Title nodes after the numeric prefix.
    pub title: Vec<Node>,
    /// Blocks between the phase heading and first step.
    pub body: Vec<Node>,
    /// Ordered steps within the phase.
    pub steps: Vec<StepSection>,
    /// Blocks after the last step.
    pub trailing: Vec<Node>,
}

/// A step section headed by `### <n>.<m>.`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepSection {
    /// Original identity for renumber tracking.
    pub identity: ItemIdentity,
    /// Current rendered step number.
    pub number: StepNumber,
    /// Title nodes after the numeric prefix.
    pub title: Vec<Node>,
    /// Blocks between the step heading and first task list.
    pub body: Vec<Node>,
    /// Ordered tasks within the step.
    pub tasks: Vec<TaskEntry>,
    /// Blocks after the last task.
    pub trailing: Vec<Node>,
}

/// A numbered task list item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskEntry {
    /// Original identity for renumber tracking.
    pub identity: ItemIdentity,
    /// Current rendered task number.
    pub number: TaskNumber,
    /// Checkbox state, when present.
    pub checked: Option<bool>,
    /// First paragraph content after the numeric prefix.
    pub summary: Vec<Node>,
    /// Additional blocks nested beneath the task.
    pub body: Vec<Node>,
}

/// Where a parsed item came from.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceId {
    /// Item parsed from the target roadmap.
    Target,
    /// Item parsed from the fragment file.
    Fragment,
}

/// Stable identity used while renumbering.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ItemIdentity {
    /// Source document.
    pub source: SourceId,
    /// Original anchor in that document.
    pub anchor: RoadmapAnchor,
}

/// Old-to-new renumbering lookup.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RenumberPlan {
    /// Mapping by source document.
    pub by_source: BTreeMap<SourceId, BTreeMap<RoadmapAnchor, RoadmapAnchor>>,
}

impl RoadmapDocument {
    /// Create an empty document.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            preamble: Vec::new(),
            phases: Vec::new(),
        }
    }
}

impl Default for RoadmapDocument {
    fn default() -> Self { Self::new() }
}

impl RoadmapFragment {
    /// Return the fragment level.
    #[must_use]
    pub const fn level(&self) -> RoadmapItemLevel {
        match self {
            Self::Phase(_) => RoadmapItemLevel::Phase,
            Self::Step(_) => RoadmapItemLevel::Step,
            Self::Task(_) => RoadmapItemLevel::Task,
        }
    }
}

/// Return the fragment level without consuming the fragment.
#[must_use]
pub const fn fragment_level(fragment: &RoadmapFragment) -> RoadmapItemLevel { fragment.level() }

impl RenumberPlan {
    /// Resolve a rewritten anchor for the given source and original anchor.
    #[must_use]
    pub fn resolve(&self, source: SourceId, anchor: RoadmapAnchor) -> Option<RoadmapAnchor> {
        self.by_source
            .get(&source)
            .and_then(|mapping| mapping.get(&anchor).copied())
    }

    /// Resolve a unique mapping across all sources when local lookup is absent.
    #[must_use]
    pub fn resolve_unique(&self, anchor: RoadmapAnchor) -> Option<RoadmapAnchor> {
        let mut matches = self
            .by_source
            .values()
            .filter_map(|mapping| mapping.get(&anchor).copied());
        let first = matches.next()?;
        if matches.next().is_some() {
            None
        } else {
            Some(first)
        }
    }

    pub(crate) fn insert(&mut self, source: SourceId, old: RoadmapAnchor, new: RoadmapAnchor) {
        self.by_source.entry(source).or_default().insert(old, new);
    }
}
