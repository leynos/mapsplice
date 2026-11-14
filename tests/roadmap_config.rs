//! `rstest` coverage for CLI environment and file configuration defaults.
//!
//! Every test supplies its own `MapEnv` and local file base through
//! `ConfigSources`, so the suite reads no process-global state and needs no
//! serialization lock.

#[path = "support/assertions.rs"]
mod assertions;
#[path = "support/config.rs"]
mod support;

use std::fmt::Debug;

use assertions::assert_contains;
use mapsplice::{MapspliceError, RunOutcome, run_from_args_with_sources};
use rstest::rstest;
use support::{TARGET_TWO_TASKS, TASK_FRAGMENT, TestResult, TestSources, Workspace, workspace};

fn assert_equal<T>(actual: &T, expected: &T)
where
    T: Debug + PartialEq,
{
    assert_eq!(actual, expected, "configuration value differs");
}

fn assert_configuration_error(error: &MapspliceError) {
    assert!(
        matches!(error, MapspliceError::Configuration { .. }),
        "expected configuration error"
    );
}

fn write_insert_inputs(test_workspace: &Workspace) -> TestResult {
    test_workspace.write_target(TARGET_TWO_TASKS)?;
    test_workspace.write_fragment(TASK_FRAGMENT)?;
    Ok(())
}

fn run_insert(test_workspace: &Workspace, sources: &TestSources) -> mapsplice::Result<RunOutcome> {
    run_from_args_with_sources(
        [
            "mapsplice",
            "insert",
            test_workspace.target.as_str(),
            "1.1.1",
            test_workspace.fragment.as_str(),
        ],
        &sources.sources(),
    )
}

#[rstest]
fn insert_after_defaults_to_insert_before(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let sources = TestSources::new(&test_workspace)?;
    write_insert_inputs(&test_workspace)?;

    let outcome = run_insert(&test_workspace, &sources)?;
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "- [ ] 1.1.1. Inserted task. Requires 1.1.1.");
    Ok(())
}

#[rstest]
fn insert_after_can_default_from_environment(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let mut sources = TestSources::new(&test_workspace)?;
    sources.set("MAPSPLICE_CMDS_INSERT_AFTER", "true");
    write_insert_inputs(&test_workspace)?;

    let outcome = run_insert(&test_workspace, &sources)?;
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
    Ok(())
}

#[rstest]
fn insert_after_can_default_from_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace
        .write_xdg_config("[cmds.insert]\nafter = true\n")
        .expect("config should be written");
    let mut sources = TestSources::new(&test_workspace)?;
    sources.set("XDG_CONFIG_HOME", xdg_home.as_str());
    write_insert_inputs(&test_workspace)?;

    let outcome = run_insert(&test_workspace, &sources)?;
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
    Ok(())
}

#[rstest]
fn insert_after_home_dotfile_default(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let home_dir = test_workspace
        .write_home_config("[cmds.insert]\nafter = true\n")
        .expect("home config should be written");
    let mut sources = TestSources::new(&test_workspace)?;
    sources.set("HOME", home_dir.as_str());
    write_insert_inputs(&test_workspace)?;

    let outcome = run_insert(&test_workspace, &sources)?;
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
    Ok(())
}

#[rstest]
fn insert_after_local_config_overrides_xdg_config(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace
        .write_xdg_config("[cmds.insert]\nafter = false\n")
        .expect("xdg config should be written");
    test_workspace
        .write_local_config("[cmds.insert]\nafter = true\n")
        .expect("local config should be written");
    let mut sources = TestSources::new(&test_workspace)?;
    sources.set("XDG_CONFIG_HOME", xdg_home.as_str());
    write_insert_inputs(&test_workspace)?;

    let outcome = run_insert(&test_workspace, &sources)?;
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
    Ok(())
}

#[rstest]
fn insert_after_xdg_config_overrides_home_dotfile(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let home_dir = test_workspace
        .write_home_config("[cmds.insert]\nafter = false\n")
        .expect("home config should be written");
    let xdg_home = test_workspace
        .write_xdg_config("[cmds.insert]\nafter = true\n")
        .expect("xdg config should be written");
    let mut sources = TestSources::new(&test_workspace)?;
    sources.set("HOME", home_dir.as_str());
    sources.set("XDG_CONFIG_HOME", xdg_home.as_str());
    write_insert_inputs(&test_workspace)?;

    let outcome = run_insert(&test_workspace, &sources)?;
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
    Ok(())
}

#[rstest]
fn insert_after_env_false_overrides_local_config_true(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_local_config("[cmds.insert]\nafter = true\n")
        .expect("local config should be written");
    let mut sources = TestSources::new(&test_workspace)?;
    sources.set("MAPSPLICE_CMDS_INSERT_AFTER", "false");
    write_insert_inputs(&test_workspace)?;

    let outcome = run_insert(&test_workspace, &sources)?;
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "- [ ] 1.1.1. Inserted task. Requires 1.1.1.");
    Ok(())
}

#[rstest]
fn insert_after_cli_flag_overrides_env_false(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let mut sources = TestSources::new(&test_workspace)?;
    sources.set("MAPSPLICE_CMDS_INSERT_AFTER", "false");
    write_insert_inputs(&test_workspace)?;

    let outcome = run_from_args_with_sources(
        [
            "mapsplice",
            "insert",
            "--after",
            test_workspace.target.as_str(),
            "1.1.1",
            test_workspace.fragment.as_str(),
        ],
        &sources.sources(),
    )?;
    let stdout = outcome.stdout.unwrap_or_default();

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
    let xdg_home = test_workspace
        .write_xdg_config("[cmds.insert]\nafter = \"later\"\n")
        .expect("config should be written");
    let mut sources = TestSources::new(&test_workspace)?;
    sources.set("XDG_CONFIG_HOME", xdg_home.as_str());
    write_insert_inputs(&test_workspace)?;

    let error = run_insert(&test_workspace, &sources).expect_err("invalid config should fail");

    assert_configuration_error(&error);
    Ok(())
}

#[rstest]
fn invalid_global_config_surfaces_configuration_error(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_local_config("in_place = \"later\"\n")
        .expect("local config should be written");
    test_workspace.write_target(TARGET_TWO_TASKS)?;
    let sources = TestSources::new(&test_workspace)?;

    let error = run_from_args_with_sources(
        ["mapsplice", "delete", test_workspace.target.as_str(), "1"],
        &sources.sources(),
    )
    .expect_err("invalid global config should fail");

    assert_configuration_error(&error);
    Ok(())
}
