//! Workspace and fixture-file helpers for golden roadmap tests.

use std::error::Error;

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use tempfile::TempDir;

use super::FixturePath;

pub(crate) type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const FRAGMENT_FILE: &str = "fragment.md";
const TARGET_FILE: &str = "target.md";

#[derive(Debug)]
pub(crate) struct GoldenWorkspace {
    pub(crate) dir: Dir,
    pub(crate) fragment: Utf8PathBuf,
    pub(crate) target: Utf8PathBuf,
    pub(crate) _tempdir: TempDir,
}

impl GoldenWorkspace {
    pub(crate) fn write_target(&self, contents: &str) -> TestResult {
        self.dir.write(TARGET_FILE, contents)?;
        Ok(())
    }

    pub(crate) fn write_fragment(&self, contents: &str) -> TestResult {
        self.dir.write(FRAGMENT_FILE, contents)?;
        Ok(())
    }

    pub(crate) fn target_contents(&self) -> TestResult<String> {
        Ok(self.dir.read_to_string(TARGET_FILE)?)
    }
}

/// Create a temporary `GoldenWorkspace` with target and fragment paths.
pub(crate) fn create_workspace() -> TestResult<GoldenWorkspace> {
    let tempdir = tempfile::tempdir()?;
    let root = Utf8PathBuf::from_path_buf(tempdir.path().to_path_buf())
        .map_err(|path| format!("temporary directory is not valid UTF-8: {}", path.display()))?;
    let dir = Dir::open_ambient_dir(&root, ambient_authority())?;
    Ok(GoldenWorkspace {
        dir,
        fragment: root.join(FRAGMENT_FILE),
        target: root.join(TARGET_FILE),
        _tempdir: tempdir,
    })
}

/// Read a fixture file addressed by a `FixturePath`.
pub(crate) fn read_fixture(path: FixturePath) -> TestResult<String> {
    let project = Dir::open_ambient_dir(env!("CARGO_MANIFEST_DIR"), ambient_authority())?;
    Ok(project.read_to_string(fixture_path(path))?)
}

/// Read the expected golden output text for a fixture case.
pub(crate) fn expected_output(path: FixturePath) -> TestResult<String> { read_fixture(path) }

fn fixture_path(path: FixturePath) -> Utf8PathBuf {
    match path {
        FixturePath::Reference { name, kind } => Utf8PathBuf::from("tests")
            .join("fixtures")
            .join("reference_rewrite")
            .join(format!("{}.{kind}.md", name, kind = kind.as_str())),
        FixturePath::Golden { case, file } => Utf8PathBuf::from("tests")
            .join("fixtures")
            .join("golden")
            .join(case)
            .join(file),
    }
}
