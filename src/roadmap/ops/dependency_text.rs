//! Dependency-clause text scanning for roadmap anchor rewrites.

use super::super::{model::RenumberPlan, parse_anchor};
use crate::roadmap::{RoadmapAnchor, model::SourceId};

/// Classification for an anchor-shaped candidate in a text value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DependencyReferenceClassification {
    Reference(RoadmapAnchor),
    InvalidDependencyToken,
    NotDependencyReference,
}

/// Dependency-reference rewrite details for one text value.
pub(super) struct DependencyRewriteReport {
    pub(super) value: String,
    pub(super) rewrite_count: u64,
    pub(super) unresolved: Vec<RoadmapAnchor>,
}

/// Rewrite anchor candidates within a text value and report rewrite details.
pub(super) fn rewrite_text_value(
    value: &str,
    source: SourceId,
    plan: &RenumberPlan,
) -> DependencyRewriteReport {
    let mut result = String::with_capacity(value.len());
    let mut rewrite_count = 0;
    let mut unresolved = Vec::new();
    let mut index = 0;

    while index < value.len() {
        let Some((start, end)) = next_anchor_candidate(value, index) else {
            if let Some(tail) = value.get(index..) {
                result.push_str(tail);
            }
            break;
        };
        if let Some(prefix) = value.get(index..start) {
            result.push_str(prefix);
        }
        let Some(candidate) = value.get(start..end) else {
            index = end;
            continue;
        };
        match classify_dependency_reference(value, start, end) {
            DependencyReferenceClassification::NotDependencyReference
            | DependencyReferenceClassification::InvalidDependencyToken => {
                result.push_str(candidate);
            }
            DependencyReferenceClassification::Reference(anchor) => {
                if let Some(mapped) = plan
                    .resolve(source, anchor)
                    .or_else(|| plan.resolve_unique(anchor))
                {
                    rewrite_count += 1;
                    result.push_str(&mapped.to_string());
                } else {
                    unresolved.push(anchor);
                    result.push_str(candidate);
                }
            }
        }
        index = end;
    }

    DependencyRewriteReport {
        value: result,
        rewrite_count,
        unresolved,
    }
}

/// Classify an anchor-shaped candidate by dependency context and anchor validity.
fn classify_dependency_reference(
    value: &str,
    start: usize,
    end: usize,
) -> DependencyReferenceClassification {
    if !has_anchor_candidate_boundaries(value, start, end)
        || !is_dependency_candidate_context(value, start)
    {
        return DependencyReferenceClassification::NotDependencyReference;
    }

    let Some(candidate) = value.get(start..end) else {
        return DependencyReferenceClassification::NotDependencyReference;
    };

    parse_anchor(candidate).map_or(
        DependencyReferenceClassification::InvalidDependencyToken,
        DependencyReferenceClassification::Reference,
    )
}

/// Return whether the candidate span has standalone anchor boundaries.
fn has_anchor_candidate_boundaries(value: &str, start: usize, end: usize) -> bool {
    let bytes = value.as_bytes();
    bytes
        .get(start)
        .is_some_and(|byte| is_anchor_start(bytes, start, *byte))
        && is_anchor_end(bytes, end)
}

/// Return whether the candidate is in a dependency context, not a section reference.
fn is_dependency_candidate_context(value: &str, start: usize) -> bool {
    !has_section_sigil(value, start) && is_dependency_anchor(value, start)
}

/// Return whether the candidate appears in a supported dependency clause.
fn is_dependency_anchor(value: &str, start: usize) -> bool {
    ["Requires"].into_iter().any(|keyword| {
        latest_keyword_before(value, start, keyword).is_some_and(|position| {
            is_keyword_boundary(value, position, keyword.len())
                && has_dependency_clause_separator(value, position + keyword.len(), start)
        })
    })
}

/// Return whether the candidate is immediately preceded by a section sigil.
fn has_section_sigil(value: &str, start: usize) -> bool {
    value
        .get(..start)
        .and_then(|prefix| prefix.chars().next_back())
        .is_some_and(|character| character == '§')
}

