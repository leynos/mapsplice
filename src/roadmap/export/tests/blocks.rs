//! Unit tests for ordered export entries and task-body blocks.

use markdown::mdast::Node;

use super::{anchors, at, list, list_item, markdown, markdown_nodes, task};
use crate::roadmap::export::blocks::{
    entries,
    list_item_entry,
    list_items,
    step_content,
    task_content,
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
    assert!(at(&exported, 1).value.is_empty());
    assert_eq!(at(&exported, 1).entries.len(), 2);
    assert_eq!(at(&at(&exported, 1).entries, 1).value, "second");
}

#[test]
fn entries_reject_a_list_without_an_empty_label() {
    let error = entries(&markdown_nodes("- stray\n")).expect_err("a bare list is unsupported");

    assert!(
        error
            .to_string()
            .contains("must follow an empty labelled paragraph"),
        "unexpected error: {error}"
    );
}

#[test]
fn entries_reject_checkbox_items_outside_task_bodies() {
    let error = entries(&markdown_nodes("Label:\n\n- [ ] one\n"))
        .expect_err("checkbox items are task-body only");

    assert!(
        error
            .to_string()
            .contains("checkbox details are only valid in task bodies"),
        "unexpected error: {error}"
    );
}

#[test]
fn entries_reject_unsupported_blocks() {
    let error = entries(&markdown_nodes("## Heading\n")).expect_err("a heading is unsupported");

    assert!(
        error
            .to_string()
            .contains("unsupported block in phase or step content"),
        "unexpected error: {error}"
    );
}

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
fn list_item_entry_requires_a_first_paragraph() {
    let mut item = list_item("- outer\n");
    item.children = vec![Node::List(list("- inner\n"))];

    let error = list_item_entry(&item).expect_err("a leading nested list is unsupported");

    assert!(
        error
            .to_string()
            .contains("list items must begin with a paragraph"),
        "unexpected error: {error}"
    );
}

#[test]
fn list_item_entry_maps_nested_paragraphs_and_bullets() {
    let source = concat!(
        "- Outer.\n\n",
        "  Continuation paragraph.\n\n",
        "  - nested one\n",
        "  - nested two\n",
    );

    let entry = list_item_entry(&list_item(source))
        .expect("nested blocks should map")
        .expect("a non-requires item should produce an entry");

    assert_eq!(entry.value, "Outer.");
    assert_eq!(entry.entries.len(), 3);
    assert_eq!(at(&entry.entries, 0).value, "Continuation paragraph.");
    assert_eq!(at(&entry.entries, 2).value, "nested two");
}

#[test]
fn list_item_entry_skips_requires_and_thematic_break_in_children() {
    let source = "- Outer.\n\n  Requires 1.1.1.\n\n  ---\n";

    let entry = list_item_entry(&list_item(source))
        .expect("nested blocks should map")
        .expect("a non-requires item should produce an entry");

    assert!(
        entry.entries.is_empty(),
        "skipped blocks should not be entries"
    );
}

#[test]
fn list_item_entry_rejects_ordered_nested_lists() {
    let source = "- Outer.\n\n  1. ordered\n";

    let error = list_item_entry(&list_item(source)).expect_err("ordered nested lists fail");

    assert!(
        error
            .to_string()
            .contains("ordered lists are not supported in task entries"),
        "unexpected error: {error}"
    );
}

#[test]
fn list_item_entry_rejects_nested_checkbox_items() {
    let source = "- Outer.\n\n  - [ ] nested checkbox\n";

    let error = list_item_entry(&list_item(source)).expect_err("nested checkboxes fail");

    assert!(
        error
            .to_string()
            .contains("checkbox detail cannot be nested beneath a non-checkbox entry"),
        "unexpected error: {error}"
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
    assert!(details.is_empty());
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
