//! Integration tests for path-scoped Markdown maintenance Make targets.

use std::{
    error::Error,
    io,
    process::{Command, ExitStatus, Output},
};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8::Dir};
use rstest::rstest;
use tempfile::TempDir;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const SAMPLE_MARKDOWN_PATHS: &str = "docs/users-guide.md docs/developers-guide.md";

#[rstest]
#[case::format_target("markdownfmt")]
fn makefile_markdown_maintenance_markdownfmt_formats_only_listed_paths(
    #[case] target: &str,
) -> TestResult {
    let output = make_markdown_dry_run(target, Some(SAMPLE_MARKDOWN_PATHS))?;
    assert_success(output.status);

    let stdout = String::from_utf8(output.stdout)?;
    assert_command_appears_before(&stdout, "mdtablefix", "markdownlint-cli2");
    assert_contains_all(
        &stdout,
        &[
            "mdtablefix",
            "--in-place",
            "docs/users-guide.md",
            "docs/developers-guide.md",
            "markdownlint-cli2",
            "--fix",
            "--no-globs",
        ],
    );
    Ok(())
}

#[test]
fn makefile_markdown_maintenance_markdownfmt_accepts_real_tool_flags() -> TestResult {
    if !command_exists("mdtablefix")? || !command_exists("markdownlint-cli2")? {
        return Ok(());
    }

    let workspace = TempDir::new()?;
    let root = utf8_temp_path(&workspace)?;
    let workspace_dir = Dir::open_ambient_dir(&root, ambient_authority())?;
    let selected_path = root.join("selected.md");

    workspace_dir.write(
        "selected.md",
        "# Selected\n\n| Name | Value |\n| - | - |\n| Alpha | Beta |\n",
    )?;

    let output = Command::new("make")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            "--always-make",
            "--no-print-directory",
            "markdownfmt",
            &format!("MARKDOWN_PATHS={selected_path}"),
        ])
        .output()?;

    assert_success(output.status);

    let selected_contents = workspace_dir.read_to_string("selected.md")?;
    assert_file_contains(&selected_path, &selected_contents, "Alpha");
    Ok(())
}

#[rstest]
#[case::lint_target("markdownlint-paths")]
fn makefile_markdown_maintenance_markdownlint_paths_lints_only_listed_paths(
    #[case] target: &str,
) -> TestResult {
    let output = make_markdown_dry_run(target, Some(SAMPLE_MARKDOWN_PATHS))?;
    assert_success(output.status);

    let stdout = String::from_utf8(output.stdout)?;
    assert_contains_all(
        &stdout,
        &[
            "markdownlint-cli2",
            "--no-globs",
            "docs/users-guide.md",
            "docs/developers-guide.md",
        ],
    );
    assert_not_contains(&stdout, "'**/*.md'");
    Ok(())
}

#[rstest]
#[case::format_target("markdownfmt")]
#[case::lint_target("markdownlint-paths")]
fn makefile_markdown_maintenance_scoped_targets_require_paths(#[case] target: &str) -> TestResult {
    let output = make_markdown_dry_run(target, None)?;

    let stderr = String::from_utf8(output.stderr)?;
    assert_failure_mentions_markdown_paths(output.status, &stderr, target);
    Ok(())
}

#[test]
fn makefile_markdown_maintenance_markdownfmt_requires_in_place_flag() -> TestResult {
    let output = make_markdown_dry_run_with_args(
        "markdownfmt",
        Some(SAMPLE_MARKDOWN_PATHS),
        &["MARKDOWN_FORMAT_FLAGS=--wrap --renumber"],
    )?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    assert_failure_mentions_in_place(output.status, &stderr);
    assert_not_contains(&stdout, "mdtablefix");
    Ok(())
}

