//! Source-span helpers for preserving unchanged roadmap Markdown.

use markdown::{
    mdast::{List, ListItem, Node},
    unist::{Point, Position},
};

/// Copy source for an unchanged node, preserving leading indentation.
///
/// The Markdown parser positions some nodes after indentation that belongs to
/// their rendered block. This helper widens the span back to the start of the
/// source line so preserved nested list blocks keep their original indent.
///
/// # Examples
///
/// ```rust
/// use markdown::{ParseOptions, mdast::Node, to_mdast};
/// #
/// # fn original_node_source(node: &Node, source: &str) -> Option<String> {
/// #     let position = node.position()?;
/// #     let prefix = source.get(..position.start.offset)?;
/// #     let start = prefix.rfind('\n').map_or(0, |index| index + 1);
/// #     source.get(start..position.end.offset).map(str::to_owned)
/// # }
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
    let prefix = source.get(..position.start.offset)?;
    let start = prefix.rfind('\n').map_or(0, |index| index + 1);
    source.get(start..position.end.offset).map(str::to_owned)
}

/// Return a position point at the beginning of its source line.
fn line_start(position: &Position, source: &str) -> Option<Point> {
    let mut start = position.start.clone();
    let prefix = source.get(..start.offset)?;
    start.offset = prefix.rfind('\n').map_or(0, |index| index + 1);
    start.column = 1;
    Some(start)
}

/// Capture each top-level list item's original source through the next item.
#[must_use]
pub(crate) fn task_item_sources(
    list: &List,
    items: &[&ListItem],
    source: &str,
) -> Vec<Option<String>> {
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let item_position = item.position.as_ref()?;
            let end = items
                .get(index + 1)
                .and_then(|next| next.position.as_ref())
                .and_then(|position| line_start(position, source))
                .or_else(|| list.position.as_ref().map(|position| position.end.clone()))?;
            original_position_source(
                &Position {
                    start: item_position.start.clone(),
                    end,
                },
                source,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    //! Unit tests for exact source-span extraction.

    use markdown::{ParseOptions, mdast::Node, to_mdast};

    use super::original_node_source;

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
            Some("  - [ ] 1.1.1. Nested task.".to_owned())
        );
    }

    #[test]
    fn original_node_source_returns_none_for_missing_position() {
        let node = Node::Text(markdown::mdast::Text {
            value: "No position".to_owned(),
            position: None,
        });

        assert_eq!(original_node_source(&node, "No position"), None);
    }
}
