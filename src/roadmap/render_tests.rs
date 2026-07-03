//! Unit tests for roadmap render fidelity.

use std::process::Command;

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8::Dir};
use rstest::rstest;
use tempfile::tempdir;

use super::{render_roadmap, text::escape_markdown};
use crate::{
    error::MapspliceError,
    roadmap::{model::TaskEntry, parse_roadmap},
};

#[test]
fn exact_nested_sub_task_round_trip() {
    let source = concat!(
        "# Example\n\n",
        "## 8. Phase one\n\n",
        "### 8.2. Step one\n\n",
        "- [ ] 8.2.3. Parent task.\n",
        "    Body before.\n",
        "  - [ ] 8.2.3.1. Nested sub-task.\n",
        "    Body after.\n",
    );
    let roadmap = parse_roadmap(source).expect("nested sub-task roadmap should parse");

    let rendered = render_roadmap(&roadmap).expect("nested sub-task roadmap should render");

    assert_eq!(rendered, source);
}

#[test]
fn non_empty_roadmap_ends_in_exactly_one_final_newline() {
    let source = concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. First task."
    );
    let roadmap = parse_roadmap(source).expect("roadmap should parse");

    let rendered = render_roadmap(&roadmap).expect("roadmap should render");

    assert!(rendered.ends_with('\n'));
    assert!(!rendered.ends_with("\n\n"));
}

#[rstest]
#[case::literal_backslash_bang("literal \\! marker", "literal \\\\\\! marker")]
#[case::bang_before_link(
    "bang ![link](https://example.com)",
    "bang \\!\\[link\\]\\(https://example.com\\)"
)]
fn literal_escape_markdown_metacharacters_survive_reparse(
    #[case] input: &str,
    #[case] expected: &str,
) {
    assert_eq!(escape_markdown(input), expected);
}

#[test]
fn literal_backslash_bang_round_trip_stays_stable() {
    let source = concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Preserve \\\\\\! marker and \\![link](https://example.com).\n",
    );
    let roadmap = parse_roadmap(source).expect("literal escape roadmap should parse");

    let rendered = render_roadmap(&roadmap).expect("literal escape roadmap should render");
    let reparsed = parse_roadmap(&rendered).expect("rendered literal escape roadmap should parse");
    let rendered_again =
        render_roadmap(&reparsed).expect("reparsed literal escape roadmap should render");

    assert_eq!(rendered, source);
    assert_eq!(rendered_again, rendered);
}

#[test]
fn render_fails_when_task_child_references_missing_sub_task() {
    let mut roadmap = parse_roadmap(concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  - [ ] 1.1.1.1. Missing sub-task.\n",
    ))
    .expect("roadmap with sub-task should parse");
    let missing_sub_task = parent_task_mut(&mut roadmap)
        .expect("roadmap should contain the parent task")
        .remove_sub_task_without_child_update_for_test(0);

    let error =
        render_roadmap(&roadmap).expect_err("orphaned sub-task child should fail rendering");

    let MapspliceError::InvalidRoadmap { message } = error else {
        panic!("expected InvalidRoadmap for orphaned sub-task child");
    };
    assert_eq!(
        message,
        format!(
            "task `1.1.1` child ordering references missing sub-task `{}`",
            missing_sub_task.number
        )
    );
}

#[test]
fn round_trip_fixture_list_covers_required_surfaces() {
    let fixture_paths = conformant_round_trip_fixture_paths()
        .expect("conformant round-trip fixture paths should be discoverable");

    assert!(!fixture_paths.is_empty());
    for required_path in REQUIRED_ROUND_TRIP_SURFACES {
        assert!(
            fixture_paths
                .iter()
                .any(|fixture_path| fixture_path == required_path),
            "missing required round-trip surface: {required_path}\npaths: {fixture_paths:#?}"
        );
    }
}

