//! Contract tests for `.github/workflows/verus.yml`.
//!
//! A workflow that silently runs the wrong verifier, or that runs the proofs
//! without the non-vacuity check, is the failure mode this guards. The
//! assertions are on parsed structure rather than on substrings of the whole
//! document, so reformatting the workflow does not break the test and moving a
//! value into the wrong job does.

use crate::{
    prover_harness::{TestResult, manifest_dir},
    workflow_scan::{Line, has_value, parse, sequence, values},
};

const WORKFLOW: &str = include_str!("../../.github/workflows/verus.yml");
const VERSION_FILE: &str = include_str!("../../tools/verus/VERSION");

/// The pinned `actions/checkout` used across this repository's workflows.
const CHECKOUT: &str = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";
/// The pinned `actions/cache` used across this repository's workflows.
const CACHE: &str = "actions/cache@55cc8345863c7cc4c66a329aec7e433d2d1c52a9";

#[test]
fn workflow_runs_both_the_proof_and_the_non_vacuity_target() {
    let lines = parse(WORKFLOW);
    assert!(
        has_value(&lines, "run", "make verus"),
        "the workflow must run the library proofs"
    );
    assert!(
        has_value(&lines, "run", "make verus-selftest"),
        "the workflow must also run the smoke proof, so a run whose verifier never started cannot \
         pass as a successful verification"
    );
}

#[test]
fn workflow_verus_version_matches_the_pinned_release() {
    let lines = parse(WORKFLOW);
    assert_eq!(
        values(&lines, "VERUS_VERSION"),
        vec![VERSION_FILE.trim()],
        "the workflow's VERUS_VERSION must equal tools/verus/VERSION; the cache key derives from \
         it, so a mismatch restores the wrong verifier"
    );
}

#[test]
fn workflow_caches_the_version_scoped_verus_installation() {
    let lines = parse(WORKFLOW);
    assert!(
        has_value(&lines, "path", ".verus/${{ env.VERUS_VERSION }}"),
        "the cache must target the version-scoped install directory"
    );
    assert!(
        has_value(
            &lines,
            "key",
            "verus-${{ runner.os }}-${{ runner.arch }}-${{ env.VERUS_VERSION }}",
        ),
        "the cache key must be platform- and version-scoped"
    );
}

#[test]
fn workflow_uses_the_repositorys_pinned_actions() {
    let lines = parse(WORKFLOW);
    let uses = values(&lines, "uses");
    assert!(
        uses.contains(&CHECKOUT),
        "checkout must use the revision pinned across this repository"
    );
    assert!(
        uses.contains(&CACHE),
        "the cache step must use the revision pinned across this repository"
    );
}

#[test]
fn workflow_checkout_does_not_persist_credentials() {
    assert!(
        has_value(&parse(WORKFLOW), "persist-credentials", "false"),
        "checkout must not leave credentials in the workspace"
    );
}

#[test]
fn workflow_triggers_on_pull_requests_and_by_hand() {
    let lines = parse(WORKFLOW);
    let types = sequence(&lines, "types");
    for required in ["opened", "synchronize", "reopened"] {
        assert!(
            types.contains(&required),
            "the workflow must trigger on {required}; parsed {types:?}"
        );
    }
    assert!(
        values(&lines, "workflow_dispatch").is_empty()
            && lines
                .iter()
                .any(|line| line.key() == Some("workflow_dispatch")),
        "the workflow must be dispatchable by hand"
    );
}

#[test]
fn workflow_permissions_are_read_only() {
    let lines = parse(WORKFLOW);
    // `permissions:` is a block mapping in this workflow, so the scope that
    // matters is the `contents:` entry under it. Asserting across every
    // `contents` entry in the document covers the workflow-level and
    // job-level scopes at once, which is the property under test: no scope may
    // widen access beyond read.
    ensure_all(
        &lines,
        "contents",
        "read",
        "no scope may grant more than read access to contents",
    );
}

/// Assert every entry under `key` has the value `value`.
///
/// A workflow can declare the same key at several scopes, and a narrower scope
/// that widens access is exactly the regression this guards, so the assertion
/// has to cover all of them rather than the first.
fn ensure_all(lines: &[Line], key: &str, value: &str, because: &str) {
    let found = values(lines, key);
    assert!(
        !found.is_empty(),
        "{key} should be declared at least once; {because}"
    );
    assert!(
        found.iter().all(|declared| *declared == value),
        "{key} should be {value:?} everywhere, found {found:?}; {because}"
    );
}

#[test]
fn the_embedded_workflow_is_the_committed_file() -> TestResult {
    let path = manifest_dir().join(".github/workflows/verus.yml");
    let on_disk = std::fs::read_to_string(&path)
        .map_err(|error| format!("read {}: {error}", path.as_str()))?;
    assert_eq!(
        on_disk, WORKFLOW,
        "the embedded workflow and the committed file must be one document"
    );
    Ok(())
}
