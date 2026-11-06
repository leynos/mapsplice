//! Generated roadmap model and rendering for the layout property.

mod renderer;

use proptest::prelude::*;

pub(super) use self::renderer::render;

/// One normalized entry in a generated roadmap.
#[derive(Clone, Debug)]
pub(super) struct EntryModel {
    /// Leading label, or `None` when the entry is unlabelled.
    key: Option<String>,
    /// Inline value following the label.
    value: String,
}

/// One structural sub-task with body content and nested checkbox details.
#[derive(Clone, Debug)]
pub(super) struct DetailModel {
    /// Sub-task title.
    title: String,
    /// Whether the title carries a dependency clause.
    requires: bool,
    /// Ordered body entries.
    body: Vec<EntryModel>,
    /// Unnumbered checkbox details nested beneath this one.
    nested: Vec<Self>,
}

/// One task with body content and ordered structural sub-tasks.
#[derive(Clone, Debug)]
pub(super) struct TaskModel {
    /// Task title.
    title: String,
    /// Whether the task checkbox is done.
    is_done: bool,
    /// Whether the title carries a dependency clause.
    requires: bool,
    /// Ordered body entries.
    entries: Vec<EntryModel>,
    /// Ordered structural sub-tasks.
    sub_tasks: Vec<DetailModel>,
}

/// One step with an optional question and at least one task.
#[derive(Clone, Debug)]
struct StepModel {
    /// Step title.
    title: String,
    /// Optional first unlabelled paragraph.
    question: Option<String>,
    /// Ordered tasks.
    tasks: Vec<TaskModel>,
}

/// One phase with optional content and at least one step.
#[derive(Clone, Debug)]
pub(super) struct PhaseModel {
    /// Phase title.
    title: String,
    /// Optional phase body entry.
    entry: Option<EntryModel>,
    /// Ordered steps.
    steps: Vec<StepModel>,
}

/// Source layout knobs exercised by the equivalence property.
#[derive(Clone, Copy, Debug)]
pub(super) struct Layout {
    /// Line ending written between rendered lines.
    line_ending: &'static str,
    /// Indentation of a structural sub-task inside its task item.
    detail_indent: usize,
    /// Indentation of a task body paragraph.
    body_indent: usize,
    /// Maximum paragraph width, or `None` for unwrapped paragraphs.
    wrap_width: Option<usize>,
    /// Whether wrapped lines end in a two-space hard break.
    has_hard_breaks: bool,
    /// Number of spaces written between words in paragraphs.
    spaces_per_gap: usize,
}

/// Compact single-line paragraphs with minimal indentation.
pub(super) const COMPACT: Layout = Layout {
    line_ending: "\n",
    detail_indent: 2,
    body_indent: 2,
    wrap_width: None,
    has_hard_breaks: false,
    spaces_per_gap: 1,
};

/// Soft-wrapped paragraphs with hard breaks and wider indentation.
pub(super) const WRAPPED: Layout = Layout {
    line_ending: "\n",
    detail_indent: 2,
    body_indent: 4,
    wrap_width: Some(24),
    has_hard_breaks: true,
    spaces_per_gap: 1,
};

/// CRLF paragraphs with expanded word gaps and deeper indentation.
pub(super) const SPACED: Layout = Layout {
    line_ending: "\r\n",
    detail_indent: 4,
    body_indent: 5,
    wrap_width: Some(16),
    has_hard_breaks: false,
    spaces_per_gap: 3,
};

/// Generate a lowercase alphabetic word usable in titles and labels.
fn word() -> impl Strategy<Value = String> { "[a-z]{3,8}" }

/// Generate a short title made of alphabetic words.
fn title() -> impl Strategy<Value = String> {
    prop::collection::vec(word(), 1..4).prop_map(|words| words.join(" "))
}

/// Generate a short alphabetic label.
fn label() -> impl Strategy<Value = String> {
    prop::collection::vec(word(), 1..3).prop_map(|words| words.join(" "))
}

/// Generate an entry that may carry a label.
fn entry() -> impl Strategy<Value = EntryModel> {
    (any::<bool>(), label(), title()).prop_map(|(is_labelled, key, value)| EntryModel {
        key: is_labelled.then_some(key),
        value,
    })
}

/// Maximum number of checkbox-detail levels rendered beneath a sub-task.
const MAX_DETAIL_DEPTH: usize = 3;

/// Generate zero to two ordered body entries.
fn entries() -> impl Strategy<Value = Vec<EntryModel>> { prop::collection::vec(entry(), 0..3) }

/// Generate nested checkbox details down to `depth` further levels.
///
/// Unnumbered checkbox details may nest arbitrarily deep, so the generator
/// recurses to a fixed bound. The bound keeps generated documents small while
/// still exercising more than one level of nesting.
fn nested_details(depth: usize) -> BoxedStrategy<Vec<DetailModel>> {
    if depth == 0 {
        return Just(Vec::new()).boxed();
    }
    prop::collection::vec(nested_detail(depth), 0..3).boxed()
}

