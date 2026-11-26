//! Source-span helpers for preserving unchanged roadmap Markdown.

use markdown::{
    mdast::{List, ListItem, Node},
    unist::Position,
};

use super::source_span_kernel::{extract_line_span, extract_span, task_item_spans};

/// Copy source for an unchanged node, preserving leading indentation.
///
/// The Markdown parser positions some nodes after indentation that belongs to
/// their rendered block. This helper widens the span back to the start of the
/// source line so preserved nested list blocks keep their original indent.
///
/// # Examples
///
/// ```rust
/// use mapsplice::doctest_support::original_node_source;
/// use markdown::{ParseOptions, mdast::Node, to_mdast};
///
/// let source = "  - [ ] 1.1.1. Nested task.\n";
/// let tree = to_mdast(source, &ParseOptions::gfm())?;
/// let Node::Root(root) = &tree else {
///     unreachable!("markdown documents parse into root nodes");
/// };
/// let node = root.children.first().expect("root should contain the list");
///
/// assert_eq!(
///     original_node_source(node, source),
///     Some("  - [ ] 1.1.1. Nested task.".to_owned())
/// );
/// # Ok::<(), markdown::message::Message>(())
/// ```
#[must_use]
pub fn original_node_source(node: &Node, source: &str) -> Option<String> {
    original_position_source(node.position()?, source)
}

/// Copy source for an unchanged span, preserving leading indentation.
#[must_use]
pub(crate) fn original_position_source(position: &Position, source: &str) -> Option<String> {
    extract_line_span(source, position.start.offset, position.end.offset).ok()
}

/// Capture each top-level list item's original source through the next item.
#[must_use]
pub(crate) fn task_item_sources(
    list: &List,
    items: &[&ListItem],
    source: &str,
) -> Vec<Option<String>> {
    let starts = items
        .iter()
        .map(|item| item.position.as_ref().map(|position| position.start.offset))
        .collect::<Vec<_>>();
    let list_end = list.position.as_ref().map(|position| position.end.offset);
    task_item_spans(&starts, list_end, source)
        .into_iter()
        .map(|candidate| candidate.and_then(|span| extract_span(source, span)))
        .collect()
}

#[cfg(test)]
mod tests {
    //! Unit tests for exact source-span extraction.

    use markdown::{ParseOptions, mdast::Node, to_mdast};
    use rstest::rstest;

    use super::{original_node_source, task_item_sources};

    #[test]
    fn original_node_source_preserves_leading_indent() {
        let source = "  - [ ] 1.1.1. Nested task.\n";
        let tree = to_mdast(source, &ParseOptions::gfm()).expect("source should parse");
        let Node::Root(root) = &tree else {
            panic!("markdown documents parse into root nodes");
        };
        let node = root.children.first().expect("root should contain the list");

        assert_eq!(
            original_node_source(node, source),
            Some("  - [ ] 1.1.1. Nested task.".to_owned()),
            "a preserved node must keep its leading indentation"
        );
    }

    #[test]
    fn original_node_source_returns_none_for_missing_position() {
        let node = Node::Text(markdown::mdast::Text {
            value: "No position".to_owned(),
            position: None,
        });

        assert_eq!(
            original_node_source(&node, "No position"),
            None,
            "an unpositioned node has no source to copy"
        );
    }

    /// Parse a two-item list and return the item sources.
    fn two_item_sources(source: &str) -> Vec<Option<String>> {
        let tree = to_mdast(source, &ParseOptions::gfm()).expect("source should parse");
        let Node::Root(root) = &tree else {
            panic!("markdown documents parse into root nodes");
        };
        let Some(Node::List(list)) = root.children.first() else {
            panic!("source should contain a list");
        };
        let items = list
            .children
            .iter()
            .filter_map(|node| match node {
                Node::ListItem(item) => Some(item),
                _ => None,
            })
            .collect::<Vec<_>>();
        task_item_sources(list, &items, source)
    }

    #[rstest]
    #[case::lf(
        "  - [ ] 1.1.1. First task.\n  - [ ] 1.1.2. Second task.\n",
        &["  - [ ] 1.1.1. First task.\n", "  - [ ] 1.1.2. Second task."],
    )]
    #[case::crlf(
        "  - [ ] 1.1.1. First task.\r\n  - [ ] 1.1.2. Second task.\r\n",
        &["  - [ ] 1.1.1. First task.\r\n", "  - [ ] 1.1.2. Second task."],
    )]
    #[case::cr(
        "  - [ ] 1.1.1. First task.\r  - [ ] 1.1.2. Second task.\r",
        &["  - [ ] 1.1.1. First task.\r", "  - [ ] 1.1.2. Second task."],
    )]
    fn task_source_stops_before_the_next_marker_indentation(
        #[case] source: &str,
        #[case] expected: &[&str],
    ) {
        let expected_sources = expected
            .iter()
            .map(|text| Some((*text).to_owned()))
            .collect::<Vec<_>>();
        assert_eq!(
            two_item_sources(source),
            expected_sources,
            "each item's source must run to the next item's line start for every line ending"
        );
    }
}
