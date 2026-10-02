//! Process-level contracts for CLI output channels and write failures.

#![cfg(target_os = "linux")]

use std::{
    error::Error,
    fmt::Display,
    io::{self, Write},
    process::{Child, Command, Output, Stdio},
    time::Duration,
};

use camino::Utf8PathBuf;
use cap_std::{
    ambient_authority,
    fs_utf8::{Dir, OpenOptions},
};
use tempfile::TempDir;
use wait_timeout::ChildExt;

/// Bound each subprocess wait so a regression cannot hang the test suite.
const CHILD_TIMEOUT: Duration = Duration::from_secs(10);

/// Error type used by process-level CLI tests.
type TestResult<T = ()> = Result<T, Box<dyn Error>>;

/// Isolated current directory with capability-scoped fixture files.
struct IsolatedWorkspace {
    directory: Dir,
    path: Utf8PathBuf,
    _temporary_directory: TempDir,
}

/// Valid fixture roadmap used to exercise successful command output.
const TARGET_ROADMAP: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. First task.\n",
);

/// Valid phase fragment used by append output cases.
const PHASE_FRAGMENT: &str = concat!(
    "## 2. Phase two\n\n",
    "### 2.1. Step two\n\n",
    "- [ ] 2.1.1. Second task.\n",
);

/// Create a temporary workspace with capability-scoped fixture access.
fn isolated_workspace() -> TestResult<IsolatedWorkspace> {
    let temporary_directory = tempfile::tempdir()?;
    let root = Utf8PathBuf::from_path_buf(temporary_directory.path().to_path_buf())
        .map_err(|path| format!("temporary path is not UTF-8: {}", path.display()))?;
    let root_directory = Dir::open_ambient_dir(&root, ambient_authority())?;
    root_directory.create_dir("workspace")?;
    let path = root.join("workspace");
    let directory = root_directory.open_dir("workspace")?;
    Ok(IsolatedWorkspace {
        directory,
        path,
        _temporary_directory: temporary_directory,
    })
}

/// Replace inherited configuration, locale, colour, and tracing inputs.
fn configure_child(command: &mut Command, workspace: &IsolatedWorkspace) {
    command
        .env_clear()
        .env("HOME", workspace.path.join("home").as_str())
        .env(
            "XDG_CONFIG_HOME",
            workspace.path.join("xdg-config").as_str(),
        )
        .env("CARGO_HOME", workspace.path.join("cargo-home").as_str())
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .env("CLICOLOR_FORCE", "0")
        .env("TERM", "dumb")
        .env("RUST_LOG", "off")
        .current_dir(workspace.path.as_std_path());
}

/// Build an isolated invocation of the compiled CLI binary.
fn cli_command(workspace: &IsolatedWorkspace, arguments: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mapsplice"));
    command.args(arguments);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    configure_child(&mut command, workspace);
    command
}

/// Wait for a child, reaping it after a timeout before returning an error.
fn collect_child_output(mut child: Child) -> io::Result<Output> {
    match child.wait_timeout(CHILD_TIMEOUT) {
        Ok(Some(_)) => {}
        Ok(None) => {
            let kill_result = child.kill();
            let reap_result = child.wait();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!(
                    "CLI child exceeded {CHILD_TIMEOUT:?}; kill: {kill_result:?}; reap: \
                     {reap_result:?}"
                ),
            ));
        }
        Err(error) => {
            let kill_result = child.kill();
            let reap_result = child.wait();
            return Err(io::Error::other(format!(
                "waiting for CLI child failed: {error}; kill: {kill_result:?}; reap: \
                 {reap_result:?}"
            )));
        }
    }
    child.wait_with_output()
}

/// Run a configured CLI command and collect its captured output.
fn run_cli(workspace: &IsolatedWorkspace, arguments: &[&str]) -> io::Result<Output> {
    let child = cli_command(workspace, arguments).spawn()?;
    collect_child_output(child)
}

/// Write the valid roadmap and fragment used by successful edit operations.
fn write_valid_inputs(workspace: &IsolatedWorkspace) -> io::Result<()> {
    workspace.directory.write("target.md", TARGET_ROADMAP)?;
    workspace.directory.write("fragment.md", PHASE_FRAGMENT)
}

/// Open Linux's deterministic failing output device through a capability dir.
fn full_device() -> io::Result<Stdio> {
    let device_directory = Dir::open_ambient_dir("/dev", ambient_authority())?;
    device_directory
        .open_with("full", OpenOptions::new().write(true))
        .map(|file| Stdio::from(file.into_std()))
}

/// Return a test error when an observed CLI contract is not satisfied.
fn verify(condition: bool, message: impl Display) -> TestResult {
    if condition {
        Ok(())
    } else {
        Err(message.to_string().into())
    }
}

#[test]
fn top_level_and_command_help_use_stdout_without_diagnostics() -> TestResult {
    let workspace = isolated_workspace()?;

    for arguments in [&["--help"][..], &["append", "--help"][..]] {
        let output = run_cli(&workspace, arguments)?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        verify(
            output.status.code() == Some(0),
            format!("help should exit 0, got {:?}", output.status.code()),
        )?;
        verify(
            stdout.starts_with("Usage:"),
            format!("help stdout was {stdout:?}"),
        )?;
        verify(
            output.stderr.is_empty(),
            format!("help stderr was {:?}", output.stderr),
        )?;
    }
    Ok(())
}

