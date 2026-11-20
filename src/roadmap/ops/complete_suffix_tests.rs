//! Unit tests for done-suffix detection, placement, and wrapping.

use rstest::rstest;

use super::test_support::{complete_ok, completion_source};

#[rstest]
#[case::terminal_punctuation_is_preserved(
    "- [ ] 1.1.1. Finish the scope.",
    Some("— done"),
    "- [x] 1.1.1. Finish the scope — done.",
    "the done suffix should sit before the title's final full stop"
)]
#[case::punctuated_suffix_replaces_title_punctuation(
    "- [ ] 1.1.1. Finish the scope.",
    Some("— done."),
    "- [x] 1.1.1. Finish the scope — done.",
    "a punctuated suffix should not double the terminal punctuation"
)]
#[case::existing_suffix_is_not_duplicated(
    "- [ ] 1.1.1. Finish the scope — done.",
    Some("— done"),
    "- [x] 1.1.1. Finish the scope — done.",
    "an already suffixed title should only gain its check mark"
)]
#[case::suffix_word_inside_a_longer_word_does_not_match(
    "- [ ] 1.1.1. Get it undone.",
    Some("done"),
    "- [x] 1.1.1. Get it undone done.",
    "a suffix should only match at a word boundary"
)]
fn suffix_placement(
    #[case] item: &str,
    #[case] suffix: Option<&str>,
    #[case] expected_item: &str,
    #[case] message: &str,
) {
    assert_eq!(
        complete_ok(&completion_source(item), "1.1.1", suffix),
        completion_source(expected_item),
        "{message}"
    );
}

#[rstest]
#[case::suffix_stays_inside_closing_markers(
    "- [ ] 1.1.1. **Finish the scope.**",
    Some("— done"),
    "- [x] 1.1.1. **Finish the scope — done.**"
)]
#[case::existing_suffix_inside_closing_markers_is_not_duplicated(
    "- [ ] 1.1.1. **Finish the scope — done.**",
    Some("— done"),
    "- [x] 1.1.1. **Finish the scope — done.**"
)]
#[case::punctuated_suffix_replaces_marker_punctuation(
    "- [ ] 1.1.1. **Finish the scope.**",
    Some("done."),
    "- [x] 1.1.1. **Finish the scope done.**"
)]
fn suffix_around_closing_inline_markers(
    #[case] item: &str,
    #[case] suffix: Option<&str>,
    #[case] expected_item: &str,
) {
    assert_eq!(
        complete_ok(&completion_source(item), "1.1.1", suffix),
        completion_source(expected_item),
        "the suffix should stay inside the closing inline markers"
    );
}

#[rstest]
#[case::bare_inline_code("- [ ] 1.1.1. Update `foo`", "- [x] 1.1.1. Update `foo` — done")]
#[case::inline_code_inside_emphasis(
    "- [ ] 1.1.1. **Update `foo`**",
    "- [x] 1.1.1. **Update `foo` — done**"
)]
fn suffix_stays_out_of_inline_code(#[case] item: &str, #[case] expected_item: &str) {
    assert_eq!(
        complete_ok(&completion_source(item), "1.1.1", Some("— done")),
        completion_source(expected_item),
        "the suffix must follow a closing backtick, not extend the code span"
    );
}

#[rstest]
#[case::formatted_punctuated_suffix_replaces_title_punctuation(
    "- [ ] 1.1.1. Finish the scope.",
    Some("**done.**"),
    "- [x] 1.1.1. Finish the scope **done.**"
)]
#[case::escaped_full_stop_is_title_content(
    "- [ ] 1.1.1. Match the literal scope\\.",
    Some("— done"),
    "- [x] 1.1.1. Match the literal scope\\. — done"
)]
#[case::trailing_literal_backslash_stays_before_the_suffix(
    "- [ ] 1.1.1. Finish the path segment Task\\",
    Some("— done"),
    "- [x] 1.1.1. Finish the path segment Task\\ — done"
)]
fn punctuation_handling_respects_formatting_and_escapes(
    #[case] item: &str,
    #[case] suffix: Option<&str>,
    #[case] expected_item: &str,
) {
    assert_eq!(
        complete_ok(&completion_source(item), "1.1.1", suffix),
        completion_source(expected_item),
        "marker-wrapped and escaped punctuation must not be misread"
    );
}

#[rstest]
#[case::literal_tilde_ending_a_url_stays_before_the_suffix(
    "- [ ] 1.1.1. Visit https://example.org/~",
    "- [x] 1.1.1. Visit https://example.org/~ — done"
)]
#[case::literal_asterisk_ending_a_glob_stays_before_the_suffix(
    "- [ ] 1.1.1. Remove every build artefact named build*",
    "- [x] 1.1.1. Remove every build artefact named build* — done"
)]
#[case::strike_span_receives_the_suffix_inside(
    "- [ ] 1.1.1. ~~Finish the scope~~",
    "- [x] 1.1.1. ~~Finish the scope — done~~"
)]
fn suffix_placement_uses_the_parsed_inline_structure(
    #[case] item: &str,
    #[case] expected_item: &str,
) {
    assert_eq!(
        complete_ok(&completion_source(item), "1.1.1", Some("— done")),
        completion_source(expected_item),
        "literal trailing markers stay put while real closing markers move"
    );
}

#[rstest]
#[case::suffix_outside_closing_markers_is_not_duplicated(
    "- [ ] 1.1.1. **Finish the scope — done**.",
    Some("— done"),
    "- [x] 1.1.1. **Finish the scope — done**."
)]
#[case::suffix_after_an_opening_marker_is_not_duplicated(
    "- [ ] 1.1.1. Finish **done**",
    Some("done"),
    "- [x] 1.1.1. Finish **done**"
)]
#[case::a_suffix_word_inside_marked_text_still_needs_a_boundary(
    "- [ ] 1.1.1. Get it **undone**",
    Some("done"),
    "- [x] 1.1.1. Get it **undone done**"
)]
fn suffix_detection_across_inline_markers(
    #[case] item: &str,
    #[case] suffix: Option<&str>,
    #[case] expected_item: &str,
) {
    assert_eq!(
        complete_ok(&completion_source(item), "1.1.1", suffix),
        completion_source(expected_item),
        "markup must not hide an existing suffix or fabricate one inside a word"
    );
}

#[test]
fn a_formatted_suffix_is_recognized_after_insertion() {
    let expected = completion_source("- [x] 1.1.1. Finish **done**");

    assert_eq!(
        complete_ok(
            &completion_source("- [ ] 1.1.1. Finish"),
            "1.1.1",
            Some("**done**")
        ),
        expected,
        "a formatted suffix without title punctuation must pass validation"
    );
}

#[test]
fn a_punctuated_suffix_inside_markers_wraps_within_eighty_columns() {
    let item = format!("- [ ] 1.1.1. **{}.**", "a".repeat(58));
    let completed = complete_ok(&completion_source(&item), "1.1.1", Some("done."));
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
fn a_title_already_ending_with_the_suffix_word_keeps_it_once() {
    let source = "## 1. Phase one\n\n### 1.1. Step one\n\n- [ ] 1.1.1. Ship it done.\n";
    let expected = "## 1. Phase one\n\n### 1.1. Step one\n\n- [x] 1.1.1. Ship it done.\n";

    assert_eq!(
        complete_ok(source, "1.1.1", Some("done")),
        expected,
        "an existing whole-word suffix should not be repeated"
    );
}
