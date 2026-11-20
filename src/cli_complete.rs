//! Complete-command arguments and done-suffix validation.
//!
//! Completion accepts one optional setting, the done suffix, which is merged
//! from the command line, the environment, and configuration files. The
//! suffix is validated here because line breaks are a configuration error: the
//! patcher inserts it inside one Markdown paragraph and must stay total over
//! single-line input.

use camino::Utf8PathBuf;
use clap::{Args, Parser};
use ortho_config::OrthoConfig;
use serde::{Deserialize, Serialize};

use super::{
    CliRequest,
    CommandKind,
    GlobalOptions,
    MapspliceError,
    Result,
    RoadmapAnchor,
    load_merged_config,
};

/// Positional inputs and optional settings for completing one roadmap item.
#[derive(Debug, Args)]
#[command(about = None)]
pub(super) struct CompleteArgs {
    /// Completion settings merged with environment and file defaults.
    #[command(flatten)]
    config: CompleteConfig,
    /// Roadmap file to modify.
    #[arg(help = None)]
    target: Utf8PathBuf,
    /// Task or addendum sub-task to mark as done.
    #[arg(help = None)]
    anchor: RoadmapAnchor,
}

/// Optional completion settings before configuration sources are merged.
#[derive(Clone, Debug, Default, Parser, Serialize, Deserialize, OrthoConfig)]
#[command(name = "complete")]
#[ortho_config(prefix = "MAPSPLICE_")]
struct CompleteConfig {
    /// Text appended to the completed item's title.
    #[arg(long, help = None)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    suffix: Option<String>,
}

impl CompleteArgs {
    /// Build a completion request after merging optional config defaults.
    pub(super) fn into_request(self, global: GlobalOptions) -> Result<CliRequest> {
        let merged = load_merged_config(&self.config)?;
        Ok(CliRequest {
            global,
            target: self.target,
            command: CommandKind::Complete {
                anchor: self.anchor,
                suffix: validated_suffix(merged.suffix)?,
            },
        })
    }
}

/// Trim the merged suffix, treating blank input as "no suffix".
///
/// A configured suffix is placed inside a Markdown bullet, so a line break
/// would either end the bullet or split the sentence. Rejecting it here keeps
/// the textual patcher total over single-line input instead of having it
/// silently repair malformed configuration.
fn validated_suffix(suffix: Option<String>) -> Result<Option<String>> {
    let Some(configured) = suffix else {
        return Ok(None);
    };
    if configured.contains(['\n', '\r']) {
        return Err(MapspliceError::Configuration {
            message: "the complete suffix must be a single line".to_owned(),
        });
    }
    let trimmed = configured.trim();
    Ok((!trimmed.is_empty()).then(|| trimmed.to_owned()))
}
