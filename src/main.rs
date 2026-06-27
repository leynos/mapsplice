//! `mapsplice` binary entry point.

use std::{
    io::{self, Write},
    process::ExitCode,
};

/// Run the CLI and report failures.
#[expect(
    clippy::print_stderr,
    reason = "diagnostics belong on stderr for the CLI"
)]
fn main() -> ExitCode {
    match mapsplice::run_from_args(std::env::args_os()) {
        Ok(outcome) => {
            if let Some(stdout) = outcome.stdout
                && let Err(error) = io::stdout().write_all(stdout.as_bytes())
            {
                eprintln!("{error}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
