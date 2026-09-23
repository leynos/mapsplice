//! Assertion helpers shared by the `roadmap_ops` test modules.

use std::fmt::Debug;

use mapsplice::MapspliceError;

/// Assert two values are equal, printing both on failure.
pub(crate) fn assert_equal<T>(actual: &T, expected: &T)
where
    T: Debug + PartialEq,
{
    assert_eq!(actual, expected);
}

/// Assert an error is a level mismatch between the anchor and the fragment.
pub(crate) fn assert_level_mismatch(error: &MapspliceError) {
    assert!(matches!(error, MapspliceError::LevelMismatch { .. }));
}

/// Assert an error reports an anchor that is absent from the roadmap.
pub(crate) fn assert_anchor_not_found(error: &MapspliceError) {
    assert!(matches!(error, MapspliceError::AnchorNotFound { .. }));
}