/// Find the latest dependency keyword before an anchor candidate.
fn latest_keyword_before(value: &str, start: usize, keyword: &str) -> Option<usize> {
    value
        .get(..start)?
        .rmatch_indices(keyword)
        .next()
        .map(|(position, _)| position)
}

/// Return whether a dependency keyword is bounded as a word.
fn is_keyword_boundary(value: &str, position: usize, length: usize) -> bool {
    let bytes = value.as_bytes();
    !bytes
        .get(position.wrapping_sub(1))
        .is_some_and(u8::is_ascii_alphanumeric)
        && !bytes
            .get(position + length)
            .is_some_and(u8::is_ascii_alphanumeric)
}

/// Return whether text between keyword and anchor still looks like a clause.
fn has_dependency_clause_separator(value: &str, after_keyword: usize, anchor_start: usize) -> bool {
    value
        .get(after_keyword..anchor_start)
        .is_some_and(|between| {
            !between.contains('\n')
                && !has_dependency_clause_terminator(between)
                && (between.contains(':') || between.starts_with(' '))
        })
}

/// Return whether text contains a dependency-clause terminator.
fn has_dependency_clause_terminator(value: &str) -> bool {
    value.char_indices().any(|(index, character)| {
        character == ';' || (character == '.' && !is_dot_between_digits(value.as_bytes(), index))
    })
}

/// Return whether a dot is part of a dotted numeric token.
fn is_dot_between_digits(bytes: &[u8], index: usize) -> bool {
    bytes
        .get(index.wrapping_sub(1))
        .is_some_and(u8::is_ascii_digit)
        && bytes.get(index + 1).is_some_and(u8::is_ascii_digit)
}

/// Find the next anchor-shaped token with leading and trailing boundaries.
fn next_anchor_candidate(value: &str, start_at: usize) -> Option<(usize, usize)> {
    let bytes = value.as_bytes();
    let mut start = start_at;

    while let Some(byte) = bytes.get(start) {
        if is_anchor_start(bytes, start, *byte) {
            let end = consume_anchor(bytes, start);
            if is_anchor_end(bytes, end) {
                return Some((start, end));
            }
        }
        start += 1;
    }

    None
}

/// Return whether an anchor candidate ends before another alphanumeric byte.
fn is_anchor_end(bytes: &[u8], end: usize) -> bool {
    !bytes.get(end).is_some_and(u8::is_ascii_alphanumeric)
}

/// Return whether a byte can start an anchor candidate.
fn is_anchor_start(bytes: &[u8], start: usize, byte: u8) -> bool {
    byte.is_ascii_digit()
        && !bytes
            .get(start.wrapping_sub(1))
            .is_some_and(u8::is_ascii_alphanumeric)
}

/// Consume a dotted numeric anchor candidate from its starting byte.
fn consume_anchor(bytes: &[u8], start: usize) -> usize {
    let mut end = consume_digits(bytes, start);
    while has_dot_digit(bytes, end) {
        end = consume_digits(bytes, end + 1);
    }
    end
}

/// Consume consecutive ASCII digits.
fn consume_digits(bytes: &[u8], start: usize) -> usize {
    let mut end = start;
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    end
}

/// Return whether a dot is followed by another digit segment.
fn has_dot_digit(bytes: &[u8], dot_index: usize) -> bool {
    matches!(
        (bytes.get(dot_index), bytes.get(dot_index + 1)),
        (Some(b'.'), Some(next)) if next.is_ascii_digit()
    )
}

#[cfg(test)]
mod tests {
    //! Unit coverage for dependency-reference classification branches.

    use rstest::rstest;

    use super::{
        DependencyReferenceClassification,
        classify_dependency_reference,
        rewrite_text_value,
    };
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
    #[case::outside_dependency_clause(
        "See 1.2",
        "1.2",
        ExpectedClassification::NotDependencyReference
    )]
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
}
