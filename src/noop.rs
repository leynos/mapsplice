//! Detection of replacement edits that change only line terminators.
//!
//! An identical-item replacement should produce no diff. Re-rendering
//! normalizes CRLF and a missing final terminator by design, so the no-op path
//! emits the target's own bytes instead. Content the renderer would normalize
//! makes the comparison fail, so the no-op path never skips a required
//! formatter normalization.

use crate::roadmap::RoadmapOperation;

/// Return whether a replacement changes only line terminators.
///
/// A missing final terminator counts as a terminator difference: the render
/// adds one, but emitting the target's own bytes is still byte-identical.
pub(crate) fn replacement_is_noop(
    operation: &RoadmapOperation,
    target_text: &str,
    rendered: &str,
) -> bool {
    if !matches!(operation, RoadmapOperation::Replace { .. }) {
        return false;
    }
    let normalized_source = line_terminators_normalized(target_text);
    let source_with_terminator = if normalized_source.ends_with('\n') {
        normalized_source
    } else {
        format!("{normalized_source}\n")
    };
    source_with_terminator == line_terminators_normalized(rendered)
}

/// Normalize CRLF terminators so source can be compared with a rendered document.
fn line_terminators_normalized(text: &str) -> String { text.replace("\r\n", "\n") }
