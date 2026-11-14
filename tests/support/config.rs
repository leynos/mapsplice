//! Shared fixtures for CLI configuration integration tests.

#[path = "workspace.rs"]
mod workspace_support;

use std::{ffi::OsStr, sync::Arc};

use camino::Utf8PathBuf;
use mapsplice::ConfigSources;
use ortho_config::MapEnv;
use rstest::fixture;
pub use workspace_support::{TestResult, Workspace};

pub const TARGET_TWO_PHASES: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. First task.\n\n",
    "## 2. Phase two\n\n",
    "### 2.1. Step two\n\n",
    "- [ ] 2.1.1. Second task. Requires 2.1.1.\n",
);

pub const TARGET_TWO_TASKS: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. First task.\n",
    "- [ ] 1.1.2. Second task. Depends on 1.1.1 and 1.1.2.\n",
);

pub const TASK_FRAGMENT: &str = "- [ ] 9.9.9. Inserted task. Requires 9.9.9.\n";

/// Injected configuration inputs rooted at a test workspace.
///
/// The fixture starts with empty home and XDG directories, so a test that does
/// not override a variable cannot discover a file outside its workspace.
pub struct TestSources {
    /// Directory used for local configuration discovery.
    base: Utf8PathBuf,
    /// Environment visible to discovery and merging.
    environment: MapEnv,
}

impl TestSources {
    /// Create sources whose home and XDG directories are empty.
    pub fn new(workspace: &Workspace) -> TestResult<Self> {
        let base = workspace.root()?;
        workspace.dir.create_dir_all("empty-xdg")?;
        workspace.dir.create_dir_all("empty-home")?;
        let environment = MapEnv::new()
            .with_var("HOME", base.join("empty-home").as_str())
            .with_var("XDG_CONFIG_HOME", base.join("empty-xdg").as_str())
            .with_var("XDG_CONFIG_DIRS", base.join("empty-xdg").as_str());
        Ok(Self { base, environment })
    }

    /// Replace one environment value visible to configuration loading.
    pub fn set(&mut self, key: &str, value: impl AsRef<OsStr>) {
        self.environment.insert(key, value);
    }

    /// Build configuration sources for one command invocation.
    pub fn sources(&self) -> ConfigSources {
        let environment = Arc::new(self.environment.clone());
        ConfigSources::new(self.base.clone(), environment.clone(), environment)
    }
}

impl Workspace {
    /// Return the UTF-8 directory that contains the workspace files.
    pub fn root(&self) -> TestResult<Utf8PathBuf> {
        self.target
            .parent()
            .map(Utf8PathBuf::from)
            .ok_or_else(|| "target path should have a parent".into())
    }

    pub fn write_xdg_config(&self, contents: &str) -> TestResult<Utf8PathBuf> {
        self.dir.create_dir_all("mapsplice")?;
        self.dir.write("mapsplice/config.toml", contents)?;
        self.root()
    }

    pub fn write_local_config(&self, contents: &str) -> TestResult {
        self.dir.write(".mapsplice.toml", contents)?;
        Ok(())
    }

    pub fn read_target(&self) -> TestResult<String> { Ok(self.dir.read_to_string("target.md")?) }

    pub fn write_home_config(&self, contents: &str) -> TestResult<Utf8PathBuf> {
        self.dir.create_dir_all("home")?;
        self.dir.write("home/.mapsplice.toml", contents)?;
        Ok(self.root()?.join("home"))
    }
}

#[fixture]
pub fn workspace() -> TestResult<Workspace> {
    let workspace = workspace_support::create_workspace()?;
    Ok(workspace)
}
