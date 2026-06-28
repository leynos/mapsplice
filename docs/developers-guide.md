# Mapsplice developers' guide

This guide is for maintainers and contributors changing `mapsplice` internals,
library APIs, command-line behaviour, tests, or build tooling.

## Architecture boundaries

`mapsplice` is split into a narrow binary adapter, a library application
boundary, and roadmap domain modules:

- `src/main.rs` initializes tracing, calls `mapsplice::run_from_args`, writes
  roadmap output to standard output, and reports diagnostics on standard error.
- `src/lib.rs` owns the application workflow: parse CLI input, read the target
  and optional fragment, translate CLI commands into roadmap operations, render
  the result, and perform in-place writes.
- `src/cli.rs` is the command-line and `ortho-config` adapter. It exposes
  `CommandKind`, `GlobalOptions`, and `CliRequest`.
- `src/roadmap` owns domain parsing, mutation, renumbering, and rendering.
  `RoadmapOperation` is the domain command type; CLI command enums must be
  translated before entering `roadmap::ops`.
- `src/fs.rs` is the capability-oriented filesystem adapter. Filesystem
  failures must surface as `MapspliceError::Io`, not roadmap validation errors.

The roadmap model stores Markdown content in `MarkdownNodes`, a value object
that keeps parser nodes behind the parse/render boundary. New roadmap fields
should prefer typed domain values over raw parser or adapter types.

## Public library APIs

The library API is intentionally small:

- `run_from_args` executes the complete CLI workflow from command-line
  arguments and returns a `RunOutcome`.
- `run_request` executes an already parsed `CliRequest`.
- `parse_roadmap_text` and `parse_fragment_text` parse supported roadmap
  Markdown into typed domain structures.
- `parse_anchor` validates canonical positive anchors such as `8`, `8.2`, and
  `8.2.3`.
- `metrics_snapshot` returns bounded process-local counters for failures,
  in-place rewrites, and dependency rewrites.

Public APIs must return typed `MapspliceError` variants. Opaque reports belong
only at external process boundaries.

## Configuration behaviour

The CLI uses `ortho-config` for optional defaults:

- Global `in_place` may be supplied by `MAPSPLICE_IN_PLACE=true`, configuration
  files, or the `--in-place` / `-i` flags.
- Insert `after` may be supplied by `MAPSPLICE_CMDS_INSERT_AFTER=true`,
  `[cmds.insert] after = true`, or `--after`.
- Required values such as target paths, anchors, and fragment paths remain
  command-line arguments.

Configuration tests must serialize process environment mutation with the shared
test guard in `tests/support/mod.rs`.

## Observability

Tracing spans exist at the process, filesystem, parse, splice, render, and
rewrite boundaries. Stable fields include operation, anchor, path, byte count,
phase count, and error class. Logs are disabled unless a subscriber is enabled
through standard tracing environment configuration.

`src/observability.rs` keeps bounded process-local counters. These are not
durable metrics; they exist to make failure and rewrite counts inspectable in
tests and embeddings without adding a metrics backend.

## Verification layers

The test suite has four layers:

- `rstest` unit tests cover parser, splice, configuration, and error behaviour.
- `rstest-bdd` scenarios exercise the compiled binary through user workflows.
- `proptest` properties cover canonical anchor round-trips and generated
  dependency rewrites across multiple insertion points.
- `trybuild` and `insta` cover compile-time API compatibility and stable CLI
  help output.

Property tests should construct valid inputs rather than filter invalid data
after generation. Any shrunk failure should be promoted to a named regression
test when it captures a real bug.

## Local tooling

Local builds use the pinned nightly toolchain in
[`../rust-toolchain.toml`](../rust-toolchain.toml) and build settings in
[`../.cargo/config.toml`](../.cargo/config.toml). The repository requires
Cranelift code generation through `codegen-backend = "cranelift"`, `clang`, and
`mold` via `link-arg=-fuse-ld=mold`. The pinned toolchain must include
`rustc-codegen-cranelift-preview`.

Run these gates before committing Rust changes:

```bash
make check-fmt
make test
make typecheck
make lint
```

Run these gates for Markdown changes:

```bash
make markdownlint
make nixie
```
