//! `rstest` coverage for roadmap anchor, fragment, and document parsing.

use mapsplice::{
    MapspliceError,
    RoadmapItemLevel,
    fragment_level,
    parse_anchor,
    parse_fragment_text,
    parse_roadmap_text,
};
use rstest::rstest;

fn invalid_roadmap_message(error: &MapspliceError) -> &str {
    match error {
        MapspliceError::InvalidRoadmap { message } => message,
        other => panic!("expected InvalidRoadmap error, got {other:?}"),
    }
}

#[rstest]
#[case("8", "8")]
#[case("8.2", "8.2")]
#[case("8.2.3", "8.2.3")]
#[case("8.2.3.4", "8.2.3.4")]
fn parse_anchor_accepts_supported_forms(#[case] raw: &str, #[case] expected: &str) {
    let anchor = parse_anchor(raw).expect("supported anchors should parse");
    assert_eq!(anchor.to_string(), expected);
}

#[rstest]
#[case("8.")]
#[case("8.2.")]
#[case("0")]
#[case("01")]
#[case("8.02")]
#[case("8.2.0")]
#[case("8.2.3.0")]
#[case("a.b")]
#[case("8.2.3.4.5")]
fn parse_anchor_rejects_invalid_forms(#[case] raw: &str) {
    let error = parse_anchor(raw).expect_err("invalid anchors must be rejected");
    assert!(matches!(error, MapspliceError::InvalidAnchor { .. }));
}

#[rstest]
#[case(
    "## 9. Phase\n\n### 9.1. Step\n\n- [ ] 9.1.1. Task.\n",
    RoadmapItemLevel::Phase
)]
#[case("### 9.2. Step\n\n- [ ] 9.2.1. Task.\n", RoadmapItemLevel::Step)]
#[case("- [ ] 9.9.9. Task.\n", RoadmapItemLevel::Task)]
#[case("- [ ] 9.9.9.9. Sub-task.\n", RoadmapItemLevel::SubTask)]
fn parse_fragment_detects_supported_level(
    #[case] fragment: &str,
    #[case] expected: RoadmapItemLevel,
) {
    let parsed = parse_fragment_text(fragment).expect("supported fragment should parse");
    assert_eq!(fragment_level(&parsed), expected);
}

#[rstest]
#[case::ordered_task_list(
    "1. [ ] 9.9.1. Ordered task.\n",
    "roadmap task lists must be unordered checklist items"
)]
#[case::non_checklist_task_item(
    "- 9.9.1. Missing checklist marker.\n",
    "roadmap task lists must be unordered checklist items"
)]
#[case::task_paragraph_without_plain_text(
    concat!("- [ ] 9.9.1. First task.\n", "- [ ] **9.9.2. Bold task.**\n"),
    "task paragraphs must start with plain text"
)]
#[case::sub_task_prefix_in_task_context(
    concat!("- [ ] 9.9.1. First task.\n", "- [ ] 9.9.2.1. Wrong level.\n"),
    "expected a task prefix in `9.9.2.1. Wrong level.`"
)]
fn parse_task_checklist_head_diagnostics(#[case] fragment: &str, #[case] expected: &str) {
    let error = parse_fragment_text(fragment).expect_err("malformed task item should fail");

    assert_eq!(invalid_roadmap_message(&error), expected);
}

#[rstest]
#[case::sub_task_paragraph_without_plain_text(
    concat!(
        "- [ ] 9.9.1.1. First sub-task.\n",
        "- [ ] **9.9.1.2. Bold sub-task.**\n",
    ),
    "sub-task paragraphs must start with plain text"
)]
#[case::task_prefix_in_sub_task_context(
    concat!("- [ ] 9.9.1.1. First sub-task.\n", "- [ ] 9.9.2. Wrong level.\n"),
    "expected a sub-task prefix in `9.9.2. Wrong level.`"
)]
fn parse_sub_task_fragment_checklist_head_diagnostics(
    #[case] fragment: &str,
    #[case] expected: &str,
) {
    let error = parse_fragment_text(fragment).expect_err("malformed sub-task item should fail");

    assert_eq!(invalid_roadmap_message(&error), expected);
}

#[rstest]
#[case::ordered_nested_sub_task_list(
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  1. [ ] 1.1.1.1. Ordered sub-task.\n",
    ),
    "roadmap sub-task lists must be unordered checklist items"
)]
#[case::non_checklist_nested_sub_task_item(
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  - 1.1.1.1. Missing checklist marker.\n",
    ),
    "roadmap sub-task lists must be unordered checklist items"
)]
fn parse_nested_sub_task_checklist_head_diagnostics(#[case] roadmap: &str, #[case] expected: &str) {
    let error = parse_roadmap_text(roadmap).expect_err("malformed sub-task item should fail");

    assert_eq!(invalid_roadmap_message(&error), expected);
}

