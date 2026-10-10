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
        "- [ ] 1.1.1. Task.\n\n  ```text\n  1. first\n  1. second\n  ```\n\n  1. first\n  3. third"
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
