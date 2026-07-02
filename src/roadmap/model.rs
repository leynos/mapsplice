//! In-memory representation of a supported roadmap document.

use std::collections::BTreeMap;

use markdown::mdast::Node;

use super::{RoadmapAnchor, RoadmapItemLevel, StepNumber, SubTaskNumber, TaskNumber};

/// A parsed roadmap document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoadmapDocument {
    /// Preamble blocks before the first phase.
    pub preamble: MarkdownNodes,
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
    /// Addendum sub-task-level fragment.
    SubTask(Vec<SubTaskEntry>),
}

/// A phase section headed by `## <n>.`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhaseSection {
    /// Original identity for renumber tracking.
    pub identity: ItemIdentity,
    /// Current rendered phase number.
    pub number: super::PhaseNumber,
    /// Title nodes after the numeric prefix.
    pub title: MarkdownNodes,
    /// Blocks between the phase heading and first step.
    pub body: MarkdownNodes,
    /// Ordered steps within the phase.
    pub steps: Vec<StepSection>,
    /// Blocks after the last step.
    pub trailing: MarkdownNodes,
}

/// A step section headed by `### <n>.<m>.`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepSection {
    /// Original identity for renumber tracking.
    pub identity: ItemIdentity,
    /// Current rendered step number.
    pub number: StepNumber,
    /// Title nodes after the numeric prefix.
    pub title: MarkdownNodes,
    /// Blocks between the step heading and first task list.
    pub body: MarkdownNodes,
    /// Ordered tasks within the step.
    pub tasks: Vec<TaskEntry>,
    /// Blocks after the last task.
    pub trailing: MarkdownNodes,
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
    pub summary: MarkdownNodes,
    /// Additional blocks nested beneath the task.
    pub body: MarkdownNodes,
    /// Ordered fourth-level sub-tasks nested beneath this task.
    pub sub_tasks: Vec<SubTaskEntry>,
    /// Original ordered child sequence beneath this task.
    pub children: Vec<TaskChild>,
}

/// One ordered child beneath a numbered task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaskChild {
    /// Non-structural Markdown body blocks.
    Body(MarkdownNodes),
    /// A first-class sub-task identified by its original identity.
    SubTask(ItemIdentity),
}

/// A numbered sub-task list item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubTaskEntry {
    /// Original identity for renumber tracking.
    pub identity: ItemIdentity,
    /// Current rendered sub-task number.
    pub number: SubTaskNumber,
    /// Checkbox state, when present.
    pub checked: Option<bool>,
    /// First paragraph content after the numeric prefix.
    pub summary: MarkdownNodes,
    /// Additional blocks nested beneath the sub-task.
    pub body: MarkdownNodes,
}

/// Roadmap-owned Markdown nodes kept behind the parse/render boundary.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MarkdownNodes {
    nodes: Vec<Node>,
    original_blocks: Vec<Option<String>>,
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
            preamble: MarkdownNodes::new(),
            phases: Vec::new(),
        }
    }
}

impl MarkdownNodes {
    /// Create an empty roadmap Markdown node collection.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            original_blocks: Vec::new(),
        }
    }

    /// Push one parsed Markdown node while preserving its original source.
    pub(crate) fn push_preserved(&mut self, node: Node, source: &str) {
        let original = original_block(&node, source);
        self.nodes.push(node);
        self.original_blocks.push(original);
    }

    /// Construct Markdown nodes from parser-owned mdast nodes.
    #[must_use]
    pub(crate) fn from_nodes(nodes: Vec<Node>) -> Self {
        Self {
            original_blocks: vec![None; nodes.len()],
            nodes,
        }
    }

    /// Return whether the collection has no Markdown nodes.
    #[must_use]
    pub const fn is_empty(&self) -> bool { self.nodes.is_empty() }

    /// Return the number of Markdown nodes in the collection.
    #[must_use]
    pub const fn len(&self) -> usize { self.nodes.len() }

    /// Return the contained nodes for parse/render adapters.
    #[must_use]
    pub(crate) fn nodes(&self) -> &[Node] { &self.nodes }

    /// Return original source blocks for parse/render adapters.
    #[must_use]
    pub(crate) fn original_blocks(&self) -> &[Option<String>] { &self.original_blocks }

    /// Return mutable contained nodes for parse/render adapters.
    pub(crate) fn nodes_mut(&mut self) -> &mut [Node] { &mut self.nodes }

    /// Clear preserved source snippets after a semantic node rewrite.
    pub(crate) fn clear_original_blocks(&mut self) { self.original_blocks.fill(None); }
}

/// Copy the exact source span for an unchanged Markdown node.
fn original_block(node: &Node, source: &str) -> Option<String> {
    let position = node.position()?;
    let prefix = source.get(..position.start.offset)?;
    let start = prefix.rfind('\n').map_or(0, |index| index + 1);
    source.get(start..position.end.offset).map(str::to_owned)
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
            Self::SubTask(_) => RoadmapItemLevel::SubTask,
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
