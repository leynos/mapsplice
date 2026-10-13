//! Property tests for task-level source preservation across mutations.

#[path = "support/task_source_operation.rs"]
mod operation_support;
#[path = "support/task_source_properties.rs"]
mod task_source_support;
#[path = "support/workspace.rs"]
mod workspace_support;

use operation_support::{
    OperationResult,
    TaskOperationRequest,
    assert_operation_result,
    assert_source_absent,
    assert_source_occurs_once,
    assert_text_occurs_once,
    run_task_operation,
};
use proptest::prelude::*;
use task_source_support::{
    TaskOperation,
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
