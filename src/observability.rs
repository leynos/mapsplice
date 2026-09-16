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
/// snapshot with [`metrics_snapshot`]. Source-preservation counters extend
/// this public snapshot so callers can observe new outcomes without a metrics
/// backend; consumers should obtain snapshots rather than construct literals.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MetricsSnapshot {
    /// Failed command count grouped by all error classes.
    pub failures: u64,
    /// In-place target rewrites completed by the current process.
    pub in_place_rewrites: u64,
    /// Dependency text replacements completed by the current process.
    pub dependency_rewrites: u64,
    /// Stable task or sub-task sources emitted without canonical rendering.
    pub preserved_source_renders: u64,
    /// Preserved task or sub-task sources invalidated by a mutation.
    pub preserved_source_invalidations: u64,
    /// Preserved task or sub-task sources rendered canonically for stability.
    pub canonical_fallbacks: u64,
    /// Preserved sources invalidated because their rendered number changed.
    pub invalidations_renumber: u64,
    /// Preserved sources invalidated because dependency text changed.
    pub invalidations_dependency_rewrite: u64,
    /// Preserved parent-task sources invalidated by a sub-task mutation.
    pub invalidations_child_mutation: u64,
    /// Canonical fallbacks caused by unstable list markers.
    pub canonical_fallbacks_unstable_list_marker: u64,
    /// Canonical fallbacks caused by unstable code fences.
    pub canonical_fallbacks_unstable_code_fence: u64,
}

/// Closed mutation causes for preserved-source invalidation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PreservationInvalidationReason {
    /// The item's rendered number changed.
    Renumber,
    /// The item's dependency text changed.
    DependencyRewrite,
    /// A structural sub-task mutation changed a parent task.
    ChildMutation,
}

/// Closed formatter-stability causes for canonical item rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CanonicalFallbackReason {
    /// An ordered or nested list marker is formatter-unstable.
    UnstableListMarker,
    /// A code-fence delimiter is formatter-unstable.
    UnstableCodeFence,
}

static FAILURES: AtomicU64 = AtomicU64::new(0);
static IN_PLACE_REWRITES: AtomicU64 = AtomicU64::new(0);
static DEPENDENCY_REWRITES: AtomicU64 = AtomicU64::new(0);
static PRESERVED_SOURCE_RENDERS: AtomicU64 = AtomicU64::new(0);
static PRESERVED_SOURCE_INVALIDATIONS: AtomicU64 = AtomicU64::new(0);
static CANONICAL_FALLBACKS: AtomicU64 = AtomicU64::new(0);
static INVALIDATIONS_RENUMBER: AtomicU64 = AtomicU64::new(0);
static INVALIDATIONS_DEPENDENCY_REWRITE: AtomicU64 = AtomicU64::new(0);
static INVALIDATIONS_CHILD_MUTATION: AtomicU64 = AtomicU64::new(0);
static CANONICAL_FALLBACKS_UNSTABLE_LIST_MARKER: AtomicU64 = AtomicU64::new(0);
static CANONICAL_FALLBACKS_UNSTABLE_CODE_FENCE: AtomicU64 = AtomicU64::new(0);

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

/// Record one formatter-stable preserved task or sub-task render.
pub(crate) fn record_preserved_source_render() {
    let total = PRESERVED_SOURCE_RENDERS.fetch_add(1, Ordering::Relaxed) + 1;
    tracing::debug!(total, "recorded preserved source render");
}

/// Record one actual preserved-source invalidation with a closed cause.
pub(crate) fn record_preserved_source_invalidation(reason: PreservationInvalidationReason) {
    let total = PRESERVED_SOURCE_INVALIDATIONS.fetch_add(1, Ordering::Relaxed) + 1;
    let reason_total = invalidation_counter(reason).fetch_add(1, Ordering::Relaxed) + 1;
    tracing::debug!(
        reason = invalidation_reason_name(reason),
        total,
        reason_total,
        "recorded preserved source invalidation"
    );
}

