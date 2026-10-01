# Debugging Plan: Cranelift panic unwinding

- **Generated:** 2026-09-30 00:15 Europe/Berlin
- **Issue ID:** `build_standard_panic` regression probe
- **Severity:** High — the configured development backend fails two tests
- **Falsification sub-agent:** `alchemist`
**Planning agent boundary**: This document was prepared by the planning agent.
Falsification must be executed by the named sub-agent, not by the planning
agent.

## Problem Statement

On Linux x86_64, the development test suite configured with Cranelift fails
`catch_unwind_catches_a_development_panic` and aborts during
`spawned_thread_panic_does_not_abort_the_process`. The corresponding explicit
LLVM-profile suite passes all 293 tests, including all three panic probes.
Determine whether explicitly selecting `panic = "unwind"` for the Cranelift
test profile makes the focused probes pass. That hypothesis has since been
falsified; the repository still needs an approved test-routing decision or an
evidence-backed backend fix before it can meet the development-build baseline.

## Context Summary

| Aspect              | Details                                                     |
| ------------------- | ----------------------------------------------------------- |
| First observed      | 2026-09-29, head `da47be7b6813192599ebf19c562f017cc2fbc789` |
| Reproduction rate   | Reproduced in two full Cranelift suite runs; 291/293 pass   |
| Affected components | Cranelift test profile and `tests/build_standard_panic.rs`  |
| Recent changes      | Development profile selected Cranelift with `-Zthreads=8`   |

### Error Artefacts

```text
test catch_unwind_catches_a_development_panic ... FAILED
fatal runtime error: failed to initiate panic, error 5, aborting
error: test failed, to rerun pass `--test build_standard_panic`
```

The second occurrence terminates with `SIGABRT`. The standalone
`#[should_panic]` probe passes under Cranelift. The explicit LLVM-profile full
suite passes all 293 tests. The repository pins `nightly-2026-03-26`; the
measured compiler is `rustc 1.96.0-nightly (80d0e4be6 2026-03-25)`.

### Evidence and Remaining Questions

The full Cranelift component provenance is not recorded in the Rust toolchain
manifest. A focused candidate using the installed `nightly-2026-09-13`
component falsified the version-fix hypothesis: both
`catch_unwind_catches_a_development_panic` and
`spawned_thread_panic_does_not_abort_the_process` fail or abort there, while
`should_panic_tests_keep_their_expected_result` passes. A sequential comparison
of those same isolated tests on the pinned `nightly-2026-03-26` and installed
`nightly-2026-09-13` found the same outcomes on both toolchains. Verbose rustc
logs confirm Cranelift was active and no explicit panic-strategy flag was
injected in this comparison. This rules out the tested toolchain-version change
as a fix; it does not establish the upstream cause.

The full pinned-toolchain `make test` run failed the two panic tests, with 212
tests not run after the abort. The explicit LLVM route previously passed all
293 tests. The task's request to route tests through LLVM while keeping
Cranelift as the development build default is still pending user approval; no
exception has been adopted. The next step is an approved routing decision or
another evidence-backed backend fix. Do not present a Cranelift exception as
decided.

______________________________________________________________________

## Hypotheses

### H1: Passing unwind directly to rustc resolves the Cranelift failures

**Claim**: Explicitly passing `-C panic=unwind` to rustc makes the failing
Cranelift panic probes pass.

**Plausibility**: Falsified for the tested Cranelift component and target.
Upstream support status and the underlying runtime/compiler cause remain
unverified.

**Prediction**: If this hypothesis held, the focused target would pass all
three tests when Make passed `-C panic=unwind` through `RUST_FLAGS`, while
retaining the repository's Cranelift backend and development flags. The retry
did not pass the two affected tests, so this prediction was not met.

#### H1 Falsification Test

| Step | Action                                                                                                                                                                                                  | Result                                                                                                                                   |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | Run only `build_standard_panic` through `make test` with `RUST_FLAGS='-D warnings -C panic=unwind'`, `CARGO_PROFILE_TEST_CODEGEN_BACKEND=cranelift`, verbose Cargo output, and one test-harness thread. | An initial retry reached rustc with both settings but failed to create a helper thread before compiling the target; it was inconclusive. |
| 2    | Retry with `CARGO_BUILD_JOBS=1`, after resource pressure eased, and capture the rustc invocation and test result.                                                                                       | Rustc received Cranelift and `-C panic=unwind`; the panic failures remained. H1 was falsified.                                           |

