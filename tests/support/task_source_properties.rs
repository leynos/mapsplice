//! Generated valid roadmap inputs and expectations for source-fidelity tests.

/// A supported structural operation used by the source-fidelity properties.
#[derive(Clone, Copy)]
pub enum TaskOperation {
    /// Insert a top-level task.
    Insert,
    /// Delete a top-level task.
    Delete,
    /// Replace a top-level task.
    Replace,
    /// Insert a sub-task into the first task.
    InsertSubTask,
}

impl TaskOperation {
    /// Convert a generated selector into a supported task operation.
    pub const fn from_selector(selector: u8) -> Self {
        match selector {
            0 => Self::Insert,
            1 => Self::Delete,
            2 => Self::Replace,
            _ => Self::InsertSubTask,
        }
    }
}

/// Valid source-format dimensions for a generated task list.
#[derive(Clone, Copy)]
pub struct TaskSourceShape {
    /// Legal leading indentation before each task marker.
    pub marker_indent: usize,
    /// Extra valid indentation after the task marker width.
    pub continuation_extra: usize,
    /// Bit field selecting optional valid list features.
    pub options: u8,
}

impl TaskSourceShape {
    /// Reverse stable labels to vary semantic task ordering.
    pub const REVERSE_LABELS: u8 = 1;
    /// Insert blank lines between task items.
    pub const LOOSE_LIST: u8 = 2;
    /// Retain a final line ending after the last task.
    pub const TRAILING_NEWLINE: u8 = 4;
    /// Use CRLF rather than LF within captured task source.
    pub const CRLF: u8 = 8;
    /// Include a nested sub-task beneath the first task.
    pub const SUB_TASK: u8 = 16;

    /// Determine whether the generated roadmap includes one shape option.
    pub const fn has(self, option: u8) -> bool { self.options & option != 0 }

    /// Return the line ending used by preserved source chunks.
    pub const fn line_ending(self) -> &'static str {
        if self.has(Self::CRLF) { "\r\n" } else { "\n" }
    }
}

/// Exact pre-operation source for a task with a stable test identity.
#[derive(Clone)]
pub struct TaskSnapshot {
    /// Stable descriptive identity used in assertion output.
    pub identity: &'static str,
    /// Verbatim task chunk captured before an operation.
    pub source: String,
}

/// A valid roadmap and every task source chunk it contained before mutation.
pub struct GeneratedRoadmap {
    /// Complete source document supplied to the CLI.
    pub source: String,
    /// Exact chunks for each top-level task in source order.
    pub tasks: [TaskSnapshot; 4],
    /// Stable visible labels used to identify canonical output.
    pub labels: [&'static str; 4],
}

/// Supply a deliberately non-canonical fragment that must render canonically.
pub const fn fragment_for(operation: TaskOperation) -> &'static str {
    match operation {
        TaskOperation::Insert => "- [ ] 9.9.9. Inserted task.\n    Inserted continuation.\n",
        TaskOperation::Replace => "- [ ] 9.9.9. Replacement task.\n    Replacement continuation.\n",
        TaskOperation::InsertSubTask => "- [ ] 9.9.9.9. Inserted sub-task.\n",
        TaskOperation::Delete => "",
    }
}

/// Build a valid multi-task roadmap with byte-distinct task source chunks.
pub fn generated_roadmap(shape: TaskSourceShape, force_sub_task: bool) -> GeneratedRoadmap {
    let has_sub_task = force_sub_task || shape.has(TaskSourceShape::SUB_TASK);
    let labels = if shape.has(TaskSourceShape::REVERSE_LABELS) {
        ["Untouched", "Changed", "Reference", "Primary"]
    } else {
        ["Primary", "Reference", "Changed", "Untouched"]
    };
    let separator = if shape.has(TaskSourceShape::LOOSE_LIST) {
        shape.line_ending()
    } else {
        ""
    };
    let mut tasks = generated_tasks(shape, labels, has_sub_task, separator);
    if !shape.has(TaskSourceShape::TRAILING_NEWLINE) {
        trim_task_terminator(&mut tasks[3].source, shape.line_ending());
    }
    let source = format!(
        "# Roadmap\n\n## 1. Phase one\n\n### 1.1. Step one\n\n{}{}{}{}",
        tasks[0].source, tasks[1].source, tasks[2].source, tasks[3].source,
    );
    if shape.has(TaskSourceShape::TRAILING_NEWLINE) {
        trim_task_terminator(&mut tasks[3].source, shape.line_ending());
    }
    GeneratedRoadmap {
        source,
        tasks,
        labels,
    }
}

