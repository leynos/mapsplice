//! Dependency-clause text scanning for roadmap anchor rewrites.

use super::super::{model::RenumberPlan, parse_anchor};
use crate::{
    error::{MapspliceError, Result},
    roadmap::model::SourceId,
};

/// Rewrite anchor candidates within a text value and return the rewrite count.
pub(super) fn rewrite_text_value(
    value: &str,
    source: SourceId,
    plan: &RenumberPlan,
) -> Result<(String, u64)> {
    let mut result = String::with_capacity(value.len());
    let mut rewrite_count = 0;
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
        if !is_dependency_anchor(value, start) {
            result.push_str(candidate);
            index = end;
            continue;
        }
        let Ok(anchor) = parse_anchor(candidate) else {
            result.push_str(candidate);
            index = end;
            continue;
        };
        let mapped = plan
            .resolve(source, anchor)
            .or_else(|| plan.resolve_unique(anchor))
            .ok_or(MapspliceError::DanglingDependency { anchor })?;
        rewrite_count += 1;
        result.push_str(&mapped.to_string());
        index = end;
    }

    Ok((result, rewrite_count))
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
