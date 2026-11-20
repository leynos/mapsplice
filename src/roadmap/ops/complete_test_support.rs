//! Shared fixtures for the in-place completion unit tests.

use super::complete_item;
use crate::{
    error::Result,
    roadmap::{parse_anchor, parse_roadmap},
};

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
