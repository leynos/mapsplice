//! Step definitions for completing roadmap items in place.
//!
//! These steps live apart from the general CLI steps so both modules stay
//! within the project's file-size budget.

use rstest_bdd_macros::{given, then, when};

use super::cli_steps::{CliFixture, state_mut};
use crate::support::{TARGET_COMPLETED_TASK, TestResult};

#[given("the target roadmap with a completed task")]
fn target_completed_task(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_target(TARGET_COMPLETED_TASK)
}

#[when("I complete task 1.1.1")]
fn complete_task_preview(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target_path().clone();
    state.run(["complete", target.as_str(), "1.1.1"])
}

#[when("I complete task 1.1.1 in place with the done marker")]
fn complete_task_in_place_with_suffix(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target_path().clone();
    state.run([
        "--in-place",
        "complete",
        "--suffix",
        "— done",
        target.as_str(),
        "1.1.1",
    ])
}

#[when("I try to complete step 1.1")]
fn complete_step_anchor(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target_path().clone();
    state.run(["complete", target.as_str(), "1.1"])
}

#[then("stdout ticks task 1.1.1 and leaves the other task open")]
fn stdout_ticks_first_task(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    assert!(
        state.stdout.contains("- [x] 1.1.1. First task."),
        "expected the completed task in stdout, got: {}",
        state.stdout
    );
    assert!(
        state.stdout.contains("- [ ] 1.1.2. Second task."),
        "expected the untouched task to stay open, got: {}",
        state.stdout
    );
    Ok(())
}

#[then("the target file ticks task 1.1.1 and marks it done before the full stop")]
fn target_marks_task_done(cli_state: &mut CliFixture) -> TestResult {
    let rewritten = state_mut(cli_state)?.read_target()?;
    assert!(
        rewritten.contains("- [x] 1.1.1. First task — done.\n"),
        "expected the done marker before the full stop, got: {rewritten}"
    );
    Ok(())
}

#[then("stderr mentions that anchor 1.1 has no checkbox")]
fn stderr_mentions_no_checkbox(cli_state: &mut CliFixture) -> TestResult {
    let stderr = &state_mut(cli_state)?.stderr;
    assert!(
        stderr.contains("cannot complete step anchor `1.1`"),
        "expected the step-anchor diagnostic, got: {stderr}"
    );
    assert!(
        stderr.contains("only tasks and addendum sub-tasks have checkboxes"),
        "expected the missing-checkbox explanation, got: {stderr}"
    );
    Ok(())
}
