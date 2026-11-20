//! CLI regressions for `Requires` clauses in nested task-body bullets.
//!
//! Issue #85: a clause reachable only through a task's structural child
//! sequence used to be skipped, so renumbering left the consumer naming
//! whichever item inherited its old number, and deleting a still-required
//! prerequisite produced a self-dependency instead of a rejection.

use std::process::Command;

use mapsplice::run_from_args;
use rstest::rstest;

// The CRLF regressions exercise this module's nested-bullet shape under the
// other line-ending convention, so they are declared here rather than at the
// crate root, whose shared fixture wiring is already over the size limit.
#[path = "crlf_line_endings.rs"]
mod crlf_line_endings;

use super::{
    assert_equal,
    assertions::assert_contains,
    ops_assertions::assert_dangling_anchor,
    support::{TASK_FRAGMENT, TestResult, Workspace, workspace},
};

/// Roadmap whose consumer reaches `1.1.1` through a nested task-body bullet.
///
/// The nested bullet is the issue #85 shape: the clause lives in a body block
/// rather than in the task summary, so it is only visible through the task's
/// structural child sequence.
const NESTED_BULLET_CONSUMER: &str = concat!(
    "# Test\n\n",
    "## 1. Phase\n\n",
    "### 1.1. Step\n\n",
    "- [ ] 1.1.1. Prerequisite.\n",
    "- [ ] 1.1.2. Consumer.\n",
    "  - Requires 1.1.1.\n",
);

/// Insertion must not silently redirect a nested consumer to the new task.
///
/// Issue #85: the inserted task inherits the consumer's old number, so a
/// clause that is never scanned leaves the consumer depending on an unrelated
/// item.
#[rstest]
#[serial_test::serial(cli_env)]
fn insert_before_task_rewrites_nested_bullet_dependency(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(NESTED_BULLET_CONSUMER)?;
    test_workspace.write_fragment(TASK_FRAGMENT)?;

    let outcome = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "1.1.1",
        test_workspace.fragment.as_str(),
    ])?;
    let stdout = outcome.stdout.unwrap_or_default();

    // The consumer block pins both halves of the contract: the consumer takes
    // the number the prerequisite vacated, and its nested clause follows the
    // prerequisite rather than the inserted task that reused `1.1.1`.
    assert_contains(&stdout, "- [ ] 1.1.3. Consumer.\n\n  - Requires 1.1.2.\n");
    Ok(())
}

/// An in-place replacement keeps a nested-bullet consumer resolving.
///
/// The clause lives in the consumer's task body, so it is only reachable
/// through the structural child sequence. Replacing the prerequisite with one
/// item carrying its own number continues the identity, and the consumer's
/// nested bullet must keep both its anchor and its bytes.
#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_replace_keeps_nested_bullet_dependency(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(NESTED_BULLET_CONSUMER)?;
    test_workspace.write_fragment("- [x] 1.1.1. Revised prerequisite.\n")?;

    run_from_args([
        "mapsplice",
        "--in-place",
        "replace",
        test_workspace.target.as_str(),
        "1.1.1",
        test_workspace.fragment.as_str(),
    ])?;
    let written = test_workspace.read_target()?;

    assert_contains(&written, "- [x] 1.1.1. Revised prerequisite.\n");
    // The consumer is untouched, so it keeps the source's exact bytes — there
    // is no blank line between the task line and its nested bullet here.
    assert_contains(&written, "- [ ] 1.1.2. Consumer.\n  - Requires 1.1.1.\n");
    Ok(())
}

/// An in-place replacement that renumbers still keeps its dependency.
///
/// The fragment carries the addressed anchor `1.1.3`, but the step has a gap,
/// so contiguous renumbering moves the replacement to `1.1.2`. The consumer's
/// clause is rewritten to the new number rather than left dangling: the
/// reference follows the identity, not the digits.
#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_replace_renumbers_item_and_rewrites_its_consumer(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(concat!(
        "# Test\n\n",
        "## 1. Phase\n\n",
        "### 1.1. Step\n\n",
        "- [ ] 1.1.1. Consumer. Requires 1.1.3.\n",
        "- [ ] 1.1.3. Prerequisite.\n",
    ))?;
    test_workspace.write_fragment("- [x] 1.1.3. Revised prerequisite.\n")?;

    run_from_args([
        "mapsplice",
        "--in-place",
        "replace",
        test_workspace.target.as_str(),
        "1.1.3",
        test_workspace.fragment.as_str(),
    ])?;
    let written = test_workspace.read_target()?;

    assert_contains(&written, "- [ ] 1.1.1. Consumer. Requires 1.1.2.\n");
    assert_contains(&written, "- [x] 1.1.2. Revised prerequisite.\n");
    Ok(())
}

