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
    /// Task items emitted from their preserved source snippets.
    pub preserved_task_items: u64,
    /// Task items emitted with canonical rendering.
    pub canonical_task_items: u64,
    /// Task items without a reusable preserved source snippet.
    pub task_source_fallbacks: u64,
}

static FAILURES: AtomicU64 = AtomicU64::new(0);
static IN_PLACE_REWRITES: AtomicU64 = AtomicU64::new(0);
static DEPENDENCY_REWRITES: AtomicU64 = AtomicU64::new(0);
static PRESERVED_TASK_ITEMS: AtomicU64 = AtomicU64::new(0);
static CANONICAL_TASK_ITEMS: AtomicU64 = AtomicU64::new(0);
static TASK_SOURCE_FALLBACKS: AtomicU64 = AtomicU64::new(0);

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

/// Record aggregate task-list rendering states.
///
/// The counters contain only item counts. The debug event deliberately omits
/// task text and roadmap identifiers so source-fidelity diagnostics remain
/// bounded.
pub fn record_task_render_states(preserved: u64, canonical: u64, fallbacks: u64) {
    let preserved_total = PRESERVED_TASK_ITEMS.fetch_add(preserved, Ordering::Relaxed) + preserved;
    let canonical_total = CANONICAL_TASK_ITEMS.fetch_add(canonical, Ordering::Relaxed) + canonical;
    let fallback_total = TASK_SOURCE_FALLBACKS.fetch_add(fallbacks, Ordering::Relaxed) + fallbacks;
    tracing::debug!(
        preserved,
        canonical,
        fallbacks,
        preserved_total,
        canonical_total,
        fallback_total,
        "recorded task-list render states"
    );
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
        preserved_task_items: PRESERVED_TASK_ITEMS.load(Ordering::Relaxed),
        canonical_task_items: CANONICAL_TASK_ITEMS.load(Ordering::Relaxed),
        task_source_fallbacks: TASK_SOURCE_FALLBACKS.load(Ordering::Relaxed),
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
        record_task_render_states,
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
                    record_task_render_states(3, 2, 2);
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
        assert_eq!(after.preserved_task_items, before.preserved_task_items + 24);
        assert_eq!(after.canonical_task_items, before.canonical_task_items + 16);
        assert_eq!(
            after.task_source_fallbacks,
            before.task_source_fallbacks + 16
        );
    }
}
