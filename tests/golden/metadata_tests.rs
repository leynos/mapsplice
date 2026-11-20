//! Metadata self-tests for the golden-fixture harness.

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};

use super::{
    ExpectedError,
    FailureOutput,
    FixtureKind,
    FixturePath,
    GoldenCommand,
    GoldenExpectation,
    GoldenWorkspace,
    SuccessOutput,
    command_args,
    expected_output,
};

#[test]
fn command_metadata_covers_supported_shapes() {
    let workspace = metadata_workspace();

    assert_structural_command_args(&workspace);
    assert_complete_command_args(&workspace);
}

/// Build the workspace paths the argument builder only reads.
fn metadata_workspace() -> GoldenWorkspace {
    GoldenWorkspace {
        dir: Dir::open_ambient_dir(env!("CARGO_MANIFEST_DIR"), ambient_authority())
            .expect("project directory should be readable"),
        fragment: Utf8PathBuf::from("/tmp/fragment.md"),
        target: Utf8PathBuf::from("/tmp/target.md"),
        _tempdir: tempfile::tempdir().expect("temporary directory should be created"),
    }
}

/// Check the fragment-based and single-anchor structural command arguments.
fn assert_structural_command_args(workspace: &GoldenWorkspace) {
    assert_eq!(
        command_arg_strings(workspace, GoldenCommand::Append, false),
        vec!["mapsplice", "append", "/tmp/target.md", "/tmp/fragment.md"],
        "append arguments should name target then fragment"
    );
    assert_eq!(
        command_arg_strings(
            workspace,
            GoldenCommand::InsertBefore { anchor: "2" },
            false,
        ),
        vec![
            "mapsplice",
            "insert",
            "/tmp/target.md",
            "2",
            "/tmp/fragment.md",
        ],
        "insert-before arguments should name target, anchor, then fragment"
    );
    assert_eq!(
        command_arg_strings(
            workspace,
            GoldenCommand::InsertAfter { anchor: "2.1" },
            false
        ),
        vec![
            "mapsplice",
            "insert",
            "--after",
            "/tmp/target.md",
            "2.1",
            "/tmp/fragment.md",
        ],
        "insert-after arguments should pass --after before the target"
    );
    assert_eq!(
        command_arg_strings(workspace, GoldenCommand::Delete { anchor: "2.1.3" }, false),
        vec!["mapsplice", "delete", "/tmp/target.md", "2.1.3"],
        "delete arguments should name target then anchor"
    );
    assert_eq!(
        command_arg_strings(workspace, GoldenCommand::Replace { anchor: "2.1.3" }, true),
        vec![
            "mapsplice",
            "--in-place",
            "replace",
            "/tmp/target.md",
            "2.1.3",
            "/tmp/fragment.md",
        ],
        "in-place replace arguments should pass --in-place first"
    );
}

/// Check the completion arguments with and without a configured suffix.
fn assert_complete_command_args(workspace: &GoldenWorkspace) {
    assert_eq!(
        command_arg_strings(
            workspace,
            GoldenCommand::Complete {
                anchor: "2.1.3",
                suffix: None,
            },
            false,
        ),
        vec!["mapsplice", "complete", "/tmp/target.md", "2.1.3"],
        "complete arguments without a suffix should omit --suffix"
    );
    assert_eq!(
        command_arg_strings(
            workspace,
            GoldenCommand::Complete {
                anchor: "2.1.3.1",
                suffix: Some("— done"),
            },
            true,
        ),
        vec![
            "mapsplice",
            "--in-place",
            "complete",
            "--suffix",
            "— done",
            "/tmp/target.md",
            "2.1.3.1",
        ],
        "complete arguments should pass the configured suffix before the target"
    );
}

#[test]
fn expectation_metadata_covers_output_modes_and_fixture_shapes() {
    let expectations = [
        GoldenExpectation::Success {
            output: SuccessOutput::Stdout {
                expected: FixturePath::Golden {
                    case: "case_name",
                    file: "expected.md",
                },
            },
        },
        GoldenExpectation::Success {
            output: SuccessOutput::StdoutTargetUnchanged {
                expected: FixturePath::Golden {
                    case: "case_name",
                    file: "expected.md",
                },
            },
        },
        GoldenExpectation::Success {
            output: SuccessOutput::InPlaceSuccess {
                expected: FixturePath::Golden {
                    case: "case_name",
                    file: "expected.md",
                },
            },
        },
        GoldenExpectation::Success {
            output: SuccessOutput::InPlaceIdempotent {
                expected: FixturePath::Golden {
                    case: "case_name",
                    file: "expected.md",
                },
            },
        },
        GoldenExpectation::Success {
            output: SuccessOutput::OriginalTargetStdout,
        },
        GoldenExpectation::Failure {
            error: ExpectedError::InvalidRoadmap,
            output: FailureOutput::TargetUnchanged,
        },
        GoldenExpectation::Failure {
            error: ExpectedError::DanglingDependency,
            output: FailureOutput::TargetUnchanged,
        },
        GoldenExpectation::Failure {
            error: ExpectedError::LevelMismatch,
            output: FailureOutput::TargetUnchanged,
        },
        GoldenExpectation::Failure {
            error: ExpectedError::MissingAnchor,
            output: FailureOutput::InPlaceTargetUnchanged,
        },
        GoldenExpectation::Failure {
            error: ExpectedError::DanglingDependency,
            output: FailureOutput::InPlaceTargetUnchanged,
        },
    ];

    let in_place_count = expectations
        .iter()
        .copied()
        .filter(|expectation| expectation.is_in_place())
        .count();

    assert_eq!(
        in_place_count, 4,
        "the expectation matrix must contain four in-place cases"
    );
}

#[test]
fn expected_output_keeps_final_newline() {
    let fixture = expected_output(FixturePath::Reference {
        name: "section_reference",
        kind: FixtureKind::Expected,
    })
    .expect("fixture should be readable");

    assert!(fixture.ends_with('\n'));
}

#[test]
fn markdownlint_config_ignores_golden_fixture_tree() {
    let project = Dir::open_ambient_dir(env!("CARGO_MANIFEST_DIR"), ambient_authority())
        .expect("project directory should be readable");
    let config = project
        .read_to_string(".markdownlint-cli2.jsonc")
        .expect("markdownlint configuration should be readable");

    assert!(
        config.contains("\"tests/fixtures/golden/**/*.md\""),
        "rendered output gate must not rely on direct fixture linting"
    );
}

fn command_arg_strings(
    workspace: &GoldenWorkspace,
    command: GoldenCommand,
    is_in_place: bool,
) -> Vec<String> {
    command_args(workspace, command, is_in_place)
        .into_iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}
