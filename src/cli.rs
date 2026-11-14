//! CLI parsing and `ortho-config` integration for `mapsplice`.

#[path = "cli_export.rs"]
mod cli_export;
#[path = "cli_config.rs"]
mod config;
#[path = "cli_request.rs"]
mod request;

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
use ortho_config::{
    OrthoConfig,
    SubcommandFileContext,
    load_and_merge_subcommand_for_with_sources_at,
};
use serde::{Deserialize, Serialize};

use self::{cli_export::ExportArgs, config::load_global_config};
pub use self::{
    cli_export::ExportFormat,
    config::ConfigSources,
    request::{CliRequest, CommandKind, GlobalOptions},
};
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
    parse_cli_request_with_sources(args, &ConfigSources::process())
}

/// Parse a CLI request using explicit configuration sources.
///
/// Use this entry point when configuration discovery and merging must not read
/// process-global state. [`ConfigSources::process`] supplies the production
/// behaviour.
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
///
/// use mapsplice::{ConfigSources, parse_cli_request_with_sources};
/// use ortho_config::MapEnv;
///
/// # fn main() -> mapsplice::Result<()> {
/// let environment = Arc::new(MapEnv::new());
/// let sources = ConfigSources::new(".", environment.clone(), environment);
/// let request = parse_cli_request_with_sources(
///     ["mapsplice", "--in-place", "delete", "roadmap.md", "2"],
///     &sources,
/// )?;
///
/// assert!(request.global.in_place);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error when command-line parsing fails or when configured
/// defaults cannot be merged.
pub fn parse_cli_request_with_sources<I, T>(args: I, sources: &ConfigSources) -> Result<CliRequest>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let command = Cli::command();
    let matches = command.try_get_matches_from(args)?;
    let cli = Cli::from_arg_matches(&matches)?;
    let global = load_global_config(&cli.global, sources)?.resolve();
    cli.command.into_request(global, &matches, sources)
}

/// Top-level clap parser before configuration defaults are resolved.
#[derive(Debug, Parser)]
#[command(author, version, about = None, long_about = None)]
struct Cli {
    /// Global flags that may be merged with other configuration sources.
    #[command(flatten)]
    global: GlobalCli,
    /// Selected operation and its command-specific arguments.
    #[command(subcommand)]
    command: Commands,
}

/// Clap's parsed subcommand variants before conversion to domain commands.
#[derive(Debug, Subcommand)]
#[command(about = None)]
enum Commands {
    /// Collect the target and fragment for an append operation.
    #[command(about = None)]
    Append(AppendArgs),
    /// Collect the insertion anchor, fragment, and optional defaults.
    #[command(about = None)]
    Insert(InsertArgs),
    /// Collect the target and anchor for a delete operation.
    #[command(about = None)]
    Delete(DeleteArgs),
    /// Collect the target, anchor, and fragment for a replacement.
    #[command(about = None)]
    Replace(ReplaceArgs),
    /// Read and export a roadmap without entering the mutation pipeline.
    #[command(about = None)]
    Export(ExportArgs),
}

impl Commands {
    /// Convert the selected subcommand into an executable CLI request.
    fn into_request(
        self,
        global: GlobalOptions,
        matches: &ArgMatches,
        sources: &ConfigSources,
    ) -> Result<CliRequest> {
        match self {
            Self::Append(args) => Ok(args.into_request(global)),
            Self::Insert(args) => args.into_request(global, subcommand_matches(matches)?, sources),
            Self::Delete(args) => Ok(args.into_request(global)),
            Self::Replace(args) => Ok(args.into_request(global)),
            Self::Export(args) => Ok(args.into_request()),
        }
    }
}

/// Positional paths for appending a fragment to a roadmap.
#[derive(Debug, Args)]
#[command(about = None)]
struct AppendArgs {
    /// Roadmap file to extend.
    #[arg(help = None)]
    target: Utf8PathBuf,
    /// Markdown fragment to append.
    #[arg(help = None)]
    fragment: Utf8PathBuf,
}

/// Positional inputs and optional settings for insertion.
#[derive(Debug, Args)]
#[command(about = None)]
struct InsertArgs {
    /// Insert settings merged with environment and file defaults.
    #[command(flatten)]
    config: InsertConfig,
    /// Roadmap file to modify.
    #[arg(help = None)]
    target: Utf8PathBuf,
    /// Existing item before or after which to insert.
    #[arg(help = None)]
    anchor: RoadmapAnchor,
    /// Markdown fragment to insert.
    #[arg(help = None)]
    fragment: Utf8PathBuf,
}

/// Optional insert settings before configuration sources are merged.
#[derive(Clone, Debug, Default, Parser, Serialize, Deserialize, OrthoConfig)]
#[command(name = "insert")]
#[ortho_config(prefix = "MAPSPLICE_")]
struct InsertConfig {
    /// Insert after the anchor when true; absence preserves lower-priority defaults.
    #[arg(long, action = ArgAction::SetTrue, help = None)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    after: Option<bool>,
}

/// Positional inputs for deleting one roadmap item.
#[derive(Debug, Args)]
#[command(about = None)]
struct DeleteArgs {
    /// Roadmap file to modify.
    #[arg(help = None)]
    target: Utf8PathBuf,
    /// Item to delete.
    #[arg(help = None)]
    anchor: RoadmapAnchor,
}

/// Positional inputs for replacing one roadmap item.
#[derive(Debug, Args)]
#[command(about = None)]
struct ReplaceArgs {
    /// Roadmap file to modify.
    #[arg(help = None)]
    target: Utf8PathBuf,
    /// Item whose content is replaced.
    #[arg(help = None)]
    anchor: RoadmapAnchor,
    /// Markdown fragment containing the replacement.
    #[arg(help = None)]
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
    fn into_request(
        self,
        global: GlobalOptions,
        matches: &ArgMatches,
        sources: &ConfigSources,
    ) -> Result<CliRequest> {
        let config = self.config.with_absent_flags_removed(matches);
        let merged = load_merged_config(&config, sources)?;
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
fn load_merged_config<C>(config: &C, sources: &ConfigSources) -> Result<C>
where
    C: CommandFactory + Default + OrthoConfig + Serialize,
{
    let files = SubcommandFileContext::new(sources.base().as_std_path(), sources.discovery());
    load_and_merge_subcommand_for_with_sources_at(config, files, sources.merge())
        .map_err(|error| configuration_error(&error))
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
