//! Preservation policy for unchanged Markdown parser nodes.

use markdown::mdast::Node;

use super::render_block;
use crate::error::Result;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ListMarker {
    Ordered(u32),
    Unordered,
}

/// Render an unchanged node from preserved source unless policy requires
/// canonical output.
pub(super) fn render_preserved_or_canonical(
    node: &Node,
    original: &str,
    indent: usize,
) -> Result<String> {
    if is_formatter_unstable(node, original) {
        render_block(node, indent)
    } else {
        Ok(trim_preserved_separator(original).to_owned())
    }
}

fn trim_preserved_separator(original: &str) -> &str { original.trim_end_matches('\n') }

fn is_formatter_unstable(node: &Node, original: &str) -> bool {
    match node {
        Node::List(_) => has_unstable_list_marker(original),
        Node::Code(_) => has_unstable_code_fence(original),
        _ => false,
    }
}

fn has_unstable_code_fence(original: &str) -> bool {
    let opener = original.lines().find(|line| !line.trim().is_empty());
    opener.is_some_and(|line| {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        indent <= 3 && (trimmed.starts_with("~~~") || trimmed.starts_with("````"))
    })
}

fn has_unstable_list_marker(original: &str) -> bool {
    let markers = original.lines().filter_map(list_marker).collect::<Vec<_>>();
    has_repeated_or_noncontiguous_ordered_marker(&markers)
        || has_overindented_nested_marker(&markers)
}

fn has_repeated_or_noncontiguous_ordered_marker(markers: &[(usize, ListMarker)]) -> bool {
    markers.windows(2).any(|window| {
        let [
            (first_indent, ListMarker::Ordered(first)),
            (second_indent, ListMarker::Ordered(second)),
        ] = window
        else {
            return false;
        };
        first_indent == second_indent && *second != first + 1
    })
}

fn has_overindented_nested_marker(markers: &[(usize, ListMarker)]) -> bool {
    markers.iter().enumerate().any(|(index, (indent, _))| {
        *indent > 0
            && markers
                .iter()
                .take(index)
                .rev()
                .find_map(|(candidate_indent, _)| {
                    (*candidate_indent < *indent).then_some(*candidate_indent)
                })
                .is_some_and(|parent_indent| *indent > parent_indent + 2)
    })
}

fn list_marker(line: &str) -> Option<(usize, ListMarker)> {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    if trimmed.starts_with("- ") && !trimmed.starts_with("- [") {
        return Some((indent, ListMarker::Unordered));
    }

    let (digits, rest) = trimmed.split_once('.')?;
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    rest.starts_with(' ')
        .then(|| {
            digits
                .parse()
                .ok()
                .map(|ordinal| (indent, ListMarker::Ordered(ordinal)))
        })
        .flatten()
}

#[cfg(test)]
mod tests {
    //! Unit tests for preserved source cleanup.

    use super::{has_unstable_code_fence, has_unstable_list_marker, trim_preserved_separator};

    #[test]
    fn trims_block_separator_newlines() {
        assert_eq!(
            trim_preserved_separator("- first\n\n- second\n"),
            "- first\n\n- second"
        );
    }

    #[test]
    fn leaves_inner_loose_list_spacing_intact() {
        assert_eq!(
            trim_preserved_separator("- first\n\n- second"),
            "- first\n\n- second"
        );
    }

    #[test]
    fn repeated_ordered_markers_are_formatter_unstable() {
        assert!(has_unstable_list_marker("1. first\n1. second"));
    }

    #[test]
    fn non_contiguous_ordered_markers_are_formatter_unstable() {
        assert!(has_unstable_list_marker("1. first\n3. third"));
    }

    #[test]
    fn overindented_nested_list_is_formatter_unstable() {
        assert!(has_unstable_list_marker("- parent\n    - child"));
    }

    #[test]
    fn loose_list_spacing_is_formatter_stable() {
        assert!(!has_unstable_list_marker("- first\n\n- second"));
    }

    #[test]
    fn tilde_fence_is_formatter_unstable() {
        assert!(has_unstable_code_fence("~~~rust\nlet answer = 42;\n~~~"));
    }

    #[test]
    fn oversized_backtick_fence_is_formatter_unstable() {
        assert!(has_unstable_code_fence("````rust\nlet answer = 42;\n````"));
    }

    #[test]
    fn indented_triple_backtick_fence_is_formatter_stable() {
        assert!(!has_unstable_code_fence(
            " ```rust\n let answer = 42;\n ```"
        ));
    }
}
