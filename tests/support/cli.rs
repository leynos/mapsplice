//! Shared fixtures for behavioural CLI tests.

use std::error::Error;

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use tempfile::TempDir;

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

pub const TARGET_TWO_PHASES: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. First task.\n\n",
    "## 2. Phase two\n\n",
    "### 2.1. Step two\n\n",
    "- [ ] 2.1.1. Second task. Requires 2.1.1.\n",
);

pub const TARGET_TWO_TASKS: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. First task.\n",
    "- [ ] 1.1.2. Second task. Depends on 1.1.1 and 1.1.2.\n",
);

pub const TARGET_THREE_PHASES: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. First task.\n\n",
    "## 2. Phase two\n\n",
    "### 2.1. Step two\n\n",
    "- [ ] 2.1.1. Middle task.\n\n",
    "## 3. Phase three\n\n",
    "### 3.1. Step three\n\n",
    "- [ ] 3.1.1. Final task. Requires 3.1.1.\n",
);

pub const PHASE_FRAGMENT: &str = concat!(
    "## 9. Inserted phase\n\n",
    "### 9.1. Added step\n\n",
    "- [ ] 9.1.1. Added task. Requires 9.1.1.\n",
);

pub const TASK_FRAGMENT: &str = "- [ ] 9.9.9. Inserted task. Requires 9.9.9.\n";

pub const REPLACEMENT_FRAGMENT: &str = concat!(
    "## 7. Replacement phase A\n\n",
    "### 7.1. Step A\n\n",
    "- [ ] 7.1.1. Replacement task A.\n\n",
    "## 8. Replacement phase B\n\n",
    "### 8.1. Step B\n\n",
    "- [ ] 8.1.1. Replacement task B. Requires 8.1.1.\n",
);

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

    pub fn read_target(&self) -> TestResult<String> { Ok(self.dir.read_to_string("target.md")?) }
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
