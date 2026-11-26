//! Contract tests for `scripts/check-source-span-mutation.sh`.
//!
//! The control copies the tracked tree, mutates only the shared executable
//! kernel text, and requires the verifier to attribute the rejection to
//! `line_start_offset`. These tests drive the script with a fake runner and a
//! minimal committed fixture tree, so each outcome — accepted, unattributed,
//! startup failure, and zero-match substitution — is asserted without a real
//! Verus installation or a Cargo build.
//!
//! Failures are reported as `Err` rather than by panicking, because
//! `clippy::panic_in_result_fn` is denied across the workspace.

use std::process::{Command, Output};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::fs_utf8::Dir;
use rstest::rstest;
use tempfile::TempDir;

use crate::prover_harness::{TestResult, fake_mutation_runner, open_dir, read_repo_file, utf8};

/// A minimal kernel containing both mutation targets and a following
/// declaration, which bounds the attribution window the script computes.
const KERNEL: &str = r"//! Fixture kernel for the mutation control.

fn line_start_offset(source: &str, offset: usize) -> usize;
requires(offset <= source.spec_bytes().len());
ensures(result => result as int == spec_line_start(source.spec_bytes(), offset as int));
before {
    let bytes = source.as_bytes();
}
while (index > 0 && !(bytes[index - 1] == b'\n' || bytes[index - 1] == b'\r'))
invariant(index <= offset)
decreases(index) {
    index -= 1;
}
            after { index }

pub(crate) fn checked_line_start(source: &str, offset: usize) -> Option<usize>;
";

/// The fake Verus location the direct script runs use.
const FAKE_Z3: &str = "/tmp/source-span-mutation/.verus/z3/z3";

/// Return an error when `condition` does not hold.
fn require(condition: bool, reason: String) -> TestResult {
    if condition {
        Ok(())
    } else {
        Err(reason.into())
    }
}

