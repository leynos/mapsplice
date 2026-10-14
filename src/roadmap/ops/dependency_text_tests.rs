//! Unit coverage for dependency-reference classification branches.

use rstest::rstest;

use super::{DependencyReferenceClassification, classify_dependency_reference, rewrite_text_value};
use crate::roadmap::{
    model::{RenumberPlan, SourceId},
    parse_anchor,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExpectedClassification {
    Reference(&'static str),
    InvalidDependencyToken,
    NotDependencyReference,
}

#[rstest]
#[case::space_separator("Requires 1.2.3", "1.2.3", ExpectedClassification::Reference("1.2.3"))]
#[case::colon_separator("Requires: 1.2.3", "1.2.3", ExpectedClassification::Reference("1.2.3"))]
#[case::hard_wrapped_separator(
    "Requires\n  1.2.3",
    "1.2.3",
    ExpectedClassification::Reference("1.2.3")
)]
#[case::crlf_wrapped_separator(
    "Requires:\r\n  1.2.3",
    "1.2.3",
    ExpectedClassification::Reference("1.2.3")
)]
#[case::comma_separated(
    "Requires 1.2.3, 2.3.4",
    "2.3.4",
    ExpectedClassification::Reference("2.3.4")
)]
#[case::version_like_zero(
    "Requires 1.4.0",
    "1.4.0",
    ExpectedClassification::InvalidDependencyToken
)]
#[case::valid_unresolved_shape(
    "Requires 99.1.1",
    "99.1.1",
    ExpectedClassification::Reference("99.1.1")
)]
#[case::section_sigil("Requires §1.2", "1.2", ExpectedClassification::NotDependencyReference)]
#[case::dependency_reference_section_sigil_in_nearby_prose(
    "See §2.1. Requires 2.1.1",
    "2.1",
    ExpectedClassification::NotDependencyReference
)]
#[case::outside_dependency_clause("See 1.2", "1.2", ExpectedClassification::NotDependencyReference)]
#[case::sentence_terminated(
    "Requires 1.2. Then 2.3",
    "2.3",
    ExpectedClassification::NotDependencyReference
)]
#[case::dependency_reference_semicolon_terminated(
    "Requires 1.2; Then 2.3",
    "2.3",
    ExpectedClassification::NotDependencyReference
)]
#[case::greedy_four_level(
    "Requires 1.2.17.1",
    "1.2.17.1",
    ExpectedClassification::Reference("1.2.17.1")
)]
fn dependency_reference_classification_handles_candidate_context(
    #[case] value: &str,
    #[case] candidate: &str,
    #[case] expected_case: ExpectedClassification,
) {
    let start = value
        .find(candidate)
        .expect("test candidate should appear in the value");
    let end = start + candidate.len();
    let expected = match expected_case {
        ExpectedClassification::Reference(raw) => DependencyReferenceClassification::Reference(
            parse_anchor(raw).expect("test anchor should parse"),
        ),
        ExpectedClassification::InvalidDependencyToken => {
            DependencyReferenceClassification::InvalidDependencyToken
        }
        ExpectedClassification::NotDependencyReference => {
            DependencyReferenceClassification::NotDependencyReference
        }
    };

    assert_eq!(classify_dependency_reference(value, start, end), expected);
}

#[rstest]
#[case::letter_prefix("Requires A1.2", "1.2")]
#[case::letter_suffix("Requires 1.2B", "1.2")]
fn dependency_reference_classification_rejects_alphanumeric_boundaries(
    #[case] value: &str,
    #[case] candidate: &str,
) {
    let start = value
        .find(candidate)
        .expect("test candidate should appear in the value");
    let end = start + candidate.len();

    assert_eq!(
        classify_dependency_reference(value, start, end),
        DependencyReferenceClassification::NotDependencyReference,
    );
}

#[rstest]
#[case::dependency_reference_rewrites_multiple_moved_references(
    "Requires 2.1.1, 2.1.1.",
    "Requires 1.1.1, 1.1.1.",
    2
)]
#[case::hard_wrapped_requires_rewrites_moved_references(
    "Requires:\r\n  2.1.1,\n  2.1.1.",
    "Requires:\r\n  1.1.1,\n  1.1.1.",
    2
)]
#[case::dependency_reference_preserves_invalid_version_token(
    "Requires 1.4.0.",
    "Requires 1.4.0.",
    0
)]
#[case::dependency_reference_preserves_section_reference("Requires §2.1.", "Requires §2.1.", 0)]
#[case::dependency_reference_preserves_prose_number(
    "Count 27. Requires none.",
    "Count 27. Requires none.",
    0
)]
#[case::dependency_reference_ignores_blocks_clause("Blocks 2.1.1.", "Blocks 2.1.1.", 0)]
fn dependency_reference_text_rewrite_scopes_mapped_references(
    #[case] value: &str,
    #[case] expected: &str,
    #[case] expected_count: u64,
) {
    let mut plan = RenumberPlan::default();
    plan.record_mapping(
        SourceId::Target,
        parse_anchor("2.1.1").expect("old test anchor should parse"),
        parse_anchor("1.1.1").expect("new test anchor should parse"),
    );

    let report = rewrite_text_value(value, SourceId::Target, &plan);

    assert_eq!(report.value, expected);
    assert_eq!(report.rewrite_count, expected_count);
    assert_eq!(report.unresolved, []);
}

#[test]
fn dependency_reference_reports_unresolved_valid_reference() {
    let plan = RenumberPlan::default();

    let report = rewrite_text_value("Requires 99.1.1.", SourceId::Target, &plan);

    assert_eq!(report.value, "Requires 99.1.1.");
    assert_eq!(report.rewrite_count, 0);
    assert_eq!(
        report.unresolved,
        [parse_anchor("99.1.1").expect("test anchor should parse")]
    );
}
