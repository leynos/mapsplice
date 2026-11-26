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
        GoldenCase,
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

/// How a continuity fixture is expected to finish.
#[derive(Clone, Copy, Debug)]
enum Outcome {
    /// Preview succeeds and its stdout matches the case's own expected file.
    Success,
    /// In-place succeeds and must reproduce another case's expected file.
    InPlace { expected_from: &'static str },
    /// The operation fails and leaves the target as the case records.
    Failure { output: FailureOutput },
}

/// One named golden fixture and the outcome its files record.
#[derive(Clone, Copy, Debug)]
struct Case {
    name: &'static str,
    anchor: &'static str,
    outcome: Outcome,
}

impl Case {
    /// A preview success whose expected file sits beside its other fixtures.
    const fn success(name: &'static str, anchor: &'static str) -> Self {
        Self {
            name,
            anchor,
            outcome: Outcome::Success,
        }
    }

    /// An in-place success that must equal its preview sibling's output.
    const fn in_place(
        name: &'static str,
        anchor: &'static str,
        expected_from: &'static str,
    ) -> Self {
        Self {
            name,
            anchor,
            outcome: Outcome::InPlace { expected_from },
        }
    }

    /// A dangling-dependency rejection with the stated target outcome.
    const fn failure(name: &'static str, anchor: &'static str, output: FailureOutput) -> Self {
        Self {
            name,
            anchor,
            outcome: Outcome::Failure { output },
        }
    }

    /// Build the runner case this fixture describes.
    fn spec(self) -> GoldenCase {
        let command = GoldenCommand::Replace {
            anchor: self.anchor,
        };
        match self.outcome {
            Outcome::Success => golden_success_case(self.name, command, true),
            Outcome::InPlace { expected_from } => golden_success_output_case(
                self.name,
                command,
                true,
                SuccessOutput::InPlaceSuccess {
                    expected: golden_fixture(expected_from, "expected.md"),
                },
            ),
            Outcome::Failure { output } => golden_failure_case(GoldenFailureSpec {
                name: self.name,
                command,
                fragment: Some(golden_fixture(self.name, "fragment.md")),
                error: ExpectedError::DanglingDependency,
                output,
            }),
        }
    }
}

/// Verify every continuity fixture against its recorded golden outcome.
#[rstest]
#[case::checks_off(Case::success("replace_depended_task_checks_off", "1.1.2"))]
#[case::checks_off_in_place(Case::in_place(
    "replace_depended_task_checks_off_in_place",
    "1.1.2",
    "replace_depended_task_checks_off"
))]
#[case::identical_preview(Case::success("replace_depended_task_identical_preview", "1.1.2"))]
#[case::identical_in_place(Case::in_place(
    "replace_depended_task_identical_in_place",
    "1.1.2",
    "replace_depended_task_identical_in_place"
))]
#[case::changed_number_preview(Case::failure(
    "replace_depended_task_changed_number_rejected_preview",
    "1.1.2",
    FailureOutput::TargetUnchanged
))]
#[case::changed_number_in_place(Case::failure(
    "replace_depended_task_changed_number_rejected_in_place",
    "1.1.2",
    FailureOutput::InPlaceTargetUnchanged
))]
#[case::dropped_anchor_multi_item(Case::failure(
    "replace_depended_task_dropped_anchor_multi_item",
    "1.1.2",
    FailureOutput::TargetUnchanged
))]
#[case::retains_referenced_descendant(Case::success(
    "replace_retains_referenced_descendant",
    "1.1"
))]
fn replacement_continuity_golden_cases(
    workspace: TestResult<GoldenWorkspace>,
    #[case] case: Case,
) -> TestResult {
    assert_golden_case(&workspace?, case.spec())
}
