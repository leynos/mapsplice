//! Fake Markdown formatter executables for Make target integration tests.

use std::{
    error::Error,
    process::{Command, Output},
};

use camino::Utf8PathBuf;
use cap_std::{
    ambient_authority,
    fs::{Permissions, PermissionsExt},
    fs_utf8::Dir,
};
use tempfile::TempDir;

/// A temporary PATH prefix with controlled formatter version output.
pub(super) struct FakeMarkdownTools {
    /// Keeps the fake executables alive for the duration of each test.
    _directory: TempDir,
    /// Supplies the directory prepended to child commands' `PATH`.
    path: Utf8PathBuf,
}

impl FakeMarkdownTools {
    /// Creates fake tools with the supplied versions.
    ///
    /// For example, `new("mdtablefix 0.6.1", "0.22.1")` writes that
    /// `markdownlint-cli2` version to a fake npm package manifest.
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary directory or any fake tool file
    /// cannot be created.
    pub(super) fn new(
        mdtablefix_output: &str,
        markdownlint_version: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let (directory, path) = utf8_temp_dir()?;
        let handle = Dir::open_ambient_dir(&path, ambient_authority())?;
        write_fake_tools(&handle, mdtablefix_output, markdownlint_version)?;
        Ok(Self {
            _directory: directory,
            path,
        })
    }

    /// Runs the named Make preflight with fake tools first on the child PATH.
    ///
    /// For example, `make_check("0.6.1", "0.22.1")` passes those required
    /// versions to the child process without changing the test process.
    ///
    /// # Errors
    ///
    /// Returns an error if the child `PATH` cannot be constructed or `make`
    /// cannot be launched.
    pub(super) fn make_check(
        &self,
        mdtablefix_version: &str,
        markdownlint_version: &str,
    ) -> Result<Output, Box<dyn Error>> {
        let original_path = std::env::var_os("PATH").unwrap_or_default();
        let path_entries = std::iter::once(self.path.as_std_path().to_path_buf())
            .chain(std::env::split_paths(&original_path));
        let path = std::env::join_paths(path_entries)
            .map_err(|error| format!("prepend fake formatters to PATH: {error}"))?;
        Ok(Command::new("make")
            .args(["--no-print-directory", "check-markdown-tools"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .env("PATH", path)
            .env("MDTABLEFIX_REQUIRED_VERSION", mdtablefix_version)
            .env("MARKDOWNLINT_CLI2_REQUIRED_VERSION", markdownlint_version)
            .output()?)
    }
}

/// Write an executable stub that prints the version text stored beside it.
fn write_fake_tool(handle: &Dir, name: &str, version_output: &str) -> Result<(), Box<dyn Error>> {
    handle.write(format!("{name}.version"), version_output)?;
    handle.write(name, "#!/bin/sh\ncat \"$0.version\"\n")?;
    handle.set_permissions(name, Permissions::from_mode(0o755))?;
    Ok(())
}

/// Create a temporary directory together with its UTF-8 path.
fn utf8_temp_dir() -> Result<(TempDir, Utf8PathBuf), Box<dyn Error>> {
    let directory = TempDir::new()?;
    let path = Utf8PathBuf::from_path_buf(directory.path().to_path_buf())
        .map_err(|path| format!("temporary directory is not UTF-8: {}", path.display()))?;
    Ok((directory, path))
}

/// Write both fake formatters and the npm manifest that reports the linter's version.
fn write_fake_tools(
    handle: &Dir,
    mdtablefix_output: &str,
    markdownlint_version: &str,
) -> Result<(), Box<dyn Error>> {
    handle.write(
        "package.json",
        format!("{{\"name\":\"markdownlint-cli2\",\"version\":\"{markdownlint_version}\"}}"),
    )?;
    write_fake_tool(handle, "mdtablefix", mdtablefix_output)?;
    write_fake_tool(handle, "markdownlint-cli2", "markdownlint-cli2 stub")
}
