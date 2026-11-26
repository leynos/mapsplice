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

/// One completion golden scenario.
struct CompleteCase {
    /// Fixture directory holding `target.md` and `expected.md`.
    fixture: &'static str,
    /// Anchor to complete.
    anchor: &'static str,
    /// Configured suffix.
    suffix: Option<&'static str>,
    /// Rewrite the target instead of printing the preview.
    in_place: bool,
}

#[rstest]
#[case::task_preview(CompleteCase {
    fixture: "complete_task_preview",
    anchor: "1.1.1",
    suffix: None,
    in_place: false,
})]
#[case::task_in_place(CompleteCase {
    fixture: "complete_task_preview",
    anchor: "1.1.1",
    suffix: None,
    in_place: true,
})]
#[case::dependency_preview(CompleteCase {
    fixture: "complete_dependency_preview",
    anchor: "1.1.1",
    suffix: None,
    in_place: false,
})]
#[case::dependency_in_place(CompleteCase {
    fixture: "complete_dependency_preview",
    anchor: "1.1.1",
    suffix: None,
    in_place: true,
})]
#[case::sub_task_preview(CompleteCase {
    fixture: "complete_sub_task_preview",
    anchor: "1.1.1.1",
    suffix: None,
    in_place: false,
})]
#[case::sub_task_in_place(CompleteCase {
    fixture: "complete_sub_task_preview",
    anchor: "1.1.1.1",
    suffix: None,
    in_place: true,
})]
#[case::wrapped_title_preview(CompleteCase {
    fixture: "complete_wrapped_title_suffix",
    anchor: "1.1.1",
    suffix: Some("— done"),
    in_place: false,
})]
#[case::wrapped_title_in_place(CompleteCase {
    fixture: "complete_wrapped_title_suffix",
    anchor: "1.1.1",
    suffix: Some("— done"),
    in_place: true,
})]
fn complete_golden_cases(
    workspace: TestResult<GoldenWorkspace>,
    #[case] case: CompleteCase,
) -> TestResult {
    let expected = golden_fixture(case.fixture, "expected.md");
    let output = if case.in_place {
        SuccessOutput::InPlaceSuccess { expected }
    } else {
        SuccessOutput::StdoutTargetUnchanged { expected }
    };
    assert_golden_case(
        &workspace?,
        golden_success_output_case(
            case.fixture,
            GoldenCommand::Complete {
                anchor: case.anchor,
                suffix: case.suffix,
            },
            false,
            output,
        ),
    )
}

#[rstest]
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
