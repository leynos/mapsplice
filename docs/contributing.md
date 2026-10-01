# Mapsplice contributing guide

This guide is for maintainers and contributors working on `mapsplice` itself.
It covers the local build prerequisites and repository gates needed to match
Continuous Integration (CI).

For architecture, public API, observability, and verification guidance, see the
[developers' guide](developers-guide.md).

## Local build prerequisites

The repository uses the pinned nightly toolchain in
[`rust-toolchain.toml`](../rust-toolchain.toml) and the auto-discovered
[`.cargo/config.toml`](../.cargo/config.toml). Bare Cargo development commands
use Cranelift, the parallel compiler frontend (`-Zthreads=8`), and, on
supported Linux targets, `clang` with the pinned `mold` linker. Standard Make
build, test, lint, and typecheck routes use the same development flags even
where their recipes assign `RUSTFLAGS` to deny warnings.

Provision the matching local toolchain and pinned linker with:

```bash
make install-build-tools
make check-build-tools
```

The checked-in toolchain file selects the nightly, rustfmt, Clippy,
rust-analyzer, LLVM tools, and the Cranelift component. The install target
verifies the published `mold` binary by SHA-256 on supported Linux
architectures. Put `$HOME/.local/bin` on `PATH` to use it locally. The check
target fails with an installation hint if a required component or tool is
missing.

Install `clang` when it is not already present:

```bash
sudo apt install clang
brew install llvm
```

Coverage sets LLVM profiles and an explicit warnings-only `RUSTFLAGS` value;
release builds and the Whitaker driver also exclude the development flags.
Installed development tools are therefore not active on every route. The
mutation workflow installs the same pinned linker before its test-bearing
builds. The `.cargo/config.toml` file is the default for ordinary development;
avoid overriding it without checking the route you intend to run.

## Development gates

Run the standard repository gates before committing code or documentation
changes:

```bash
make check-fmt
make lint
make typecheck
make test
make fmt
make markdownlint
make spelling
make nixie
```

`make check-fmt`, `make lint`, `make typecheck`, and `make test` are required
for Rust changes. `make fmt` formats Markdown; `make markdownlint`,
`make spelling`, and `make nixie` are distinct required checks for Markdown
changes, especially documents with Mermaid diagrams. `make nixie` uses the
CI-installed `merman-cli` renderer, validates tracked Markdown files one at a
time, and defaults `NIXIE_MAX_CONCURRENCY=1` plus `NIXIE_RENDERER_THREADS=1`
for deterministic serial validation. Override `NIXIE_MAX_CONCURRENCY` only when
comparing local renderer concurrency; the default command remains the required
gate.
