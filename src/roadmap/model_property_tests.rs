//! Generated model checks for task and sub-task preservation invariants.

use std::fmt::Write;

use proptest::prelude::*;

use super::{TaskChild, TaskEntry};
use crate::roadmap::{
    RoadmapOperation,
    apply_command,
    parse_anchor,
    parse_fragment,
    parse_roadmap,
};

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        .. ProptestConfig::default()
    })]

    /// Check that generated sub-task mutations preserve child alignment and
    /// clear only the affected preservation state.
    #[test]
    fn generated_sub_task_mutations_preserve_alignment_and_untouched_source(
        sub_task_count in 2usize..5,
        operation in 0u8..4,
    ) {
        let source = roadmap_with_sub_tasks(sub_task_count)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        let mut roadmap = parse_roadmap(&source)
            .expect("generated roadmap should parse");
        let original_first_sub_task_source = parent_task(&roadmap)?
            .sub_tasks()
            .first()
            .and_then(|sub_task| sub_task.original_source())
            .map(str::to_owned)
            .expect("target sub-task should retain parsed source");
        let target = format!("1.1.1.{sub_task_count}");
        let (roadmap_operation, fragment) = generated_operation(operation, &target)?;

        apply_command(&mut roadmap, roadmap_operation, fragment)
            .expect("generated structural mutation should succeed");

        let task = parent_task(&roadmap)?;
        assert_child_alignment(task)?;
        prop_assert!(task.original_source().is_none());
        if operation == 3 {
            prop_assert!(task
                .sub_tasks()
                .iter()
                .skip(1)
                .all(|sub_task| sub_task.original_source().is_none()));
        } else {
            let first_sub_task = task
                .sub_tasks()
                .first()
                .ok_or_else(|| TestCaseError::fail("unchanged sub-task should remain present"))?;
            prop_assert_eq!(first_sub_task.original_source(), Some(original_first_sub_task_source.as_str()));
        }
    }

    /// Check that dependency rewrites clear the changed item and its parent.
    #[test]
    fn generated_dependency_rewrites_clear_affected_item_and_parent_source(
        dependency_in_sub_task in any::<bool>(),
    ) {
        let mut roadmap = parse_roadmap(&dependency_roadmap(dependency_in_sub_task))
            .expect("generated dependency roadmap should parse");
        let fragment = parse_fragment(concat!(
            "## 9. Inserted phase\n\n",
            "### 9.1. Inserted step\n\n",
            "- [ ] 9.1.1. Inserted task.\n"
        ))
        .expect("generated phase fragment should parse");
        apply_command(
            &mut roadmap,
            RoadmapOperation::Insert {
                anchor: parse_anchor("1").expect("phase anchor should parse"),
                after: true,
            },
            Some(fragment),
        )
            .expect("generated dependency rewrite should succeed");

        let task = parent_task(&roadmap)?;
        prop_assert!(task.original_source().is_none());
        if dependency_in_sub_task {
            let first_sub_task = task
                .sub_tasks()
                .first()
                .ok_or_else(|| TestCaseError::fail("dependency sub-task should remain present"))?;
            prop_assert!(first_sub_task.original_source().is_none());
        }
    }
}

/// Build one valid bounded operation and optional fragment for a generated
/// sub-task mutation.
fn generated_operation(
    operation: u8,
    target: &str,
) -> Result<(RoadmapOperation, Option<super::RoadmapFragment>), TestCaseError> {
    let anchor = parse_anchor(target).map_err(|error| TestCaseError::fail(error.to_string()))?;
    match operation {
        0 => insert_sub_task_after(anchor),
        1 => Ok((RoadmapOperation::Delete { anchor }, None)),
        2 => replace_sub_task(anchor),
        _ => insert_sub_task_before_first(),
    }
}

/// Build an insertion operation that leaves existing sub-task numbers stable.
fn insert_sub_task_after(
    anchor: crate::roadmap::RoadmapAnchor,
) -> Result<(RoadmapOperation, Option<super::RoadmapFragment>), TestCaseError> {
    Ok((
        RoadmapOperation::Insert {
            anchor,
            after: true,
        },
        Some(sub_task_fragment("  - [ ] 1.1.1.1. Inserted sub-task.\n")?),
    ))
}

