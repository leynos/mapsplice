//! Shared golden-fixture harness for roadmap integration tests.

#[cfg(test)]
mod metadata_tests;

use std::{error::Error, ffi::OsString};

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use mapsplice::{MapspliceError, RunOutcome, run_from_args};
use tempfile::TempDir;

pub(crate) type TestResult<T = ()> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub(crate) struct GoldenWorkspace {
    dir: Dir,
    fragment: Utf8PathBuf,
    target: Utf8PathBuf,
    _tempdir: TempDir,
}

impl GoldenWorkspace {
    fn write_target(&self, contents: &str) -> TestResult {
        self.dir.write("target.md", contents)?;
        Ok(())
    }

    fn write_fragment(&self, contents: &str) -> TestResult {
        self.dir.write("fragment.md", contents)?;
        Ok(())
    }

    fn target_contents(&self) -> TestResult<String> { Ok(self.dir.read_to_string("target.md")?) }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GoldenCase {
    name: &'static str,
    command: GoldenCommand,
    target: FixturePath,
    fragment: Option<FixturePath>,
    expectation: GoldenExpectation,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum GoldenCommand {
    Append,
    InsertBefore { anchor: &'static str },
    InsertAfter { anchor: &'static str },
    Delete { anchor: &'static str },
    Replace { anchor: &'static str },
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum GoldenExpectation {
    Success {
        expected: FixturePath,
        output: SuccessOutput,
    },
    Failure {
        error: ExpectedError,
        output: FailureOutput,
    },
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum SuccessOutput {
    Stdout,
    StdoutTargetUnchanged,
    InPlaceSuccess,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum FailureOutput {
    TargetUnchanged,
    InPlaceTargetUnchanged,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ExpectedError {
    DanglingDependency,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum FixturePath {
    Reference {
        name: &'static str,
        kind: &'static str,
    },
    Golden {
        case: &'static str,
        file: &'static str,
    },
}

pub(crate) fn create_workspace() -> TestResult<GoldenWorkspace> {
    let tempdir = tempfile::tempdir()?;
    let root = Utf8PathBuf::from_path_buf(tempdir.path().to_path_buf())
        .map_err(|path| format!("temporary directory is not valid UTF-8: {}", path.display()))?;
    let dir = Dir::open_ambient_dir(&root, ambient_authority())?;
    Ok(GoldenWorkspace {
        dir,
        fragment: root.join("fragment.md"),
        target: root.join("target.md"),
        _tempdir: tempdir,
    })
}

pub(crate) const fn reference_delete_case(name: &'static str) -> GoldenCase {
    GoldenCase {
        name,
        command: GoldenCommand::Delete { anchor: "1" },
        target: FixturePath::Reference {
            name,
            kind: "input",
        },
        fragment: None,
        expectation: GoldenExpectation::Success {
            expected: FixturePath::Reference {
                name,
                kind: "expected",
            },
            output: SuccessOutput::Stdout,
        },
    }
}

pub(crate) fn assert_golden_case(workspace: &GoldenWorkspace, case: GoldenCase) -> TestResult {
    let target = read_fixture(case.target)?;
    workspace.write_target(&target)?;
    if let Some(fragment) = case.fragment {
        workspace.write_fragment(&read_fixture(fragment)?)?;
    }

    let is_in_place = case.expectation.is_in_place();
    let run_result = run_from_args(command_args(workspace, case.command, is_in_place));

    match case.expectation {
        GoldenExpectation::Success { expected, output } => {
            let outcome = run_result?;
            assert_success(&SuccessAssertion {
                name: case.name,
                workspace,
                original_target: &target,
                expected,
                output,
                outcome: &outcome,
            })
        }
        GoldenExpectation::Failure {
            error: expected,
            output,
        } => match run_result {
            Ok(success) => Err(format!(
                "golden fixture `{}` succeeded unexpectedly: {success:?}",
                case.name
            )
            .into()),
            Err(actual) => {
                assert_expected_error(case.name, actual, expected)?;
                assert_failure_output(case.name, workspace, &target, output)
            }
        },
    }
}

struct SuccessAssertion<'a> {
    name: &'a str,
    workspace: &'a GoldenWorkspace,
    original_target: &'a str,
    expected: FixturePath,
    output: SuccessOutput,
    outcome: &'a RunOutcome,
}

fn assert_success(assertion: &SuccessAssertion<'_>) -> TestResult {
    let expected_body = expected_output(assertion.expected)?;
    match assertion.output {
        SuccessOutput::Stdout => assert_stdout(assertion.name, assertion.outcome, &expected_body),
        SuccessOutput::StdoutTargetUnchanged => {
            assert_stdout(assertion.name, assertion.outcome, &expected_body)?;
            assert_target(
                assertion.name,
                assertion.workspace,
                assertion.original_target,
            )
        }
        SuccessOutput::InPlaceSuccess => {
            assert_no_stdout(assertion.name, assertion.outcome)?;
            assert_written_path(assertion.name, assertion.workspace, assertion.outcome)?;
            assert_target(assertion.name, assertion.workspace, &expected_body)
        }
    }
}

fn assert_stdout(name: &str, outcome: &RunOutcome, expected: &str) -> TestResult {
    match outcome.stdout.as_deref() {
        Some(actual) if actual == expected => Ok(()),
        Some(actual) => Err(format!(
            "golden fixture `{name}` differed\nexpected:\n{expected}\nactual:\n{actual}"
        )
        .into()),
        None => Err(format!("golden fixture `{name}` emitted no stdout").into()),
    }
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
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "golden fixture `{name}` target differed\nexpected:\n{expected}\nactual:\n{actual}"
        )
        .into())
    }
}

fn assert_expected_error(
    name: &str,
    actual: MapspliceError,
    expected: ExpectedError,
) -> TestResult {
    match (actual, expected) {
        (MapspliceError::DanglingDependency { .. }, ExpectedError::DanglingDependency) => Ok(()),
        (unexpected, _) => {
            Err(format!("golden fixture `{name}` returned unexpected error `{unexpected}`").into())
        }
    }
}

fn assert_failure_output(
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

fn command_args(
    workspace: &GoldenWorkspace,
    command: GoldenCommand,
    is_in_place: bool,
) -> Vec<OsString> {
    let mut args = vec![OsString::from("mapsplice")];
    if is_in_place {
        args.push(OsString::from("--in-place"));
    }

    match command {
        GoldenCommand::Append => {
            args.push(OsString::from("append"));
            args.push(workspace.target.as_os_str().to_owned());
            args.push(workspace.fragment.as_os_str().to_owned());
        }
        GoldenCommand::InsertBefore { anchor } => {
            args.push(OsString::from("insert"));
            args.push(workspace.target.as_os_str().to_owned());
            args.push(OsString::from(anchor));
            args.push(workspace.fragment.as_os_str().to_owned());
        }
        GoldenCommand::InsertAfter { anchor } => {
            args.push(OsString::from("insert"));
            args.push(OsString::from("--after"));
            args.push(workspace.target.as_os_str().to_owned());
            args.push(OsString::from(anchor));
            args.push(workspace.fragment.as_os_str().to_owned());
        }
        GoldenCommand::Delete { anchor } => {
            args.push(OsString::from("delete"));
            args.push(workspace.target.as_os_str().to_owned());
            args.push(OsString::from(anchor));
        }
        GoldenCommand::Replace { anchor } => {
            args.push(OsString::from("replace"));
            args.push(workspace.target.as_os_str().to_owned());
            args.push(OsString::from(anchor));
            args.push(workspace.fragment.as_os_str().to_owned());
        }
    }
    args
}

impl GoldenExpectation {
    const fn is_in_place(self) -> bool {
        matches!(
            self,
            Self::Success {
                output: SuccessOutput::InPlaceSuccess,
                ..
            } | Self::Failure {
                output: FailureOutput::InPlaceTargetUnchanged,
                ..
            }
        )
    }
}

fn read_fixture(path: FixturePath) -> TestResult<String> {
    let project = Dir::open_ambient_dir(env!("CARGO_MANIFEST_DIR"), ambient_authority())?;
    Ok(project.read_to_string(fixture_path(path))?)
}

fn expected_output(path: FixturePath) -> TestResult<String> {
    let mut expected = read_fixture(path)?;
    if expected.ends_with('\n') {
        expected.pop();
    }
    Ok(expected)
}

fn fixture_path(path: FixturePath) -> Utf8PathBuf {
    match path {
        FixturePath::Reference { name, kind } => Utf8PathBuf::from("tests")
            .join("fixtures")
            .join("reference_rewrite")
            .join(format!("{name}.{kind}.md")),
        FixturePath::Golden { case, file } => Utf8PathBuf::from("tests")
            .join("fixtures")
            .join("golden")
            .join(case)
            .join(file),
    }
}
