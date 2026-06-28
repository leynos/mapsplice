# Plan mapsplice roadmap splicing CLI

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

`mapsplice` should let a maintainer splice roadmap content into another roadmap
without hand-editing numbering or dependency references. After this change, a
user will be able to run commands such as
`mapsplice insert docs/roadmap.md 8 new-phase.md` or
`mapsplice replace docs/roadmap.md 8 replacement.md` and receive a complete
renumbered roadmap on standard output by default, or overwrite the target with
`--in-place` / `-i`.

Success is observable in three ways. First, unit tests prove that parsing,
level validation, renumbering, and dependency rewrites behave correctly.
Second, behaviour tests prove the real binary edits files and standard output
as documented. Third, the tool can process a realistic roadmap structure like
the Wireframe example, which uses phase headings, step headings, and numbered
task list items with prose such as `Requires 9.1.2.` inside task bodies.

## Repository orientation

The repository is currently a stub. `src/main.rs` only prints a placeholder,
there is no library crate yet, and there is no project-specific user guide. The
shared reference material already in the repository is the main source of truth
for the implementation:

- `docs/ortho-config-users-guide.md` explains how `OrthoConfig` composes
  configuration, handles subcommands, and supports selected-subcommand merges.
- `docs/rstest-bdd-users-guide.md` explains how to structure `rstest-bdd`
  feature files, fixtures, and step state.
- `docs/documentation-style-guide.md` defines the documentation conventions
  that any new user-facing documentation must follow.

The external references that shape the plan are also important. The `markdown`
crate documentation shows a parser plus mdast (Markdown abstract syntax tree)
node types, but does not document a supported Markdown writer API in the same
way it documents `to_mdast()`. The sample Wireframe roadmap shows a consistent
grammar: document preamble, phase headings at level 2, step headings at level
3, and numbered task list items that may contain dependency prose inside the
task body.

## Scope and grammar assumptions

This plan assumes the supported roadmap grammar is intentionally narrow rather
than fully general Markdown editing. The implementation will accept roadmap
documents that follow this shape:

1. Optional document preamble before the first phase.
2. Phase headings encoded as level-2 Markdown headings whose visible text
   starts with a phase number such as `8.`.
3. Step headings encoded as level-3 Markdown headings whose visible text starts
   with a step number such as `8.2.`.
4. Tasks encoded as Markdown list items within a step, where the first inline
   text in the task starts with a task number such as `8.2.3.`.
5. Optional continuation paragraphs or nested bullets inside a task body.

The structural target of an operation is determined by the anchor supplied on
the command line:

- `8` targets a phase.
- `8.2` targets a step.
- `8.2.3` targets a task.

Files supplied to `insert` or `replace` must contain one or more contiguous
items at the same structural level as the target anchor. A task file cannot be
inserted at phase level. A phase file cannot be inserted after a task anchor.

This plan interprets `append <target> <file-to-append>` as phase-level append
to the end of the target roadmap, because no anchor is supplied. If approval
feedback requires append-at-step or append-at-task semantics, that is a
material scope change and should be handled as a revision to this plan before
implementation.

## Constraints

- Use `ortho-config` for CLI and configuration composition. The final CLI
  should not bypass `OrthoConfig` with a hand-rolled parser.
- Use the `markdown` crate for Markdown parsing and mdast-based structural
  analysis.
- The default behaviour must emit the rewritten roadmap to standard output.
- `--in-place` / `-i` must rewrite the target file instead of writing the
  result to standard output.
- Level matching is strict. Operations must reject mismatched source and target
  levels with a typed, user-facing error.
- Renumber every subsequent phase, step, and task affected by an operation.
- Rewrite dependency references that point to renumbered phases, steps, or
  tasks.
- Add unit tests with `rstest`.
- Add behaviour tests with `rstest-bdd`.
- Keep code files below 400 lines by splitting the implementation into focused
  modules.
- Use capability-oriented filesystem paths through `cap_std` and `camino`
  rather than `std::fs` and `std::path`.
- Keep this ExecPlan current while implementation is in flight.

## Tolerances (exception triggers)

- If supporting the requested behaviour would require a fully general Markdown
  round-trip writer rather than a roadmap-focused renderer, stop and escalate.
- If the `markdown` crate cannot provide stable position data for headings,
  list items, and inline text needed to rewrite dependency references, stop and
  escalate.
