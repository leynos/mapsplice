//! Runner for golden roadmap fixture cases.
//!
//! This module loads fixtures described by [`GoldenCase`], prepares a
//! [`GoldenWorkspace`], drives `mapsplice` through `run_from_args`, and
//! dispatches outcomes to `assert_expected_error`, `assert_failure_output`, and
//! `assert_success`.

use std::ffi::OsString;

use mapsplice::{MapspliceError, RunOutcome, run_from_args};

use super::{
    FailureOutput,
    GoldenCase,
    GoldenCommand,
    GoldenExpectation,
    GoldenWorkspace,
    SuccessAssertion,
    SuccessOutput,
    TestResult,
    assert_expected_error,
    assert_failure_output,
    assert_success,
    read_fixture,
};

pub(crate) fn assert_golden_case(workspace: &GoldenWorkspace, case: GoldenCase) -> TestResult {
    ensure_required_fragment(case)?;
    let target = prepare_workspace(workspace, case)?;
    let context = RunContext {
        case,
        workspace,
        target: &target,
    };

    let is_in_place = case.expectation.is_in_place();
    let run_result = run_from_args(command_args(workspace, case.command, is_in_place));

    match case.expectation {
        GoldenExpectation::Success { output } => {
            assert_successful_case(&context, output, run_result)
        }
        GoldenExpectation::Failure {
            error: expected,
            output,
        } => assert_failed_case(&context, expected, output, run_result),
    }
}

fn ensure_required_fragment(case: GoldenCase) -> TestResult {
    let needs_fragment = matches!(
        case.command,
        GoldenCommand::Append
            | GoldenCommand::InsertBefore { .. }
            | GoldenCommand::InsertAfter { .. }
            | GoldenCommand::Replace { .. }
    );
    if needs_fragment && case.fragment.is_none() {
        Err(format!(
            "golden fixture `{}` uses {:?} but has no fragment fixture",
            case.name, case.command
        )
        .into())
    } else {
        Ok(())
    }
}

struct RunContext<'a> {
    case: GoldenCase,
    workspace: &'a GoldenWorkspace,
    target: &'a str,
}

fn prepare_workspace(workspace: &GoldenWorkspace, case: GoldenCase) -> TestResult<String> {
    let target = read_fixture(case.target)?;
    workspace.write_target(&target)?;
    if let Some(fragment) = case.fragment {
        workspace.write_fragment(&read_fixture(fragment)?)?;
    }
    Ok(target)
}

fn assert_successful_case(
    context: &RunContext<'_>,
    output: SuccessOutput,
    run_result: Result<RunOutcome, MapspliceError>,
) -> TestResult {
    let outcome = run_result.map_err(|err| {
        format!(
            "golden fixture `{}` failed unexpectedly: {err}",
            context.case.name
        )
    })?;
    assert_success(&SuccessAssertion {
        name: context.case.name,
        workspace: context.workspace,
        original_target: context.target,
        output,
        outcome: &outcome,
    })
}

fn assert_failed_case(
    context: &RunContext<'_>,
    expected: super::ExpectedError,
    output: FailureOutput,
    run_result: Result<RunOutcome, MapspliceError>,
) -> TestResult {
    match run_result {
        Ok(success) => Err(format!(
            "golden fixture `{}` succeeded unexpectedly: {success:?}",
            context.case.name
        )
        .into()),
        Err(actual) => {
            assert_expected_error(context.case.name, actual, expected)?;
            assert_failure_output(context.case.name, context.workspace, context.target, output)
        }
    }
}

pub(crate) fn command_args(
    workspace: &GoldenWorkspace,
    command: GoldenCommand,
    is_in_place: bool,
) -> Vec<OsString> {
    let mut args = vec![OsString::from("mapsplice")];
    if is_in_place {
        args.push(OsString::from("--in-place"));
    }
    let target_arg = workspace.target.as_os_str().to_owned();
    let fragment_arg = || workspace.fragment.as_os_str().to_owned();

    match command {
        GoldenCommand::Append => {
            args.push(OsString::from("append"));
            args.push(target_arg.clone());
            args.push(fragment_arg());
        }
        GoldenCommand::InsertBefore { anchor } => {
            args.push(OsString::from("insert"));
            args.push(target_arg.clone());
            args.push(OsString::from(anchor));
            args.push(fragment_arg());
        }
        GoldenCommand::InsertAfter { anchor } => {
            args.push(OsString::from("insert"));
            args.push(OsString::from("--after"));
            args.push(target_arg.clone());
            args.push(OsString::from(anchor));
            args.push(fragment_arg());
        }
        GoldenCommand::Delete { anchor } => {
            args.push(OsString::from("delete"));
            args.push(target_arg.clone());
            args.push(OsString::from(anchor));
        }
        GoldenCommand::Replace { anchor } => {
            args.push(OsString::from("replace"));
            args.push(target_arg);
            args.push(OsString::from(anchor));
            args.push(fragment_arg());
        }
    }
    args
}