/// Deleting a still-required prerequisite must fail, not create a self-dependency.
#[rstest]
#[serial_test::serial(cli_env)]
fn delete_task_required_by_nested_bullet_is_rejected(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(NESTED_BULLET_CONSUMER)?;
    let original = test_workspace.read_target()?;

    let error = run_from_args([
        "mapsplice",
        "delete",
        test_workspace.target.as_str(),
        "1.1.1",
    ])
    .expect_err("deleting a required prerequisite must fail");

    assert_dangling_anchor(&error, "1.1.1");
    assert_equal(&test_workspace.read_target()?, &original);
    Ok(())
}

/// An in-place delete must reject the edit before touching the file.
#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_delete_required_by_nested_bullet_leaves_target_byte_identical(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(NESTED_BULLET_CONSUMER)?;
    let original = test_workspace.read_target()?;

    let error = run_from_args([
        "mapsplice",
        "--in-place",
        "delete",
        test_workspace.target.as_str(),
        "1.1.1",
    ])
    .expect_err("in-place delete of a required prerequisite must fail");

    assert_dangling_anchor(&error, "1.1.1");
    assert_equal(&test_workspace.read_target()?, &original);
    Ok(())
}

/// A rejected preview must emit no roadmap body at all.
///
/// This is the only test that distinguishes preview mode from in-place mode
/// from the outside. The in-process entry point returns `Err` and so yields no
/// outcome whose stdout could be inspected, which makes an in-process
/// "no stdout" assertion vacuous; spawning the real process is what shows
/// whether a roadmap escaped to stdout before the failure.
#[rstest]
#[serial_test::serial(cli_env)]
fn preview_delete_required_by_nested_bullet_emits_no_body(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(NESTED_BULLET_CONSUMER)?;
    let original = test_workspace.read_target()?;

    let output = Command::new(env!("CARGO_BIN_EXE_mapsplice"))
        .args(["delete", test_workspace.target.as_str(), "1.1.1"])
        .output()?;

    if output.status.success() {
        return Err("preview delete of a required prerequisite must fail".into());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        return Err(format!(
            "a rejected preview must emit no roadmap body, but stdout held:\n{stdout}"
        )
        .into());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.contains("1.1.1") {
        return Err(format!("the diagnostic must name the stranded anchor, got:\n{stderr}").into());
    }
    assert_equal(&test_workspace.read_target()?, &original);
    Ok(())
}

/// Both non-nested clause forms must keep rewriting now that nesting works.
#[rstest]
#[serial_test::serial(cli_env)]
fn inline_and_continuation_clauses_still_rewrite(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(concat!(
        "# Test\n\n",
        "## 1. Phase\n\n",
        "### 1.1. Step\n\n",
        "- [ ] 1.1.1. Prerequisite.\n",
        "- [ ] 1.1.2. Inline consumer. Requires 1.1.1.\n",
        "- [ ] 1.1.3. Continuation consumer.\n",
        "  Requires 1.1.1.\n",
    ))?;
    test_workspace.write_fragment(TASK_FRAGMENT)?;

    let outcome = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "1.1.1",
        test_workspace.fragment.as_str(),
    ])?;
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "- [ ] 1.1.3. Inline consumer. Requires 1.1.2.");
    assert_contains(
        &stdout,
        "- [ ] 1.1.4. Continuation consumer.\n  Requires 1.1.2.",
    );
    Ok(())
}

/// Incidental numbers and fenced examples must survive a nested-clause rewrite.
#[rstest]
#[serial_test::serial(cli_env)]
fn nested_bullet_rewrite_preserves_incidental_numbers_and_code(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(concat!(
        "# Test\n\n",
        "## 1. Phase\n\n",
        "### 1.1. Step\n\n",
        "- [ ] 1.1.1. Prerequisite.\n",
        "- [ ] 1.1.2. Consumer.\n",
        "  - Requires 1.1.1. See §2.1, release 1.4.0, count 27.\n",
        "  - Blocks 1.1.1.\n",
        "  ```text\n",
        "  Requires 1.1.1.\n",
        "  ```\n",
    ))?;
    test_workspace.write_fragment(TASK_FRAGMENT)?;

    let outcome = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "1.1.1",
        test_workspace.fragment.as_str(),
    ])?;
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(
        &stdout,
        "- Requires 1.1.2. See §2.1, release 1.4.0, count 27.",
    );
    assert_contains(&stdout, "- Blocks 1.1.1.");
    // The fenced block must survive byte-for-byte, including its fence and its
    // indentation. Asserting on the bare clause text would not establish that:
    // the inserted task's own self-reference is rewritten to `Requires 1.1.1.`
    // as well, so that assertion would pass whether or not the fence survived.
    assert_contains(&stdout, "  ```text\n  Requires 1.1.1.\n  ```\n");
    Ok(())
}
