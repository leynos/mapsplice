//! Unit tests for the in-place completion patcher.

use rstest::rstest;

use super::complete_item;
use crate::{
    error::MapspliceError,
    roadmap::{RoadmapItemLevel, parse_anchor, parse_roadmap},
};

const ROADMAP: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. First task.\n",
    "- [ ] 1.1.2. Second task. Requires 1.1.1.\n",
    "- [ ] 1.1.3. Parent task.\n",
    "  - [ ] 1.1.3.1. Addendum sub-task.\n",
);

/// Complete `anchor` in `source`, failing the test on error.
fn complete_ok(source: &str, anchor: &str, suffix: Option<&str>) -> String {
    let roadmap = parse_roadmap(source).expect("fixture roadmap should parse");
    let parsed_anchor = parse_anchor(anchor).expect("fixture anchor should parse");
    complete_item(&roadmap, source, parsed_anchor, suffix).expect("completion should succeed")
}

#[test]
fn completes_a_leaf_task_without_touching_anything_else() {
    let expected = ROADMAP.replace("- [ ] 1.1.1.", "- [x] 1.1.1.");

    assert_eq!(
        complete_ok(ROADMAP, "1.1.1", None),
        expected,
        "completion should change only the addressed checkbox"
    );
}

#[test]
fn completing_a_depended_upon_task_leaves_its_dependants_alone() {
    let completed = complete_ok(ROADMAP, "1.1.1", None);

    assert!(
        completed.contains("- [x] 1.1.1. First task."),
        "the depended-upon task should be checked: {completed}"
    );
    assert!(
        completed.contains("Second task. Requires 1.1.1."),
        "the dependant's Requires clause must survive: {completed}"
    );
}

#[test]
fn completes_an_addendum_sub_task_only() {
    let completed = complete_ok(ROADMAP, "1.1.3.1", None);

    assert!(
        completed.contains("  - [x] 1.1.3.1. Addendum sub-task."),
        "the addendum should be checked: {completed}"
    );
    assert!(
        completed.contains("- [ ] 1.1.3. Parent task."),
        "the parent task must stay unchecked: {completed}"
    );
}

#[test]
fn completing_a_parent_task_leaves_its_addendum_alone() {
    let completed = complete_ok(ROADMAP, "1.1.3", None);

    assert!(
        completed.contains("- [x] 1.1.3. Parent task."),
        "the parent task should be checked: {completed}"
    );
    assert!(
        completed.contains("  - [ ] 1.1.3.1. Addendum sub-task."),
        "the addendum must stay unchecked: {completed}"
    );
}

/// Build a one-step roadmap around one task line.
fn completion_source(item: &str) -> String {
    format!("## 1. Phase one\n\n### 1.1. Step one\n\n{item}\n")
}

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
fn completing_twice_equals_completing_once() {
    let source = "## 1. Phase one\n\n### 1.1. Step one\n\n- [ ] 1.1.1. Finish the scope.\n";

    let first = complete_ok(source, "1.1.1", Some("— done"));
    let second = complete_ok(&first, "1.1.1", Some("— done"));

    assert_eq!(second, first, "the second run should be byte-identical");
}

