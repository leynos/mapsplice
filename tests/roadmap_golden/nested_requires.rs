//! Golden coverage for `Requires` clauses in nested task-body bullets.

use rstest::rstest;

use super::{
    golden::{
        ExpectedError,
        FailureOutput,
        GoldenCommand,
        GoldenFailureSpec,
        GoldenWorkspace,
        TestResult,
        assert_golden_case,
        golden_failure_case,
        golden_success_case,
    },
    workspace,
};

/// Verify a `Requires` clause inside a nested task-body bullet is rewritten.
///
/// Regression for issue #85: the nested bullet carries the clause reachable
/// only through [`mapsplice`]'s structural child sequence, so the inserted task
/// used to leave the consumer pointing at the unrelated item that inherited
/// `1.1.1`.
#[rstest]
#[serial_test::serial(cli_env)]
fn c3_nested_bullet_requires_rewrite(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "c3_nested_bullet_requires_rewrite",
            GoldenCommand::InsertBefore { anchor: "1.1.1" },
            true,
        ),
    )
}

/// Verify insertion renumbers all three clause positions and leaves prose alone.
#[rstest]
#[serial_test::serial(cli_env)]
fn f4_nested_requires_clause_positions(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "f4_nested_requires_clause_positions",
            GoldenCommand::InsertBefore { anchor: "1.1.1" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn c3_nested_bullet_requires_failure(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_failure_case(GoldenFailureSpec {
            name: "c3_nested_bullet_requires_failure",
            command: GoldenCommand::Delete { anchor: "1.1.1" },
            fragment: None,
            error: ExpectedError::DanglingDependency,
            output: FailureOutput::TargetUnchanged,
        }),
    )
}

/// Verify an in-place delete of a still-required item writes nothing at all.
#[rstest]
#[serial_test::serial(cli_env)]
fn f5_nested_requires_delete_in_place(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_failure_case(GoldenFailureSpec {
            name: "f5_nested_requires_delete_in_place",
            command: GoldenCommand::Delete { anchor: "1.1.1" },
            fragment: None,
            error: ExpectedError::DanglingDependency,
            output: FailureOutput::InPlaceTargetUnchanged,
        }),
    )
}

/// Verify preview mode rejects the same deletion without rewriting the target.
#[rstest]
#[serial_test::serial(cli_env)]
fn c6_nested_requires_preview_failure(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_failure_case(GoldenFailureSpec {
            name: "c6_nested_requires_preview_failure",
            command: GoldenCommand::Delete { anchor: "1.1.1" },
            fragment: None,
            error: ExpectedError::DanglingDependency,
            output: FailureOutput::TargetUnchanged,
        }),
    )
}

/// Verify incidental numbers and fenced examples survive a nested-clause delete.
#[rstest]
#[serial_test::serial(cli_env)]
fn c2_nested_prose_and_code_not_rewritten(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "c2_nested_prose_and_code_not_rewritten",
            GoldenCommand::Delete { anchor: "1.1.5" },
            false,
        ),
    )
}
