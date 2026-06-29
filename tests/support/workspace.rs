//! Shared temporary workspace support for integration tests.

use std::error::Error;

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use tempfile::TempDir;

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Workspace {
    pub dir: Dir,
    pub target: Utf8PathBuf,
    pub fragment: Utf8PathBuf,
    _tempdir: TempDir,
}

impl Workspace {
    pub fn write_target(&self, contents: &str) -> TestResult {
        self.dir.write("target.md", contents)?;
        Ok(())
    }

    pub fn write_fragment(&self, contents: &str) -> TestResult {
        self.dir.write("fragment.md", contents)?;
        Ok(())
    }
}

pub fn create_workspace() -> TestResult<Workspace> {
    let tempdir = tempfile::tempdir()?;
    let root = Utf8PathBuf::from_path_buf(tempdir.path().to_path_buf())
        .map_err(|path| format!("temporary directory is not valid UTF-8: {}", path.display()))?;
    let dir = Dir::open_ambient_dir(&root, ambient_authority())?;
    Ok(Workspace {
        dir,
        target: root.join("target.md"),
        fragment: root.join("fragment.md"),
        _tempdir: tempdir,
    })
}
