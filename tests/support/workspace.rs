//! Shared temporary workspace support for integration tests.

use std::error::Error;

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use tempfile::TempDir;

/// Fallible result type used by integration-test workspace helpers.
pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

/// Temporary capability-scoped workspace for integration tests.
#[derive(Debug)]
pub struct Workspace {
    /// Capability-scoped directory rooted at the temporary workspace.
    pub dir: Dir,
    /// UTF-8 path to the `target.md` file inside the workspace.
    pub target: Utf8PathBuf,
    /// UTF-8 path to the `fragment.md` file inside the workspace.
    pub fragment: Utf8PathBuf,
    _tempdir: TempDir,
}

impl Workspace {
    /// Write Markdown contents to `target.md`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let workspace = create_workspace()?;
    /// workspace.write_target("# Roadmap\n")?;
    /// ```
    pub fn write_target(&self, contents: &str) -> TestResult {
        self.dir.write("target.md", contents)?;
        Ok(())
    }

    /// Write Markdown contents to `fragment.md`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let workspace = create_workspace()?;
    /// workspace.write_fragment("- [ ] 1.1.1. Added task.\n")?;
    /// ```
    pub fn write_fragment(&self, contents: &str) -> TestResult {
        self.dir.write("fragment.md", contents)?;
        Ok(())
    }
}

/// Create a temporary workspace with capability-scoped file access.
///
/// The returned workspace owns a temporary directory, opens it as a
/// `cap_std::fs_utf8::Dir`, and exposes UTF-8 paths for `target.md` and
/// `fragment.md`.
///
/// # Examples
///
/// ```ignore
/// let workspace = create_workspace()?;
/// assert!(workspace.target.ends_with("target.md"));
/// assert!(workspace.fragment.ends_with("fragment.md"));
/// ```
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
