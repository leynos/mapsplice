//! Configuration discovery and parsing for the CLI adapter.
//!
//! Configuration inputs are split into the two capabilities `ortho-config`
//! defines: named environment lookups used during discovery, and environment
//! enumeration used for the merge layer. [`ConfigSources`] pairs those
//! capabilities with an explicit local file base, so production code can read
//! the live process and tests can supply isolated values.

use std::{ffi::OsString, io, sync::Arc};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8::Dir};
use ortho_config::{EnvSource, ProcessEnv, SharedEnvSource, SharedScanEnvSource, toml};

use super::GlobalCli;
use crate::error::{MapspliceError, Result};

/// Sources used to resolve configuration.
///
/// Production code uses [`ConfigSources::process`]. Tests and embedding
/// applications use [`ConfigSources::new`] with an explicit base and injected
/// environment sources, so no process-global state is read.
#[derive(Clone, Debug)]
pub struct ConfigSources {
    /// Directory used to resolve the local configuration file.
    base: Utf8PathBuf,
    /// Named lookups consulted during discovery.
    discovery: SharedEnvSource,
    /// Enumerable source used for the environment merge layer.
    merge: SharedScanEnvSource,
}

impl ConfigSources {
    /// Use the live process environment and the current working directory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mapsplice::ConfigSources;
    ///
    /// let sources = ConfigSources::process();
    /// drop(sources);
    /// ```
    #[must_use]
    pub fn process() -> Self {
        Self {
            base: Utf8PathBuf::from("."),
            discovery: Arc::new(ProcessEnv),
            merge: Arc::new(ProcessEnv),
        }
    }

    /// Build sources from explicit discovery and merging inputs.
    ///
    /// `base` is the directory that contains the local `.mapsplice.toml`
    /// candidate. Naming both XDG variables in the discovery source keeps
    /// discovery from falling back to the host's `/etc/xdg`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    ///
    /// use mapsplice::ConfigSources;
    /// use ortho_config::MapEnv;
    ///
    /// let environment = Arc::new(
    ///     MapEnv::new()
    ///         .with_var("XDG_CONFIG_HOME", "/srv/mapsplice/config")
    ///         .with_var("XDG_CONFIG_DIRS", "/srv/mapsplice/fallback"),
    /// );
    /// let sources = ConfigSources::new("/srv/mapsplice/project", environment.clone(), environment);
    /// drop(sources);
    /// ```
    #[must_use]
    pub fn new(
        base: impl Into<Utf8PathBuf>,
        discovery: SharedEnvSource,
        merge: SharedScanEnvSource,
    ) -> Self {
        Self {
            base: base.into(),
            discovery,
            merge,
        }
    }

    /// Return the base directory used for local configuration discovery.
    pub(super) fn base(&self) -> &Utf8Path { &self.base }

    /// Return the source used for named discovery lookups.
    pub(super) fn discovery(&self) -> &dyn EnvSource { self.discovery.as_ref() }

    /// Return the source used for the environment merge layer.
    pub(super) fn merge(&self) -> SharedScanEnvSource { Arc::clone(&self.merge) }

    /// Return the local configuration path in increasing precedence order.
    fn local_config_path(&self) -> Utf8PathBuf {
        if self.base == Utf8Path::new(".") {
            // Keep the historical diagnostic spelling for process discovery.
            Utf8PathBuf::from(".mapsplice.toml")
        } else {
            self.base.join(".mapsplice.toml")
        }
    }
}

/// Merge global defaults below explicit command-line global options.
pub(super) fn load_global_config(config: &GlobalCli, sources: &ConfigSources) -> Result<GlobalCli> {
    let mut merged = GlobalCli {
        in_place: global_config_file_default(sources)?,
    };
    if let Some(in_place) = global_env_default(sources)? {
        merged.in_place = Some(in_place);
    }
    if config.in_place == Some(true) {
        merged.in_place = config.in_place;
    }
    Ok(merged)
}

/// Load the `in_place` default from discovered TOML configuration files.
fn global_config_file_default(sources: &ConfigSources) -> Result<Option<bool>> {
    let mut default = None;
    for path in global_config_candidates(sources) {
        if let Ok(contents) = read_config_candidate(&path)?
            && let Some(in_place) = parse_global_config(&path, &contents)?
        {
            default = Some(in_place);
        }
    }
    Ok(default)
}

/// Return global configuration paths in increasing precedence order.
fn global_config_candidates(sources: &ConfigSources) -> Vec<Utf8PathBuf> {
    let mut paths = Vec::new();
    if let Some(raw_xdg_home) = sources.discovery().get("XDG_CONFIG_HOME")
        && let Ok(xdg_home) = Utf8PathBuf::from_path_buf(raw_xdg_home.into())
    {
        paths.push(xdg_home.join("mapsplice").join("config.toml"));
    }
    paths.push(sources.local_config_path());
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

/// Load the `in_place` default from the discovery source.
fn global_env_default(sources: &ConfigSources) -> Result<Option<bool>> {
    sources.discovery().get("MAPSPLICE_IN_PLACE").map_or_else(
        || Ok(None),
        |raw| env_value_to_bool("MAPSPLICE_IN_PLACE", raw).map(Some),
    )
}

/// Convert one environment value into a boolean or a configuration error.
fn env_value_to_bool(name: &str, raw: OsString) -> Result<bool> {
    raw.into_string().map_or_else(
        |value| {
            Err(MapspliceError::Configuration {
                message: format!("{name} is not valid Unicode: {}", value.display()),
            })
        },
        |value| parse_bool_env(name, &value),
    )
}

/// Parse a boolean environment variable value.
fn parse_bool_env(name: &str, raw: &str) -> Result<bool> {
    raw.parse::<bool>()
        .map_err(|error| MapspliceError::Configuration {
            message: format!("{name} must be a boolean: {error}"),
        })
}
