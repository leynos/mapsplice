//! Contract tests for `.github/workflows/verus.yml`.
//!
//! A workflow that silently runs the wrong verifier, or that runs the proofs
//! without the non-vacuity check, is the failure mode this guards. The
//! assertions are on parsed structure rather than on substrings of the whole
//! document, so reformatting the workflow does not break the test and moving a
//! value into the wrong job does.
//!
//! Parsing and checking are both fallible, so these tests report failures as
//! `Err` rather than panicking: `clippy::panic_in_result_fn` is denied across
//! the workspace, and the same rule makes the failure messages travel with the
//! error rather than as a backtrace.

use crate::{
    prover_harness::{TestResult, read_repo_file},
    workflow_scan::{Line, has_value, parse, sequence, step_values, values},
};

const WORKFLOW: &str = include_str!("../../.github/workflows/verus.yml");

/// The shared action that installs the provers, which the workflow must use.
const INSTALL_ACTION: &str = "leynos/shared-actions/.github/actions/install-prover-tools@";

/// Parse the workflow, mapping a reader failure into the test result type.
fn workflow() -> TestResult<Vec<Line>> {
    parse(WORKFLOW).map_err(|error| format!("parse verus.yml: {error}").into())
}

/// Return an error when `condition` does not hold.
fn require(condition: bool, reason: String) -> TestResult {
    if condition {
        Ok(())
    } else {
        Err(reason.into())
    }
}

#[test]
fn workflow_runs_both_the_proof_and_the_non_vacuity_target() -> TestResult {
    let lines = workflow()?;
    require(
        has_value(&lines, "run", "make verus"),
        "the workflow must run the library proofs".to_owned(),
    )?;
    require(
        has_value(&lines, "run", "make verus-selftest"),
        concat!(
            "the workflow must also run the smoke proof, so a run whose verifier never started ",
            "cannot pass as a successful verification",
        )
        .to_owned(),
    )
}

#[test]
fn workflow_installs_verus_only_through_the_shared_action() -> TestResult {
    let lines = workflow()?;
    let uses = values(&lines, "uses");
    require(
        uses.iter()
            .any(|used| used.starts_with(INSTALL_ACTION) && is_full_sha(used)),
        format!("Verus must be installed by the shared action, pinned by SHA; found {uses:?}"),
    )?;
    require(
        installer_tools(&lines).contains(&"verus"),
        "the installer step itself must be asked for Verus".to_owned(),
    )?;
    require(
        !uses.iter().any(|used| used.starts_with("actions/cache@")),
        format!("the action owns the cache; the workflow must keep none; found {uses:?}"),
    )
}

/// The `tools:` values of the step that uses the shared installer.
fn installer_tools(lines: &[Line]) -> Vec<&str> {
    step_values(lines, |used| used.starts_with(INSTALL_ACTION), "tools")
}

#[test]
fn tools_verus_on_another_step_does_not_satisfy_the_installer_check() -> TestResult {
    let yaml = format!(
        "steps:\n  - name: Install\n    uses: {INSTALL_ACTION}{sha}\n    with:\n      tools: \
         kani\n  - name: Other\n    uses: actions/x@y\n    with:\n      tools: verus\n",
        sha = "a".repeat(40),
    );
    let lines = parse(&yaml)?;
    require(
        has_value(&lines, "tools", "verus") && !installer_tools(&lines).contains(&"verus"),
        "a `tools: verus` outside the installer step must not count".to_owned(),
    )
}

/// True when `used` ends in a 40-digit hexadecimal revision.
fn is_full_sha(used: &str) -> bool {
    used.rsplit_once('@').is_some_and(|(_, revision)| {
        revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

#[test]
fn workflow_checkout_does_not_persist_credentials() -> TestResult {
    require(
        has_value(&workflow()?, "persist-credentials", "false"),
        "checkout must not leave credentials in the workspace".to_owned(),
    )
}

#[test]
fn workflow_triggers_on_pull_requests_and_by_hand() -> TestResult {
    let lines = workflow()?;
    let types = sequence(&lines, "types");
    for required in ["opened", "synchronize", "reopened"] {
        require(
            types.contains(&required),
            format!("the workflow must trigger on {required}; parsed {types:?}"),
        )?;
    }
    require(
        lines
            .iter()
            .any(|line| line.key() == Some("workflow_dispatch")),
        "the workflow must be dispatchable by hand".to_owned(),
    )
}

#[test]
fn workflow_permissions_are_read_only() -> TestResult {
    let lines = workflow()?;
    // `permissions:` is a block mapping in this workflow, so the scope that
    // matters is each `contents:` entry beneath it. Asserting across every
    // such entry in the document covers the workflow-level and job-level
    // scopes at once, which is the property under test: no scope may widen
    // access beyond read.
    let scopes = values(&lines, "contents");
    require(
        !scopes.is_empty(),
        "the workflow must declare a contents permission".to_owned(),
    )?;
    require(
        scopes.iter().all(|declared| *declared == "read"),
        format!("no scope may grant more than read access to contents; found {scopes:?}"),
    )
}

#[test]
fn the_embedded_workflow_is_the_committed_file() -> TestResult {
    let on_disk = read_repo_file(".github/workflows/verus.yml")?;
    require(
        on_disk == WORKFLOW,
        "the embedded workflow and the committed file must be one document".to_owned(),
    )
}
