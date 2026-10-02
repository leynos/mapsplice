//! Invocation and assertion helpers for task source preservation tests.

use mapsplice::run_from_args;
use proptest::{prelude::prop_assert_eq, test_runner::TestCaseResult};

use crate::task_source_support::{GeneratedRoadmap, TaskOperation, TaskSnapshot, canonical_task};

/// One generated operation and its file inputs.
pub(super) struct TaskOperationRequest<'a> {
    pub(super) operation: TaskOperation,
    pub(super) insert_after: bool,
    pub(super) anchor_index: usize,
    pub(super) target: &'a str,
    pub(super) fragment: &'a str,
}

/// Run one generated operation with only valid task or sub-task anchors.
pub(super) fn run_task_operation(
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
pub(super) struct OperationResult<'a> {
    pub(super) operation: TaskOperation,
    pub(super) after: bool,
    pub(super) anchor_index: usize,
    pub(super) roadmap: &'a GeneratedRoadmap,
    pub(super) snapshots: &'a [TaskSnapshot; 4],
    pub(super) output: &'a str,
}

/// Assert the exact source or canonical output expected for one operation.
pub(super) fn assert_operation_result(result: &OperationResult<'_>) -> TestCaseResult {
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
pub(super) fn assert_source_occurs_once(output: &str, snapshot: &TaskSnapshot) -> TestCaseResult {
    assert_text_occurs_once(
        output,
        &snapshot.source,
        &format!("{} ({})", snapshot.identity, snapshot.anchor),
    )
}

/// Require a changed task's prior source chunk to be absent from output.
pub(super) fn assert_source_absent(output: &str, snapshot: &TaskSnapshot) -> TestCaseResult {
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
pub(super) fn assert_text_occurs_once(
    output: &str,
    expected: &str,
    context: &str,
) -> TestCaseResult {
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
