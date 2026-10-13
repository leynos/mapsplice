//! Unit tests for roadmap model invariants.

use super::{RenumberPlan, SourceId};
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
    // retired. Target text must not resolve onto it, or a surviving consumer
    // would silently inherit a different item.
    plan.record_mapping(SourceId::Fragment, old, anchor("1.1.1"));

    assert_eq!(plan.resolve_reference(SourceId::Target, old), None);
}
