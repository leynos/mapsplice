//! Adapter from Markdown parser positions to roadmap source coordinates.
//!
//! Roadmap diagnostics record [`SourceLocation`] values so that the error types
//! never depend on the Markdown AST. This module is the single place where a
//! parser [`Position`] crosses into the roadmap domain, which keeps the
//! conversion at the parser boundary instead of inside the shared error type.

use markdown::unist::Position;

use crate::error::SourceLocation;

/// Convert a Markdown parser position to one-based roadmap coordinates.
pub(crate) const fn source_location(position: &Position) -> SourceLocation {
    SourceLocation {
        line: position.start.line,
        column: position.start.column,
    }
}
