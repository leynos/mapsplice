//! Regression tests for suffix parsing, prefix boundaries, and wrapping.
//!
//! Each test pins a defect found in review. Block-shaped suffixes were read as
//! block syntax and skipped, an entity-encoded prefix separator defeated the
//! numbered prefix boundary, and a wrapped insertion let a literal trailing
//! backslash escape the inserted line break.

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