/// Record one canonical rendering fallback with a closed formatter cause.
pub(crate) fn record_canonical_fallback(reason: CanonicalFallbackReason) {
    let total = CANONICAL_FALLBACKS.fetch_add(1, Ordering::Relaxed) + 1;
    let reason_total = canonical_fallback_counter(reason).fetch_add(1, Ordering::Relaxed) + 1;
    tracing::debug!(
        reason = canonical_fallback_reason_name(reason),
        total,
        reason_total,
        "recorded canonical source fallback"
    );
}

/// Return the counter for a preserved-source invalidation cause.
fn invalidation_counter(reason: PreservationInvalidationReason) -> &'static AtomicU64 {
    match reason {
        PreservationInvalidationReason::Renumber => &INVALIDATIONS_RENUMBER,
        PreservationInvalidationReason::DependencyRewrite => &INVALIDATIONS_DEPENDENCY_REWRITE,
        PreservationInvalidationReason::ChildMutation => &INVALIDATIONS_CHILD_MUTATION,
    }
}

/// Return the stable tracing label for a preserved-source invalidation cause.
const fn invalidation_reason_name(reason: PreservationInvalidationReason) -> &'static str {
    match reason {
        PreservationInvalidationReason::Renumber => "renumber",
        PreservationInvalidationReason::DependencyRewrite => "dependency_rewrite",
        PreservationInvalidationReason::ChildMutation => "child_mutation",
    }
}

/// Return the counter for a formatter-stability canonical fallback cause.
fn canonical_fallback_counter(reason: CanonicalFallbackReason) -> &'static AtomicU64 {
    match reason {
        CanonicalFallbackReason::UnstableListMarker => &CANONICAL_FALLBACKS_UNSTABLE_LIST_MARKER,
        CanonicalFallbackReason::UnstableCodeFence => &CANONICAL_FALLBACKS_UNSTABLE_CODE_FENCE,
    }
}

/// Return the stable tracing label for a formatter-stability fallback cause.
const fn canonical_fallback_reason_name(reason: CanonicalFallbackReason) -> &'static str {
    match reason {
        CanonicalFallbackReason::UnstableListMarker => "unstable_list_marker",
        CanonicalFallbackReason::UnstableCodeFence => "unstable_code_fence",
    }
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
        preserved_source_renders: PRESERVED_SOURCE_RENDERS.load(Ordering::Relaxed),
        preserved_source_invalidations: PRESERVED_SOURCE_INVALIDATIONS.load(Ordering::Relaxed),
        canonical_fallbacks: CANONICAL_FALLBACKS.load(Ordering::Relaxed),
        invalidations_renumber: INVALIDATIONS_RENUMBER.load(Ordering::Relaxed),
        invalidations_dependency_rewrite: INVALIDATIONS_DEPENDENCY_REWRITE.load(Ordering::Relaxed),
        invalidations_child_mutation: INVALIDATIONS_CHILD_MUTATION.load(Ordering::Relaxed),
        canonical_fallbacks_unstable_list_marker: CANONICAL_FALLBACKS_UNSTABLE_LIST_MARKER
            .load(Ordering::Relaxed),
        canonical_fallbacks_unstable_code_fence: CANONICAL_FALLBACKS_UNSTABLE_CODE_FENCE
            .load(Ordering::Relaxed),
    }
}

#[cfg(test)]
mod tests {
    //! Concurrency coverage for process-local observability counters.

    use std::thread;

    use super::{
        metrics_snapshot,
        record_dependency_rewrites,
        record_failure,
        record_in_place_rewrite,
    };

    #[test]
    fn counters_record_concurrent_increments() {
        let before = metrics_snapshot();
        let handles = (0..8)
            .map(|_| {
                thread::spawn(|| {
                    record_failure("test");
                    record_in_place_rewrite();
                    record_dependency_rewrites(2);
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().expect("counter worker should finish");
        }

        let after = metrics_snapshot();
        assert_eq!(after.failures, before.failures + 8);
        assert_eq!(after.in_place_rewrites, before.in_place_rewrites + 8);
        assert_eq!(after.dependency_rewrites, before.dependency_rewrites + 16);
    }
}
