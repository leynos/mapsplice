//! Unit tests for task-body details and numbered mixed-list checkboxes.

use std::collections::HashSet;

use super::{anchors, at, list, task};
use crate::roadmap::{
    RoadmapAnchor,
    TaskNumber,
    export::{
        blocks::{TaskContentBuilder, task_content},
        details::append_task_list,
    },
    parse_anchor,
};

/// Resolve a task anchor for builder construction.
fn task_number(raw: &str) -> TaskNumber {
    match parse_anchor(raw).expect("task anchor should parse") {
        RoadmapAnchor::Task(number) => number,
        other => panic!("expected a task anchor, got {other:?}"),
    }
}

/// Wrap task-body Markdown in a roadmap with one task.
fn document(body: &str) -> String {
    format!(
        concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. Task one.\n\n",
            "{body}",
        ),
        body = body
    )
}

#[test]
fn structural_sub_tasks_become_numbered_details() {
    let source = document("  - [ ] 1.1.1.1. Sub-task one.\n\n    Sub body paragraph.\n");

    let (_, details) =
        task_content(&task(&source), &anchors(&source)).expect("structural details should export");

    assert_eq!(details.len(), 1);
    assert_eq!(at(&details, 0).number.as_deref(), Some("1.1.1.1"));
    assert_eq!(at(&details, 0).title, "Sub-task one.");
    assert_eq!(at(&at(&details, 0).entries, 0).value, "Sub body paragraph.");
}

#[test]
fn mixed_numbered_checkbox_details_are_mapped_in_source_order() {
    let source = document(concat!(
        "  - A plain bullet.\n",
        "  - [ ] 1.1.1.1. Mixed one.\n",
        "  - [x] 1.1.1.2. Mixed two.\n",
    ));

    let (_, details) =
        task_content(&task(&source), &anchors(&source)).expect("mixed details should export");

    assert_eq!(details.len(), 2);
    assert_eq!(at(&details, 0).number.as_deref(), Some("1.1.1.1"));
    assert_eq!(at(&details, 0).title, "Mixed one.");
    assert_eq!(at(&details, 1).number.as_deref(), Some("1.1.1.2"));
}

#[test]
fn mixed_numbered_details_must_be_in_source_order() {
    let anchors = HashSet::new();
    let mut content = TaskContentBuilder::for_task(task_number("1.1.1"), &anchors);
    let list = list(concat!(
        "- A plain bullet.\n",
        "- [ ] 1.1.1.2. Out of order.\n",
    ));

    let error = append_task_list(&list, &mut content).expect_err("skipping an ordinal is invalid");

    assert!(
        error
            .to_string()
            .contains("out of order; expected ordinal 1"),
        "unexpected error: {error}"
    );
}

#[test]
fn mixed_numbered_details_must_belong_to_their_task() {
    let anchors = HashSet::new();
    let mut content = TaskContentBuilder::for_task(task_number("1.1.1"), &anchors);
    let list = list(concat!(
        "- A plain bullet.\n",
        "- [ ] 1.1.2.1. Foreign detail.\n",
    ));

    let error =
        append_task_list(&list, &mut content).expect_err("a detail from another task is invalid");

    assert!(
        error
            .to_string()
            .contains("does not belong to task `1.1.1`"),
        "unexpected error: {error}"
    );
}

#[test]
fn numbered_details_are_rejected_beneath_a_checkbox_detail_builder() {
    let anchors = HashSet::new();
    let mut content = TaskContentBuilder::nested(&anchors);
    let list = list("- [ ] 1.1.1.1. Nested numbered.\n");

    let error = append_task_list(&list, &mut content)
        .expect_err("numbered details cannot nest beneath a checkbox detail");

    assert!(
        error
            .to_string()
            .contains("is not valid beneath a checkbox detail"),
        "unexpected error: {error}"
    );
}

#[test]
fn numbered_details_are_rejected_beneath_checkbox_details() {
    let source = document(concat!(
        "  - [ ] Detail one.\n\n",
        "    - [ ] 1.1.1.1. Nested numbered.\n",
    ));

    let error = task_content(&task(&source), &anchors(&source))
        .expect_err("numbered details cannot nest beneath a checkbox detail");

    assert!(
        error
            .to_string()
            .contains("is not valid beneath a checkbox detail"),
        "unexpected error: {error}"
    );
}

#[test]
fn numbered_title_ignores_non_anchor_prefixes() {
    let plain = document("  - [ ] Version 1.0 notes.\n");
    let (_, plain_details) =
        task_content(&task(&plain), &anchors(&plain)).expect("plain detail should export");
    assert_eq!(at(&plain_details, 0).number, None);
    assert_eq!(at(&plain_details, 0).title, "Version 1.0 notes.");

    let emphasis = document("  - [ ] *Emphasized detail*.\n");
    let (_, emphasis_details) =
        task_content(&task(&emphasis), &anchors(&emphasis)).expect("emphasis detail should export");
    assert_eq!(at(&emphasis_details, 0).number, None);
    assert_eq!(at(&emphasis_details, 0).title, "*Emphasized detail*.");
}

#[test]
fn numbered_title_rejects_a_prefix_that_names_a_non_sub_task() {
    let source = document(concat!("  - A plain bullet.\n", "  - [ ] 1.1. Detail.\n",));

    let error = task_content(&task(&source), &anchors(&source))
        .expect_err("a step-level prefix must not be accepted");

    assert!(
        error.to_string().contains("must name a sub-task"),
        "unexpected error: {error}"
    );
}

#[test]
fn numbered_title_rejects_a_malformed_prefix() {
    let source = document(concat!(
        "  - A plain bullet.\n",
        "  - [ ] 1.1.1.1.1. Deep detail.\n",
    ));

    let error = task_content(&task(&source), &anchors(&source))
        .expect_err("an over-deep prefix is malformed");

    assert!(
        error
            .to_string()
            .contains("malformed numbered checkbox detail prefix"),
        "unexpected error: {error}"
    );
}

#[test]
fn nested_detail_paragraphs_and_thematic_breaks_are_mapped() {
    let source = document(concat!(
        "  - [ ] Detail one.\n\n",
        "    Detail paragraph.\n\n",
        "    ---\n",
    ));

    let (_, details) =
        task_content(&task(&source), &anchors(&source)).expect("nested detail content should map");

    assert_eq!(at(&details, 0).entries.len(), 1);
    assert_eq!(at(&at(&details, 0).entries, 0).value, "Detail paragraph.");
}

#[test]
fn nested_detail_rejects_unsupported_blocks() {
    let source = document(concat!(
        "  - [ ] Detail one.\n\n",
        "    ```text\n",
        "    code\n",
        "    ```\n",
    ));

    let error = task_content(&task(&source), &anchors(&source))
        .expect_err("a code block has no export representation");

    assert!(
        error
            .to_string()
            .contains("unsupported block in checkbox detail"),
        "unexpected error: {error}"
    );
}
