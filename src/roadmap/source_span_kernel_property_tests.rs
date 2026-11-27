//! Generated checks for the source-span kernel oracles.

use proptest::prelude::*;

use super::{
    super::{checked_line_span, checked_line_start, extract_line_span},
    extract_oracle,
    line_start_oracle,
    source_text,
};

proptest! {
#![proptest_config(ProptestConfig {
    cases: 64,
    .. ProptestConfig::default()
})]

/// The scan agrees with `rfind` at every offset.
#[test]
fn checked_line_start_matches_the_rfind_oracle(source in source_text()) {
    for offset in 0..=source.len() {
        let actual = checked_line_start(&source, offset);
        let expected = line_start_oracle(&source, offset);
        prop_assert_eq!(
            actual,
            expected,
            "offset {} of {:?} must match the rfind oracle",
            offset,
            source,
        );
    }
}

/// Extraction agrees with the widened `str::get` oracle for every
/// ordered pair of character-boundary offsets.
#[test]
fn extraction_matches_the_widened_get_oracle(source in source_text()) {
    for start in 0..=source.len() {
        if !source.is_char_boundary(start) {
            continue;
        }
        for end in start..=source.len() {
            if !source.is_char_boundary(end) {
                continue;
            }
            let actual = extract_line_span(&source, start, end);
            let expected = extract_oracle(&source, start, end)
                .ok_or_else(|| TestCaseError::fail("oracle rejected a boundary span"))?;
            prop_assert_eq!(
                actual,
                Ok(expected),
                "span {}..{} of {:?} must match the widened get oracle",
                start,
                end,
                source,
            );
        }
    }
}

/// Non-boundary endpoints are rejected before any copy.
#[test]
fn non_boundary_endpoints_are_rejected(source in source_text()) {
    for offset in 0..=source.len() {
        if source.is_char_boundary(offset) {
            continue;
        }
        let actual = checked_line_span(&source, offset, offset);
        prop_assert!(
            actual.is_err(),
            "offset {} of {:?} is not a character boundary",
            offset,
            source,
        );
    }
}

/// The byte-level check agrees with `str::is_char_boundary`.
#[test]
fn checked_line_start_accepts_exactly_the_char_boundaries(source in source_text()) {
    for offset in 0..=source.len() {
        let actual = checked_line_start(&source, offset).is_some();
        prop_assert_eq!(
            actual,
            source.is_char_boundary(offset),
            "offset {} of {:?} must agree with str::is_char_boundary",
            offset,
            source,
        );
    }
}
}
