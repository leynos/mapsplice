//! `mapsplice` binary entry point.

use std::{
    fmt::Display,
    io::{self, Write},
    process::ExitCode,
};

use mapsplice::{MapspliceError, RunOutcome};

/// Run the CLI and report failures.
fn main() -> ExitCode {
    init_tracing();
    match mapsplice::run_from_args(std::env::args_os()) {
        Ok(outcome) => emit_outcome(outcome),
        Err(MapspliceError::Clap(error)) => emit_clap_display(&error),
        Err(error) => report_error(&error),
    }
}

/// Emit optional CLI output and return its write status.
///
/// For example, an outcome without stdout exits successfully without writing.
fn emit_outcome(outcome: RunOutcome) -> ExitCode {
    outcome
        .stdout
        .map_or(ExitCode::SUCCESS, |stdout| write_stdout(&stdout))
}

/// Write CLI output, treating a closed pipe as a successful exit.
///
/// For example, a consumer that closes the pipe early does not report a CLI failure.
fn write_stdout(stdout: &str) -> ExitCode {
    if let Err(error) = io::stdout().write_all(stdout.as_bytes()) {
        if error.kind() == io::ErrorKind::BrokenPipe {
            return ExitCode::SUCCESS;
        }
        report_stdout_write_failure(&error);
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// Print a Clap display message and map its exit status to the process result.
///
/// For example, help text uses Clap's successful display status.
fn emit_clap_display(error: &clap::Error) -> ExitCode {
    let exit_code = error.exit_code();
    if let Err(print_error) = error.print() {
        if print_error.kind() == io::ErrorKind::BrokenPipe {
            return ExitCode::SUCCESS;
        }
        report_clap_display_write_failure(&print_error);
        return ExitCode::FAILURE;
    }
    exit_code_from_i32(exit_code)
}

/// Report a command error on stderr and return failure.
///
/// For example, an invalid roadmap emits its diagnostic before exiting.
fn report_error(error: &MapspliceError) -> ExitCode {
    tracing::error!(error_class = error.class(), "mapsplice command failed");
    report_stderr_diagnostic(error);
    ExitCode::FAILURE
}

/// Trace a stdout write failure and report it through stderr.
///
/// For example, a non-pipe stdout failure keeps the CLI exit status unsuccessful.
fn report_stdout_write_failure(error: &io::Error) {
    tracing::error!(error = %error, error_class = "stdout", "failed to write CLI output");
    report_stderr_diagnostic(error);
}

/// Trace a failed Clap display and report its write error through stderr.
///
/// For example, a failed help display retains Clap's existing failure status.
fn report_clap_display_write_failure(error: &io::Error) {
    tracing::error!(error = %error, error_class = "cli_display", "failed to print CLI display output");
    report_stderr_diagnostic(error);
}

/// Record a failure while writing a diagnostic to stderr without panicking.
///
/// For example, a closed stderr sink produces a tracing event when a subscriber exists.
fn report_stderr_write_failure(error: &io::Error) {
    tracing::error!(error = %error, error_class = "stderr", "failed to write CLI error output");
}

/// Write a diagnostic to stderr and trace any failure to write it.
///
/// For example, a broken stderr sink is reported without changing the caller's exit status.
fn report_stderr_diagnostic(message: &impl Display) {
    if let Err(error) = emit_stderr_line(message) {
        report_stderr_write_failure(&error);
    }
}

/// Write one displayable message and a trailing newline to an injected writer.
///
/// For example, writing `failed` appends one LF byte.
fn write_stderr_line(writer: &mut impl Write, message: &impl Display) -> io::Result<()> {
    writeln!(writer, "{message}")
}

/// Write one diagnostic line directly to standard error.
///
/// For example, `"failed"` is emitted with one trailing newline.
fn emit_stderr_line(message: &impl Display) -> io::Result<()> {
    let stderr = io::stderr();
    let mut locked_stderr = stderr.lock();
    write_stderr_line(&mut locked_stderr, message)
}

/// Initialize stderr tracing with the default environment filter when possible.
///
/// For example, `RUST_LOG=debug` enables debug events when no subscriber exists.
fn init_tracing() {
    drop(
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_writer(io::stderr)
            .try_init(),
    );
}

/// Convert Clap's zero status to success and all other statuses to failure.
///
/// For example, a help-display status of zero becomes `ExitCode::SUCCESS`.
const fn exit_code_from_i32(code: i32) -> ExitCode {
    if code == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod stderr_tests {
    //! Tests for the byte and error contract of explicit stderr writes.

    use std::io::{self, Write};

    use super::write_stderr_line;

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "writer failed",
            ))
        }

        fn flush(&mut self) -> io::Result<()> { Ok(()) }
    }

    #[test]
    fn writes_message_with_one_trailing_newline() {
        let mut output = Vec::new();

        write_stderr_line(&mut output, &"CLI failure")
            .expect("writing to an in-memory buffer should succeed");

        assert_eq!(output, b"CLI failure\n");
    }

    #[test]
    fn returns_writer_failure() {
        let mut writer = FailingWriter;

        let error = write_stderr_line(&mut writer, &"CLI failure")
            .expect_err("the writer error should be returned");

        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    }
}

#[cfg(test)]
mod diagnostic_tracing_tests {
    //! Tests that structured tracing excludes unbounded user diagnostics.

    use std::sync::{Arc, Mutex};

    use tracing_subscriber::fmt::MakeWriter;

    use super::{MapspliceError, report_error};

    /// Shared bytes captured from the scoped tracing subscriber.
    #[derive(Clone)]
    struct SharedWriter(Arc<Mutex<Vec<u8>>>);

    /// Write guard that appends one event's formatted bytes to shared storage.
    struct SharedWriterGuard(Arc<Mutex<Vec<u8>>>);

    impl<'a> MakeWriter<'a> for SharedWriter {
        type Writer = SharedWriterGuard;

        fn make_writer(&'a self) -> Self::Writer { SharedWriterGuard(Arc::clone(&self.0)) }
    }

    impl std::io::Write for SharedWriterGuard {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let mut output = self
                .0
                .lock()
                .map_err(|_| std::io::Error::other("captured tracing output was poisoned"))?;
            output.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    }

    #[test]
    fn command_trace_keeps_error_class_without_user_supplied_message() {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_writer(SharedWriter(Arc::clone(&captured)))
            .finish();
        let marker = "user-supplied-diagnostic-marker";
        let error = MapspliceError::InvalidRoadmap {
            message: marker.to_owned(),
        };

        let exit_code = tracing::subscriber::with_default(subscriber, || report_error(&error));

        assert_eq!(exit_code, std::process::ExitCode::FAILURE);
        let output_bytes = captured
            .lock()
            .expect("captured tracing output should remain available")
            .clone();
        let trace_output =
            String::from_utf8(output_bytes).expect("formatted tracing event should be valid UTF-8");
        assert!(
            trace_output.contains("error_class=\"invalid_roadmap\""),
            "trace omitted the stable error class: {trace_output:?}"
        );
        assert!(
            trace_output.contains("mapsplice command failed"),
            "trace omitted its static event message: {trace_output:?}"
        );
        assert!(
            !trace_output.contains("error="),
            "trace retained the unbounded error field: {trace_output:?}"
        );
        assert!(
            !trace_output.contains(marker),
            "trace exposed the user-supplied error message: {trace_output:?}"
        );
    }
}
