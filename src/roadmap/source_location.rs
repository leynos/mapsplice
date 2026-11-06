//! Adapter from Markdown parser positions to roadmap source coordinates.
//!
//! Roadmap diagnostics record [`SourceLocation`] values so that the error types
//! never depend on the Markdown AST. This module is the single place where a
//! parser [`Position`] crosses into the roadmap domain, which keeps the
//! conversion at the parser boundary instead of inside the shared error type.

use std::ops::Range;

use markdown::{
    mdast::Text,
    unist::{Point, Position},
};

use crate::error::SourceLocation;

/// Convert a Markdown parser position to one-based roadmap coordinates.
pub(crate) const fn source_location(position: &Position) -> SourceLocation {
    SourceLocation {
        line: position.start.line,
        column: position.start.column,
    }
}

/// Map a byte range inside a text node to the source coordinates of its start.
///
/// Markdown normalizes soft line endings and strips the indentation of
/// continuation lines, so a text node's value is not a slice of the document.
/// When the document is known, the token is located inside the node's own
/// source span, which keeps a diagnostic on the token even on a wrapped line.
/// Without the document the value is folded from the node's start, which is
/// exact for a single-line node and for the first line of a wrapped one.
///
/// # Examples
///
/// ```text
/// // Source, parsed into one text node whose value is
/// // "Requires 1.1.2\nand 1.0.1.":
/// //
/// //   line 7: "  Requires 1.1.2"
/// //   line 8: "  and 1.0.1."
/// //
/// // Offset 19..24 is "1.0.1"; the source location is line 8, column 7.
/// ```
pub(crate) fn source_location_at(
    text: &Text,
    source: Option<&str>,
    range: Range<usize>,
) -> Option<SourceLocation> {
    let position = text.position.as_ref()?;
    if let Some(source) = source
        && let Some(location) = locate_in_source(text, source, &range)
    {
        return Some(location);
    }
    fold_from_start(&position.start, &text.value, range.start)
}

/// Locate a token inside the node's own span of the original document.
fn locate_in_source(text: &Text, source: &str, range: &Range<usize>) -> Option<SourceLocation> {
    let position = text.position.as_ref()?;
    let token = text.value.get(range.clone())?;
    if token.is_empty() {
        return None;
    }
    let span = position.start.offset..position.end.offset;
    let node_source = source.get(span.clone())?;
    // Repeated anchors are common, so match the same occurrence the value has.
    let occurrence = text.value.get(..range.start)?.matches(token).count();
    let token_offset = node_source.match_indices(token).nth(occurrence)?.0;
    let prefix = source.get(span.start..span.start + token_offset)?;
    fold_from_start(&position.start, prefix, prefix.len())
}

/// Advance a start point over a prefix, counting line breaks and characters.
fn fold_from_start(start: &Point, value: &str, offset: usize) -> Option<SourceLocation> {
    let prefix = value.get(..offset)?;
    let (line, column) =
        prefix
            .chars()
            .fold((start.line, start.column), |(line, column), character| {
                if character == '\n' {
                    (line + 1, 1)
                } else {
                    (line, column + 1)
                }
            });
    Some(SourceLocation { line, column })
}
