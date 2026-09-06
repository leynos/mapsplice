//! Property tests for task-level source preservation across mutations.

#[path = "support/workspace.rs"]
mod support;

use mapsplice::run_from_args;
use proptest::prelude::*;
use support::create_workspace;

#[derive(Clone, Copy)]
struct TaskSourceShape {
    marker_indent: usize,
    options: u8,
}

impl TaskSourceShape {
    const REVERSE_LABELS: u8 = 1;
    const SUB_TASK: u8 = 2;
    const LOOSE_LIST: u8 = 4;
    const DEPENDENCY: u8 = 8;
    const TRAILING_NEWLINE: u8 = 16;

    const fn has(self, option: u8) -> bool { self.options & option != 0 }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        .. ProptestConfig::default()
    })]

    #[test]
    fn task_operations_preserve_generated_unchanged_task_sources(
        operation in 0u8..3,
        marker_indent in 0usize..4,
        options in 0u8..32,
    ) {
        let workspace = create_workspace().expect("workspace fixture should initialize");
        let shape = TaskSourceShape {
            marker_indent,
            options,
        };
        let (target, first_task, second_task) = generated_task_source_document(shape);
        workspace
            .write_target(&target)
            .expect("target should be written");

        if operation == 0 {
            workspace
                .write_fragment("- [ ] 1.1.9. Inserted task.\n")
                .expect("fragment should be written");
        } else if operation == 2 {
            workspace
                .write_fragment("- [ ] 1.1.9. Replacement task.\n")
                .expect("fragment should be written");
        }
        let output = run_task_operation(
            operation,
            workspace.target.as_str(),
            workspace.fragment.as_str(),
        )
            .expect("task operation should succeed")
            .stdout
            .unwrap_or_default();

        prop_assert!(
            output.contains(&first_task),
            "first untouched task changed after operation {operation}:\nexpected:\n{first_task}\nactual:\n{output}"
        );
        if !shape.has(TaskSourceShape::DEPENDENCY) {
            let expected_second = if operation == 0 {
                second_task.as_str()
            } else {
                second_task.trim_end_matches('\n')
            };
            prop_assert!(
                output.contains(expected_second),
                "second untouched task changed after operation {operation}:\nexpected:\n{expected_second}\nactual:\n{output}"
            );
        }
    }
}

fn run_task_operation(
    operation: u8,
    target: &str,
    fragment: &str,
) -> mapsplice::Result<mapsplice::RunOutcome> {
    match operation {
        0 => run_from_args(["mapsplice", "insert", "--after", target, "1.1.2", fragment]),
        1 => run_from_args(["mapsplice", "delete", target, "1.1.3"]),
        _ => run_from_args(["mapsplice", "replace", target, "1.1.3", fragment]),
    }
}

fn generated_task_source_document(shape: TaskSourceShape) -> (String, String, String) {
    let marker = " ".repeat(shape.marker_indent);
    let continuation = " ".repeat(shape.marker_indent + 2);
    let separator = if shape.has(TaskSourceShape::LOOSE_LIST) {
        "\n"
    } else {
        ""
    };
    let (first_label, second_label) = if shape.has(TaskSourceShape::REVERSE_LABELS) {
        ("Second", "First")
    } else {
        ("First", "Second")
    };
    let dependency = if shape.has(TaskSourceShape::DEPENDENCY) {
        " Requires 1.1.1."
    } else {
        ""
    };
    let nested = if shape.has(TaskSourceShape::SUB_TASK) {
        format!("{continuation}- [ ] 1.1.1.1. Nested task.\n")
    } else {
        String::new()
    };
    let first_task = format!(
        "{marker}- [ ] 1.1.1. {first_label} task.\n{continuation}{first_label} \
         continuation.\n{nested}{separator}"
    );
    let second_task = format!(
        "{marker}- [ ] 1.1.2. {second_label} task.{dependency}\n{continuation}{second_label} \
         continuation.\n{separator}"
    );
    let third_task = format!("{marker}- [ ] 1.1.3. Edited task.\n");
    let mut target = format!(
        "# Roadmap\n\n## 1. Phase one\n\n### 1.1. Step \
         one\n\n{first_task}{second_task}{third_task}"
    );
    if !shape.has(TaskSourceShape::TRAILING_NEWLINE) {
        target = target.strip_suffix('\n').unwrap_or(&target).to_owned();
    }
    (target, first_task, second_task)
}
