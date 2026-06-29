# Mapsplice contributing guide

This guide is for maintainers and contributors working on `mapsplice` itself.
It covers the local build prerequisites and repository gates needed to match
Continuous Integration (CI).

For architecture, public API, observability, and verification guidance, see the
[developers' guide](developers-guide.md).

## Local build prerequisites

The repository uses the pinned nightly toolchain in
[`rust-toolchain.toml`](../rust-toolchain.toml) and build settings in
[`.cargo/config.toml`](../.cargo/config.toml). Local builds must use the
Cranelift code generation backend through `codegen-backend = "cranelift"` and
the configured Rust flags, including `-Zthreads=8` and `link-arg=-fuse-ld=mold`.

Provision a matching local environment with:

```bash
rustup toolchain install nightly-2026-03-26 \
  --component rustfmt \
  --component clippy \
  --component rustc-codegen-cranelift-preview
```

The checked-in `rust-toolchain.toml` selects this toolchain for commands run in
the repository.

Install `clang` when it is not already present:

```bash
sudo apt install clang
brew install llvm
```

Install `mold` through the system package manager or from the published binary
releases at <https://github.com/rui314/mold/releases>:

```bash
sudo apt install mold
brew install mold
```

The `.cargo/config.toml` file enables the required unstable Cargo and Rust
compiler features for this repository. Do not override those settings during
normal development; they keep local builds aligned with CI.

## Development gates

Run the standard repository gates before committing code or documentation
changes:

```bash
make check-fmt
make lint
make typecheck
make test
make markdownlint
make nixie
```

`make check-fmt`, `make lint`, `make typecheck`, and `make test` are required
for Rust changes. `make markdownlint` and `make nixie` are required for
Markdown changes, especially documents with Mermaid diagrams.
