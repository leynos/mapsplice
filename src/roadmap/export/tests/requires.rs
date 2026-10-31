//! Unit tests for export dependency scanning and formatted keywords.

use std::collections::HashSet;

use markdown::{
    ParseOptions,
    mdast::{MdxJsxFlowElement, MdxJsxTextElement, Node},
    to_mdast,
};
use rstest::rstest;

use super::{anchors, at, list_item, markdown_nodes, paragraph, task};
use crate::roadmap::export::requires::{
    from_checkbox,
    from_markdown,
    from_task,
    is_requires_clause,
    node_children,
};

/// Parse a Markdown fixture and return its root node.
fn root(source: &str) -> Node {
    to_mdast(source, &ParseOptions::gfm()).expect("fixture Markdown should parse")
}

/// Return the first child of the parsed root.
fn first_child(source: &str) -> Node {
    match root(source) {
        Node::Root(root) => root
            .children
            .into_iter()
            .next()
            .expect("fixture should have a root child"),
        other => other,
    }
}

/// Return the first inline child of the first paragraph.
fn first_inline(source: &str) -> Node {
    match first_child(source) {
        Node::Paragraph(paragraph) => paragraph
            .children
            .into_iter()
            .next()
            .expect("fixture paragraph should have a child"),
        other => panic!("fixture should start with a paragraph, got {other:?}"),
    }
}

#[rstest]
#[case::plain("Requires 1.1.1.", true)]
#[case::strong("**Requires** 1.1.1.", true)]
#[case::keyword_only("Requires", true)]
#[case::longer_word("Required 1.1.1.", false)]
#[case::unrelated("Notes.", false)]
fn is_requires_clause_recognises_only_whole_keywords(#[case] source: &str, #[case] expected: bool) {
    let result = is_requires_clause(&paragraph(source).children).expect("text should render");

    assert_eq!(result, expected);
}

#[test]
fn from_markdown_reports_a_dangling_dependency() {
    let title = markdown_nodes("Requires 9.9.9.");

    let error = from_markdown(&title, &[], &HashSet::new())
        .expect_err("an unknown dependency anchor is dangling");

    assert!(
        error.to_string().contains("9.9.9"),
        "unexpected error: {error}"
    );
}

#[test]
fn from_task_excludes_dependencies_owned_by_nested_sub_tasks() {
    let source = concat!(
        "# Example\n\n",
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Task one. Requires 1.1.2.\n\n",
        "  - [ ] 1.1.1.1. Sub-task one. Requires 1.1.3.\n\n",
        "- [ ] 1.1.2. Task two.\n",
    );

    let dependencies =
        from_task(&task(source), &anchors(source)).expect("task dependencies should scan");

    assert_eq!(dependencies, vec!["1.1.2".to_owned()]);
}

#[test]
fn from_checkbox_deduplicates_repeated_dependencies() {
    let anchors = anchors(concat!(
        "# Example\n\n",
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Task one.\n",
    ));
    let item = list_item("- [ ] Detail. Requires 1.1.1. Requires 1.1.1.\n");

    let dependencies = from_checkbox(&item, &anchors).expect("checkbox dependencies should scan");

    assert_eq!(dependencies, vec!["1.1.1".to_owned()]);
}

#[test]
fn formatted_requires_keyword_is_rejected_where_the_scanner_cannot_reach_it() {
    let nodes = markdown_nodes("*See **Requires** 1.1.1.*");

    let error = from_markdown(&nodes, &[], &HashSet::new())
        .expect_err("a nested formatted keyword cannot be scanned");

    assert!(
        error
            .to_string()
            .contains("formatted `Requires` keyword cannot be scanned"),
        "unexpected error: {error}"
    );
}

#[test]
fn node_children_covers_every_parsed_container() {
    let containers = [
        root("### heading"),
        first_child("> quoted"),
        first_child("- item"),
        first_inline("*emphasis*"),
        first_inline("~~deleted~~"),
        first_inline("[link](https://example.com)"),
        first_inline("[reference][id]\n\n[id]: https://example.com\n"),
        first_child("| a |\n| --- |\n| b |\n"),
        first_child("[^1]: note"),
        first_child("plain"),
    ];

    for container in &containers {
        assert!(
            node_children(container).is_some(),
            "expected children for {container:?}"
        );
    }

    let parsed = first_child("| a |\n| --- |\n| b |\n");
    let Node::Table(table) = parsed else {
        panic!("fixture should contain a table");
    };
    let first_row = at(&table.children, 0);
    assert!(node_children(first_row).is_some());
    let Node::TableRow(row) = first_row else {
        panic!("table should contain a row");
    };
    assert!(node_children(at(&row.children, 0)).is_some());
}

#[test]
fn node_children_covers_mdx_elements() {
    let flow = Node::MdxJsxFlowElement(MdxJsxFlowElement {
        children: Vec::new(),
        position: None,
        name: None,
        attributes: Vec::new(),
    });
    let text = Node::MdxJsxTextElement(MdxJsxTextElement {
        children: Vec::new(),
        position: None,
        name: None,
        attributes: Vec::new(),
    });

    assert!(node_children(&flow).is_some());
    assert!(node_children(&text).is_some());
}
