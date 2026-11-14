//! Parsed CLI request types shared by the parsing and execution layers.

use camino::Utf8PathBuf;

use super::ExportFormat;
use crate::roadmap::RoadmapAnchor;

/// Resolved global options.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GlobalOptions {
    /// Rewrite the target file instead of printing to stdout.
    pub in_place: bool,
}

/// Parsed CLI request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliRequest {
    /// Resolved global options.
    pub global: GlobalOptions,
    /// Target roadmap file.
    pub target: Utf8PathBuf,
    /// Requested operation.
    pub command: CommandKind,
}

/// Supported splice commands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandKind {
    /// Append one or more phases to the end of the roadmap.
    Append {
        /// Fragment file.
        fragment: Utf8PathBuf,
    },
    /// Insert sibling items before or after the anchor.
    Insert {
        /// Anchor to insert around.
        anchor: RoadmapAnchor,
        /// Insert after the anchor when true.
        after: bool,
        /// Fragment file.
        fragment: Utf8PathBuf,
    },
    /// Delete the addressed item.
    Delete {
        /// Anchor to delete.
        anchor: RoadmapAnchor,
    },
    /// Replace the addressed item with fragment content.
    Replace {
        /// Anchor to replace.
        anchor: RoadmapAnchor,
        /// Fragment file.
        fragment: Utf8PathBuf,
    },
    /// Export the roadmap as a stable machine-readable document.
    Export {
        /// Output format.
        format: ExportFormat,
        /// Compare the fresh export to a committed JSON file.
        check: Option<Utf8PathBuf>,
    },
}

impl CommandKind {
    /// Return the fragment path used by the command, if any.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use camino::Utf8PathBuf;
    /// use mapsplice::{CommandKind, parse_anchor};
    ///
    /// # fn main() -> mapsplice::Result<()> {
    /// let fragment = Utf8PathBuf::from("fragment.md");
    /// let anchor = parse_anchor("1")?;
    /// let task_anchor = parse_anchor("1.1.1")?;
    ///
    /// assert_eq!(
    ///     CommandKind::Append {
    ///         fragment: fragment.clone()
    ///     }
    ///     .fragment_path(),
    ///     Some(&fragment)
    /// );
    /// assert_eq!(
    ///     CommandKind::Insert {
    ///         anchor,
    ///         after: true,
    ///         fragment: fragment.clone()
    ///     }
    ///     .fragment_path(),
    ///     Some(&fragment)
    /// );
    /// assert_eq!(
    ///     CommandKind::Replace {
    ///         anchor: task_anchor,
    ///         fragment: fragment.clone()
    ///     }
    ///     .fragment_path(),
    ///     Some(&fragment)
    /// );
    /// assert_eq!(CommandKind::Delete { anchor }.fragment_path(), None);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn fragment_path(&self) -> Option<&Utf8PathBuf> {
        match self {
            Self::Append { fragment }
            | Self::Insert { fragment, .. }
            | Self::Replace { fragment, .. } => Some(fragment),
            Self::Delete { .. } | Self::Export { .. } => None,
        }
    }
}
