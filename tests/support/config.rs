//! Shared fixtures for CLI configuration integration tests.
//!
//! Configuration tests run the compiled `mapsplice` binary as a child process,
//! so each test owns its environment variables and working directory without
//! touching the test process.

#[path = "workspace.rs"]
mod workspace_support;

use std::process::{Command, Output};

use camino::Utf8PathBuf;
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

    /// Start a child-process invocation rooted at this workspace.
    ///
    /// The child starts with an empty environment and the workspace directory as
    /// its working directory, so no developer configuration leaks into a test.
    pub fn invocation(&self) -> TestResult<CliInvocation> {
        let root = self
            .target
            .parent()
            .ok_or_else(|| "target path should have a parent".to_owned())?;
        // Pin the discovery roots so a missing `HOME` cannot fall back to the
        // passwd home directory; tests that set them explicitly override these.
        Ok(CliInvocation {
            env_vars: vec![
                ("HOME", root.join("home").to_string()),
                ("XDG_CONFIG_HOME", root.join("empty-xdg").to_string()),
            ],
            cwd: root.to_path_buf(),
        })
    }
}

/// Captured result of one child-process CLI run.
pub struct CliOutcome {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

impl CliOutcome {
    fn from_output(output: &Output) -> Self {
        Self {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// Return standard output, or the child's diagnostics when it failed.
    pub fn into_stdout(self) -> TestResult<String> {
        if self.success {
            Ok(self.stdout)
        } else {
            Err(format!("mapsplice failed unexpectedly: {}", self.stderr).into())
        }
    }
}

/// Environment and working directory for one child-process CLI run.
pub struct CliInvocation {
    env_vars: Vec<(&'static str, String)>,
    cwd: Utf8PathBuf,
}

impl CliInvocation {
    pub fn set_env(&mut self, key: &'static str, value: impl AsRef<str>) {
        self.env_vars.push((key, value.as_ref().to_owned()));
    }

    /// Run `mapsplice` with the given arguments and collect its output.
    pub fn run(&self, arguments: &[&str]) -> TestResult<CliOutcome> {
        let mut command = Command::new(env!("CARGO_BIN_EXE_mapsplice"));
        command
            .args(arguments)
            .env_clear()
            .envs(self.env_vars.iter().map(|(key, value)| (*key, value)))
            .current_dir(self.cwd.as_std_path());
        Ok(CliOutcome::from_output(&command.output()?))
    }
}

#[fixture]
pub fn workspace() -> TestResult<Workspace> {
    let workspace = workspace_support::create_workspace()?;
    Ok(workspace)
}