#[test]
fn noop_round_trip_property_holds_for_all_conformant_fixtures() {
    for fixture_path in conformant_round_trip_fixture_paths()
        .expect("conformant round-trip fixture paths should be discoverable")
    {
        let source = read_fixture(&fixture_path).unwrap_or_else(|error| {
            panic!("fixture should be readable as UTF-8: {fixture_path}: {error}");
        });
        let roadmap = parse_roadmap(&source).unwrap_or_else(|error| {
            panic!("fixture should parse as a conformant roadmap: {fixture_path}: {error}");
        });
        let rendered = render_roadmap(&roadmap).unwrap_or_else(|error| {
            panic!("fixture should render after parsing: {fixture_path}: {error}");
        });

        assert_eq!(
            rendered, source,
            "no-op parse/render drifted for fixture: {fixture_path}"
        );

        let rendered_roadmap = parse_roadmap(&rendered).unwrap_or_else(|error| {
            panic!("rendered fixture should parse again: {fixture_path}: {error}");
        });
        let rendered_again = render_roadmap(&rendered_roadmap).unwrap_or_else(|error| {
            panic!("rendered fixture should render again: {fixture_path}: {error}");
        });

        assert_eq!(
            rendered_again, rendered,
            "second parse/render drifted for fixture: {fixture_path}"
        );

        assert_formatter_noop(&fixture_path, &rendered).unwrap_or_else(|error| {
            panic!("house formatter changed rendered fixture {fixture_path}: {error}");
        });
    }
}

const REQUIRED_ROUND_TRIP_SURFACES: &[&str] = &[
    "tests/fixtures/golden/preamble_preserved/target.md",
    "tests/fixtures/golden/tables_preserved/target.md",
    "tests/fixtures/golden/code_blocks_preserved/target.md",
    "tests/fixtures/golden/nested_bullets/target.md",
    "tests/fixtures/golden/c4_addendum_render_fidelity/target.md",
    "tests/fixtures/golden/literal_backslash_escape/target.md",
    "tests/fixtures/reference_rewrite/multi_id_requires.input.md",
    "tests/fixtures/reference_rewrite/section_reference.input.md",
    "tests/fixtures/reference_rewrite/substring_non_match.input.md",
    "tests/fixtures/reference_rewrite/version_quantity.input.md",
];

fn conformant_round_trip_fixture_paths() -> Result<Vec<Utf8PathBuf>, String> {
    let repository = Dir::open_ambient_dir(".", ambient_authority())
        .map_err(|error| format!("open repository directory: {error}"))?;
    let mut fixture_paths = Vec::new();

    collect_named_fixture_paths(
        &repository,
        Utf8Path::new("tests/fixtures/golden"),
        &["expected.md", "target.md"],
        &mut fixture_paths,
    )?;
    collect_suffixed_fixture_paths(
        &repository,
        Utf8Path::new("tests/fixtures/reference_rewrite"),
        &[".expected.md", ".input.md"],
        &mut fixture_paths,
    )?;

    fixture_paths.retain(|fixture_path| !is_excluded_round_trip_fixture(fixture_path));
    fixture_paths.sort();

    Ok(fixture_paths)
}

fn parent_task_mut(roadmap: &mut crate::roadmap::RoadmapDocument) -> Option<&mut TaskEntry> {
    roadmap
        .phases
        .first_mut()
        .and_then(|phase| phase.steps.first_mut())
        .and_then(|step| step.tasks.first_mut())
}

fn collect_named_fixture_paths(
    repository: &Dir,
    directory: &Utf8Path,
    file_names: &[&str],
    fixture_paths: &mut Vec<Utf8PathBuf>,
) -> Result<(), String> {
    collect_fixture_paths(repository, directory, fixture_paths, |file_name| {
        file_names.contains(&file_name)
    })
}

fn collect_suffixed_fixture_paths(
    repository: &Dir,
    directory: &Utf8Path,
    suffixes: &[&str],
    fixture_paths: &mut Vec<Utf8PathBuf>,
) -> Result<(), String> {
    collect_fixture_paths(repository, directory, fixture_paths, |file_name| {
        suffixes.iter().any(|suffix| file_name.ends_with(suffix))
    })
}