- If more than two additional external crates beyond the obvious runtime and
  test set are needed, stop and justify each dependency before adding it.
- If command semantics for `append` remain disputed after review, stop and
  resolve that ambiguity before implementing.
- If behaviour tests still fail after three full red-green iterations, stop and
  document the blocking gap.
- If the implementation needs to rewrite documentation files outside this
  repository's own README and user guide, stop and confirm the broader scope.

## Risks

- Risk: The `markdown` crate documents parsing and mdast nodes clearly, but its
  public documentation does not present a stable Markdown writer API for full
  round-tripping. Severity: high Likelihood: medium Mitigation: treat mdast as
  the parser and validation layer, project the roadmap into a purpose-built
  intermediate representation, and render only the supported roadmap grammar
  deterministically.

- Risk: Dependency references appear as prose such as `Requires 9.1.2.` rather
  than dedicated metadata. Severity: medium Likelihood: high Mitigation:
  rewrite only structural identifiers found in parsed text nodes whose token
  values match known roadmap IDs, and cover false-positive and false-negative
  cases with unit tests.

- Risk: A fragment file may contain malformed numbering that superficially
  matches Markdown structure but not the expected roadmap hierarchy. Severity:
  medium Likelihood: medium Mitigation: validate inserted fragments against the
  same roadmap grammar used for the target document before any mutation is
  attempted.

- Risk: `ortho-config` is more powerful than this tool strictly needs, which
  can tempt needless configuration surface area. Severity: low Likelihood:
  medium Mitigation: keep configuration narrow. Required positional arguments
  stay on the CLI, while optional flags such as `in_place` and future defaults
  are the only configuration-backed fields.

## Proposed implementation

### 1. Establish the domain model

Create a small set of newtypes and enums that model the roadmap rather than
passing raw strings around. At minimum this should include `PhaseNumber`,
`StepNumber`, `TaskNumber`, a parsed `RoadmapAnchor`, and a `RoadmapItemLevel`
enum. Each type should expose parsing, display, and ordering behaviour so the
renumbering logic is explicit and testable.

Project the mdast into a roadmap-specific intermediate representation:
`RoadmapDocument` for the full file, `PhaseSection`, `StepSection`, and
`TaskEntry`. Each node should retain the content required for re-emission, plus
the parsed numeric identifier and any source-position data needed for precise
dependency token rewriting inside text nodes.

### 2. Build the parser and validator on mdast

Parse target and fragment files with `markdown::to_mdast()` using options that
enable GitHub Flavoured Markdown constructs required by roadmap documents,
especially task lists and footnotes.

Walk the mdast root and enforce the roadmap grammar:

1. Collect preamble nodes before the first phase.
2. Recognize phase headings from level-2 headings with a leading phase number.
3. Recognize step headings from level-3 headings nested under the current
   phase.
4. Recognize task entries from list items nested under the current step.
5. Reject structure that breaks hierarchy, such as a step before a phase or a
   task without a containing step.

Validation should happen before mutation. If either the target or fragment file
is structurally invalid, the command must fail without producing partial output.

### 3. Implement splice operations against the roadmap model

Implement `append`, `insert`, `delete`, and `replace` against the intermediate
representation instead of editing strings directly.

- `append` adds one or more phase sections to the end of the target roadmap.
- `insert` places sibling items before the anchor by default, or after it when
  `--after` is present.
- `delete` removes exactly one item at the addressed level.
- `replace` swaps the addressed item with one or more sibling items from the
  fragment file.

Every operation should return a fresh `RoadmapDocument` plus a renumbering map
that records `old_id -> new_id` for every changed phase, step, and task. That
map becomes the only source of truth for dependency rewrites.

### 4. Renumber structure and dependency references

After mutation, renumber the roadmap in a single top-down pass:

1. Reassign sequential phase numbers.
2. Reassign step numbers within each phase using the new parent number.
3. Reassign task numbers within each step using the new parent step number.

Then traverse every text-bearing node captured in the roadmap model and rewrite
identifier tokens that exactly match a renumbered item in the map. The rewrite
must preserve surrounding prose and punctuation, so `Requires 9.1.2.` becomes
`Requires 10.1.2.` when the renumbering map says so. Tokens that are not known
roadmap identifiers must be left unchanged.

### 5. Render the supported roadmap grammar deterministically

Do not assume the `markdown` crate can serialize arbitrary mdast back to
Markdown in the required shape. Instead, emit a deterministic renderer for the
supported roadmap subset. The renderer should preserve:

