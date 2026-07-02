//! Contract-focused golden fixture coverage.

use rstest::{fixture, rstest};

use super::golden::{
    ExpectedError,
    FailureOutput,
    FixturePath,
    GoldenCase,
    GoldenCommand,
    GoldenExpectation,
    GoldenWorkspace,
    SuccessOutput,
    TestResult,
    assert_golden_case,
    create_workspace,
};

#[fixture]
fn workspace() -> TestResult<GoldenWorkspace> {
    let workspace = create_workspace()?;
    Ok(workspace)
}

fn golden_success_case(
    name: &'static str,
    command: GoldenCommand,
    has_fragment: bool,
) -> GoldenCase {
    golden_success_output_case(
        name,
        command,
        has_fragment,
        SuccessOutput::Stdout {
            expected: golden_fixture(name, "expected.md"),
        },
    )
}

fn golden_success_output_case(
    name: &'static str,
    command: GoldenCommand,
    has_fragment: bool,
    output: SuccessOutput,
) -> GoldenCase {
    GoldenCase {
        name,
        command,
        target: golden_fixture(name, "target.md"),
        fragment: has_fragment.then_some(golden_fixture(name, "fragment.md")),
        expectation: GoldenExpectation::Success { output },
    }
}

#[derive(Clone, Copy)]
struct FailureSpec {
    name: &'static str,
    command: GoldenCommand,
    fragment: Option<FixturePath>,
    error: ExpectedError,
    output: FailureOutput,
}

const fn golden_failure_case(spec: FailureSpec) -> GoldenCase {
    GoldenCase {
        name: spec.name,
        command: spec.command,
        target: golden_fixture(spec.name, "target.md"),
        fragment: spec.fragment,
        expectation: GoldenExpectation::Failure {
            error: spec.error,
            output: spec.output,
        },
    }
}

const fn golden_fixture(case: &'static str, file: &'static str) -> FixturePath {
    FixturePath::Golden { case, file }
}

#[rstest]
#[serial_test::serial(cli_env)]
fn f1_minimal_untouched_content(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "f1_minimal_untouched_content",
            GoldenCommand::Replace { anchor: "1.1.1" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn f2_minimal_renumber_diff(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "f2_minimal_renumber_diff",
            GoldenCommand::InsertBefore { anchor: "1.1.2" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn f3_c5_identity_replace(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_output_case(
            "f3_c5_identity_replace",
            GoldenCommand::Replace { anchor: "1.1.1" },
            true,
            SuccessOutput::OriginalTargetStdout,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn f4_formatter_stability_smoke(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "f4_formatter_stability_smoke",
            GoldenCommand::InsertAfter { anchor: "1.1.1" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn c2_contiguous_renumber(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "c2_contiguous_renumber",
            GoldenCommand::InsertBefore { anchor: "2" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn c4_addendum_renumber(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "c4_addendum_renumber",
            GoldenCommand::InsertBefore { anchor: "1" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn c4_addendum_render_fidelity(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "c4_addendum_render_fidelity",
            GoldenCommand::Delete { anchor: "1.1.2" },
            false,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn c3_dangling_requires_failure(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_failure_case(FailureSpec {
            name: "c3_dangling_requires_failure",
            command: GoldenCommand::Delete { anchor: "1" },
            fragment: None,
            error: ExpectedError::DanglingDependency,
            output: FailureOutput::TargetUnchanged,
        }),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn c6_stdout_target_unchanged(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_output_case(
            "c6_stdout_target_unchanged",
            GoldenCommand::InsertAfter { anchor: "1.1.1" },
            true,
            SuccessOutput::StdoutTargetUnchanged {
                expected: golden_fixture("c6_stdout_target_unchanged", "expected.md"),
            },
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn c6_in_place_success(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_output_case(
            "c6_in_place_success",
            GoldenCommand::Delete { anchor: "1.1.1" },
            false,
            SuccessOutput::InPlaceSuccess {
                expected: golden_fixture("c6_in_place_success", "expected.md"),
            },
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn f5_malformed_grammar_failure(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_failure_case(FailureSpec {
            name: "f5_malformed_grammar_failure",
            command: GoldenCommand::Delete { anchor: "1" },
            fragment: None,
            error: ExpectedError::InvalidRoadmap,
            output: FailureOutput::TargetUnchanged,
        }),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn f5_level_mismatch_failure(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_failure_case(FailureSpec {
            name: "f5_level_mismatch_failure",
            command: GoldenCommand::InsertBefore { anchor: "1" },
            fragment: Some(golden_fixture("f5_level_mismatch_failure", "fragment.md")),
            error: ExpectedError::LevelMismatch,
            output: FailureOutput::TargetUnchanged,
        }),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn f5_missing_anchor_in_place_failure(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_failure_case(FailureSpec {
            name: "f5_missing_anchor_in_place_failure",
            command: GoldenCommand::Delete { anchor: "99" },
            fragment: None,
            error: ExpectedError::MissingAnchor,
            output: FailureOutput::InPlaceTargetUnchanged,
        }),
    )
}
