# mapsplice

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](
https://deepwiki.com/leynos/mapsplice)

*Splice one roadmap into another without spending your afternoon renumbering
phases, steps, and task dependencies by hand.*

`mapsplice` is a small Rust CLI for teams who keep their plans in Markdown and
would rather automate the fiddly bits. It parses a constrained roadmap format,
applies structural edits, renumbers the affected items, and rewrites matching
dependency references on the way out.

______________________________________________________________________

## Why mapsplice?

- **Make roadmap edits mechanical**: Insert, replace, or delete sections
  without manually renumbering the rest of the document.
- **Keep dependency prose in sync**: References such as `Requires 8.2.3.` get
  rewritten when the target item moves.
- **Stay review-friendly**: The default mode writes the rewritten roadmap to
  standard output, so changes can be inspected or diffed before replacing the
  source file.

______________________________________________________________________

## Quick start

### Installation

```bash
cargo install --path .
```

### Basic usage

```bash
cat > target.md <<'EOF'
## 1. Phase one

### 1.1. Step one

- [ ] 1.1.1. First task.
EOF

cat > fragment.md <<'EOF'
## 9. Inserted phase

### 9.1. Added step

- [ ] 9.1.1. Added task. Requires 1.1.1.
EOF

mapsplice append target.md fragment.md > updated.md
```

Use `-i` or `--in-place` to rewrite the target file instead of printing the
result.

______________________________________________________________________

## Local build prerequisites

Local builds use the pinned nightly toolchain in
[`rust-toolchain.toml`](rust-toolchain.toml) and the build configuration in
[`.cargo/config.toml`](.cargo/config.toml). Development builds use LLVM with
the configured Rust flags, including `-Zthreads=8` and `link-arg=-fuse-ld=mold`
on supported Linux targets. Cranelift is excluded from the development default
because the pinned backend fails panic-unwinding probes; see
[issue #115](https://github.com/leynos/mapsplice/issues/115).

Provision the core local tooling with:

```bash
rustup toolchain install nightly-2026-03-26 \
  --component rustfmt \
  --component clippy \
  --component rust-analyzer \
  --component llvm-tools
```

The checked-in `rust-toolchain.toml` selects this toolchain for commands run in
the repository.

On Linux, install `clang` through your system package manager if it is not
already present. Then run the repository installer:

```bash
make install-build-tools
```

The installer provisions the pinned Rust toolchain components on all platforms
and provisions `mold` with its matching `ld.mold` 2.41.0 only on supported
Linux hosts. macOS keeps its platform linker. The full maintainer workflow
lives in [docs/contributing.md](docs/contributing.md).

______________________________________________________________________

## Features

- Parse roadmap phases from level-2 headings, steps from level-3 headings, and
  tasks from numbered Markdown list items.
- Support `append`, `insert`, `delete`, and `replace` operations.
- Enforce strict level matching so a task fragment cannot be inserted where a
  phase belongs.
- Renumber downstream phases, steps, and tasks after every structural edit.
- Rewrite matching dependency references in supported `Requires` clauses.
- Export a deterministic, versioned JSON roadmap and detect drift against a
  committed export; see the [users' guide](docs/users-guide.md#export) and the
  [roadmap export format](docs/roadmap-export-format.md).

______________________________________________________________________

## Learn more

- [Users' Guide](docs/users-guide.md) — command semantics, worked examples,
  roadmap format rules, and compatibility and migration notes
- [Migration notes for 0.2.0](docs/migrations/0.2.0.md) — unreleased CLI help
  presentation and diagnostic logging changes
- [Developers' Guide](docs/developers-guide.md) — APIs, architecture,
  observability, and verification layers
- [Implementation plan](docs/execplans/initial-tool.md) — design decisions and
  architecture notes
- [Contributing guide](docs/contributing.md) — local prerequisites and quality
  gates
- [Agent instructions](AGENTS.md) — repository workflow and coding standards

______________________________________________________________________

## Licence

ISC — see [LICENSE](LICENSE) for details.

______________________________________________________________________

## Contributing

Contributions are welcome. Please read [AGENTS.md](AGENTS.md) before opening a
pull request so the repository's gate and commit workflow stays intact.