- document preamble paragraphs and other non-roadmap blocks carried through the
  model,
- phase and step headings,
- task checkboxes and body paragraphs,
- nested bullet lists beneath tasks,
- inline formatting already present in the parsed content.

The renderer only needs to cover constructs accepted by the roadmap parser. If
an unsupported node type appears inside a roadmap task body, the parser should
fail with a clear error rather than emitting lossy output.

### 6. Build the CLI with ortho-config

Introduce a root CLI and subcommand enum using `clap` plus `OrthoConfig`.
Global options should be minimal:

- `--in-place` / `-i`
- optional configuration discovery controls if they come for free from
  `ortho-config`

Each subcommand should derive `OrthoConfig` so optional defaults can be loaded
through the standard `cmds.<subcommand>` namespace later, even if the initial
release only uses configuration for optional flags. Required positional values
such as target path, anchor, and fragment path should remain CLI-only inputs.

The implementation should parse the selected subcommand, merge its optional
configuration through the ortho-config pattern documented in
`docs/ortho-config-users-guide.md`, execute the splice, and either print the
result to standard output or atomically rewrite the target file.

### 7. Add tests before wiring the happy path

Start with failing unit tests and behaviour tests, then implement to green.

Unit tests should use `rstest` for:

- anchor parsing and level classification,
- roadmap grammar validation,
- append, insert, delete, and replace mutations,
- renumber maps for phases, steps, and tasks,
- dependency rewrite cases such as `Requires 9.1.2.` and
  `Depends on 8.2.3 and 8.2.4.`,
- level mismatch rejection,
- default standard-output mode versus in-place mode decision logic.

Behaviour tests should use `rstest-bdd` feature files and step definitions to
exercise the real binary. A `CliState` fixture should own a temporary working
directory, target file contents, fragment file contents, last command output,
and exit status. Scenarios should cover:

1. append emits the rewritten roadmap to standard output and leaves the target
   file unchanged,
2. insert before a phase renumbers later phases and dependencies,
3. insert `--after` a task renumbers later tasks within the step,
4. delete removes an addressed phase and rewrites downstream identifiers,
5. replace swaps a phase with multiple phases from a fragment file,
6. in-place mode rewrites the target file and emits no roadmap body on
   standard output,
7. level mismatch returns a clear failure.

### 8. Finish the user-facing documentation

Update `README.md` so the repository no longer looks like a generated stub.
Document the command forms requested by the user, the default standard-output
behaviour, the meaning of `--in-place`, and the supported roadmap grammar.

If the README becomes too dense, add `docs/users-guide.md` and link to it from
the README. Any new documentation must follow
`docs/documentation-style-guide.md`.

## File plan

The implementation should stay modular to satisfy the repository's size and
clarity constraints. A likely layout is:

```plaintext
src/main.rs
src/cli.rs
src/error.rs
src/fs.rs
src/lib.rs
src/roadmap/mod.rs
src/roadmap/model.rs
src/roadmap/parse/mod.rs
src/roadmap/parse/document.rs
src/roadmap/parse/fragment.rs
src/roadmap/ops/mod.rs
src/roadmap/ops/rewrite.rs
src/roadmap/render.rs
tests/roadmap_unit.rs
tests/behaviour_cli.rs
tests/steps/mod.rs
tests/steps/cli_steps.rs
tests/features/mapsplice.feature
```

The exact file split can change during implementation, but the core rule is
that parsing, mutation, rendering, and CLI concerns stay separate.

## Validation and observable checks

During implementation, use the repository Make targets and capture logs with
`tee` to `/tmp`. Run them sequentially, not in parallel.

```bash
make check-fmt 2>&1 | tee /tmp/check-fmt-mapsplice-initial-tool.out
make lint 2>&1 | tee /tmp/lint-mapsplice-initial-tool.out
make test 2>&1 | tee /tmp/test-mapsplice-initial-tool.out
make fmt 2>&1 | tee /tmp/fmt-mapsplice-initial-tool.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-initial-tool.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-initial-tool.out
```

The final implementation is acceptable only when all relevant commands pass and
the logs show no hidden truncation surprises. In addition, at least these
manual spot checks should succeed:

