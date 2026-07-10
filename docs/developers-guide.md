# Mapsplice developers' guide

This guide is for maintainers and contributors changing `mapsplice` internals,
library APIs, command-line behaviour, tests, or build tooling.

## Spelling policy

Run `make spelling` to enforce en-GB-oxendict prose spelling. The generated
`typos.toml` starts from the shared estate dictionary, refreshes its untracked
local cache only when the authority is newer, and then applies the narrow
repository policy in `typos.local.toml`. Edit the local policy and regenerate
the configuration rather than changing generated entries by hand.

## 1. Normative references

The source-of-truth documents for internal changes are:

- [Design document](mapsplice-design.md) for architecture boundaries and
  roadmap grammar.
- [Accepted decision record and implementation plan](execplans/initial-tool.md)
  for the initial tool decisions.
- [Roadmap](roadmap.md) for planned structural work.
- [Contributing guide](contributing.md) for local prerequisites and quality
  gates.
- [Documentation style guide](documentation-style-guide.md) for Markdown
  conventions.

## 2. Architecture boundaries

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

## 3. Public library APIs

The library API is intentionally small:

- `run_from_args` executes the complete CLI workflow from command-line
  arguments and returns a `RunOutcome`.
- `run_request` executes an already parsed `CliRequest`.
- `parse_roadmap` and `parse_fragment` parse supported roadmap Markdown into
  typed domain structures.
- `parse_anchor` validates canonical positive anchors such as `8`, `8.2`,
  `8.2.3`, and `8.2.3.1`.
- `metrics_snapshot` returns bounded process-local counters for failures,
  in-place rewrites, and dependency rewrites.

Public APIs must return typed `MapspliceError` variants. Opaque reports belong
only at external process boundaries.

## 4. Configuration behaviour

Configuration behaviour has two current owners:

- `src/cli_config.rs` owns global `in_place` discovery and parsing. It reads
  `$XDG_CONFIG_HOME/mapsplice/config.toml` and local `./.mapsplice.toml`,
  applies `MAPSPLICE_IN_PLACE`, then lets `--in-place` / `-i` force `true`.
- `src/cli.rs::InsertConfig` owns the insert command's `after` option. It
  removes Clap's implicit `false` value when `--after` is absent, then merges
  defaults through `ortho_config::load_and_merge_subcommand_for`.
- Required values such as target paths, anchors, and fragment paths remain
  command-line arguments.

Future optional configuration settings must document which loader owns their
discovery path. Add or update `tests/roadmap_config.rs` coverage for every
source and precedence claim before changing the users' guide.

Configuration tests must serialize process environment and current-directory
mutation with the shared `ProcessStateGuard` in `tests/support/config.rs`.

## 5. Observability

Tracing spans exist at the process, filesystem, parse, splice, render, and
rewrite boundaries. Stable fields include operation, anchor, path, byte count,
phase count, and error class. Logs are disabled unless a subscriber is enabled
through standard tracing environment configuration.

`src/observability.rs` keeps bounded process-local counters. These are not
durable metrics; they exist to make failure and rewrite counts inspectable in
tests and embeddings without adding a metrics backend.

## 6. Verification layers

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

Task-list source preservation has a separate internal boundary from ordinary
Markdown-node source preservation. `src/roadmap/source_preservation.rs`
extracts source spans, while `StepSection::task_list_source` stores the exact
source for the first parsed task list in an unchanged step. Render validates
the task model before reusing that source, and mutation or dependency-rewrite
code must call `StepSection::clear_task_list_source` whenever the task list
itself changes.

Dependency-reference rewrite coverage is layered around the internal
`classify_dependency_reference` predicate in
`src/roadmap/ops/dependency_text.rs`. Unit tests cover the classifier branches,
behavioural tests cover unresolved valid references, invalid version-like
tokens, mapped rewrites, and scoped preservation through the compiled binary.
Property tests cover generated invalid dependency tokens, incidental numeric
text, scoped reference preservation beside mapped `Requires` references, and
append preservation across generated task-list shapes.

## 7. Local tooling

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
MARKDOWN_PATHS='docs/users-guide.md docs/developers-guide.md' make markdownfmt
MARKDOWN_PATHS='docs/users-guide.md docs/developers-guide.md' make markdownlint-paths
make markdownlint
make nixie
```

`MARKDOWN_PATHS` is a whitespace-separated list of existing Markdown paths to
format or lint. Use `make markdownfmt` for narrow Markdown maintenance;
`make fmt` remains repository-wide and can reformat unrelated Markdown files.

`make nixie` validates Mermaid diagrams in tracked Markdown files through the
CI-installed `merman-cli` renderer. The target runs one Markdown file at a time
and defaults both `NIXIE_MAX_CONCURRENCY` and `NIXIE_RENDERER_THREADS` to `1`,
so the default command is the serial comparison path used to prove CI
determinism. Contributors can still override the renderer job cap with, for
example, `NIXIE_MAX_CONCURRENCY=2 make nixie` when investigating local
performance, but the serial default is the gate that must pass before
committing Markdown changes.
