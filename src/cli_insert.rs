//! Insert-command arguments and their configuration merge.
//!
//! The insert command has the only optional setting whose absence must stay
//! distinguishable from `false`: clap records a bare `--after` flag as `true`
//! but also records its absence as `false`, which would shadow a configured
//! default. The argument type and that correction live together here so the
//! parent CLI module stays within its size budget.

use camino::Utf8PathBuf;
use clap::{ArgAction, ArgMatches, Args, Parser, parser::ValueSource};
use ortho_config::OrthoConfig;
use serde::{Deserialize, Serialize};

use super::{CliRequest, CommandKind, GlobalOptions, Result, RoadmapAnchor, load_merged_config};

/// Positional inputs and optional settings for insertion.
#[derive(Debug, Args)]
#[command(about = None)]
pub(super) struct InsertArgs {
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

impl InsertArgs {
    /// Build an insert request after merging optional config defaults.
    pub(super) fn into_request(
        self,
        global: GlobalOptions,
        matches: &ArgMatches,
    ) -> Result<CliRequest> {
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
