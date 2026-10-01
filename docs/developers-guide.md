# Mapsplice developers' guide

This guide is for maintainers and contributors changing `mapsplice` internals,
library APIs, command-line behaviour, tests, or build tooling.

## Spelling policy

Run `make spelling` to enforce en-GB-oxendict prose spelling. The gate
regenerates `typos.toml` on every run from the live shared estate dictionary
and the `typos.local.toml` overlay, so a word added to the shared dictionary
needs no change here. Because the dictionary is live, `typos.toml` is never
drift checked in continuous integration. Add narrow repository-specific
identifier, API, proper-name, or fixture exceptions to `typos.local.toml`;
hand-editing `typos.toml` is not supported and any edits are overwritten on the
next run.

`TYPOS_CONFIG_BUILDER_VERSION` in the `Makefile` pins the
`typos-config-builder` release the gate runs (currently `v0.1.3`). Raise it
together with the regenerated `typos.toml`, never on its own.

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
  in-place rewrites, dependency rewrites, and source-preservation outcomes.

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

`MetricsSnapshot` reports `preserved_source_renders`,
`preserved_source_invalidations`, and `canonical_fallbacks` for the aggregate
source-preservation outcomes. It also reports the reason-specific counters
`invalidations_renumber`, `invalidations_dependency_rewrite`,
`invalidations_child_mutation`, `canonical_fallbacks_unstable_list_marker`, and
`canonical_fallbacks_unstable_code_fence`. All counters are atomic and
process-local; they are snapshots for diagnostics and tests, not durable
telemetry.

The closed `PreservationInvalidationReason` set is `Renumber`,
`DependencyRewrite`, and `ChildMutation`. The closed `CanonicalFallbackReason`
set is `UnstableListMarker` and `UnstableCodeFence`. Invalidation is recorded
at task or sub-task mutation points in `src/roadmap/model_task_entry.rs` and
`src/roadmap/ops/rewrite.rs`. Preservation and canonical-fallback outcomes are
recorded at the source-preservation rendering decision in
`src/roadmap/render_preservation.rs`.

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
itself changes. Target parsing also captures each task and addendum sub-task's
list-item source in `TaskEntry::original_source` and
`SubTaskEntry::original_source`; fragment items do not receive preserved source.
`src/roadmap/model_task_entry.rs` owns the per-item accessors and structural
invalidation, while `src/roadmap/render_task.rs` reuses preserved source before
falling back to canonical rendering. Clear an item only when its own number or
text changes, when dependency-related content changes, or when a structural
descendant changes. Preserve the source only when it is formatter-stable;
otherwise use the canonical fallback. Unchanged task and sub-task chunks remain
byte-identical, including continuation indentation. Invalidated or
formatter-unstable items use canonical rendering with the two-space
continuation convention.

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
Cranelift code generation through `codegen-backend = "cranelift"`, the parallel
rustc frontend, and, on supported Linux targets, `clang` plus pinned `mold`.
Cargo discovers these defaults from `.cargo/config.toml`; Make restates the
flags on gate recipes that set `RUSTFLAGS`. Run `make install-build-tools` and
`make check-build-tools` before development builds. The pinned toolchain
includes rustfmt, Clippy, rust-analyzer, LLVM tools, and Cranelift. Coverage,
release, and Whitaker use explicit non-development routes; see
[the contributing guide](contributing.md) for local setup.

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
The `make fmt` and `make check-fmt` targets select tracked and unignored
untracked Markdown files directly through mdtablefix. The two indented-code
golden fixtures that are not formatter-stable have `.txt` extensions; their
contents remain byte-identical to the authored test data.

`make nixie` validates Mermaid diagrams in tracked Markdown files through the
CI-installed `merman-cli` renderer. The target runs one Markdown file at a time
and defaults both `NIXIE_MAX_CONCURRENCY` and `NIXIE_RENDERER_THREADS` to `1`,
so the default command is the serial comparison path used to prove CI
determinism. Contributors can still override the renderer job cap with, for
example, `NIXIE_MAX_CONCURRENCY=2 make nixie` when investigating local
performance, but the serial default is the gate that must pass before
committing Markdown changes.

## The build standard

Development, test, lint, and typecheck builds use the parallel `rustc` frontend
(`-Zthreads=8`) and, on Linux, the `mold` linker (`-Clink-arg=-fuse-ld=mold`).
These are defaults in `.cargo/config.toml`, which Cargo discovers on its own,
so a bare `cargo build` gets them. `mold` ships for Linux only, so the linker
flag lives in a Linux-only table and macOS and Windows keep their platform
linker. Cargo selects one `rustflags` source rather than merging them, so every
source repeats the same flags apart from the linker.

