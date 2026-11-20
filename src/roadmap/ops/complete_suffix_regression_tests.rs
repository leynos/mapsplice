//! Regression tests for suffix parsing, prefix boundaries, and wrapping.
//!
//! Each test pins a defect found in review. Block-shaped suffixes were read as
//! block syntax and skipped, an entity-encoded prefix separator defeated the
//! numbered prefix boundary, and a wrapped insertion let a literal trailing
//! backslash escape the inserted line break. Encoded terminal punctuation was
//! left ahead of the suffix, inline HTML that renders nothing was treated as a
//! word boundary, a suffix word ending in a backslash could lose it at a wrap
//! point, a suffix ending in a backslash could escape the punctuation that
//! followed it, and an unbreakable suffix word wider than a continuation line
//! could overflow the column limit.

use rstest::rstest;

use super::{
    super::complete_suffix::title_has_suffix,
    test_support::{complete_ok, completion_source},
};
use crate::{
    error::MapspliceError,
    roadmap::{model::MarkdownNodes, parse_roadmap},
};

/// Return the parsed summary of the first task in a completed roadmap.
fn first_task_summary(source: &str) -> MarkdownNodes {
    let roadmap = parse_roadmap(source).expect("completed roadmap should parse");
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
    task.summary.clone()
}

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
    assert!(
        title_has_suffix(&first_task_summary(&completed), "Task\\ — done"),
        "the parsed title must keep the literal backslash: {completed}"
    );
}

#[test]
fn a_wrapped_suffix_pairs_its_own_trailing_backslash() {
    let content = "a".repeat(60);
    let source = completion_source(&format!("- [ ] 1.1.1. {content}"));
    let expected = completion_source(&format!("- [x] 1.1.1. {content} Task\\\\\n      done"));

    let completed = complete_ok(&source, "1.1.1", Some("Task\\ done"))
        .expect("fixture completion should succeed");

    assert_eq!(
        completed, expected,
        "the inserted break must not let the suffix backslash escape the newline"
    );
    assert!(
        title_has_suffix(&first_task_summary(&completed), "Task\\ done"),
        "the parsed title must keep the literal backslash: {completed}"
    );
}

#[rstest]
#[case::a_title_full_stop(
    "- [ ] 1.1.1. Finish the scope.",
    "- [x] 1.1.1. Finish the scope done\\\\."
)]
#[case::a_full_stop_inside_closing_markers(
    "- [ ] 1.1.1. **Finish the scope.**",
    "- [x] 1.1.1. **Finish the scope done\\\\.**"
)]
fn a_trailing_suffix_backslash_is_paired_before_punctuation(
    #[case] item: &str,
    #[case] expected_item: &str,
) {
    let completed = complete_ok(&completion_source(item), "1.1.1", Some("done\\"))
        .expect("fixture completion should succeed");

    assert_eq!(
        completed,
        completion_source(expected_item),
        "the backslash must stay literal before the punctuation it precedes"
    );
    assert!(
        title_has_suffix(&first_task_summary(&completed), "done\\"),
        "the completed title must already carry the configured suffix: {completed}"
    );
}

#[test]
fn a_trailing_suffix_backslash_at_the_end_of_the_document_stays_single() {
    let source = "## 1. Phase one\n\n### 1.1. Step one\n\n- [ ] 1.1.1. Finish the scope";
    let expected = "## 1. Phase one\n\n### 1.1. Step one\n\n- [x] 1.1.1. Finish the scope done\\";

    let completed =
        complete_ok(source, "1.1.1", Some("done\\")).expect("fixture completion should succeed");

    assert_eq!(
        completed, expected,
        "nothing follows the suffix at the end of the document, so no pairing is needed"
    );
    assert!(
        title_has_suffix(&first_task_summary(&completed), "done\\"),
        "the parsed title must keep the literal backslash: {completed}"
    );
}

#[test]
fn a_backslash_that_cannot_be_paired_fails_closed() {
    let content = "a".repeat(61);
    let source = completion_source(&format!("- [ ] 1.1.1. {content}"));

    let error = complete_ok(&source, "1.1.1", Some("Task\\ done"))
        .expect_err("a line at the column limit must fail closed");

    assert!(
        matches!(error, MapspliceError::UnsafeCompletionEdit { .. }),
        "the failure must be a typed unsafe-edit error: {error:?}"
    );
}

#[test]
fn a_suffix_word_too_wide_for_a_continuation_line_fails_closed() {
    let suffix = "a".repeat(75);
    let source = completion_source("- [ ] 1.1.1. Finish the scope");

    let error = complete_ok(&source, "1.1.1", Some(&suffix))
        .expect_err("an unbreakable overlong word must fail closed");

    assert!(
        matches!(error, MapspliceError::UnsafeCompletionEdit { .. }),
        "the failure must be a typed unsafe-edit error: {error:?}"
    );
}

#[test]
fn a_suffix_word_that_exactly_fills_a_continuation_line_is_allowed() {
    let suffix = "a".repeat(74);
    let expected = completion_source(&format!("- [x] 1.1.1. Finish the scope\n      {suffix}"));

    let completed = complete_ok(
        &completion_source("- [ ] 1.1.1. Finish the scope"),
        "1.1.1",
        Some(&suffix),
    )
    .expect("a word fitting the column limit should complete");

    assert_eq!(
        completed, expected,
        "the continuation line must carry the word at the hanging indent"
    );
    assert!(
        completed.lines().all(|line| line.chars().count() <= 80),
        "every completed line must stay within 80 columns: {completed}"
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
#[case::a_hyphenated_tag_name_is_not_a_break(
    "- [ ] 1.1.1. Get it un<br-widget></br-widget>done.",
    "- [x] 1.1.1. Get it un<br-widget></br-widget>done done."
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
