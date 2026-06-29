//! Lightweight counters for process-local CLI observability.
//!
//! # Example
//!
//! ```rust
//! use mapsplice::observability::{
//!     metrics_snapshot,
//!     record_dependency_rewrites,
//!     record_failure,
//!     record_in_place_rewrite,
//! };
//!
//! let before = metrics_snapshot();
//! record_failure("invalid_anchor");
//! record_in_place_rewrite();
//! record_dependency_rewrites(2);
//! let after = metrics_snapshot();
//!
//! assert_eq!(after.failures, before.failures + 1);
//! assert_eq!(after.in_place_rewrites, before.in_place_rewrites + 1);
//! assert_eq!(after.dependency_rewrites, before.dependency_rewrites + 2);
//! ```

use std::sync::atomic::{AtomicU64, Ordering};

/// Snapshot of process-local `mapsplice` counters.
///
/// See the module-level example for recording counters and reading this
/// snapshot with [`metrics_snapshot`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MetricsSnapshot {
    /// Failed command count grouped by all error classes.
    pub failures: u64,
    /// In-place target rewrites completed by the current process.
    pub in_place_rewrites: u64,
    /// Dependency text replacements completed by the current process.
    pub dependency_rewrites: u64,
}

static FAILURES: AtomicU64 = AtomicU64::new(0);
static IN_PLACE_REWRITES: AtomicU64 = AtomicU64::new(0);
static DEPENDENCY_REWRITES: AtomicU64 = AtomicU64::new(0);

/// Record one failed command.
///
/// See the module-level example for recording failures and observing the
/// resulting [`MetricsSnapshot`].
pub fn record_failure(error_class: &'static str) {
    let total = FAILURES.fetch_add(1, Ordering::Relaxed) + 1;
    tracing::debug!(error_class, total, "recorded mapsplice failure");
}

/// Record one successful in-place rewrite.
///
/// See the module-level example for recording rewrites and observing the
/// resulting [`MetricsSnapshot`].
pub fn record_in_place_rewrite() {
    let total = IN_PLACE_REWRITES.fetch_add(1, Ordering::Relaxed) + 1;
    tracing::debug!(total, "recorded in-place rewrite");
}

/// Record dependency text replacements.
///
/// See the module-level example for recording replacements and observing the
/// resulting [`MetricsSnapshot`].
pub fn record_dependency_rewrites(count: u64) {
    if count == 0 {
        return;
    }
    let total = DEPENDENCY_REWRITES.fetch_add(count, Ordering::Relaxed) + count;
    tracing::debug!(count, total, "recorded dependency rewrites");
}

/// Return a snapshot of process-local counters.
///
/// # Example
///
/// ```rust
/// use mapsplice::observability::{
///     metrics_snapshot,
///     record_dependency_rewrites,
///     record_failure,
///     record_in_place_rewrite,
/// };
///
/// let before = metrics_snapshot();
/// record_failure("invalid_anchor");
/// record_in_place_rewrite();
/// record_dependency_rewrites(2);
/// let after = metrics_snapshot();
///
/// assert_eq!(after.failures, before.failures + 1);
/// assert_eq!(after.in_place_rewrites, before.in_place_rewrites + 1);
/// assert_eq!(after.dependency_rewrites, before.dependency_rewrites + 2);
/// ```
#[must_use]
pub fn metrics_snapshot() -> MetricsSnapshot {
    MetricsSnapshot {
        failures: FAILURES.load(Ordering::Relaxed),
        in_place_rewrites: IN_PLACE_REWRITES.load(Ordering::Relaxed),
        dependency_rewrites: DEPENDENCY_REWRITES.load(Ordering::Relaxed),
    }
}
