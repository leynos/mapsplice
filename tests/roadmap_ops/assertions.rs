//! Assertion helpers shared by the `roadmap_ops` test modules.

use std::fmt::Debug;

use mapsplice::MapspliceError::{self, AnchorNotFound, LevelMismatch};

/// Assert two values are equal, printing both on failure.
pub(crate) fn assert_equal<T>(actual: &T, expected: &T)
where
    T: Debug + PartialEq,
{
    assert_eq!(actual, expected, "roadmap operation result differs");
}

/// Assert an error is a level mismatch between the anchor and the fragment.
pub(crate) fn assert_level_mismatch(error: &MapspliceError) {
    let is_level_mismatch = matches!(error, LevelMismatch { .. });
    assert!(is_level_mismatch, "expected level mismatch");
}

/// Assert an error reports an anchor that is absent from the roadmap.
pub(crate) fn assert_anchor_not_found(error: &MapspliceError) {
    let is_anchor_not_found = matches!(error, AnchorNotFound { .. });
    assert!(is_anchor_not_found, "expected missing anchor");
}
