//! Shared golden-fixture harness for roadmap integration tests.

mod assertions;
mod case;
#[cfg(test)]
mod metadata_tests;
mod runner;
mod workspace;

pub(crate) use assertions::{
    SuccessAssertion,
    assert_expected_error,
    assert_failure_output,
    assert_success,
};
pub(crate) use case::{
    ExpectedError,
    FailureOutput,
    FixtureKind,
    FixturePath,
    GoldenCase,
    GoldenCommand,
    GoldenExpectation,
    SuccessOutput,
    reference_delete_case,
};
pub(crate) use runner::{assert_golden_case, command_args};
pub(crate) use workspace::{
    GoldenWorkspace,
    TestResult,
    create_workspace,
    expected_output,
    read_fixture,
};
