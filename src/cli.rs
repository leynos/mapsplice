//! CLI parsing and `ortho-config` integration for `mapsplice`.

use std::ffi::OsString;

use camino::Utf8PathBuf;
use clap::{ArgAction, Args, CommandFactory, FromArgMatches, Parser, Subcommand};
use ortho_config::{OrthoConfig, load_and_merge_subcommand_for};
use serde::{Deserialize, Serialize};

use crate::{
    error::{MapspliceError, Result},
    roadmap::RoadmapAnchor,
};

/// Global options that can be loaded through `ortho-config`.
#[derive(Clone, Debug, Default, Parser)]
#[command(next_help_heading = "Global options")]
pub struct GlobalCli {
    /// Rewrite the target file instead of printing to stdout.
    #[arg(short = 'i', long = "in-place", global = true, action = ArgAction::SetTrue)]
    pub in_place: bool,
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
pub fn parse_cli_request<I, T>(args: I) -> Result<CliRequest>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let command = Cli::command();
    let matches = command.try_get_matches_from(args)?;
    let cli = Cli::from_arg_matches(&matches)?;
    let global = cli.global.resolve();
    cli.command.into_request(global)
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
    fn into_request(self, global: GlobalOptions) -> Result<CliRequest> {
        match self {
            Self::Append(args) => Ok(args.into_request(global)),
            Self::Insert(args) => args.into_request(global),
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
    const fn resolve(self) -> GlobalOptions {
        GlobalOptions {
            in_place: self.in_place,
        }
    }
}

impl AppendArgs {
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
    fn into_request(self, global: GlobalOptions) -> Result<CliRequest> {
        let merged = load_merged_config(&self.config)?;
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

impl DeleteArgs {
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

fn load_merged_config<C>(config: &C) -> Result<C>
where
    C: CommandFactory + Default + OrthoConfig + Serialize,
{
    load_and_merge_subcommand_for(config).map_err(|error| MapspliceError::Configuration {
        message: error.to_string(),
    })
}
