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
    workflow_scan::{Line, has_value, parse, sequence, values},
};

const WORKFLOW: &str = include_str!("../../.github/workflows/verus.yml");
const VERSION_FILE: &str = include_str!("../../tools/verus/VERSION");

/// The pinned `actions/checkout` used across this repository's workflows.
const CHECKOUT: &str = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";
/// The pinned `actions/cache` used across this repository's workflows.
const CACHE: &str = "actions/cache@55cc8345863c7cc4c66a329aec7e433d2d1c52a9";

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
        "the workflow must also run the smoke proof, so a run whose verifier never started cannot \
         pass as a successful verification"
            .to_owned(),
    )
}

#[test]
fn workflow_verus_version_matches_the_pinned_release() -> TestResult {
    let lines = workflow()?;
    let declared = values(&lines, "VERUS_VERSION");
    require(
        declared == vec![VERSION_FILE.trim()],
        format!(
            "the workflow's VERUS_VERSION must equal tools/verus/VERSION; the cache key derives \
             from it, so a mismatch restores the wrong verifier. Declared {declared:?}, pinned \
             {:?}",
            VERSION_FILE.trim()
        ),
    )
}

#[test]
fn workflow_caches_the_version_scoped_verus_installation() -> TestResult {
    let lines = workflow()?;
    require(
        has_value(&lines, "path", ".verus/${{ env.VERUS_VERSION }}"),
        "the cache must target the version-scoped install directory".to_owned(),
    )?;
    require(
        has_value(
            &lines,
            "key",
            "verus-${{ runner.os }}-${{ runner.arch }}-${{ env.VERUS_VERSION }}",
        ),
        "the cache key must be platform- and version-scoped".to_owned(),
    )
}

#[test]
fn workflow_uses_the_repositorys_pinned_actions() -> TestResult {
    let lines = workflow()?;
    let uses = values(&lines, "uses");
    require(
        uses.contains(&CHECKOUT),
        format!("checkout must use the revision pinned across this repository; found {uses:?}"),
    )?;
    require(
        uses.contains(&CACHE),
        format!(
            "the cache step must use the revision pinned across this repository; found {uses:?}"
        ),
    )
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
