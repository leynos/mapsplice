# Debugging Plan: Cranelift panic unwinding

- **Generated:** 2026-09-30 00:15 Europe/Berlin
- **Issue ID:** `build_standard_panic` regression probe
- **Disposition:** Resolved by an approved LLVM development default; Cranelift
  remains excluded pending review
- **Falsification sub-agent:** `alchemist`
**Planning agent boundary**: This document was prepared by the planning agent.
Falsification must be executed by the named sub-agent, not by the planning
agent.

## Problem Statement

At the time of investigation, on Linux x86_64, the development test suite
configured with Cranelift failed `catch_unwind_catches_a_development_panic` and
aborted during `spawned_thread_panic_does_not_abort_the_process`. The
corresponding explicit LLVM-profile suite passes all 293 tests, including all
three panic probes. The explicit `panic = "unwind"` hypothesis was falsified,
and a later bare `rustc` comparison isolated the failure to the Cranelift
backend. The approved route therefore uses LLVM for all development builds and
tests while retaining the parallel frontend and pinned Linux linker. Reconsider
Cranelift only after the evidence is reviewed under
[issue #115](https://github.com/leynos/mapsplice/issues/115) on 2027-04-01.

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
`spawned_thread_panic_does_not_abort_the_process` failed or aborted there, while
`should_panic_tests_keep_their_expected_result` passes. A sequential
comparison of those same isolated tests on the pinned `nightly-2026-03-26` and
installed `nightly-2026-09-13` found the same outcomes on both toolchains.
Verbose rustc logs confirm Cranelift was active and no explicit panic-strategy
flag was injected in this comparison. This rules out the tested
toolchain-version change as a fix; it does not establish the upstream cause.

The full pinned-toolchain `make test` run failed the two panic tests, with 212
tests not run after the abort. The explicit LLVM route previously passed all
293 tests. The approved routing decision excludes Cranelift from every
development default, including builds and checks, rather than limiting the
exception to tests. A bare-compiler experiment further isolates the observed
failure from Cargo routing and development flags; it does not identify the
underlying cause. This is a scoped exception to the selected build-default
rule, pending the review date in issue #115; it is not evidence that Cranelift
is generally incompatible.

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
   failures. The approved development route uses LLVM pending the review in
   issue #115.

## Termination Criteria

- **Probe question resolved**: H1 is falsified by a confirmed Cranelift and
  `panic=unwind` invocation with the same failures. This does not identify the
  underlying compiler/runtime cause or approve an exception.
- **Routing decision**: Resolved. LLVM is the development default for build,
  check, lint, documentation, and test commands. Reconsider Cranelift at the
  issue review date, or earlier if a new backend fix is demonstrated.

## Notes for Executing Agent

The focused tests described here have been run. Their evidence is recorded
below. Preserve the approved LLVM default until the issue review; do not infer
that Cranelift can be re-enabled from a passing test on another target or
toolchain.

### H2: The panic failures come from Cargo or the development wrappers

**Claim**: Removing Cargo, dependencies, the parallel frontend, and the
development linker from the experiment makes the unchanged panic-probe source
pass under Cranelift.

**Result**: Falsified on the pinned `nightly-2026-03-26` compiler. The same
source compiled and passed under LLVM (compile exit 0, test exit 0); under
Cranelift it compiled (exit 0) and failed the panic probes (test exit 101).
This isolates the observed behaviour from Cargo routing and the development
flags, but does not identify the compiler or runtime cause.

The experiment used bare `rustc` with the original
`tests/build_standard_panic.rs` source. Logs are
`/tmp/mapsplice-cranelift-plan-20261001/h2-llvm-run.out` and
`/tmp/mapsplice-cranelift-plan-20261001/h2-cranelift-run.out`.

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

The historical pinned full `make test` run failed the same two panic tests; 212
of 293 tests did not run after the abort. See
`/tmp/test-034e7665-13aa-4a68-b7b0-1ce103d5e371-rust-baseline-hardening-mapsplice-20260930T0300.out`.
The explicit LLVM full suite passed 293/293. The current approved route uses
LLVM for all development gates. Issue #115 names `leynos` as owner in its body
and schedules review for 2027-04-01; GitHub assignee metadata was unavailable
to this run. A future change to the route needs fresh evidence and a recorded
decision.
