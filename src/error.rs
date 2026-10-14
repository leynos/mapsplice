//! Error types for `mapsplice`.

use camino::Utf8PathBuf;
use markdown::unist::Point;
use thiserror::Error;

use crate::roadmap::{RoadmapAnchor, RoadmapItemLevel};

/// Convenient result alias for `mapsplice`.
pub type Result<T> = std::result::Result<T, MapspliceError>;

/// One-based source coordinates captured from a Markdown parser point.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceLocation {
    /// One-based source line.
    pub line: usize,
    /// One-based source column.
    pub column: usize,
}

impl From<&Point> for SourceLocation {
    fn from(point: &Point) -> Self {
        Self {
            line: point.line,
            column: point.column,
        }
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "line {}, column {}", self.line, self.column)
    }
}

/// Errors raised while parsing or rewriting roadmap documents.
#[derive(Debug, Error)]
pub enum MapspliceError {
    /// The provided anchor string is invalid.
    #[error("invalid roadmap anchor `{anchor}`")]
    InvalidAnchor {
        /// Unparsable anchor text.
        anchor: String,
    },

    /// The document structure is not a supported roadmap.
    #[error("{prefix}{message}", prefix = location.map_or_else(String::new, |source_location| format!("{source_location}: ")))]
    InvalidRoadmap {
        /// Human-readable validation error.
        message: String,
        /// Source position of the rejected Markdown node, when known.
        location: Option<SourceLocation>,
    },

    /// The fragment level does not match the target anchor level.
    #[error("cannot use {found} content with {expected} anchor `{anchor}`")]
    LevelMismatch {
        /// Anchor being targeted.
        anchor: RoadmapAnchor,
        /// Expected fragment level.
        expected: RoadmapItemLevel,
        /// Level found in the fragment.
        found: RoadmapItemLevel,
    },

    /// An append command received a fragment at the wrong structural level.
    #[error("cannot append {found} content; append expects {expected} content")]
    AppendLevelMismatch {
        /// Expected fragment level.
        expected: RoadmapItemLevel,
        /// Level found in the fragment.
        found: RoadmapItemLevel,
    },

    /// A command expected fragment input but none was provided.
    #[error("command `{command}` requires a fragment file")]
    MissingFragment {
        /// Command name.
        command: &'static str,
    },

    /// A command did not need a fragment file but one was provided.
    #[error("command `{command}` does not accept a fragment file")]
    UnexpectedFragment {
        /// Command name.
        command: &'static str,
    },

    /// The requested anchor is not present in the target document.
    #[error("anchor `{anchor}` was not found in the target roadmap")]
    AnchorNotFound {
        /// Missing anchor.
        anchor: RoadmapAnchor,
    },

    /// A dependency clause references an anchor that no longer exists.
    #[error("{prefix}dependency anchor `{anchor}` was not found in the target roadmap", prefix = location.map_or_else(String::new, |source_location| format!("{source_location}: ")))]
    DanglingDependency {
        /// Missing dependency anchor.
        anchor: RoadmapAnchor,
        /// Source position of the unresolved reference, when known.
        location: Option<SourceLocation>,
    },

    /// The committed JSON export differs from the Markdown source.
    #[error(
        "export `{export}` differs from roadmap `{roadmap}`:\n{differences}\nregenerate with \
         `{regenerate}`"
    )]
    ExportDrift {
        /// Committed JSON export path.
        export: Utf8PathBuf,
        /// Canonical Markdown roadmap path.
        roadmap: Utf8PathBuf,
        /// Human-readable semantic or formatting differences.
        differences: String,
        /// Command that regenerates the committed export.
        regenerate: String,
    },

    /// A committed JSON export could not be read or does not match the schema.
    #[error("invalid export `{path}`: {message}")]
    InvalidExport {
        /// Committed JSON export path.
        path: Utf8PathBuf,
        /// Read or schema diagnostic, including JSON line and column when available.
        message: String,
    },

    /// The in-memory roadmap could not be serialized to JSON.
    #[error("failed to serialize roadmap export: {message}")]
    JsonSerialization {
        /// Underlying serialization diagnostic.
        message: String,
    },

    /// Markdown parsing failed.
    #[error("failed to parse markdown: {message}")]
    Markdown {
        /// Parser diagnostic.
        message: String,
    },

    /// CLI parsing failed.
    #[error(transparent)]
    Clap(#[from] clap::Error),

    /// Configuration loading failed.
    #[error("{message}")]
    Configuration {
        /// Human-readable configuration error.
        message: String,
    },

    /// Filesystem operation failed.
    #[error("{action} `{path}`: {source}")]
    Io {
        /// Failed action.
        action: &'static str,
        /// Path involved in the failure.
        path: Utf8PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

impl MapspliceError {
    /// Return a stable diagnostic class for logs and metrics.
    #[must_use]
    pub const fn class(&self) -> &'static str {
        match self {
            Self::InvalidAnchor { .. } => "invalid_anchor",
            Self::InvalidRoadmap { .. } => "invalid_roadmap",
            Self::LevelMismatch { .. } => "level_mismatch",
            Self::AppendLevelMismatch { .. } => "append_level_mismatch",
            Self::MissingFragment { .. } => "missing_fragment",
            Self::UnexpectedFragment { .. } => "unexpected_fragment",
            Self::AnchorNotFound { .. } => "anchor_not_found",
            Self::DanglingDependency { .. } => "dangling_dependency",
            Self::ExportDrift { .. } => "export_drift",
            Self::InvalidExport { .. } => "invalid_export",
            Self::JsonSerialization { .. } => "json_serialization",
            Self::Markdown { .. } => "markdown",
            Self::Clap(_) => "cli",
            Self::Configuration { .. } => "configuration",
            Self::Io { .. } => "io",
        }
    }
}
