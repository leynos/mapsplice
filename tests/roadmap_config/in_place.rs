//! In-place configuration defaults and precedence tests.

use rstest::rstest;

use super::{CliInvocation, TestResult, Workspace, assert_contains, assert_equal, workspace};
use crate::support::TARGET_TWO_PHASES;

fn assert_delete_default(
    test_workspace: &Workspace,
    invocation: &CliInvocation,
    in_place: bool,
) -> TestResult {
    test_workspace.write_target(TARGET_TWO_PHASES)?;
    let stdout = invocation
        .run(&["delete", test_workspace.target.as_str(), "1"])?
        .into_stdout()?;
    let contents = test_workspace.read_target()?;

    if in_place {
        assert_equal(&stdout, &String::new());
        assert_contains(&contents, "## 1. Phase two");
    } else {
        assert_contains(&stdout, "## 1. Phase two");
        assert_contains(&contents, "## 1. Phase one");
    }
    Ok(())
}

#[rstest]
fn in_place_can_default_from_environment(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("MAPSPLICE_IN_PLACE", "true");
    assert_delete_default(&test_workspace, &invocation, true)
}

#[rstest]
fn in_place_can_default_from_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace.write_xdg_config("in_place = true\n")?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
    assert_delete_default(&test_workspace, &invocation, true)
}

#[rstest]
fn in_place_can_default_from_local_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_local_config("in_place = true\n")?;
    let invocation = test_workspace.invocation()?;
    assert_delete_default(&test_workspace, &invocation, true)
}

#[rstest]
fn in_place_local_config_overrides_xdg_config(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace.write_xdg_config("in_place = false\n")?;
    test_workspace.write_local_config("in_place = true\n")?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
    assert_delete_default(&test_workspace, &invocation, true)
}

#[rstest]
fn in_place_env_false_overrides_local_config_true(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_local_config("in_place = true\n")?;
    let mut invocation = test_workspace.invocation()?;
    invocation.set_env("MAPSPLICE_IN_PLACE", "false");
    assert_delete_default(&test_workspace, &invocation, false)
}
