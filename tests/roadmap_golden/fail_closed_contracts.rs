//! Fail-closed in-place contract matrix.

use rstest::{fixture, rstest};

use super::golden::{
    ExpectedError,
    FailureOutput,
    FixturePath,
    GoldenCommand,
    GoldenFailureSpec,
    GoldenWorkspace,
    TestResult,
    assert_golden_case,
    create_workspace,
    golden_failure_case,
    golden_fixture,
};

#[fixture]
fn workspace() -> TestResult<GoldenWorkspace> {
    let workspace = create_workspace()?;
    Ok(workspace)
}

/// One fail-closed in-place command case.
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
#[case::f7_complete_missing_anchor_in_place(FailClosedCase::new(
    "f7_complete_missing_anchor_in_place",
    GoldenCommand::Complete {
        anchor: "1.1.9",
        suffix: None,
    },
    None,
    ExpectedError::MissingAnchor
))]
#[case::f7_complete_phase_anchor_in_place(FailClosedCase::new(
    "f7_complete_phase_anchor_in_place",
    GoldenCommand::Complete {
        anchor: "1",
        suffix: None,
    },
    None,
    ExpectedError::UnsupportedAnchorLevel
))]
#[case::f7_complete_step_anchor_in_place(FailClosedCase::new(
    "f7_complete_step_anchor_in_place",
    GoldenCommand::Complete {
        anchor: "1.1",
        suffix: Some("— done"),
    },
    None,
    ExpectedError::UnsupportedAnchorLevel
))]
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
