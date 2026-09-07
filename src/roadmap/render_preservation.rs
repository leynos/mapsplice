//! Preservation policy for unchanged Markdown parser nodes.

use markdown::mdast::Node;

use super::render_block;
use crate::{
    error::Result,
    observability::{
        CanonicalFallbackReason,
        record_canonical_fallback,
        record_preserved_source_render,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ListMarker {
    Ordered(u32),
    Unordered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Fence {
    character: char,
    length: usize,
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

/// Render preserved task-item source unless formatting policy requires canonical output.
pub(super) fn render_preserved_task_or_canonical(
    original: &str,
    canonical: impl FnOnce() -> Result<String>,
) -> Result<String> {
    formatter_fallback_reason(original).map_or_else(
        || {
            record_preserved_source_render();
            Ok(trim_preserved_separator(original).to_owned())
        },
        |reason| {
            record_canonical_fallback(reason);
            canonical()
        },
    )
}

/// Return the first closed formatter-stability reason requiring canonical output.
fn formatter_fallback_reason(original: &str) -> Option<CanonicalFallbackReason> {
    if has_unstable_list_marker(original) {
        Some(CanonicalFallbackReason::UnstableListMarker)
    } else if has_unstable_code_fence(original) {
        Some(CanonicalFallbackReason::UnstableCodeFence)
    } else {
        None
    }
}

/// Remove only the separator newlines outside a preserved source span.
fn trim_preserved_separator(original: &str) -> &str { original.trim_end_matches('\n') }

/// Decide whether a node's preserved source requires canonical rendering.
fn is_formatter_unstable(node: &Node, original: &str) -> bool {
    match node {
        Node::List(_) => has_unstable_list_marker(original),
        Node::Code(_) => has_unstable_code_fence(original),
        _ => false,
    }
}

/// Detect fence forms whose formatting is not stable under the canonical renderer.
fn has_unstable_code_fence(original: &str) -> bool {
    let mut open_fence = None;
    original.lines().any(|line| {
        if let Some(fence) = open_fence {
            if closes_fence(line, fence) {
                open_fence = None;
            }
            return false;
        }
        let Some(fence) = fence(line) else {
            return false;
        };
        if fence.character == '~' || fence.length >= 4 {
            true
        } else {
            open_fence = Some(fence);
            false
        }
    })
}

/// Detect ordered or nested list markers that can change during formatting.
fn has_unstable_list_marker(original: &str) -> bool {
    let mut open_fence = None;
    let checklist_indent = checklist_marker_indent(original);
    let mut previous_line_was_blank = false;
    let markers = original
        .lines()
        .filter_map(|line| {
            if let Some(fence) = open_fence {
                if closes_fence(line, fence) {
                    open_fence = None;
                }
                return None;
            }
            if let Some(fence) = fence(line) {
                open_fence = Some(fence);
                return None;
            }
            if previous_line_was_blank && is_indented_code_block_line(line, checklist_indent) {
                previous_line_was_blank = false;
                return None;
            }
            previous_line_was_blank = line.trim().is_empty();
            list_marker(line)
        })
        .collect::<Vec<_>>();
    has_repeated_or_noncontiguous_ordered_marker(&markers)
        || has_overindented_nested_marker(&markers)
}

/// Return the leading indentation of the first task-list marker in `original`.
fn checklist_marker_indent(original: &str) -> Option<usize> {
    original.lines().find_map(|line| {
        let trimmed = line.trim_start();
        (trimmed.starts_with("- [ ") || trimmed.starts_with("- [x]"))
            .then_some(line.len() - trimmed.len())
    })
}

/// Return whether a blank-separated line is indented as a task item's code body.
fn is_indented_code_block_line(line: &str, checklist_indent: Option<usize>) -> bool {
    let trimmed = line.trim_start();
    checklist_indent.is_some_and(|indent| line.len() - trimmed.len() >= indent + 10)
}

/// Parse a Markdown fence opener from a line, if present.
fn fence(line: &str) -> Option<Fence> {
    let trimmed = line.trim_start();
    let character = trimmed.chars().next()?;
    matches!(character, '`' | '~').then_some(())?;
    let length = trimmed
        .chars()
        .take_while(|candidate| *candidate == character)
        .count();
    (length >= 3).then_some(Fence { character, length })
}

/// Return whether a line closes a fence with at least the opener's width.
fn closes_fence(line: &str, opening: Fence) -> bool {
    let trimmed = line.trim_start();
    let remainder = trimmed.trim_start_matches(opening.character);
    let length = trimmed.len() - remainder.len();
    length >= opening.length && remainder.trim().is_empty()
}

/// Detect adjacent ordered markers that repeat or skip an ordinal.
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

/// Detect nested markers indented beyond the supported canonical layout.
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

/// Parse a list marker and its leading indentation from a Markdown line.
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
    fn ordered_markers_inside_fenced_bodies_are_formatter_stable() {
        assert!(!has_unstable_list_marker(
            "- [ ] 1.1.1. Task.\n\n  ```text\n  1. first\n  1. second\n  3. third\n  ```"
        ));
    }

    #[test]
    fn ordered_markers_outside_fenced_bodies_are_formatter_unstable() {
        assert!(has_unstable_list_marker(
            "- [ ] 1.1.1. Task.\n\n  ```text\n  1. first\n  1. second\n  ```\n\n  1. first\n  3. \
             third"
        ));
    }

    #[test]
    fn ordered_markers_inside_indented_code_blocks_are_formatter_stable() {
        assert!(!has_unstable_list_marker(
            "- [ ] 1.1.1. Task.\n\n          1. first\n          3. third"
        ));
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
    fn nested_unstable_fences_are_formatter_unstable() {
        assert!(has_unstable_code_fence(
            "- [ ] 1.1.1. Task.\n\n  ~~~rust\n  let answer = 42;\n  ~~~"
        ));
        assert!(has_unstable_code_fence(
            "- [ ] 1.1.1. Task.\n\n  ````rust\n  let answer = 42;\n  ````"
        ));
    }

    #[test]
    fn canonical_fence_content_is_formatter_stable() {
        assert!(!has_unstable_code_fence(
            "- [ ] 1.1.1. Task.\n\n  ```text\n  ~~~\n  1. code item\n  ```"
        ));
    }

    #[test]
    fn indented_triple_backtick_fence_is_formatter_stable() {
        assert!(!has_unstable_code_fence(
            " ```rust\n let answer = 42;\n ```"
        ));
    }
}
