//! Integration tests for the Makefile Mermaid validation target.

use std::{
    error::Error,
    process::{Command, ExitStatus},
};

use rstest::rstest;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

#[rstest]
#[case::default_concurrency(None, None, "1", "1")]
#[case::serial_override(Some("1"), None, "1", "1")]
#[case::concurrency_override(Some("2"), None, "2", "1")]
#[case::renderer_thread_override(None, Some("2"), "1", "2")]
fn nixie_target_passes_bounded_concurrency_to_merman(
    #[case] concurrency_override: Option<&str>,
    #[case] renderer_threads_override: Option<&str>,
    #[case] expected_concurrency: &str,
    #[case] expected_renderer_threads: &str,
) -> TestResult {
    let output = make_nixie_dry_run(concurrency_override, renderer_threads_override)?;

    assert_nixie_contract(&output, expected_concurrency, expected_renderer_threads);
    Ok(())
}

fn make_nixie_dry_run(
    concurrency_override: Option<&str>,
    renderer_threads_override: Option<&str>,
) -> TestResult<String> {
    let mut command = Command::new("make");
    command.current_dir(env!("CARGO_MANIFEST_DIR")).args([
        "--dry-run",
        "--always-make",
        "--no-print-directory",
        "nixie",
        "MERMAN=echo-merman",
    ]);

    if let Some(concurrency) = concurrency_override {
        command.arg(format!("NIXIE_MAX_CONCURRENCY={concurrency}"));
    }
    if let Some(renderer_threads) = renderer_threads_override {
        command.arg(format!("NIXIE_RENDERER_THREADS={renderer_threads}"));
    }

    let command_output = command.output()?;

    assert_success(command_output.status);
    Ok(String::from_utf8(command_output.stdout)?)
}

fn assert_success(status: ExitStatus) {
    assert!(status.success(), "make dry-run failed with status {status}");
}

fn assert_nixie_contract(
    output: &str,
    expected_concurrency: &str,
    expected_renderer_threads: &str,
) {
    assert_nixie_command_contains(
        output,
        &format!("RAYON_NUM_THREADS=\"{expected_renderer_threads}\""),
    );
    assert_nixie_command_contains(output, "echo-merman");
    assert_nixie_command_contains(output, "-j");
    assert_nixie_command_contains(output, expected_concurrency);
    assert_nixie_command_contains(output, "-i");
    assert_nixie_command_contains(output, "-a");
}

fn assert_nixie_command_contains(output: &str, expected: &str) {
    assert!(
        output.contains(expected),
        "expected nixie dry-run output to contain {expected:?}, got {output:?}",
    );
}
