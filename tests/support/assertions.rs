//! Shared assertion helpers for integration tests.

/// Assert that `haystack` contains `needle`.
pub fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "expected haystack to contain needle\nneedle:\n{needle}\nhaystack:\n{haystack}"
    );
}
