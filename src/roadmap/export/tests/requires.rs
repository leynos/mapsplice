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
    RequiresParagraph,
    classify_paragraph,
    from_checkbox,
    from_markdown,
    from_task,
    node_children,
    paragraph_entry_or_clause,
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
#[case::plain("Requires 1.1.1.", RequiresParagraph::ClauseOnly)]
#[case::strong("**Requires** 1.1.1.", RequiresParagraph::ClauseOnly)]
#[case::strong_with_prose("**Requires** further review.", RequiresParagraph::ClauseOnly)]
#[case::keyword_only("Requires", RequiresParagraph::ClauseOnly)]
#[case::longer_word("Required 1.1.1.", RequiresParagraph::Content)]
#[case::unrelated("Notes.", RequiresParagraph::Content)]
#[case::connector("Requires 1.1.1 and 1.1.2.", RequiresParagraph::ClauseOnly)]
#[case::escaped_strong("\\*\\*Requires\\*\\* further review.", RequiresParagraph::Content)]
#[case::trailing_prose(
    "Requires 1.1.1. Success: keep this.",
    RequiresParagraph::ClauseWithProse("Success: keep this.".to_owned())
)]
#[case::formatted_prose(
    "Requires 1.1.1. Success: **keep this**.",
    RequiresParagraph::ClauseWithProse("Success: **keep this**.".to_owned())
)]
#[case::formatted_before_prose(
    "Requires 1.1.1. **Success**: keep this.",
    RequiresParagraph::ClauseWithProse("**Success**: keep this.".to_owned())
)]
fn classify_paragraph_separates_clauses_from_content(
    #[case] source: &str,
    #[case] expected: RequiresParagraph,
) {
    let result = classify_paragraph(&paragraph(source).children).expect("text should render");

    assert_eq!(result, expected, "unexpected classification for {source:?}");
}

/// Escaped asterisks are ordinary prose that happens to render exactly like a
/// bold keyword. If classification read the rendered prefix, this paragraph
/// would be dropped as a dependency clause and an edit to its text would never
/// surface as export drift.
#[test]
fn escaped_literal_asterisks_export_as_content() {
    let paragraph = paragraph("\\*\\*Requires\\*\\* further review.");

    let entry = paragraph_entry_or_clause(&paragraph)
        .expect("literal text should classify")
        .expect("escaped asterisks are content, not a clause");

    assert_eq!(
        entry,
        (None, "**Requires** further review.".to_owned()),
        "escaped asterisks should be exported as unlabelled content"
    );
}

/// A formatting node after the clause must not stop keyword recognition or
/// swallow the labelled prose that follows the dependency clause.
#[test]
fn formatted_prose_after_a_clause_keeps_its_label() {
    let paragraph = paragraph("Requires 1.1.1. Success: **keep this**.");

    let entry = paragraph_entry_or_clause(&paragraph)
        .expect("text should classify")
        .expect("prose after a clause is content");

    assert_eq!(
        entry,
        (Some("Success".to_owned()), "**keep this**.".to_owned()),
        "the Success entry should keep its label and formatted value"
    );
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

    let dependencies =
        from_checkbox(&item, None, &anchors).expect("checkbox dependencies should scan");

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

/// Dependency scanning descends into every container node the parser produces.
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
