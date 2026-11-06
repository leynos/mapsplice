//! Unit tests for ordered export entries and task-body blocks.

use markdown::mdast::Node;
use rstest::rstest;

use super::{anchors, at, list, list_item, markdown, markdown_nodes, task};
use crate::roadmap::export::{
    blocks::{entries, list_item_entries, list_items, step_content, task_content},
    schema::ExportDetail,
};

#[test]
fn entries_skip_requires_clauses_and_thematic_breaks() {
    let source = concat!(
        "Intro paragraph.\n\n",
        "Requires 1.1.1.\n\n",
        "---\n\n",
        "Label:\n\n",
        "- first\n",
        "- second\n",
    );

    let exported = entries(&markdown_nodes(source)).expect("phase content should export");

    assert_eq!(exported.len(), 2);
    assert_eq!(at(&exported, 0).value, "Intro paragraph.");
    assert_eq!(at(&exported, 1).key.as_deref(), Some("Label"));
    assert_eq!(at(&exported, 1).value, "");
    assert_eq!(at(&exported, 1).entries.len(), 2);
    assert_eq!(at(&at(&exported, 1).entries, 1).value, "second");
}

/// Structures outside the export grammar are rejected with a specific reason.
#[rstest]
#[case::bare_list("- stray\n", "must follow an empty labelled paragraph")]
#[case::checkbox_outside_task(
    "Label:\n\n- [ ] one\n",
    "checkbox details are only valid in task bodies"
)]
#[case::heading("## Heading\n", "unsupported block in phase or step content")]
fn entries_reject_unsupported_structures(#[case] source: &str, #[case] expected: &str) {
    let error = entries(&markdown_nodes(source)).expect_err("the structure is unsupported");

    assert!(
        error.to_string().contains(expected),
        "unexpected error: {error}"
    );
}

/// The leading unlabelled paragraph becomes the step question.
#[test]
fn step_content_takes_the_first_unlabelled_paragraph_as_the_question() {
    let source = "What is it?\n\nNote: label\n\nAnother line.\n";

    let (question, exported) =
        step_content(&markdown_nodes(source)).expect("step content should export");

    assert_eq!(question.as_deref(), Some("What is it?"));
    assert_eq!(exported.len(), 2);
    assert_eq!(at(&exported, 0).key.as_deref(), Some("Note"));
    assert_eq!(at(&exported, 1).value, "Another line.");
}

#[test]
fn step_content_keeps_labelled_paragraphs_as_entries() {
    let (question, exported) =
        step_content(&markdown_nodes("Note: answer\n")).expect("step content should export");

    assert_eq!(question, None);
    assert_eq!(at(&exported, 0).key.as_deref(), Some("Note"));
}

#[test]
fn list_items_reject_ordered_lists() {
    let error = list_items(&list("1. one\n")).expect_err("ordered lists are unsupported");

    assert!(
        error
            .to_string()
            .contains("ordered lists are not supported"),
        "unexpected error: {error}"
    );
}

#[test]
fn list_items_reject_non_list_item_children() {
    let mut list = list("- one\n");
    list.children.extend(markdown("stray paragraph"));

    let error = list_items(&list).expect_err("a list must contain only list items");

    assert!(
        error
            .to_string()
            .contains("list must contain only list items"),
        "unexpected error: {error}"
    );
}

#[test]
fn list_item_entries_requires_a_first_paragraph() {
    let mut item = list_item("- outer\n");
    item.children = vec![Node::List(list("- inner\n"))];

    let error = list_item_entries(&item).expect_err("a leading nested list is unsupported");

    assert!(
        error
            .to_string()
            .contains("list items must begin with a paragraph"),
        "unexpected error: {error}"
    );
}

#[test]
fn list_item_entries_maps_nested_paragraphs_and_bullets() {
    let source = concat!(
        "- Outer.\n\n",
        "  Continuation paragraph.\n\n",
        "  - nested one\n",
        "  - nested two\n",
    );

    let entries = list_item_entries(&list_item(source)).expect("nested blocks should map");

    assert_eq!(entries.len(), 1);
    let entry = at(&entries, 0);
    assert_eq!(entry.value, "Outer.");
    assert_eq!(entry.entries.len(), 3);
    assert_eq!(at(&entry.entries, 0).value, "Continuation paragraph.");
    assert_eq!(at(&entry.entries, 2).value, "nested two");
}

#[test]
fn list_item_entries_skips_requires_and_thematic_break_in_children() {
    let source = "- Outer.\n\n  Requires 1.1.1.\n\n  ---\n";

    let entries = list_item_entries(&list_item(source)).expect("nested blocks should map");

    assert_eq!(entries.len(), 1);
    assert!(
        at(&entries, 0).entries.is_empty(),
        "skipped blocks should not be entries"
    );
}

#[test]
fn list_item_entries_exports_supported_children_of_a_requires_bullet() {
    let source = "- Requires 1.1.1.\n\n  Success: retain this criterion.\n";

    let entries = list_item_entries(&list_item(source))
        .expect("supported children of a dependency bullet should export");

    assert_eq!(entries.len(), 1);
    assert_eq!(at(&entries, 0).key.as_deref(), Some("Success"));
    assert_eq!(at(&entries, 0).value, "retain this criterion.");
}

/// Each unsupported child is reported where it appears, whatever the parent.
#[rstest]
#[case::checkbox_beneath_dependency_bullet(
    "- Requires 1.1.1.\n\n  - [ ] nested checkbox\n",
    "checkbox detail cannot be nested beneath a non-checkbox entry"
)]
#[case::ordered_list_beneath_plain_entry(
    "- Outer.\n\n  1. ordered\n",
    "ordered lists are not supported in task entries"
)]
#[case::checkbox_beneath_plain_entry(
    "- Outer.\n\n  - [ ] nested checkbox\n",
    "checkbox detail cannot be nested beneath a non-checkbox entry"
)]
fn list_item_entries_rejects_unsupported_children(
    #[case] source: &str,
    #[case] expected_message: &str,
) {
    let error =
        list_item_entries(&list_item(source)).expect_err("unsupported children must be rejected");
    let diagnostic = error.to_string();

    assert!(
        diagnostic.contains(expected_message),
        "unexpected error: {diagnostic}"
    );
    assert!(
        diagnostic.contains("line 3, column 3"),
        "the rejection should carry the child's location: {diagnostic}"
    );
}

#[test]
fn task_content_maps_body_paragraphs_and_thematic_breaks() {
    let source = concat!(
        "# Example\n\n",
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Task one.\n\n",
        "  Body paragraph.\n\n",
        "  ---\n",
    );

    let (exported, details) =
        task_content(&task(source), &anchors(source)).expect("task body should export");

    assert_eq!(exported.len(), 1);
    assert_eq!(at(&exported, 0).value, "Body paragraph.");
    assert_eq!(details, Vec::<ExportDetail>::new());
}

#[test]
fn task_content_rejects_ordered_lists_in_task_bodies() {
    let source = concat!(
        "# Example\n\n",
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Task one.\n\n",
        "  1. ordered body item\n",
    );

    let error = task_content(&task(source), &anchors(source))
        .expect_err("ordered task-body lists are unsupported");

    assert!(
        error
            .to_string()
            .contains("ordered lists are not supported in task bodies"),
        "unexpected error: {error}"
    );
}
