//! `mapsplice` binary entry point.

use std::{
    io::{self, Write},
    process::ExitCode,
};

use clap::error::ErrorKind;
use mapsplice::{MapspliceError, RunOutcome};

/// Run the CLI and report failures.
fn main() -> ExitCode {
    init_tracing();
    match mapsplice::run_from_args(std::env::args_os()) {
        Ok(outcome) => emit_outcome(outcome),
        Err(MapspliceError::Clap(error)) if is_display_request(error.kind()) => {
            emit_clap_display(&error)
        }
        Err(error) => report_error(&error),
    }
}

fn emit_outcome(outcome: RunOutcome) -> ExitCode {
    outcome
        .stdout
        .map_or(ExitCode::SUCCESS, |stdout| write_stdout(&stdout))
}

#[expect(
    clippy::print_stderr,
    reason = "diagnostics belong on stderr for the CLI"
)]
fn write_stdout(stdout: &str) -> ExitCode {
    if let Err(error) = io::stdout().write_all(stdout.as_bytes()) {
        if error.kind() == io::ErrorKind::BrokenPipe {
            return ExitCode::SUCCESS;
        }
        tracing::error!(error = %error, error_class = "stdout", "failed to write CLI output");
        eprintln!("{error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

const fn is_display_request(kind: ErrorKind) -> bool {
    matches!(kind, ErrorKind::DisplayHelp | ErrorKind::DisplayVersion)
}

#[expect(
    clippy::print_stderr,
    reason = "diagnostics belong on stderr for the CLI"
)]
fn emit_clap_display(error: &clap::Error) -> ExitCode {
    let exit_code = error.exit_code();
    if let Err(print_error) = error.print() {
        tracing::error!(error = %print_error, error_class = "cli_display", "failed to print CLI display output");
        eprintln!("{print_error}");
        return ExitCode::FAILURE;
    }
    exit_code_from_i32(exit_code)
}

#[expect(
    clippy::print_stderr,
    reason = "diagnostics belong on stderr for the CLI"
)]
fn report_error(error: &MapspliceError) -> ExitCode {
    tracing::error!(error = %error, error_class = error.class(), "mapsplice command failed");
    eprintln!("{error}");
    ExitCode::FAILURE
}

fn init_tracing() {
    drop(
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_writer(io::stderr)
            .try_init(),
    );
}

const fn exit_code_from_i32(code: i32) -> ExitCode {
    if code == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
