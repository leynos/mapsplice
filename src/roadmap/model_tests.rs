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
