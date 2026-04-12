//! `mapsplice` binary entry point.

use std::process::ExitCode;

/// Run the CLI and report failures.
#[expect(
    clippy::print_stdout,
    reason = "roadmap output is the intended CLI behaviour"
)]
#[expect(
    clippy::print_stderr,
    reason = "diagnostics belong on stderr for the CLI"
)]
fn main() -> ExitCode {
    match mapsplice::run_from_args(std::env::args_os()) {
        Ok(outcome) => {
            if let Some(stdout) = outcome.stdout {
                println!("{stdout}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