#[test]
fn an_already_checked_item_is_untouched_even_with_a_new_suffix() {
    let source = "## 1. Phase one\n\n### 1.1. Step one\n\n- [x] 1.1.1. Finish the scope.\n";

    assert_eq!(
        complete_ok(source, "1.1.1", Some("— done")),
        source,
        "a checked item must not gain a suffix on a later run"
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

#[test]
fn a_checkbox_like_sequence_in_the_title_is_not_the_checkbox() {
    let source = "## 1. Phase one\n\n### 1.1. Step one\n\n- [ ] 1.1.1. Copy [ ] markers.\n";
    let expected = "## 1. Phase one\n\n### 1.1. Step one\n\n- [x] 1.1.1. Copy [ ] markers.\n";

    assert_eq!(
        complete_ok(source, "1.1.1", None),
        expected,
        "only the item's own marker should change"
    );
}

#[test]
fn a_wrapped_title_wraps_the_suffix_with_the_title_hanging_indent() {
    let source = concat!(
        "## 1. Phase one\n\n### 1.1. Step one\n\n",
        "- [ ] 1.1.1. A wrapped task title that continues\n",
        "  keeps the dependent task reference visible in the rendered roadmap output.\n",
        "\n",
        "  - Requires 1.1.1.\n",
        "  - Success: easy.\n",
    );
    let expected = concat!(
        "## 1. Phase one\n\n### 1.1. Step one\n\n",
        "- [x] 1.1.1. A wrapped task title that continues\n",
        "  keeps the dependent task reference visible in the rendered roadmap output —\n",
        "      done.\n",
        "\n",
        "  - Requires 1.1.1.\n",
        "  - Success: easy.\n",
    );

    assert_eq!(
        complete_ok(source, "1.1.1", Some("— done")),
        expected,
        "only the final title line should be refilled, at six spaces of hanging indent"
    );
}

#[test]
fn an_addendum_suffix_uses_eight_spaces_of_hanging_indent() {
    let source = concat!(
        "## 1. Phase one\n\n### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  - [ ] 1.1.1.1. Addendum title under the limit until the suffix lands here.\n",
    );
    let expected = concat!(
        "## 1. Phase one\n\n### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  - [x] 1.1.1.1. Addendum title under the limit until the suffix lands here —\n",
        "        done.\n",
    );

    assert_eq!(
        complete_ok(source, "1.1.1.1", Some("— done")),
        expected,
        "a nested addendum should wrap beneath its own content column"
    );
}

#[test]
fn crlf_line_endings_are_used_for_inserted_breaks() {
    let source = concat!(
        "## 1. Phase one\r\n\r\n### 1.1. Step one\r\n\r\n",
        "- [ ] 1.1.1. A wrapped task title that continues\r\n",
        "  keeps the dependent task reference visible in the rendered roadmap output.\r\n",
    );
    let completed = complete_ok(source, "1.1.1", Some("— done"));

    assert!(
        completed.contains("output —\r\n      done.\r\n"),
        "the inserted break should be CRLF: {completed:?}"
    );
    assert!(
        !completed.contains("output —\n"),
        "the inserted break must not be a bare LF: {completed:?}"
    );
}

#[test]
fn cr_only_line_endings_are_used_for_inserted_breaks() {
    let source = concat!(
        "## 1. Phase one\r\r### 1.1. Step one\r\r",
        "- [ ] 1.1.1. A wrapped task title that continues\r",
        "  keeps the dependent task reference visible in the rendered roadmap output.\r",
    );
    let completed = complete_ok(source, "1.1.1", Some("— done"));

    assert!(
        completed.contains("output —\r      done.\r"),
        "the inserted break should be a bare CR: {completed:?}"
    );
}

#[test]
fn multibyte_titles_keep_their_bytes_intact() {
    let source = "## 1. Phase one\n\n### 1.1. Step one\n\n- [ ] 1.1.1. Café — naïve résumé.\n";
    let expected =
        "## 1. Phase one\n\n### 1.1. Step one\n\n- [x] 1.1.1. Café — naïve résumé — done.\n";

    assert_eq!(
        complete_ok(source, "1.1.1", Some("— done")),
        expected,
        "byte offsets should not split multibyte characters"
    );
}

#[rstest]
#[case::phase("1", RoadmapItemLevel::Phase)]
#[case::step("1.1", RoadmapItemLevel::Step)]
fn phase_and_step_anchors_have_no_checkbox_to_complete(
    #[case] anchor: &str,
    #[case] expected_level: RoadmapItemLevel,
) {
    let roadmap = parse_roadmap(ROADMAP).expect("fixture roadmap should parse");
    let parsed_anchor = parse_anchor(anchor).expect("fixture anchor should parse");

    let error = complete_item(&roadmap, ROADMAP, parsed_anchor, None)
        .expect_err("phase and step anchors should be rejected");

    match error {
        MapspliceError::UnsupportedAnchorLevel {
            anchor: reported_anchor,
            level,
        } => {
            assert_eq!(level, expected_level, "unexpected anchor level");
            assert_eq!(reported_anchor, parsed_anchor, "unexpected rejected anchor");
        }
        other => panic!("expected an unsupported anchor level error, got {other:?}"),
    }
}

#[test]
fn a_missing_anchor_reports_anchor_not_found() {
    let roadmap = parse_roadmap(ROADMAP).expect("fixture roadmap should parse");
    let anchor = parse_anchor("9.9.9").expect("fixture anchor should parse");

    let error = complete_item(&roadmap, ROADMAP, anchor, None)
        .expect_err("an absent anchor should be rejected");

    assert!(
        matches!(error, MapspliceError::AnchorNotFound { .. }),
        "expected an anchor-not-found error, got {error:?}"
    );
}
