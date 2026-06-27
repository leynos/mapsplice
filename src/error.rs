//! Error types for `mapsplice`.

use camino::Utf8PathBuf;
use thiserror::Error;

use crate::roadmap::{RoadmapAnchor, RoadmapItemLevel};

/// Convenient result alias for `mapsplice`.
pub type Result<T> = std::result::Result<T, MapspliceError>;

/// Errors raised while parsing or rewriting roadmap documents.
#[derive(Debug, Error)]
pub enum MapspliceError {
    /// The provided anchor string is invalid.
    #[error("invalid roadmap anchor `{anchor}`")]
    InvalidAnchor {
        /// Unparseable anchor text.
        anchor: String,
    },

    /// The document structure is not a supported roadmap.
    #[error("{message}")]
    InvalidRoadmap {
        /// Human-readable validation error.
        message: String,
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
