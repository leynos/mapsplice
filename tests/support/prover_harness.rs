//! Shared fixtures for the Verus harness contract tests.

use std::{
    error::Error,
    process::{Command, Output},
};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{
    ambient_authority,
    fs::{Permissions, PermissionsExt},
    fs_utf8::Dir,
};
use tempfile::TempDir;

/// Test-local result alias, matching the surrounding integration tests.
pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

/// The repository root, derived from the manifest directory.
#[must_use]
pub fn manifest_dir() -> Utf8PathBuf { Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")) }

/// Reinterpret a temporary-directory path as UTF-8.
///
/// # Errors
///
/// Returns an error when the path is not valid UTF-8.
pub fn utf8(path: &std::path::Path) -> TestResult<&Utf8Path> {
    Utf8Path::from_path(path).ok_or_else(|| "temporary directory path should be UTF-8".into())
}

/// Open a directory as a capability-scoped handle.
///
/// # Errors
///
/// Returns an error when the directory cannot be opened.
pub fn open_dir(dir: &Utf8Path) -> TestResult<Dir> {
    Dir::open_ambient_dir(dir, ambient_authority())
        .map_err(|error| format!("open directory {dir}: {error}").into())
}

/// Read a fixture shipped beside the tests.
///
/// # Errors
///
/// Returns an error when the fixture is absent or unreadable.
pub fn fixture_text(name: &str, extension: &str) -> TestResult<String> {
    let path = format!("tests/data/verification_ledger/{name}.{extension}");
    let root = manifest_dir();
    open_dir(&root)?
        .read_to_string(&path)
        .map_err(|error| format!("read verification-ledger fixture {path}: {error}").into())
}

/// Read a repository file through a capability-scoped handle.
///
/// The workspace denies `std::fs` access that bypasses the capability policy,
/// so contract tests read the files they assert against the same way the rest
/// of the suite does. `path` is relative to the repository root.
///
/// # Errors
///
/// Returns an error when the file is absent or unreadable.
pub fn read_repo_file(path: &str) -> TestResult<String> {
    open_dir(&manifest_dir())?
        .read_to_string(path)
        .map_err(|error| format!("read {path}: {error}").into())
}

/// Path to the verification-ledger checker under test.
#[must_use]
pub fn ledger_script_path() -> Utf8PathBuf {
    manifest_dir().join("scripts/check-verification-ledger.sh")
}

/// Run the ledger checker against a scratch tree and capture its output.
///
/// `RG` is cleared so the script resolves ripgrep from `PATH` rather than
/// inheriting an override from the developer's shell.
///
/// # Errors
///
/// Returns an error when the script cannot be executed.
pub fn run_ledger_check(directory: &Utf8Path) -> TestResult<Output> {
    Command::new(ledger_script_path())
        .arg(directory)
        .env_remove("RG")
        .output()
        .map_err(|error| format!("execute check-verification-ledger.sh: {error}").into())
}

/// A fake `prover-tools` runner plus the log it writes.
pub struct FakeProverTools {
    _directory: TempDir,
    path: Utf8PathBuf,
    log_path: Utf8PathBuf,
    smoke_mode: &'static str,
}

impl FakeProverTools {
    /// Return the recorded invocations, one per line.
    ///
    /// # Errors
    ///
    /// Returns an error when the log cannot be read.
    pub fn log(&self) -> TestResult<String> {
        let root = self
            .path
            .parent()
            .ok_or("fake prover-tools path has no parent")?;
        open_dir(root)?
            .read_to_string("prover-tools.log")
            .map_err(|error| format!("read fake prover-tools log: {error}").into())
    }

    /// Build a `make` invocation of `target` against this fake runner.
    #[must_use]
    pub fn make(&self, target: &str) -> Command {
        let mut command = Command::new("make");
        command
            .arg("--no-print-directory")
            .arg(target)
            .current_dir(manifest_dir())
            .env("PROVER_TOOLS", &self.path)
            .env(
                "VERUS_RUN",
                format!("{} verus run --repo-root .", self.path),
            )
            .env("FAKE_PROVER_TOOLS_LOG", &self.log_path)
            .env("FAKE_PROVER_TOOLS_SMOKE_MODE", self.smoke_mode);
        command
    }
}

/// Create a fake `prover-tools` runner that logs its arguments.
///
/// `smoke_mode` selects how it answers the smoke-proof invocation:
///
/// - `rejected`: print the runner's failure banner and exit 1, as the real runner does when Verus
///   rejects a proof.
/// - `accepted`: exit 0, standing in for a verifier that proved the falsity.
/// - `unrelated_failure`: exit 1 with output that is not the runner's banner, standing in for a
///   missing install or a crashed verifier.
///
/// # Errors
///
/// Returns an error when the scratch directory cannot be populated.
pub fn fake_prover_tools(smoke_mode: &'static str) -> TestResult<FakeProverTools> {
    let directory = TempDir::new().map_err(|error| format!("create scratch: {error}"))?;
    let root = utf8(directory.path())?;
    let handle = open_dir(root)?;
    let path = root.join("prover-tools");
    let log_path = root.join("prover-tools.log");
    let script = r#"#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "${FAKE_PROVER_TOOLS_LOG:?}"
if [[ "$*" == "verus run --repo-root . --proof-file verus/smoke.rs" ]]; then
    case "${FAKE_PROVER_TOOLS_SMOKE_MODE:?}" in
        rejected)
            echo "Verus proofs failed (exit 1)."
            exit 1
            ;;
        accepted) exit 0 ;;
        unrelated_failure)
            echo "runner unavailable"
            exit 1
            ;;
    esac
fi
"#;
    handle
        .write("prover-tools", script)
        .map_err(|error| format!("write fake prover-tools runner: {error}"))?;
    handle
        .set_permissions("prover-tools", Permissions::from_mode(0o755))
        .map_err(|error| format!("make fake prover-tools runner executable: {error}"))?;
    Ok(FakeProverTools {
        _directory: directory,
        path,
        log_path,
        smoke_mode,
    })
}
