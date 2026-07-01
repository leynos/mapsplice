//! Golden fixture coverage for roadmap reference rewriting.

use std::error::Error;

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use mapsplice::run_from_args;
use rstest::{fixture, rstest};
use tempfile::TempDir;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
struct GoldenWorkspace {
    dir: Dir,
    target: Utf8PathBuf,
    _tempdir: TempDir,
}

impl GoldenWorkspace {
    fn write_target(&self, contents: &str) -> TestResult {
        self.dir.write("target.md", contents)?;
        Ok(())
    }
}

#[fixture]
fn workspace() -> TestResult<GoldenWorkspace> {
    let tempdir = tempfile::tempdir()?;
    let root = Utf8PathBuf::from_path_buf(tempdir.path().to_path_buf())
        .map_err(|path| format!("temporary directory is not valid UTF-8: {}", path.display()))?;
    let dir = Dir::open_ambient_dir(&root, ambient_authority())?;
    Ok(GoldenWorkspace {
        dir,
        target: root.join("target.md"),
        _tempdir: tempdir,
    })
}

#[rstest]
#[serial_test::serial(cli_env)]
fn section_reference(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_delete_case(&workspace?, "section_reference")
}

#[rstest]
#[serial_test::serial(cli_env)]
fn version_quantity(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_delete_case(&workspace?, "version_quantity")
}

#[rstest]
#[serial_test::serial(cli_env)]
fn section_reference_outside_requires(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_delete_case(&workspace?, "section_reference_outside_requires")
}

#[rstest]
#[serial_test::serial(cli_env)]
fn substring_non_match(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_delete_case(&workspace?, "substring_non_match")
}

#[rstest]
#[serial_test::serial(cli_env)]
fn multi_id_requires(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_delete_case(&workspace?, "multi_id_requires")
}

fn assert_golden_delete_case(workspace: &GoldenWorkspace, name: &str) -> TestResult {
    let input = read_reference_fixture(name, "input")?;
    let expected = expected_output(name)?;
    workspace.write_target(&input)?;

    let outcome = run_from_args(["mapsplice", "delete", workspace.target.as_str(), "1"])?;

    match outcome.stdout.as_deref() {
        Some(actual) if actual == expected => Ok(()),
        Some(actual) => Err(format!(
            "golden fixture `{name}` differed\nexpected:\n{expected}\nactual:\n{actual}"
        )
        .into()),
        None => Err(format!("golden fixture `{name}` emitted no stdout").into()),
    }
}

fn read_reference_fixture(name: &str, kind: &str) -> TestResult<String> {
    let project = Dir::open_ambient_dir(env!("CARGO_MANIFEST_DIR"), ambient_authority())?;
    let path = reference_fixture_path(name, kind);
    Ok(project.read_to_string(path)?)
}

fn expected_output(name: &str) -> TestResult<String> {
    let mut expected = read_reference_fixture(name, "expected")?;
    if expected.ends_with('\n') {
        expected.pop();
    }
    Ok(expected)
}

fn reference_fixture_path(name: &str, kind: &str) -> Utf8PathBuf {
    Utf8PathBuf::from("tests")
        .join("fixtures")
        .join("reference_rewrite")
        .join(format!("{name}.{kind}.md"))
}
