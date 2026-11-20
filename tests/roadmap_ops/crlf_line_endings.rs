//! CLI regressions for CRLF roadmaps.
//!
//! Issue #85 lists line endings among the inputs the tests must vary. The
//! rejection cases pin whole-file byte identity, so a `\r` stripped on the way
//! to a write cannot hide behind line-by-line assertions. The replacement cases
//! separate preserved source, which keeps interior CRLF terminators, from the
//! documented document-ending rule, which always emits a final LF.

use mapsplice::run_from_args;
use rstest::rstest;

use super::{
    assertions::assert_contains,
    ops_assertions::{assert_dangling_anchor, assert_equal},
    support::{TASK_FRAGMENT, TestResult, Workspace, workspace},
};

/// The nested-bullet roadmap with CRLF line endings.
///
/// Every line ends in `\r\n`, including the nested clause bullet. The parser
/// accepts CRLF and the rejection path compares whole-file bytes, so this
/// fixture must be written and read without translation for the assertion to
/// mean anything.
const CRLF_NESTED_BULLET_CONSUMER: &str = concat!(
    "# Test\r\n\r\n",
    "## 1. Phase\r\n\r\n",
    "### 1.1. Step\r\n\r\n",
    "- [ ] 1.1.1. Prerequisite.\r\n",
    "- [ ] 1.1.2. Consumer.\r\n",
    "  - Requires 1.1.1.\r\n",
);

/// A rejected in-place edit must leave a CRLF target byte-identical, carriage returns and all.
///
/// The mode matters: preview never writes the file, so its byte-identity
/// assertion would hold even if rejection normalized line endings on the way
/// to a write. Only the in-place path can fail that way, which is why this
/// mirrors `in_place_delete_required_by_nested_bullet_leaves_target_byte_identical`
/// rather than `delete_task_required_by_nested_bullet_is_rejected`. A `\r`
/// stripped from the untouched file would still leave a file that looks correct
/// line-by-line, so the assertion is on the whole file rather than on any line.
#[rstest]
#[serial_test::serial(cli_env)]
fn crlf_rejection_leaves_target_byte_identical(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(CRLF_NESTED_BULLET_CONSUMER)?;
    let original = test_workspace.read_target()?;

    let error = run_from_args([
        "mapsplice",
        "--in-place",
        "delete",
        test_workspace.target.as_str(),
        "1.1.1",
    ])
    .expect_err("in-place delete of a required prerequisite must fail under CRLF too");

    assert_dangling_anchor(&error, "1.1.1");
    assert_equal(&test_workspace.read_target()?, &original);
    Ok(())
}

/// An in-place replacement normalizes only the document's final terminator.
///
/// The replacement item is canonically rendered and comes back with LF. The
/// consumer task it does not touch keeps its source line, carriage return
/// included, but its nested clause is the document's last line, and a non-empty
/// rendered document always ends in one LF. The clause therefore loses only
/// that final terminator; its text and anchor are unchanged. The companion
/// regression below puts a task after the consumer, where the clause keeps its
/// `\r\n`, so the two together separate preserved source from the documented
/// document-ending rule.
#[rstest]
#[serial_test::serial(cli_env)]
fn crlf_in_place_replace_normalizes_only_the_final_terminator(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(CRLF_NESTED_BULLET_CONSUMER)?;
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

    assert_contains(&written, "- [ ] 1.1.2. Consumer.\r\n  - Requires 1.1.1.\n");
    assert_contains(&written, "- [x] 1.1.1. Revised prerequisite.\n");
    Ok(())
}

/// An interior nested clause keeps its CRLF terminator.
///
/// The same in-place replacement leaves the consumer's clause unrewritten, and
/// because a later task follows it, the clause is not the document's final
/// line. Its `\r\n` therefore survives; only the document's last line gains the
/// canonical LF. The pair of regressions separates preserved source from the
/// document-ending rule instead of attributing both to the clause.
#[rstest]
#[serial_test::serial(cli_env)]
fn crlf_in_place_replace_keeps_an_interior_clause_terminator(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(concat!(
        "# Test\r\n\r\n",
        "## 1. Phase\r\n\r\n",
        "### 1.1. Step\r\n\r\n",
        "- [ ] 1.1.1. Prerequisite.\r\n",
        "- [ ] 1.1.2. Consumer.\r\n",
        "  - Requires 1.1.1.\r\n",
        "- [ ] 1.1.3. Tail task.\r\n",
    ))?;
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

    assert_contains(
        &written,
        concat!(
            "- [ ] 1.1.2. Consumer.\r\n",
            "  - Requires 1.1.1.\r\n",
            "- [ ] 1.1.3. Tail task.\n",
        ),
    );
    Ok(())
}

/// A CRLF target rewrites its nested clause correctly, normalizing line endings.
///
/// The renumbering contract is unchanged by the line-ending convention: the
/// clause follows its prerequisite rather than the inserted task that reused
/// `1.1.1`. What differs is the rendering — the rewritten region comes back as
/// LF, which is the documented normalization rather than a regression. Asserting
/// the LF form is deliberate: it pins the actual behaviour, so a future change
/// that starts preserving CRLF through a rewrite is visible here instead of
/// passing unnoticed.
#[rstest]
#[serial_test::serial(cli_env)]
fn crlf_target_rewrites_nested_clause_and_normalizes_line_endings(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(CRLF_NESTED_BULLET_CONSUMER)?;
    test_workspace.write_fragment(TASK_FRAGMENT)?;

    let outcome = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "1.1.1",
        test_workspace.fragment.as_str(),
    ])?;
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "- [ ] 1.1.3. Consumer.\n\n  - Requires 1.1.2.\n");
    Ok(())
}