/// Generate one unnumbered checkbox detail that may nest further.
fn nested_detail(depth: usize) -> BoxedStrategy<DetailModel> {
    (
        title(),
        any::<bool>(),
        entries(),
        nested_details(depth.saturating_sub(1)),
    )
        .prop_map(|(title, requires, body, nested)| DetailModel {
            title,
            requires,
            body,
            nested,
        })
        .boxed()
}

/// Generate a structural sub-task with optional nested checkbox details.
fn detail() -> impl Strategy<Value = DetailModel> {
    (
        title(),
        any::<bool>(),
        entries(),
        prop::collection::vec(nested_detail(MAX_DETAIL_DEPTH - 1), 0..3),
    )
        .prop_map(|(title, requires, body, nested)| DetailModel {
            title,
            requires,
            body,
            nested,
        })
}

/// Generate a task with body entries and ordered structural sub-tasks.
fn task() -> impl Strategy<Value = TaskModel> {
    (
        title(),
        any::<bool>(),
        any::<bool>(),
        entries(),
        prop::collection::vec(detail(), 0..3),
    )
        .prop_map(|(title, is_done, requires, entries, sub_tasks)| TaskModel {
            title,
            is_done,
            requires,
            entries,
            sub_tasks,
        })
}

/// Generate a step with at least one task.
fn step() -> impl Strategy<Value = StepModel> {
    (
        title(),
        prop::option::of(title()),
        prop::collection::vec(task(), 1..3),
    )
        .prop_map(|(title, question, tasks)| StepModel {
            title,
            question,
            tasks,
        })
}

/// Generate a phase with at least one step.
fn phase() -> impl Strategy<Value = PhaseModel> {
    (
        title(),
        prop::option::of(entry()),
        prop::collection::vec(step(), 1..3),
    )
        .prop_map(|(title, entry, steps)| PhaseModel {
            title,
            entry,
            steps,
        })
}

/// Generate a roadmap with at least one phase.
pub(super) fn roadmap_model() -> impl Strategy<Value = Vec<PhaseModel>> {
    prop::collection::vec(phase(), 1..3)
}

/// Generate one or two ordered body entries.
fn structural_entries() -> impl Strategy<Value = Vec<EntryModel>> {
    prop::collection::vec(entry(), 1..3)
}

/// Generate a checkbox detail with no further nesting.
fn leaf_detail() -> impl Strategy<Value = DetailModel> {
    (title(), any::<bool>(), entries()).prop_map(|(title, requires, body)| DetailModel {
        title,
        requires,
        body,
        nested: Vec::new(),
    })
}

/// Generate a checkbox detail that owns exactly one deeper detail.
fn branch_detail() -> impl Strategy<Value = DetailModel> {
    (title(), any::<bool>(), entries(), leaf_detail()).prop_map(
        |(title, requires, body, nested)| DetailModel {
            title,
            requires,
            body,
            nested: vec![nested],
        },
    )
}

/// Generate a sub-task that owns two nested details, one of them recursive.
fn structural_detail() -> impl Strategy<Value = DetailModel> {
    (
        title(),
        any::<bool>(),
        entries(),
        prop::collection::vec(branch_detail(), 2..3),
    )
        .prop_map(|(title, requires, body, nested)| DetailModel {
            title,
            requires,
            body,
            nested,
        })
}

/// Generate a task with a body entry and two ordered sub-tasks.
fn structural_task() -> impl Strategy<Value = TaskModel> {
    (
        title(),
        any::<bool>(),
        any::<bool>(),
        structural_entries(),
        prop::collection::vec(structural_detail(), 2..3),
    )
        .prop_map(|(title, is_done, requires, entries, sub_tasks)| TaskModel {
            title,
            is_done,
            requires,
            entries,
            sub_tasks,
        })
}

/// Generate a step with two ordered tasks.
fn structural_step() -> impl Strategy<Value = StepModel> {
    (
        title(),
        prop::option::of(title()),
        prop::collection::vec(structural_task(), 2..3),
    )
        .prop_map(|(title, question, tasks)| StepModel {
            title,
            question,
            tasks,
        })
}

/// Generate a phase with at least one step.
fn structural_phase() -> impl Strategy<Value = PhaseModel> {
    (
        title(),
        prop::option::of(entry()),
        prop::collection::vec(structural_step(), 1..3),
    )
        .prop_map(|(title, entry, steps)| PhaseModel {
            title,
            entry,
            steps,
        })
}

/// Generate a roadmap whose first item of every level exists.
///
/// The structural drift cases address the first phase, step, task, and
/// sub-task, and need at least two tasks, two sub-tasks, two nested details,
/// and one body entry to add, remove, or permute. This generator guarantees
/// those shapes while still varying titles, checkbox states, labels, and
/// dependency clauses.
pub(super) fn structural_model() -> impl Strategy<Value = Vec<PhaseModel>> {
    prop::collection::vec(structural_phase(), 1..3)
}
