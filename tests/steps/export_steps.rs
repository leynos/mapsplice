//! Behavioural steps for JSON export and drift checking.

use rstest_bdd_macros::{given, then, when};

use super::cli_steps::{CliFixture, state_mut};
use crate::support::{TestResult, Workspace};

const EXPORT_ROADMAP: &str = concat!(
    "## 1. Phase\n\n",
    "### 1.1. Step\n\n",
    "- [ ] 1.1.1. First task. Requires 1.1.2.\n",
    "- [ ] 1.1.2. Second task.\n",
);

/// Path of the committed JSON export beside the workspace roadmap.
fn export_path(workspace: &Workspace) -> camino::Utf8PathBuf {
    workspace.target.with_file_name("roadmap.json")
}

#[given("the target roadmap for export")]
fn target_roadmap_for_export(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_target(EXPORT_ROADMAP)
}

#[given("a committed JSON export for the target")]
fn committed_json_export(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.workspace.target.clone();
    state.run(["export", target.as_str()])?;
    state.original_export.clone_from(&state.stdout);
    state
        .workspace
        .dir
        .write("roadmap.json", &state.original_export)?;
    Ok(())
}

#[when("I export the roadmap as JSON")]
fn export_roadmap_json(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.workspace.target.clone();
    state.run(["export", "--format", "json", target.as_str()])
}

#[when("I check the committed JSON export")]
fn check_committed_json_export(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.workspace.target.clone();
    let export = export_path(&state.workspace);
    state.run(["export", "--check", export.as_str(), target.as_str()])
}

#[when("I change a task title and check the export")]
fn change_task_title_and_check_export(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let current = state.workspace.dir.read_to_string("target.md")?;
    let changed = current.replacen("First task.", "Renamed task.", 1);
    state.write_target(&changed)?;
    let target = state.workspace.target.clone();
    let export = export_path(&state.workspace);
    state.run(["export", "--check", export.as_str(), target.as_str()])
}

#[when("I change a dependency and check the export")]
fn change_dependency_and_check_export(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let current = state.workspace.dir.read_to_string("target.md")?;
    let changed = current.replacen("Requires 1.1.2.", "Requires 1.1.1.", 1);
    state.write_target(&changed)?;
    let target = state.workspace.target.clone();
    let export = export_path(&state.workspace);
    state.run(["export", "--check", export.as_str(), target.as_str()])
}

#[when("I change a checkbox and check the export")]
fn change_checkbox_and_check_export(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let current = state.workspace.dir.read_to_string("target.md")?;
    let changed = current.replacen("- [ ] 1.1.2.", "- [x] 1.1.2.", 1);
    state.write_target(&changed)?;
    let target = state.workspace.target.clone();
    let export = export_path(&state.workspace);
    state.run(["export", "--check", export.as_str(), target.as_str()])
}

#[then("stdout contains the exported format version")]
fn stdout_contains_export_version(cli_state: &mut CliFixture) -> TestResult {
    let stdout = &state_mut(cli_state)?.stdout;
    assert!(stdout.contains("\"format_version\": 1"));
    Ok(())
}

#[then("stderr names the changed task")]
fn stderr_names_changed_task(cli_state: &mut CliFixture) -> TestResult {
    let stderr = &state_mut(cli_state)?.stderr;
    assert!(stderr.contains("task 1.1.1 title"), "{stderr}");
    Ok(())
}

#[then("stderr names the changed dependency")]
fn stderr_names_changed_dependency(cli_state: &mut CliFixture) -> TestResult {
    let stderr = &state_mut(cli_state)?.stderr;
    assert!(stderr.contains("task 1.1.1 requires"), "{stderr}");
    Ok(())
}

#[then("stderr names the changed checkbox state")]
fn stderr_names_changed_checkbox(cli_state: &mut CliFixture) -> TestResult {
    let stderr = &state_mut(cli_state)?.stderr;
    assert!(stderr.contains("task 1.1.2 state"), "{stderr}");
    Ok(())
}

#[then("the roadmap and export files remain unchanged by the command")]
fn export_inputs_remain_unchanged(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    assert_eq!(
        state.workspace.dir.read_to_string("target.md")?,
        state.original_target
    );
    assert_eq!(
        state.workspace.dir.read_to_string("roadmap.json")?,
        state.original_export
    );
    Ok(())
}
