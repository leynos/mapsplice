//! Contract-focused golden fixture coverage.

use rstest::{fixture, rstest};

use super::golden::{
    ExpectedError,
    FailureOutput,
    FixturePath,
    GoldenCommand,
    GoldenFailureSpec,
    GoldenWorkspace,
    SuccessOutput,
    TestResult,
    assert_golden_case,
    create_workspace,
    golden_failure_case,
    golden_fixture,
    golden_success_case,
    golden_success_output_case,
};

#[fixture]
fn workspace() -> TestResult<GoldenWorkspace> {
    let workspace = create_workspace()?;
    Ok(workspace)
}

#[derive(Clone, Copy, Debug)]
struct FailClosedCase {
    name: &'static str,
    command: GoldenCommand,
    fragment: Option<FixturePath>,
    error: ExpectedError,
}

impl FailClosedCase {
    const fn new(
        name: &'static str,
        command: GoldenCommand,
        fragment: Option<FixturePath>,
        error: ExpectedError,
    ) -> Self {
        Self {
            name,
            command,
            fragment,
            error,
        }
    }
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
        golden_failure_case(GoldenFailureSpec {
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
        golden_failure_case(GoldenFailureSpec {
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
        golden_failure_case(GoldenFailureSpec {
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
fn f5_append_level_mismatch_failure(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_failure_case(GoldenFailureSpec {
            name: "f5_append_level_mismatch_failure",
            command: GoldenCommand::Append,
            fragment: Some(golden_fixture(
                "f5_append_level_mismatch_failure",
                "fragment.md",
            )),
            error: ExpectedError::AppendLevelMismatch,
            output: FailureOutput::InPlaceTargetUnchanged,
        }),
    )
}

#[rstest]
#[case::f5_append_malformed_target_in_place(FailClosedCase::new(
    "f5_append_malformed_target_in_place",
    GoldenCommand::Append,
    Some(golden_fixture("f5_append_malformed_target_in_place", "fragment.md")),
    ExpectedError::InvalidRoadmap
))]
#[case::f5_insert_malformed_target_in_place(
    FailClosedCase::new(
    "f5_insert_malformed_target_in_place",
    GoldenCommand::InsertBefore { anchor: "1" },
    Some(golden_fixture("f5_insert_malformed_target_in_place", "fragment.md")),
    ExpectedError::InvalidRoadmap
    )
)]
#[case::f5_delete_malformed_target_in_place(
    FailClosedCase::new(
    "f5_delete_malformed_target_in_place",
    GoldenCommand::Delete { anchor: "1" },
    None,
    ExpectedError::InvalidRoadmap
    )
)]
#[case::f5_replace_malformed_target_in_place(
    FailClosedCase::new(
    "f5_replace_malformed_target_in_place",
    GoldenCommand::Replace { anchor: "1" },
    Some(golden_fixture("f5_replace_malformed_target_in_place", "fragment.md")),
    ExpectedError::InvalidRoadmap
    )
)]
#[case::f5_append_malformed_fragment_in_place(FailClosedCase::new(
    "f5_append_malformed_fragment_in_place",
    GoldenCommand::Append,
    Some(golden_fixture("f5_append_malformed_fragment_in_place", "fragment.md")),
    ExpectedError::InvalidRoadmap
))]
#[case::f5_insert_malformed_fragment_in_place(
    FailClosedCase::new(
    "f5_insert_malformed_fragment_in_place",
    GoldenCommand::InsertBefore { anchor: "1" },
    Some(golden_fixture("f5_insert_malformed_fragment_in_place", "fragment.md")),
    ExpectedError::InvalidRoadmap
    )
)]
#[case::f5_replace_malformed_fragment_in_place(
    FailClosedCase::new(
    "f5_replace_malformed_fragment_in_place",
    GoldenCommand::Replace { anchor: "1" },
    Some(golden_fixture("f5_replace_malformed_fragment_in_place", "fragment.md")),
    ExpectedError::InvalidRoadmap
    )
)]
#[case::f5_insert_level_mismatch_in_place(
    FailClosedCase::new(
    "f5_insert_level_mismatch_in_place",
    GoldenCommand::InsertBefore { anchor: "1" },
    Some(golden_fixture("f5_insert_level_mismatch_in_place", "fragment.md")),
    ExpectedError::LevelMismatch
    )
)]
#[case::f5_replace_level_mismatch_in_place(
    FailClosedCase::new(
    "f5_replace_level_mismatch_in_place",
    GoldenCommand::Replace { anchor: "1" },
    Some(golden_fixture("f5_replace_level_mismatch_in_place", "fragment.md")),
    ExpectedError::LevelMismatch
    )
)]
#[case::f5_insert_missing_phase_anchor_in_place(
    FailClosedCase::new(
    "f5_insert_missing_phase_anchor_in_place",
    GoldenCommand::InsertBefore { anchor: "99" },
    Some(golden_fixture("f5_insert_missing_phase_anchor_in_place", "fragment.md")),
    ExpectedError::MissingAnchor
    )
)]
#[case::f5_delete_missing_step_anchor_in_place(
    FailClosedCase::new(
    "f5_delete_missing_step_anchor_in_place",
    GoldenCommand::Delete { anchor: "1.9" },
    None,
    ExpectedError::MissingAnchor
    )
)]
#[case::f5_replace_missing_task_anchor_in_place(
    FailClosedCase::new(
    "f5_replace_missing_task_anchor_in_place",
    GoldenCommand::Replace { anchor: "1.1.9" },
    Some(golden_fixture("f5_replace_missing_task_anchor_in_place", "fragment.md")),
    ExpectedError::MissingAnchor
    )
)]
#[case::f5_delete_missing_addendum_anchor_in_place(
    FailClosedCase::new(
    "f5_delete_missing_addendum_anchor_in_place",
    GoldenCommand::Delete { anchor: "1.1.1.9" },
    None,
    ExpectedError::MissingAnchor
    )
)]
#[case::f5_dependency_rewrite_failure_in_place(
    FailClosedCase::new(
    "f5_dependency_rewrite_failure_in_place",
    GoldenCommand::Delete { anchor: "1" },
    None,
    ExpectedError::DanglingDependency
    )
)]
#[serial_test::serial(cli_env)]
fn f5_in_place_boundary_matrix(
    workspace: TestResult<GoldenWorkspace>,
    #[case] case: FailClosedCase,
) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_failure_case(GoldenFailureSpec {
            name: case.name,
            command: case.command,
            fragment: case.fragment,
            error: case.error,
            output: FailureOutput::InPlaceTargetUnchanged,
        }),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn f5_render_failure_in_place(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_failure_case(GoldenFailureSpec {
            name: "f5_render_failure_in_place",
            command: GoldenCommand::Append,
            fragment: Some(golden_fixture("f5_render_failure_in_place", "fragment.md")),
            error: ExpectedError::InvalidRoadmap,
            output: FailureOutput::InPlaceTargetUnchanged,
        }),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn f5_missing_anchor_in_place_failure(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_failure_case(GoldenFailureSpec {
            name: "f5_missing_anchor_in_place_failure",
            command: GoldenCommand::Delete { anchor: "99" },
            fragment: None,
            error: ExpectedError::MissingAnchor,
            output: FailureOutput::InPlaceTargetUnchanged,
        }),
    )
}
