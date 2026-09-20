//! Property tests for task-level source preservation across mutations.

#[path = "support/task_source_properties.rs"]
mod task_source_support;
#[path = "support/workspace.rs"]
mod workspace_support;

use mapsplice::run_from_args;
use proptest::{prelude::*, test_runner::TestCaseResult};
use task_source_support::{
    GeneratedRoadmap,
    TaskOperation,
    TaskSnapshot,
    TaskSourceShape,
    canonical_parent_with_sub_tasks,
    canonical_task,
    fragment_for,
    generated_roadmap,
};
use workspace_support::create_workspace;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 24,
        .. ProptestConfig::default()
    })]

    #[test]
    fn task_operations_preserve_exact_unchanged_sources(
        operation_selector in 0u8..4,
        marker_indent in 0usize..4,
        continuation_extra in 0usize..2,
        options in 0u8..32,
        insert_after in any::<bool>(),
        anchor_selector in 0u8..12,
    ) {
        let operation = TaskOperation::from_selector(operation_selector);
        let anchor_index = operation.task_anchor_index(anchor_selector);
        let roadmap = generated_roadmap(
            TaskSourceShape {
                marker_indent,
                continuation_extra,
                options,
            },
            matches!(operation, TaskOperation::InsertSubTask),
        );
        let snapshots = roadmap.tasks.clone();
        let workspace = create_workspace().expect("workspace fixture should initialize");
        workspace
            .write_target(&roadmap.source)
            .expect("target should be written");
        if !matches!(operation, TaskOperation::Delete) {
            workspace
                .write_fragment(fragment_for(operation))
                .expect("fragment should be written");
        }
        let operation_outcome = run_task_operation(&TaskOperationRequest {
            operation,
            insert_after,
            anchor_index,
            target: workspace.target.as_str(),
            fragment: workspace.fragment.as_str(),
        });
        prop_assert!(
            operation_outcome.is_ok(),
            "generated task operation should succeed: {operation_outcome:?}",
        );
        let output = operation_outcome
            .expect("checked generated operation result")
            .stdout
            .unwrap_or_default();

        assert_operation_result(&OperationResult {
            operation,
            after: insert_after,
            anchor_index,
            roadmap: &roadmap,
            snapshots: &snapshots,
            output: &output,
        })?;
    }

    #[test]
    fn two_operation_sequences_keep_unchanged_task_sources(
        marker_indent in 0usize..4,
        continuation_extra in 0usize..2,
        options in 0u8..32,
    ) {
        let roadmap = generated_roadmap(
            TaskSourceShape {
                marker_indent,
                continuation_extra,
                options,
            },
            false,
        );
        let first_snapshot = roadmap.tasks[0].clone();
        let workspace = create_workspace().expect("workspace fixture should initialize");
        workspace.write_target(&roadmap.source).expect("target should be written");
        workspace
            .write_fragment(fragment_for(TaskOperation::Insert))
            .expect("insert fragment should be written");
        let inserted = run_task_operation(&TaskOperationRequest {
            operation: TaskOperation::Insert,
            insert_after: true,
            anchor_index: 1,
            target: workspace.target.as_str(),
            fragment: workspace.fragment.as_str(),
        })
        .expect("first generated operation should succeed")
        .stdout
        .unwrap_or_default();
        workspace
            .write_target(&inserted)
            .expect("intermediate roadmap should be written");
        workspace
            .write_fragment(fragment_for(TaskOperation::Replace))
            .expect("replacement fragment should be written");
        let output = run_task_operation(&TaskOperationRequest {
            operation: TaskOperation::Replace,
            insert_after: false,
            anchor_index: 1,
            target: workspace.target.as_str(),
            fragment: workspace.fragment.as_str(),
        })
        .expect("second generated operation should succeed")
        .stdout
        .unwrap_or_default();

        assert_source_occurs_once(&output, &first_snapshot)?;
        assert_text_occurs_once(
            &output,
            &canonical_task("1.1.2", "Replacement", None),
            "replacement after bounded sequence",
        )?;
    }
}

