//! Successive-command consistency for the public roadmap mutation API.
//!
//! [`apply_command`] mutates a parsed document in place, so a caller may apply
//! several operations without rendering and re-parsing between them. Every
//! committed operation re-anchors item identities to the numbers it rendered,
//! and these regressions pin that behaviour for the two cases where stale
//! anchors used to surface on the next command: an in-place replacement that
//! continues an identity, and an ordinary renumbering shift.

use mapsplice::{
    Result,
    RoadmapDocument,
    RoadmapOperation,
    apply_command,
    parse_anchor,
    parse_fragment,
    parse_roadmap,
    render_roadmap,
};

const CONSUMER_TARGET: &str = concat!(
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. Consumer. Requires 1.1.2.\n",
    "- [x] 1.1.2. Depended-on task.\n",
);

const CHECKED_OFF_FRAGMENT: &str = "- [x] 1.1.2. Revised depended-on task.\n";
const INSERTED_TASK_FRAGMENT: &str = "- [ ] 1.1.1. Inserted task.\n";
const APPENDED_PHASE_FRAGMENT: &str = concat!(
    "## 2. Appended phase\n\n",
    "### 2.1. Appended step\n\n",
    "- [ ] 2.1.1. Appended task.\n",
);

/// A second command still resolves a reference to an identity continued by an
/// in-place replacement.
#[test]
fn append_after_in_place_replace_resolves_the_continued_identity() {
    let mut document = parse_roadmap(CONSUMER_TARGET).expect("test roadmap should parse");
    let replacement = parse_fragment(CHECKED_OFF_FRAGMENT).expect("test fragment should parse");
    let anchor = parse_anchor("1.1.2").expect("test anchor should parse");
    apply_command(
        &mut document,
        RoadmapOperation::Replace { anchor },
        Some(replacement),
    )
    .expect("the in-place replacement should succeed");
    append_phase(&mut document).expect("appending a phase should succeed");

    let output = render_roadmap(&document).expect("mutated roadmap should render");
    assert!(
        output.contains("- [ ] 1.1.1. Consumer. Requires 1.1.2.\n"),
        "the consumer clause should survive a later command, got:\n{output}"
    );
    assert!(
        output.contains("- [x] 1.1.2. Revised depended-on task.\n"),
        "the replacement should still occupy 1.1.2, got:\n{output}"
    );
}

/// A second command still resolves references rewritten by a renumbering
/// insertion, even though the first command changed every affected number.
#[test]
fn append_after_renumbering_insert_resolves_rewritten_references() {
    let mut document = parse_roadmap(CONSUMER_TARGET).expect("test roadmap should parse");
    let insertion = parse_fragment(INSERTED_TASK_FRAGMENT).expect("test fragment should parse");
    let anchor = parse_anchor("1.1.1").expect("test anchor should parse");
    apply_command(
        &mut document,
        RoadmapOperation::Insert {
            anchor,
            after: false,
        },
        Some(insertion),
    )
    .expect("the insertion should succeed");
    append_phase(&mut document).expect("appending a phase should succeed");

    let output = render_roadmap(&document).expect("mutated roadmap should render");
    assert!(
        output.contains("- [ ] 1.1.2. Consumer. Requires 1.1.3.\n"),
        "the rewritten consumer clause should survive a later command, got:\n{output}"
    );
}

/// Append one phase through the public mutation API.
fn append_phase(document: &mut RoadmapDocument) -> Result<()> {
    let fragment = parse_fragment(APPENDED_PHASE_FRAGMENT)?;
    apply_command(document, RoadmapOperation::Append, Some(fragment)).map(|_| ())
}
