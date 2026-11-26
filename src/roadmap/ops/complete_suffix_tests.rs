//! Unit tests for done-suffix detection, placement, and wrapping.

use rstest::rstest;

use super::test_support::{CompletionCase, assert_completion_case, complete_ok, completion_source};

#[rstest]
#[case::terminal_punctuation_is_preserved(CompletionCase::new(
    "- [ ] 1.1.1. Finish the scope.",
    Some("— done"),
    "- [x] 1.1.1. Finish the scope — done."
))]
#[case::punctuated_suffix_replaces_title_punctuation(CompletionCase::new(
    "- [ ] 1.1.1. Finish the scope.",
    Some("— done."),
    "- [x] 1.1.1. Finish the scope — done."
))]
#[case::existing_suffix_is_not_duplicated(CompletionCase::new(
    "- [ ] 1.1.1. Finish the scope — done.",
    Some("— done"),
    "- [x] 1.1.1. Finish the scope — done."
))]
#[case::suffix_keeps_its_punctuation_when_the_title_has_none(CompletionCase::new(
    "- [ ] 1.1.1. Finish",
    Some("— done."),
    "- [x] 1.1.1. Finish — done."
))]
#[case::suffix_word_inside_a_longer_word_does_not_match(CompletionCase::new(
    "- [ ] 1.1.1. Get it undone.",
    Some("done"),
    "- [x] 1.1.1. Get it undone done."
))]
#[case::formatted_suffix_is_recognized_after_insertion(CompletionCase::new(
    "- [ ] 1.1.1. Finish",
    Some("**done**"),
    "- [x] 1.1.1. Finish **done**"
))]
#[case::literal_trailing_marker_does_not_hide_a_missing_suffix(CompletionCase::new(
    "- [ ] 1.1.1. Remove every build artefact named build*",
    Some("build"),
    "- [x] 1.1.1. Remove every build artefact named build* build"
))]
#[case::marker_carrying_suffix_passes_its_own_validation(CompletionCase::new(
    "- [ ] 1.1.1. Remove the artefact",
    Some("build*"),
    "- [x] 1.1.1. Remove the artefact build*"
))]
#[case::existing_literal_marker_suffix_is_not_duplicated(CompletionCase::new(
    "- [ ] 1.1.1. Remove every artefact named build*",
    Some("build*"),
    "- [x] 1.1.1. Remove every artefact named build*"
))]
#[case::existing_suffix_with_other_punctuation_is_not_duplicated(CompletionCase::new(
    "- [ ] 1.1.1. Finish — done!",
    Some("— done."),
    "- [x] 1.1.1. Finish — done!"
))]
#[case::existing_marker_suffix_with_other_punctuation_is_not_duplicated(CompletionCase::new(
    "- [ ] 1.1.1. **Finish — done!**",
    Some("**done.**"),
    "- [x] 1.1.1. **Finish — done!**"
))]
fn suffix_placement(#[case] case: CompletionCase) { assert_completion_case(&case); }

#[rstest]
#[case::suffix_stays_inside_closing_markers(CompletionCase::new(
    "- [ ] 1.1.1. **Finish the scope.**",
    Some("— done"),
    "- [x] 1.1.1. **Finish the scope — done.**"
))]
#[case::existing_suffix_inside_closing_markers_is_not_duplicated(CompletionCase::new(
    "- [ ] 1.1.1. **Finish the scope — done.**",
    Some("— done"),
    "- [x] 1.1.1. **Finish the scope — done.**"
))]
#[case::punctuated_suffix_replaces_marker_punctuation(CompletionCase::new(
    "- [ ] 1.1.1. **Finish the scope.**",
    Some("done."),
    "- [x] 1.1.1. **Finish the scope done.**"
))]
#[case::unpunctuated_marker_suffix_follows_closing_markers(CompletionCase::new(
    "- [ ] 1.1.1. **Finish the scope**",
    Some("**done**"),
    "- [x] 1.1.1. **Finish the scope** **done**"
))]
#[case::marker_suffix_precedes_plain_terminal_punctuation(CompletionCase::new(
    "- [ ] 1.1.1. Finish the scope.",
    Some("**done**"),
    "- [x] 1.1.1. Finish the scope **done**."
))]
#[case::punctuated_marker_suffix_keeps_plain_punctuation(CompletionCase::new(
    "- [ ] 1.1.1. Finish the scope.",
    Some("**done.**"),
    "- [x] 1.1.1. Finish the scope **done**."
))]
fn suffix_around_closing_inline_markers(#[case] case: CompletionCase) {
    assert_completion_case(&case);
}

