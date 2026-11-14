//! In-place configuration defaults and precedence tests.

use rstest::rstest;

use super::{
    TestResult,
    TestSources,
    Workspace,
    assert_contains,
    assert_equal,
    run_from_args_with_sources,
    workspace,
};
use crate::support::TARGET_TWO_PHASES;

fn assert_delete_default(
    test_workspace: &Workspace,
    sources: &TestSources,
    in_place: bool,
) -> TestResult {
    test_workspace.write_target(TARGET_TWO_PHASES)?;
    let outcome = run_from_args_with_sources(
        ["mapsplice", "delete", test_workspace.target.as_str(), "1"],
        &sources.sources(),
    )?;
    let contents = test_workspace.read_target()?;

    if in_place {
        assert_equal(&outcome.stdout, &None);
        assert_contains(&contents, "## 1. Phase two");
    } else {
        let stdout = outcome.stdout.unwrap_or_default();
        assert_contains(&stdout, "## 1. Phase two");
        assert_contains(&contents, "## 1. Phase one");
    }
    Ok(())
}

#[rstest]
fn in_place_defaults_to_standard_output(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let sources = TestSources::new(&test_workspace)?;

    assert_delete_default(&test_workspace, &sources, false)
}

#[rstest]
fn in_place_can_default_from_environment(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let mut sources = TestSources::new(&test_workspace)?;
    sources.set("MAPSPLICE_IN_PLACE", "true");

    assert_delete_default(&test_workspace, &sources, true)
}

#[rstest]
fn in_place_can_default_from_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace
        .write_xdg_config("in_place = true\n")
        .expect("config should be written");
    let mut sources = TestSources::new(&test_workspace)?;
    sources.set("XDG_CONFIG_HOME", xdg_home.as_str());

    assert_delete_default(&test_workspace, &sources, true)
}

#[rstest]
fn in_place_can_default_from_local_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_local_config("in_place = true\n")
        .expect("local config should be written");
    let sources = TestSources::new(&test_workspace)?;

    assert_delete_default(&test_workspace, &sources, true)
}

#[rstest]
fn in_place_local_config_overrides_xdg_config(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace
        .write_xdg_config("in_place = false\n")
        .expect("xdg config should be written");
    test_workspace
        .write_local_config("in_place = true\n")
        .expect("local config should be written");
    let mut sources = TestSources::new(&test_workspace)?;
    sources.set("XDG_CONFIG_HOME", xdg_home.as_str());

    assert_delete_default(&test_workspace, &sources, true)
}

#[rstest]
fn in_place_env_false_overrides_local_config_true(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_local_config("in_place = true\n")
        .expect("local config should be written");
    let mut sources = TestSources::new(&test_workspace)?;
    sources.set("MAPSPLICE_IN_PLACE", "false");

    assert_delete_default(&test_workspace, &sources, false)
}

#[rstest]
fn in_place_cli_flag_overrides_env_false(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(TARGET_TWO_PHASES)?;
    let mut sources = TestSources::new(&test_workspace)?;
    sources.set("MAPSPLICE_IN_PLACE", "false");

    let outcome = run_from_args_with_sources(
        [
            "mapsplice",
            "--in-place",
            "delete",
            test_workspace.target.as_str(),
            "1",
        ],
        &sources.sources(),
    )?;
    let contents = test_workspace.read_target()?;

    assert_equal(&outcome.stdout, &None);
    assert_contains(&contents, "## 1. Phase two");
    Ok(())
}
