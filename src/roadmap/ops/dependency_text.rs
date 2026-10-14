//! Dependency-clause text scanning for roadmap anchor rewrites.

use std::ops::Range;

use super::super::{model::RenumberPlan, parse_anchor};
use crate::roadmap::{RoadmapAnchor, model::SourceId};

/// Classification for an anchor-shaped candidate in a text value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DependencyReferenceClassification {
    /// Valid anchor appearing in a dependency clause.
    Reference(RoadmapAnchor),
    /// Dependency token that cannot be parsed as a valid anchor.
    InvalidDependencyToken,
    /// Candidate outside a supported dependency context.
    NotDependencyReference,
}

/// Dependency-reference rewrite details for one text value.
pub(super) struct DependencyRewriteReport {
    /// Text after eligible anchor rewrites.
    pub(super) value: String,
    /// Number of references whose anchor changed.
    pub(super) rewrite_count: u64,
    /// Valid dependency anchors with no resolvable mapping.
    pub(super) unresolved: Vec<RoadmapAnchor>,
}

/// One anchor-like token in a dependency clause.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DependencyCandidate {
    /// Byte range occupied by the token in its source text node.
    pub(super) range: Range<usize>,
    /// Parsed anchor, or `None` when the token is malformed.
    pub(crate) anchor: Option<RoadmapAnchor>,
}

/// Find valid anchors and malformed anchor-like tokens in dependency clauses.
pub(super) fn dependency_candidates(value: &str) -> Vec<DependencyCandidate> {
    let mut candidates = Vec::new();
    let mut index = 0;
    while let Some((start, end)) = next_anchor_candidate(value, index) {
        if let Some(anchor) = match classify_dependency_reference(value, start, end) {
            DependencyReferenceClassification::Reference(anchor) => Some(Some(anchor)),
            DependencyReferenceClassification::InvalidDependencyToken => Some(None),
            DependencyReferenceClassification::NotDependencyReference => None,
        } {
            candidates.push(DependencyCandidate {
                range: start..end,
                anchor,
            });
        }
        index = end;
    }
    candidates
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

    for candidate in dependency_candidates(value) {
        let start = candidate.range.start;
        let end = candidate.range.end;
        result.push_str(value.get(index..start).unwrap_or_default());
        let token = value.get(start..end).unwrap_or_default();
        if let Some(anchor) = candidate.anchor {
            if let Some(mapped) = plan.resolve_reference(source, anchor) {
                rewrite_count += u64::from(mapped != anchor);
                result.push_str(&mapped.to_string());
            } else {
                unresolved.push(anchor);
                result.push_str(token);
            }
        } else {
            result.push_str(token);
        }
        index = end;
    }
    result.push_str(value.get(index..).unwrap_or_default());

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
            !has_dependency_clause_terminator(between)
                && (between.contains(':') || between.chars().any(char::is_whitespace))
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
#[path = "dependency_text_tests.rs"]
mod tests;
