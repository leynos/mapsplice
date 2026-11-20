//! Regression tests for suffix parsing, prefix boundaries, and wrapping.
//!
//! Each test pins a defect found in review. Block-shaped suffixes were read as
//! block syntax and skipped, an entity-encoded prefix separator defeated the
//! numbered prefix boundary, and a wrapped insertion let a literal trailing
//! backslash escape the inserted line break. Encoded terminal punctuation was
//! left ahead of the suffix, and inline HTML that renders nothing was treated
//! as a word boundary.

use rstest::rstest;

use super::{
    super::complete_suffix::title_has_suffix,
    test_support::{complete_ok, completion_source},
};
use crate::roadmap::parse_roadmap;

#[rstest]
#[case::a_heading_marker_stays_literal(
    "- [ ] 1.1.1. Finish done.",
    Some("# done"),
    "- [x] 1.1.1. Finish done # done."
)]
#[case::a_bullet_marker_stays_literal(
    "- [ ] 1.1.1. Finish done.",
    Some("- done"),
    "- [x] 1.1.1. Finish done - done."
)]
fn block_shaped_suffixes_are_treated_as_inline_text(
    #[case] item: &str,
    #[case] suffix: Option<&str>,
    #[case] expected_item: &str,
) {
    assert_eq!(
        complete_ok(&completion_source(item), "1.1.1", suffix)
            .expect("fixture completion should succeed"),
        completion_source(expected_item),
        "block syntax in a configured suffix must not hide it from insertion"
    );
}

#[test]
fn an_encoded_prefix_separator_still_takes_a_suffix() {
    let item = "- [ ] 1.1.1&#46; Finish the scope.";
    let expected = "- [x] 1.1.1&#46; Finish the scope — done.";

    assert_eq!(
        complete_ok(&completion_source(item), "1.1.1", Some("— done"))
            .expect("fixture completion should succeed"),
        completion_source(expected),
        "the prefix boundary must not depend on the separator's spelling"
    );
}

#[test]
fn a_wrapped_suffix_preserves_a_literal_trailing_backslash() {
    let content = format!("{} Task\\", "a".repeat(60));
    let source = completion_source(&format!("- [ ] 1.1.1. {content}"));
    let expected = completion_source(&format!("- [x] 1.1.1. {content}\\\n      — done"));

    let completed =
        complete_ok(&source, "1.1.1", Some("— done")).expect("fixture completion should succeed");

    assert_eq!(
        completed, expected,
        "the inserted break must not let the backslash escape the newline"
    );
    let roadmap = parse_roadmap(&completed).expect("completed roadmap should parse");
    let phase = roadmap
        .phases
        .first()
        .expect("the completed roadmap should keep its phase");
    let step = phase
        .steps
        .first()
        .expect("the completed roadmap should keep its step");
    let task = step
        .tasks()
        .first()
        .expect("the completed roadmap should keep its task");
    assert!(
        title_has_suffix(&task.summary, "Task\\ — done"),
        "the parsed title must keep the literal backslash: {completed}"
    );
}

#[rstest]
#[case::an_encoded_full_stop_stays_last(
    "- [ ] 1.1.1. Finish the scope&#46;",
    "— done",
    "- [x] 1.1.1. Finish the scope — done&#46;"
)]
#[case::an_encoded_ellipsis_stays_last(
    "- [ ] 1.1.1. Finish the scope&hellip;",
    "— done",
    "- [x] 1.1.1. Finish the scope — done&hellip;"
)]
#[case::a_punctuated_suffix_replaces_an_encoded_full_stop(
    "- [ ] 1.1.1. Finish the scope&#46;",
    "— done.",
    "- [x] 1.1.1. Finish the scope — done."
)]
#[case::an_escaped_reference_is_literal_text(
    "- [ ] 1.1.1. Finish the scope\\&#46;",
    "— done",
    "- [x] 1.1.1. Finish the scope\\&#46; — done"
)]
fn character_references_count_as_their_rendered_punctuation(
    #[case] item: &str,
    #[case] suffix: &str,
    #[case] expected_item: &str,
) {
    assert_eq!(
        complete_ok(&completion_source(item), "1.1.1", Some(suffix))
            .expect("fixture completion should succeed"),
        completion_source(expected_item),
        "encoded terminal punctuation must keep its place after the suffix"
    );
}

#[rstest]
#[case::an_empty_span_joins_the_surrounding_word(
    "- [ ] 1.1.1. Get it un<span></span>done.",
    "- [x] 1.1.1. Get it un<span></span>done done."
)]
#[case::a_break_tag_separates_the_surrounding_words(
    "- [ ] 1.1.1. Get it un<br>done.",
    "- [x] 1.1.1. Get it un<br>done."
)]
fn inline_html_is_a_boundary_only_when_it_renders_a_break(
    #[case] item: &str,
    #[case] expected_item: &str,
) {
    assert_eq!(
        complete_ok(&completion_source(item), "1.1.1", Some("done"))
            .expect("fixture completion should succeed"),
        completion_source(expected_item),
        "only inline HTML that renders a break may separate words"
    );
}
