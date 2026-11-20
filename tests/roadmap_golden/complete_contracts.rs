//! Completion golden fixtures for in-place checkbox edits.

use rstest::{fixture, rstest};

use super::golden::{
    GoldenCommand,
    GoldenWorkspace,
    SuccessOutput,
    TestResult,
    assert_golden_case,
    create_workspace,
    golden_fixture,
    golden_success_case,
    golden_success_output_case,
};

#[fixture]
fn workspace() -> TestResult<GoldenWorkspace> {
    let workspace = create_workspace()?;
    Ok(workspace)
}

#[rstest]
#[serial_test::serial(cli_env)]
fn complete_task_preview(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "complete_task_preview",
            GoldenCommand::Complete {
                anchor: "1.1.1",
                suffix: None,
            },
            false,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn complete_dependency_preview(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "complete_dependency_preview",
            GoldenCommand::Complete {
                anchor: "1.1.1",
                suffix: None,
            },
            false,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn complete_sub_task_preview(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "complete_sub_task_preview",
            GoldenCommand::Complete {
                anchor: "1.1.1.1",
                suffix: None,
            },
            false,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn complete_wrapped_title_suffix(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "complete_wrapped_title_suffix",
            GoldenCommand::Complete {
                anchor: "1.1.1",
                suffix: Some("— done"),
            },
            false,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn complete_in_place(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_output_case(
            "complete_in_place",
            GoldenCommand::Complete {
                anchor: "1.1.2",
                suffix: None,
            },
            false,
            SuccessOutput::InPlaceSuccess {
                expected: golden_fixture("complete_in_place", "expected.md"),
            },
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn complete_idempotent(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_output_case(
            "complete_idempotent",
            GoldenCommand::Complete {
                anchor: "1.1.1",
                suffix: Some("— done"),
            },
            false,
            SuccessOutput::InPlaceIdempotent {
                expected: golden_fixture("complete_idempotent", "expected.md"),
            },
        ),
    )
}