#[test]
fn invalid_input_has_only_a_stderr_diagnostic_and_fails() -> TestResult {
    let workspace = isolated_workspace()?;
    let output = run_cli(&workspace, &["--invalid-option"])?;
    let stderr = String::from_utf8_lossy(&output.stderr);

    verify(
        output.status.code() == Some(1),
        format!(
            "invalid input exit was {:?}, expected 1",
            output.status.code()
        ),
    )?;
    verify(output.stdout.is_empty(), "invalid input wrote stdout")?;
    verify(
        stderr.starts_with("error:"),
        format!("invalid-input stderr was {stderr:?}"),
    )?;
    Ok(())
}

#[test]
fn ordinary_command_failure_has_only_a_stderr_diagnostic_and_fails() -> TestResult {
    let workspace = isolated_workspace()?;
    write_valid_inputs(&workspace)?;
    let output = run_cli(&workspace, &["delete", "target.md", "99"])?;
    let stderr = String::from_utf8_lossy(&output.stderr);

    verify(
        output.status.code() == Some(1),
        format!(
            "missing-anchor exit was {:?}, expected 1",
            output.status.code()
        ),
    )?;
    verify(output.stdout.is_empty(), "failed command wrote stdout")?;
    verify(
        stderr.contains("anchor"),
        format!("command diagnostic was {stderr:?}"),
    )?;
    Ok(())
}

#[test]
fn roadmap_stdout_write_failure_reports_stderr_and_fails() -> TestResult {
    let workspace = isolated_workspace()?;
    write_valid_inputs(&workspace)?;
    let mut command = cli_command(&workspace, &["append", "target.md", "fragment.md"]);
    command.stdout(full_device()?);
    let child = command.spawn()?;
    let output = collect_child_output(child)?;

    verify(
        output.status.code() == Some(1),
        format!(
            "roadmap write exit was {:?}, expected 1",
            output.status.code()
        ),
    )?;
    verify(
        !output.stderr.is_empty(),
        "roadmap write failure should report a stderr diagnostic",
    )?;
    Ok(())
}

#[test]
fn help_stdout_write_failure_reports_stderr_and_fails() -> TestResult {
    let workspace = isolated_workspace()?;
    let mut command = cli_command(&workspace, &["--help"]);
    command.stdout(full_device()?);
    let child = command.spawn()?;
    let output = collect_child_output(child)?;

    verify(
        output.status.code() == Some(1),
        format!("help write exit was {:?}, expected 1", output.status.code()),
    )?;
    verify(
        !output.stderr.is_empty(),
        "help write failure should report a stderr diagnostic",
    )?;
    Ok(())
}

#[test]
fn clap_diagnostic_stderr_write_failure_fails_without_panicking() -> TestResult {
    let workspace = isolated_workspace()?;
    let mut command = cli_command(&workspace, &["--invalid-option"]);
    command.stderr(full_device()?);
    let child = command.spawn()?;
    let output = collect_child_output(child)?;

    verify(
        output.status.code() == Some(1),
        format!(
            "Clap diagnostic sink failure had status {:?}, expected 1",
            output.status.code()
        ),
    )?;
    Ok(())
}

#[test]
fn command_diagnostic_stderr_write_failure_fails_without_panicking() -> TestResult {
    let workspace = isolated_workspace()?;
    write_valid_inputs(&workspace)?;
    let mut command = cli_command(&workspace, &["delete", "target.md", "99"]);
    command.stderr(full_device()?);
    let child = command.spawn()?;
    let output = collect_child_output(child)?;

    verify(
        output.status.code() == Some(1),
        format!(
            "command diagnostic sink failure had status {:?}, expected 1",
            output.status.code()
        ),
    )?;
    Ok(())
}

/// Start the CLI only after the parent has closed its stdout reader.
fn run_with_closed_stdout(workspace: &IsolatedWorkspace, arguments: &[&str]) -> io::Result<Output> {
    let mut command = Command::new("/bin/sh");
    command
        .arg("-c")
        .arg("IFS= read -r _; exec \"$@\"")
        .arg("mapsplice-test-gate")
        .arg(env!("CARGO_BIN_EXE_mapsplice"))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    configure_child(&mut command, workspace);
    let mut child = command.spawn()?;
    drop(child.stdout.take());

    let release_result = child.stdin.as_mut().map_or_else(
        || Err(io::Error::other("gated child stdin was not piped")),
        |stdin| stdin.write_all(b"start\n"),
    );
    if let Err(error) = release_result {
        let kill_result = child.kill();
        let reap_result = child.wait();
        return Err(io::Error::other(format!(
            "could not release gated CLI child: {error}; kill: {kill_result:?}; reap: \
             {reap_result:?}"
        )));
    }
    drop(child.stdin.take());
    collect_child_output(child)
}

#[test]
fn broken_stdout_pipes_succeed_for_roadmap_output_and_help() -> TestResult {
    for (arguments, needs_inputs) in [
        (&["append", "target.md", "fragment.md"][..], true),
        (&["--help"][..], false),
    ] {
        let workspace = isolated_workspace()?;
        if needs_inputs {
            write_valid_inputs(&workspace)?;
        }
        let output = run_with_closed_stdout(&workspace, arguments)?;

        verify(
            output.status.code() == Some(0),
            format!(
                "closed-pipe command {arguments:?} had status {:?}, expected 0",
                output.status.code()
            ),
        )?;
    }
    Ok(())
}
