//! Compile-fail coverage for the extensible metrics snapshot contract.

use mapsplice::MetricsSnapshot;

fn main() {
    let _snapshot = MetricsSnapshot {
        failures: 0,
        in_place_rewrites: 0,
        dependency_rewrites: 0,
        preserved_source_renders: 0,
        preserved_source_invalidations: 0,
        canonical_fallbacks: 0,
        invalidations_renumber: 0,
        invalidations_dependency_rewrite: 0,
        invalidations_child_mutation: 0,
        canonical_fallbacks_unstable_list_marker: 0,
        canonical_fallbacks_unstable_code_fence: 0,
    };
}
