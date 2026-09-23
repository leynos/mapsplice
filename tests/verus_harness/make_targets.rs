//! Contract tests for the Makefile's Verus targets.
//!
//! `make verus` must install the pinned verifier before running, and
//! `make verus-selftest` must fail loudly in the two ways that matter: when
//! the smoke proof is accepted, and when the runner fails for a reason that is
//! not the verifier's own rejection. A selftest that passes because the tool
//! was missing is worse than no selftest, because it reads as evidence.

use std::process::Command;

use rstest::rstest;

use crate::prover_harness::{TestResult, fake_prover_tools};

/// Run `make <target>` against a fake runner and return its output.
fn run_make(target: &str, smoke_mode: &'static str) -> TestResult<std::process::Output> {
    let runner = fake_prover_tools(smoke_mode)?;
    let output = runner
        .make(target)
        .output()
        .map_err(|error| format!("run make {target}: {error}"))?;
    // The runner is dropped here, so the log is captured before it goes away.
    let log = runner.log()?;
    assert_installed_and_ran(&log, target);
    Ok(output)
}

/// Assert the runner was asked to install, then to run the target's proof file.
fn assert_installed_and_ran(log: &str, target: &str) {
    let proof_file = match target {
        "verus" => "verus/lib.rs",
        "verus-selftest" => "verus/smoke.rs",
        other => panic!("unexpected target {other}"),
    };
    assert!(
        log.lines()
            .any(|line| line == "verus install --repo-root ."),
        "{target} must install the pinned verifier first; log was {log:?}"
    );
    let expected = format!("verus run --repo-root . --proof-file {proof_file}");
    assert!(
        log.lines().any(|line| line == expected),
        "{target} must run the proof through the pinned runner; log was {log:?}"
    );
}

#[test]
fn make_verus_installs_and_runs_the_library_proof() -> TestResult {
    let output = run_make("verus", "rejected")?;
    assert!(output.status.success(), "make verus should succeed");
    Ok(())
}

#[rstest]
#[case::rejected_proof("rejected", true, None)]
#[case::accepted_proof("accepted", false, Some("Verus smoke proof unexpectedly succeeded"))]
#[case::unrelated_runner_failure(
    "unrelated_failure",
    false,
    Some("Verus smoke proof did not reach the verifier")
)]
fn make_verus_selftest_accepts_only_a_rejected_smoke_proof(
    #[case] smoke_mode: &'static str,
    #[case] should_succeed: bool,
    #[case] expected_diagnostic: Option<&str>,
) -> TestResult {
    let output = run_make("verus-selftest", smoke_mode)?;
    assert_eq!(
        output.status.success(),
        should_succeed,
        "smoke mode {smoke_mode:?}"
    );
    if let Some(expected) = expected_diagnostic {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected),
            "expected diagnostic {expected:?}, got {stderr:?}"
        );
    }
    Ok(())
}

#[test]
fn make_verus_targets_require_the_pinned_runner() -> TestResult {
    // The prover-tools binary is resolved from a commit-pinned REF file, so CI
    // and a developer's shell install the same verifier. A floating reference
    // would make the proofs reproducible only by luck.
    let makefile = std::fs::read_to_string(crate::prover_harness::manifest_dir().join("Makefile"))
        .map_err(|error| format!("read Makefile: {error}"))?;
    assert!(
        makefile.contains(
            "git+https://github.com/leynos/rust-prover-tools@$(shell cat \
             tools/rust-prover-tools/REF)"
        ),
        "the prover-tools runner must be pinned to the REF file"
    );
    Ok(())
}

#[test]
fn verus_targets_are_declared_and_phony() -> TestResult {
    let makefile = std::fs::read_to_string(crate::prover_harness::manifest_dir().join("Makefile"))
        .map_err(|error| format!("read Makefile: {error}"))?;
    // `.PHONY` spans continuation lines, so the declaration list is assembled
    // from the whole file rather than from one line.
    let phony: Vec<&str> = makefile
        .lines()
        .skip_while(|line| !line.starts_with(".PHONY:"))
        .take_while(|line| line.starts_with(".PHONY:") || line.starts_with('\t'))
        .flat_map(|line| line.trim_start_matches(".PHONY:").split_whitespace())
        .collect();
    for target in [
        "verus",
        "verus-selftest",
        "verus-install",
        "check-prover-tools",
        "check-verification-ledger",
    ] {
        assert!(
            makefile.contains(&format!("\n{target}:")),
            "{target} must define a target"
        );
        assert!(
            phony.contains(&target),
            "{target} must be .PHONY, so a file of that name cannot shadow it; declared: {phony:?}"
        );
    }
    Ok(())
}

#[test]
fn running_a_target_under_a_missing_runner_fails_before_make_runs_it() -> TestResult {
    // `check-prover-tools` is the guard that turns a missing runner into a
    // clear message rather than a shell "command not found" further down.
    let output = Command::new("make")
        .arg("--no-print-directory")
        .arg("check-prover-tools")
        .current_dir(crate::prover_harness::manifest_dir())
        .env("PROVER_TOOLS", "definitely-not-a-real-runner-binary")
        .output()
        .map_err(|error| format!("run make check-prover-tools: {error}"))?;
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("is required for Verus verification"),
        "missing runner must be reported, not silently tolerated"
    );
    Ok(())
}
