//! Unit tests for roadmap model invariants.

use super::{ItemIdentity, RenumberPlan, SourceId};
use crate::roadmap::{RoadmapAnchor, parse_anchor};

fn anchor(raw: &str) -> RoadmapAnchor {
    match parse_anchor(raw) {
        Ok(anchor) => anchor,
        Err(error) => panic!("test anchor should parse: {error}"),
    }
}

#[test]
fn renumber_plan_resolves_source_local_mapping() {
    let mut plan = RenumberPlan::default();
    let old = anchor("1.1.1");
    let new = anchor("2.1.1");

    plan.record_mapping(SourceId::Target, old, new);

    assert_eq!(plan.resolve(SourceId::Target, old), Some(new));
}

#[test]
fn renumber_plan_resolves_unique_cross_source_mapping() {
    let mut plan = RenumberPlan::default();
    let old = anchor("1.1.1");
    let new = anchor("2.1.1");

    plan.record_mapping(SourceId::Fragment, old, new);

    assert_eq!(plan.resolve_unique(old), Some(new));
}

#[test]
fn renumber_plan_rejects_ambiguous_cross_source_mapping() {
    let mut plan = RenumberPlan::default();
    let old = anchor("1.1.1");

    plan.record_mapping(SourceId::Target, old, anchor("2.1.1"));
    plan.record_mapping(SourceId::Fragment, old, anchor("3.1.1"));

    assert_eq!(plan.resolve_unique(old), None);
}

#[test]
fn renumber_plan_missing_source_local_mapping_returns_none() {
    let mut plan = RenumberPlan::default();
    let old = anchor("1.1.1");

    plan.record_mapping(SourceId::Fragment, old, anchor("2.1.1"));

    assert_eq!(plan.resolve(SourceId::Target, old), None);
}

#[test]
fn renumber_plan_prefers_the_source_local_mapping_for_references() {
    let mut plan = RenumberPlan::default();
    let old = anchor("1.1.1");

    plan.record_mapping(SourceId::Target, old, anchor("1.1.1"));
    plan.record_mapping(SourceId::Fragment, old, anchor("3.1.1"));

    assert_eq!(plan.resolve_reference(SourceId::Target, old), Some(old));
}

#[test]
fn renumber_plan_resolves_fragment_reference_through_the_cross_source_fallback() {
    let mut plan = RenumberPlan::default();
    let old = anchor("1.1.1");

    plan.record_mapping(SourceId::Target, old, anchor("2.1.1"));

    assert_eq!(
        plan.resolve_reference(SourceId::Fragment, old),
        Some(anchor("2.1.1"))
    );
}

#[test]
fn renumber_plan_withholds_the_cross_source_fallback_from_target_references() {
    let mut plan = RenumberPlan::default();
    let old = anchor("1.1.1");

    // The fragment task kept the anchor of the target identity a replace
    // retired without continuing it — a changed-number or multi-item
    // replacement. Target text must not resolve onto it, or a surviving
    // consumer would silently inherit a different item. A same-number
    // replacement grants an explicit continuity alias instead; the
    // cross-source fallback itself stays closed to target text.
    plan.record_mapping(SourceId::Fragment, old, anchor("1.1.1"));

    assert_eq!(plan.resolve_reference(SourceId::Target, old), None);
}

#[test]
fn renumber_plan_continuity_alias_resolves_target_text() {
    let mut plan = RenumberPlan::default();
    let retired = ItemIdentity {
        source: SourceId::Target,
        anchor: anchor("1.1.2"),
    };

    let recorded = plan.record_continuity(retired, anchor("1.1.2"));

    assert!(
        recorded,
        "an absent identity should accept a continuity alias"
    );
    assert_eq!(
        plan.resolve_reference(SourceId::Target, retired.anchor),
        Some(anchor("1.1.2")),
        "target text should resolve through the continuity alias"
    );
}

#[test]
fn renumber_plan_continuity_alias_is_not_a_cross_source_mapping() {
    let mut plan = RenumberPlan::default();
    let retired = ItemIdentity {
        source: SourceId::Target,
        anchor: anchor("1.1.2"),
    };

    plan.record_continuity(retired, anchor("1.1.2"));

    assert_eq!(
        plan.resolve_unique(retired.anchor),
        None,
        "the cross-source fallback must not see a continuity alias"
    );
    assert_eq!(
        plan.resolve_reference(SourceId::Fragment, retired.anchor),
        None,
        "fragment text must not inherit a target continuity alias"
    );
}

#[test]
fn renumber_plan_continuity_does_not_overwrite_a_surviving_mapping() {
    let mut plan = RenumberPlan::default();
    let retired = ItemIdentity {
        source: SourceId::Target,
        anchor: anchor("1.1.2"),
    };
    let survivor = anchor("1.1.3");

    plan.record_mapping(SourceId::Target, retired.anchor, survivor);
    let recorded = plan.record_continuity(retired, anchor("1.1.2"));

    assert!(
        !recorded,
        "a surviving identity's mapping must not be displaced"
    );
    assert_eq!(
        plan.resolve(SourceId::Target, retired.anchor),
        Some(survivor),
        "the surviving identity should keep its renumbered anchor"
    );
}

#[test]
fn renumber_plan_continuity_does_not_replace_an_existing_alias() {
    let mut plan = RenumberPlan::default();
    let retired = ItemIdentity {
        source: SourceId::Target,
        anchor: anchor("1.1.2"),
    };

    let first = plan.record_continuity(retired, anchor("2.1.2"));
    let second = plan.record_continuity(retired, anchor("3.1.2"));

    assert!(first, "the first alias should be recorded");
    assert!(!second, "a second alias must not overwrite the first");
    assert_eq!(
        plan.resolve(SourceId::Target, retired.anchor),
        Some(anchor("2.1.2")),
        "the first recorded alias should remain authoritative"
    );
}
