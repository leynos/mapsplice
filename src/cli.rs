//! CLI parsing and `ortho-config` integration for `mapsplice`.

#[path = "cli_config.rs"]
mod config;

use std::ffi::OsString;

use camino::Utf8PathBuf;
use clap::{
    ArgAction,
    ArgMatches,
    Args,
    CommandFactory,
    FromArgMatches,
    Parser,
    Subcommand,
    parser::ValueSource,
};
use ortho_config::{OrthoConfig, load_and_merge_subcommand_for};
use serde::{Deserialize, Serialize};

use self::config::load_global_config;
use crate::{
    error::{MapspliceError, Result},
    roadmap::RoadmapAnchor,
};

/// Global options that can be loaded through `ortho-config`.
#[derive(Clone, Debug, Default, Parser, Serialize, Deserialize, OrthoConfig)]
#[command(next_help_heading = "Global options")]
#[ortho_config(prefix = "MAPSPLICE_")]
pub struct GlobalCli {
    /// Rewrite the target file instead of printing to stdout.
    #[arg(short = 'i', long = "in-place", global = true, action = ArgAction::SetTrue)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_place: Option<bool>,
}

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
            Self::Delete { .. } => None,
        }
    }
}

/// Parse a CLI request from command-line arguments.
///
/// # Examples
///
/// ```rust
/// use mapsplice::{CommandKind, parse_cli_request};
///
/// # fn main() -> mapsplice::Result<()> {
/// let request = parse_cli_request(["mapsplice", "--in-place", "delete", "roadmap.md", "2"])?;
///
/// assert!(request.global.in_place);
/// assert_eq!(request.target.as_str(), "roadmap.md");
/// assert!(matches!(request.command, CommandKind::Delete { .. }));
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error when command-line parsing fails or when configured
/// defaults cannot be merged.
pub fn parse_cli_request<I, T>(args: I) -> Result<CliRequest>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let command = Cli::command();
    let matches = command.try_get_matches_from(args)?;
    let cli = Cli::from_arg_matches(&matches)?;
    let global = load_global_config(&cli.global)?.resolve();
    cli.command.into_request(global, &matches)
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(flatten)]
    global: GlobalCli,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Append(AppendArgs),
    Insert(InsertArgs),
    Delete(DeleteArgs),
    Replace(ReplaceArgs),
}

impl Commands {
    /// Convert the selected subcommand into an executable CLI request.
    fn into_request(self, global: GlobalOptions, matches: &ArgMatches) -> Result<CliRequest> {
        match self {
            Self::Append(args) => Ok(args.into_request(global)),
            Self::Insert(args) => args.into_request(global, subcommand_matches(matches)?),
            Self::Delete(args) => Ok(args.into_request(global)),
            Self::Replace(args) => Ok(args.into_request(global)),
        }
    }
}

#[derive(Debug, Args)]
struct AppendArgs {
    target: Utf8PathBuf,
    fragment: Utf8PathBuf,
}

#[derive(Debug, Args)]
struct InsertArgs {
    #[command(flatten)]
    config: InsertConfig,
    target: Utf8PathBuf,
    anchor: RoadmapAnchor,
    fragment: Utf8PathBuf,
}

#[derive(Clone, Debug, Default, Parser, Serialize, Deserialize, OrthoConfig)]
#[command(name = "insert")]
#[ortho_config(prefix = "MAPSPLICE_")]
struct InsertConfig {
    #[arg(long, action = ArgAction::SetTrue)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    after: Option<bool>,
}

#[derive(Debug, Args)]
struct DeleteArgs {
    target: Utf8PathBuf,
    anchor: RoadmapAnchor,
}

#[derive(Debug, Args)]
struct ReplaceArgs {
    target: Utf8PathBuf,
    anchor: RoadmapAnchor,
    fragment: Utf8PathBuf,
}

impl GlobalCli {
    /// Resolve merged global CLI values into plain runtime options.
    fn resolve(self) -> GlobalOptions {
        GlobalOptions {
            in_place: self.in_place.unwrap_or(false),
        }
    }
}

impl AppendArgs {
    /// Build an append request from parsed positional arguments.
    fn into_request(self, global: GlobalOptions) -> CliRequest {
        CliRequest {
            global,
            target: self.target,
            command: CommandKind::Append {
                fragment: self.fragment,
            },
        }
    }
}

impl InsertArgs {
    /// Build an insert request after merging optional config defaults.
    fn into_request(self, global: GlobalOptions, matches: &ArgMatches) -> Result<CliRequest> {
        let config = self.config.with_absent_flags_removed(matches);
        let merged = load_merged_config(&config)?;
        Ok(CliRequest {
            global,
            target: self.target,
            command: CommandKind::Insert {
                anchor: self.anchor,
                after: merged.after.unwrap_or(false),
                fragment: self.fragment,
            },
        })
    }
}

impl InsertConfig {
    /// Remove clap's implicit `false` value when `--after` was not provided.
    fn with_absent_flags_removed(mut self, matches: &ArgMatches) -> Self {
        if matches.value_source("after") != Some(ValueSource::CommandLine) {
            self.after = None;
        }
        self
    }
}

impl DeleteArgs {
    /// Build a delete request from parsed positional arguments.
    fn into_request(self, global: GlobalOptions) -> CliRequest {
        CliRequest {
            global,
            target: self.target,
            command: CommandKind::Delete {
                anchor: self.anchor,
            },
        }
    }
}

impl ReplaceArgs {
    /// Build a replace request from parsed positional arguments.
    fn into_request(self, global: GlobalOptions) -> CliRequest {
        CliRequest {
            global,
            target: self.target,
            command: CommandKind::Replace {
                anchor: self.anchor,
                fragment: self.fragment,
            },
        }
    }
}

/// Merge CLI, environment, and configuration-file values for one config type.
fn load_merged_config<C>(config: &C) -> Result<C>
where
    C: CommandFactory + Default + OrthoConfig + Serialize,
{
    load_and_merge_subcommand_for(config).map_err(|error| configuration_error(&error))
}

/// Return the selected subcommand's matches for match-aware config merging.
fn subcommand_matches(matches: &ArgMatches) -> Result<&ArgMatches> {
    matches
        .subcommand()
        .map(|(_, subcommand)| subcommand)
        .ok_or_else(|| MapspliceError::Configuration {
            message: "selected command was not present in parsed arguments".to_owned(),
        })
}

/// Convert an `ortho-config` error into the library error type.
fn configuration_error(error: &impl ToString) -> MapspliceError {
    MapspliceError::Configuration {
        message: error.to_string(),
    }
}
