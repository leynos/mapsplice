//! Generated roadmap model and rendering for the layout property.

use proptest::prelude::*;

/// One normalized entry in a generated roadmap.
#[derive(Clone, Debug)]
struct EntryModel {
    /// Leading label, or `None` when the entry is unlabelled.
    key: Option<String>,
    /// Inline value following the label.
    value: String,
}

/// One structural sub-task with body content and nested checkbox details.
#[derive(Clone, Debug)]
struct DetailModel {
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
struct TaskModel {
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

/// Generate zero to two ordered body entries.
fn entries() -> impl Strategy<Value = Vec<EntryModel>> { prop::collection::vec(entry(), 0..3) }

/// Generate a nested checkbox detail, which cannot own numbered details.
fn nested_detail() -> impl Strategy<Value = DetailModel> {
    (title(), any::<bool>(), entries()).prop_map(|(title, requires, body)| DetailModel {
        title,
        requires,
        body,
        nested: Vec::new(),
    })
}

/// Generate a structural sub-task with optional nested checkbox details.
fn detail() -> impl Strategy<Value = DetailModel> {
    (
        title(),
        any::<bool>(),
        entries(),
        prop::collection::vec(nested_detail(), 0..2),
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

/// Split text into wrap tokens, keeping each word's colon attached to it.
///
/// A label may span several words, and the exporter normalizes whitespace
/// inside a label before recognizing it. Allowing a wrap between label words
/// exercises that normalization: the split form must export identically to the
/// single-line form.
fn wrap_tokens(text: &str) -> Vec<String> { text.split_whitespace().map(str::to_owned).collect() }

/// Soft-wrap text at token boundaries.
fn wrap(text: &str, width: usize, gap: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for token in wrap_tokens(text) {
        let separator = if current.is_empty() { "" } else { gap };
        if !current.is_empty() && current.len() + separator.len() + token.len() > width {
            lines.push(std::mem::take(&mut current));
            current.push_str(&token);
        } else {
            current.push_str(separator);
            current.push_str(&token);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Render one paragraph under an indentation with the requested layout.
fn paragraph_lines(indent: usize, text: &str, layout: Layout) -> Vec<String> {
    let gap = " ".repeat(layout.spaces_per_gap);
    let mut lines = layout.wrap_width.map_or_else(
        || vec![wrap_tokens(text).join(&gap)],
        |width| wrap(text, width, &gap),
    );
    if layout.has_hard_breaks {
        let last = lines.len().saturating_sub(1);
        for (index, line) in lines.iter_mut().enumerate() {
            if index < last {
                line.push_str("  ");
            }
        }
    }
    let pad = " ".repeat(indent);
    lines
        .into_iter()
        .map(|line| format!("{pad}{line}"))
        .collect()
}

/// Render an entry as paragraph text.
fn entry_text(entry: &EntryModel) -> String {
    entry.key.as_ref().map_or_else(
        || entry.value.clone(),
        |key| format!("{key}: {}", entry.value),
    )
}

/// Position of a task within the rendered roadmap.
#[derive(Clone, Copy)]
struct TaskPosition {
    /// One-based phase number.
    phase: usize,
    /// One-based step number within the phase.
    step: usize,
    /// One-based task number within the step.
    task: usize,
}

impl TaskPosition {
    /// Format the dotted anchor prefix for this task.
    fn anchor(self) -> String { format!("{}.{}.{}", self.phase, self.step, self.task) }
}

/// Render a dependency clause, or nothing when the item has none.
const fn requires_text(has_requires: bool) -> &'static str {
    if has_requires { " Requires 1.1.1." } else { "" }
}

/// Render ordered body entries followed by a blank line each.
fn render_entries(lines: &mut Vec<String>, indent: usize, entries: &[EntryModel], layout: Layout) {
    for entry in entries {
        lines.extend(paragraph_lines(indent, &entry_text(entry), layout));
        lines.push(String::new());
    }
}

/// Render one task, its body entries, and its ordered sub-tasks.
fn render_task(lines: &mut Vec<String>, position: TaskPosition, task: &TaskModel, layout: Layout) {
    let marker = if task.is_done { "[x]" } else { "[ ]" };
    let anchor = position.anchor();
    lines.push(format!(
        "- {marker} {anchor}. {}{}",
        task.title,
        requires_text(task.requires)
    ));
    lines.push(String::new());
    render_entries(lines, layout.body_indent, &task.entries, layout);
    for (index, sub_task) in task.sub_tasks.iter().enumerate() {
        render_detail(
            lines,
            DetailPosition {
                indent: layout.detail_indent,
                anchor: &format!("{anchor}.{}", index + 1),
            },
            sub_task,
            layout,
        );
    }
}

/// Indentation and anchor prefix for one numbered detail.
#[derive(Clone, Copy)]
struct DetailPosition<'anchor> {
    /// Leading spaces before the list marker.
    indent: usize,
    /// Dotted anchor prefix written before the title.
    anchor: &'anchor str,
}

/// Render a numbered detail and any unnumbered details nested beneath it.
fn render_detail(
    lines: &mut Vec<String>,
    position: DetailPosition<'_>,
    detail: &DetailModel,
    layout: Layout,
) {
    let indent = position.indent;
    let anchor = position.anchor;
    lines.push(format!(
        "{}- [ ] {anchor}. {}{}",
        " ".repeat(indent),
        detail.title,
        requires_text(detail.requires)
    ));
    lines.push(String::new());
    render_entries(lines, indent + 2, &detail.body, layout);
    for nested in &detail.nested {
        lines.push(format!(
            "{}- [ ] {}{}",
            " ".repeat(indent + 2),
            nested.title,
            requires_text(nested.requires)
        ));
        lines.push(String::new());
        render_entries(lines, indent + 4, &nested.body, layout);
    }
}

/// Render a roadmap model as canonical Markdown under one layout.
pub(super) fn render(model: &[PhaseModel], layout: Layout) -> String {
    let mut lines = vec!["# Example".to_owned(), String::new()];
    for (phase_index, phase) in model.iter().enumerate() {
        let phase_number = phase_index + 1;
        lines.push(format!("## {phase_number}. {}", phase.title));
        lines.push(String::new());
        if let Some(entry) = &phase.entry {
            lines.extend(paragraph_lines(0, &entry_text(entry), layout));
            lines.push(String::new());
        }
        for (step_index, step) in phase.steps.iter().enumerate() {
            let step_number = step_index + 1;
            lines.push(format!("### {phase_number}.{step_number}. {}", step.title));
            lines.push(String::new());
            if let Some(question) = &step.question {
                lines.extend(paragraph_lines(0, question, layout));
                lines.push(String::new());
            }
            for (task_index, task) in step.tasks.iter().enumerate() {
                render_task(
                    &mut lines,
                    TaskPosition {
                        phase: phase_number,
                        step: step_number,
                        task: task_index + 1,
                    },
                    task,
                    layout,
                );
            }
        }
    }
    let mut rendered = lines.join(layout.line_ending);
    rendered.push_str(layout.line_ending);
    rendered
}
