//! Completion suffix configuration defaults and precedence tests.
//!
//! Each case runs the compiled binary as a child process, so environment
//! variables and the working directory belong to the child alone.

use rstest::rstest;

use super::{
    CliInvocation,
    CliOutcome,
    TARGET_TWO_TASKS,
    TestResult,
    Workspace,
    assert_contains,
    assert_equal,
    empty_xdg_home,
    workspace,
};

/// Configuration files one case writes before running `complete`.
#[derive(Default)]
struct ConfigFiles {
    /// XDG configuration file contents.
    xdg: Option<&'static str>,
    /// Local `.mapsplice.toml` contents.
    local: Option<&'static str>,
    /// Home dotfile contents, with an empty XDG config directory.
    home: Option<&'static str>,
}

/// One completion configuration scenario.
struct SuffixCase {
    /// Environment variables set on the child.
    env: &'static [(&'static str, &'static str)],
    /// Configuration files written before the run.
    files: ConfigFiles,
    /// Extra command-line flags.
    flags: &'static [&'static str],
    /// Expected stdout line.
    expected: &'static str,
}

impl SuffixCase {
    /// Build one scenario from its literal parts.
    const fn new(
        env: &'static [(&'static str, &'static str)],
        files: ConfigFiles,
        flags: &'static [&'static str],
        expected: &'static str,
    ) -> Self {
        Self {
            env,
            files,
            flags,
            expected,
        }
    }

    /// Write the case's configuration files and apply its environment.
    fn prepare(&self, workspace: &Workspace, invocation: &mut CliInvocation) -> TestResult<()> {
        for (key, value) in self.env {
            invocation.set_env(key, *value);
        }
        if let Some(contents) = self.files.xdg {
            let xdg_home = workspace.write_xdg_config(contents)?;
            invocation.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
        }
        if let Some(contents) = self.files.local {
            workspace.write_local_config(contents)?;
        }
        if let Some(contents) = self.files.home {
            let home_dir = workspace.write_home_config(contents)?;
            let xdg_home = empty_xdg_home(workspace)?;
            invocation.set_env("HOME", home_dir.as_str());
            invocation.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
        }
        Ok(())
    }
}

/// Write the standard target and run `complete` for anchor `1.1.1`.
fn run_complete(
    workspace: &Workspace,
    invocation: &CliInvocation,
    flags: &[&str],
) -> TestResult<CliOutcome> {
    workspace.write_target(TARGET_TWO_TASKS)?;
    let mut arguments = vec!["complete"];
    arguments.extend_from_slice(flags);
    arguments.extend([workspace.target.as_str(), "1.1.1"]);
    invocation.run(&arguments)
}

/// Run `complete` for anchor `1.1.1` and return its preview output.
fn complete_preview(
    workspace: &Workspace,
    invocation: &CliInvocation,
    flags: &[&str],
) -> TestResult<String> {
    run_complete(workspace, invocation, flags)?.into_stdout()
}

#[rstest]
#[case::environment_default(SuffixCase::new(
    &[("MAPSPLICE_CMDS_COMPLETE_SUFFIX", "— env")],
    ConfigFiles::default(),
    &[],
    "- [x] 1.1.1. First task — env."
))]
#[case::config_file_default(SuffixCase::new(
    &[],
    ConfigFiles {
        xdg: Some("[cmds.complete]\nsuffix = \"— file\"\n"),
        ..ConfigFiles::default()
    },
    &[],
    "- [x] 1.1.1. First task — file."
))]
#[case::home_dotfile_default(SuffixCase::new(
    &[],
    ConfigFiles {
        home: Some("[cmds.complete]\nsuffix = \"— home\"\n"),
        ..ConfigFiles::default()
    },
    &[],
    "- [x] 1.1.1. First task — home."
))]
#[case::local_config_overrides_xdg(SuffixCase::new(
    &[],
    ConfigFiles {
        xdg: Some("[cmds.complete]\nsuffix = \"— xdg\"\n"),
        local: Some("[cmds.complete]\nsuffix = \"— local\"\n"),
        home: None,
    },
    &[],
    "- [x] 1.1.1. First task — local."
))]
#[case::environment_overrides_local(SuffixCase::new(
    &[("MAPSPLICE_CMDS_COMPLETE_SUFFIX", "— env")],
    ConfigFiles {
        local: Some("[cmds.complete]\nsuffix = \"— local\"\n"),
        ..ConfigFiles::default()
    },
    &[],
    "- [x] 1.1.1. First task — env."
))]
#[case::cli_flag_overrides_environment(SuffixCase::new(
    &[("MAPSPLICE_CMDS_COMPLETE_SUFFIX", "— env")],
    ConfigFiles::default(),
    &["--suffix", "— cli"],
    "- [x] 1.1.1. First task — cli."
))]
fn complete_suffix_precedence(
    #[case] case: SuffixCase,
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    let mut invocation = test_workspace.invocation()?;
    case.prepare(&test_workspace, &mut invocation)?;

    let stdout = complete_preview(&test_workspace, &invocation, case.flags)?;

    assert_contains(&stdout, case.expected);
    Ok(())
}

/// An explicitly empty environment value selects no suffix, overriding the file.
///
/// This records the observed `ortho-config` merge: environment values are
/// parsed as present options even when empty, so the CLI adapter's blank-input
/// rule turns the empty option into "no suffix" rather than falling back.
#[rstest]
fn complete_suffix_empty_environment_overrides_file(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_local_config("[cmds.complete]\nsuffix = \"— file\"\n")?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("MAPSPLICE_CMDS_COMPLETE_SUFFIX", "");

    let stdout = complete_preview(&test_workspace, &invocation, &[])?;

    assert_contains(&stdout, "- [x] 1.1.1. First task.");
    if stdout.contains("— file") {
        return Err(
            format!("an empty environment suffix must suppress the file suffix: {stdout}").into(),
        );
    }
    Ok(())
}

#[rstest]
fn invalid_complete_config_surfaces_configuration_error(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace.write_xdg_config("[cmds.complete\nsuffix = \"— broken\"\n")?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
    // The child reports `MapspliceError::class()` as a tracing field, which is
    // how a process boundary exposes the typed `Configuration` variant.
    invocation.set_env("RUST_LOG", "error");

    let outcome = run_complete(&test_workspace, &invocation, &[])?;

    assert_equal(&outcome.success, &false);
    assert_contains(&outcome.stderr, "mapsplice command failed");
    assert_contains(&outcome.stderr, "\"configuration\"");
    // The malformed table header fails before the suffix value is read, so the
    // diagnostic names the unparsable file rather than the value inside it.
    assert_contains(&outcome.stderr, "config.toml");
    Ok(())
}

#[rstest]
fn multiline_complete_suffix_surfaces_configuration_error(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("MAPSPLICE_CMDS_COMPLETE_SUFFIX", "done\nagain");
    invocation.set_env("RUST_LOG", "error");

    let outcome = run_complete(&test_workspace, &invocation, &[])?;

    assert_equal(&outcome.success, &false);
    assert_contains(&outcome.stderr, "mapsplice command failed");
    assert_contains(&outcome.stderr, "\"configuration\"");
    Ok(())
}
