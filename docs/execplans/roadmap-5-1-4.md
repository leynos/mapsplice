# Encapsulate roadmap mutation invariants

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

The df12-build roadmap workflow has approved this plan for implementation.

## Purpose / big picture

Roadmap task 5.1.4, "Encapsulate roadmap mutation invariants", is a
behaviour-preserving refactor after the parser and helper consolidation work.
The current code has two mutable seams that make future roadmap grammar changes
riskier than necessary:

- `RenumberPlan` stores a nested `BTreeMap` in a public field, even though its
  callers should only record and resolve old-to-new anchor mappings.
- `TaskChildren` and sub-task splice code keep parallel ordered child state by
  pushing or splicing `sub_tasks` and `children` separately.

After this change, maintainers should mutate those structures through small
methods that preserve the nested-map and ordered-child invariants. Users should
observe no CLI, rendering, diagnostic, metrics, or public command behaviour
change. The observable proof is that existing roadmap operations still
renumber task bodies, sub-tasks, and dependency references exactly as before,
while compile-time coverage prevents external callers from directly mutating
the now-hidden task child vectors.

## Constraints

- Work only inside
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-4`.
- Do not edit the root/control worktree.
- Treat `origin/main` as canonical and `docs/roadmap.md` as the roadmap source
  of truth.
- Implement only roadmap task 5.1.4 from `docs/roadmap.md`:
  "Hide direct mutation of `RenumberPlan` and `TaskChildren` internals behind
  methods that preserve nested-map and ordered-child invariants."
- Preserve `docs/mapsplice-design.md` section 2, "Non-negotiable constraints":
  parsing remains mdast-based through the locked `markdown` crate and edits run
  through the roadmap model rather than raw-string surgery.
- Preserve `docs/mapsplice-design.md` section 4, "The roadmap grammar
  (normative reference)": phases, steps, tasks, and addendum sub-tasks keep the
  same accepted grammar.
- Preserve `docs/mapsplice-design.md` section 5, "Fidelity guarantees",
  especially F1 content preservation, F2 minimal diff, F3 round-trip
  stability, F4 gate-clean output, and F5 fail-closed behaviour.
- Preserve `docs/mapsplice-design.md` section 6, "Functional and contract
  guarantees", especially C1 operations, C2 contiguous renumbering, C3
  dependency-reference rewriting, C4 first-class addendum sub-tasks, C5
  idempotence, and C6 output modes.
- Preserve `docs/mapsplice-design.md` section 8, "Fixture and test
  requirements": unit tests cover the model, renumbering, and reference
  resolver in isolation; behavioural tests cover CLI flows and output modes;
  golden comparison covers render fidelity.
- Preserve `docs/developers-guide.md` section 2, "Architecture boundaries":
  `src/lib.rs` owns the application workflow and `src/roadmap` owns domain
  parsing, mutation, renumbering, and rendering.
- Preserve `docs/developers-guide.md` section 3, "Public library APIs": keep
  `run_from_args`, `run_request`, `parse_roadmap`, `parse_fragment`,
  `parse_anchor`, and `metrics_snapshot` available with the same signatures.
- Follow `docs/developers-guide.md` section 6, "Verification layers": use
  `rstest` unit tests for finite operation and model matrices, `rstest-bdd`
  for user workflows already covered by feature scenarios, `proptest` for
  generated dependency-renumber behaviour, `trybuild` for public compile-time
  API compatibility, and `insta` only for stable large artefacts.
- Follow `docs/developers-guide.md` section 7, "Local tooling": run Rust gates
  before committing code and use path-scoped Markdown maintenance for Markdown
  changes.
- Follow `docs/users-guide.md`, "The roadmap shape `mapsplice` expects",
  "Command overview", "Output modes", and "Validation rules and failure
  cases". This task must not change accepted command syntax or user-facing
  output modes.
- Follow `docs/documentation-style-guide.md`: prose uses en-GB Oxford spelling,
  sentence-case headings, fenced code languages, and 80-column wrapping.
- Follow `docs/execplans/initial-tool.md` decisions: model splice operations
  against a roadmap-specific intermediate representation, keep parser nodes
  behind `MarkdownNodes`, resolve dependency rewrites through source-aware
  renumbering, and keep parsing and mutation split into nested modules.
- Do not add a new external dependency.
- Do not change accepted roadmap grammar, rendered Markdown, CLI arguments,
  public error variants, public metrics fields, or diagnostic text.
- Keep every Rust source file under 400 lines.
- Use Red-Green-Refactor. Because this is a behaviour-preserving refactor, the
  red stage may be either a focused failing test before the production change
  or a deliberate temporary mutation proving the new or existing test fails for
  the intended reason. Revert temporary mutations before committing.
- Format only changed Markdown files. Do not run repository-global Markdown
  formatters such as `make fmt` or `mdformat-all`.
- Run tests, lint, and formatting gates sequentially with `tee` logs under
  `/tmp`. Do not run test, lint, or format gates in parallel.
- Use the shared Cargo cache. Do not create an isolated Cargo cache.
- Do not treat Memtrace, Leta, Firecrawl, GrepAI, or sem unavailability as a
  product blocker. Record the exact failed command or MCP call and continue
  with bounded local documentation, source, and test evidence.

## Tolerances

- Stop and escalate if implementation requires a new crate, a public command
  behaviour change, a public error variant change, accepted grammar changes, or
  a rendered-output change outside tests that deliberately pin unchanged
  behaviour.
- Stop and escalate if `run_from_args`, `run_request`, `parse_roadmap`,
  `parse_fragment`, `parse_anchor`, or `metrics_snapshot` signatures would need
  to change.
- The planned implementation may touch only these files unless a focused test
  proves a necessary adjacent change:
  `docs/execplans/roadmap-5-1-4.md`, `docs/roadmap.md`,
  `src/roadmap/model.rs`, `src/roadmap/parse/task_children.rs`,
  `src/roadmap/parse/mod.rs`, `src/roadmap/ops/sub_task.rs`,
  `src/roadmap/ops/rewrite.rs`, `src/roadmap/render.rs`,
  `src/roadmap/ops/dependency_text.rs`, `src/roadmap/render_tests.rs`,
  `tests/compile_time.rs`, `tests/ui/public_api.rs`,
  `tests/ui/model_invariants.rs`, `tests/ui/model_invariants.stderr`,
  `tests/roadmap_properties.rs`, `tests/roadmap_sub_tasks.rs`,
  `tests/roadmap_sub_task_invariants.rs`, and focused test-support modules
  needed by those tests.
- Stop and escalate if exact rendered Markdown changes in any existing golden,
  BDD, or CLI behaviour test.
- Stop and escalate if making `TaskEntry` fields private would require
  changing the documented public library API signatures named above. It is
  acceptable to add read-only accessor methods on `TaskEntry` and adjust tests
  away from direct field access, because `TaskEntry` is not named in
  `docs/developers-guide.md` section 3 as a public entry point.
- Stop and escalate if `RenumberPlan` needs to become part of the public API to
  satisfy this task. It is an internal roadmap-domain type and should stay
  internal.
- Stop and escalate if this refactor changes dependency rewrite counts,
  dangling dependency diagnostics, `AnchorNotFound` anchors, or
  `InvalidRoadmap` messages.
- Stop and escalate if the net production-code diff exceeds 180 lines or any
  Rust source file exceeds 400 lines after the change.
- Stop and escalate if `tests/roadmap_ops.rs` would need new lines. It is
  already 399 lines; add focused coverage elsewhere.
- Stop and escalate if the same focused test still fails after three
  implementation attempts.

## Risks

- Risk: hiding `TaskEntry` child vectors could force broad test churn because
  integration tests inspect parsed sub-tasks directly. Severity: medium.
  Likelihood: medium. Mitigation: add small read-only accessors and update
  tests to inspect through them rather than through public fields.
- Risk: moving sub-task splice logic into `TaskEntry` methods could change the
  error selected for a corrupt model with a missing child reference. Severity:
  medium. Likelihood: medium. Mitigation: pin the existing
  `InvalidRoadmap` messages before changing the splice code.
- Risk: `RenumberPlan` encapsulation could alter ambiguous cross-source
  dependency resolution. Severity: medium. Likelihood: low. Mitigation: add
  model-level unit tests for source-local lookup, unique cross-source lookup,
  and ambiguous cross-source lookup before hiding the map.
- Risk: compile-fail tests can create noisy `.stderr` churn. Severity: low.
  Likelihood: medium. Mitigation: use the locked `trybuild` 1.0.117 workflow,
  review the generated `wip/*.stderr`, move only the intended stderr file, and
  run `make all` to verify stability.
- Risk: advisory tools may be unavailable in the implementation session.
  Severity: low. Likelihood: high. Mitigation: retry Memtrace and Leta before
  editing, record exact failures if they persist, and proceed with bounded
  source inspection and tests.

## Progress

- [x] (2026-07-03T10:34:32Z) Confirmed the current branch is
  `roadmap-5-1-4`, so this plan is
  `docs/execplans/roadmap-5-1-4.md`.
- [x] (2026-07-03T10:34:32Z) Loaded planning, navigation, Rust design, testing,
  verification, semantic-history, Firecrawl, and commit-message guidance:
  `execplans`, `leta`, `rust-router`, `rust-types-and-apis`,
  `rust-unit-testing`, `rust-verification`, `proptest`, `firecrawl-mcp`,
  `sem`, and `commit-message`.
- [x] (2026-07-03T10:34:32Z) Memtrace discovery failed twice with
  `mcp__memtrace.list_indexed_repositories -> user cancelled MCP tool call`.
  Per task instructions, this is recorded as a tooling failure and is not a
  product blocker.
- [x] (2026-07-03T10:34:32Z) Leta setup failed with
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-4`
  returning `Error: IO error: Read-only file system (os error 30)`, and
  `leta files` returning `Error: Failed to start daemon`. Per task
  instructions, this is recorded as a tooling failure and is not a product
  blocker.
- [x] (2026-07-03T10:34:32Z) Firecrawl official-doc verification failed with
  `mcp__firecrawl.firecrawl_scrape` returning `user cancelled MCP tool call`
  for the `rstest`, `proptest`, and `trybuild` docs.rs URLs. Locked local crate
  source is cited below instead, and load-bearing behaviour is pinned by tests.
- [x] (2026-07-03T10:34:32Z) Used `sem diff --from origin/main --to HEAD
  --format json` to confirm this planning branch had no code changes before
  drafting the ExecPlan.
- [x] (2026-07-03T10:34:32Z) Drafted the first-round ExecPlan.
- [x] (2026-07-03T11:00:00Z) Started the approved implementation session on
  branch `roadmap-5-1-4`.
- [x] (2026-07-03T11:00:00Z) Retried Memtrace repository discovery twice.
  Both `mcp__memtrace.list_indexed_repositories` calls returned
  `user cancelled MCP tool call`, so implementation continues with bounded
  branch-local evidence as allowed by this plan.
- [x] (2026-07-03T11:00:00Z) Retried Leta. `leta workspace add` and
  `leta files src/roadmap` succeeded, and `leta show` located `TaskEntry`,
  `RenumberPlan`, `TaskChildren`, `split_task_children`, and
  `parse_sub_task_list`; `leta refs RenumberPlan` failed with
  `Error: Failed to start daemon`.
- [x] (2026-07-03T11:15:18Z) Work Item 1: Pin current mutation-invariant
  behaviour.
  Added `RenumberPlan` model tests for source-local, unique cross-source,
  ambiguous cross-source, and missing source-local lookup; added
  `tests/roadmap_sub_task_invariants.rs` to pin insert, delete, and replace
  body-order and dependency-rewrite behaviour without pushing
  `tests/roadmap_sub_tasks.rs` over 400 lines.
- [x] (2026-07-03T11:15:18Z) Work Item 1 red proof: temporarily changed the
  expected inserted sub-task text to `Unexpected sub-task`; `cargo test --test
  roadmap_sub_tasks insert_sub_task_preserves_interleaved_body_order
  -- --nocapture` failed with the missing expected text, then the temporary
  mutation was reverted.
- [x] (2026-07-03T11:15:18Z) Work Item 1 focused validation passed:
  `cargo test --test roadmap_sub_tasks -- --nocapture`,
  `cargo test --test roadmap_sub_task_invariants -- --nocapture`,
  `cargo test --test roadmap_properties -- --nocapture`, and
  `cargo test roadmap::model`.
- [x] (2026-07-03T11:15:18Z) Work Item 1 deterministic gates passed via
  scrutineer after formatting and lint fixes: `make all`,
  `make markdownlint`, and `make nixie`. Logs:
  `/tmp/make-all-mapsplice-roadmap-5-1-4-item-1.out`,
  `/tmp/markdownlint-mapsplice-roadmap-5-1-4-item-1.out`, and
  `/tmp/nixie-mapsplice-roadmap-5-1-4-item-1.out`.
- [x] (2026-07-03T11:15:18Z) Work Item 1 CodeRabbit review was attempted via
  scrutineer and deferred before producing findings because the sandbox has no
  default network route. Log:
  `/tmp/coderabbit-mapsplice-roadmap-5-1-4-item-1.out`.
- [x] (2026-07-03T11:21:45Z) Work Item 2: Hide `RenumberPlan` map mutation.
  Made `RenumberPlan::by_source` private and renamed the crate-private mutation
  seam to `record_mapping`, then updated renumbering, dependency-text tests,
  and model tests to use that method.
- [x] (2026-07-03T11:21:45Z) Work Item 2 red proof: after renaming the method
  but before updating callers, `cargo test
  roadmap::ops::dependency_text::tests -- --nocapture` failed with
  `no method named insert found for struct RenumberPlan` in the stale call
  sites.
- [x] (2026-07-03T11:21:45Z) Work Item 2 focused validation passed:
  `cargo test roadmap::model`, `cargo test
  roadmap::ops::dependency_text::tests -- --nocapture`, and
  `cargo test --test roadmap_properties -- --nocapture`.
- [x] (2026-07-03T11:21:45Z) Work Item 2 deterministic gate passed via
  scrutineer: `make all`. Log:
  `/tmp/make-all-mapsplice-roadmap-5-1-4-item-2.out`.
- [x] (2026-07-03T11:21:45Z) Work Item 2 CodeRabbit review was attempted via
  scrutineer and deferred before producing findings because the sandbox has no
  default network route. Log:
  `/tmp/coderabbit-mapsplice-roadmap-5-1-4-item-2.out`.
- [x] (2026-07-03T11:29:13Z) Work Item 3: Encapsulate parser task-child
  accumulation. Made `TaskChildren` fields private and routed parser mutation
  through `push_body_node`, `push_sub_task`, `next_sub_task_ordinal`, and
  `finish`.
- [x] (2026-07-03T11:29:13Z) Work Item 3 red proof: after making the
  accumulator fields private but before updating parser call sites, `cargo
  test --test roadmap_sub_tasks
  parse_roadmap_keeps_nested_numbered_sub_tasks_structural -- --nocapture`
  failed with private-field errors for `body`, `sub_tasks`, and `ordered`.
- [x] (2026-07-03T11:29:13Z) Work Item 3 focused validation passed:
  `cargo test --test roadmap_sub_tasks -- --nocapture`,
  `cargo test --test roadmap_parse -- --nocapture`, and
  `cargo test --test roadmap_golden -- --nocapture`.
- [x] (2026-07-03T11:29:13Z) Work Item 3 deterministic gate passed via
  scrutineer after a Clippy shadowing fix: `make all`. Log:
  `/tmp/make-all-mapsplice-roadmap-5-1-4-item-3.out`.
- [x] (2026-07-03T11:29:13Z) Work Item 3 CodeRabbit review was attempted via
  scrutineer and deferred before producing findings because the sandbox has no
  default network route. Log:
  `/tmp/coderabbit-mapsplice-roadmap-5-1-4-item-3.out`.
- [x] (2026-07-03T12:25:00Z) Work Item 4: Route sub-task splices through
  `TaskEntry` methods. Made `TaskEntry::sub_tasks` and
  `TaskEntry::children` private, added read-only accessors plus
  invariant-preserving splice methods, routed render, rewrite, parser, and
  sub-task operations through those seams, and added compile-time coverage in
  `tests/ui/model_invariants.rs`.
- [x] (2026-07-03T12:25:00Z) Work Item 4 red proof: before field privacy, the
  new `trybuild` compile-fail fixture reported
  `Expected test case to fail to compile, but it succeeded`. After field
  privacy, the fixture produced and then matched
  `tests/ui/model_invariants.stderr` with `E0616`.
- [x] (2026-07-03T12:25:00Z) Work Item 4 focused validation passed:
  `cargo test --test compile_time -- --nocapture`,
  `cargo test --test roadmap_sub_tasks -- --nocapture`,
  `cargo test --test roadmap_properties -- --nocapture`, and
  `cargo test --test roadmap_render -- --nocapture`.
- [x] (2026-07-03T12:25:00Z) Work Item 4 deterministic gates passed via
  scrutineer after format and Clippy fixes: `make all`, `make markdownlint`,
  and `make nixie`. Logs:
  `/tmp/make-all-mapsplice-roadmap-5-1-4.out`,
  `/tmp/markdownlint-mapsplice-roadmap-5-1-4.out`, and
  `/tmp/nixie-mapsplice-roadmap-5-1-4.out`.
- [x] (2026-07-03T12:25:00Z) Work Item 4 CodeRabbit review was attempted via
  scrutineer and deferred before producing findings because the sandbox has no
  default network route. Log:
  `/tmp/coderabbit-mapsplice-roadmap-5-1-4-item-4.out`.
- [x] (2026-07-03T12:40:00Z) Work Item 5: Mark roadmap task 5.1.4 complete.
  Updated `docs/roadmap.md`, set this ExecPlan to `COMPLETE`, and recorded
  final outcomes after Work Items 1-4 were committed and gate-clean.
- [x] (2026-07-03T12:40:00Z) Work Item 5 deterministic gates passed via
  scrutineer: `make all`, `make markdownlint`, and `make nixie`. Logs:
  `/tmp/make-all-mapsplice-roadmap-5-1-4-final.out`,
  `/tmp/markdownlint-mapsplice-roadmap-5-1-4-final.out`, and
  `/tmp/nixie-mapsplice-roadmap-5-1-4-final.out`.
- [x] (2026-07-03T12:40:00Z) Work Item 5 CodeRabbit review was attempted via
  scrutineer and deferred before producing findings because the sandbox has no
  default network route. Log:
  `/tmp/coderabbit-mapsplice-roadmap-5-1-4-final.out`.

## Surprises & discoveries

- Observation: Memtrace was present in deferred tool metadata, but the host
  session cancelled `list_indexed_repositories` twice.
  Evidence: both MCP calls returned `user cancelled MCP tool call`.
  Impact: this plan uses bounded branch-local documentation and source evidence
  instead of canonical main-branch Memtrace graph evidence.
- Observation: Leta could not initialise or list files in this sandbox.
  Evidence: `leta workspace add` failed with `Read-only file system`; `leta
  files` failed with `Failed to start daemon`.
  Impact: branch-local verification used exact text search and bounded file
  inspection for this planning round.
- Observation: Firecrawl docs.rs scraping was unavailable in this session.
  Evidence: scrapes for `rstest` 0.26.1, `proptest` 1.11.0, and `trybuild`
  1.0.117 all returned `user cancelled MCP tool call`.
  Impact: the plan cites locked local crate source and requires tests for
  load-bearing behaviour rather than relying on unchecked external docs.
- Observation: `Cargo.toml` specifies `trybuild = "1.0.114"`, while
  `Cargo.lock` currently pins `trybuild` 1.0.117.
  Evidence: `Cargo.lock` lines for `trybuild` report version 1.0.117.
  Impact: compile-fail test workflow in this plan is pinned to the locked
  1.0.117 source.
- Observation: Memtrace was still unavailable in the implementation session.
  Evidence: two `mcp__memtrace.list_indexed_repositories` calls returned
  `user cancelled MCP tool call`.
  Impact: implementation uses branch-local Leta where available plus bounded
  source and test inspection.
- Observation: Leta availability improved after the planning round but was not
  complete.
  Evidence: `leta workspace add` and `leta files src/roadmap` succeeded;
  `leta refs RenumberPlan` returned `Error: Failed to start daemon`.
  Impact: symbol bodies came from Leta, while reference-style checks use
  focused source inspection.
- Observation: Work Item 1 needed a focused new integration test file to keep
  existing files within the repository line-count rule.
  Evidence: placing the interleaved-body matrix in `tests/roadmap_sub_tasks.rs`
  would have made that file 417 lines; moving the matrix to
  `tests/roadmap_sub_task_invariants.rs` keeps the original file at 272 lines
  and the new file at 160 lines.
  Impact: the plan's allowed file list now includes the focused integration
  test file.
- Observation: CodeRabbit review could not run in this sandbox.
  Evidence: `/tmp/coderabbit-mapsplice-roadmap-5-1-4-item-1.out` contains
  `{"type":"status","phase":"deferred","status":"deferred coderabbit review:
  no default network route visible in this sandbox"}`.
  Impact: Work Item 1 carries a deferred-review open issue, with deterministic
  gates green.
- Observation: CodeRabbit review remained unavailable for Work Item 2.
  Evidence: `/tmp/coderabbit-mapsplice-roadmap-5-1-4-item-2.out` contains
  `{"type":"status","phase":"deferred","status":"deferred coderabbit review:
  no default network route visible in this sandbox"}`.
  Impact: Work Item 2 carries the same deferred-review open issue, with
  deterministic gates green.
- Observation: CodeRabbit review remained unavailable for Work Item 3.
  Evidence: `/tmp/coderabbit-mapsplice-roadmap-5-1-4-item-3.out` contains
  `{"type":"status","phase":"deferred","status":"deferred coderabbit review:
  no default network route visible in this sandbox"}`.
  Impact: Work Item 3 carries the same deferred-review open issue, with
  deterministic gates green.
- Observation: `TaskChildren::next_sub_task_ordinal` must be called once per
  sub-task list, before the parser pushes that list's sub-tasks.
  Evidence: recomputing from the growing accumulator inside the loop made the
  second sub-task in a list expect ordinal 3 and failed
  `parse_roadmap_keeps_nested_numbered_sub_tasks_structural` with
  `sub-task '1.1.1.2' is not in document order`.
  Impact: `parse_sub_task_list` now snapshots the starting ordinal before the
  loop and applies checked offsets for list-local siblings.
- Observation: `trybuild` can leave a generated coordination lock that makes
  later compile-time test runs appear to hang before case output.
  Evidence: after a timed-out compile-time run, no Cargo or rustc process held
  the lock, manual Cargo against `target/tests/trybuild/mapsplice/Cargo.toml`
  produced the expected `E0616`, and removing
  `target/tests/trybuild/mapsplice/.lock` let `trybuild` write
  `wip/model_invariants.stderr`.
  Impact: the checked-in compile-fail fixture is stable, and stale generated
  trybuild locks are treated as local test artefacts rather than product
  failures.
- Observation: Work Item 4 needed model-owned argument structs to satisfy the
  repository's strict Clippy policy.
  Evidence: `make all` failed with `missing_const_for_fn` and
  `too_many_arguments` on the first `TaskEntry` constructor and splice method
  shapes.
  Impact: `TaskEntryParts` and `SubTaskSplice` now carry parser construction
  state and paired splice indices without exposing the underlying vectors.
- Observation: CodeRabbit review remained unavailable for Work Item 4.
  Evidence: `/tmp/coderabbit-mapsplice-roadmap-5-1-4-item-4.out` contains
  `{"type":"status","phase":"deferred","status":"deferred coderabbit review:
  no default network route visible in this sandbox"}`.
  Impact: Work Item 4 carries the same deferred-review open issue, with
  deterministic gates green.
- Observation: CodeRabbit review remained unavailable for Work Item 5.
  Evidence: `/tmp/coderabbit-mapsplice-roadmap-5-1-4-final.out` contains
  `{"type":"status","phase":"deferred","status":"deferred coderabbit review:
  no default network route visible in this sandbox"}`.
  Impact: Work Item 5 carries the same deferred-review open issue, with final
  deterministic gates green.

## Decision log

- Decision: keep the task behaviour-preserving and avoid grammar, diagnostic,
  CLI, metrics, and rendered-output changes.
  Rationale: roadmap task 5.1.4 is consolidation work in roadmap section 5,
  not a user-facing feature.
  Date/Author: 2026-07-03T10:34:32Z, Codex.
- Decision: use compile-time proof for hidden task child mutation rather than
  relying only on source review.
  Rationale: the success criterion says callers cannot bypass
  invariant-preserving methods. `trybuild::TestCases::compile_fail` is already
  a locked dev dependency, and compile-fail coverage makes field privacy
  observable.
  Date/Author: 2026-07-03T10:34:32Z, Codex.
- Decision: encapsulate `TaskChildren` first at the parser accumulator seam,
  then route sub-task splice mutation through `TaskEntry` methods.
  Rationale: parser construction and splice operations both maintain the same
  ordered-child invariant, but the changes can be made as separate, gate-clean
  commits.
  Date/Author: 2026-07-03T10:34:32Z, Codex.
- Decision: cite locked local crate source where Firecrawl official-doc checks
  were cancelled, and require focused tests for load-bearing behaviour.
  Rationale: the task requires verified mechanisms. Local crate source plus
  red/green tests is stronger than unchecked memory of external API behaviour.
  Date/Author: 2026-07-03T10:34:32Z, Codex.
- Decision: place the Work Item 1 interleaved sub-task mutation matrix in
  `tests/roadmap_sub_task_invariants.rs`.
  Rationale: the existing `tests/roadmap_sub_tasks.rs` file would exceed the
  400-line code-file limit with the new matrix. A focused integration test file
  preserves the same behavioural coverage without creating a long file.
  Date/Author: 2026-07-03T11:15:18Z, Codex.
- Decision: pin the current replace-with-multiple-sub-tasks dependency mapping
  as-is.
  Rationale: replacing `1.1.1.2` with two sub-tasks currently rewrites
  references to `1.1.1.3`, the final replacement item. Roadmap task 5.1.4 is a
  behaviour-preserving invariant refactor, so Work Item 1 records that current
  behaviour rather than changing it.
  Date/Author: 2026-07-03T11:15:18Z, Codex.
- Decision: rename `RenumberPlan::insert` to
  `RenumberPlan::record_mapping`.
  Rationale: `record_mapping` describes the domain operation and avoids
  suggesting callers can manipulate the nested map as a generic collection.
  Date/Author: 2026-07-03T11:21:45Z, Codex.
- Decision: keep list-local sub-task ordinal offsets in `parse_sub_task_list`
  rather than in `TaskChildren::push_sub_task`.
  Rationale: the accumulator owns the current starting ordinal, but each mdast
  list supplies its own sibling offset. Keeping the offset calculation beside
  the loop preserves the existing document-order diagnostic and avoids storing
  temporary list state in `TaskChildren`.
  Date/Author: 2026-07-03T11:29:13Z, Codex.
- Decision: introduce `TaskEntryParts` and `SubTaskSplice` instead of keeping
  the initial index-heavy method signatures.
  Rationale: these small domain structs avoid Clippy argument-count failures,
  make parser construction and paired splice locations explicit, and still
  preserve the invariant that callers cannot mutate `sub_tasks` and
  `children` directly.
  Date/Author: 2026-07-03T12:25:00Z, Codex.
- Decision: validate parser-created task child identity consistency inside
  `TaskEntry::from_parts`.
  Rationale: moving parser construction behind the model seam is a good place
  to reject mismatched structural sub-task identities before rendering or
  mutation operations observe an inconsistent task.
  Date/Author: 2026-07-03T12:25:00Z, Codex.

## Context and orientation

The roadmap domain model lives in `src/roadmap/model.rs`. The relevant current
state is:

- `TaskEntry` stores `sub_tasks: Vec<SubTaskEntry>` and
  `children: Vec<TaskChild>` as public fields. `children` records the original
  ordered mixture of body blocks and sub-task identities. Rendering walks
  `children`, and each `TaskChild::SubTask` must resolve to exactly one
  `SubTaskEntry` by identity.
- `RenumberPlan` stores a public `by_source:
  BTreeMap<SourceId, BTreeMap<RoadmapAnchor, RoadmapAnchor>>`. Existing methods
  already resolve source-local mappings and unique cross-source mappings.
- `TaskChildren` in `src/roadmap/parse/task_children.rs` is the parser-side
  accumulator for `body`, `sub_tasks`, and `ordered`. `src/roadmap/parse/mod.rs`
  currently pushes into those fields directly.
- `src/roadmap/ops/sub_task.rs` currently computes a sub-task index and child
  index, then separately splices or removes from `task.sub_tasks` and
  `task.children`.
- `src/roadmap/ops/rewrite.rs` builds the renumber plan and uses it to rewrite
  dependencies in task and sub-task text.

The source evidence gathered in this planning round:

- `src/roadmap/model.rs` lines 68-81 show public `TaskEntry` `sub_tasks` and
  `children` fields.
- `src/roadmap/model.rs` lines 133-137 show public `RenumberPlan::by_source`.
- `src/roadmap/model.rs` lines 229-255 show the existing `resolve`,
  `resolve_unique`, and crate-private `insert` methods.
- `src/roadmap/parse/task_children.rs` lines 5-25 show public-super
  `TaskChildren` fields and a `flush_body` method that pushes a body child.
- `src/roadmap/parse/mod.rs` lines 176-204 show direct parser access to
  `TaskChildren::body`, `sub_tasks`, and `ordered`.
- `src/roadmap/parse/mod.rs` lines 207-235 show direct parser pushes to
  `ordered` and `sub_tasks` for parsed sub-tasks.
- `src/roadmap/ops/sub_task.rs` lines 13-53 show insert, delete, and replace
  mutating `sub_tasks` and `children` separately.
- `src/roadmap/ops/rewrite.rs` lines 37-83 show `RenumberPlan::insert` in the
  renumber traversal.
- `tests/roadmap_sub_tasks.rs` already covers parsed sub-tasks, task-body
  preservation, sub-task renumbering, and dependency rewrites through the CLI.
- `tests/roadmap_properties.rs` already covers generated dependency reference
  preservation and rewriting.
- Local locked `rstest` 0.26.1 source
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rstest-0.26.1/src/lib.rs`
  lines 573-611 document `#[rstest]`, fixtures, and `#[case]` support; lines
  704-735 document independent parameterized cases.
- Local locked `proptest` 1.11.0 source
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/prelude.rs`
  lines 23-30 re-export `Strategy`, `ProptestConfig`, `TestCaseError`,
  `prop_assert`, `prop_assert_eq`, `prop_assume`, `prop_compose`, and
  `proptest`.
- Local locked `serial_test` 3.5.0 source
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serial_test-3.5.0/src/lib.rs`
  lines 28-31 document serial execution guarantees and lines 151-152
  re-export `serial`.
- Local locked `trybuild` 1.0.117 source
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/trybuild-1.0.117/src/lib.rs`
  lines 112-129 document the `wip` and `TRYBUILD=overwrite` stderr workflow,
  lines 133-141 describe compile-fail test use, and lines 321-328 expose
  `TestCases::pass` and `TestCases::compile_fail`.
- Local locked `markdown` 1.0.0 source
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/lib.rs`
  lines 49, 72, and 150-160 expose `mdast`, `ParseOptions`, and `to_mdast`.

## Plan of work

### Work Item 1: Pin current mutation-invariant behaviour

This commit adds focused tests before changing production code. It implements
the proof obligations from `docs/mapsplice-design.md` sections 5, 6, and 8;
`docs/developers-guide.md` sections 2 and 6; and
`docs/roadmap.md` task 5.1.4.

Skills to load for this work item: `rust-router`, `rust-unit-testing`,
`rust-verification`, `proptest`, `leta`, `sem`, and `execplans`.

Read these documents before editing: `AGENTS.md`, `docs/mapsplice-design.md`
sections 5-8, `docs/developers-guide.md` sections 2, 3, 6, and 7,
`docs/users-guide.md` sections "The roadmap shape `mapsplice` expects" and
"Output modes", and `docs/roadmap.md` section 5.1.4.

Add model-level unit tests in `src/roadmap/model.rs` for `RenumberPlan`:

- source-local lookup resolves the mapping for the requested source;
- unique cross-source lookup returns the mapped anchor when exactly one source
  contains the old anchor;
- ambiguous cross-source lookup returns `None` when target and fragment both
  map the same old anchor;
- missing source-local lookup returns `None`.

Add or extend focused tests in `src/roadmap/ops/sub_task.rs` or
`tests/roadmap_sub_tasks.rs` to pin the ordered-child invariant:

- a task body before, between, and after sub-tasks keeps body blocks in the same
  rendered order after insert-before, insert-after, delete, and replace;
- a corrupt task whose child order references a missing sub-task still fails
  with the current `InvalidRoadmap` message;
- dependency references in task bodies and sub-task bodies still rewrite after
  sub-task mutations.

Add one property test to `tests/roadmap_properties.rs` only if the concrete
matrix above does not cover enough body/sub-task interleavings. Construct valid
roadmap snippets directly; do not filter invalid generated input with
`prop_assume`.

Red proof: before production changes, temporarily break one assertion or one
small helper and run the focused test command. Confirm the test fails for the
intended invariant, then revert the temporary mutation. The committed state of
this work item is test-only and must pass.

Focused validation commands:

```bash
cargo test --test roadmap_sub_tasks -- --nocapture 2>&1 | tee /tmp/test-sub-tasks-mapsplice-roadmap-5-1-4-item-1.out
cargo test --test roadmap_properties -- --nocapture 2>&1 | tee /tmp/test-properties-mapsplice-roadmap-5-1-4-item-1.out
cargo test roadmap::model 2>&1 | tee /tmp/test-model-mapsplice-roadmap-5-1-4-item-1.out
```

Gate commands before committing:

```bash
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-5-1-4-item-1.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-4-item-1.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-4-item-1.out
```

Commit message subject:

```plaintext
Pin roadmap mutation invariants
```

### Work Item 2: Hide `RenumberPlan` map mutation

This commit hides the nested renumber-map field and routes construction through
the existing method seam. It implements `docs/mapsplice-design.md` C2 and C3,
`docs/developers-guide.md` section 2's domain boundary, and
`docs/roadmap.md` task 5.1.4's nested-map invariant.

Skills to load for this work item: `rust-router`, `rust-types-and-apis`,
`rust-unit-testing`, `leta`, `sem`, and `execplans`.

In `src/roadmap/model.rs`:

- make `RenumberPlan::by_source` private;
- rename the crate-private mutation method from `insert` to
  `record_mapping`, unless local naming review finds `insert` clearer in every
  call site;
- keep `resolve` and `resolve_unique` public on the type, with unchanged
  semantics;
- add a crate-private `is_empty` or `len` helper only if a focused call site
  needs it. Do not expose the nested map by reference.

In `src/roadmap/ops/rewrite.rs` and `src/roadmap/ops/dependency_text.rs`,
replace direct construction calls with the chosen method name. No caller may
use `by_source` directly after this work item.

Red proof: after making the field private but before updating all call sites,
run `cargo test roadmap::ops::dependency_text::tests -- --nocapture` and
confirm compilation fails at the outdated direct or old-method usage. Then
finish the call-site updates and rerun focused tests.

Focused validation commands:

```bash
cargo test roadmap::model 2>&1 | tee /tmp/test-model-mapsplice-roadmap-5-1-4-item-2.out
cargo test roadmap::ops::dependency_text::tests -- --nocapture 2>&1 | tee /tmp/test-dependency-text-mapsplice-roadmap-5-1-4-item-2.out
cargo test --test roadmap_properties -- --nocapture 2>&1 | tee /tmp/test-properties-mapsplice-roadmap-5-1-4-item-2.out
```

Gate commands before committing:

```bash
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-5-1-4-item-2.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-4-item-2.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-4-item-2.out
```

Commit message subject:

```plaintext
Hide renumber-plan map mutation
```

### Work Item 3: Encapsulate parser task-child accumulation

This commit makes `TaskChildren` maintain its own body, sub-task, and ordered
child state. It implements `docs/mapsplice-design.md` C4, F1, and F3;
`docs/developers-guide.md` section 2's parse/model boundary; and
`docs/roadmap.md` task 5.1.4's ordered-child invariant.

Skills to load for this work item: `rust-router`, `rust-types-and-apis`,
`rust-unit-testing`, `leta`, `sem`, and `execplans`.

In `src/roadmap/parse/task_children.rs`:

- make `body`, `sub_tasks`, and `ordered` private;
- keep `new`;
- replace `flush_body` with a private helper if only the accumulator needs it;
- add `push_body_node(&mut self, node: Node, source_text: &str)` to preserve
  body snippets through `MarkdownNodes::push_preserved`;
- add `push_sub_task(&mut self, sub_task: SubTaskEntry)` that flushes pending
  body before recording both `TaskChild::SubTask(sub_task.identity)` and the
  sub-task entry;
- add `next_sub_task_ordinal(&self) -> Result<u32>` or
  `expected_sub_task_ordinal(&self, offset: usize) -> Result<u32>` so overflow
  handling remains in one place with the current diagnostic text;
- add `finish(self) -> (MarkdownNodes, Vec<SubTaskEntry>, Vec<TaskChild>)`
  that flushes trailing body and returns the three owned values.

In `src/roadmap/parse/mod.rs`, route `split_task_children` and
`parse_sub_task_list` through those methods. Do not push to `TaskChildren`
fields directly.

Red proof: make the fields private first and run `cargo test --test
roadmap_sub_tasks parse_roadmap_keeps_nested_numbered_sub_tasks_structural
-- --nocapture`; confirm compilation fails at direct parser field access. Then
replace the call sites and rerun focused tests.

Focused validation commands:

```bash
cargo test --test roadmap_sub_tasks -- --nocapture 2>&1 | tee /tmp/test-sub-tasks-mapsplice-roadmap-5-1-4-item-3.out
cargo test --test roadmap_parse -- --nocapture 2>&1 | tee /tmp/test-parse-mapsplice-roadmap-5-1-4-item-3.out
cargo test --test roadmap_golden -- --nocapture 2>&1 | tee /tmp/test-golden-mapsplice-roadmap-5-1-4-item-3.out
```

Gate commands before committing:

```bash
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-5-1-4-item-3.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-4-item-3.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-4-item-3.out
```

Commit message subject:

```plaintext
Encapsulate task-child parsing
```

### Work Item 4: Route sub-task splices through `TaskEntry` methods

This commit hides direct mutation of the parallel sub-task and ordered-child
vectors behind invariant-preserving methods on `TaskEntry`. It implements
`docs/mapsplice-design.md` C4 and F5, `docs/developers-guide.md` sections 2
and 6, and `docs/roadmap.md` task 5.1.4's "callers cannot bypass" success
criterion.

Skills to load for this work item: `rust-router`, `rust-types-and-apis`,
`rust-unit-testing`, `rust-verification`, `proptest`, `leta`, `sem`, and
`execplans`.

In `src/roadmap/model.rs`:

- make `TaskEntry::sub_tasks` and `TaskEntry::children` private;
- add `TaskEntry::sub_tasks(&self) -> &[SubTaskEntry]`;
- add `TaskEntry::sub_tasks_mut(&mut self) -> &mut [SubTaskEntry]` only if
  renumbering and dependency rewriting cannot be expressed through narrower
  task methods;
- add `TaskEntry::children(&self) -> &[TaskChild]` only for renderer or tests
  that need read-only ordered children;
- add `TaskEntry::find_sub_task_index(&self, target: SubTaskNumber) ->
  Option<usize>`;
- add `TaskEntry::insert_sub_tasks(&mut self, sub_task_index: usize,
  child_index: usize, after: bool, sub_tasks: Vec<SubTaskEntry>)`;
- add `TaskEntry::delete_sub_task(&mut self, sub_task_index: usize,
  child_index: usize)`;
- add `TaskEntry::replace_sub_task(&mut self, sub_task_index: usize,
  child_index: usize, sub_tasks: Vec<SubTaskEntry>)`;
- ensure those methods build `TaskChild::SubTask` values from the inserted
  sub-task identities internally. No external caller should pass both
  `Vec<SubTaskEntry>` and a parallel `Vec<TaskChild>`.

If the method signatures above feel too index-heavy during implementation, do
not invent a broad abstraction. Keep index discovery in `src/roadmap/ops/sub_task.rs`
and keep the actual vector mutation inside `TaskEntry`; the invariant being
encapsulated is the paired mutation, not target lookup.

In `src/roadmap/ops/sub_task.rs`, replace the direct `task.sub_tasks` and
`task.children` mutations with these methods. Preserve the existing
`AnchorNotFound` and `InvalidRoadmap` diagnostics.

In `src/roadmap/render.rs`, `src/roadmap/ops/rewrite.rs`,
`src/roadmap/render_tests.rs`, `tests/roadmap_sub_tasks.rs`, and any affected
test support, replace read access with the new read-only accessors or narrow
methods. Do not expose mutable child vectors to tests.

In `tests/compile_time.rs`, add:

```rust
let cases = trybuild::TestCases::new();
cases.compile_fail("tests/ui/model_invariants.rs");
```

Create `tests/ui/model_invariants.rs` to parse a roadmap and attempt to mutate
`task.sub_tasks` or `task.children` directly. The test must fail because those
fields are private. Use the locked `trybuild` 1.0.117 workflow: first run the
compile test, inspect the generated `wip/model_invariants.stderr`, move it to
`tests/ui/model_invariants.stderr`, and rerun until stable.

Red proof: before making the fields private, the new compile-fail test should
fail because the snippet unexpectedly compiles. After making fields private and
updating call sites, the same test should pass by matching the checked-in
stderr.

Focused validation commands:

```bash
cargo test --test compile_time -- --nocapture 2>&1 | tee /tmp/test-compile-time-mapsplice-roadmap-5-1-4-item-4.out
cargo test --test roadmap_sub_tasks -- --nocapture 2>&1 | tee /tmp/test-sub-tasks-mapsplice-roadmap-5-1-4-item-4.out
cargo test --test roadmap_properties -- --nocapture 2>&1 | tee /tmp/test-properties-mapsplice-roadmap-5-1-4-item-4.out
cargo test --test roadmap_render -- --nocapture 2>&1 | tee /tmp/test-render-mapsplice-roadmap-5-1-4-item-4.out
```

Gate commands before committing:

```bash
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-5-1-4-item-4.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-4-item-4.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-4-item-4.out
```

Commit message subject:

```plaintext
Encapsulate sub-task child mutation
```

### Work Item 5: Mark roadmap task 5.1.4 complete

This commit updates living documentation only after all code and tests above
are complete. It implements `docs/roadmap.md` task 5.1.4's status update and
AGENTS.md documentation-maintenance guidance.

Skills to load for this work item: `execplans`, `en-gb-oxendict-style`,
`commit-message`, and `sem`.

In `docs/roadmap.md`, change task 5.1.4 from `[ ]` to `[x]` only after Work
Items 1-4 are committed and gate-clean.

Update this ExecPlan's `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` sections with final evidence. Change status to
`COMPLETE` only after final gates pass.

Format only the changed Markdown paths. Because both files definitely exist at
this point, the direct path list is safe:

```bash
mdtablefix --renumber docs/execplans/roadmap-5-1-4.md docs/roadmap.md 2>&1 | tee /tmp/markdownfmt-mapsplice-roadmap-5-1-4-final-mdtablefix.out
markdownlint-cli2 --fix docs/execplans/roadmap-5-1-4.md docs/roadmap.md 2>&1 | tee /tmp/markdownfmt-mapsplice-roadmap-5-1-4-final-markdownlint.out
```

Gate commands before committing:

```bash
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-5-1-4-final.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-4-final.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-4-final.out
```

Commit message subject:

```plaintext
Complete roadmap 5.1.4
```

## Concrete steps

Start each implementation session with:

```bash
cd /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-4
git branch --show-current
git status --short
```

Expected branch output:

```plaintext
roadmap-5-1-4
```

Before editing code, retry advisory tools and record failures or evidence in
this plan:

```bash
leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-4
leta files | head -n 120
sem diff --from origin/main --to HEAD --format json
```

If Memtrace MCP calls are available, run:

```plaintext
mcp__memtrace.list_indexed_repositories
mcp__memtrace.find_code(repo_id="mapsplice", view="committed", query="RenumberPlan TaskChildren sub_tasks children")
mcp__memtrace.get_symbol_context(repo_id="mapsplice", view="committed", symbol="RenumberPlan")
mcp__memtrace.get_symbol_context(repo_id="mapsplice", view="committed", symbol="TaskChildren")
```

If a Memtrace call is cancelled, rejected, or unavailable, record the exact
tool output in `Surprises & discoveries` and proceed with bounded local source
inspection.

For every work item:

1. Update `Progress` before editing.
2. Make the smallest change that satisfies that work item.
3. Run the focused validation commands for that work item.
4. Run `make all`, `make markdownlint`, and `make nixie` sequentially with
   `tee`.
5. Update this ExecPlan with evidence and any surprises.
6. Stage only the files belonging to that work item.
7. Commit with `git commit -F "$COMMIT_MSG_DIR/COMMIT_MSG.md"` using the
   `commit-message` skill.

## Validation and acceptance

The final acceptance criteria are:

- Users can still run `append`, `insert`, `delete`, and `replace` with the same
  command syntax and output modes documented in `docs/users-guide.md`.
- Existing golden, BDD, property, unit, and compile-time tests pass.
- `RenumberPlan` no longer exposes its nested map for direct mutation.
- `TaskChildren` no longer exposes `body`, `sub_tasks`, or `ordered` for
  direct parser mutation.
- Sub-task splice operations mutate ordered children through `TaskEntry`
  methods that update structural sub-tasks and ordered child references
  together.
- Compile-time coverage proves an external caller cannot mutate task
  `sub_tasks` or `children` fields directly.
- Tests pin task-body order, sub-task order, and dependency-renumber behaviour
  after encapsulation.
- No public CLI, documented public library entry point, diagnostic text,
  rendered Markdown fixture, or metrics behaviour changes.

Full final validation:

```bash
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-5-1-4-final.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-4-final.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-4-final.out
```

Expected outcome: all three commands exit 0. `make all` includes the current
`typecheck` target on `origin/main`.

## Idempotence and recovery

All planned production changes are local refactors and should be safe to retry.
If a work item fails after partial edits, inspect `git diff` and either finish
the same work item or manually revert only the lines changed in that failed
attempt. Do not use `git reset --hard` or `git checkout --` unless the user
explicitly requests it.

If `trybuild` creates `wip/model_invariants.stderr`, inspect it before moving
it. Move only the intended stderr file to `tests/ui/model_invariants.stderr`.
Remove any unrelated `wip` artefacts created by mistaken compile-fail runs.

If Markdown formatting changes unrelated files, stop, inspect `git diff`, and
park or discard unrelated formatter churn using a named stash:

```bash
git stash push -m 'df12-stash v1 task=5.1.4 kind=discard reason="unrelated markdown formatter churn"' -- <paths>
```

Do not stash with a bare or default message.

## Interfaces and dependencies

No new external dependencies are allowed.

The implementation must use these existing Rust interfaces and locked crates:

- `std::collections::BTreeMap`, already used by `RenumberPlan`.
- `markdown` 1.0.0, locked in `Cargo.lock`, for mdast parsing only. The plan
  does not require a Markdown writer API.
- `rstest` 0.26.1, locked in `Cargo.lock`, for named matrix tests and fixtures.
- `proptest` 1.11.0, locked in `Cargo.lock`, for generated reference rewrite
  invariants if Work Item 1 needs generated coverage.
- `serial_test` 3.5.0, locked in `Cargo.lock`, only for tests that mutate
  process-global state.
- `trybuild` 1.0.117, locked in `Cargo.lock`, for compile-fail proof that
  task child vectors cannot be directly mutated by external callers.

The target internal methods at the end of this plan are:

```rust
impl RenumberPlan {
    pub fn resolve(&self, source: SourceId, anchor: RoadmapAnchor) -> Option<RoadmapAnchor>;
    pub fn resolve_unique(&self, anchor: RoadmapAnchor) -> Option<RoadmapAnchor>;
    pub(crate) fn record_mapping(
        &mut self,
        source: SourceId,
        old: RoadmapAnchor,
        new: RoadmapAnchor,
    );
}

impl TaskChildren {
    pub(super) const fn new() -> Self;
    pub(super) fn push_body_node(&mut self, node: Node, source_text: &str);
    pub(super) fn push_sub_task(&mut self, sub_task: SubTaskEntry);
    pub(super) fn expected_sub_task_ordinal(&self, offset: usize) -> Result<u32>;
    pub(super) fn finish(self) -> (MarkdownNodes, Vec<SubTaskEntry>, Vec<TaskChild>);
}

impl TaskEntry {
    pub(crate) fn from_parts(parts: TaskEntryParts) -> Result<Self>;
    pub fn sub_tasks(&self) -> &[SubTaskEntry];
    pub(crate) fn sub_tasks_mut(&mut self) -> &mut [SubTaskEntry];
    pub(crate) fn children(&self) -> &[TaskChild];
    pub(crate) fn find_sub_task_index(&self, target: SubTaskNumber) -> Option<usize>;
    pub(crate) fn insert_sub_tasks(
        &mut self,
        splice: SubTaskSplice,
        after: bool,
        sub_tasks: Vec<SubTaskEntry>,
    );
    pub(crate) fn delete_sub_task(&mut self, splice: SubTaskSplice);
    pub(crate) fn replace_sub_task(
        &mut self,
        splice: SubTaskSplice,
        sub_tasks: Vec<SubTaskEntry>,
    );
}
```

If implementation proves a listed signature needs a narrower helper to preserve
diagnostics or borrowing clarity, record the change in `Decision Log` and keep
the invariant: callers must not directly mutate the nested renumber map or the
paired sub-task/ordered-child vectors.

## Outcomes & retrospective

Completed roadmap task 5.1.4 as five atomic commits:

- `Pin roadmap mutation invariants`
- `Hide renumber-plan map mutation`
- `Encapsulate task-child parsing`
- `Encapsulate sub-task child mutation`
- `Complete roadmap 5.1.4`

`RenumberPlan` no longer exposes its nested source map for direct mutation.
`TaskChildren` no longer exposes parser accumulator vectors. `TaskEntry`
sub-task and ordered-child vectors are private and are mutated through
model-owned methods that update both structures together. Compile-time
coverage now proves external callers cannot clear `TaskEntry::sub_tasks`
directly, and focused runtime tests cover task-body order, sub-task order, and
dependency rewrite behaviour.

Final validation passed through scrutineer for Work Item 5:
`make all`, `make markdownlint`, and `make nixie` all exited 0. CodeRabbit
review was attempted for every work item and deferred each time because this
sandbox has no default network route; no CodeRabbit findings were produced.

## Revision note

- 2026-07-03T10:34:32Z: Created the first-round draft for roadmap task 5.1.4.
  The plan records advisory-tool failures, pins locked test-library behaviour
  to local source evidence, and decomposes the task into five independently
  committable work items.
