//! Guards the Verus workflow, Make targets, and verification-ledger contract.
//!
//! `make verus-selftest` is the executable non-vacuity check in continuous
//! integration. It runs `verus/smoke.rs` and requires the deliberately false
//! assertion to fail, which means a run whose verifier never started — a
//! missing install, a changed banner, a runner that exits early — would
//! otherwise look like success. The Makefile tests here drive the real targets
//! with a controlled fake runner, so they assert on the harness's own logic
//! rather than on a local Verus installation.
//!
//! The test bodies live in `tests/verus_harness/`, and the harness they share
//! in `tests/support/prover_harness.rs`.

#![cfg(unix)]

#[path = "verus_harness/ledger.rs"]
mod ledger;
#[path = "verus_harness/make_targets.rs"]
mod make_targets;
#[path = "support/prover_harness.rs"]
mod prover_harness;
#[path = "verus_harness/workflow.rs"]
mod workflow;
#[path = "support/workflow_scan.rs"]
mod workflow_scan;