An assigned `RUSTFLAGS` replaces the configuration's flags, so the Makefile
recipes that set it compose the standard's flags onto any inherited value (CI's
`setup-rust` exports one). Two builds are deliberately excluded: coverage
assigns `RUSTFLAGS` without the fast flags, because a measurement should not
depend on them, and the release recipe and workflow keep the platform linker,
because they assign `RUSTFLAGS` (even an empty value displaces the
configuration). Cargo has no per-profile `rustflags`, so a direct
`cargo build --release` takes the configuration's flags unless `RUSTFLAGS` is
assigned too.

On Linux, install `mold` before building: the configuration names it, so a
build without it fails at link time. CI installs it through `setup-rust`'s
`install-mold` input. `tests/build_standard_contract.rs` holds the standard. It
reads the configuration sources, the commands `make -n` prints for each
development target on a Linux host and a macOS host (each keeping the caller's
own `RUSTFLAGS`) and for the release target (the coverage exclusion is checked
in the workflow steps) on a Linux host, and the `setup-rust` steps of the CI
workflows (each must pass `install-mold`), so a flag lost through a recipe or
workflow edit fails there. The decision is recorded in
[ADR 001](adr-001-rust-build-standard.md). The contract runs `make -n`, so a
direct `cargo test` needs GNU make on the `PATH`. It fails when `make` is
missing instead of skipping, so a missing tool cannot read as a pass.

### Cold-cache allowance for the trybuild tests

`.config/nextest.toml` keeps the 180 s per-test allowance that the coverage
action writes when a repository has no file of its own, and gives the
`compile_time` tests three times that. They run a nested Cargo that compiles
the whole dependency graph, and a pull request has no warm sccache directory
until the default branch has written one (`setup-rust` owns that directory and
only a push to the default branch writes it). Before that cache exists, the
nested build alone can exceed 180 s on a GitHub-hosted runner.

### Cranelift

Cranelift is the development-profile codegen backend. The full suite was
measured under it on the pinned `nightly-2026-03-26` on 2026-09-28: all 290
nextest tests and the doctests pass. Coverage selects LLVM explicitly
(`CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm`), because instrumentation needs it,
and release builds use the release profile, which Cranelift does not touch.
Re-measure the whole suite on the next toolchain bump; if it fails, record the
failing tests here as an exception and remove the backend from
`.cargo/config.toml`.

## 8. Workflow pins and Dependabot

Dependabot owns the upgrade of GitHub Actions and reusable workflows, including
calls into `leynos/shared-actions`. Contract tests that assert a caller's exact
commit SHA create a lockstep dependency: every time Dependabot opens a bump PR,
the test fails until a human edits the pinned constant to match. That defeats
the purpose of automated dependency updates and turns a routine bump into a
manual chore.

Contract tests may still verify the *shape* of a reusable-workflow caller. They
must not verify the specific SHA value.

- Do assert the workflow references the correct reusable workflow path.
- Do assert the ref is pinned to a full 40-character commit SHA, not a
  mutable branch such as `main` or `rolling`.
- Do assert the expected `on:` triggers, least-privilege `permissions:`, and
  the inputs the caller relies on.
- Do not hard-code the current SHA value as an expected string. Match it with
  a pattern instead.
- Do not fail a test purely because Dependabot bumped the pinned SHA.

```python
import re

SHA_RE = re.compile(r"^[0-9a-f]{40}$")


def test_uses_pinned_full_sha(caller_step):
    ref = caller_step["uses"].split("@")[-1]
    assert SHA_RE.match(ref), f"expected a 40-hex commit SHA, got {ref!r}"
```

If a workflow's behaviour genuinely depends on a feature only present from a
particular commit onwards, express that as a comment or a changelog note, not
as a test assertion on the SHA string.

## Lint baseline

[`Cargo.toml`](../Cargo.toml) holds the package's Clippy, Rust, and rustdoc
lint levels. [`clippy.toml`](../clippy.toml) sets the complexity ceiling to 9,
the argument limit to 4, the line limit to 70, and the nesting limit to 4. The
selected estate baseline is Concordat revision
`902d034d9da8e7ca33a0d4032770519dd1609de2`; this repository also requires
`missing_assert_message`, `missing_docs_in_private_items`, and
`unsafe_code = "forbid"`. Fix findings in source. Any unavoidable exception
needs a narrow scope and a stated reason; do not use `#[allow]` to carry a
backlog.

This package has no Cargo workspace. If it gains one, put the authoritative
tables under `[workspace.lints.*]` and make every member inherit them with
`[lints] workspace = true`. Use the components in
[`rust-toolchain.toml`](../rust-toolchain.toml), including rustfmt, Clippy, and
Cranelift; the build and verification routes also require their documented
tools. Keep environment access at an explicit CLI configuration boundary and
inject an environment reader into code that needs deterministic testing. The
`disallowed_methods` lint level is enabled, but an approved method list has not
yet been selected for this repository; it does not currently enforce the
environment-access rule.
