//! CLI parsing and `ortho-config` integration for `mapsplice`.

use std::{env, ffi::OsString, io};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8::Dir};
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
use ortho_config::{OrthoConfig, load_and_merge_subcommand_for, toml};
use serde::{Deserialize, Serialize};

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

/// Merge global defaults below explicit command-line global options.
fn load_global_config(config: &GlobalCli) -> Result<GlobalCli> {
    let mut merged = GlobalCli {
        in_place: global_config_file_default()?,
    };
    if let Some(in_place) = global_env_default()? {
        merged.in_place = Some(in_place);
    }
    if config.in_place == Some(true) {
        merged.in_place = config.in_place;
    }
    Ok(merged)
}

/// Load the `in_place` default from discovered TOML configuration files.
fn global_config_file_default() -> Result<Option<bool>> {
    let mut default = None;
    for path in global_config_candidates() {
        if let Ok(contents) = read_config_candidate(&path)?
            && let Some(in_place) = parse_global_config(&path, &contents)?
        {
            default = Some(in_place);
        }
    }
    Ok(default)
}

/// Return global configuration paths in increasing precedence order.
fn global_config_candidates() -> Vec<Utf8PathBuf> {
    let mut paths = Vec::new();
    if let Some(raw_xdg_home) = env::var_os("XDG_CONFIG_HOME")
        && let Ok(xdg_home) = Utf8PathBuf::from_path_buf(raw_xdg_home.into())
    {
        paths.push(xdg_home.join("mapsplice").join("config.toml"));
    }
    paths.push(Utf8PathBuf::from(".mapsplice.toml"));
    paths
}

/// Read one optional configuration candidate through a directory capability.
fn read_config_candidate(path: &Utf8Path) -> Result<std::result::Result<String, ()>> {
    let Some(parent) = path.parent() else {
        return Err(MapspliceError::Configuration {
            message: format!("configuration path `{path}` has no parent"),
        });
    };
    let Some(file_name) = path.file_name() else {
        return Err(MapspliceError::Configuration {
            message: format!("configuration path `{path}` has no file name"),
        });
    };
    let dir = match Dir::open_ambient_dir(parent, ambient_authority()) {
        Ok(dir) => dir,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Err(())),
        Err(error) => {
            return Err(MapspliceError::Configuration {
                message: format!("failed to open configuration directory `{parent}`: {error}"),
            });
        }
    };
    dir.read_to_string(file_name).map(Ok).or_else(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            Ok(Err(()))
        } else {
            Err(MapspliceError::Configuration {
                message: format!("failed to read `{path}`: {error}"),
            })
        }
    })
}

/// Parse the top-level global configuration from one TOML file.
fn parse_global_config(path: &Utf8Path, contents: &str) -> Result<Option<bool>> {
    let document =
        toml::from_str::<toml::Value>(contents).map_err(|error| MapspliceError::Configuration {
            message: format!("failed to parse `{path}`: {error}"),
        })?;
    document.get("in_place").map_or_else(
        || Ok(None),
        |in_place| {
            in_place
                .as_bool()
                .map(Some)
                .ok_or_else(|| MapspliceError::Configuration {
                    message: format!("`in_place` in `{path}` must be a boolean"),
                })
        },
    )
}

/// Load the `in_place` default from the environment.
fn global_env_default() -> Result<Option<bool>> {
    env::var("MAPSPLICE_IN_PLACE").map_or_else(
        |error| match error {
            env::VarError::NotPresent => Ok(None),
            env::VarError::NotUnicode(value) => Err(MapspliceError::Configuration {
                message: format!(
                    "MAPSPLICE_IN_PLACE is not valid Unicode: {}",
                    value.display()
                ),
            }),
        },
        |raw| parse_bool_env("MAPSPLICE_IN_PLACE", &raw).map(Some),
    )
}

/// Parse a boolean environment variable value.
fn parse_bool_env(name: &str, raw: &str) -> Result<bool> {
    raw.parse::<bool>()
        .map_err(|error| MapspliceError::Configuration {
            message: format!("{name} must be a boolean: {error}"),
        })
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
