//! Clap arguments for read-only machine-readable roadmap export.

use camino::Utf8PathBuf;
use clap::{Args, ValueEnum};

use super::{CliRequest, CommandKind, GlobalOptions};

/// Output serialization formats supported by the export command.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum ExportFormat {
    /// Versioned JSON roadmap document.
    #[default]
    Json,
}

impl ExportFormat {
    /// Return the stable lowercase spelling used in diagnostics and spans.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mapsplice::ExportFormat;
    ///
    /// assert_eq!(ExportFormat::Json.as_str(), "json");
    /// ```
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
        }
    }
}

/// Positional roadmap input and optional export validation path.
#[derive(Debug, Args)]
#[command(about = None)]
pub(super) struct ExportArgs {
    /// Canonical Markdown roadmap to export.
    #[arg(help = None)]
    pub(super) target: Utf8PathBuf,
    /// Output format.
    #[arg(long, value_enum, default_value_t = ExportFormat::Json, help = None)]
    pub(super) format: ExportFormat,
    /// Compare the fresh export with this committed JSON file.
    #[arg(long, value_name = "EXPORT", help = None)]
    pub(super) check: Option<Utf8PathBuf>,
}

impl ExportArgs {
    /// Construct an export request without merging export-specific settings.
    pub(super) fn into_request(self) -> CliRequest {
        CliRequest {
            global: GlobalOptions { in_place: false },
            target: self.target,
            command: CommandKind::Export {
                format: self.format,
                check: self.check,
            },
        }
    }
}
