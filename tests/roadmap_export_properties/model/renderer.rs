//! Render a generated roadmap model as canonical Markdown.
//!
//! The renderer exists so the property tests can vary layout without changing
//! semantics: whitespace, wrapping, line endings, and indentation are knobs on
//! [`Layout`], while the document structure comes from the model generators.

use super::{DetailModel, EntryModel, Layout, PhaseModel, TaskModel};

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

/// Accumulates rendered lines under one layout.
struct Renderer {
    /// Rendered lines in document order.
    lines: Vec<String>,
    /// Layout applied to every rendered paragraph.
    layout: Layout,
}

impl Renderer {
    /// Start an empty renderer.
    const fn new(layout: Layout) -> Self {
        Self {
            lines: Vec::new(),
            layout,
        }
    }

    /// Render ordered body entries, each followed by a blank line.
    fn entries(&mut self, indent: usize, entries: &[EntryModel]) {
        for entry in entries {
            self.lines
                .extend(paragraph_lines(indent, &entry_text(entry), self.layout));
            self.lines.push(String::new());
        }
    }

    /// Render one task, its body entries, and its ordered sub-tasks.
    fn task(&mut self, position: TaskPosition, task: &TaskModel) {
        let marker = if task.is_done { "[x]" } else { "[ ]" };
        let anchor = position.anchor();
        self.lines.push(format!(
            "- {marker} {anchor}. {}{}",
            task.title,
            requires_text(task.requires)
        ));
        self.lines.push(String::new());
        self.entries(self.layout.body_indent, &task.entries);
        for (index, sub_task) in task.sub_tasks.iter().enumerate() {
            let sub_task_anchor = format!("{anchor}.{}", index + 1);
            self.detail(self.layout.detail_indent, Some(&sub_task_anchor), sub_task);
        }
    }

    /// Render a numbered or unnumbered detail and everything nested beneath it.
    ///
    /// Only the outermost sub-task carries an anchor; nested checkbox details
    /// are unnumbered and are matched by ordinal when drift is reported.
    fn detail(&mut self, indent: usize, anchor: Option<&str>, detail: &DetailModel) {
        let prefix = anchor.map_or_else(String::new, |label| format!("{label}. "));
        self.lines.push(format!(
            "{}- [ ] {prefix}{}{}",
            " ".repeat(indent),
            detail.title,
            requires_text(detail.requires)
        ));
        self.lines.push(String::new());
        self.entries(indent + 2, &detail.body);
        for nested in &detail.nested {
            self.detail(indent + 2, None, nested);
        }
    }

    /// Join the rendered lines with the requested line ending.
    fn finish(self) -> String {
        let mut rendered = self.lines.join(self.layout.line_ending);
        rendered.push_str(self.layout.line_ending);
        rendered
    }
}

/// Render a roadmap model as canonical Markdown under one layout.
pub(crate) fn render(model: &[PhaseModel], layout: Layout) -> String {
    let mut renderer = Renderer::new(layout);
    renderer.lines.push("# Example".to_owned());
    renderer.lines.push(String::new());
    for (phase_index, phase) in model.iter().enumerate() {
        let phase_number = phase_index + 1;
        renderer
            .lines
            .push(format!("## {phase_number}. {}", phase.title));
        renderer.lines.push(String::new());
        if let Some(entry) = &phase.entry {
            renderer
                .lines
                .extend(paragraph_lines(0, &entry_text(entry), layout));
            renderer.lines.push(String::new());
        }
        for (step_index, step) in phase.steps.iter().enumerate() {
            let step_number = step_index + 1;
            renderer
                .lines
                .push(format!("### {phase_number}.{step_number}. {}", step.title));
            renderer.lines.push(String::new());
            if let Some(question) = &step.question {
                renderer.lines.extend(paragraph_lines(0, question, layout));
                renderer.lines.push(String::new());
            }
            for (task_index, task) in step.tasks.iter().enumerate() {
                renderer.task(
                    TaskPosition {
                        phase: phase_number,
                        step: step_number,
                        task: task_index + 1,
                    },
                    task,
                );
            }
        }
    }
    renderer.finish()
}
