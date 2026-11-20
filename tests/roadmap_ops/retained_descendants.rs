//! CLI regressions for descendants retained by a step or phase replacement.
//!
//! A step or phase replacement retires the addressed heading, but a descendant
//! whose number also appears in the single-item fragment continues in place.
//! An external consumer may require that descendant, so the operation must
//! succeed and rewrite the consumer to the descendant's new number rather than
//! reject the edit or leave the clause naming an identity that has gone.

use mapsplice::run_from_args;
use rstest::rstest;

use super::{
    assertions::assert_contains,
    support::{TestResult, Workspace, workspace},
};

/// A step replacement retains the task a later step requires.
///
/// The fragment holds only task `1.1.2`, so the retained task lands at
/// `1.1.1`; the consumer in step `1.2` must follow it there.
#[rstest]
#[serial_test::serial(observability_metrics)]
fn step_replacement_retains_a_referenced_task(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(concat!(
        "# Roadmap\n\n",
        "## 1. Alpha\n\n",
        "### 1.1. Alpha step\n\n",
        "- [ ] 1.1.1. First task.\n",
        "- [ ] 1.1.2. Depended-on task.\n\n",
        "### 1.2. Beta step\n\n",
        "- [ ] 1.2.1. Consumer. Requires 1.1.2.\n",
    ))?;
    test_workspace.write_fragment(concat!(
        "### 1.1. Revised alpha step\n\n",
        "- [x] 1.1.2. Revised depended-on task.\n",
    ))?;

    let outcome = run_from_args([
        "mapsplice",
        "replace",
        test_workspace.target.as_str(),
        "1.1",
        test_workspace.fragment.as_str(),
    ])?;
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "### 1.1. Revised alpha step");
    assert_contains(&stdout, "- [x] 1.1.1. Revised depended-on task.");
    assert_contains(&stdout, "- [ ] 1.2.1. Consumer. Requires 1.1.1.");
    Ok(())
}

/// A phase replacement retains the step and task a later phase requires.
#[rstest]
#[serial_test::serial(observability_metrics)]
fn phase_replacement_retains_referenced_descendants(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(concat!(
        "# Roadmap\n\n",
        "## 1. Alpha\n\n",
        "### 1.1. Alpha step\n\n",
        "- [ ] 1.1.1. Depended-on task.\n\n",
        "## 2. Beta\n\n",
        "### 2.1. Beta step\n\n",
        "- [ ] 2.1.1. Task consumer. Requires 1.1.1.\n",
        "- [ ] 2.1.2. Step consumer. Requires 1.1.\n",
    ))?;
    test_workspace.write_fragment(concat!(
        "## 1. Replacement alpha\n\n",
        "### 1.1. Replacement step\n\n",
        "- [x] 1.1.1. Revised depended-on task.\n",
    ))?;

    let outcome = run_from_args([
        "mapsplice",
        "replace",
        test_workspace.target.as_str(),
        "1",
        test_workspace.fragment.as_str(),
    ])?;
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "## 1. Replacement alpha");
    assert_contains(&stdout, "### 1.1. Replacement step");
    assert_contains(&stdout, "- [x] 1.1.1. Revised depended-on task.");
    assert_contains(&stdout, "- [ ] 2.1.1. Task consumer. Requires 1.1.1.");
    assert_contains(&stdout, "- [ ] 2.1.2. Step consumer. Requires 1.1.");
    Ok(())
}
