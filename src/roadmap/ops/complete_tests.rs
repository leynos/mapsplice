//! Unit tests for the in-place completion patcher.

use rstest::rstest;

use super::{
    complete_item,
    test_support::{complete_ok, completion_source},
};
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

#[test]
fn completes_a_leaf_task_without_touching_anything_else() {
    let expected = ROADMAP.replace("- [ ] 1.1.1.", "- [x] 1.1.1.");

    assert_eq!(
        complete_ok(ROADMAP, "1.1.1", None).expect("fixture completion should succeed"),
        expected,
        "completion should change only the addressed checkbox"
    );
}

#[rstest]
#[case::a_depended_upon_task("1.1.1", "- [x] 1.1.1. First task.", "Second task. Requires 1.1.1.")]
#[case::an_addendum_sub_task(
    "1.1.3.1",
    "  - [x] 1.1.3.1. Addendum sub-task.",
    "- [ ] 1.1.3. Parent task."
)]
#[case::a_parent_task(
    "1.1.3",
    "- [x] 1.1.3. Parent task.",
    "  - [ ] 1.1.3.1. Addendum sub-task."
)]
fn completing_one_item_leaves_its_relatives_alone(
    #[case] anchor: &str,
    #[case] checked_line: &str,
    #[case] untouched_line: &str,
) {
    let completed = complete_ok(ROADMAP, anchor, None).expect("fixture completion should succeed");

    assert!(
        completed.contains(checked_line),
        "the addressed item should be checked: {completed}"
    );
    assert!(
        completed.contains(untouched_line),
        "the related item must keep its state: {completed}"
    );
}

#[test]
fn completing_twice_equals_completing_once() {
    let source = "## 1. Phase one\n\n### 1.1. Step one\n\n- [ ] 1.1.1. Finish the scope.\n";

    let first =
        complete_ok(source, "1.1.1", Some("— done")).expect("fixture completion should succeed");
    let second =
        complete_ok(&first, "1.1.1", Some("— done")).expect("fixture completion should succeed");

    assert_eq!(second, first, "the second run should be byte-identical");
}

#[test]
fn an_already_checked_item_is_untouched_even_with_a_new_suffix() {
    let source = "## 1. Phase one\n\n### 1.1. Step one\n\n- [x] 1.1.1. Finish the scope.\n";

    assert_eq!(
        complete_ok(source, "1.1.1", Some("— done")).expect("fixture completion should succeed"),
        source,
        "a checked item must not gain a suffix on a later run"
    );
}

#[test]
fn a_checkbox_like_sequence_in_the_title_is_not_the_checkbox() {
    let source = "## 1. Phase one\n\n### 1.1. Step one\n\n- [ ] 1.1.1. Copy [ ] markers.\n";
    let expected = "## 1. Phase one\n\n### 1.1. Step one\n\n- [x] 1.1.1. Copy [ ] markers.\n";

    assert_eq!(
        complete_ok(source, "1.1.1", None).expect("fixture completion should succeed"),
        expected,
        "only the item's own marker should change"
    );
}

#[test]
fn a_tab_marked_unchecked_item_is_completed() {
    let source = completion_source("- [\t] 1.1.1. Finish the scope.");
    let expected = completion_source("- [x] 1.1.1. Finish the scope.");

    assert_eq!(
        complete_ok(&source, "1.1.1", None).expect("fixture completion should succeed"),
        expected,
        "a tab, which the parser accepts as unchecked, should become checked"
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
        complete_ok(source, "1.1.1", Some("— done")).expect("fixture completion should succeed"),
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
        complete_ok(source, "1.1.1.1", Some("— done")).expect("fixture completion should succeed"),
        expected,
        "a nested addendum should wrap beneath its own content column"
    );
}

/// One document line-ending scenario.
struct LineEndingCase {
    /// Line ending used by the document.
    ending: &'static str,
    /// Expected text around the inserted break.
    inserted_break: &'static str,
    /// Text that would reveal the wrong line ending.
    forbidden: Option<&'static str>,
}

#[rstest]
#[case::crlf(LineEndingCase {
    ending: "\r\n",
    inserted_break: "output —\r\n      done.",
    forbidden: Some("output —\n"),
})]
#[case::cr(LineEndingCase {
    ending: "\r",
    inserted_break: "output —\r      done.",
    forbidden: None,
})]
fn inserted_breaks_use_the_document_line_ending(#[case] case: LineEndingCase) {
    let source = format!(
        concat!(
            "## 1. Phase one{0}{0}### 1.1. Step one{0}{0}",
            "- [ ] 1.1.1. A wrapped task title that continues{0}",
            "  keeps the dependent task reference visible in the rendered roadmap output.{0}",
        ),
        case.ending
    );

    let completed =
        complete_ok(&source, "1.1.1", Some("— done")).expect("fixture completion should succeed");

    assert!(
        completed.contains(case.inserted_break),
        "the inserted break should use the document line ending: {completed:?}"
    );
    if let Some(forbidden) = case.forbidden {
        assert!(
            !completed.contains(forbidden),
            "the inserted break must not use another line ending: {completed:?}"
        );
    }
}

#[test]
fn multibyte_titles_keep_their_bytes_intact() {
    let source = "## 1. Phase one\n\n### 1.1. Step one\n\n- [ ] 1.1.1. Café — naïve résumé.\n";
    let expected =
        "## 1. Phase one\n\n### 1.1. Step one\n\n- [x] 1.1.1. Café — naïve résumé — done.\n";

    assert_eq!(
        complete_ok(source, "1.1.1", Some("— done")).expect("fixture completion should succeed"),
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

#[rstest]
#[case::task("9.9.9")]
#[case::sub_task_with_absent_parent("9.9.9.1")]
#[case::sub_task_with_present_parent("1.1.1.1")]
fn a_missing_anchor_reports_the_requested_anchor(#[case] anchor: &str) {
    let roadmap = parse_roadmap(ROADMAP).expect("fixture roadmap should parse");
    let requested = parse_anchor(anchor).expect("fixture anchor should parse");

    let error = complete_item(&roadmap, ROADMAP, requested, None)
        .expect_err("an absent anchor should be rejected");

    match error {
        MapspliceError::AnchorNotFound { anchor: reported } => assert_eq!(
            reported, requested,
            "the diagnostic should name the anchor that was asked for"
        ),
        other => panic!("expected an anchor-not-found error, got {other:?}"),
    }
}
