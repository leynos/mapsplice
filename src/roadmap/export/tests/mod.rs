//! Direct unit tests for the JSON export adapter and its drift report.

mod blocks;
mod details;
mod drift;
mod requires;
mod schema;
mod text;

use std::collections::HashSet;

use markdown::{
    ParseOptions,
    mdast::{List, ListItem, Node, Paragraph},
    to_mdast,
};

use super::{export_task, known_anchor_numbers, serialize, to_schema};
use crate::roadmap::{
    RoadmapDocument,
    model::{MarkdownNodes, TaskEntry},
    parse_roadmap,
};

/// Borrow the fixture item at `index`, failing with context when it is absent.
fn at<T>(items: &[T], index: usize) -> &T {
    items
        .get(index)
        .expect("fixture should contain the requested item")
}

/// Parse Markdown and return its top-level syntax nodes.
fn markdown(source: &str) -> Vec<Node> {
    match to_mdast(source, &ParseOptions::gfm()).expect("fixture Markdown should parse") {
        Node::Root(root) => root.children,
        other => vec![other],
    }
}

/// Wrap parsed Markdown in the roadmap-owned node collection.
fn markdown_nodes(source: &str) -> MarkdownNodes { MarkdownNodes::from_nodes(markdown(source)) }

/// Parse a roadmap fixture, failing the test when it is not valid.
fn roadmap(source: &str) -> RoadmapDocument {
    parse_roadmap(source).expect("fixture roadmap should parse")
}

/// Return the first paragraph of a Markdown fixture.
fn paragraph(source: &str) -> Paragraph {
    markdown(source)
        .into_iter()
        .find_map(|node| match node {
            Node::Paragraph(paragraph) => Some(paragraph),
            _ => None,
        })
        .expect("fixture should contain a paragraph")
}

/// Return the first list of a Markdown fixture.
fn list(source: &str) -> List {
    markdown(source)
        .into_iter()
        .find_map(|node| match node {
            Node::List(list) => Some(list),
            _ => None,
        })
        .expect("fixture should contain a list")
}

/// Return the first list item of a Markdown fixture.
fn list_item(source: &str) -> ListItem {
    list(source)
        .children
        .into_iter()
        .find_map(|node| match node {
            Node::ListItem(item) => Some(item),
            _ => None,
        })
        .expect("fixture should contain a list item")
}

/// Return the first task of the first step of the first phase.
fn task(source: &str) -> TaskEntry {
    roadmap(source)
        .phases
        .first()
        .and_then(|phase| phase.steps.first())
        .and_then(|step| step.tasks().first())
        .cloned()
        .expect("fixture should contain a task")
}

/// Return every known anchor in a roadmap fixture.
fn anchors(source: &str) -> HashSet<String> { known_anchor_numbers(&roadmap(source)) }

/// A roadmap with one phase, one step, and one task.
const BASIC: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. Task one.\n",
);

#[test]
fn serialize_is_pretty_json_with_one_final_newline() {
    let document = to_schema(&roadmap(BASIC)).expect("export should succeed");

    let rendered = serialize(&document).expect("serialization should succeed");

    assert!(
        rendered.ends_with("}\n"),
        "serialized JSON should end with a newline"
    );
    assert!(
        !rendered.ends_with("\n\n"),
        "only one trailing newline is allowed"
    );
    let decoded: serde_json::Value =
        serde_json::from_str(&rendered).expect("serialized JSON should decode");
    assert_eq!(
        decoded.get("format_version"),
        Some(&serde_json::json!(1)),
        "serialized JSON should carry the current format version"
    );
}

/// A roadmap exercising phase, step, task, and nested detail content.
const LAYERED: &str = concat!(
    "# Example\n\n",
    "## 1. Phase **one**\n\n",
    "Phase introduction.\n\n",
    "### 1.1. Step **one**\n\n",
    "What problem does it solve?\n\n",
    "Note: the answer.\n\n",
    "- [ ] 1.1.1. Task **title**.\n\n",
    "  Task body paragraph.\n\n",
    "  - [ ] 1.1.1.1. Sub-task **one**.\n\n",
    "### 1.2. Step two\n",
);

#[test]
fn to_schema_normalizes_phase_and_step_titles() {
    let document = to_schema(&roadmap(LAYERED)).expect("export should succeed");

    let phase = document.phases.first().expect("phase should be exported");
    assert_eq!(phase.title, "Phase **one**");
    assert_eq!(at(&phase.entries, 0).value, "Phase introduction.");
    let step = phase.steps.first().expect("step should be exported");
    assert_eq!(step.title, "Step **one**");
    assert_eq!(
        step.question.as_deref(),
        Some("What problem does it solve?")
    );
    assert_eq!(at(&step.entries, 0).key.as_deref(), Some("Note"));
    assert_eq!(at(&step.entries, 0).value, "the answer.");
}

#[test]
fn to_schema_normalizes_task_and_detail_titles() {
    let document = to_schema(&roadmap(LAYERED)).expect("export should succeed");

    let task = document
        .phases
        .first()
        .and_then(|phase| phase.steps.first())
        .and_then(|step| step.tasks.first())
        .expect("task should be exported");
    assert_eq!(task.title, "Task **title**.");
    assert_eq!(at(&task.entries, 0).value, "Task body paragraph.");
    let detail = task.details.first().expect("sub-task should be exported");
    assert_eq!(detail.number.as_deref(), Some("1.1.1.1"));
    assert_eq!(detail.title, "Sub-task **one**.");
}

#[test]
fn export_task_rejects_a_task_without_a_checkbox() {
    let mut task = task(BASIC);
    task.set_checked(None);

    let error = export_task(&task, &anchors(BASIC))
        .expect_err("a task without a checkbox state cannot be exported");

    assert!(
        error.to_string().contains("has no checkbox state"),
        "unexpected error: {error}"
    );
}

#[test]
fn known_anchor_numbers_include_mixed_checkbox_sub_tasks() {
    let source = concat!(
        "# Example\n\n",
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Task one.\n\n",
        "  A body paragraph.\n\n",
        "  - A plain body bullet.\n",
        "  - [ ] 1.1.1.1. Mixed sub-task.\n",
    );

    let anchors = anchors(source);

    assert!(anchors.contains("1.1.1"), "task anchor should be known");
    assert!(
        anchors.contains("1.1.1.1"),
        "mixed-list sub-task anchor should be known"
    );
}
