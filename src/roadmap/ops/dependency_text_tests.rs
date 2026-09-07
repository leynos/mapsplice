//! Regression coverage for dependency source-preservation invalidation.

use super::{
    super::{
        model::{RenumberPlan, SourceId},
        parse_anchor,
    },
    dependency_text::rewrite_text_value,
};

#[test]
fn dependency_reference_does_not_count_an_unchanged_mapping() {
    let mut plan = RenumberPlan::default();
    let anchor = parse_anchor("1.1.1").expect("test anchor should parse");
    plan.record_mapping(SourceId::Target, anchor, anchor);

    let report = rewrite_text_value("Requires 1.1.1.", SourceId::Target, &plan);

    assert_eq!(report.value, "Requires 1.1.1.");
    assert_eq!(report.rewrite_count, 0);
    assert_eq!(report.unresolved, []);
}
