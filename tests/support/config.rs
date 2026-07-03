//! Shared fixtures for CLI configuration integration tests.

#[path = "workspace.rs"]
mod workspace_support;

use std::{
    env,
    ffi::OsString,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

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

    pub fn enter_root(&self, guard: &mut ProcessStateGuard) -> TestResult {
        let parent = self
            .target
            .parent()
            .ok_or_else(|| "target path should have a parent".to_owned())?;
        guard.enter_dir(parent)
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

pub struct ProcessStateGuard {
    _lock: MutexGuard<'static, ()>,
    saved_env: Vec<(&'static str, Option<OsString>)>,
    saved_cwd: Option<PathBuf>,
}

impl ProcessStateGuard {
    pub fn acquire() -> TestResult<Self> {
        let lock = ENV_LOCK.lock()?;
        Ok(Self {
            _lock: lock,
            saved_env: Vec::new(),
            saved_cwd: None,
        })
    }

    pub fn set_env(&mut self, key: &'static str, value: impl AsRef<str>) {
        self.remember_env(key);
        // SAFETY: tests mutate process environment only while holding ENV_LOCK,
        // and the guard restores the previous value before releasing it.
        unsafe {
            env::set_var(key, value.as_ref());
        }
    }

    pub fn remove_env(&mut self, key: &'static str) {
        self.remember_env(key);
        // SAFETY: tests mutate process environment only while holding ENV_LOCK,
        // and the guard restores the previous value before releasing it.
        unsafe {
            env::remove_var(key);
        }
    }

    pub fn enter_dir(&mut self, path: &Utf8Path) -> TestResult {
        if self.saved_cwd.is_none() {
            self.saved_cwd = Some(env::current_dir()?);
        }
        env::set_current_dir(path.as_std_path())?;
        Ok(())
    }

    fn remember_env(&mut self, key: &'static str) {
        if self
            .saved_env
            .iter()
            .any(|(saved_key, _)| *saved_key == key)
        {
            return;
        }
        self.saved_env.push((key, env::var_os(key)));
    }
}

impl Drop for ProcessStateGuard {
    fn drop(&mut self) {
        if let Some(previous) = &self.saved_cwd
            && let Err(_error) = env::set_current_dir(previous)
        {}
        // SAFETY: ProcessStateGuard owns ENV_LOCK for its full lifetime,
        // serializing environment mutation and restoration in tests.
        unsafe {
            for (key, saved_env_value) in self.saved_env.iter().rev() {
                if let Some(original_value) = saved_env_value {
                    env::set_var(key, original_value);
                } else {
                    env::remove_var(key);
                }
            }
        }
    }
}

#[fixture]
pub fn workspace() -> TestResult<Workspace> {
    let workspace = workspace_support::create_workspace()?;
    Ok(workspace)
}