/// Run one local Git command in `root`.
fn git(root: &Utf8Path, arguments: &[&str]) -> TestResult<()> {
    let status = Command::new("git")
        .args([
            "-c",
            "user.email=mutation-control@example.com",
            "-c",
            "user.name=Mutation Control",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(arguments)
        .current_dir(root)
        .status()
        .map_err(|error| format!("run git {arguments:?}: {error}"))?;
    require(
        status.success(),
        format!("git {arguments:?} must succeed, exited {status}"),
    )
}

/// Write one fixture file through the capability-scoped handle.
fn write(handle: &Dir, path: &str, contents: &str) -> TestResult<()> {
    handle
        .write(path, contents)
        .map_err(|error| format!("write {path}: {error}").into())
}

/// Create a committed fixture repository containing `kernel`.
fn mutation_repo(kernel: &str) -> TestResult<TempDir> {
    let directory = TempDir::new().map_err(|error| format!("create scratch: {error}"))?;
    let root = utf8(directory.path())?;
    let handle = open_dir(root)?;
    handle
        .create_dir("scripts")
        .map_err(|error| format!("create scripts: {error}"))?;
    handle
        .create_dir("src")
        .map_err(|error| format!("create src: {error}"))?;
    handle
        .create_dir("src/roadmap")
        .map_err(|error| format!("create src/roadmap: {error}"))?;
    handle
        .create_dir("verus")
        .map_err(|error| format!("create verus: {error}"))?;
    write(
        &handle,
        "scripts/check-source-span-mutation.sh",
        &read_repo_file("scripts/check-source-span-mutation.sh")?,
    )?;
    write(&handle, "src/roadmap/source_span_kernel.macro.rs", kernel)?;
    write(&handle, "verus/lib.rs", "fn main() {}\n")?;
    git(root, &["init", "-q"])?;
    git(root, &["add", "."])?;
    git(root, &["commit", "-q", "-m", "fixture"])?;
    Ok(directory)
}

/// A scratch log the fake runner appends its invocations to.
struct RunnerLog {
    _directory: TempDir,
    root: Utf8PathBuf,
}

impl RunnerLog {
    fn new() -> TestResult<Self> {
        let directory = TempDir::new().map_err(|error| format!("create log scratch: {error}"))?;
        let root = Utf8PathBuf::from_path_buf(directory.path().to_path_buf())
            .map_err(|path| format!("log scratch is not UTF-8: {path:?}"))?;
        Ok(Self {
            _directory: directory,
            root,
        })
    }

    fn path(&self) -> Utf8PathBuf { self.root.join("fake-runner.log") }

    fn text(&self) -> TestResult<String> {
        open_dir(&self.root)?
            .read_to_string("fake-runner.log")
            .map_err(|error| format!("read fake runner log: {error}").into())
    }
}

/// Run the control against `repository` with the fake runner in `mode`.
fn run_control(
    repository: &Utf8Path,
    runner: &Utf8Path,
    mode: &'static str,
    log: &Utf8Path,
) -> TestResult<Output> {
    Command::new("bash")
        .arg(repository.join("scripts/check-source-span-mutation.sh"))
        .arg("--repo-root")
        .arg(repository)
        .arg("--verus-only")
        .current_dir(repository)
        .env(
            "VERUS_RUN",
            format!("{runner} verus run --repo-root . --no-install-missing"),
        )
        .env("VERUS_Z3_PATH", FAKE_Z3)
        .env("FAKE_MUTATION_MODE", mode)
        .env("FAKE_MUTATION_LOG", log)
        .output()
        .map_err(|error| format!("run mutation control: {error}").into())
}

/// Run the control against a fresh fixture and return its output and log.
fn control_output(kernel: &str, mode: &'static str) -> TestResult<(Output, RunnerLog)> {
    let repository = mutation_repo(kernel)?;
    let runner = fake_mutation_runner()?;
    let log = RunnerLog::new()?;
    let output = run_control(utf8(repository.path())?, runner.path(), mode, &log.path())?;
    Ok((output, log))
}

#[test]
fn a_faithful_runner_accepts_the_control_and_each_mutation_is_rejected() -> TestResult {
    let (output, log) = control_output(KERNEL, "faithful")?;
    require(
        output.status.success(),
        format!(
            "the faithful runner must satisfy the control; stderr was {}",
            String::from_utf8_lossy(&output.stderr)
        ),
    )?;
    let invocations = log.text()?;
    let runs = invocations
        .lines()
        .filter(|line| line.starts_with("verus run "))
        .collect::<Vec<_>>();
    require(
        runs.len() == 3,
        format!(
            "the control must run the unmutated copy and both mutations; log was {invocations:?}"
        ),
    )?;
    for line in &runs {
        require(
            line.contains("--no-install-missing"),
            format!("every control run must forbid installation; log was {invocations:?}"),
        )?;
        require(
            !line.contains(" install"),
            format!("the control must install no prover; log was {invocations:?}"),
        )?;
    }
    let z3_lines = invocations
        .lines()
        .filter(|line| line.starts_with("VERUS_Z3_PATH="))
        .collect::<Vec<_>>();
    require(
        z3_lines.len() == 3 && z3_lines.iter().all(|line| line.ends_with(FAKE_Z3)),
        format!("every control run must receive the configured solver; log was {invocations:?}"),
    )
}

#[rstest]
#[case::accepted("accepted", "mutation was accepted by the verifier")]
#[case::unattributed("unattributed", "names no source location")]
#[case::startup_failure("startup_failure", "did not reach the verifier")]
fn a_non_attributed_outcome_fails_the_control(
    #[case] mode: &'static str,
    #[case] expected: &str,
) -> TestResult {
    let (output, _log) = control_output(KERNEL, mode)?;
    require(
        !output.status.success(),
        format!("mode {mode:?} must fail the control"),
    )?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    require(
        stderr.contains(expected),
        format!("mode {mode:?} should report {expected:?}, got {stderr:?}"),
    )
}

#[test]
fn a_missing_mutation_target_is_reported_as_a_zero_match() -> TestResult {
    // The kernel keeps the return target but not the CR target, so the control
    // verifies the unmutated copy and then fails to substitute the CR text.
    let kernel = KERNEL.replace(
        "while (index > 0 && !(bytes[index - 1] == b'\\n' || bytes[index - 1] == b'\\r'))",
        "while (index > 0)",
    );
    let (output, _log) = control_output(&kernel, "accepted")?;
    require(
        output.status.code() == Some(2),
        format!(
            "a zero-match substitution must exit 2, exited {}; stderr was {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ),
    )?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    require(
        stderr.contains("matched no unique source text"),
        format!("a zero-match substitution must be named, got {stderr:?}"),
    )
}
