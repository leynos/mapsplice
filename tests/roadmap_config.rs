//! `rstest` coverage for CLI environment and file configuration defaults.

#[path = "support/config.rs"]
mod support;

use std::{env, fmt::Debug};

use camino::Utf8PathBuf;
use mapsplice::{MapspliceError, run_from_args};
use rstest::rstest;
use support::{ProcessState, TARGET_TWO_TASKS, TASK_FRAGMENT, TestResult, Workspace, workspace};

fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "expected `{needle}` in `{haystack}`"
    );
}

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

#[rstest]
#[serial_test::serial(cli_env)]
fn process_state_restores_multiple_env_vars_and_cwd(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    let first_key = "MAPSPLICE_TEST_PROCESS_STATE_ONE";
    let second_key = "MAPSPLICE_TEST_PROCESS_STATE_TWO";
    let removed_key = "MAPSPLICE_TEST_PROCESS_STATE_REMOVED";
    let original_first = env::var_os(first_key);
    let original_second = env::var_os(second_key);
    let original_removed = env::var_os(removed_key);
    let original_cwd = env::current_dir()?;
    let home_dir = test_workspace.write_home_config("in_place = true\n")?;
    let expected_cwd = test_workspace
        .target
        .parent()
        .ok_or_else(|| "target path should have a parent".to_owned())?
        .to_path_buf();

    let mut state = ProcessState::default();
    state.set_env(first_key, "alpha");
    state.set_env(second_key, "beta");
    state.set_env(removed_key, "temporary");
    state.remove_env(removed_key);
    test_workspace.enter_root(&mut state)?;

    state.run(|| {
        assert_equal(&env::var(first_key)?, &"alpha".to_owned());
        assert_equal(&env::var(second_key)?, &"beta".to_owned());
        assert_equal(&env::var_os(removed_key), &None);
        assert_equal(&env::current_dir()?, &expected_cwd.into_std_path_buf());
        if !home_dir.join(".mapsplice.toml").exists() {
            return Err("home configuration file should exist".into());
        }
        Ok(())
    })?;

    assert_equal(&env::var_os(first_key), &original_first);
    assert_equal(&env::var_os(second_key), &original_second);
    assert_equal(&env::var_os(removed_key), &original_removed);
    assert_equal(&env::current_dir()?, &original_cwd);
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_after_can_default_from_environment(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let mut state = ProcessState::default();
    state.set_env("MAPSPLICE_CMDS_INSERT_AFTER", "true");
    state.run(|| {
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
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_after_can_default_from_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace
        .write_xdg_config("[cmds.insert]\nafter = true\n")
        .expect("config should be written");
    let mut state = ProcessState::default();
    state.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
    state.run(|| {
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
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_after_home_dotfile_default(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let home_dir = test_workspace
        .write_home_config("[cmds.insert]\nafter = true\n")
        .expect("home config should be written");
    let xdg_home = empty_xdg_home(&test_workspace)?;
    let mut state = ProcessState::default();
    state.set_env("HOME", home_dir.as_str());
    state.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
    state.remove_env("MAPSPLICE_CMDS_INSERT_AFTER");
    test_workspace.enter_root(&mut state)?;
    state.run(|| {
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
        .expect("insert command should discover home dotfile default");

        let stdout = outcome.stdout.unwrap_or_default();
        assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
        Ok(())
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_after_local_config_overrides_xdg_config(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace
        .write_xdg_config("[cmds.insert]\nafter = false\n")
        .expect("xdg config should be written");
    test_workspace
        .write_local_config("[cmds.insert]\nafter = true\n")
        .expect("local config should be written");
    let mut state = ProcessState::default();
    state.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
    state.remove_env("MAPSPLICE_CMDS_INSERT_AFTER");
    test_workspace.enter_root(&mut state)?;
    state.run(|| {
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
        .expect("insert command should prefer local config");

        let stdout = outcome.stdout.unwrap_or_default();
        assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
        Ok(())
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_after_xdg_config_overrides_home_dotfile(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let home_dir = test_workspace
        .write_home_config("[cmds.insert]\nafter = false\n")
        .expect("home config should be written");
    let xdg_home = test_workspace
        .write_xdg_config("[cmds.insert]\nafter = true\n")
        .expect("xdg config should be written");
    let mut state = ProcessState::default();
    state.set_env("HOME", home_dir.as_str());
    state.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
    state.remove_env("MAPSPLICE_CMDS_INSERT_AFTER");
    test_workspace.enter_root(&mut state)?;
    state.run(|| {
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
        .expect("insert command should prefer xdg config over home");

        let stdout = outcome.stdout.unwrap_or_default();
        assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
        Ok(())
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_after_env_false_overrides_local_config_true(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_local_config("[cmds.insert]\nafter = true\n")
        .expect("local config should be written");
    let mut state = ProcessState::default();
    state.set_env("MAPSPLICE_CMDS_INSERT_AFTER", "false");
    test_workspace.enter_root(&mut state)?;
    state.run(|| {
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
        .expect("insert command should prefer environment false");

        let stdout = outcome.stdout.unwrap_or_default();
        assert_contains(&stdout, "- [ ] 1.1.1. Inserted task. Requires 1.1.1.");
        Ok(())
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_after_cli_flag_overrides_env_false(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let mut state = ProcessState::default();
    state.set_env("MAPSPLICE_CMDS_INSERT_AFTER", "false");
    test_workspace.enter_root(&mut state)?;
    state.run(|| {
        test_workspace
            .write_target(TARGET_TWO_TASKS)
            .expect("target should be written");
        test_workspace
            .write_fragment(TASK_FRAGMENT)
            .expect("fragment should be written");

        let outcome = run_from_args([
            "mapsplice",
            "insert",
            "--after",
            test_workspace.target.as_str(),
            "1.1.1",
            test_workspace.fragment.as_str(),
        ])
        .expect("insert command should prefer cli flag");

        let stdout = outcome.stdout.unwrap_or_default();
        assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
        Ok(())
    })
}

#[path = "roadmap_config/in_place.rs"]
mod in_place;

#[rstest]
#[serial_test::serial(cli_env)]
fn invalid_insert_config_surfaces_configuration_error(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace
        .write_xdg_config("[cmds.insert]\nafter = \"later\"\n")
        .expect("config should be written");
    let mut state = ProcessState::default();
    state.set_env("XDG_CONFIG_HOME", xdg_home.as_str());
    state.run(|| {
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
    })
}
