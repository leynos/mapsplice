//! Pure decision kernel behind dependency-reference resolution.
//!
//! Issue #85 made the identity-preservation rule load-bearing: a target-text
//! reference must never resolve through the cross-source fallback, or a
//! consumer silently re-points at an unrelated item that inherited its
//! prerequisite's number. That rule is a two-line decision buried in
//! [`RenumberPlan::resolve_reference`], where a property test can only sample
//! it and a proof cannot reach it at all.
//!
//! The decision is therefore extracted here as a pure function over its own
//! inputs, with no map, no document, and no I/O. `verus/lib.rs` proves the
//! obligations over this exact text: the body of
//! [`select_resolution`] is spliced from `verus/kernels/select_resolution.body.rs`,
//! so the verified text and the compiled text are one artefact rather than two
//! implementations that can drift.
//!
//! [`RenumberPlan::resolve_reference`]: super::super::model::RenumberPlan::resolve_reference

/// Choose which mapping a dependency reference resolves through.
///
/// The source-local mapping wins whenever it exists. The cross-source value is
/// accepted only for fragment text, which is the whole of the identity rule:
/// a target item's identity is the anchor it held before the operation, while
/// a fragment item's identity is its fragment-local spelling, and allowing
/// target text to fall back would let a replacement item spelled like its
/// predecessor absorb a surviving consumer's clause.
///
/// Both inputs are already-resolved anchors, so this function performs no
/// lookup and cannot fail; it decides only which of two answers is used.
#[must_use]
pub const fn select_resolution<T: Copy>(
    local: Option<T>,
    cross_unique: Option<T>,
    source_is_fragment: bool,
) -> Option<T> {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/verus/kernels/select_resolution.body.rs"
    ))
}

#[cfg(test)]
mod tests {
    //! Unit coverage for the resolution decision's four input cases.

    use rstest::rstest;

    use super::select_resolution;

    /// Return the letters `1`, `2`, or `3` for the cases below.
    const ONE: Option<u8> = Some(1);
    const TWO: Option<u8> = Some(2);

    #[rstest]
    #[case::local_wins_for_target(ONE, TWO, false, ONE)]
    #[case::local_wins_for_fragment(ONE, TWO, true, ONE)]
    #[case::target_never_falls_back(None, TWO, false, None)]
    #[case::fragment_uses_cross_unique(None, TWO, true, TWO)]
    #[case::both_absent(None, None, false, None)]
    #[case::both_absent_for_fragment(None, None, true, None)]
    fn select_resolution_chooses_the_documented_mapping(
        #[case] local: Option<u8>,
        #[case] cross_unique: Option<u8>,
        #[case] source_is_fragment: bool,
        #[case] expected: Option<u8>,
    ) {
        assert_eq!(
            select_resolution(local, cross_unique, source_is_fragment),
            expected
        );
    }
}
