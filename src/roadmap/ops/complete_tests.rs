//! Unit tests for the in-place completion patcher.

use rstest::rstest;

use super::{complete_item, test_support::complete_ok};
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
