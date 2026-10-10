# Architectural decision record (ADR) 001: Adopt the Rust build standard for development builds

## Status

Accepted on 8 October 2026. Development, test, lint and typecheck builds use
the parallel `rustc` frontend (`-Zthreads=8`) and, on Linux, the `mold` linker,
while coverage and release builds stay off the fast flags.

## Date

2026-10-08.

## Context and problem statement

Edit-compile cycles are dominated by frontend time and by link time. The
estate's build standard (`rust-build-defaults`) addresses both. Cargo applies
exactly one `rustflags` source, and an assigned `RUSTFLAGS` replaces every
configuration source, so a recipe or workflow step that assigns `RUSTFLAGS`
silently loses the flags unless it restates them. Measurements and shipped
artefacts also need a baseline that the fast flags would disturb.

## Decision drivers

- Faster local and CI development builds without changing what the code means.
- Coverage numbers that do not depend on the fast flags.
- A release that ships from the platform linker.
- A flag lost through a recipe or workflow edit must fail a test, not pass
  quietly.

## Options considered

- Configure the flags only in `.cargo/config.toml`. This leaves every recipe
  and CI step that assigns `RUSTFLAGS` without them.
- Restate the flags in each recipe and step, held by a contract test. This is
  the option taken.
- Run coverage on the development backend. Cranelift does not emit the
  instrumentation coverage needs, so the build would report nothing useful.

## Decision outcome

`.cargo/config.toml` carries the flags in every `rustflags` source, and adds the
`mold` linker flag to the Linux table only. The Makefile composes the same
flags into each development recipe, keeping the caller's own `RUSTFLAGS`.
Coverage assigns its own flags and ignores the caller's. The release recipe
keeps the caller's flags and names neither fast flag, so it ships from the
platform linker; a direct `cargo build --release` takes the configuration's
flags unless `RUSTFLAGS` is assigned. Cranelift is the development-profile code
generator, selected in `.cargo/config.toml`; coverage selects LLVM explicitly
because instrumentation needs it.

`tests/build_standard_contract.rs` reads the Cargo configuration sources and
the commands `make -n` prints for each target, and the `setup-rust` steps of
the workflows, so a flag lost through a recipe or workflow edit fails there.

## Consequences

- Linux builds require `mold`. For `x86_64-unknown-linux-gnu`,
  `.cargo/config.toml` selects `clang` explicitly, so `clang` is required there.
- A change to a recipe or workflow step that assigns `RUSTFLAGS` must restate
  the flags, and the contract says which clause fails when it does not.
- Re-measure the suite under Cranelift on the next toolchain bump.

## Addendum (2026-10-10)

This addendum supersedes only the statements above about Cranelift selection
and review timing, the coverage backend, and inherited `RUSTFLAGS`. The current
toolchain is pinned to `nightly-2026-03-26`. LLVM is the development default;
Cranelift is excluded after the `catch_unwind` probe failed and a
spawned-thread panic aborted the test process on that toolchain. The
development route keeps the parallel frontend and uses target-aware Clang
wrappers with `mold` for supported Linux targets; other platforms use their
platform linker. Coverage explicitly selects LLVM, and `make release` selects
the LLVM release profile and uses the platform linker. Make recipes assign
`RUSTFLAGS` from `RUST_FLAGS` and the selected route flags; inherited
`RUSTFLAGS` is not retained. They filter development-owned flags from inherited
`CARGO_ENCODED_RUSTFLAGS` before composing the selected route.

Reconsider Cranelift under
[issue #115](https://github.com/leynos/mapsplice/issues/115) at its 2027-04-01
review. The
[investigation](debugging/debugging-plan-2026-09-30-mapsplice-cranelift-unwind.md)
records the unwind failure.
