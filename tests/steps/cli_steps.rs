//! Step definitions backing the behavioural CLI scenarios.

use std::process::Command;

use camino::Utf8PathBuf;
use rstest::fixture;
use rstest_bdd_macros::{given, then, when};

use crate::support::{
    PHASE_FRAGMENT,
    REPLACEMENT_FRAGMENT,
    TARGET_THREE_PHASES,
    TARGET_TWO_PHASES,
    TARGET_TWO_TASKS,
    TASK_FRAGMENT,
    TestResult,
    Workspace,
    create_workspace,
};

pub(crate) type CliFixture = TestResult<CliState>;

#[derive(Debug)]
pub(crate) struct CliState {
    workspace: Workspace,
    binary: Utf8PathBuf,
    stdout: String,
    stderr: String,
    success: bool,
}

impl CliState {
    fn write_target(&self, contents: &str) -> TestResult { self.workspace.write_target(contents) }

    fn write_fragment(&self, contents: &str) -> TestResult {
        self.workspace.write_fragment(contents)
    }

    fn read_target(&self) -> TestResult<String> { self.workspace.read_target() }

    const fn target_path(&self) -> &Utf8PathBuf { &self.workspace.target }

    const fn fragment_path(&self) -> &Utf8PathBuf { &self.workspace.fragment }

    fn run<I, S>(&mut self, args: I) -> TestResult
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let output = Command::new(self.binary.as_str())
            .args(args.into_iter().map(|arg| arg.as_ref().to_owned()))
            .output()?;
        self.stdout = String::from_utf8(output.stdout)?;
        self.stderr = String::from_utf8(output.stderr)?;
        self.success = output.status.success();
        Ok(())
    }
}

#[fixture]
pub(crate) fn cli_state() -> CliFixture {
    let workspace = create_workspace()?;
    let binary = Utf8PathBuf::from(env!("CARGO_BIN_EXE_mapsplice"));
    Ok(CliState {
        workspace,
        binary,
        stdout: String::new(),
        stderr: String::new(),
        success: false,
    })
}

fn state_mut(cli_state: &mut CliFixture) -> TestResult<&mut CliState> {
    match cli_state {
        Ok(state) => Ok(state),
        Err(error) => Err(error.to_string().into()),
    }
}

#[given("the target roadmap with two phases")]
fn target_two_phases(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_target(TARGET_TWO_PHASES)
}

#[given("the target roadmap with one step and two tasks")]
fn target_two_tasks(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_target(TARGET_TWO_TASKS)
}

#[given("the target roadmap with three phases")]
fn target_three_phases(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_target(TARGET_THREE_PHASES)
}

#[given("the phase fragment roadmap")]
fn phase_fragment(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_fragment(PHASE_FRAGMENT)
}

#[given("the task fragment roadmap")]
fn task_fragment(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_fragment(TASK_FRAGMENT)
}

#[given("the replacement fragment roadmap")]
fn replacement_fragment(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_fragment(REPLACEMENT_FRAGMENT)
}

#[when("I append the phase fragment")]
fn append_phase_fragment(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target_path().clone();
    let fragment = state.fragment_path().clone();
    state.run(["append", target.as_str(), fragment.as_str()])
}

#[when("I insert the phase fragment before phase 2")]
fn insert_before_phase(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target_path().clone();
    let fragment = state.fragment_path().clone();
    state.run(["insert", target.as_str(), "2", fragment.as_str()])
}

#[when("I insert the task fragment after task 1.1.1")]
fn insert_after_task(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target_path().clone();
    let fragment = state.fragment_path().clone();
    state.run([
        "insert",
        "--after",
        target.as_str(),
        "1.1.1",
        fragment.as_str(),
    ])
}

#[when("I delete phase 2")]
fn delete_phase_two(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target_path().clone();
    state.run(["delete", target.as_str(), "2"])
}

#[when("I replace phase 2 with the replacement fragment")]
fn replace_phase_two(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target_path().clone();
    let fragment = state.fragment_path().clone();
    state.run(["replace", target.as_str(), "2", fragment.as_str()])
}

#[when("I delete phase 1 in place")]
fn delete_in_place(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target_path().clone();
    state.run(["--in-place", "delete", target.as_str(), "1"])
}

#[when("I try to insert the mismatched fragment before phase 2")]
fn insert_mismatch(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target_path().clone();
    let fragment = state.fragment_path().clone();
    state.run(["insert", target.as_str(), "2", fragment.as_str()])
}

#[when("I try to delete missing phase 99")]
fn delete_missing_phase(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target_path().clone();
    state.run(["delete", target.as_str(), "99"])
}

#[then("the command succeeds")]
fn command_succeeds(cli_state: &mut CliFixture) -> TestResult {
    assert!(state_mut(cli_state)?.success, "expected command to succeed");
    Ok(())
}

#[then("the command fails")]
fn command_fails(cli_state: &mut CliFixture) -> TestResult {
    assert!(!state_mut(cli_state)?.success, "expected command to fail");
    Ok(())
}

#[then("stdout contains the appended phase as phase 3")]
fn stdout_contains_phase_three(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    assert!(state.stdout.contains("## 3. Inserted phase"));
    Ok(())
}

#[then("the target file remains unchanged")]
fn target_unchanged(cli_state: &mut CliFixture) -> TestResult {
    assert_eq!(state_mut(cli_state)?.read_target()?, TARGET_TWO_PHASES);
    Ok(())
}

#[then("stdout renumbers phase two to phase 3 and rewrites its dependency")]
fn stdout_renumbers_phase_and_dependency(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    assert!(state.stdout.contains("## 3. Phase two"));
    assert!(state.stdout.contains("Requires 3.1.1."));
    Ok(())
}

#[then("stdout renumbers the old second task to 1.1.3")]
fn stdout_renumbers_second_task(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    assert!(state.stdout.contains("- [ ] 1.1.3. Second task."));
    Ok(())
}

#[then("stdout removes phase 2 and rewrites the remaining dependency to 2.1.1")]
fn stdout_rewrites_after_delete(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    assert!(state.stdout.contains("## 2. Phase three"));
    assert!(state.stdout.contains("Requires 2.1.1."));
    Ok(())
}

#[then("stdout contains replacement phases 2 and 3")]
fn stdout_contains_replacements(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    assert!(state.stdout.contains("## 2. Replacement phase A"));
    assert!(state.stdout.contains("## 3. Replacement phase B"));
    Ok(())
}

#[then("stdout is empty")]
fn stdout_is_empty(cli_state: &mut CliFixture) -> TestResult {
    assert!(state_mut(cli_state)?.stdout.trim().is_empty());
    Ok(())
}

#[then("the target file now starts with phase 1 titled Phase two")]
fn target_rewritten_in_place(cli_state: &mut CliFixture) -> TestResult {
    let rewritten = state_mut(cli_state)?.read_target()?;
    assert!(rewritten.contains("## 1. Phase two"));
    Ok(())
}

#[then("stderr mentions the phase versus task mismatch")]
fn stderr_mentions_mismatch(cli_state: &mut CliFixture) -> TestResult {
    let stderr = &state_mut(cli_state)?.stderr;
    assert!(stderr.contains("cannot use task content with phase anchor `2`"));
    Ok(())
}

#[then("stderr mentions that anchor 99 was not found")]
fn stderr_mentions_missing_anchor(cli_state: &mut CliFixture) -> TestResult {
    let stderr = &state_mut(cli_state)?.stderr;
    assert!(stderr.contains("anchor `99` was not found in the target roadmap"));
    Ok(())
}
