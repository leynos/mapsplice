//! Shared fixtures for CLI configuration integration tests.

#[path = "workspace.rs"]
mod workspace_support;

use std::{env, path::PathBuf, sync::Mutex};

use camino::{Utf8Path, Utf8PathBuf};
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

static ENV_LOCK: Mutex<()> = Mutex::new(());

impl Workspace {
    pub fn write_xdg_config(&self, contents: &str) -> TestResult<Utf8PathBuf> {
        self.dir.create_dir_all("mapsplice")?;
        self.dir.write("mapsplice/config.toml", contents)?;
        let parent = self
            .target
            .parent()
            .ok_or_else(|| "target path should have a parent".to_owned())?;
        Ok(parent.to_path_buf())
    }

    pub fn write_local_config(&self, contents: &str) -> TestResult {
        self.dir.write(".mapsplice.toml", contents)?;
        Ok(())
    }

    pub fn enter_root(&self, state: &mut ProcessState) -> TestResult {
        let parent = self
            .target
            .parent()
            .ok_or_else(|| "target path should have a parent".to_owned())?;
        state.enter_dir(parent);
        Ok(())
    }

    pub fn read_target(&self) -> TestResult<String> { Ok(self.dir.read_to_string("target.md")?) }

    pub fn write_home_config(&self, contents: &str) -> TestResult<Utf8PathBuf> {
        self.dir.create_dir_all("home")?;
        self.dir.write("home/.mapsplice.toml", contents)?;
        let parent = self
            .target
            .parent()
            .ok_or_else(|| "target path should have a parent".to_owned())?;
        Ok(parent.join("home"))
    }
}

#[derive(Default)]
pub struct ProcessState {
    env_vars: Vec<(&'static str, Option<String>)>,
    cwd: Option<Utf8PathBuf>,
}

impl ProcessState {
    pub fn set_env(&mut self, key: &'static str, value: impl AsRef<str>) {
        self.env_vars.push((key, Some(value.as_ref().to_owned())));
    }

    pub fn remove_env(&mut self, key: &'static str) { self.env_vars.push((key, None)); }

    pub fn enter_dir(&mut self, path: &Utf8Path) { self.cwd = Some(path.to_path_buf()); }

    pub fn run<R>(&self, action: impl FnOnce() -> TestResult<R>) -> TestResult<R> {
        let _lock = ENV_LOCK.lock()?;
        temp_env::with_vars(&self.env_vars, || {
            let _cwd = self.cwd.as_deref().map(CwdRestore::enter).transpose()?;
            action()
        })
    }
}

struct CwdRestore(PathBuf);

impl CwdRestore {
    fn enter(path: &Utf8Path) -> TestResult<Self> {
        let previous = env::current_dir()?;
        env::set_current_dir(path.as_std_path())?;
        Ok(Self(previous))
    }
}

impl Drop for CwdRestore {
    fn drop(&mut self) { if let Err(_error) = env::set_current_dir(&self.0) {} }
}

#[fixture]
pub fn workspace() -> TestResult<Workspace> {
    let workspace = workspace_support::create_workspace()?;
    Ok(workspace)
}
