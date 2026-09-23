//! Render generated roadmaps and fragments whose dependencies are modelled.
//!
//! Rendering takes the [`Model`] alongside the generation parameters, so the
//! text and the expected dependency graph are produced from the same selectors
//! but through different code paths: the graph in [`super::oracle`], the bytes
//! here. That separation is what makes the oracle an independent check rather
//! than a restatement of the generator.

use std::fmt::{self, Write as _};

use super::oracle::{
    ClauseForm,
    FRAGMENT_TASKS,
    ItemId,
    Level,
    Model,
    PHASE_COUNT,
    STEPS_PER_PHASE,
    SUB_TASKS_PER_TAIL,
    TASKS_PER_STEP,
    old_anchor,
};

/// Byte-level shape of the generated Markdown.
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    /// Bit field selecting optional valid document features.
    pub options: u8,
}

impl Shape {
    /// Begin the document with a preamble before the first phase.
    pub const PREAMBLE: u8 = 1;
    /// Add a thematic break and trailing prose after the last phase.
    ///
    /// The trailing block uses a thematic break rather than a heading: a
    /// non-roadmap heading *inside* the roadmap body is rejected, so a `##`
    /// section here would make the generated document invalid rather than
    /// merely decorated.
    pub const TRAILING_SECTION: u8 = 2;
    /// Wrap one task's unrelated prose across a hard line break.
    pub const WRAPPED_PROSE: u8 = 4;
    /// Give every task a distinct checkbox state, including unchecked.
    pub const CHECKBOX_STATES: u8 = 8;
    /// Indent sub-tasks by four spaces rather than two.
    pub const DEEP_SUB_TASK_INDENT: u8 = 16;

    /// Return whether one shape option is selected.
    #[must_use]
    pub const fn has(self, option: u8) -> bool { self.options & option != 0 }
}

/// Clause indentation inside a task body, measured from the document margin.
///
/// A task marker is `- [ ] `, whose content starts two columns in, so this is
/// the body's own indent and is independent of how deeply sub-tasks nest.
const TASK_CLAUSE_INDENT: &str = "  ";

/// A generated target roadmap, its fragment, and the model behind both.
pub struct Generated {
    /// Complete target document, with LF line endings.
    pub target: String,
    /// Complete fragment document, with LF line endings.
    pub fragment: String,
    /// The dependency graph, held separately from the rendered text.
    pub model: Model,
}

/// Build a target roadmap, a fragment, and the model that governs both.
///
/// `edge_selectors` supplies one selector per generated step; `shape` selects
/// optional valid document features. Every clause generated is in one of the
/// three positions `docs/mapsplice-design.md` section 7 recognizes.
pub fn generate(edge_selectors: &[u8], shape: Shape) -> Result<Generated, fmt::Error> {
    let model = Model::generate(edge_selectors);
    let target = render_target(shape, &model)?;
    let fragment = render_fragment()?;
    Ok(Generated {
        target,
        fragment,
        model,
    })
}

/// Render the target roadmap, spacing each clause according to its form.
fn render_target(shape: Shape, model: &Model) -> Result<String, fmt::Error> {
    let sub_task_indent = if shape.has(Shape::DEEP_SUB_TASK_INDENT) {
        "    "
    } else {
        "  "
    };
    let mut document = String::new();
    if shape.has(Shape::PREAMBLE) {
        document.push_str("# Generated roadmap\n\nPreamble text before the first phase.\n\n");
    }
    for phase in 0..PHASE_COUNT {
        writeln!(document, "## {}. {}\n", phase + 1, phase_label(phase))?;
        for offset in 0..STEPS_PER_PHASE {
            let step = phase * STEPS_PER_PHASE + offset;
            writeln!(
                document,
                "### {}.{}. {}\n",
                phase + 1,
                offset + 1,
                step_label(step)
            )?;
            for position in 0..TASKS_PER_STEP {
                let task = step * TASKS_PER_STEP + position;
                let id = ItemId::new(Level::Task, task);
                let rendered = render_task(id, shape, model, sub_task_indent)?;
                document.push_str(&rendered);
            }
        }
    }
    if shape.has(Shape::TRAILING_SECTION) {
        document.push_str("---\n\nUnrelated prose after the last phase.\n");
    }
    Ok(document)
}

/// Render one task, including its clause and its addendum sub-tasks.
///
/// A clause stays in its container's body only while it sits within three
/// columns of that container's content start; beyond that the Markdown parser
/// reads it as an indented code block and the recognizer never sees a clause.
/// Each container therefore passes its own absolute body indent: two columns
/// for a task (`- [ ] ` starts its content there) and two columns past the
/// sub-task marker for a sub-task. Deriving a sub-task's indent by appending
/// to the task's would over-indent it and move the clause out of reach.
fn render_task(
    id: ItemId,
    shape: Shape,
    model: &Model,
    sub_task_indent: &str,
) -> Result<String, fmt::Error> {
    let checked = checkbox(shape, id.index);
    let mut item = format!(
        "- [{}] {}. {}\n",
        checked,
        old_anchor(id),
        task_label(id.index)
    );
    render_clauses(&mut item, model, id, TASK_CLAUSE_INDENT);
    item.push_str(&incidental_prose());
    if shape.has(Shape::WRAPPED_PROSE) {
        item.push_str("  Unrelated prose wrapped\n  across two source lines.\n");
    }
    let sub_task_body_indent = format!("{sub_task_indent}  ");
    for (position, sub_task) in sub_tasks_of_task(id.index).iter().enumerate() {
        writeln!(
            item,
            "{sub_task_indent}- [ ] {}.{}. {}",
            old_anchor(id),
            position + 1,
            sub_task_label(id.index, position)
        )?;
        render_clauses(&mut item, model, *sub_task, &sub_task_body_indent);
    }
    Ok(item)
}

