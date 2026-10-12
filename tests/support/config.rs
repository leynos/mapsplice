//! Shared fixtures for CLI configuration integration tests.

#[path = "workspace.rs"]
mod workspace_support;

use std::{env, io::Write, path::PathBuf, sync::Mutex};

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
        // `temp_env::with_vars` and `CwdRestore` restore changes during unwinding,
        // so a prior test's panic must not prevent later tests from taking this lock.
        let _lock = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
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
    fn drop(&mut self) {
        if let Err(error) = env::set_current_dir(&self.0) {
            let diagnostic = format!(
                "failed to restore working directory {}: {error}",
                self.0.display()
            );
            if std::thread::panicking() {
                let _write_result = writeln!(std::io::stderr().lock(), "{diagnostic}");
            } else {
                panic!("{diagnostic}");
            }
        }
    }
}

#[fixture]
pub fn workspace() -> TestResult<Workspace> {
    let workspace = workspace_support::create_workspace()?;
    Ok(workspace)
}

#[cfg(test)]
mod cwd_restore_tests {
    //! Verify that an un-restorable working directory fails its enclosing test.

    use std::panic::catch_unwind;

    use super::{CwdRestore, ProcessState, TestResult};

    #[test]
    fn restoration_failure_fails_the_enclosing_test() {
        let temporary_directory =
            tempfile::tempdir().expect("temporary directory creation should succeed");
        let missing_directory = temporary_directory.path().join("removed-working-directory");

        let panic = catch_unwind(|| drop(CwdRestore(missing_directory.clone())));
        let failure = panic.expect_err("working-directory restore failure must be reported");
        let diagnostic = failure
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| {
                failure
                    .downcast_ref::<&str>()
                    .map(|message| (*message).to_owned())
            })
            .unwrap_or_default();

        assert!(
            diagnostic.contains(&missing_directory.to_string_lossy().to_string()),
            "restore diagnostic should name the missing directory; observed {diagnostic:?}"
        );
    }

    fn panic_while_holding_state() -> ! {
        panic!("intentional panic while holding the process state guard");
    }

    fn require_process_state(
        key: &str,
        expected_environment: Option<&std::ffi::OsStr>,
        expected_directory: &std::path::Path,
    ) -> TestResult {
        if std::env::var_os(key).as_deref() != expected_environment {
            return Err("process state environment was not restored or applied".into());
        }
        if std::env::current_dir()? != expected_directory {
            return Err("process state working directory was not restored or applied".into());
        }
        Ok(())
    }

    #[test]
    fn process_state_recovers_after_panic_and_restores_state() -> TestResult {
        const KEY: &str = "MAPSPLICE_PROCESS_STATE_PANIC_TEST";
        let baseline = ProcessState::default()
            .run(|| Ok((std::env::var_os(KEY), std::env::current_dir()?)))?;
        let temporary_directory = tempfile::tempdir()?;
        let temporary_path = camino::Utf8Path::from_path(temporary_directory.path())
            .ok_or("temporary directory must have a UTF-8 path")?;
        let mut state = ProcessState::default();
        state.set_env(KEY, "temporary");
        state.enter_dir(temporary_path);
        let verify_temporary = || {
            require_process_state(
                KEY,
                Some(std::ffi::OsStr::new("temporary")),
                temporary_directory.path(),
            )
        };
        let verify_restored = || require_process_state(KEY, baseline.0.as_deref(), &baseline.1);

        let caught = catch_unwind(|| {
            state.run::<()>(|| {
                verify_temporary()?;
                panic_while_holding_state()
            })
        });
        if caught.is_ok() {
            return Err("the action must unwind while holding the lock".into());
        }
        ProcessState::default().run(verify_restored)?;
        state.run(verify_temporary)?;
        ProcessState::default().run(verify_restored)
    }
}
