//! Property tests for deterministic export layout and drift reporting.
//!
//! The layout property renders the same semantic roadmap with different
//! wrapping, line endings, whitespace, and indentation, and requires each
//! source layout to produce byte-identical JSON. The drift property mutates
//! one field at every nesting level of a committed export and requires the
//! drift report to name that level.

mod drift;
mod layout;
mod model;

#[path = "../support/export_workspace.rs"]
mod export_workspace;