#[test]
fn makefile_markdown_maintenance_markdownfmt_does_not_touch_unlisted_files() -> TestResult {
    let workspace = TempDir::new()?;
    let root = utf8_temp_path(&workspace)?;
    let workspace_dir = Dir::open_ambient_dir(&root, ambient_authority())?;
    let selected_path = root.join("selected.md");
    let sentinel_path = root.join("sentinel.md");
    let mdfix_wrapper = root.join("mdfix-wrapper.sh");
    let mdlint_wrapper = root.join("mdlint-wrapper.sh");

    workspace_dir.write("selected.md", "# Selected\n\n")?;
    workspace_dir.write("sentinel.md", "# Sentinel\n\n")?;
    write_markdown_wrapper(&workspace_dir, "mdfix-wrapper.sh", "mdfix")?;
    write_markdown_wrapper(&workspace_dir, "mdlint-wrapper.sh", "mdlint")?;

    let output = Command::new("make")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            "--always-make",
            "--no-print-directory",
            "markdownfmt",
            &format!("MARKDOWN_PATHS={selected_path}"),
            &format!("MDFIX=sh {mdfix_wrapper}"),
            &format!("MDLINT=sh {mdlint_wrapper}"),
        ])
        .output()?;

    assert_success(output.status);

    let selected_contents = workspace_dir.read_to_string("selected.md")?;
    assert_file_contains(&selected_path, &selected_contents, "mdfix");
    assert_file_contains(&selected_path, &selected_contents, "mdlint");

    let sentinel_contents = workspace_dir.read_to_string("sentinel.md")?;
    assert_file_equals(&sentinel_path, &sentinel_contents, "# Sentinel\n\n");
    Ok(())
}

fn make_markdown_dry_run(target: &str, markdown_paths: Option<&str>) -> TestResult<Output> {
    make_markdown_dry_run_with_args(target, markdown_paths, &[])
}

fn make_markdown_dry_run_with_args(
    target: &str,
    markdown_paths: Option<&str>,
    extra_args: &[&str],
) -> TestResult<Output> {
    let mut command = Command::new("make");
    command.current_dir(env!("CARGO_MANIFEST_DIR")).args([
        "--dry-run",
        "--always-make",
        "--no-print-directory",
        target,
    ]);

    if let Some(paths) = markdown_paths {
        command.arg(format!("MARKDOWN_PATHS={paths}"));
    }

    command.args(extra_args);

    Ok(command.output()?)
}

fn command_exists(command_name: &str) -> TestResult<bool> {
    match Command::new(command_name)
        .arg("--version")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
    {
        Ok(output) => Ok(output.status.success()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn utf8_temp_path(workspace: &TempDir) -> TestResult<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(workspace.path().to_path_buf()).map_err(|path| {
        format!("temporary directory is not valid UTF-8: {}", path.display()).into()
    })
}

fn write_markdown_wrapper(dir: &Dir, path: &str, marker: &str) -> TestResult {
    dir.write(
        path,
        format!(
            r#"for arg in "$@"; do
  case "$arg" in
    *.md)
      printf '%s\n' "{marker}" >> "$arg"
      ;;
  esac
done
"#
        ),
    )?;
    Ok(())
}

fn assert_success(status: ExitStatus) {
    assert!(status.success(), "make failed with status {status}");
}

fn assert_contains_all(output: &str, expected_values: &[&str]) {
    for expected in expected_values {
        assert!(
            output.contains(expected),
            "expected Markdown dry-run output to contain {expected:?}, got {output:?}",
        );
    }
}

fn assert_command_appears_before(output: &str, first: &str, second: &str) {
    let Some(first_index) = output.find(first) else {
        panic!("expected dry-run output to contain {first:?}: {output:?}");
    };
    let Some(second_index) = output.find(second) else {
        panic!("expected dry-run output to contain {second:?}: {output:?}");
    };

    assert!(
        first_index < second_index,
        "expected {first:?} to run before {second:?}, got {output:?}",
    );
}

fn assert_not_contains(output: &str, unexpected: &str) {
    assert!(
        !output.contains(unexpected),
        "scoped lint target must not use the repository-wide Markdown glob: {output:?}",
    );
}

fn assert_failure_mentions_markdown_paths(status: ExitStatus, stderr: &str, target: &str) {
    assert!(
        !status.success(),
        "make dry-run for {target} should fail when MARKDOWN_PATHS is unset",
    );
    assert!(
        stderr.contains("MARKDOWN_PATHS"),
        "empty MARKDOWN_PATHS diagnostic should name the variable, got {stderr:?}",
    );
}

fn assert_failure_mentions_in_place(status: ExitStatus, stderr: &str) {
    assert!(
        !status.success(),
        "make dry-run for markdownfmt should fail when --in-place is missing",
    );
    assert!(
        stderr.contains("--in-place"),
        "missing MARKDOWN_FORMAT_FLAGS diagnostic should name --in-place, got {stderr:?}",
    );
}

fn assert_file_contains(path: &Utf8Path, contents: &str, expected: &str) {
    assert!(
        contents.contains(expected),
        "expected {path} to contain {expected:?}, got {contents:?}",
    );
}

fn assert_file_equals(path: &Utf8Path, contents: &str, expected: &str) {
    assert_eq!(contents, expected, "expected {path} to remain unchanged");
}
