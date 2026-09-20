//! Preservation policy for unchanged Markdown parser nodes.

use markdown::mdast::Node;

use super::render_block;
use crate::{
    error::Result,
    roadmap::preservation_events::{CanonicalFallbackReason, PreservationRenderOutcome},
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

/// Stateful linear scan for formatter-unstable task-list content.
struct TaskListStabilityScan {
    open_fence: Option<Fence>,
    checklist_indent: Option<usize>,
    previous_line_was_blank: bool,
    in_indented_code: bool,
    previous_marker: Option<(usize, ListMarker)>,
    marker_indents: Vec<usize>,
    has_unstable_code_fence: bool,
}

impl TaskListStabilityScan {
    /// Create a scan configured for the task-list indentation in `original`.
    fn new(original: &str) -> Self {
        Self {
            open_fence: None,
            checklist_indent: checklist_marker_indent(original),
            previous_line_was_blank: false,
            in_indented_code: false,
            previous_marker: None,
            marker_indents: Vec::new(),
            has_unstable_code_fence: false,
        }
    }

    /// Scan one source line and return whether it makes list formatting unstable.
    fn scan_line(&mut self, line: &str) -> bool {
        let line_is_blank = line.trim().is_empty();
        if self.open_fence.is_some() && self.is_fenced_line(line) {
            self.previous_line_was_blank = line_is_blank;
            return false;
        }
        if self.is_indented_code_line(line) {
            self.previous_line_was_blank = line_is_blank;
            return false;
        }
        if self.is_fenced_line(line) {
            self.previous_line_was_blank = line_is_blank;
            return false;
        }
        self.previous_line_was_blank = line_is_blank;
        list_marker(line).is_some_and(|marker| self.is_unstable_marker(marker))
    }

    /// Return whether `line` is part of a fenced code region.
    fn is_fenced_line(&mut self, line: &str) -> bool {
        if let Some(opening) = self.open_fence {
            if closes_fence(line, opening) {
                self.open_fence = None;
            }
            return true;
        }
        let Some(opening) = fence(line) else {
            return false;
        };
        self.has_unstable_code_fence |= opening.character == '~' || opening.length >= 4;
        self.open_fence = Some(opening);
        true
    }

    /// Return whether `line` belongs to an indented-code region in a task body.
    fn is_indented_code_line(&mut self, line: &str) -> bool {
        if self.in_indented_code {
            if line.trim().is_empty() || is_indented_code_block_line(line, self.checklist_indent) {
                return true;
            }
            self.in_indented_code = false;
        }
        if self.previous_line_was_blank && is_indented_code_block_line(line, self.checklist_indent)
        {
            self.in_indented_code = true;
            return true;
        }
        false
    }

    /// Update the linear marker state and report a formatter-unstable marker.
    fn is_unstable_marker(&mut self, marker: (usize, ListMarker)) -> bool {
        let (indent, kind) = marker;
        let has_unstable_order = matches!(
            (self.previous_marker, kind),
            (
                Some((previous_indent, ListMarker::Ordered(previous))),
                ListMarker::Ordered(current),
            ) if previous_indent == indent && current != previous + 1
        );
        while self
            .marker_indents
            .last()
            .is_some_and(|parent| *parent >= indent)
        {
            self.marker_indents.pop();
        }
        let has_overindented_nesting = indent > 0
            && self
                .marker_indents
                .last()
                .is_some_and(|parent| indent > parent + 2);
        self.marker_indents.push(indent);
        self.previous_marker = Some(marker);
        has_unstable_order || has_overindented_nesting
    }
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
) -> Result<(String, PreservationRenderOutcome)> {
    formatter_fallback_reason(original).map_or_else(
        || {
            Ok((
                trim_preserved_separator(original).to_owned(),
                PreservationRenderOutcome::PreservedSource,
            ))
        },
        |reason| {
            canonical().map(|rendered| {
                (
                    rendered,
                    PreservationRenderOutcome::CanonicalFallback(reason),
                )
            })
        },
    )
}

/// Return the first closed formatter-stability reason requiring canonical output.
fn formatter_fallback_reason(original: &str) -> Option<CanonicalFallbackReason> {
    let mut scan = TaskListStabilityScan::new(original);
    for line in original.lines() {
        if scan.scan_line(line) {
            return Some(CanonicalFallbackReason::UnstableListMarker);
        }
    }
    scan.has_unstable_code_fence
        .then_some(CanonicalFallbackReason::UnstableCodeFence)
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
    let mut scan = TaskListStabilityScan::new(original);
    original.lines().any(|line| scan.scan_line(line))
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

    use super::{
        formatter_fallback_reason,
        has_unstable_code_fence,
        has_unstable_list_marker,
        trim_preserved_separator,
    };

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
        let source = concat!(
            "- [ ] 1.1.1. Task.\n\n",
            "          1. first\n",
            "          3. second\n",
            "          1. third\n",
            "          ~~~"
        );
        assert!(!has_unstable_list_marker(source));
        assert!(formatter_fallback_reason(source).is_none());
    }

    #[test]
    fn ordered_markers_after_a_fence_are_not_indented_code() {
        let source = concat!(
            "- [ ] 1.1.1. Task.\n\n",
            "  ```text\n",
            "  fenced content\n",
            "  ```\n",
            "          1. first marker\n",
            "          3. non-contiguous marker"
        );
        assert!(has_unstable_list_marker(source));
        assert_eq!(
            formatter_fallback_reason(source),
            Some(super::CanonicalFallbackReason::UnstableListMarker)
        );
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