#[rstest]
#[case::bare_inline_code(CompletionCase::new(
    "- [ ] 1.1.1. Update `foo`",
    Some("— done"),
    "- [x] 1.1.1. Update `foo` — done"
))]
#[case::inline_code_inside_emphasis(CompletionCase::new(
    "- [ ] 1.1.1. **Update `foo`**",
    Some("— done"),
    "- [x] 1.1.1. **Update `foo` — done**"
))]
fn suffix_stays_out_of_inline_code(#[case] case: CompletionCase) { assert_completion_case(&case); }

#[rstest]
#[case::formatted_punctuated_suffix_keeps_title_punctuation(CompletionCase::new(
    "- [ ] 1.1.1. Finish the scope.",
    Some("**done.**"),
    "- [x] 1.1.1. Finish the scope **done**."
))]
#[case::escaped_full_stop_is_title_content(CompletionCase::new(
    "- [ ] 1.1.1. Match the literal scope\\.",
    Some("— done"),
    "- [x] 1.1.1. Match the literal scope\\. — done"
))]
#[case::trailing_literal_backslash_stays_before_the_suffix(CompletionCase::new(
    "- [ ] 1.1.1. Finish the path segment Task\\",
    Some("— done"),
    "- [x] 1.1.1. Finish the path segment Task\\ — done"
))]
#[case::a_backslash_does_not_escape_an_ellipsis(CompletionCase::new(
    "- [ ] 1.1.1. Finish the scope\\…",
    Some("— done"),
    "- [x] 1.1.1. Finish the scope\\ — done…"
))]
#[case::an_unpunctuated_link_suffix_keeps_the_title_punctuation(CompletionCase::new(
    "- [ ] 1.1.1. Finish the scope.",
    Some("[done](https://example.org)"),
    "- [x] 1.1.1. Finish the scope [done](https://example.org)."
))]
#[case::a_question_mark_survives_a_punctuated_suffix(CompletionCase::new(
    "- [ ] 1.1.1. Is it ready?",
    Some("— done."),
    "- [x] 1.1.1. Is it ready — done?"
))]
#[case::a_punctuation_only_title_keeps_its_terminal(CompletionCase::new(
    "- [ ] 1.1.1. ?",
    Some("— done."),
    "- [x] 1.1.1. — done?"
))]
#[case::a_question_mark_survives_a_marker_suffix(CompletionCase::new(
    "- [ ] 1.1.1. Is it ready?",
    Some("**done.**"),
    "- [x] 1.1.1. Is it ready **done**?"
))]
fn punctuation_handling_respects_formatting_and_escapes(#[case] case: CompletionCase) {
    assert_completion_case(&case);
}

#[rstest]
#[case::literal_tilde_ending_a_url_stays_before_the_suffix(CompletionCase::new(
    "- [ ] 1.1.1. Visit https://example.org/~",
    Some("— done"),
    "- [x] 1.1.1. Visit https://example.org/~ — done"
))]
#[case::literal_asterisk_ending_a_glob_stays_before_the_suffix(CompletionCase::new(
    "- [ ] 1.1.1. Remove every build artefact named build*",
    Some("— done"),
    "- [x] 1.1.1. Remove every build artefact named build* — done"
))]
#[case::strike_span_receives_the_suffix_inside(CompletionCase::new(
    "- [ ] 1.1.1. ~~Finish the scope~~",
    Some("— done"),
    "- [x] 1.1.1. ~~Finish the scope — done~~"
))]
fn suffix_placement_uses_the_parsed_inline_structure(#[case] case: CompletionCase) {
    assert_completion_case(&case);
}

