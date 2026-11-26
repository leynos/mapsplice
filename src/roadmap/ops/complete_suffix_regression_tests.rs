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
    test_support::{CompletionCase, assert_completion_case, complete_ok, completion_source},
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

/// One wrapped-backslash scenario.
struct WrappedBackslashCase {
    /// Text already ending the title before the inserted suffix.
    item_tail: &'static str,
    /// Configured suffix.
    suffix: &'static str,
    /// Expected title bytes after the inserted words.
    expected_tail: &'static str,
    /// Suffix the completed title must parse as carrying.
    rendered_suffix: &'static str,
}

impl WrappedBackslashCase {
    /// Build one scenario from its literal parts.
    const fn new(
        item_tail: &'static str,
        suffix: &'static str,
        expected_tail: &'static str,
        rendered_suffix: &'static str,
    ) -> Self {
        Self {
            item_tail,
            suffix,
            expected_tail,
            rendered_suffix,
        }
    }
}

#[rstest]
#[case::a_heading_marker_stays_literal(CompletionCase::new(
    "- [ ] 1.1.1. Finish done.",
    Some("# done"),
    "- [x] 1.1.1. Finish done # done."
))]
#[case::a_bullet_marker_stays_literal(CompletionCase::new(
    "- [ ] 1.1.1. Finish done.",
    Some("- done"),
    "- [x] 1.1.1. Finish done - done."
))]
#[case::an_encoded_prefix_separator_still_takes_a_suffix(CompletionCase::new(
    "- [ ] 1.1.1&#46; Finish the scope.",
    Some("— done"),
    "- [x] 1.1.1&#46; Finish the scope — done."
))]
fn block_shaped_suffixes_are_treated_as_inline_text(#[case] case: CompletionCase) {
    assert_completion_case(&case);
}

#[rstest]
#[case::a_title_backslash(WrappedBackslashCase::new(
    " Task\\",
    "— done",
    " Task\\\\\n      — done",
    "Task\\ — done"
))]
#[case::a_suffix_backslash(WrappedBackslashCase::new(
    "",
    "Task\\ done",
    " Task\\\\\n      done",
    "Task\\ done"
))]
fn a_wrapped_suffix_pairs_a_literal_backslash(#[case] case: WrappedBackslashCase) {
    let content = "a".repeat(60);
    let source = completion_source(&format!("- [ ] 1.1.1. {content}{}", case.item_tail));
    let expected = completion_source(&format!("- [x] 1.1.1. {content}{}", case.expected_tail));

    let completed = complete_ok(&source, "1.1.1", Some(case.suffix))
        .expect("fixture completion should succeed");

    assert_eq!(
        completed, expected,
        "the inserted break must not let a backslash escape the newline: {completed}"
    );
    assert!(
        title_has_suffix(&first_task_summary(&completed), case.rendered_suffix),
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
fn unsafe_suffix_layouts_fail_closed() {
    let near_limit = completion_source(&format!("- [ ] 1.1.1. {}", "a".repeat(61)));
    let overlong_word = "a".repeat(75);
    let short_title = completion_source("- [ ] 1.1.1. Finish the scope");

    for (source, suffix, reason) in [
        (near_limit, "Task\\ done", "a line at the column limit"),
        (
            short_title,
            overlong_word.as_str(),
            "an unbreakable overlong word",
        ),
    ] {
        let error = complete_ok(&source, "1.1.1", Some(suffix))
            .expect_err("a suffix that does not fit must fail closed");

        assert!(
            matches!(error, MapspliceError::UnsafeCompletionEdit { .. }),
            "{reason} must produce a typed unsafe-edit error: {error:?}"
        );
    }
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
#[case::an_encoded_full_stop_stays_last(CompletionCase::new(
    "- [ ] 1.1.1. Finish the scope&#46;",
    Some("— done"),
    "- [x] 1.1.1. Finish the scope — done&#46;"
))]
#[case::an_encoded_ellipsis_stays_last(CompletionCase::new(
    "- [ ] 1.1.1. Finish the scope&hellip;",
    Some("— done"),
    "- [x] 1.1.1. Finish the scope — done&hellip;"
))]
#[case::a_punctuated_suffix_replaces_an_encoded_full_stop(CompletionCase::new(
    "- [ ] 1.1.1. Finish the scope&#46;",
    Some("— done."),
    "- [x] 1.1.1. Finish the scope — done."
))]
#[case::an_escaped_reference_is_literal_text(CompletionCase::new(
    "- [ ] 1.1.1. Finish the scope\\&#46;",
    Some("— done"),
    "- [x] 1.1.1. Finish the scope\\&#46; — done"
))]
fn character_references_count_as_their_rendered_punctuation(#[case] case: CompletionCase) {
    assert_completion_case(&case);
}

#[rstest]
#[case::an_empty_span_joins_the_surrounding_word(CompletionCase::new(
    "- [ ] 1.1.1. Get it un<span></span>done.",
    Some("done"),
    "- [x] 1.1.1. Get it un<span></span>done done."
))]
#[case::a_break_tag_separates_the_surrounding_words(CompletionCase::new(
    "- [ ] 1.1.1. Get it un<br>done.",
    Some("done"),
    "- [x] 1.1.1. Get it un<br>done."
))]
#[case::a_hyphenated_tag_name_is_not_a_break(CompletionCase::new(
    "- [ ] 1.1.1. Get it un<br-widget></br-widget>done.",
    Some("done"),
    "- [x] 1.1.1. Get it un<br-widget></br-widget>done done."
))]
fn inline_html_is_a_boundary_only_when_it_renders_a_break(#[case] case: CompletionCase) {
    assert_completion_case(&case);
}