```bash
cargo run -- append fixtures/roadmaps/target.md fixtures/roadmaps/phase.md
cargo run -- insert fixtures/roadmaps/target.md 8 fixtures/roadmaps/phase.md
cargo run -- insert --after fixtures/roadmaps/target.md 8.2.3 fixtures/roadmaps/task.md
cargo run -- delete fixtures/roadmaps/target.md 8
cargo run -- replace fixtures/roadmaps/target.md 8 fixtures/roadmaps/replacement.md
```

Expected outcome: each command exits successfully when inputs match the roadmap
grammar, emits a correctly renumbered roadmap to standard output by default,
and rewrites the target only when `--in-place` is supplied.

## Progress

- [x] 2026-04-12 00:00Z: Reviewed repository state, local guidance, the
  `markdown` crate docs, and the Wireframe roadmap example.
- [x] 2026-04-12 00:00Z: Drafted the initial ExecPlan at
  `docs/execplans/initial-tool.md`.
- [x] 2026-04-12 19:55Z: Received approval to proceed with implementation.
- [x] 2026-04-12 19:55Z: Re-read the Rust, testing, and `ortho-config`
  reference guides before touching the code.
- [x] 2026-04-12 19:55Z: Confirmed the approved parsing strategy still holds:
  `markdown` provides `to_mdast()` but no documented Markdown writer API for
  the required round-trip behaviour.
- [x] 2026-04-12 20:10Z: Scaffolded the crate, added the typed roadmap model,
  parser, renderer, splice engine, capability-oriented filesystem helpers, and
  the first working CLI boundary.
- [x] 2026-04-12 20:15Z: Completed manual smoke checks for `append` and
  `insert` against temporary roadmap files, confirming renumbering and
  dependency rewrites on the happy path.
- [x] 2026-04-12 20:35Z: Added `rstest` unit coverage for anchor parsing,
  fragment classification, append/insert/delete/replace semantics, in-place
  rewrites, and level mismatch rejection.
- [x] 2026-04-12 20:35Z: Added `rstest-bdd` feature coverage that drives the
  compiled binary through stdout mode, in-place mode, replace, delete,
  insert-after, and mismatch failure scenarios.
- [x] 2026-04-12 20:40Z: Replaced the README stub with real user-facing
  command, grammar, and configuration documentation.
- [x] 2026-04-12 21:05Z: Split the parser into `parse::document` and
  `parse::fragment`, and split the mutation engine into `ops` plus
  `ops::rewrite`, to satisfy the repository's file-size and complexity rules
  without weakening validation.
- [x] 2026-04-12 21:15Z: Replayed the full gate stack on the final tree:
  `make fmt`, `make check-fmt`, `make lint`, `make test`, `make markdownlint`,
  and `make nixie` all passed.
- [x] Implement the roadmap parser, mutation engine, CLI, tests, and
  documentation.
- [x] Run code and documentation gates, then commit the approved work.
- [x] 2026-06-28 00:00Z: Addressed review-gate follow-up by adding
  `ortho-config` env/file merge tests, property tests, trybuild/API coverage,
  CLI help snapshots, structured tracing, process-local counters, and
  `docs/developers-guide.md`.

## Surprises & Discoveries

- The repository is effectively empty apart from shared documentation and a
  stub `main.rs`, so the implementation will create the project structure from
  scratch.
- The `markdown` crate documentation explicitly advertises parsing and mdast
  access, but not a full Markdown writer API suitable for guaranteed
  round-tripping.
- The sample Wireframe roadmap expresses dependencies as ordinary prose inside
  task bodies, for example `Requires 9.1.2.`, which means dependency rewrites
  must traverse parsed text content rather than rely on a dedicated field.
- The `ortho-config` guide now documents two useful patterns for this CLI:
  per-subcommand merges via `load_and_merge_subcommand_for`, and global-plus-
  selected-subcommand merging via `load_globals_and_merge_selected_subcommand`.
- The `grepai` project index returned no useful hits for this tiny crate, so
  local exploration is falling back to `leta` plus targeted file reads.
- `ortho-config` subcommand merging returns an `Arc<OrthoError>` and is a good
  fit for subcommand-scoped optional settings, but the global `--in-place` flag
  is cleaner as a plain clap-global process option than as a forced top-level
  merge.
- Fragment numbering can collide with target numbering during dependency
  rewrites. A naïve `old_id -> new_id` map is therefore ambiguous when both
  documents define the same anchor text.
- `rstest-bdd` scenario bodies accept unit `Result` returns, but fallible
  fixtures need an explicit helper to unwrap `Result<Fixture, Error>` into a
  borrowed fixture value without propagating `&mut Error` references.
