//! Completion suffix configuration defaults and precedence tests.

use rstest::rstest;

use super::{
    ProcessState,
    TestResult,
    Workspace,
    assert_configuration_error,
    assert_contains,
    empty_xdg_home,
    run_from_args,
    workspace,
};
use crate::support::TARGET_TWO_TASKS;

/// Run `complete` for anchor `1.1.1` and return its preview output.
fn complete_preview(workspace: &Workspace) -> String {
    let outcome = run_from_args(["mapsplice", "complete", workspace.target.as_str(), "1.1.1"])
        .expect("complete command should succeed");
    outcome.stdout.unwrap_or_default()
}

#[rstest]
#[serial_test::serial(cli_env)]
fn complete_suffix_can_default_from_environment(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let mut state = ProcessState::default();
    state.set_env("MAPSPLICE_CMDS_COMPLETE_SUFFIX", "— env");
    state.run(|| {
        test_workspace.write_target(TARGET_TWO_TASKS)?;
        let stdout = complete_preview(&test_workspace);
        assert_contains(&stdout, "- [x] 1.1.1. First task — env.");
        Ok(())
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn complete_suffix_can_default_from_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace.write_xdg_config("[cmds.complete]\nsuffix = \"— file\"\n")?;
    let mut state = ProcessState::default();
    state.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
    state.remove_env("MAPSPLICE_CMDS_COMPLETE_SUFFIX");
    state.run(|| {
        test_workspace.write_target(TARGET_TWO_TASKS)?;
        let stdout = complete_preview(&test_workspace);
        assert_contains(&stdout, "- [x] 1.1.1. First task — file.");
        Ok(())
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn complete_suffix_home_dotfile_default(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let home_dir = test_workspace.write_home_config("[cmds.complete]\nsuffix = \"— home\"\n")?;
    let xdg_home = empty_xdg_home(&test_workspace)?;
    let mut state = ProcessState::default();
    state.set_env("HOME", home_dir.as_str());
    state.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
    state.remove_env("MAPSPLICE_CMDS_COMPLETE_SUFFIX");
    test_workspace.enter_root(&mut state)?;
    state.run(|| {
        test_workspace.write_target(TARGET_TWO_TASKS)?;
        let stdout = complete_preview(&test_workspace);
        assert_contains(&stdout, "- [x] 1.1.1. First task — home.");
        Ok(())
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn complete_suffix_local_config_overrides_xdg_config(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace.write_xdg_config("[cmds.complete]\nsuffix = \"— xdg\"\n")?;
    test_workspace.write_local_config("[cmds.complete]\nsuffix = \"— local\"\n")?;
    let mut state = ProcessState::default();
    state.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
    state.remove_env("MAPSPLICE_CMDS_COMPLETE_SUFFIX");
    test_workspace.enter_root(&mut state)?;
    state.run(|| {
        test_workspace.write_target(TARGET_TWO_TASKS)?;
        let stdout = complete_preview(&test_workspace);
        assert_contains(&stdout, "- [x] 1.1.1. First task — local.");
        Ok(())
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn complete_suffix_environment_overrides_local_config(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_local_config("[cmds.complete]\nsuffix = \"— local\"\n")?;
    let mut state = ProcessState::default();
    state.set_env("MAPSPLICE_CMDS_COMPLETE_SUFFIX", "— env");
    test_workspace.enter_root(&mut state)?;
    state.run(|| {
        test_workspace.write_target(TARGET_TWO_TASKS)?;
        let stdout = complete_preview(&test_workspace);
        assert_contains(&stdout, "- [x] 1.1.1. First task — env.");
        Ok(())
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn complete_suffix_cli_flag_overrides_environment(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let mut state = ProcessState::default();
    state.set_env("MAPSPLICE_CMDS_COMPLETE_SUFFIX", "— env");
    state.run(|| {
        test_workspace.write_target(TARGET_TWO_TASKS)?;
        let outcome = run_from_args([
            "mapsplice",
            "complete",
            "--suffix",
            "— cli",
            test_workspace.target.as_str(),
            "1.1.1",
        ])
        .expect("complete command should prefer the cli suffix");
        let stdout = outcome.stdout.unwrap_or_default();
        assert_contains(&stdout, "- [x] 1.1.1. First task — cli.");
        Ok(())
    })
}

/// An explicitly empty environment value selects no suffix, overriding the file.
///
/// This records the observed `ortho-config` merge: environment values are
/// parsed as present options even when empty, so the CLI adapter's blank-input
/// rule turns the empty option into "no suffix" rather than falling back.
#[rstest]
#[serial_test::serial(cli_env)]
fn complete_suffix_empty_environment_overrides_file(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_local_config("[cmds.complete]\nsuffix = \"— file\"\n")?;
    let mut state = ProcessState::default();
    state.set_env("MAPSPLICE_CMDS_COMPLETE_SUFFIX", "");
    test_workspace.enter_root(&mut state)?;
    state.run(|| {
        test_workspace.write_target(TARGET_TWO_TASKS)?;
        let stdout = complete_preview(&test_workspace);
        assert_contains(&stdout, "- [x] 1.1.1. First task.");
        if stdout.contains("— file") {
            return Err(format!(
                "an empty environment suffix must suppress the file suffix: {stdout}"
            )
            .into());
        }
        Ok(())
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn invalid_complete_config_surfaces_configuration_error(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace.write_xdg_config("[cmds.complete\nsuffix = \"— broken\"\n")?;
    let mut state = ProcessState::default();
    state.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
    state.run(|| {
        test_workspace.write_target(TARGET_TWO_TASKS)?;
        let error = run_from_args([
            "mapsplice",
            "complete",
            test_workspace.target.as_str(),
            "1.1.1",
        ])
        .expect_err("invalid config should fail");
        assert_configuration_error(&error);
        Ok(())
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn multiline_complete_suffix_surfaces_configuration_error(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    let mut state = ProcessState::default();
    state.set_env("MAPSPLICE_CMDS_COMPLETE_SUFFIX", "done\nagain");
    state.run(|| {
        test_workspace.write_target(TARGET_TWO_TASKS)?;
        let error = run_from_args([
            "mapsplice",
            "complete",
            test_workspace.target.as_str(),
            "1.1.1",
        ])
        .expect_err("a multiline suffix should fail");
        assert_configuration_error(&error);
        Ok(())
    })
}