/// Build the ordered task chunks for one generated roadmap.
fn generated_tasks(
    shape: TaskSourceShape,
    labels: [&'static str; 4],
    has_sub_task: bool,
    separator: &str,
) -> [TaskSnapshot; 4] {
    [
        TaskSnapshot {
            identity: "first task",
            source: task_source(
                shape,
                &TaskSourceSpec {
                    number: "1.1.1",
                    label: labels[0],
                    dependency: None,
                    has_sub_task,
                    separator,
                },
            ),
        },
        TaskSnapshot {
            identity: "dependency task",
            source: task_source(
                shape,
                &TaskSourceSpec {
                    number: "1.1.2",
                    label: labels[1],
                    dependency: Some("Requires 1.1.3."),
                    has_sub_task: false,
                    separator,
                },
            ),
        },
        TaskSnapshot {
            identity: "edited task",
            source: task_source(
                shape,
                &TaskSourceSpec {
                    number: "1.1.3",
                    label: labels[2],
                    dependency: None,
                    has_sub_task: false,
                    separator,
                },
            ),
        },
        TaskSnapshot {
            identity: "stable task",
            source: task_source(
                shape,
                &TaskSourceSpec {
                    number: "1.1.4",
                    label: labels[3],
                    dependency: None,
                    has_sub_task: false,
                    separator: "",
                },
            ),
        },
    ]
}

/// Render the canonical two-space source expected for a changed task.
pub fn canonical_task(number: &str, label: &str, dependency: Option<&str>) -> String {
    let dependency_suffix = dependency.map_or(String::new(), |value| format!(" {value}"));
    format!(
        concat!(
            "- [ ] {number}. {label} task.{dependency}\n",
            "  {label} continuation.",
        ),
        number = number,
        label = label,
        dependency = dependency_suffix,
    )
}

/// Render a parent task whose sub-task edit requires canonical output.
pub fn canonical_parent_with_sub_tasks(label: &str, insert_after: bool) -> String {
    let parent = canonical_task("1.1.1", label, None);
    let existing = format!(
        "  - [ ] 1.1.1.{}. Nested task.\n    Nested continuation.",
        if insert_after { 1 } else { 2 },
    );
    let inserted = format!(
        "  - [ ] 1.1.1.{}. Inserted sub-task.",
        if insert_after { 2 } else { 1 },
    );
    if insert_after {
        format!("{parent}\n{existing}\n{inserted}")
    } else {
        format!("{parent}\n{inserted}\n{existing}")
    }
}

/// Source fields needed to construct one valid generated task.
struct TaskSourceSpec<'a> {
    number: &'a str,
    label: &'a str,
    dependency: Option<&'a str>,
    has_sub_task: bool,
    separator: &'a str,
}

/// Build one valid task source chunk with generated indentation and spacing.
fn task_source(shape: TaskSourceShape, spec: &TaskSourceSpec<'_>) -> String {
    let marker = " ".repeat(shape.marker_indent);
    let continuation = " ".repeat(shape.marker_indent + 2 + shape.continuation_extra);
    let line_ending = shape.line_ending();
    let dependency_suffix = spec
        .dependency
        .map_or(String::new(), |value| format!(" {value}"));
    let mut source = format!(
        concat!(
            "{marker}- [ ] {number}. {label} task.{dependency}{line_ending}",
            "{continuation}{label} continuation.{line_ending}",
        ),
        marker = marker,
        number = spec.number,
        label = spec.label,
        dependency = dependency_suffix,
        line_ending = line_ending,
        continuation = continuation,
    );
    if spec.has_sub_task {
        let sub_task_indent = " ".repeat(shape.marker_indent + 2);
        let sub_task_body_indent = " ".repeat(shape.marker_indent + 4);
        source.push_str(&format!(
            concat!(
                "{sub_task_indent}- [ ] {number}.1. Nested task.{line_ending}",
                "{sub_task_body_indent}Nested continuation.{line_ending}",
            ),
            sub_task_indent = sub_task_indent,
            number = spec.number,
            line_ending = line_ending,
            sub_task_body_indent = sub_task_body_indent,
        ));
    }
    source.push_str(spec.separator);
    source
}

/// Remove the final line ending from a task snapshot without reallocating it.
fn trim_task_terminator(source: &mut String, line_ending: &str) {
    if source.ends_with(line_ending) {
        source.truncate(source.len() - line_ending.len());
    }
}
