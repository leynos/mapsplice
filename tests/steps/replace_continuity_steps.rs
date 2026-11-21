//! Steps for replacement identity-continuity scenarios.
//!
//! Issue #228: a one-item replacement carrying the addressed task's own number
//! edits that task rather than retiring it, so a consumer's `Requires` clause
//! keeps its bytes. A replacement that renumbers the task still retires the
//! identity, and a surviving consumer must fail the operation.

use rstest_bdd_macros::{given, then, when};

use super::cli_steps::{CliFixture, state_mut};
use crate::support::TestResult;

/// Roadmap whose consumer requires a later task in the same step.
const DEPENDED_UPON_TASK_TARGET: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. Consumer. Requires 1.1.2.\n",
    "- [ ] 1.1.2. Draft the plan.\n",
);

/// One-item fragment reproducing the addressed task's own number.
const REPRODUCING_FRAGMENT: &str = "- [x] 1.1.2. Draft the plan carefully.\n";

/// One-item fragment renumbering the addressed task away from `1.1.2`.
const RENUMBERED_FRAGMENT: &str = "- [x] 1.1.5. Draft the plan carefully.\n";

#[given("the target roadmap with a depended-upon task")]
fn target_with_depended_upon_task(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_target(DEPENDED_UPON_TASK_TARGET)
}

#[given("the fragment reproducing the depended-upon task")]
fn reproducing_fragment(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_fragment(REPRODUCING_FRAGMENT)
}

#[given("the fragment renumbering the depended-upon task")]
fn renumbering_fragment(cli_state: &mut CliFixture) -> TestResult {
    state_mut(cli_state)?.write_fragment(RENUMBERED_FRAGMENT)
}

/// Run the in-place replacement both `when` steps share.
fn replace_depended_upon_task_in_place(cli_state: &mut CliFixture) -> TestResult {
    let state = state_mut(cli_state)?;
    let target = state.target_path().clone();
    let fragment = state.fragment_path().clone();
    state.run([
        "--in-place",
        "replace",
        target.as_str(),
        "1.1.2",
        fragment.as_str(),
    ])
}

#[when("I replace the depended-upon task in place")]
fn replace_in_place(cli_state: &mut CliFixture) -> TestResult {
    replace_depended_upon_task_in_place(cli_state)
}

#[when("I try to replace the depended-upon task in place")]
fn try_replace_in_place(cli_state: &mut CliFixture) -> TestResult {
    replace_depended_upon_task_in_place(cli_state)
}

#[then("the target file shows the checked-off task and keeps the consumer's clause")]
fn target_shows_checked_off_task(cli_state: &mut CliFixture) -> TestResult {
    let written = state_mut(cli_state)?.read_target()?;
    assert!(
        written.contains("- [x] 1.1.2. Draft the plan carefully.\n"),
        "the replacement task must be checked off, got:\n{written}"
    );
    assert!(
        written.contains("- [ ] 1.1.1. Consumer. Requires 1.1.2.\n"),
        "the consumer's clause must stay byte-identical, got:\n{written}"
    );
    Ok(())
}

#[then("stderr mentions dangling dependency anchor 1.1.2")]
fn stderr_mentions_dangling_anchor(cli_state: &mut CliFixture) -> TestResult {
    let stderr = &state_mut(cli_state)?.stderr;
    if stderr.contains("dependency anchor `1.1.2` was not found in the target roadmap") {
        return Ok(());
    }
    Err(format!("expected dangling dependency diagnostic, got: {stderr}").into())
}