#[test]
fn dependency_rewrite_invalidates_only_its_referencing_task() {
    let roadmap = generated_roadmap(
        TaskSourceShape {
            marker_indent: 0,
            continuation_extra: 0,
            options: TaskSourceShape::TRAILING_NEWLINE,
        },
        false,
    );
    let snapshots = roadmap.tasks.clone();
    let workspace = create_workspace().expect("workspace fixture should initialize");
    workspace
        .write_target(&roadmap.source)
        .expect("target should be written");
    workspace
        .write_fragment(fragment_for(TaskOperation::Insert))
        .expect("insert fragment should be written");
    let output = run_task_operation(&TaskOperationRequest {
        operation: TaskOperation::Insert,
        insert_after: true,
        anchor_index: 1,
        target: workspace.target.as_str(),
        fragment: workspace.fragment.as_str(),
    })
    .expect("dependency rewrite should succeed")
    .stdout
    .unwrap_or_default();

    assert_source_occurs_once(&output, &snapshots[0])
        .expect("unchanged first task should retain its source");
    assert_source_absent(&output, &snapshots[1])
        .expect("dependency-rewritten task should lose stale source");
    assert_text_occurs_once(
        &output,
        &canonical_task("1.1.2", roadmap.labels[1], Some("Requires 1.1.4.")),
        "dependency-rewritten task",
    )
    .expect("dependency-rewritten task should render canonically");
}

#[test]
fn final_document_terminator_is_not_part_of_the_last_task_snapshot() {
    let roadmap = generated_roadmap(
        TaskSourceShape {
            marker_indent: 0,
            continuation_extra: 0,
            options: TaskSourceShape::TRAILING_NEWLINE,
        },
        false,
    );

    assert!(roadmap.source.ends_with('\n'));
    assert!(!roadmap.tasks[3].source.ends_with('\n'));
}

#[test]
fn sub_task_expectation_follows_the_requested_insertion_side() {
    let before = canonical_parent_with_sub_tasks("Primary", false);
    let after = canonical_parent_with_sub_tasks("Primary", true);
    let original = concat!(
        "- [ ] 1.1.1. Primary task.\n",
        "  Primary continuation.\n",
        "  - [ ] 1.1.1.1. Nested task.\n",
        "    Nested continuation."
    );

    assert!(before.find("Inserted sub-task.") < before.find("Nested task."));
    assert!(after.find("Nested task.") < after.find("Inserted sub-task."));
    assert_eq!(before.matches(original).count(), 0);
    assert_eq!(after.matches(original).count(), 1);
}

/// One generated operation and its file inputs.
struct TaskOperationRequest<'a> {
    operation: TaskOperation,
    insert_after: bool,
    anchor_index: usize,
    target: &'a str,
    fragment: &'a str,
}

/// Run one generated operation with only valid task or sub-task anchors.
fn run_task_operation(
    request: &TaskOperationRequest<'_>,
) -> mapsplice::Result<mapsplice::RunOutcome> {
    let task_anchor = format!("1.1.{}", request.anchor_index + 1);
    let mut arguments = vec!["mapsplice".to_owned()];
    match request.operation {
        TaskOperation::Insert => {
            arguments.push("insert".to_owned());
            if request.insert_after {
                arguments.push("--after".to_owned());
            }
            arguments.extend([
                request.target.to_owned(),
                task_anchor,
                request.fragment.to_owned(),
            ]);
        }
        TaskOperation::Delete => {
            arguments.extend(["delete".to_owned(), request.target.to_owned(), task_anchor]);
        }
        TaskOperation::Replace => arguments.extend([
            "replace".to_owned(),
            request.target.to_owned(),
            task_anchor,
            request.fragment.to_owned(),
        ]),
        TaskOperation::InsertSubTask => {
            arguments.push("insert".to_owned());
            if request.insert_after {
                arguments.push("--after".to_owned());
            }
            arguments.extend([
                request.target.to_owned(),
                "1.1.1.1".to_owned(),
                request.fragment.to_owned(),
            ]);
        }
    }
    run_from_args(arguments)
}

/// Inputs and output used to verify one generated task operation.
struct OperationResult<'a> {
    operation: TaskOperation,
    after: bool,
    anchor_index: usize,
    roadmap: &'a GeneratedRoadmap,
    snapshots: &'a [TaskSnapshot; 4],
    output: &'a str,
}

/// Assert the exact source or canonical output expected for one operation.
fn assert_operation_result(result: &OperationResult<'_>) -> TestCaseResult {
    match result.operation {
        TaskOperation::Insert => assert_insert_result(result),
        TaskOperation::Delete => assert_delete_result(result),
        TaskOperation::Replace => assert_replace_result(result),
        TaskOperation::InsertSubTask => assert_sub_task_insert_result(
            result.after,
            result.roadmap,
            result.snapshots,
            result.output,
        ),
    }
}