**Tooling**: Run
`make test DOC_TEST_TARGETS=false
TEST_CMD='test --test build_standard_panic'
TEST_FLAGS='-vv -- --test-threads=1' RUST_FLAGS='-D warnings -C panic=unwind'`
with `CARGO_PROFILE_TEST_CODEGEN_BACKEND=cranelift` and `CARGO_BUILD_JOBS=1`
in the command environment. The job limit avoids the observed shared process
limit while retaining the compiler's configured frontend flags. Capture the
exact rustc invocation and test result. The alchemist may run only this focused
experiment.

**Confidence on falsification**: High. The retry recompiled the target with
`-C panic=unwind` and `-Zcodegen-backend=cranelift`; the catch-unwind test
still failed and the spawned-thread panic still aborted. A separate sequential
comparison also reproduced the two failures on both tested nightly toolchains.

______________________________________________________________________

## Recommended Execution Order

1. **H1** — Complete. Explicit rustc unwind did not resolve the Cranelift
   failures. The next action is an approved test-routing decision or a new,
   evidence-backed backend investigation.

## Termination Criteria

- **Probe question resolved**: H1 is falsified by a confirmed Cranelift and
  `panic=unwind` invocation with the same failures. This does not identify the
  underlying compiler/runtime cause or approve an exception.
- **Next decision**: Obtain approval for an explicit non-development test route
  or provide evidence for a different backend fix. The routing request remains
  pending.

## Notes for Executing Agent

The focused tests described here have been run. Their evidence is recorded
below. Do not change the default backend, modify tests, or infer an approved
exception from the result.

## Latest experiment disposition

The initial `CARGO_PROFILE_TEST_PANIC=unwind` run was inconclusive because
Cargo ignored the setting for the test profile and reused a cached executable.
The first explicit `-C panic=unwind` attempt also failed before compiling the
target because the compiler could not create a helper thread. The later
`CARGO_BUILD_JOBS=1` retry recompiled the target and falsified H1: catch-unwind
still failed, `#[should_panic]` passed, and the spawned-thread panic aborted.
Log:

- `/tmp/cranelift-unwind-mapsplice-rust-baseline-hardening-20260930T001500Z.out`
- `/tmp/cranelift-explicit-unwind-mapsplice-rust-baseline-hardening-20260930T002800Z.out`
- `/tmp/cranelift-explicit-unwind-jobs1-mapsplice-rust-baseline-hardening-20260930T004500Z.out`

The `nightly-2026-03-26` versus `nightly-2026-09-13` sequential comparison
produced matching results: catch-unwind failed, spawned-thread panic aborted,
and `#[should_panic]` passed on both. Verbose rustc output confirmed Cranelift
with no injected panic-strategy flag. Logs:

- `/tmp/cranelift-20260326-catch-unwind-mapsplice-20260930T020000Z.out`
- `/tmp/cranelift-20260326-spawned-panic-mapsplice-20260930T020000Z.out`
- `/tmp/cranelift-20260326-should-panic-mapsplice-20260930T020000Z.out`
- `/tmp/cranelift-20260913-catch-unwind-mapsplice-20260930T020000Z.out`
- `/tmp/cranelift-20260913-spawned-panic-mapsplice-20260930T020000Z.out`
- `/tmp/cranelift-20260913-should-panic-mapsplice-20260930T020000Z.out`

The current pinned full `make test` run failed the same two panic tests; 212 of
293 tests did not run after the abort. See
`/tmp/test-034e7665-13aa-4a68-b7b0-1ce103d5e371-rust-baseline-hardening-mapsplice-20260930T0300.out`.
The explicit LLVM full suite previously passed 293/293. The test-routing
exception request remains pending; no exception is approved. Resolve the next
step through an approved routing decision or a separately evidenced backend fix.
