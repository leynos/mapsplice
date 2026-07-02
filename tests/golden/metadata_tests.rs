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
    let workspace = GoldenWorkspace {
        dir: Dir::open_ambient_dir(env!("CARGO_MANIFEST_DIR"), ambient_authority())
            .expect("project directory should be readable"),
        fragment: Utf8PathBuf::from("/tmp/fragment.md"),
        target: Utf8PathBuf::from("/tmp/target.md"),
        _tempdir: tempfile::tempdir().expect("temporary directory should be created"),
    };

    assert_eq!(
        command_arg_strings(&workspace, GoldenCommand::Append, false),
        vec!["mapsplice", "append", "/tmp/target.md", "/tmp/fragment.md"]
    );
    assert_eq!(
        command_arg_strings(
            &workspace,
            GoldenCommand::InsertBefore { anchor: "2" },
            false,
        ),
        vec![
            "mapsplice",
            "insert",
            "/tmp/target.md",
            "2",
            "/tmp/fragment.md",
        ]
    );
    assert_eq!(
        command_arg_strings(
            &workspace,
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
        ]
    );
    assert_eq!(
        command_arg_strings(&workspace, GoldenCommand::Delete { anchor: "2.1.3" }, false),
        vec!["mapsplice", "delete", "/tmp/target.md", "2.1.3"]
    );
    assert_eq!(
        command_arg_strings(&workspace, GoldenCommand::Replace { anchor: "2.1.3" }, true),
        vec![
            "mapsplice",
            "--in-place",
            "replace",
            "/tmp/target.md",
            "2.1.3",
            "/tmp/fragment.md",
        ]
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
            output: SuccessOutput::OriginalTargetStdout,
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

    assert_eq!(in_place_count, 3);
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