/// Assert insertion invalidates renumbered and dependency-rewritten task text.
fn assert_insert_result(result: &OperationResult<'_>) -> TestCaseResult {
    let insertion_index = result.anchor_index + usize::from(result.after);
    for ((index, snapshot), label) in result
        .snapshots
        .iter()
        .enumerate()
        .zip(result.roadmap.labels)
    {
        if index < insertion_index && index != 1 {
            assert_source_occurs_once(result.output, snapshot)?;
        } else {
            assert_source_absent(result.output, snapshot)?;
            let number = index + 1 + usize::from(index >= insertion_index);
            let dependency_target = 3 + usize::from(2 >= insertion_index);
            assert_text_occurs_once(
                result.output,
                &canonical_original_task(snapshot, label, number, dependency_target),
                snapshot.identity,
            )?;
        }
    }
    assert_text_occurs_once(
        result.output,
        &canonical_task(&format!("1.1.{}", insertion_index + 1), "Inserted", None),
        "inserted task",
    )
}

/// Assert deletion keeps only the task whose text and number are unchanged.
fn assert_delete_result(result: &OperationResult<'_>) -> TestCaseResult {
    for ((index, snapshot), label) in result
        .snapshots
        .iter()
        .enumerate()
        .zip(result.roadmap.labels)
    {
        if index == result.anchor_index {
            assert_source_absent(result.output, snapshot)?;
        } else if task_source_survives_delete(index, result.anchor_index) {
            assert_source_occurs_once(result.output, snapshot)?;
        } else {
            assert_source_absent(result.output, snapshot)?;
            let number = index + 1 - usize::from(index > result.anchor_index);
            let dependency_target = 3 - usize::from(2 > result.anchor_index);
            assert_text_occurs_once(
                result.output,
                &canonical_original_task(snapshot, label, number, dependency_target),
                snapshot.identity,
            )?;
        }
    }
    Ok(())
}

/// Return whether this generated deletion leaves a task source unchanged.
const fn task_source_survives_delete(index: usize, deleted_index: usize) -> bool {
    index == 0 && deleted_index == 1
}

/// Assert replacement preserves all sibling source chunks exactly once.
fn assert_replace_result(result: &OperationResult<'_>) -> TestCaseResult {
    for (index, snapshot) in result.snapshots.iter().enumerate() {
        if index == result.anchor_index {
            assert_source_absent(result.output, snapshot)?;
        } else {
            assert_source_occurs_once(result.output, snapshot)?;
        }
    }
    assert_text_occurs_once(
        result.output,
        &canonical_task(
            &format!("1.1.{}", result.anchor_index + 1),
            "Replacement",
            None,
        ),
        "replacement task",
    )
}

/// Render one original task after a structural operation changes its number.
fn canonical_original_task(
    snapshot: &TaskSnapshot,
    label: &str,
    number: usize,
    dependency_target: usize,
) -> String {
    let dependency =
        (snapshot.anchor == "1.1.2").then(|| format!("Requires 1.1.{dependency_target}."));
    canonical_task(&format!("1.1.{number}"), label, dependency.as_deref())
}

/// Assert a sub-task edit invalidates only its parent task source.
fn assert_sub_task_insert_result(
    after: bool,
    roadmap: &GeneratedRoadmap,
    snapshots: &[TaskSnapshot; 4],
    output: &str,
) -> TestCaseResult {
    for snapshot in &snapshots[1..] {
        assert_source_occurs_once(output, snapshot)?;
    }
    assert_text_occurs_once(
        output,
        &canonical_task("1.1.1", roadmap.labels[0], None),
        "canonically rendered sub-task parent",
    )?;
    assert_text_occurs_once(
        output,
        &format!(
            "  - [ ] 1.1.1.{}. Inserted sub-task.",
            if after { 2 } else { 1 },
        ),
        "canonically rendered inserted sub-task",
    )
}

/// Count one exact source chunk and fail if canonical output was duplicated.
fn assert_source_occurs_once(output: &str, snapshot: &TaskSnapshot) -> TestCaseResult {
    assert_text_occurs_once(
        output,
        &snapshot.source,
        &format!("{} ({})", snapshot.identity, snapshot.anchor),
    )
}

/// Require a changed task's prior source chunk to be absent from output.
fn assert_source_absent(output: &str, snapshot: &TaskSnapshot) -> TestCaseResult {
    prop_assert_eq!(
        output.matches(&snapshot.source).count(),
        0,
        "stale source for {} ({}) remained in output:\n{}",
        snapshot.identity,
        snapshot.anchor,
        output,
    );
    Ok(())
}

/// Require one exact occurrence to detect stale-plus-canonical duplication.
fn assert_text_occurs_once(output: &str, expected: &str, context: &str) -> TestCaseResult {
    prop_assert_eq!(
        output.matches(expected).count(),
        1,
        "expected exactly one {}:\nexpected:\n{}\nactual:\n{}",
        context,
        expected,
        output,
    );
    Ok(())
}
