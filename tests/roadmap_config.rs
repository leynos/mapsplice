//! `rstest` coverage for CLI environment and file configuration defaults.
//!
//! Each case runs the compiled binary as a child process, so environment
//! variables and the working directory belong to the child alone.

#[path = "support/assertions.rs"]
mod assertions;
#[path = "support/config.rs"]
mod support;

use std::fmt::Debug;

use assertions::assert_contains;
use camino::Utf8PathBuf;
use rstest::rstest;
use support::{
    CliInvocation,
    CliOutcome,
    TARGET_TWO_TASKS,
    TASK_FRAGMENT,
    TestResult,
    Workspace,
    workspace,
};

fn assert_equal<T>(actual: &T, expected: &T)
where
    T: Debug + PartialEq,
{
    assert_eq!(actual, expected, "configuration value differs");
}

fn workspace_root(workspace: &Workspace) -> TestResult<Utf8PathBuf> {
    workspace
        .target
        .parent()
        .map(Utf8PathBuf::from)
        .ok_or_else(|| "target path should have a parent".into())
}

fn empty_xdg_home(workspace: &Workspace) -> TestResult<Utf8PathBuf> {
    workspace.dir.create_dir_all("empty-xdg")?;
    Ok(workspace_root(workspace)?.join("empty-xdg"))
}

/// Write the standard fixtures and run `insert` with the given extra flags.
fn run_insert(
    workspace: &Workspace,
    invocation: &CliInvocation,
    flags: &[&str],
) -> TestResult<CliOutcome> {
    workspace.write_target(TARGET_TWO_TASKS)?;
    workspace.write_fragment(TASK_FRAGMENT)?;
    let mut arguments = vec!["insert"];
    arguments.extend_from_slice(flags);
    arguments.extend([
        workspace.target.as_str(),
        "1.1.1",
        workspace.fragment.as_str(),
    ]);
    invocation.run(&arguments)
}

#[rstest]
fn insert_after_can_default_from_environment(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("MAPSPLICE_CMDS_INSERT_AFTER", "true");

    let stdout = run_insert(&test_workspace, &invocation, &[])?.into_stdout()?;

    assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
    Ok(())
}

#[rstest]
fn insert_after_can_default_from_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace.write_xdg_config("[cmds.insert]\nafter = true\n")?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("XDG_CONFIG_HOME", xdg_home.as_str());

    let stdout = run_insert(&test_workspace, &invocation, &[])?.into_stdout()?;

    assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
    Ok(())
}

#[rstest]
fn insert_after_home_dotfile_default(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let home_dir = test_workspace.write_home_config("[cmds.insert]\nafter = true\n")?;
    let xdg_home = empty_xdg_home(&test_workspace)?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("HOME", home_dir.as_str());
    invocation.set_env("XDG_CONFIG_HOME", xdg_home.as_str());

    let stdout = run_insert(&test_workspace, &invocation, &[])?.into_stdout()?;

    assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
    Ok(())
}

#[rstest]
fn insert_after_local_config_overrides_xdg_config(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace.write_xdg_config("[cmds.insert]\nafter = false\n")?;
    test_workspace.write_local_config("[cmds.insert]\nafter = true\n")?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("XDG_CONFIG_HOME", xdg_home.as_str());

    let stdout = run_insert(&test_workspace, &invocation, &[])?.into_stdout()?;

    assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
    Ok(())
}

#[rstest]
fn insert_after_xdg_config_overrides_home_dotfile(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let home_dir = test_workspace.write_home_config("[cmds.insert]\nafter = false\n")?;
    let xdg_home = test_workspace.write_xdg_config("[cmds.insert]\nafter = true\n")?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("HOME", home_dir.as_str());
    invocation.set_env("XDG_CONFIG_HOME", xdg_home.as_str());

    let stdout = run_insert(&test_workspace, &invocation, &[])?.into_stdout()?;

    assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
    Ok(())
}

