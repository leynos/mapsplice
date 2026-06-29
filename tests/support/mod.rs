//! Shared fixtures for integration and behavioural tests.

#![expect(
    clippy::allow_attributes,
    reason = "dead-code suppressions are scoped to helpers shared across test crates"
)]

use std::{
    env,
    error::Error,
    ffi::OsString,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8::Dir};
use rstest::fixture;
use tempfile::TempDir;

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

#[allow(
    dead_code,
    reason = "shared fixtures are used by different test crates"
)]
pub const TARGET_TWO_PHASES: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. First task.\n\n",
    "## 2. Phase two\n\n",
    "### 2.1. Step two\n\n",
    "- [ ] 2.1.1. Second task. Requires 2.1.1.\n",
);

#[allow(
    dead_code,
    reason = "shared fixtures are used by different test crates"
)]
pub const TARGET_TWO_TASKS: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. First task.\n",
    "- [ ] 1.1.2. Second task. Depends on 1.1.1 and 1.1.2.\n",
);

#[allow(
    dead_code,
    reason = "shared fixtures are used by different test crates"
)]
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

#[allow(
    dead_code,
    reason = "shared fixtures are used by different test crates"
)]
pub const PHASE_FRAGMENT: &str = concat!(
    "## 9. Inserted phase\n\n",
    "### 9.1. Added step\n\n",
    "- [ ] 9.1.1. Added task. Requires 9.1.1.\n",
);

#[allow(
    dead_code,
    reason = "shared fixtures are used by different test crates"
)]
pub const TASK_FRAGMENT: &str = "- [ ] 9.9.9. Inserted task. Requires 9.9.9.\n";

#[allow(
    dead_code,
    reason = "shared fixtures are used by different test crates"
)]
pub const REPLACEMENT_FRAGMENT: &str = concat!(
    "## 7. Replacement phase A\n\n",
    "### 7.1. Step A\n\n",
    "- [ ] 7.1.1. Replacement task A.\n\n",
    "## 8. Replacement phase B\n\n",
    "### 8.1. Step B\n\n",
    "- [ ] 8.1.1. Replacement task B. Requires 8.1.1.\n",
);

#[allow(
    dead_code,
    reason = "shared fixtures are used by different test crates"
)]
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug)]
pub struct Workspace {
    pub dir: Dir,
    pub target: Utf8PathBuf,
    pub fragment: Utf8PathBuf,
    _tempdir: TempDir,
}

impl Workspace {
    pub fn write_target(&self, contents: &str) -> TestResult {
        self.dir.write("target.md", contents)?;
        Ok(())
    }

    pub fn write_fragment(&self, contents: &str) -> TestResult {
        self.dir.write("fragment.md", contents)?;
        Ok(())
    }

    #[allow(
        dead_code,
        reason = "shared fixtures are used by different test crates"
    )]
    pub fn write_xdg_config(&self, contents: &str) -> TestResult<Utf8PathBuf> {
        self.dir.create_dir_all("mapsplice")?;
        self.dir.write("mapsplice/config.toml", contents)?;
        let parent = self
            .target
            .parent()
            .ok_or_else(|| "target path should have a parent".to_owned())?;
        Ok(parent.to_path_buf())
    }

    #[allow(
        dead_code,
        reason = "shared fixtures are used by different test crates"
    )]
    pub fn write_local_config(&self, contents: &str) -> TestResult {
        self.dir.write(".mapsplice.toml", contents)?;
        Ok(())
    }

    #[allow(
        dead_code,
        reason = "shared fixtures are used by different test crates"
    )]
    pub fn enter_root(&self) -> TestResult<CwdGuard> {
        let parent = self
            .target
            .parent()
            .ok_or_else(|| "target path should have a parent".to_owned())?;
        CwdGuard::enter(parent)
    }

    #[allow(
        dead_code,
        reason = "shared fixtures are used by different test crates"
    )]
    pub fn read_target(&self) -> TestResult<String> { Ok(self.dir.read_to_string("target.md")?) }
}

#[allow(
    dead_code,
    reason = "shared fixtures are used by different test crates"
)]
pub struct EnvVarGuard {
    _lock: MutexGuard<'static, ()>,
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvVarGuard {
    #[allow(
        dead_code,
        reason = "shared fixtures are used by different test crates"
    )]
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

#[allow(
    dead_code,
    reason = "shared fixtures are used by different test crates"
)]
pub struct CwdGuard {
    _lock: MutexGuard<'static, ()>,
    previous: PathBuf,
}

impl CwdGuard {
    #[allow(
        dead_code,
        reason = "shared fixtures are used by different test crates"
    )]
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

pub fn create_workspace() -> TestResult<Workspace> {
    let tempdir = tempfile::tempdir()?;
    let root = Utf8PathBuf::from_path_buf(tempdir.path().to_path_buf())
        .map_err(|path| format!("temporary directory is not valid UTF-8: {}", path.display()))?;
    let dir = Dir::open_ambient_dir(&root, ambient_authority())?;
    Ok(Workspace {
        _tempdir: tempdir,
        dir,
        target: root.join("target.md"),
        fragment: root.join("fragment.md"),
    })
}

#[fixture]
pub fn workspace() -> TestResult<Workspace> {
    let workspace = create_workspace()?;
    Ok(workspace)
}
