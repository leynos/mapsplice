//! Property tests for deterministic export layout and drift reporting.
//!
//! The first property renders the same semantic roadmap with different
//! wrapping, line endings, whitespace, and indentation, and requires each
//! source layout to produce byte-identical JSON. The second mutates one field
//! at every nesting level of a committed export and requires the drift report
//! to name that level.

// These properties only write roadmap targets; the shared fixture also offers
// fragment helpers that the splice test binaries use.
#[expect(
    dead_code,
    reason = "the export property test writes only roadmap targets, not fragments"
)]
#[path = "support/workspace.rs"]
mod workspace_support;

use camino::Utf8Path;
use mapsplice::{
    CliRequest,
    CommandKind,
    ExportFormat,
    GlobalOptions,
    MapspliceError,
    run_request,
};
use proptest::prelude::*;
use serde_json::{Value, json};
use workspace_support::{TestResult, Workspace, create_workspace};

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
struct PhaseModel {
    /// Phase title.
    title: String,
    /// Optional phase body entry.
    entry: Option<EntryModel>,
    /// Ordered steps.
    steps: Vec<StepModel>,
}

/// Source layout knobs exercised by the equivalence property.
#[derive(Clone, Copy, Debug)]
struct Layout {
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
const COMPACT: Layout = Layout {
    line_ending: "\n",
    detail_indent: 2,
    body_indent: 2,
    wrap_width: None,
    has_hard_breaks: false,
    spaces_per_gap: 1,
};

/// Soft-wrapped paragraphs with hard breaks and wider indentation.
const WRAPPED: Layout = Layout {
    line_ending: "\n",
    detail_indent: 2,
    body_indent: 4,
    wrap_width: Some(24),
    has_hard_breaks: true,
    spaces_per_gap: 1,
};

/// CRLF paragraphs with expanded word gaps and deeper indentation.
const SPACED: Layout = Layout {
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
fn roadmap_model() -> impl Strategy<Value = Vec<PhaseModel>> {
    prop::collection::vec(phase(), 1..3)
}

/// Split text into wrap tokens whose labels stay attached to their colon.
///
/// A label is only recognised when it contains no line break, so wrapping
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
fn render(model: &[PhaseModel], layout: Layout) -> String {
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

/// Export one roadmap source through the library request path.
fn export_source(source: &str) -> TestResult<String> {
    let workspace = create_workspace()?;
    workspace.write_target(source)?;
    let outcome = run_request(CliRequest {
        global: GlobalOptions::default(),
        target: workspace.target.clone(),
        command: CommandKind::Export {
            format: ExportFormat::Json,
            check: None,
        },
    })?;
    outcome
        .stdout
        .ok_or_else(|| "export should write JSON to stdout".into())
}

/// Run the committed-export check for a workspace.
fn check_export(
    workspace: &Workspace,
    export_path: &Utf8Path,
) -> Result<mapsplice::RunOutcome, MapspliceError> {
    run_request(CliRequest {
        global: GlobalOptions::default(),
        target: workspace.target.clone(),
        command: CommandKind::Export {
            format: ExportFormat::Json,
            check: Some(export_path.to_owned()),
        },
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        .. ProptestConfig::default()
    })]

    #[test]
    fn layout_variants_export_identical_json(model in roadmap_model()) {
        let mut exports = Vec::new();
        for layout in [COMPACT, WRAPPED, SPACED] {
            let source = render(&model, layout);
            let exported = export_source(&source)
                .unwrap_or_else(|error| panic!("layout {layout:?} failed to export: {error}"));
            exports.push((layout, exported));
        }
        let Some((first_layout, first_export)) = exports.first() else {
            prop_assert!(false, "at least one layout should be rendered");
            return Ok(());
        };
        for (layout, exported) in exports.iter().skip(1) {
            prop_assert_eq!(
                exported,
                first_export,
                "layout {:?} differs from layout {:?}",
                layout,
                first_layout
            );
        }
    }
}

/// One JSON mutation and the drift fragment it must produce.
struct DriftCase {
    /// Short description used in assertion messages.
    description: &'static str,
    /// JSON pointer to the field that the mutation replaces.
    pointer: &'static str,
    /// Fragment that the drift report must contain.
    expected: &'static str,
    /// Mutator that returns whether its target path existed.
    apply: fn(&mut Value, &str, &str) -> bool,
}

/// Replace a string field.
fn set_string(root: &mut Value, pointer: &str, replacement: &str) -> bool {
    root.pointer_mut(pointer).is_some_and(|slot| {
        *slot = Value::String(replacement.to_owned());
        true
    })
}

/// Replace a checkbox state field.
fn set_state(root: &mut Value, pointer: &str, _replacement: &str) -> bool {
    root.pointer_mut(pointer).is_some_and(|slot| {
        *slot = json!("done");
        true
    })
}

/// Replace a dependency list with an anchor that is not in the roadmap.
fn set_requires(root: &mut Value, pointer: &str, _replacement: &str) -> bool {
    root.pointer_mut(pointer).is_some_and(|slot| {
        *slot = json!(["9.9.9"]);
        true
    })
}

/// Replace the export format version.
fn set_format_version(root: &mut Value, _pointer: &str, _replacement: &str) -> bool {
    root.pointer_mut("/format_version").is_some_and(|slot| {
        *slot = json!(2);
        true
    })
}

/// Mutations covering every nesting level of the exported document.
const DRIFT_CASES: &[DriftCase] = &[
    DriftCase {
        description: "format version",
        pointer: "/format_version",
        expected: "~ format_version:",
        apply: set_format_version,
    },
    DriftCase {
        description: "phase title",
        pointer: "/phases/0/title",
        expected: "~ phase 1 title",
        apply: set_string,
    },
    DriftCase {
        description: "phase requires",
        pointer: "/phases/0/requires",
        expected: "~ phase 1 requires",
        apply: set_requires,
    },
    DriftCase {
        description: "phase entry",
        pointer: "/phases/0/entries/0/value",
        expected: "~ phase 1 entry 1 value",
        apply: set_string,
    },
    DriftCase {
        description: "step title",
        pointer: "/phases/0/steps/0/title",
        expected: "~ step 1.1 title",
        apply: set_string,
    },
    DriftCase {
        description: "step question",
        pointer: "/phases/0/steps/0/question",
        expected: "~ step 1.1 question",
        apply: set_string,
    },
    DriftCase {
        description: "step requires",
        pointer: "/phases/0/steps/0/requires",
        expected: "~ step 1.1 requires",
        apply: set_requires,
    },
    DriftCase {
        description: "step entry",
        pointer: "/phases/0/steps/0/entries/0/value",
        expected: "~ step 1.1 entry 1 value",
        apply: set_string,
    },
    DriftCase {
        description: "step trailing entry",
        pointer: "/phases/0/steps/0/trailing/0/value",
        expected: "~ step 1.1 trailing entry 1 value",
        apply: set_string,
    },
    DriftCase {
        description: "task title",
        pointer: "/phases/0/steps/0/tasks/0/title",
        expected: "~ task 1.1.1 title",
        apply: set_string,
    },
    DriftCase {
        description: "task state",
        pointer: "/phases/0/steps/0/tasks/0/state",
        expected: "~ task 1.1.1 state",
        apply: set_state,
    },
    DriftCase {
        description: "task requires",
        pointer: "/phases/0/steps/0/tasks/0/requires",
        expected: "~ task 1.1.1 requires",
        apply: set_requires,
    },
    DriftCase {
        description: "task entry",
        pointer: "/phases/0/steps/0/tasks/0/entries/0/value",
        expected: "~ task 1.1.1 entry 1 value",
        apply: set_string,
    },
    DriftCase {
        description: "sub-task title",
        pointer: "/phases/0/steps/0/tasks/0/details/0/title",
        expected: "~ sub-task 1.1.1.1 title",
        apply: set_string,
    },
    DriftCase {
        description: "sub-task state",
        pointer: "/phases/0/steps/0/tasks/0/details/0/state",
        expected: "~ sub-task 1.1.1.1 state",
        apply: set_state,
    },
    DriftCase {
        description: "sub-task requires",
        pointer: "/phases/0/steps/0/tasks/0/details/0/requires",
        expected: "~ sub-task 1.1.1.1 requires",
        apply: set_requires,
    },
    DriftCase {
        description: "sub-task entry",
        pointer: "/phases/0/steps/0/tasks/0/details/0/entries/0/value",
        expected: "~ sub-task 1.1.1.1 entry 1 value",
        apply: set_string,
    },
    DriftCase {
        description: "nested detail title",
        pointer: "/phases/0/steps/0/tasks/0/details/0/details/0/title",
        expected: "~ detail 1 of sub-task 1.1.1.1 title",
        apply: set_string,
    },
    DriftCase {
        description: "nested detail state",
        pointer: "/phases/0/steps/0/tasks/0/details/0/details/0/state",
        expected: "~ detail 1 of sub-task 1.1.1.1 state",
        apply: set_state,
    },
    DriftCase {
        description: "nested detail entry",
        pointer: "/phases/0/steps/0/tasks/0/details/0/details/0/entries/0/value",
        expected: "~ detail 1 of sub-task 1.1.1.1 entry 1 value",
        apply: set_string,
    },
];

/// A roadmap with content at every export nesting level.
const NESTED_ROADMAP: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "Phase note: phase value.\n\n",
    "### 1.1. Step one\n\n",
    "What question?\n\n",
    "Step note: step value.\n\n",
    "- [ ] 1.1.1. Task one.\n\n",
    "  Task note: task value.\n\n",
    "  - [ ] 1.1.1.1. Detail one.\n\n",
    "    Detail note: detail value.\n\n",
    "    - [ ] Nested detail.\n\n",
    "      Nested note: nested value.\n\n",
    "Step close: step trailing value.\n",
);

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 8,
        .. ProptestConfig::default()
    })]

    #[test]
    fn drift_reports_mutations_at_every_nesting_level(replacement in "[a-z]{4,10}") {
        let workspace = create_workspace().expect("workspace fixture should initialize");
        workspace
            .write_target(NESTED_ROADMAP)
            .expect("fixture roadmap should be written");
        let committed = export_source(NESTED_ROADMAP).expect("fixture export should succeed");
        let export_path = workspace.target.with_file_name("roadmap.json");

        for case in DRIFT_CASES {
            let mut mutated: Value =
                serde_json::from_str(&committed).expect("export should decode as JSON");
            prop_assert!(
                (case.apply)(&mut mutated, case.pointer, &replacement),
                "{}: pointer {:?} is missing from the export",
                case.description,
                case.pointer
            );
            let encoded =
                serde_json::to_string(&mutated).expect("mutated export should encode as JSON");
            workspace
                .dir
                .write("roadmap.json", &encoded)
                .expect("mutated export should be written");

            let error = check_export(&workspace, &export_path)
                .expect_err("a mutated committed export should drift");
            let message = error.to_string();
            prop_assert!(
                message.contains(case.expected),
                "{}: expected {:?} in drift report: {message}",
                case.description,
                case.expected
            );
        }
    }
}
