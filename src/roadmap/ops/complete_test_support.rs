//! Shared fixtures for the in-place completion unit tests.

use super::complete_item;
use crate::roadmap::{parse_anchor, parse_roadmap};

/// Complete `anchor` in `source`, failing the test on error.
pub(super) fn complete_ok(source: &str, anchor: &str, suffix: Option<&str>) -> String {
    let roadmap = parse_roadmap(source).expect("fixture roadmap should parse");
    let parsed_anchor = parse_anchor(anchor).expect("fixture anchor should parse");
    complete_item(&roadmap, source, parsed_anchor, suffix).expect("completion should succeed")
}

/// Build a one-step roadmap around one task line.
pub(super) fn completion_source(item: &str) -> String {
    format!("## 1. Phase one\n\n### 1.1. Step one\n\n{item}\n")
}