- The repository's Whitaker policy is materially stricter than plain Clippy:
  it rejects oversized modules, branch-heavy conditions, clustered control-flow
  bumps, and assertion-heavy tests that return `Result`, so the final shape had
  to be refactored around those maintainability rules instead of merely
  compiling.
- Review gating treats configuration, compile-time API compatibility, property
  coverage, and observability as part of the initial tool contract rather than
  later hardening. The test matrix now includes env/file config defaults,
  generated anchors and dependency rewrites, trybuild API checks, and focused
  CLI help snapshots.

## Decision Log

- Decision: treat the supported roadmap syntax as a constrained document
  grammar instead of attempting arbitrary Markdown surgery. Rationale: the user
  asked for a simple CLI tool, and the `markdown` crate does not document a
  full round-trip writer in the same way it documents its parser and mdast.

- Decision: model splice operations against a roadmap-specific intermediate
  representation rather than mutating raw strings. Rationale: structural edits,
  renumbering, and dependency rewrites are easier to validate and unit test
  when the domain model is explicit.

- Decision: interpret `append` as phase-level append only in the first version.
  Rationale: the command form has no anchor, so step-level or task-level append
  semantics would be ambiguous.

- Decision: keep the plan approval-gated.
  Rationale: the repository instructions and the execplans skill both require a
  draft-first workflow for non-trivial work.

- Decision: introduce `src/lib.rs` alongside a thin `src/main.rs`.
  Rationale: the library crate gives the roadmap engine a stable home for unit
  tests and future doctests, while the binary remains a narrow CLI/reporting
  boundary.

- Decision: use an `OrthoConfig`-derived global CLI struct that holds
  per-command optional settings, while keeping the process-wide `--in-place`
  flag as a clap-global option at the binary edge. Rationale: this preserves
  the user's requested CLI surface, keeps required positional values CLI-only,
  and avoids forcing top-level config merging into the wrong `ortho-config`
  abstraction.

- Decision: resolve dependency rewrites by looking up source-local renumbered
  anchors first, then falling back to a unique cross-source mapping when the
  anchor text is only defined once across target and fragment. Rationale: this
  handles internal dependencies in inserted fragments and downstream rewrites
  in the original target without collapsing colliding fragment and target
  identifiers into one ambiguous map.

- Decision: keep the behavioural harness on compile-time binary discovery via
  `env!("CARGO_BIN_EXE_mapsplice")`. Rationale: `nextest` does not provide the
  same runtime environment lookup the first fixture draft expected, while Cargo
  reliably embeds the binary path for integration tests at compile time.

- Decision: split both parsing and mutation into nested submodules instead of
  keeping one file per concern. Rationale: Whitaker's module-size and
  complexity lints are part of the repository contract, so the maintainable
  design is `parse::{document,fragment}` plus `ops::rewrite`, not a pair of
  near-400-line files.

- Decision: keep parser nodes in the domain behind a dedicated `MarkdownNodes`
  value object for the initial implementation. Rationale: a full Markdown
  abstraction would be larger than the current scope, while raw `Vec<Node>`
  fields make parse/render adapter concerns too visible in the roadmap model.

- Decision: add process-local observability counters rather than a metrics
  backend. Rationale: the CLI needs stable failure and rewrite counts for tests
  and embeddings, but adding a registry or exporter would be premature for the
  first release.

## Outcomes & Retrospective

The implementation is complete and the repository gates are green. The final
shape is a small library-backed CLI with a capability-oriented filesystem
boundary, typed roadmap identifiers, mdast-driven parsing into a roadmap
intermediate representation, deterministic rendering of the supported roadmap
grammar, source-aware renumbering plus dependency rewrites, `rstest` unit
coverage, and `rstest-bdd` binary-level behavioural coverage.

The most important retrospective lesson is that the repository's
maintainability rules changed the architecture in useful ways. The final
submodule split kept the parser and rewrite logic readable, and the stricter
lint rules pushed the tests towards clearer fixture naming and explicit
test-style assertions instead of opaque fallible test bodies.

The review-hardening pass expanded the quality envelope beyond example-based
tests. Configuration defaults now have direct env/file coverage, generated
properties exercise anchor and rewrite invariants, compile-time tests pin the
public API, and structured spans plus counters make CLI failures inspectable
without changing stdout semantics.
