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
test profile makes the focused probes pass before deciding how the repository
can meet the development-build baseline.

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

### Information Gaps

The full Cranelift component provenance is not recorded in the Rust toolchain
manifest. Current upstream documentation says panic unwinding is unsupported or
experimental, but it does not establish the behaviour of the component
installed for this pinned nightly. The first focused attempt set
`CARGO_PROFILE_TEST_PANIC=unwind`; Cargo warned that the panic setting is
ignored for the test profile and reused a cached executable. That result was
inconclusive because rustc did not receive the requested strategy. Two later
attempts confirmed both Cranelift and `-C panic=unwind` reached rustc, but the
compiler could not create a helper thread before the test target recompiled.
The first resource snapshot showed 512 tasks against a limit of 574. A later
read-only host inspection showed 9,151 Linux threads and 8,133 tasks in
`lody-daemon.service` against `pids.max=8192`; a subsequent read showed about
7,990 tasks. The current process listing is about 8,984 threads. These are
different snapshots and scopes; the small latest decrease is not enough to
justify another compile attempt. No process was stopped.

______________________________________________________________________

## Hypotheses

### H1: Passing unwind directly to rustc resolves the Cranelift failures

**Claim**: The failing Cranelift tests can unwind if rustc receives
`-C panic=unwind`; forcing that compiler flag on the focused test target will
make the `catch_unwind`, joined-thread panic, and `#[should_panic]` probes pass.

**Plausibility**: Low — upstream Cranelift documentation currently lists panic
unwinding as unsupported or experimental, but the exact installed component is
unverified and Cargo's test-profile environment override was ignored.

**Prediction**: If this hypothesis holds, the focused target will pass all
three tests when Make passes `-C panic=unwind` through `RUST_FLAGS`, while
retaining the repository's Cranelift backend and development flags.

#### H1 Falsification Plan

| Step | Action                                                                                                                                                                                                  | Expected Negative Result                                                                                                                                            |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | Run only `build_standard_panic` through `make test` with `RUST_FLAGS='-D warnings -C panic=unwind'`, `CARGO_PROFILE_TEST_CODEGEN_BACKEND=cranelift`, verbose Cargo output, and one test-harness thread. | Rustc receives both `-Zcodegen-backend=cranelift` and `-C panic=unwind`, yet `catch_unwind` still fails or the joined-thread panic still aborts. This falsifies H1. |
| 2    | If rustc does not receive both settings or the target is reused without recompilation, report the experiment as inconclusive. Do not alter repository configuration or run the full suite.              | Missing effective settings cannot discriminate H1.                                                                                                                  |

**Tooling**: Run
`make test DOC_TEST_TARGETS=false
TEST_CMD='test --test build_standard_panic'
TEST_FLAGS='-vv -- --test-threads=1' RUST_FLAGS='-D warnings -C panic=unwind'`
with `CARGO_PROFILE_TEST_CODEGEN_BACKEND=cranelift` and `CARGO_BUILD_JOBS=1`
in the command environment. The job limit avoids the observed shared process
limit while retaining the compiler's configured frontend flags. Capture the
exact rustc invocation and test result. The alchemist may run only this focused
experiment.

**Confidence on falsification**: High if rustc receives `-C panic=unwind` and
`-Zcodegen-backend=cranelift` and the same panic failures remain. A successful
focused run supports H1 but still requires the full suite and backend evidence.

______________________________________________________________________

## Recommended Execution Order

1. **H1** — This single focused run tests explicit rustc unwind support without
   repeating the full workspace suite.

## Termination Criteria

- **Root cause identified**: H1 is falsified by a confirmed Cranelift and
  `panic=unwind` invocation with the same failures, or supported by all three
  focused tests passing under those explicit settings.
- **Escalation trigger**: The setting is not observable in rustc output, the
  focused target fails to compile for an unrelated reason, or results differ
  from the two full-suite observations.

## Notes for Executing Agent

Use the current delivery worktree and shared Cargo cache. No repository gate
may run concurrently with this experiment. Do not edit files, change the
default backend, modify tests, or infer an approved exception from the result.
Return the command, exact rustc panic setting, three test results, and log path.

## Latest experiment disposition

The initial `CARGO_PROFILE_TEST_PANIC=unwind` run was inconclusive because
Cargo ignored the setting for the test profile and reused a cached executable.
The explicit `-C panic=unwind` retry and its `CARGO_BUILD_JOBS=1` retry both
reached rustc with Cranelift selected, but Cranelift failed to create a helper
thread while compiling a dependency. Neither run reached the panic target, so
H1 remains inconclusive. Logs:

- `/tmp/cranelift-unwind-mapsplice-rust-baseline-hardening-20260930T001500Z.out`
- `/tmp/cranelift-explicit-unwind-mapsplice-rust-baseline-hardening-20260930T002800Z.out`
- `/tmp/cranelift-explicit-unwind-jobs1-mapsplice-rust-baseline-hardening-20260930T004500Z.out`

The full suite evidence remains 291/293 under the development Cranelift route
and 293/293 under the explicit LLVM route. Do not infer a Cranelift exception
from these results; the user's approved resolution is still required. Wait for
a material reduction in shared thread pressure before requesting another
minimal alchemist experiment.
