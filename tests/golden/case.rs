//! Typed case metadata for golden roadmap fixture tests.
//!
//! This module defines the vocabulary consumed by `runner.rs` to execute
//! `mapsplice` in a `workspace.rs` fixture directory and by `assertions.rs` to
//! validate the resulting output or typed failure.

#[derive(Clone, Copy, Debug)]
pub(crate) struct GoldenCase {
    pub(crate) name: &'static str,
    pub(crate) command: GoldenCommand,
    pub(crate) target: FixturePath,
    pub(crate) fragment: Option<FixturePath>,
    pub(crate) expectation: GoldenExpectation,
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
        output: SuccessOutput,
    },
    Failure {
        error: ExpectedError,
        output: FailureOutput,
    },
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum SuccessOutput {
    Stdout { expected: FixturePath },
    StdoutTargetUnchanged { expected: FixturePath },
    InPlaceSuccess { expected: FixturePath },
    OriginalTargetStdout,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum FailureOutput {
    TargetUnchanged,
    InPlaceTargetUnchanged,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ExpectedError {
    DanglingDependency,
    LevelMismatch,
    MissingAnchor,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum FixturePath {
    Reference {
        name: &'static str,
        kind: FixtureKind,
    },
    Golden {
        case: &'static str,
        file: &'static str,
    },
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum FixtureKind {
    Expected,
    Input,
}

impl FixtureKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Expected => "expected",
            Self::Input => "input",
        }
    }
}

impl GoldenExpectation {
    pub(crate) const fn is_in_place(self) -> bool {
        matches!(
            self,
            Self::Success {
                output: SuccessOutput::InPlaceSuccess { .. },
            } | Self::Failure {
                output: FailureOutput::InPlaceTargetUnchanged,
                ..
            }
        )
    }
}

/// Build a delete case from the named reference fixture pair.
///
/// The case name and reference fixture name intentionally share the same
/// identifier; callers must keep the matching `*.input.md` and `*.expected.md`
/// files in sync with `name`.
pub(crate) const fn reference_delete_case(name: &'static str, anchor: &'static str) -> GoldenCase {
    GoldenCase {
        name,
        command: GoldenCommand::Delete { anchor },
        target: FixturePath::Reference {
            name,
            kind: FixtureKind::Input,
        },
        fragment: None,
        expectation: GoldenExpectation::Success {
            output: SuccessOutput::Stdout {
                expected: FixturePath::Reference {
                    name,
                    kind: FixtureKind::Expected,
                },
            },
        },
    }
}
