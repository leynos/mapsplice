//! In-place configuration defaults and precedence tests.

use rstest::rstest;

use super::{
    ProcessState,
    TestResult,
    Workspace,
    assert_contains,
    assert_equal,
    run_from_args,
    workspace,
};
use crate::support::TARGET_TWO_PHASES;

#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_can_default_from_environment(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let mut state = ProcessState::default();
    state.set_env("MAPSPLICE_IN_PLACE", "true");
    state.run(|| {
        test_workspace
            .write_target(TARGET_TWO_PHASES)
            .expect("target should be written");

        let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "1"])
            .expect("delete command should succeed with in-place environment default");

        assert_equal(&outcome.stdout, &None);
        assert_contains(&test_workspace.read_target()?, "## 1. Phase two");
        Ok(())
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_can_default_from_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace
        .write_xdg_config("in_place = true\n")
        .expect("config should be written");
    let mut state = ProcessState::default();
    state.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
    state.run(|| {
        test_workspace
            .write_target(TARGET_TWO_PHASES)
            .expect("target should be written");

        let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "1"])
            .expect("delete command should succeed with in-place config default");

        assert_equal(&outcome.stdout, &None);
        assert_contains(&test_workspace.read_target()?, "## 1. Phase two");
        Ok(())
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_can_default_from_local_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_local_config("in_place = true\n")
        .expect("local config should be written");
    let mut state = ProcessState::default();
    test_workspace.enter_root(&mut state)?;
    state.run(|| {
        test_workspace
            .write_target(TARGET_TWO_PHASES)
            .expect("target should be written");

        let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "1"])
            .expect("delete command should succeed with local in-place config default");

        assert_equal(&outcome.stdout, &None);
        assert_contains(&test_workspace.read_target()?, "## 1. Phase two");
        Ok(())
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_local_config_overrides_xdg_config(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace
        .write_xdg_config("in_place = false\n")
        .expect("xdg config should be written");
    test_workspace
        .write_local_config("in_place = true\n")
        .expect("local config should be written");
    let mut state = ProcessState::default();
    state.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
    state.remove_env("MAPSPLICE_IN_PLACE");
    test_workspace.enter_root(&mut state)?;
    state.run(|| {
        test_workspace
            .write_target(TARGET_TWO_PHASES)
            .expect("target should be written");

        let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "1"])
            .expect("delete command should prefer local in-place config");

        assert_equal(&outcome.stdout, &None);
        assert_contains(&test_workspace.read_target()?, "## 1. Phase two");
        Ok(())
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_env_false_overrides_local_config_true(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_local_config("in_place = true\n")
        .expect("local config should be written");
    let mut state = ProcessState::default();
    state.set_env("MAPSPLICE_IN_PLACE", "false");
    test_workspace.enter_root(&mut state)?;
    state.run(|| {
        test_workspace
            .write_target(TARGET_TWO_PHASES)
            .expect("target should be written");

        let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "1"])
            .expect("delete command should prefer environment false");

        let stdout = outcome.stdout.unwrap_or_default();
        assert_contains(&stdout, "## 1. Phase two");
        assert_contains(&test_workspace.read_target()?, "## 1. Phase one");
        Ok(())
    })
}
