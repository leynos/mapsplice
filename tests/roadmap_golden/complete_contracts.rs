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
    golden_success_output_case,
};

#[fixture]
fn workspace() -> TestResult<GoldenWorkspace> {
    let workspace = create_workspace()?;
    Ok(workspace)
}

#[rstest]
#[case::task("complete_task_preview", "1.1.1", None)]
#[case::dependency("complete_dependency_preview", "1.1.1", None)]
#[case::sub_task("complete_sub_task_preview", "1.1.1.1", None)]
#[case::wrapped_title_suffix("complete_wrapped_title_suffix", "1.1.1", Some("— done"))]
#[serial_test::serial(cli_env)]
fn complete_preview_golden_cases(
    workspace: TestResult<GoldenWorkspace>,
    #[case] fixture_name: &'static str,
    #[case] anchor: &'static str,
    #[case] suffix: Option<&'static str>,
) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_output_case(
            fixture_name,
            GoldenCommand::Complete { anchor, suffix },
            false,
            SuccessOutput::StdoutTargetUnchanged {
                expected: golden_fixture(fixture_name, "expected.md"),
            },
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn complete_preview_of_a_completed_item_prints_the_original(
    workspace: TestResult<GoldenWorkspace>,
) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_output_case(
            "complete_preview_completed",
            GoldenCommand::Complete {
                anchor: "1.1.1",
                suffix: Some("— done"),
            },
            false,
            SuccessOutput::StdoutTargetUnchanged {
                expected: golden_fixture("complete_preview_completed", "expected.md"),
            },
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