#[rstest]
fn insert_after_env_false_overrides_local_config_true(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_local_config("[cmds.insert]\nafter = true\n")?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("MAPSPLICE_CMDS_INSERT_AFTER", "false");

    let stdout = run_insert(&test_workspace, &invocation, &[])?.into_stdout()?;

    assert_contains(&stdout, "- [ ] 1.1.1. Inserted task. Requires 1.1.1.");
    Ok(())
}

#[rstest]
fn insert_after_cli_flag_overrides_env_false(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("MAPSPLICE_CMDS_INSERT_AFTER", "false");

    let stdout = run_insert(&test_workspace, &invocation, &["--after"])?.into_stdout()?;

    assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
    Ok(())
}

#[path = "roadmap_config/in_place.rs"]
mod in_place;

#[rstest]
fn invalid_insert_config_surfaces_configuration_error(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace.write_xdg_config("[cmds.insert]\nafter = \"later\"\n")?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
    // The child reports `MapspliceError::class()` as a tracing field, which is
    // how a process boundary exposes the typed `Configuration` variant.
    invocation.set_env("RUST_LOG", "error");

    let outcome = run_insert(&test_workspace, &invocation, &[])?;

    assert_equal(&outcome.success, &false);
    assert_contains(&outcome.stderr, "mapsplice command failed");
    assert_contains(&outcome.stderr, "\"configuration\"");
    assert_contains(&outcome.stderr, "later");
    Ok(())
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
fn complete_suffix_can_default_from_environment(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("MAPSPLICE_CMDS_COMPLETE_SUFFIX", "— env");

    let stdout = complete_preview(&test_workspace, &invocation, &[])?;

    assert_contains(&stdout, "- [x] 1.1.1. First task — env.");
    Ok(())
}

#[rstest]
fn complete_suffix_can_default_from_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace.write_xdg_config("[cmds.complete]\nsuffix = \"— file\"\n")?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("XDG_CONFIG_HOME", xdg_home.as_str());

    let stdout = complete_preview(&test_workspace, &invocation, &[])?;

    assert_contains(&stdout, "- [x] 1.1.1. First task — file.");
    Ok(())
}

#[rstest]
fn complete_suffix_home_dotfile_default(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let home_dir = test_workspace.write_home_config("[cmds.complete]\nsuffix = \"— home\"\n")?;
    let xdg_home = empty_xdg_home(&test_workspace)?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("HOME", home_dir.as_str());
    invocation.set_env("XDG_CONFIG_HOME", xdg_home.as_str());

    let stdout = complete_preview(&test_workspace, &invocation, &[])?;

    assert_contains(&stdout, "- [x] 1.1.1. First task — home.");
    Ok(())
}

#[rstest]
fn complete_suffix_local_config_overrides_xdg_config(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace.write_xdg_config("[cmds.complete]\nsuffix = \"— xdg\"\n")?;
    test_workspace.write_local_config("[cmds.complete]\nsuffix = \"— local\"\n")?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("XDG_CONFIG_HOME", xdg_home.as_str());

    let stdout = complete_preview(&test_workspace, &invocation, &[])?;

    assert_contains(&stdout, "- [x] 1.1.1. First task — local.");
    Ok(())
}

#[rstest]
fn complete_suffix_environment_overrides_local_config(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_local_config("[cmds.complete]\nsuffix = \"— local\"\n")?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("MAPSPLICE_CMDS_COMPLETE_SUFFIX", "— env");

    let stdout = complete_preview(&test_workspace, &invocation, &[])?;

    assert_contains(&stdout, "- [x] 1.1.1. First task — env.");
    Ok(())
}

#[rstest]
fn complete_suffix_cli_flag_overrides_environment(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("MAPSPLICE_CMDS_COMPLETE_SUFFIX", "— env");

    let stdout = complete_preview(&test_workspace, &invocation, &["--suffix", "— cli"])?;

    assert_contains(&stdout, "- [x] 1.1.1. First task — cli.");
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
        return Err(format!(
            "an empty environment suffix must suppress the file suffix: {stdout}"
        )
        .into());
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
    assert_contains(&outcome.stderr, "broken");
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