/// Append every clause whose consumer is `consumer`, in the edge's own form.
///
/// The clauses are written into `item` rather than returned, so the caller can
/// keep a container's summary, clauses, and incidental prose in source order.
fn render_clauses(item: &mut String, model: &Model, consumer: ItemId, body_indent: &str) {
    for edge in model.edges_from(consumer) {
        let clause = format!("Requires {}.", old_anchor(edge.prerequisite));
        let rendered = render_clause(edge.form, &clause, body_indent);
        if edge.form == ClauseForm::Inline {
            // The inline form shares the summary line, which the caller has
            // already terminated with a newline. Join it there rather than
            // starting a fresh line: a one-space-indented line is read as a
            // continuation, so appending it would generate the wrong position
            // and leave the inline position untested.
            let newline = item.pop();
            debug_assert_eq!(newline, Some('\n'));
            item.push_str(&rendered);
            item.push('\n');
        } else {
            item.push_str(&rendered);
        }
    }
}

/// Render one clause in its generated form.
///
/// `body_indent` is the consumer container's body indent, measured absolutely
/// from the document margin; every non-inline form starts its line there. The
/// inline form carries a single leading space and is meant to be joined to the
/// summary line by the caller, which is what makes it inline rather than a
/// continuation. The nested-bullet forms supply their own list marker.
fn render_clause(form: ClauseForm, clause: &str, body_indent: &str) -> String {
    match form {
        ClauseForm::Inline => format!(" {clause}"),
        ClauseForm::Continuation => format!("\n{body_indent}{clause}\n"),
        ClauseForm::NestedBullet => format!("\n{body_indent}- {clause}\n"),
        ClauseForm::NestedSubTaskBullet => format!("\n{body_indent}  - {clause}\n"),
    }
}

/// Return the addendum identities belonging to one task index.
fn sub_tasks_of_task(task: usize) -> Vec<ItemId> {
    let step = task.div_euclid(TASKS_PER_STEP);
    let is_tail = task.rem_euclid(TASKS_PER_STEP) == TASKS_PER_STEP - 1;
    if !is_tail {
        return Vec::new();
    }
    (0..SUB_TASKS_PER_TAIL)
        .map(|sub_task| ItemId::new(Level::SubTask, step * SUB_TASKS_PER_TAIL + sub_task))
        .collect()
}

/// Return the rendered checkbox state for one item.
const fn checkbox(shape: Shape, index: usize) -> &'static str {
    if !shape.has(Shape::CHECKBOX_STATES) {
        return " ";
    }
    match index.rem_euclid(2) {
        0 => " ",
        _ => "x",
    }
}

/// Summary text the generator renders for one phase identity.
#[must_use]
pub fn phase_label(phase: usize) -> String { format!("Generated phase {phase}") }

/// Summary text the generator renders for one step identity.
#[must_use]
pub fn step_label(step: usize) -> String { format!("Generated step {step}") }

/// Summary text the generator renders for one task identity.
#[must_use]
pub fn task_label(task: usize) -> String { format!("Generated task {task}") }

/// Summary text the generator renders for one addendum sub-task identity.
#[must_use]
pub fn sub_task_label(task: usize, position: usize) -> String {
    format!("Generated sub-task {task}-{position}")
}

/// Summary text the generator renders for one fragment task position.
#[must_use]
pub fn fragment_label(index: usize) -> String { format!("Generated fragment task {index}") }

/// Unrelated numeric prose the generator emits beside each clause.
///
/// Every entry is a number-shaped token that shares an anchor's spelling with
/// some roadmap item, so a rewriter that substituted every number rather than
/// resolving dependency contexts would corrupt it. Each is deliberately
/// outside the dependency grammar: a `§`-sigilled section reference, a
/// semantic version, an incidental quantity, and a clause-shaped line inside a
/// fenced code example.
pub const INCIDENTAL_TEXTS: [&str; 4] = [
    "See §1.1.",
    "Released 1.4.0.",
    "Count 27.",
    "Requires 1.1.1.",
];

/// Return the incidental prose block the generator emits for one task.
///
/// The fenced example is last so the block stays a single well-formed body.
#[must_use]
pub fn incidental_prose() -> String {
    let mut text = format!(
        "  {}\n  {}\n  {}\n",
        INCIDENTAL_TEXTS[0], INCIDENTAL_TEXTS[1], INCIDENTAL_TEXTS[2]
    );
    text.push_str("  ```text\n  Requires 1.1.1.\n  ```\n");
    text
}

/// Render the fragment tasks spliced by an insert or replace edit.
///
/// A task fragment is recognized by its three-level anchor, so the tasks are
/// spelled `1.1.n` even though the fragment is a standalone document. The
/// target's numbering replaces those numbers on splice, so they are only a
/// parse-time device, not an identity. No clause is generated inside the
/// fragment: a fragment clause would resolve differently depending on which
/// target it lands in and so could not be predicted from the model alone.
pub fn render_fragment() -> Result<String, fmt::Error> {
    let mut document = String::new();
    for index in 0..FRAGMENT_TASKS {
        writeln!(
            document,
            "- [ ] 1.1.{}. {}",
            index + 1,
            fragment_label(index)
        )?;
    }
    Ok(document)
}
