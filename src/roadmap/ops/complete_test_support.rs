//! Shared fixtures for the in-place completion unit tests.

use super::complete_item;
use crate::{
    error::Result,
    roadmap::{parse_anchor, parse_roadmap},
};

/// One table row asserting the completed text of a single item line.
pub(super) struct CompletionCase {
    /// Original one-line item.
    pub(super) item: &'static str,
    /// Configured done suffix.
    pub(super) suffix: Option<&'static str>,
    /// Expected completed item line.
    pub(super) expected: &'static str,
}

impl CompletionCase {
    /// Build a table row from its three literal parts.
    pub(super) const fn new(
        item: &'static str,
        suffix: Option<&'static str>,
        expected: &'static str,
    ) -> Self {
        Self {
            item,
            suffix,
            expected,
        }
    }
}

/// Assert that completing one table row yields its expected item line.
pub(super) fn assert_completion_case(case: &CompletionCase) {
    assert_eq!(
        complete_ok(&completion_source(case.item), "1.1.1", case.suffix)
            .expect("fixture completion should succeed"),
        completion_source(case.expected),
        "completing {:?} with suffix {:?} must yield {:?}",
        case.item,
        case.suffix,
        case.expected
    );
}

/// Complete `anchor` in `source`, propagating any parsing or edit failure.
pub(super) fn complete_ok(source: &str, anchor: &str, suffix: Option<&str>) -> Result<String> {
    let roadmap = parse_roadmap(source)?;
    let parsed_anchor = parse_anchor(anchor)?;
    complete_item(&roadmap, source, parsed_anchor, suffix)
}

/// Build a one-step roadmap around one task line.
pub(super) fn completion_source(item: &str) -> String {
    format!("## 1. Phase one\n\n### 1.1. Step one\n\n{item}\n")
}
