//! `rstest` coverage for CLI environment and file configuration defaults.

#[path = "support/config.rs"]
mod support;

use std::fmt::Debug;

use mapsplice::{MapspliceError, run_from_args};
use rstest::rstest;
use support::{
    EnvVarGuard,
    TARGET_TWO_PHASES,
    TARGET_TWO_TASKS,
    TASK_FRAGMENT,
    TestResult,
    Workspace,
    workspace,
};

fn assert_contains(haystack: &str, needle: &str) {
    assert!(haystack.contains(needle));
}

fn assert_equal<T>(actual: &T, expected: &T)
where
    T: Debug + PartialEq,
{
    assert_eq!(actual, expected);
}

fn assert_configuration_error(error: &MapspliceError) {
    assert!(matches!(error, MapspliceError::Configuration { .. }));
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_after_can_default_from_environment(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let _after = EnvVarGuard::set("MAPSPLICE_CMDS_INSERT_AFTER", "true")
        .expect("environment guard should install");
    test_workspace
        .write_target(TARGET_TWO_TASKS)
        .expect("target should be written");
    test_workspace
        .write_fragment(TASK_FRAGMENT)
        .expect("fragment should be written");

    let outcome = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "1.1.1",
        test_workspace.fragment.as_str(),
    ])
    .expect("insert command should succeed with environment default");

    let stdout = outcome.stdout.unwrap_or_default();
    assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_after_can_default_from_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace
        .write_xdg_config("[cmds.insert]\nafter = true\n")
        .expect("config should be written");
    let _config =
        EnvVarGuard::set("XDG_CONFIG_HOME", xdg_home.as_str()).expect("xdg guard should install");
    test_workspace
        .write_target(TARGET_TWO_TASKS)
        .expect("target should be written");
    test_workspace
        .write_fragment(TASK_FRAGMENT)
        .expect("fragment should be written");

    let outcome = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "1.1.1",
        test_workspace.fragment.as_str(),
    ])
    .expect("insert command should succeed with config default");

    let stdout = outcome.stdout.unwrap_or_default();
    assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_can_default_from_environment(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let _in_place =
        EnvVarGuard::set("MAPSPLICE_IN_PLACE", "true").expect("environment guard should install");
    test_workspace
        .write_target(TARGET_TWO_PHASES)
        .expect("target should be written");

    let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "1"])
        .expect("delete command should succeed with in-place environment default");

    assert_equal(&outcome.stdout, &None);
    assert_contains(&test_workspace.read_target()?, "## 1. Phase two");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_can_default_from_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace
        .write_xdg_config("in_place = true\n")
        .expect("config should be written");
    let _config =
        EnvVarGuard::set("XDG_CONFIG_HOME", xdg_home.as_str()).expect("xdg guard should install");
    test_workspace
        .write_target(TARGET_TWO_PHASES)
        .expect("target should be written");

    let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "1"])
        .expect("delete command should succeed with in-place config default");

    assert_equal(&outcome.stdout, &None);
    assert_contains(&test_workspace.read_target()?, "## 1. Phase two");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_can_default_from_local_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_local_config("in_place = true\n")
        .expect("local config should be written");
    let _cwd = test_workspace
        .enter_root()
        .expect("current directory guard should install");
    test_workspace
        .write_target(TARGET_TWO_PHASES)
        .expect("target should be written");

    let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "1"])
        .expect("delete command should succeed with local in-place config default");

    assert_equal(&outcome.stdout, &None);
    assert_contains(&test_workspace.read_target()?, "## 1. Phase two");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn invalid_insert_config_surfaces_configuration_error(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace
        .write_xdg_config("[cmds.insert]\nafter = \"later\"\n")
        .expect("config should be written");
    let _config =
        EnvVarGuard::set("XDG_CONFIG_HOME", xdg_home.as_str()).expect("xdg guard should install");
    test_workspace
        .write_target(TARGET_TWO_TASKS)
        .expect("target should be written");
    test_workspace
        .write_fragment(TASK_FRAGMENT)
        .expect("fragment should be written");

    let error = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "1.1.1",
        test_workspace.fragment.as_str(),
    ])
    .expect_err("invalid config should fail");

    assert_configuration_error(&error);
    Ok(())
}
