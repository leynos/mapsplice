//! Shared recognition of numbered checkbox items.
//!
//! The parser and the export adapter must agree on which nested checkbox items
//! carry roadmap numbering. Both call this module so that a document the parser
//! accepts cannot be rejected later by the exporter, and so that a document the
//! exporter rejects is refused at parse time with a source location.

use markdown::mdast::{List, ListItem, Node, Paragraph};

/// Return the dotted numeric prefix of a checkbox item's opening paragraph.
///
/// The prefix is the text before the first `. ` separator when it consists only
/// of ASCII digits and dots and contains at least one dot. That is the same
/// anchor grammar [`crate::roadmap::parse_anchor`] accepts, so a prefix this
/// function reports is one the exporter will try to interpret as a number.
///
/// # Examples
///
/// A paragraph whose text begins `1.2.3. Detail` reports `Some("1.2.3")`,
/// while `Detail` and `1.2.3 Detail` (no separator) report `None`.
pub(crate) fn numbered_prefix(paragraph: &Paragraph) -> Option<&str> {
    let Some(Node::Text(text)) = paragraph.children.first() else {
        return None;
    };
    let (prefix, _) = text.value.split_once(". ")?;
    is_dotted_numeric(prefix).then_some(prefix)
}

/// Return whether a list item is a checkbox whose opening paragraph is numbered.
pub(crate) fn is_numbered_checkbox(item: &ListItem) -> bool {
    item.checked.is_some()
        && matches!(
            item.children.first(),
            Some(Node::Paragraph(paragraph)) if numbered_prefix(paragraph).is_some()
        )
}

/// Return whether a list contains a numbered checkbox at any depth.
///
/// Sub-task bodies are opaque Markdown, so a numbered checkbox anywhere inside
/// them would be a roadmap structure the exporter cannot represent.
pub(crate) fn contains_numbered_checkbox(list: &List) -> bool {
    find_numbered_checkbox(list).is_some()
}

/// Return the first numbered checkbox nested beneath a list item.
///
/// A numbered checkbox directly in `list` belongs to that list's own level and
/// is not reported. Only items inside a child list are returned, which is the
/// shape the exporter rejects beneath a checkbox detail.
pub(crate) fn nested_numbered_checkbox(list: &List) -> Option<&ListItem> {
    list_items(list).find_map(|item| child_lists(item).find_map(find_numbered_checkbox))
}

/// Return the first numbered checkbox in a list or any of its descendants.
fn find_numbered_checkbox(list: &List) -> Option<&ListItem> {
    list_items(list).find_map(|item| {
        if is_numbered_checkbox(item) {
            Some(item)
        } else {
            child_lists(item).find_map(find_numbered_checkbox)
        }
    })
}

/// Iterate the list items of a list, ignoring any non-item children.
fn list_items(list: &List) -> impl Iterator<Item = &ListItem> {
    list.children.iter().filter_map(|node| match node {
        Node::ListItem(item) => Some(item),
        _ => None,
    })
}

/// Iterate the child lists of one list item.
fn child_lists(item: &ListItem) -> impl Iterator<Item = &List> {
    item.children.iter().filter_map(|node| match node {
        Node::List(list) => Some(list),
        _ => None,
    })
}

/// Return whether a prefix is a dotted numeric anchor such as `1.2.3`.
fn is_dotted_numeric(prefix: &str) -> bool {
    prefix.contains('.')
        && prefix
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.')
}

#[cfg(test)]
mod tests {
    //! Unit coverage for the shared numbered-checkbox predicates.

    use markdown::mdast::{List, ListItem, Node, Paragraph, Text};

    use super::{contains_numbered_checkbox, is_numbered_checkbox, nested_numbered_checkbox};

    /// Build a paragraph node with one text child.
    fn paragraph(value: &str) -> Node {
        Node::Paragraph(Paragraph {
            children: vec![Node::Text(Text {
                value: value.to_owned(),
                position: None,
            })],
            position: None,
        })
    }

    /// Build a list item with the supplied checkbox state and children.
    fn item(checked: Option<bool>, children: Vec<Node>) -> Node {
        Node::ListItem(ListItem {
            children,
            position: None,
            spread: false,
            checked,
        })
    }

    /// Build an unordered list from the supplied items.
    fn list(items: Vec<Node>) -> List {
        List {
            children: items,
            position: None,
            ordered: false,
            spread: false,
            start: None,
        }
    }

    #[test]
    fn numbered_checkbox_requires_a_checkbox_and_a_dotted_prefix() {
        let numbered = list(vec![item(Some(false), vec![paragraph("1.2.3. Detail")])]);
        let unnumbered = list(vec![item(Some(false), vec![paragraph("Detail")])]);
        let plain = list(vec![item(None, vec![paragraph("1.2.3. Detail")])]);

        let Some(Node::ListItem(item)) = numbered.children.first() else {
            panic!("expected a list item");
        };
        assert!(is_numbered_checkbox(item));
        assert!(!contains_numbered_checkbox(&unnumbered));
        assert!(!contains_numbered_checkbox(&plain));
    }

    #[test]
    fn nested_numbered_checkbox_ignores_the_lists_own_level() {
        let top_level = list(vec![item(Some(false), vec![paragraph("1.2.3. Detail")])]);
        let nested = list(vec![item(
            Some(false),
            vec![
                paragraph("Detail"),
                Node::List(list(vec![item(
                    Some(false),
                    vec![paragraph("1.2.4. Nested")],
                )])),
            ],
        )]);

        assert!(nested_numbered_checkbox(&top_level).is_none());
        assert!(nested_numbered_checkbox(&nested).is_some());
        assert!(contains_numbered_checkbox(&nested));
    }
}
