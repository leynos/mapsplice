//! Global configuration discovery and parsing for the CLI adapter.

use std::{env, io};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8::Dir};
use ortho_config::toml;

use super::GlobalCli;
use crate::error::{MapspliceError, Result};

/// Merge global defaults below explicit command-line global options.
pub(super) fn load_global_config(config: &GlobalCli) -> Result<GlobalCli> {
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
    let parent = path
        .parent()
        .filter(|parent| !parent.as_str().is_empty())
        .unwrap_or_else(|| Utf8Path::new("."));
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