#[rstest]
#[case::suffix_outside_closing_markers_is_not_duplicated(CompletionCase::new(
    "- [ ] 1.1.1. **Finish the scope — done**.",
    Some("— done"),
    "- [x] 1.1.1. **Finish the scope — done**."
))]
#[case::suffix_after_an_opening_marker_is_not_duplicated(CompletionCase::new(
    "- [ ] 1.1.1. Finish **done**",
    Some("done"),
    "- [x] 1.1.1. Finish **done**"
))]
#[case::a_suffix_word_inside_marked_text_still_needs_a_boundary(CompletionCase::new(
    "- [ ] 1.1.1. Get it **undone**",
    Some("done"),
    "- [x] 1.1.1. Get it **undone done**"
))]
fn suffix_detection_across_inline_markers(#[case] case: CompletionCase) {
    assert_completion_case(&case);
}

#[rstest]
#[case::a_colon_separates_the_suffix(CompletionCase::new(
    "- [ ] 1.1.1. Status:done.",
    Some("done"),
    "- [x] 1.1.1. Status:done."
))]
#[case::an_unspaced_dash_separates_the_suffix(CompletionCase::new(
    "- [ ] 1.1.1. Finish the scope—done.",
    Some("done"),
    "- [x] 1.1.1. Finish the scope—done."
))]
#[case::an_underscore_binds_the_suffix_word(CompletionCase::new(
    "- [ ] 1.1.1. Keep foo_done.",
    Some("done"),
    "- [x] 1.1.1. Keep foo_done done."
))]
fn suffix_detection_treats_punctuation_as_a_word_boundary(#[case] case: CompletionCase) {
    assert_completion_case(&case);
}

#[test]
fn a_punctuated_suffix_inside_markers_wraps_within_eighty_columns() {
    let item = format!("- [ ] 1.1.1. **{}.**", "a".repeat(58));
    let completed = complete_ok(&completion_source(&item), "1.1.1", Some("done."))
        .expect("fixture completion should succeed");
    let longest = completed
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);

    assert!(
        longest <= 80,
        "every completed line must stay within 80 columns: {completed}"
    );
    assert!(
        completed.contains("      done.**"),
        "the suffix should wrap before the preserved closing markers: {completed}"
    );
}

#[test]
fn a_marker_suffix_keeps_the_title_punctuation_within_eighty_columns() {
    let content = "a".repeat(57);
    let item = format!("- [ ] 1.1.1. {content}.");
    let expected = format!("- [x] 1.1.1. {content} **done**.");

    assert_eq!(
        complete_ok(&completion_source(&item), "1.1.1", Some("**done.**"))
            .expect("fixture completion should succeed"),
        completion_source(&expected),
        "the suffix core and the kept full stop should fit one line"
    );
    assert!(
        expected.chars().count() <= 80,
        "the fixture should stay within 80 columns: {expected}"
    );
}

#[test]
fn a_title_already_ending_with_the_suffix_word_keeps_it_once() {
    let source = "## 1. Phase one\n\n### 1.1. Step one\n\n- [ ] 1.1.1. Ship it done.\n";
    let expected = "## 1. Phase one\n\n### 1.1. Step one\n\n- [x] 1.1.1. Ship it done.\n";

    assert_eq!(
        complete_ok(source, "1.1.1", Some("done")).expect("fixture completion should succeed"),
        expected,
        "an existing whole-word suffix should not be repeated"
    );
}
