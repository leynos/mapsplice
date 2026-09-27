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

Which answer a reference resolves through is a separate, proven boundary.
`RenumberPlan::resolve_reference` in `src/roadmap/model.rs` calls
`select_resolution` in `src/roadmap/ops/remap_kernel.rs`, a pure `const fn`
over two already-computed anchors and a `bool` source flag; it performs no
lookup and cannot fail. The kernel's body is shared as a `macro_rules!`
definition in `src/roadmap/ops/select_resolution.macro.rs`, expanded by both
the production `const fn` and the proof in `verus/lib.rs`, so the verified text
and the compiled text are one artefact rather than two that can drift. The body
is domain-owned and the proof reaches into `src/` for it, so the dependency
runs from verification infrastructure to the domain kernel rather than the
reverse. The splice is a macro rather than a bare `include!` because Whitaker's
`bumpy_road_function` lint cannot see through `include!` and aborts the
compiler; `verus/lib.rs` documents that convention, and
[mapsplice-design.md](mapsplice-design.md) records the dependency-clause
grammar the rewriter recognizes.

`TaskEntry::body_children_mut` in `src/roadmap/model_task_entry.rs` is what
lets a rewrite reach a `Requires` clause in a nested task-body bullet: it
exposes the task's structural child sequence, which is where the parser drains
nested body Markdown. A clause is reachable through a task's body blocks rather
than through its summary. `docs/verification.md` records the proof obligations
and the boundary of what is proven — the clause recognizer is not verified.

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

### 7.1 Verus proofs

`make verus` verifies the production-used kernels, and `make verus-selftest`
runs a deliberately false proof that must be rejected. Both are required in CI:
a run whose verifier never started reports success on the first target and
failure on the second, so the pair cannot pass vacuously. `make lint` depends on
`check-verification-ledger`, which requires `rg` (ripgrep) on `PATH` — a fresh
environment without it fails the lint gate before compiling anything.

The pinned inputs are `tools/verus/VERSION` (the release),
`tools/verus/SHA256SUMS` (its archive digests), and
`tools/rust-prover-tools/REF` (the commit of the runner that resolves and
invokes it). `tests/verus_harness.rs` asserts the workflow's `VERUS_VERSION`
matches `tools/verus/VERSION`, because the CI cache key derives from it and a
mismatch would silently restore the wrong verifier.
`scripts/check-verification-ledger.sh` requires every symbol named in the
ledger's claim table to exist as a declaration below `src/`, so a renamed
kernel cannot leave a claim behind as prose. A row whose executable-function
cell reads `Pending` is exempt, because it declares work that has not landed
rather than asserting a result.

Proofs complement the property and end-to-end tests rather than replacing them.
The kernel's trusted boundary and the obligations that were deliberately left
unproved are recorded in [verification.md](verification.md); the Markdown
clause recognizer is outside every proof, which is why the CLI regressions and
golden fixtures carry that part of the contract.

### 7.2 The domain-purity gate

`make lint` also depends on `check-domain-purity`, which runs
`scripts/check-domain-purity.sh`. The roadmap domain owns the Markdown model,
the splice, and the rendering; whatever reads a file, reads an environment
variable, or walks the filesystem belongs to an adapter that calls the domain,
never to the domain itself. Filesystem access lives in `src/fs.rs`. The gate
scans the production sources under `src/roadmap` and fails if any of them names
infrastructure — `fs`, `path`, `io`, `process`, `env`, `net`, or `thread` —
through a `use` declaration, a grouped import, a qualified module path, a
process call, or a build-time macro such as `env!` or `include_str!`. The rule
matters because nothing in the compiler enforces it: the resolved kernel is
spliced with a macro, and the obvious way to locate a file for `include!` is
`env!("CARGO_MANIFEST_DIR")`, which an earlier revision of this repository did.

Test files are exempt by name, since `src/roadmap/render_tests.rs` drives the
rendering path through the built binary and legitimately names `std::process`.
The exemption is by file rather than by region, so no brace count has to be
trusted, and `tests/domain_purity.rs` asserts it from both sides: the same text
in a production file must fail.

Like the verification-ledger check, the gate is necessary rather than
sufficient, and matching is textual. A name surviving only in a doc comment
triggers it, so a comment discussing an escape hatch is treated as an escape
hatch — the conservative direction, and visible in review rather than silent.

`MARKDOWN_PATHS` is a whitespace-separated list of existing Markdown paths to
format or lint. Use `make markdownfmt` for narrow Markdown maintenance;
`make fmt` remains repository-wide and can reformat unrelated Markdown files.
The `make fmt` and `make check-fmt` targets select tracked and unignored
Markdown files. They deliberately exclude
`tests/fixtures/golden/insert_task_preserves_indented_code_markers/target.md`
and `expected.md`, whose non-contiguous ordered-looking lines are indented-code
fixtures for byte-identical source preservation and are not formatter-stable.

`make nixie` validates Mermaid diagrams in tracked Markdown files through the
CI-installed `merman-cli` renderer. The target runs one Markdown file at a time
and defaults both `NIXIE_MAX_CONCURRENCY` and `NIXIE_RENDERER_THREADS` to `1`,
so the default command is the serial comparison path used to prove CI
determinism. Contributors can still override the renderer job cap with, for
example, `NIXIE_MAX_CONCURRENCY=2 make nixie` when investigating local
performance, but the serial default is the gate that must pass before
committing Markdown changes.

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