fn collect_fixture_paths(
    repository: &Dir,
    directory: &Utf8Path,
    fixture_paths: &mut Vec<Utf8PathBuf>,
    should_include_file: impl Fn(&str) -> bool + Copy,
) -> Result<(), String> {
    let directory_entries = repository
        .read_dir(directory)
        .map_err(|error| format!("read fixture directory {directory}: {error}"))?;

    for entry_result in directory_entries {
        let directory_entry =
            entry_result.map_err(|error| format!("read fixture entry in {directory}: {error}"))?;
        let file_name = directory_entry
            .file_name()
            .map_err(|error| format!("read fixture file name in {directory}: {error}"))?;
        let fixture_path = directory.join(file_name.as_str());
        let entry_file_type = directory_entry
            .file_type()
            .map_err(|error| format!("read fixture file type for {fixture_path}: {error}"))?;

        if entry_file_type.is_dir() {
            collect_fixture_paths(
                repository,
                &fixture_path,
                fixture_paths,
                should_include_file,
            )?;
        } else if entry_file_type.is_file() && should_include_file(file_name.as_str()) {
            fixture_paths.push(fixture_path);
        }
    }

    Ok(())
}

fn is_excluded_round_trip_fixture(fixture_path: &Utf8Path) -> bool {
    // F5 fixtures intentionally exercise fail-closed inputs and operations,
    // so they are not part of the conformant no-op rendering corpus.
    fixture_path
        .as_str()
        .starts_with("tests/fixtures/golden/f5_")
}

fn read_fixture(fixture_path: &Utf8Path) -> Result<String, String> {
    Dir::open_ambient_dir(".", ambient_authority())
        .map_err(|error| format!("open repository directory: {error}"))?
        .read_to_string(fixture_path)
        .map_err(|error| format!("read fixture {fixture_path}: {error}"))
}

fn assert_formatter_noop(fixture_path: &Utf8Path, rendered: &str) -> Result<(), String> {
    let temporary_directory =
        tempdir().map_err(|error| format!("create temporary formatter directory: {error}"))?;
    let temporary_path = Utf8PathBuf::from_path_buf(temporary_directory.path().to_path_buf())
        .map_err(|path| {
            format!(
                "temporary formatter directory is not UTF-8: {}",
                path.display()
            )
        })?;
    let temporary_root = Dir::open_ambient_dir(&temporary_path, ambient_authority())
        .map_err(|error| format!("open temporary formatter directory: {error}"))?;
    let rendered_path = temporary_path.join("rendered.md");

    temporary_root
        .write("rendered.md", rendered)
        .map_err(|error| format!("write temporary rendered fixture {fixture_path}: {error}"))?;

    run_formatter_command(
        fixture_path,
        "mdtablefix",
        [
            "--wrap",
            "--renumber",
            "--breaks",
            "--ellipsis",
            "--fences",
            "--in-place",
        ],
        &rendered_path,
    )?;
    run_formatter_command(fixture_path, "markdownlint-cli2", ["--fix"], &rendered_path)?;

    let formatted = temporary_root
        .read_to_string("rendered.md")
        .map_err(|error| format!("read formatted fixture {fixture_path}: {error}"))?;

    if formatted == rendered {
        Ok(())
    } else {
        Err(format!(
            "formatter output differed for \
             {fixture_path}\nrendered:\n{rendered}\nformatted:\n{formatted}"
        ))
    }
}

fn run_formatter_command<const ARG_COUNT: usize>(
    fixture_path: &Utf8Path,
    command: &str,
    arguments: [&str; ARG_COUNT],
    rendered_path: &Utf8Path,
) -> Result<(), String> {
    let command_output = Command::new(command)
        .args(arguments)
        .arg(rendered_path.as_os_str())
        .output()
        .map_err(|error| format!("run {command} for {fixture_path}: {error}"))?;

    if command_output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{command} failed for {fixture_path}\n{}",
            format_command_output(&command_output)
        ))
    }
}

fn format_command_output(command_output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&command_output.stdout);
    let stderr = String::from_utf8_lossy(&command_output.stderr);

    format!(
        "status: {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        command_output.status
    )
}
