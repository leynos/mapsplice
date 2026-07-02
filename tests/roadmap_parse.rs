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
