//! Golden coverage for identity continuity in replacement.
//!
//! A one-item replacement numbered as the addressed task is an edit of that
//! task, so a surviving consumer's clause keeps its bytes. A changed number or
//! a multi-item replacement still retires the addressed identity, and a
//! surviving clause still fails the operation.

use rstest::rstest;

use super::{
    golden::{
        ExpectedError,
        FailureOutput,
        GoldenCommand,
        GoldenFailureSpec,
        GoldenWorkspace,
        SuccessOutput,
        TestResult,
        assert_golden_case,
        golden_failure_case,
        golden_fixture,
        golden_success_case,
        golden_success_output_case,
    },
    workspace,
};

/// Verify checking off a depended-on task leaves every consumer clause alone.
#[rstest]
fn replace_depended_task_checks_off(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "replace_depended_task_checks_off",
            GoldenCommand::Replace { anchor: "1.1.2" },
            true,
        ),
    )
}

/// Verify the in-place result is the preview result written to the target.
///
/// The expected fixture is the preview case's own, so the two modes cannot
/// drift apart without this case failing.
#[rstest]
fn replace_depended_task_checks_off_in_place(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_output_case(
            "replace_depended_task_checks_off_in_place",
            GoldenCommand::Replace { anchor: "1.1.2" },
            true,
            SuccessOutput::InPlaceSuccess {
                expected: golden_fixture("replace_depended_task_checks_off", "expected.md"),
            },
        ),
    )
}

/// Verify a changed fragment number still retires the identity in preview.
#[rstest]
fn replace_depended_task_changed_number_rejected_preview(
    workspace: TestResult<GoldenWorkspace>,
) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_failure_case(GoldenFailureSpec {
            name: "replace_depended_task_changed_number_rejected_preview",
            command: GoldenCommand::Replace { anchor: "1.1.2" },
            fragment: Some(golden_fixture(
                "replace_depended_task_changed_number_rejected_preview",
                "fragment.md",
            )),
            error: ExpectedError::DanglingDependency,
            output: FailureOutput::TargetUnchanged,
        }),
    )
}

/// Verify the same rejection in place leaves the target byte-identical.
#[rstest]
fn replace_depended_task_changed_number_rejected_in_place(
    workspace: TestResult<GoldenWorkspace>,
) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_failure_case(GoldenFailureSpec {
            name: "replace_depended_task_changed_number_rejected_in_place",
            command: GoldenCommand::Replace { anchor: "1.1.2" },
            fragment: Some(golden_fixture(
                "replace_depended_task_changed_number_rejected_in_place",
                "fragment.md",
            )),
            error: ExpectedError::DanglingDependency,
            output: FailureOutput::InPlaceTargetUnchanged,
        }),
    )
}

/// Verify a multi-item replacement without the addressed number is rejected.
#[rstest]
fn replace_depended_task_dropped_anchor_multi_item(
    workspace: TestResult<GoldenWorkspace>,
) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_failure_case(GoldenFailureSpec {
            name: "replace_depended_task_dropped_anchor_multi_item",
            command: GoldenCommand::Replace { anchor: "1.1.2" },
            fragment: Some(golden_fixture(
                "replace_depended_task_dropped_anchor_multi_item",
                "fragment.md",
            )),
            error: ExpectedError::DanglingDependency,
            output: FailureOutput::TargetUnchanged,
        }),
    )
}

/// Verify a uniquely paired descendant keeps its external consumer resolving.
///
/// Replacing step `1.1` with a step holding only task `1.1.2` continues that
/// task, which lands at `1.1.1`. The consumer in step `1.2` must follow the
/// task to its new number rather than be rejected or left naming the old one.
#[rstest]
fn replace_retains_referenced_descendant(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "replace_retains_referenced_descendant",
            GoldenCommand::Replace { anchor: "1.1" },
            true,
        ),
    )
}
