//! In-memory representation of a supported roadmap document.

use std::{fmt, sync::Arc};

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
#[path = "mixed_list_property_tests.rs"]
mod mixed_list_property_tests;
#[cfg(test)]
#[path = "model_property_tests.rs"]
mod property_tests;
#[path = "model_task_entry.rs"]
mod task_entry;
#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;

pub(crate) use super::renumber_plan::RenumberPlan;

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
    pub(super) tasks: Vec<TaskEntry>,
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
    pub(super) number: TaskNumber,
    /// Checkbox state, when present.
    pub(super) checked: Option<bool>,
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
    /// Original source and anchor identity for renumbering.
    pub(crate) identity: ItemIdentity,
    /// Current task number.
    pub(crate) number: TaskNumber,
    /// Parsed checklist state, when present.
    pub(crate) checked: Option<bool>,
    /// First paragraph after the task number.
    pub(crate) summary: MarkdownNodes,
    /// Additional Markdown blocks beneath the task.
    pub(crate) body: MarkdownNodes,
    /// Exact item source while its content remains unchanged.
    pub(crate) original_source: Option<String>,
    /// Structural sub-tasks nested beneath this task.
    pub(crate) sub_tasks: Vec<SubTaskEntry>,
    /// Body spans and sub-task identities in source order.
    pub(crate) children: Vec<TaskChild>,
}

/// Location for a sub-task splice in both structural task vectors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SubTaskSplice {
    /// Position in the task's structural sub-task vector.
    pub(crate) sub_task_index: usize,
    /// Matching position in the task's ordered child vector.
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
#[derive(Clone, Default)]
pub struct MarkdownNodes {
    /// Parsed Markdown nodes retained for rendering.
    nodes: Vec<Node>,
    /// Original source block for each aligned node, when available.
    original_blocks: Vec<Option<String>>,
    /// Document these nodes were parsed from, when known.
    ///
    /// Diagnostics that map an offset inside a text node back to the source
    /// need this: the parser normalizes soft line endings and strips their
    /// indentation, so the node's value alone cannot locate a token on a
    /// wrapped line.
    source: Option<Arc<str>>,
}

impl PartialEq for MarkdownNodes {
    /// Retained source text is provenance for diagnostics, not document identity.
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes && self.original_blocks == other.original_blocks
    }
}

impl Eq for MarkdownNodes {}

impl fmt::Debug for MarkdownNodes {
    /// Elide the retained document so failure output stays readable.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MarkdownNodes")
            .field("nodes", &self.nodes)
            .field("original_blocks", &self.original_blocks)
            .finish_non_exhaustive()
    }
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

impl RoadmapDocument {
    /// Create an empty document.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            preamble: MarkdownNodes::new(),
            phases: Vec::new(),
        }
    }

    /// Re-anchor every retained identity to the number its item now holds.
    ///
    /// An operation rewrites dependency references against a renumber plan
    /// keyed by the anchors items held when the operation started. Those
    /// anchors are stale as soon as the operation commits: the next operation
    /// reads the rendered numbers, so its plan must be keyed by them. Resetting
    /// each identity to its current number, and to the target source because
    /// spliced fragment content is now part of the target document, makes the
    /// committed document identical in identity terms to a fresh parse of its
    /// own rendering.
    ///
    /// This is what lets a caller apply successive operations without
    /// rendering and re-parsing between them. A continuity alias granted by an
    /// in-place replacement is therefore only needed for the operation that
    /// granted it; the re-anchored identity carries the continuation forward.
    pub(crate) fn reanchor_identities(&mut self) {
        for phase in &mut self.phases {
            phase.identity = identity_at(phase.number.into());
            for step in &mut phase.steps {
                reanchor_step(step);
            }
        }
    }
}

/// Return the target-sourced identity for an item's current anchor.
const fn identity_at(anchor: RoadmapAnchor) -> ItemIdentity {
    ItemIdentity {
        source: SourceId::Target,
        anchor,
    }
}

/// Re-anchor one step, its tasks, and their sub-tasks.
fn reanchor_step(step: &mut StepSection) {
    step.identity = identity_at(step.number.into());
    for task in &mut step.tasks {
        reanchor_task(task);
    }
}

/// Re-anchor a task and its sub-tasks, realigning structural child markers.
///
/// The parser and every splice keep exactly one [`TaskChild::SubTask`] marker
/// per sub-task, in the same order, so the marker sequence is updated by
/// position rather than by the old identity: duplicate fragment anchors are
/// possible before they are re-anchored to distinct numbers.
fn reanchor_task(task: &mut TaskEntry) {
    task.identity = identity_at(RoadmapAnchor::Task(task.number));
    for sub_task in &mut task.sub_tasks {
        sub_task.identity = identity_at(RoadmapAnchor::SubTask(sub_task.number));
    }

    let mut reanchored = task.sub_tasks.iter().map(|sub_task| sub_task.identity);
    for marker in task.children.iter_mut().filter_map(|child| match child {
        TaskChild::SubTask(identity) => Some(identity),
        TaskChild::Body(_) => None,
    }) {
        if let Some(identity) = reanchored.next() {
            *marker = identity;
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
            source: None,
        }
    }

    /// Push one parsed Markdown node while preserving its original source.
    pub(crate) fn push_preserved(&mut self, node: Node, source: &Arc<str>) {
        let original = original_block(&node, source);
        self.nodes.push(node);
        self.original_blocks.push(original);
        self.remember_source(source);
    }

    /// Construct Markdown nodes that remember the document they came from.
    ///
    /// Retaining the document is what lets a later diagnostic place a token on
    /// a wrapped line; rendering is unaffected. The handle is shared with every
    /// other collection from the same document, so this costs a pointer bump
    /// rather than another copy of the source.
    #[must_use]
    pub(crate) fn from_nodes_in(nodes: Vec<Node>, source: &Arc<str>) -> Self {
        Self {
            original_blocks: vec![None; nodes.len()],
            nodes,
            source: Some(Arc::clone(source)),
        }
    }

    /// Return the document these nodes were parsed from, when known.
    ///
    /// The handle is cloned rather than borrowed so a caller can hold it while
    /// mutating the nodes it came from.
    #[must_use]
    pub(crate) fn source(&self) -> Option<Arc<str>> { self.source.clone() }

    /// Record the owning document handle the first time a node arrives.
    fn remember_source(&mut self, source: &Arc<str>) {
        if self.source.is_none() {
            self.source = Some(Arc::clone(source));
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
