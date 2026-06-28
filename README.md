# mapsplice

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

- [ ] 9.1.1. Added task. Requires 9.1.1.
EOF

mapsplice append target.md fragment.md > updated.md
```

Use `-i` or `--in-place` to rewrite the target file instead of printing the
result.

______________________________________________________________________

## Local build prerequisites

Local builds use the pinned nightly toolchain in
[`rust-toolchain.toml`](rust-toolchain.toml) and the build configuration in
[`.cargo/config.toml`](.cargo/config.toml). The repository requires
`codegen-backend = "cranelift"` plus the configured Rust flags, including
`-Zthreads=8` and `link-arg=-fuse-ld=mold`, during builds.

Provision the core local tooling with:

```bash
rustup toolchain install nightly-2026-03-26 \
  --component rustfmt \
  --component clippy \
  --component rustc-codegen-cranelift-preview
rustup default nightly-2026-03-26
```

Install `clang` and `mold` through a system package manager when they are not
already present:

```bash
sudo apt install clang mold
brew install llvm mold
```

For platforms without a packaged `mold`, use the published binaries at
<https://github.com/rui314/mold/releases>. The full maintainer workflow lives in
[docs/contributing.md](docs/contributing.md).

______________________________________________________________________

## Features

- Parse roadmap phases from level-2 headings, steps from level-3 headings, and
  tasks from numbered Markdown list items.
- Support `append`, `insert`, `delete`, and `replace` operations.
- Enforce strict level matching so a task fragment cannot be inserted where a
  phase belongs.
- Renumber downstream phases, steps, and tasks after every structural edit.
- Rewrite matching dependency references found in roadmap text nodes.

______________________________________________________________________

## Learn more

- [Users' Guide](docs/users-guide.md) — command semantics, worked examples, and
  roadmap format rules
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
