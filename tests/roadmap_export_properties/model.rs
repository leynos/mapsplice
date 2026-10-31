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

/// One structural sub-task with optional body content.
#[derive(Clone, Debug)]
struct DetailModel {
    /// Sub-task title.
    title: String,
    /// Optional body entry.
    body: Option<EntryModel>,
}

/// One task with optional body content and a structural sub-task.
#[derive(Clone, Debug)]
struct TaskModel {
    /// Task title.
    title: String,
    /// Whether the task checkbox is done.
    is_done: bool,
    /// Optional task body entry.
    body: Option<EntryModel>,
    /// Optional structural sub-task.
    sub_task: Option<DetailModel>,
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

/// Generate a structural sub-task.
fn detail() -> impl Strategy<Value = DetailModel> {
    (title(), prop::option::of(entry())).prop_map(|(title, body)| DetailModel { title, body })
}

/// Generate a task with optional body content and sub-task.
fn task() -> impl Strategy<Value = TaskModel> {
    (
        title(),
        any::<bool>(),
        prop::option::of(entry()),
        prop::option::of(detail()),
    )
        .prop_map(|(title, is_done, body, sub_task)| TaskModel {
            title,
            is_done,
            body,
            sub_task,
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

/// Split text into wrap tokens whose labels stay attached to their colon.
///
/// A label is only recognized when it contains no line break, so wrapping
/// between the words of a label would legitimately change the export. Keeping
/// each colon-terminated label in one token models the same semantics under a
/// tighter layout rather than asserting that layout can be arbitrary.
fn wrap_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut pending = Vec::new();
    for word in text.split_whitespace() {
        pending.push(word);
        if word.ends_with(':') {
            tokens.push(pending.join(" "));
            pending.clear();
        }
    }
    if !pending.is_empty() {
        tokens.push(pending.join(" "));
    }
    tokens
}

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

/// Render one task, its body, and its structural sub-task.
fn render_task(lines: &mut Vec<String>, position: TaskPosition, task: &TaskModel, layout: Layout) {
    let marker = if task.is_done { "[x]" } else { "[ ]" };
    let anchor = position.anchor();
    lines.push(format!("- {marker} {anchor}. {}", task.title));
    lines.push(String::new());
    if let Some(body) = &task.body {
        lines.extend(paragraph_lines(
            layout.body_indent,
            &entry_text(body),
            layout,
        ));
        lines.push(String::new());
    }
    if let Some(sub_task) = &task.sub_task {
        lines.push(format!(
            "{}- [ ] {anchor}.1. {}",
            " ".repeat(layout.detail_indent),
            sub_task.title
        ));
        lines.push(String::new());
        if let Some(body) = &sub_task.body {
            lines.extend(paragraph_lines(
                layout.detail_indent + 2,
                &entry_text(body),
                layout,
            ));
            lines.push(String::new());
        }
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
