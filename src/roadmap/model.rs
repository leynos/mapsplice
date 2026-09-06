//! In-memory representation of a supported roadmap document.

use std::collections::BTreeMap;

use markdown::mdast::Node;

use super::{
    RoadmapAnchor,
    RoadmapItemLevel,
    StepNumber,
    SubTaskNumber,
    TaskNumber,
    source_preservation::original_node_source,
};
use crate::error::{MapspliceError, Result};

#[cfg(test)]
#[path = "model_property_tests.rs"]
mod property_tests;
#[path = "model_task_entry.rs"]
mod task_entry;
#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;

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
    /// Exact original task list source while every task remains unchanged.
    pub(crate) task_list_source: Option<String>,
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
    /// Exact list-item source while this task and its descendants are unchanged.
    pub(crate) original_source: Option<String>,
    /// Ordered fourth-level sub-tasks nested beneath this task.
    sub_tasks: Vec<SubTaskEntry>,
    /// Original ordered child sequence beneath this task.
    children: Vec<TaskChild>,
}

/// Parser-owned parts required to build one task entry.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct TaskEntryParts {
    pub(crate) identity: ItemIdentity,
    pub(crate) number: TaskNumber,
    pub(crate) checked: Option<bool>,
    pub(crate) summary: MarkdownNodes,
    pub(crate) body: MarkdownNodes,
    pub(crate) original_source: Option<String>,
    pub(crate) sub_tasks: Vec<SubTaskEntry>,
    pub(crate) children: Vec<TaskChild>,
}

/// Location for a sub-task splice in both structural task vectors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SubTaskSplice {
    pub(crate) sub_task_index: usize,
    pub(crate) child_index: usize,
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
    /// Exact list-item source while this sub-task and its descendants are unchanged.
    pub(crate) original_source: Option<String>,
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
    by_source: BTreeMap<SourceId, BTreeMap<RoadmapAnchor, RoadmapAnchor>>,
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

/// Return the exact source span for a Markdown node when the parser recorded one.
fn original_block(n: &Node, s: &str) -> Option<String> { original_node_source(n, s) }

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

    pub(crate) fn record_mapping(
        &mut self,
        source: SourceId,
        old: RoadmapAnchor,
        new: RoadmapAnchor,
    ) {
        self.by_source.entry(source).or_default().insert(old, new);
    }
}
