//! Shared fixtures for unit-style integration tests.

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
pub use workspace_support::{TestResult, Workspace, create_workspace};

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

pub const TARGET_THREE_PHASES: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. First task.\n\n",
    "## 2. Phase two\n\n",
    "### 2.1. Step two\n\n",
    "- [ ] 2.1.1. Middle task.\n\n",
    "## 3. Phase three\n\n",
    "### 3.1. Step three\n\n",
    "- [ ] 3.1.1. Final task. Requires 3.1.1.\n",
);

pub const PHASE_FRAGMENT: &str = concat!(
    "## 9. Inserted phase\n\n",
    "### 9.1. Added step\n\n",
    "- [ ] 9.1.1. Added task. Requires 9.1.1.\n",
);

pub const TASK_FRAGMENT: &str = "- [ ] 9.9.9. Inserted task. Requires 9.9.9.\n";

pub const REPLACEMENT_FRAGMENT: &str = concat!(
    "## 7. Replacement phase A\n\n",
    "### 7.1. Step A\n\n",
    "- [ ] 7.1.1. Replacement task A.\n\n",
    "## 8. Replacement phase B\n\n",
    "### 8.1. Step B\n\n",
    "- [ ] 8.1.1. Replacement task B. Requires 8.1.1.\n",
);

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

    pub fn enter_root(&self) -> TestResult<CwdGuard> {
        let parent = self
            .target
            .parent()
            .ok_or_else(|| "target path should have a parent".to_owned())?;
        CwdGuard::enter(parent)
    }

    pub fn read_target(&self) -> TestResult<String> { Ok(self.dir.read_to_string("target.md")?) }
}

pub struct EnvVarGuard {
    _lock: MutexGuard<'static, ()>,
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvVarGuard {
    pub fn set(key: &'static str, value: impl AsRef<str>) -> TestResult<Self> {
        let lock = ENV_LOCK.lock()?;
        let previous = env::var_os(key);
        // SAFETY: tests mutate process environment only while holding ENV_LOCK,
        // and the guard restores the previous value before releasing it.
        unsafe {
            env::set_var(key, value.as_ref());
        }
        Ok(Self {
            _lock: lock,
            key,
            previous,
        })
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        // SAFETY: EnvVarGuard owns ENV_LOCK for its full lifetime, serializing
        // environment mutation and restoration in tests.
        unsafe {
            if let Some(previous) = &self.previous {
                env::set_var(self.key, previous);
            } else {
                env::remove_var(self.key);
            }
        }
    }
}

pub struct CwdGuard {
    _lock: MutexGuard<'static, ()>,
    previous: PathBuf,
}

impl CwdGuard {
    pub fn enter(path: &Utf8Path) -> TestResult<Self> {
        let lock = ENV_LOCK.lock()?;
        let previous = env::current_dir()?;
        env::set_current_dir(path.as_std_path())?;
        Ok(Self {
            _lock: lock,
            previous,
        })
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) { if let Err(_error) = env::set_current_dir(&self.previous) {} }
}

#[fixture]
pub fn workspace() -> TestResult<Workspace> {
    let workspace = create_workspace()?;
    Ok(workspace)
}