/// Build a replacement operation for the generated final sub-task.
fn replace_sub_task(
    anchor: crate::roadmap::RoadmapAnchor,
) -> Result<(RoadmapOperation, Option<super::RoadmapFragment>), TestCaseError> {
    Ok((
        RoadmapOperation::Replace { anchor },
        Some(sub_task_fragment(
            "  - [ ] 1.1.1.1. Replacement sub-task.\n",
        )?),
    ))
}

/// Build an insertion operation that renumbers every existing sub-task.
fn insert_sub_task_before_first()
-> Result<(RoadmapOperation, Option<super::RoadmapFragment>), TestCaseError> {
    let anchor = parse_anchor("1.1.1.1").map_err(|error| TestCaseError::fail(error.to_string()))?;
    Ok((
        RoadmapOperation::Insert {
            anchor,
            after: false,
        },
        Some(sub_task_fragment(
            "  - [ ] 1.1.1.1. Renumbering sub-task.\n",
        )?),
    ))
}

/// Parse a known-valid generated sub-task fragment.
fn sub_task_fragment(source: &str) -> Result<super::RoadmapFragment, TestCaseError> {
    parse_fragment(source).map_err(|error| TestCaseError::fail(error.to_string()))
}

/// Verify that structural sub-task children match each sub-task exactly once.
fn assert_child_alignment(task: &TaskEntry) -> Result<(), TestCaseError> {
    let child_identities = task
        .children()
        .iter()
        .filter_map(|child| match child {
            TaskChild::Body(_) => None,
            TaskChild::SubTask(identity) => Some(*identity),
        })
        .collect::<Vec<_>>();
    let sub_task_identities = task
        .sub_tasks()
        .iter()
        .map(|sub_task| sub_task.identity)
        .collect::<Vec<_>>();

    prop_assert_eq!(&child_identities, &sub_task_identities);
    prop_assert_eq!(
        child_identities.len(),
        child_identities
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    );
    Ok(())
}

/// Return the generated parent task used by the model property cases.
fn parent_task(roadmap: &super::RoadmapDocument) -> Result<&TaskEntry, TestCaseError> {
    roadmap
        .phases
        .first()
        .and_then(|phase| phase.steps.first())
        .and_then(|step| step.tasks.first())
        .ok_or_else(|| TestCaseError::fail("generated roadmap should contain a parent task"))
}

/// Build a valid roadmap with a source-preservable nested sub-task body.
fn roadmap_with_sub_tasks(sub_task_count: usize) -> Result<String, std::fmt::Error> {
    let mut sub_tasks = concat!(
        "  - [ ] 1.1.1.1. Preserved sub-task.\n\n",
        "      ```text\n",
        "      preserved nested body\n",
        "      ```\n\n"
    )
    .to_owned();
    for index in 2..=sub_task_count {
        writeln!(sub_tasks, "  - [ ] 1.1.1.{index}. Sub-task {index}.")?;
    }
    Ok(format!(
        "# Roadmap\n\n## 1. Phase\n\n### 1.1. Step\n\n- [ ] 1.1.1. Parent task.\n{sub_tasks}"
    ))
}

/// Build a two-phase roadmap whose dependency is on the task or its sub-task.
fn dependency_roadmap(dependency_in_sub_task: bool) -> String {
    let body = if dependency_in_sub_task {
        "  - [ ] 1.1.1.1. Nested task Requires 1.1.1.\n"
    } else {
        ""
    };
    let summary = if dependency_in_sub_task {
        "Parent task."
    } else {
        "Parent task Requires 1.1.1."
    };
    format!(
        concat!(
            "# Roadmap\n\n",
            "## 1. Phase\n\n",
            "### 1.1. Step\n\n",
            "- [ ] 1.1.1. {summary}\n",
            "{body}\n",
            "## 2. Later phase\n\n",
            "### 2.1. Later step\n\n",
            "- [ ] 2.1.1. Later task.\n"
        ),
        summary = summary,
        body = body,
    )
}