#[rstest]
#[case::wrong_parent(
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  - [ ] 1.1.2.1. Wrong parent.\n",
    ),
    "sub-task `1.1.2.1` does not belong to task `1.1.1`"
)]
#[case::out_of_order(
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  - [ ] 1.1.1.1. First sub-task.\n",
        "  - [ ] 1.1.1.3. Third sub-task.\n",
    ),
    "sub-task `1.1.1.3` is not in document order"
)]
fn parse_sub_task_checklist_validation_diagnostics(#[case] roadmap: &str, #[case] expected: &str) {
    let error = parse_roadmap_text(roadmap).expect_err("malformed sub-task should fail");

    assert_eq!(invalid_roadmap_message(&error), expected);
}

#[rstest]
fn parse_roadmap_keeps_preamble_and_structure() {
    let roadmap = parse_roadmap_text(concat!(
        "# Example\n\n",
        "## Guiding principles\n\n",
        "- Be careful.\n\n",
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. First task.\n",
    ))
    .expect("supported roadmap should parse");

    assert_eq!(roadmap.preamble.len(), 3);
    assert_eq!(roadmap.phases.len(), 1);
    let first_phase = roadmap
        .phases
        .first()
        .expect("roadmap should contain one phase");
    assert_eq!(first_phase.steps.len(), 1);
    let first_step = first_phase
        .steps
        .first()
        .expect("phase should contain one step");
    assert_eq!(first_step.tasks.len(), 1);
}

#[rstest]
fn parse_roadmap_rejects_task_from_another_step() {
    let error = parse_roadmap_text(concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.2.1. Wrong step.\n",
    ))
    .expect_err("task numbers must belong to their containing step");

    assert_eq!(
        invalid_roadmap_message(&error),
        "task `1.2.1` does not belong to step `1.1`",
    );
}

#[rstest]
fn parse_step_fragment_rejects_task_from_another_step() {
    let error = parse_fragment_text(concat!("### 9.1. Step\n\n", "- [ ] 9.2.1. Wrong step.\n",))
        .expect_err("step fragments must reject tasks from another step");

    assert_eq!(
        invalid_roadmap_message(&error),
        "task `9.2.1` does not belong to step `9.1`",
    );
}

#[rstest]
#[case::task_fragment_with_trailing_paragraph(
    concat!("- [ ] 9.1.1. First.\n\n", "Trailing paragraph.\n"),
    "task fragments must contain only a single task list"
)]
#[case::sub_task_fragment_with_trailing_paragraph(
    concat!("- [ ] 9.1.1.1. First.\n\n", "Trailing paragraph.\n"),
    "sub-task fragments must contain only a single sub-task list"
)]
fn parse_single_list_fragments_reject_extra_root_nodes(
    #[case] fragment: &str,
    #[case] expected: &str,
) {
    let error = parse_fragment_text(fragment).expect_err("extra root nodes should fail");

    assert_eq!(invalid_roadmap_message(&error), expected);
}

#[rstest]
fn parse_task_fragment_keeps_sibling_step_diagnostic() {
    let error = parse_fragment_text(concat!("- [ ] 9.1.1. First.\n", "- [ ] 9.2.1. Second.\n",))
        .expect_err("task fragments must contain tasks from one step");

    assert_eq!(
        invalid_roadmap_message(&error),
        "task fragments must contain tasks from one step",
    );
}

#[rstest]
fn parse_sub_task_fragment_keeps_sibling_task_diagnostic() {
    let error = parse_fragment_text(concat!(
        "- [ ] 9.1.1.1. First.\n",
        "- [ ] 9.1.2.1. Second.\n",
    ))
    .expect_err("sub-task fragments must contain sub-tasks from one task");

    assert_eq!(
        invalid_roadmap_message(&error),
        "sub-task fragments must contain sub-tasks from one task",
    );
}

#[rstest]
#[case::non_step_heading(
    concat!("### 9.1. Step\n\n", "#### Non-step heading\n"),
    "step fragments must contain only step sections"
)]
#[case::task_after_trailing_content(
    concat!(
        "### 9.1. Step\n\n",
        "- [ ] 9.1.1. First.\n\n",
        "Trailing paragraph.\n\n",
        "- [ ] 9.1.2. Second.\n",
    ),
    "task list for step `9.1` cannot appear after trailing step content"
)]
#[case::sibling_step_from_another_phase(
    concat!("### 9.1. First step\n\n", "### 10.1. Second step\n"),
    "step fragments must contain steps from one phase"
)]
fn parse_step_fragment_keeps_lifecycle_diagnostics(#[case] fragment: &str, #[case] expected: &str) {
    let error = parse_fragment_text(fragment).expect_err("malformed step fragment should fail");

    assert_eq!(invalid_roadmap_message(&error), expected);
}
