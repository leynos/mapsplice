//! Assertion helpers for golden roadmap fixture outcomes.

use mapsplice::{MapspliceError, RunOutcome};

use super::{
    ExpectedError,
    FailureOutput,
    GoldenWorkspace,
    SuccessOutput,
    TestResult,
    expected_output,
};

pub(crate) struct SuccessAssertion<'a> {
    pub(crate) name: &'a str,
    pub(crate) workspace: &'a GoldenWorkspace,
    pub(crate) original_target: &'a str,
    pub(crate) output: SuccessOutput,
    pub(crate) outcome: &'a RunOutcome,
}

pub(crate) fn assert_success(assertion: &SuccessAssertion<'_>) -> TestResult {
    match assertion.output {
        SuccessOutput::Stdout { expected } => assert_stdout_fixture(assertion, expected),
        SuccessOutput::StdoutTargetUnchanged { expected } => {
            assert_stdout_target_unchanged(assertion, expected)
        }
        SuccessOutput::InPlaceSuccess { expected } => assert_in_place_success(assertion, expected),
        SuccessOutput::OriginalTargetStdout => {
            assert_stdout(assertion.name, assertion.outcome, assertion.original_target)
        }
    }
}

fn assert_stdout_fixture(
    assertion: &SuccessAssertion<'_>,
    expected: super::FixturePath,
) -> TestResult {
    let expected_body = expected_output(expected)?;
    assert_stdout(assertion.name, assertion.outcome, &expected_body)
}

fn assert_stdout_target_unchanged(
    assertion: &SuccessAssertion<'_>,
    expected: super::FixturePath,
) -> TestResult {
    assert_stdout_fixture(assertion, expected)?;
    assert_target(
        assertion.name,
        assertion.workspace,
        assertion.original_target,
    )
}

fn assert_in_place_success(
    assertion: &SuccessAssertion<'_>,
    expected: super::FixturePath,
) -> TestResult {
    let expected_body = expected_output(expected)?;
    assert_no_stdout(assertion.name, assertion.outcome)?;
    assert_written_path(assertion.name, assertion.workspace, assertion.outcome)?;
    assert_target(assertion.name, assertion.workspace, &expected_body)
}

pub(crate) fn assert_stdout(name: &str, outcome: &RunOutcome, expected: &str) -> TestResult {
    outcome.stdout.as_deref().map_or_else(
        || Err(format!("golden fixture `{name}` emitted no stdout").into()),
        |actual| compare_text(name, "stdout", actual, expected),
    )
}

fn assert_no_stdout(name: &str, outcome: &RunOutcome) -> TestResult {
    outcome.stdout.as_deref().map_or_else(
        || Ok(()),
        |actual| {
            Err(
                format!("golden fixture `{name}` emitted stdout in in-place mode:\n{actual}")
                    .into(),
            )
        },
    )
}

fn assert_written_path(
    name: &str,
    workspace: &GoldenWorkspace,
    outcome: &RunOutcome,
) -> TestResult {
    match outcome.written_path.as_ref() {
        Some(path) if path == &workspace.target => Ok(()),
        Some(path) => Err(format!(
            "golden fixture `{name}` wrote unexpected path `{path}` instead of `{}`",
            workspace.target
        )
        .into()),
        None => Err(format!("golden fixture `{name}` returned no written path").into()),
    }
}

fn assert_target(name: &str, workspace: &GoldenWorkspace, expected: &str) -> TestResult {
    let actual = workspace.target_contents()?;
    compare_text(name, "target", &actual, expected)
}

fn compare_text(name: &str, label: &str, actual: &str, expected: &str) -> TestResult {
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "golden fixture `{name}` {label} differed\nexpected:\n{expected}\nactual:\n{actual}"
        )
        .into())
    }
}

pub(crate) fn assert_expected_error(
    name: &str,
    actual: MapspliceError,
    expected: ExpectedError,
) -> TestResult {
    match (actual, expected) {
        (MapspliceError::DanglingDependency { .. }, ExpectedError::DanglingDependency)
        | (MapspliceError::LevelMismatch { .. }, ExpectedError::LevelMismatch)
        | (MapspliceError::AnchorNotFound { .. }, ExpectedError::MissingAnchor) => Ok(()),
        (unexpected, expected_error) => Err(format!(
            "golden fixture `{name}` expected error {expected_error:?} but got `{unexpected}`"
        )
        .into()),
    }
}

pub(crate) fn assert_failure_output(
    name: &str,
    workspace: &GoldenWorkspace,
    original_target: &str,
    output: FailureOutput,
) -> TestResult {
    match output {
        FailureOutput::TargetUnchanged | FailureOutput::InPlaceTargetUnchanged => {
            assert_target(name, workspace, original_target)
        }
    }
}
