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
    /// Append the fragment to the target roadmap.
    Append,
    /// Insert the fragment before the addressed roadmap item.
    InsertBefore { anchor: &'static str },
    /// Insert the fragment after the addressed roadmap item.
    InsertAfter { anchor: &'static str },
    /// Delete the addressed roadmap item without using a fragment.
    Delete { anchor: &'static str },
    /// Replace the addressed roadmap item with the fragment.
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
    /// Assert command stdout equals the expected fixture.
    Stdout { expected: FixturePath },
    /// Assert command stdout equals the expected fixture and the target is unchanged.
    StdoutTargetUnchanged { expected: FixturePath },
    /// Assert in-place mode writes the expected fixture to the target with no stdout.
    InPlaceSuccess { expected: FixturePath },
    /// Assert command stdout reproduces the original target exactly.
    OriginalTargetStdout,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum FailureOutput {
    /// Assert the target remains byte-identical after a non-in-place failure.
    TargetUnchanged,
    /// Assert the target remains byte-identical after an in-place failure.
    InPlaceTargetUnchanged,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ExpectedError {
    /// Expect an invalid-roadmap diagnostic.
    InvalidRoadmap,
    /// Expect a dangling dependency diagnostic.
    DanglingDependency,
    /// Expect a source/fragment level mismatch diagnostic.
    LevelMismatch,
    /// Expect a missing-anchor diagnostic.
    MissingAnchor,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum FixturePath {
    /// Fixture under `tests/fixtures/reference_rewrite/<name>.<kind>.md`.
    Reference {
        name: &'static str,
        kind: FixtureKind,
    },
    /// Fixture under `tests/fixtures/golden/<case>/<file>`.
    Golden {
        case: &'static str,
        file: &'static str,
    },
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum FixtureKind {
    /// Expected output fixture.
    Expected,
    /// Input target fixture.
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
