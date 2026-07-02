# Audit Operations for Fail-Closed Behaviour

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

## Purpose / Big Picture

Roadmap task 4.1.1, "Audit the operations for fail-closed behaviour", proves
that each structural operation rejects invalid input before any output or
in-place write can occur. The four operations are `append`, `insert`, `delete`,
and `replace`. Fail-closed means that malformed grammar, fragment parse
failure, fragment-level mismatch, missing anchors, operation-application
failure, dependency-rewrite failure, and render failure all return typed
`MapspliceError` values without committing a damaged roadmap.

The observable outcome is a contract-level audit matrix plus a unit-level
staging assertion. A maintainer can run the focused tests and then `make all`,
`make markdownlint`, and `make nixie` and see green results. `docs/roadmap.md`
marks task 4.1.1 complete only after those tests prove the contract.

## Constraints

- Work only inside
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-1-1`.
- Do not edit the root/control worktree.
- Treat `origin/main` as the integration branch and `docs/roadmap.md` as the
  roadmap source of truth.
- Do not implement during the draft phase. This is planning round 3 and must
  be approved before implementation begins.
- Preserve the command semantics in `docs/users-guide.md`: stdout mode writes
  the rendered roadmap to standard output, while `--in-place` rewrites the
  target only on success.
- Preserve the fail-closed contract in `docs/mapsplice-design.md` section 5
  (F5) and section 6 (C6). Invalid input must not produce partial rendered
  output, and in-place writes must happen only after validation, mutation,
  renumbering, dependency rewriting, and rendering all succeed.
- Keep public errors typed as `MapspliceError`, per
  `docs/developers-guide.md` section 3 and `AGENTS.md` "Error Handling".
- Do not add external dependencies. Use locked test dependencies already
  present in `Cargo.lock`.
- Keep every code file below 400 lines and add module-level comments to any
  new Rust module, per `AGENTS.md` "Code Style and Structure" and "Rust
  Specific Guidance".
- Follow en-GB Oxford spelling in prose, comments, and commit messages.
- Use Red-Green-Refactor where the current implementation is deficient. Where
  the current implementation already passes the new audit tests, prove the
  tests can fail with a temporary local mutation and revert that mutation
  before committing.
- Start every implementation work item from a clean baseline:
  `git status --short` must print nothing. If it prints any unrelated dirty
  file, stop before editing and resolve the ownership question rather than
  using a dynamic `git diff --name-only` formatter list.
- Format only the explicit Markdown paths named by the current work item. Do
  not run repo-global Markdown formatters such as `make fmt` or
  `mdformat-all`.
- Run test, lint, and formatting gates sequentially with `tee` logs under
  `/tmp`. Do not run tests in parallel.
- Use the shared Cargo cache. Do not create an isolated Cargo cache.

## Tolerances

- Stop and escalate if implementation requires a public API signature change,
  a new external dependency, or a change to the accepted roadmap grammar.
- Stop and escalate if the audit needs changes outside these planned files:
  `docs/execplans/roadmap-4-1-1.md`, `docs/roadmap.md`,
  `tests/roadmap_golden/contracts.rs`, `tests/golden/assertions.rs`,
  `tests/golden/case.rs`, `tests/golden/runner.rs`,
  `tests/fixtures/golden/*`, `src/lib.rs`, `src/roadmap/ops/mod.rs`,
  `src/roadmap/parse/document.rs`, `src/roadmap/parse/fragment.rs`, and
  `src/roadmap/render.rs`. The source files are listed because a failing audit
  may expose a real bug; tests and documentation remain the expected primary
  surface.
- Stop and escalate if the net implementation exceeds 260 changed lines
  excluding golden fixture Markdown.
- Stop and escalate if an operation can only be made fail-closed by changing
  successful output formatting or dependency-reference rewriting semantics.
- Stop and escalate if a focused test still fails for the same reason after
  three implementation attempts.
- Stop and escalate if `make all` fails for an unrelated pre-existing issue
  that cannot be isolated with a focused command and a log.
- Stop and escalate if multiple valid interpretations of "no partial output"
  remain after applying `docs/users-guide.md` "Output modes" and
  `docs/mapsplice-design.md` section 5 (F5).

## Risks

- Risk: Existing coverage may already pass for several fail-closed cases,
  making a pure red test impossible without a temporary mutation.
  Severity: medium.
  Likelihood: high.
  Mitigation: use mutation-style proof for audit-only cases. Temporarily
  mutate the relevant branch-local code, run the new focused test to observe
  the expected failure, then immediately revert the temporary mutation before
  committing.

- Risk: The golden harness checks no-output indirectly, because
  `run_from_args` returns `Err` rather than `RunOutcome::stdout` on failure.
  Severity: medium.
  Likelihood: medium.
  Mitigation: keep using the harness assertion that any failure case returning
  `Ok(RunOutcome::stdout(_))` is a test failure, and pair it with unchanged
  target assertions for stdout and in-place modes.

- Risk: Append level mismatch uses `MapspliceError::AppendLevelMismatch`,
  while insert and replace use `MapspliceError::LevelMismatch`.
  Severity: low.
  Likelihood: high.
  Mitigation: extend the golden failure vocabulary to distinguish append-level
  mismatch from anchor-level mismatch instead of weakening the assertion to a
  string match.

- Risk: A bug could mutate the parsed roadmap before returning an error, but
  still leave the target file unchanged when no in-place write occurs.
  Severity: high.
  Likelihood: medium.
  Mitigation: add a unit-level `apply_command` assertion that the input
  `RoadmapDocument` remains unchanged on `Err`. This directly observes the
  staging-order contract the golden `run_from_args` boundary cannot see.

- Risk: Render failures may be asserted only by temporary mutation proof, which
  would leave F5's render-failure branch unprotected after the commit.
  Severity: high.
  Likelihood: medium.
  Mitigation: add a durable golden `--in-place` failure case that uses a
  parse-valid target with an unsupported inline image in a task summary. The
  operation must reach `render_roadmap`, return
  `MapspliceError::InvalidRoadmap`, produce no `RunOutcome::stdout`, and leave
  the target byte-identical.

## Progress

- [x] (2026-07-02T08:23:55Z) Read `AGENTS.md`,
  `docs/mapsplice-design.md`, `docs/developers-guide.md`,
  `docs/users-guide.md`, `docs/contributing.md`,
  `docs/documentation-style-guide.md`, `docs/scripting-standards.md`, and the
  4.1.1 roadmap entry.
- [x] (2026-07-02T08:23:55Z) Loaded the `execplans`, `leta`, `rust-router`,
  `rust-errors`, `rust-unit-testing`, `rust-verification`,
  `domain-cli-and-daemons`, `sem`, and `firecrawl-mcp` skills.
- [x] (2026-07-02T08:34:40Z) Re-read the required planning, Leta, Memtrace,
  Firecrawl, Rust router, semantic diff, commit-message, and en-GB Oxford
  spelling skills for planning round 2.
- [x] (2026-07-02T08:34:40Z) Attempted Memtrace primary discovery with
  `list_indexed_repositories`; the MCP tool returned
  `user cancelled MCP tool call`. Continued with bounded branch-local evidence.
- [x] (2026-07-02T08:34:40Z) Confirmed Leta workspace registration with
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-1-1`;
  symbol reads such as `leta show run_request`, `leta show apply_command`, and
  `leta show assert_failed_case` succeeded. `leta files src` and
  `leta files tests` failed with `Error: Failed to start daemon`, so file-tree
  orientation used precise known-path inspection.
- [x] (2026-07-02T08:34:40Z) Ran `sem diff --from origin/main --to HEAD
  --format json`; it reported the existing ExecPlan addition on this branch.
- [x] (2026-07-02T08:34:40Z) Attempted Firecrawl scrape for the locked
  `rstest` docs.rs page; it returned `user cancelled MCP tool call`. Continued
  with local Cargo registry source.
- [x] (2026-07-02T08:34:40Z) Verified branch-local code paths for
  `run_request`, `apply_command`, golden failure assertions, append/insert/
  delete/replace operations, render errors, and CLI behavioural steps.
- [x] (2026-07-02T08:34:40Z) Revised the plan to address the round-2 design
  review blockers: explicit Markdown file lists, in-place coverage across
  shared failure boundaries, and unit-level `apply_command` staging proof.
- [x] (2026-07-02T09:18:00Z) Re-read the `execplans`, `leta`,
  `memtrace-first`, `rust-router`, `rust-errors`, `rust-unit-testing`,
  `rust-verification`, `proptest`, `domain-cli-and-daemons`, `firecrawl-mcp`,
  and `sem` skills for planning round 3.
- [x] (2026-07-02T09:18:00Z) Attempted Memtrace primary discovery with
  `mcp__memtrace.list_indexed_repositories`; the MCP host returned
  `user cancelled MCP tool call`. Continued with bounded branch-local evidence.
- [x] (2026-07-02T09:18:00Z) Verified branch-local render flow with
  `leta show run_request`, `leta show run_from_args`,
  `leta show render_roadmap`, `leta show render_block`,
  `leta show render_inline_node`, `leta show apply_command`,
  `leta show assert_failed_case`, and `leta show assert_failure_output`.
  Later Leta queries failed with `Error: Connection closed unexpectedly`, so
  exact known-path source windows were used for parser and harness details.
- [x] (2026-07-02T09:18:00Z) Attempted
  `mcp__firecrawl.firecrawl_scrape` for
  `https://docs.rs/rstest/0.26.1/rstest/`; the MCP host returned
  `user cancelled MCP tool call`. Continued with `Cargo.lock` and local Cargo
  registry source for locked dependency behaviour.
- [x] (2026-07-02T09:18:00Z) Verified the durable render-failure mechanism:
  `markdown` 1.0.0 defines `Node::Image`, `parse_task_paragraph` preserves
  inline nodes after the task-number prefix, and
  `render_inline_node` rejects unsupported inline nodes with
  `MapspliceError::InvalidRoadmap`.
- [x] (2026-07-02T09:30:00Z) Draft reviewed and approved for implementation
  by the df12-build roadmap workflow task assignment.
- [x] (2026-07-02T09:30:00Z) Work item 1 complete: extended the failure
  assertion vocabulary with `ExpectedError::AppendLevelMismatch`, added the
  `f5_append_level_mismatch_failure` in-place golden case, observed the red
  failure before the assertion mapping, and passed the focused test plus
  `make all`, `make markdownlint`, and `make nixie`.
- [x] (2026-07-02T09:45:00Z) Work item 2 complete: added
  `apply_command_leaves_roadmap_unchanged_on_error`, proved it catches an
  early staged assignment with a temporary mutation, reverted that mutation,
  and passed the focused test plus `make all`.
- [x] (2026-07-02T10:05:00Z) Work item 3 complete: added the
  `f5_in_place_boundary_matrix` golden matrix for target parse, fragment
  parse, fragment level, missing anchor, and dependency-rewrite failures;
  proved the in-place boundary catches an early target rewrite with a
  temporary mutation; reverted that mutation; and passed the focused test plus
  `make all`, `make markdownlint`, and `make nixie`.
- [x] (2026-07-02T10:25:00Z) Work item 4 complete: added the
  `f5_render_failure_in_place` golden case with a parse-valid unsupported
  inline image, proved an early in-place rewrite changes the target with a
  temporary mutation, reverted that mutation, and passed the focused test plus
  `make all`, `make markdownlint`, and `make nixie`.
- [x] (2026-07-02T10:35:00Z) Work item 5 complete: audited the validated
  failure matrix and render case against bounded source windows for
  `run_request` and `apply_command`; no implementation gaps were exposed, so
  no production changes were required.
- [x] (2026-07-02T10:50:00Z) Work item 6 complete: reconciled this ExecPlan
  and marked `docs/roadmap.md` task 4.1.1 complete after the audit matrix,
  render-failure case, and final gates were green.
- [x] (2026-07-02T09:32:58Z) Fix round 1 complete: rebased
  `roadmap-4-1-1` onto `origin/main`, removing the branch-relative deletion
  of `docs/execplans/roadmap-3-1-3.md`, `docs/issues/audit-3.1.3.md`, and
  `tests/golden/format_gate.rs`; confirmed `docs/roadmap.md` keeps task 3.1.3
  complete and `tests/golden/mod.rs` re-exports
  `assert_gate_clean_rendered_output`.

## Surprises & Discoveries

- Observation: `src/lib.rs::run_request` reads the target, parses the roadmap,
  loads and parses the fragment, applies the command, renders the staged
  roadmap, and only then calls `rewrite_utf8` for `--in-place`.
  Evidence: `leta show run_request`.
  Impact: in-place behaviour can be tested at the library boundary for every
  user-reachable failure before rewrite.

- Observation: `src/roadmap/ops/mod.rs::apply_command` clones the roadmap into
  `staged`, mutates the clone, renumbers it, rewrites dependencies, and assigns
  `*roadmap = staged` only after all those steps succeed.
  Evidence: `leta show apply_command`.
  Impact: staging-order acceptance needs a direct unit assertion, because
  golden `run_from_args` failures cannot observe a temporary internal
  assignment that is not returned or written.

- Observation: `tests/golden/runner.rs::assert_failed_case` fails any golden
  failure case that returns `Ok`, including `RunOutcome::stdout`, before it
  checks the unchanged target.
  Evidence: `leta show assert_failed_case`.
  Impact: the existing golden harness is sufficient for no-output assertions
  at the `run_from_args` boundary.

- Observation: `src/roadmap/render.rs::render_block` and
  `render_inline_node` return `MapspliceError::InvalidRoadmap` for unsupported
  mdast nodes.
  Evidence: `leta show render_block` and `leta show render_inline_node`.
  Impact: render failures are real error paths. The plan requires a committed
  golden failure case that reaches `render_roadmap`, plus temporary mutation
  proof that moving `rewrite_utf8` earlier than render or dependency rewriting
  is caught by unchanged-target assertions.

- Observation: `markdown` 1.0.0 defines `Node::Image`, and
  `src/roadmap/parse/mod.rs::parse_task_paragraph` keeps inline nodes that
  follow the task-number prefix in the task summary.
  Evidence:
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/mdast.rs`
  and `sed -n '284,310p' src/roadmap/parse/mod.rs`.
  Impact: a task such as `- [ ] 1.1.1. Task ![alt](image.png)` is parse-valid
  but fails during summary rendering, making it a durable render-failure
  fixture.

- Observation: `tests/golden/assertions.rs::assert_expected_error` does not
  currently distinguish `AppendLevelMismatch`.
  Evidence: `leta show assert_expected_error`.
  Impact: work item 1 must extend the expected-error enum before the append
  mismatch audit can be precise.

- Observation: Implementation-session Memtrace and Leta calls were unavailable.
  Evidence: `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`; `leta workspace add` returned
  `Error: IO error: Read-only file system (os error 30)`; and `leta show`
  calls returned `Error: Failed to start daemon`.
  Impact: work item 1 used the plan's already bounded branch-local evidence
  plus exact known-path inspection. This did not block implementation because
  the approved plan explicitly permits fallback when these tools fail.

- Observation: The requested `scrutineer` sub-agent could not run gates in
  this session.
  Evidence: the sub-agent returned `You've hit your usage limit for
  GPT-5.3-Codex-Spark. Switch to another model now, or try again at Jul 7th,
  2026 12:20 PM.`
  Impact: work item 1 gates were run directly from the assigned worktree with
  `tee` logs under `/tmp`.

- Observation: Fix round 1 could not use Memtrace or `scrutineer`.
  Evidence: `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`; `goose run --recipe
  /home/leynos/.config/goose/recipes/scrutineer.yaml --no-session --max-turns
  30` returned `No provider configured. Run 'goose configure' first.`
  Impact: the fix used `sem diff`, branch-local Leta/file verification, and
  direct sequential gate execution from the assigned worktree with `/tmp`
  logs.

- Observation: The branch was behind `origin/main` by two commits before fix
  round 1.
  Evidence: `git status --short --branch` printed
  `## roadmap-4-1-1...origin/main [ahead 9, behind 2]`; after
  `git rebase origin/main`, it printed
  `## roadmap-4-1-1...origin/main [ahead 9]`.
  Impact: the blocking review finding was an integration drift issue rather
  than a new code defect in the fail-closed audit itself.

## Decision Log

- Decision: Use the existing golden fixture harness as the main behavioural
  audit surface.
  Rationale: It already drives `run_from_args`, prepares real target and
  fragment files, supports success and failure expectations, fails on any
  unexpected `RunOutcome`, and asserts target preservation on failure.
  Date/Author: 2026-07-02T08:23:55Z, Codex.

- Decision: Treat unavailable Memtrace and Firecrawl calls as recorded
  planning evidence, not blockers.
  Rationale: The task explicitly says advisory-tool unavailability must not set
  the ExecPlan status to blocked. Branch-local Leta/source inspection, docs,
  tests, `sem`, and locked local Cargo registry source were sufficient to
  produce an implementable plan. Implementation should retry Firecrawl if it
  introduces any new load-bearing external-library assumption.
  Date/Author: 2026-07-02T08:34:40Z, Codex.

- Decision: Do not add a new dependency or new test framework for this audit.
  Rationale: `Cargo.toml` and `Cargo.lock` already include `rstest`,
  `serial_test`, `tempfile`, `cap-std`, and `rstest-bdd`. Existing integration
  helpers already create temporary file-backed workspaces and run the library
  boundary.
  Date/Author: 2026-07-02T08:23:55Z, Codex.

- Decision: Use explicit per-work-item Markdown path lists for formatting.
  Rationale: `git diff --name-only --diff-filter=ACM -- '*.md'` can include
  unrelated dirty Markdown that pre-dated the current work item. A clean
  baseline plus literal path lists prevents formatter churn from crossing
  ownership boundaries.
  Date/Author: 2026-07-02T08:34:40Z, Codex.

- Decision: Keep staging-order mutation proof at `apply_command` unit level.
  Rationale: A golden `run_from_args` failure that returns no output and writes
  no file cannot observe an internal `*roadmap = staged` assignment before a
  later error. The acceptance criterion belongs beside the operation boundary.
  Date/Author: 2026-07-02T08:34:40Z, Codex.

- Decision: Add render failure as its own durable golden work item instead of
  folding it into temporary rewrite-order mutation proof.
  Rationale: The F5 render-failure contract must remain protected after the
  implementation commit. A parse-valid unsupported inline image in a task
  summary reaches `render_roadmap`, returns `InvalidRoadmap`, and lets the
  golden runner prove no `RunOutcome::stdout` and no in-place write.
  Date/Author: 2026-07-02T09:18:00Z, Codex.

- Decision: Rebase onto `origin/main` for fix round 1 rather than editing
  around the drift.
  Rationale: The blocking review identified main-branch artefacts that this
  branch would delete if merged. Rebasing preserved the branch's 4.1.1 audit
  work while restoring the landed 3.1.3 plan, issue audit, roadmap status, and
  rendered-output format gate module.
  Date/Author: 2026-07-02T09:32:58Z, Codex.

## Outcomes & Retrospective

Work item 1 is complete. The new append-specific mismatch fixture proved the
golden failure vocabulary needed a distinct `AppendLevelMismatch` variant: the
red run failed with
`expected error AppendLevelMismatch but got cannot append task content; append
expects phase content`, and the green run passed after the assertion mapping.
No production code changes were required for this work item.

Work item 2 is complete. The new unit-level staging assertion confirms that
`apply_command` leaves the caller's `RoadmapDocument` unchanged when dependency
rewriting fails after the staged clone has already been mutated. A temporary
early assignment to `*roadmap` made the test fail, and removing the mutation
restored the green result.

Work item 3 is complete. The in-place matrix now covers the shared
fail-closed boundaries for all four operations with typed error assertions and
byte-identical target preservation. A temporary early target write in
`run_request` made the focused in-place case fail, confirming the matrix would
catch a rewrite-order regression. No production code changes were required.

Work item 4 is complete. The durable render-failure fixture reaches
`render_roadmap` after a successful append, returns typed
`MapspliceError::InvalidRoadmap` for an unsupported inline image, emits no
`RunOutcome::stdout`, and leaves the in-place target byte-identical. A
temporary early `rewrite_utf8` before rendering made the focused case fail,
then the mutation was reverted.

Work item 5 is complete. The audit found no production gap after the staging,
matrix, and render-failure tests all passed and their temporary mutations
failed as expected. Bounded source inspection confirmed `run_request` writes
in place only after parse, fragment loading, operation application, dependency
rewrite, and render success, while `apply_command` commits its staged clone
only after dependency rewriting succeeds.

Work item 6 is complete. The roadmap now marks 4.1.1 complete, and this
ExecPlan records the audit as complete. No user-guide or design-document
wording needed changing because the implemented tests confirmed the existing
F5 and C6 contracts rather than changing the contract.

Fix round 1 is complete. The branch is now integrated with `origin/main` and
no longer deletes the 3.1.3 F4 artefacts. The restored `format_gate` module is
imported by `tests/golden/mod.rs`, re-exports
`assert_gate_clean_rendered_output`, and is exercised by `make all` through the
two `golden::format_gate` tests. CodeRabbit remains an open deferred-review
item because the sandbox has no default network route.

## Context and Orientation

`mapsplice` is a Rust command-line tool that edits roadmap-shaped Markdown. The
normal flow is documented in `docs/mapsplice-design.md` section 3: parse
Markdown into a roadmap model, apply one splice operation, renumber, rewrite
dependency references, render, and write either to stdout or the target file.

The main implementation boundary is `src/lib.rs::run_request`. It translates
CLI input into a roadmap operation, reads the target, parses the target and
fragment, calls `src/roadmap/ops/mod.rs::apply_command`, renders the result,
and writes in place only when requested. The operation boundary is
`src/roadmap/ops/mod.rs::apply_command`; it stages work on a cloned roadmap
before committing the staged document back to the caller.

The failure assertions live under `tests/golden`. `tests/golden/runner.rs`
executes `run_from_args` against a temporary workspace.
`tests/golden/case.rs` defines the failure vocabulary.
`tests/golden/assertions.rs` maps an actual `MapspliceError` to the expected
error class and asserts the target file is unchanged. Contract-level tests
live in `tests/roadmap_golden/contracts.rs`.

Locked dependency behaviour verified for this plan:

- `rstest` is locked at 0.26.1 in `Cargo.lock`. The local registry source at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rstest-0.26.1/src/lib.rs`
  document `#[fixture]`, `#[rstest]`, and `#[case]` for fixtures and
  parameterized tests.
- `rstest-bdd` is locked at 0.5.0. The local registry source at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rstest-bdd-0.5.0/src/lib.rs`
  export step and scenario support, but this plan does not require new BDD
  macros unless the golden harness is later shown insufficient.
- `tempfile` is locked at 3.27.0. The local registry source at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tempfile-3.27.0/src/lib.rs`
  state that `tempdir()` creates a temporary directory and `TempDir` removes it
  on drop; existing `tests/support/workspace.rs` and
  `tests/golden/workspace.rs` keep the `TempDir` alive in workspace structs.
- `markdown` is locked at 1.0.0 in `Cargo.lock`. The local registry source at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/mdast.rs`
  defines `Node::Image` and `Image`, which is the load-bearing parser node for
  the durable render-failure fixture.

## Plan of Work

### Work item 1: Extend the failure assertion vocabulary

Documentation and skills to read first: `AGENTS.md` "Rust Specific Guidance",
`AGENTS.md` "Testing", `docs/developers-guide.md` sections 2, 3, and 6,
`docs/mapsplice-design.md` sections 5 and 6, `rust-router`,
`rust-errors`, and `rust-unit-testing`.

Edit `tests/golden/case.rs` to add an
`ExpectedError::AppendLevelMismatch` variant. Edit
`tests/golden/assertions.rs::assert_expected_error` to match
`MapspliceError::AppendLevelMismatch` only for that expected error. Keep the
existing `ExpectedError::LevelMismatch` mapped only to
`MapspliceError::LevelMismatch`.

Tests for this item are test-infrastructure tests. Add one small in-place
contract test case in `tests/roadmap_golden/contracts.rs` that uses an append
command with a task-level fragment, expects
`ExpectedError::AppendLevelMismatch`, and asserts
`FailureOutput::InPlaceTargetUnchanged`. Add the minimal definite fixtures
under
`tests/fixtures/golden/f5_append_level_mismatch_failure/target.md` and
`tests/fixtures/golden/f5_append_level_mismatch_failure/fragment.md`.

Red proof: before adding the enum mapping, add the test and run the focused
command. It must fail because `ExpectedError::AppendLevelMismatch` does not
exist yet or because the assertion cannot match the actual append mismatch.
Then add the enum mapping and rerun the focused command.

Changed Markdown files for this work item:
`docs/execplans/roadmap-4-1-1.md`,
`tests/fixtures/golden/f5_append_level_mismatch_failure/target.md`, and
`tests/fixtures/golden/f5_append_level_mismatch_failure/fragment.md`.

### Work item 2: Pin `apply_command` failure staging

Documentation and skills to read first: `docs/mapsplice-design.md` section 5
(F5), section 6 (C1-C3 and C6), and section 8;
`docs/developers-guide.md` sections 2, 3, and 6; `rust-router`,
`rust-errors`, `rust-unit-testing`, and `rust-verification`.

Add a unit test beside `src/roadmap/ops/mod.rs::apply_command`, or in the
existing test module for roadmap operations, named
`apply_command_leaves_roadmap_unchanged_on_error`. Construct a valid
`RoadmapDocument`, clone it as `original`, invoke `apply_command` with an
operation that fails after mutating the staged clone, and assert:

```rust
assert!(matches!(result, Err(MapspliceError::DanglingDependency { .. })));
assert_eq!(roadmap, original);
```

Use a dangling dependency failure for the red proof because it happens after
the operation has modified the staged document and after renumbering has
started, which directly tests that `*roadmap = staged` is not performed before
dependency rewriting succeeds. This unit test is required even if all golden
tests pass, because the golden harness cannot observe an internal assignment
that is later hidden by an error return.

Red proof: temporarily move `*roadmap = staged` above
`rewrite_dependencies(&mut staged, &plan)?` in `apply_command`, run the focused
unit test, and observe `assert_eq!(roadmap, original)` fail. Revert the
temporary mutation before the green commit.

Changed Markdown files for this work item: none.

### Work item 3: Add the in-place fail-closed boundary matrix

Documentation and skills to read first: `docs/users-guide.md` "Command
details", "Output modes", and "Validation rules and failure cases";
`docs/mapsplice-design.md` sections 4, 5 (F5), 6 (C1 and C6), and 8;
`docs/developers-guide.md` sections 2 and 6; `rust-unit-testing`;
`rust-verification`; and `domain-cli-and-daemons`.

Expand `tests/roadmap_golden/contracts.rs` with named `rstest` cases or small
individual tests that exercise `--in-place` for each shared failure boundary
below. Every case must assert a typed `MapspliceError` and
`FailureOutput::InPlaceTargetUnchanged`.

The required matrix is implemented by these definite fixture-backed cases:

- Target parse boundary:
  `f5_append_malformed_target_in_place`,
  `f5_insert_malformed_target_in_place`,
  `f5_delete_malformed_target_in_place`, and
  `f5_replace_malformed_target_in_place`.
- Fragment parse boundary:
  `f5_append_malformed_fragment_in_place`,
  `f5_insert_malformed_fragment_in_place`, and
  `f5_replace_malformed_fragment_in_place`.
- Fragment level boundary:
  `f5_append_level_mismatch_failure` from work item 1,
  `f5_insert_level_mismatch_in_place`, and
  `f5_replace_level_mismatch_in_place`.
- Anchor lookup boundary:
  `f5_insert_missing_phase_anchor_in_place`,
  `f5_delete_missing_step_anchor_in_place`,
  `f5_replace_missing_task_anchor_in_place`, and
  `f5_delete_missing_addendum_anchor_in_place`.
- Operation application and dependency-rewrite boundary:
  `f5_dependency_rewrite_failure_in_place`, using a mutation that otherwise
  reaches `rewrite_dependencies`.

This is a test-backed common-boundary argument rather than an exhaustive
cartesian product. It is sufficient because branch-local Leta inspection shows
all operations enter the same `run_request` sequence, `insert` and `replace`
share `validate_fragment_level`, anchored operations use the same typed
`AnchorNotFound` surface, and `rewrite_utf8` exists only after
`render_roadmap(&roadmap)?` in `run_request`. Work item 2 separately proves
that operation staging is not committed to the caller on `Err`, and work item
4 separately proves a real `render_roadmap` error.

Use golden fixture files under `tests/fixtures/golden/` for target and
fragment bodies. Reuse an existing fixture only when the case name still makes
the contract obvious. Prefer a small fixture per failure so each test failure
names the exact contract that regressed.

Red proof: if branch-local code already satisfies a matrix row, use the
smallest temporary mutation for that row, run the focused test to observe the
expected failure, and revert the mutation before committing. Examples include
temporarily skipping `parse_fragment` for one malformed-fragment case, removing
one `validate_fragment_level` call, or returning `Ok(0)` before
`rewrite_dependencies`.

Changed Markdown files for this work item:
`docs/execplans/roadmap-4-1-1.md`, every `target.md` and `fragment.md` under
the fixture directories named above, and no optional snapshot files.

### Work item 4: Add the durable render-failure case

Documentation and skills to read first: `docs/users-guide.md` "Output modes"
and "Validation rules and failure cases"; `docs/mapsplice-design.md` sections
5 (F5), 6 (C6), and 8; `docs/developers-guide.md` sections 2, 3, and 6;
`rust-router`, `rust-errors`, `rust-unit-testing`, `rust-verification`,
`domain-cli-and-daemons`, and `proptest` only if a generated render invariant
is considered after this concrete regression is committed.

Add a named golden failure test in `tests/roadmap_golden/contracts.rs` named
`f5_render_failure_in_place`. The case must drive `run_from_args` through
`GoldenCommand::Append`, must run with `--in-place`, must expect
`ExpectedError::InvalidRoadmap`, and must assert
`FailureOutput::InPlaceTargetUnchanged`.

Add definite fixtures under
`tests/fixtures/golden/f5_render_failure_in_place/target.md` and
`tests/fixtures/golden/f5_render_failure_in_place/fragment.md`. The target must
be otherwise valid roadmap grammar but include an unsupported inline image in a
task summary, for example:

```markdown
## 1. Existing phase

### 1.1. Existing step

- [ ] 1.1.1. Existing task ![unsupported](unsupported.png)
```

The fragment must be a valid phase-level append fragment. This shape is
load-bearing: `parse_task_paragraph` keeps the image node in the summary,
`apply_command` succeeds for the append, and `render_inline_node` returns
`MapspliceError::InvalidRoadmap` for `Node::Image` during
`render_roadmap(&roadmap)?`.

Acceptance for this work item is stricter than temporary mutation proof. The
committed test must prove that `run_from_args` returns
`Err(MapspliceError::InvalidRoadmap { .. })`, the golden runner sees no
`RunOutcome::stdout` because any `Ok(RunOutcome::stdout(_))` is an unexpected
success, and the in-place target remains byte-identical. Then perform one
temporary local mutation that moves `rewrite_utf8` above
`render_roadmap(&roadmap)?` in `src/lib.rs::run_request`; the focused test
must fail because the target changes. Revert the temporary mutation before any
commit.

Changed Markdown files for this work item:
`docs/execplans/roadmap-4-1-1.md`,
`tests/fixtures/golden/f5_render_failure_in_place/target.md`, and
`tests/fixtures/golden/f5_render_failure_in_place/fragment.md`.

### Work item 5: Close implementation gaps exposed by the matrix

Documentation and skills to read first: `docs/developers-guide.md` sections 2,
3, and 5; `docs/mapsplice-design.md` sections 5 and 6; `rust-errors`;
`rust-unit-testing`; and `domain-cli-and-daemons`. Use Memtrace
`get_symbol_context`, `get_impact`, and `get_timeline` before changing
`run_request`, `apply_command`, parser entry points, render code, or file
rewrite code if Memtrace is available in the implementation session. Use Leta
symbol navigation first if its daemon is available; otherwise record the exact
failure and inspect bounded source windows.

If all work item 2, work item 3, and work item 4 tests pass without production
changes and the temporary mutation checks fail as expected, this item becomes
an audit commit that only records progress in this ExecPlan. If any matrix or
render-failure test exposes a real bug, make the smallest production change
that preserves these
invariants:

- `src/lib.rs::run_request` must call `rewrite_utf8` only after target parse,
  fragment parse and level validation, operation application, dependency
  rewriting, and rendering have all succeeded.
- `src/roadmap/ops/mod.rs::apply_command` must continue staging mutations on a
  clone and must not assign the staged roadmap back to the caller until all
  validation, renumbering, and dependency rewriting has succeeded.
- Parser failures from `src/roadmap/parse/document.rs` and
  `src/roadmap/parse/fragment.rs` must remain typed
  `MapspliceError::InvalidRoadmap` or `MapspliceError::Markdown` as
  appropriate.
- Render failures from `src/roadmap/render.rs` must remain typed
  `MapspliceError::InvalidRoadmap` and must not trigger `rewrite_utf8`.

Tests for this item are the failing focused matrix, render-failure, or staging
test from work items 2 through 4 plus any new regression test needed to name
the bug precisely. Do not weaken the test if it reveals a production defect.

Changed Markdown files for this work item:
`docs/execplans/roadmap-4-1-1.md`.

### Work item 6: Reconcile documentation and roadmap status

Documentation and skills to read first: `docs/documentation-style-guide.md`,
`docs/developers-guide.md` section 7, `docs/contributing.md` "Development
gates", `docs/mapsplice-design.md` sections 5, 6, and 8, `docs/users-guide.md`
"Output modes" and "Validation rules and failure cases", `docs/roadmap.md`
section 4.1.1, `execplans`, and `en-gb-oxendict-style`.

Update this ExecPlan with completed evidence, decisions, and surprises. If
implementation changed behaviour or clarified the contract, update the
relevant design or guide section. Mark `docs/roadmap.md` task 4.1.1 complete
only after the failure matrix and final gates are green.

Tests for this item are documentation gates plus the full repository gate. Use
the explicit Markdown path list below for formatting, then run `make all`,
`make markdownlint`, and `make nixie`.

Changed Markdown files for this work item:
`docs/execplans/roadmap-4-1-1.md` and `docs/roadmap.md`. If, and only if, this
work item also edits `docs/mapsplice-design.md`, `docs/developers-guide.md`,
or `docs/users-guide.md`, add that literal path to the formatter command before
running it.

## Concrete Steps

All commands run from
`/home/leynos/Projects/mapsplice.worktrees/roadmap-4-1-1`.

Before each work item:

```bash
git status --short
git branch --show-current
```

Expected status and branch:

```plaintext
roadmap-4-1-1
```

`git status --short` must print no dirty files before starting a work item. If
it prints anything, do not build a formatter list from `git diff`. Stop and
determine whether the dirty file belongs to the current work item.

For each Red-Green-Refactor loop, use focused commands first and write logs
with `tee`. Replace `<filter>` with the new test name or stable substring:

```bash
make test TEST_FLAGS='--workspace --all-targets --all-features <filter>' \
  2>&1 | tee /tmp/test-mapsplice-roadmap-4-1-1-<work-item>.out
```

For work item 1, after creating the listed fixture files, format only these
explicit paths:

```bash
mdtablefix \
  docs/execplans/roadmap-4-1-1.md \
  tests/fixtures/golden/f5_append_level_mismatch_failure/target.md \
  tests/fixtures/golden/f5_append_level_mismatch_failure/fragment.md
markdownlint-cli2 --fix \
  docs/execplans/roadmap-4-1-1.md \
  tests/fixtures/golden/f5_append_level_mismatch_failure/target.md \
  tests/fixtures/golden/f5_append_level_mismatch_failure/fragment.md
```

For work item 2, format only this explicit path:

```bash
mdtablefix docs/execplans/roadmap-4-1-1.md
markdownlint-cli2 --fix docs/execplans/roadmap-4-1-1.md
```

For work item 3, after creating every listed fixture file, format only these
explicit paths:

```bash
mdtablefix \
  docs/execplans/roadmap-4-1-1.md \
  tests/fixtures/golden/f5_append_malformed_target_in_place/target.md \
  tests/fixtures/golden/f5_append_malformed_target_in_place/fragment.md \
  tests/fixtures/golden/f5_insert_malformed_target_in_place/target.md \
  tests/fixtures/golden/f5_insert_malformed_target_in_place/fragment.md \
  tests/fixtures/golden/f5_delete_malformed_target_in_place/target.md \
  tests/fixtures/golden/f5_replace_malformed_target_in_place/target.md \
  tests/fixtures/golden/f5_replace_malformed_target_in_place/fragment.md \
  tests/fixtures/golden/f5_append_malformed_fragment_in_place/target.md \
  tests/fixtures/golden/f5_append_malformed_fragment_in_place/fragment.md \
  tests/fixtures/golden/f5_insert_malformed_fragment_in_place/target.md \
  tests/fixtures/golden/f5_insert_malformed_fragment_in_place/fragment.md \
  tests/fixtures/golden/f5_replace_malformed_fragment_in_place/target.md \
  tests/fixtures/golden/f5_replace_malformed_fragment_in_place/fragment.md \
  tests/fixtures/golden/f5_insert_level_mismatch_in_place/target.md \
  tests/fixtures/golden/f5_insert_level_mismatch_in_place/fragment.md \
  tests/fixtures/golden/f5_replace_level_mismatch_in_place/target.md \
  tests/fixtures/golden/f5_replace_level_mismatch_in_place/fragment.md \
  tests/fixtures/golden/f5_insert_missing_phase_anchor_in_place/target.md \
  tests/fixtures/golden/f5_insert_missing_phase_anchor_in_place/fragment.md \
  tests/fixtures/golden/f5_delete_missing_step_anchor_in_place/target.md \
  tests/fixtures/golden/f5_replace_missing_task_anchor_in_place/target.md \
  tests/fixtures/golden/f5_replace_missing_task_anchor_in_place/fragment.md \
  tests/fixtures/golden/f5_delete_missing_addendum_anchor_in_place/target.md \
  tests/fixtures/golden/f5_dependency_rewrite_failure_in_place/target.md
markdownlint-cli2 --fix \
  docs/execplans/roadmap-4-1-1.md \
  tests/fixtures/golden/f5_append_malformed_target_in_place/target.md \
  tests/fixtures/golden/f5_append_malformed_target_in_place/fragment.md \
  tests/fixtures/golden/f5_insert_malformed_target_in_place/target.md \
  tests/fixtures/golden/f5_insert_malformed_target_in_place/fragment.md \
  tests/fixtures/golden/f5_delete_malformed_target_in_place/target.md \
  tests/fixtures/golden/f5_replace_malformed_target_in_place/target.md \
  tests/fixtures/golden/f5_replace_malformed_target_in_place/fragment.md \
  tests/fixtures/golden/f5_append_malformed_fragment_in_place/target.md \
  tests/fixtures/golden/f5_append_malformed_fragment_in_place/fragment.md \
  tests/fixtures/golden/f5_insert_malformed_fragment_in_place/target.md \
  tests/fixtures/golden/f5_insert_malformed_fragment_in_place/fragment.md \
  tests/fixtures/golden/f5_replace_malformed_fragment_in_place/target.md \
  tests/fixtures/golden/f5_replace_malformed_fragment_in_place/fragment.md \
  tests/fixtures/golden/f5_insert_level_mismatch_in_place/target.md \
  tests/fixtures/golden/f5_insert_level_mismatch_in_place/fragment.md \
  tests/fixtures/golden/f5_replace_level_mismatch_in_place/target.md \
  tests/fixtures/golden/f5_replace_level_mismatch_in_place/fragment.md \
  tests/fixtures/golden/f5_insert_missing_phase_anchor_in_place/target.md \
  tests/fixtures/golden/f5_insert_missing_phase_anchor_in_place/fragment.md \
  tests/fixtures/golden/f5_delete_missing_step_anchor_in_place/target.md \
  tests/fixtures/golden/f5_replace_missing_task_anchor_in_place/target.md \
  tests/fixtures/golden/f5_replace_missing_task_anchor_in_place/fragment.md \
  tests/fixtures/golden/f5_delete_missing_addendum_anchor_in_place/target.md \
  tests/fixtures/golden/f5_dependency_rewrite_failure_in_place/target.md
```

For work item 4, after creating the listed fixture files, format only these
explicit paths:

```bash
mdtablefix \
  docs/execplans/roadmap-4-1-1.md \
  tests/fixtures/golden/f5_render_failure_in_place/target.md \
  tests/fixtures/golden/f5_render_failure_in_place/fragment.md
markdownlint-cli2 --fix \
  docs/execplans/roadmap-4-1-1.md \
  tests/fixtures/golden/f5_render_failure_in_place/target.md \
  tests/fixtures/golden/f5_render_failure_in_place/fragment.md
```

For work item 5, format only this explicit path:

```bash
mdtablefix docs/execplans/roadmap-4-1-1.md
markdownlint-cli2 --fix docs/execplans/roadmap-4-1-1.md
```

For work item 6, format only the explicit files edited by that work item. The
minimum command is:

```bash
mdtablefix docs/execplans/roadmap-4-1-1.md docs/roadmap.md
markdownlint-cli2 --fix docs/execplans/roadmap-4-1-1.md docs/roadmap.md
```

If work item 6 edits another documentation file, append that existing literal
path to both commands. Do not use `git diff --name-only` to discover formatter
targets.

Then run the required gates sequentially:

```bash
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-4-1-1-<work-item>.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-4-1-1-<work-item>.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-4-1-1-<work-item>.out
```

Commit each completed work item only after its gates pass.

## Validation and Acceptance

Acceptance requires all of the following:

- The new operation failure matrix proves typed errors for target grammar,
  fragment grammar, fragment-level mismatch, missing anchors, and dependency
  rewrite failure.
- The committed `f5_render_failure_in_place` golden case reaches
  `render_roadmap`, returns `MapspliceError::InvalidRoadmap`, returns no
  `RunOutcome::stdout`, and leaves the target byte-identical.
- Failure cases return `Err(MapspliceError::...)` through `run_from_args`; no
  failure case returns `RunOutcome::stdout`.
- Failure cases that run with `--in-place` leave the target file
  byte-identical to its original fixture.
- `apply_command_leaves_roadmap_unchanged_on_error` proves the caller's
  `RoadmapDocument` remains unchanged on `Err`.
- Temporary mutation checks demonstrate that the new tests fail when staging,
  validation, dependency rewrite, render, or in-place rewrite ordering is
  broken, and those mutations are reverted before any commit.
- `docs/roadmap.md` marks 4.1.1 complete only after the tests and gates pass.
- `make all`, `make markdownlint`, and `make nixie` pass on the final tree.

Quality criteria:

- Tests: focused failure-matrix and staging tests pass, followed by
  `make all`.
- Lint/typecheck: included in `make all`, which expands to formatting check,
  docs, Clippy, typecheck, `cargo check`, nextest, and doctests on this branch.
- Documentation: changed Markdown files are path-formatted from explicit file
  lists, then `make markdownlint` and `make nixie` pass.
- Security: no new dependencies, no raw filesystem writes outside the existing
  `rewrite_utf8` path, and no weakening of fail-closed error handling.

## Idempotence and Recovery

The test and documentation steps are idempotent. Re-running the focused tests
or repository gates should not change tracked files. Markdown formatters may
change only the literal paths named by the current work item.

If a temporary mutation is used for red proof, revert it immediately with a
targeted patch before continuing. Do not use `git reset --hard` or
`git checkout --` unless explicitly instructed by the user. If formatter churn
touches unrelated Markdown, park or discard it with a named stash following
the required format:

```bash
git stash push -m 'df12-stash v1 task=4.1.1 kind=discard reason="unrelated formatter churn"' -- <paths>
```

If a gate fails because of an unrelated pre-existing issue, keep the failing
log in `/tmp`, record the exact command and symptom in this ExecPlan, and
escalate only after isolating it from the work item.

## Artifacts and Notes

Planning evidence gathered in this round:

- `git branch --show-current` returned `roadmap-4-1-1`.
- `git status --short` printed no dirty files before this revision.
- `sem diff --from origin/main --to HEAD --format json` reported the existing
  ExecPlan addition on this branch.
- `make -n all` showed that `make all` includes formatting check, docs,
  Clippy, typecheck, `cargo check`, nextest, and doctests.
- `make -n markdownlint` showed `markdownlint-cli2 '**/*.md'`.
- `make -n nixie` showed `nixie --no-sandbox`.
- `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`.
- `mcp__firecrawl.firecrawl_scrape` for the `rstest` 0.26.1 docs.rs page
  returned `user cancelled MCP tool call`; direct docs.rs web fetch also
  produced no usable content in this planning session.
- `Cargo.lock` and local Cargo registry source were used to verify locked
  dependency behaviour for `rstest`, `rstest-bdd`, `tempfile`, and `markdown`.
- Work item 1 focused red log:
  `/tmp/test-mapsplice-roadmap-4-1-1-work-item-1-red.out`.
- Work item 1 focused green log:
  `/tmp/test-mapsplice-roadmap-4-1-1-work-item-1-green.out`.
- Work item 1 deterministic gate logs:
  `/tmp/make-all-mapsplice-roadmap-4-1-1-work-item-1.out`,
  `/tmp/markdownlint-mapsplice-roadmap-4-1-1-work-item-1.out`, and
  `/tmp/nixie-mapsplice-roadmap-4-1-1-work-item-1.out`.
- Work item 1 CodeRabbit attempt:
  `/tmp/coderabbit-mapsplice-roadmap-4-1-1-work-item-1.out` reported
  `deferred coderabbit review: no default network route visible in this
  sandbox`.
- Work item 2 focused logs:
  `/tmp/test-mapsplice-roadmap-4-1-1-work-item-2-green-baseline.out`,
  `/tmp/test-mapsplice-roadmap-4-1-1-work-item-2-mutation-red.out`, and
  `/tmp/test-mapsplice-roadmap-4-1-1-work-item-2-green-after-lint-fix.out`.
- Work item 2 deterministic gate log:
  `/tmp/make-all-mapsplice-roadmap-4-1-1-work-item-2.out`.
- Work item 2 CodeRabbit attempt:
  `/tmp/coderabbit-mapsplice-roadmap-4-1-1-work-item-2.out` reported
  `deferred coderabbit review: no default network route visible in this
  sandbox`.
- Work item 3 focused logs:
  `/tmp/test-mapsplice-roadmap-4-1-1-work-item-3-green.out`,
  `/tmp/test-mapsplice-roadmap-4-1-1-work-item-3-mutation-red.out`,
  `/tmp/test-mapsplice-roadmap-4-1-1-work-item-3-green-after-mutation.out`,
  and
  `/tmp/test-mapsplice-roadmap-4-1-1-work-item-3-green-after-clippy-fix.out`.
- Work item 3 deterministic gate logs:
  `/tmp/make-all-mapsplice-roadmap-4-1-1-work-item-3.out`,
  `/tmp/markdownlint-mapsplice-roadmap-4-1-1-work-item-3.out`, and
  `/tmp/nixie-mapsplice-roadmap-4-1-1-work-item-3.out`.
- Work item 3 CodeRabbit attempt:
  `/tmp/coderabbit-mapsplice-roadmap-4-1-1-work-item-3.out` reported
  `deferred coderabbit review: no default network route visible in this
  sandbox`.
- Work item 4 focused logs:
  `/tmp/test-mapsplice-roadmap-4-1-1-work-item-4-green.out`,
  `/tmp/test-mapsplice-roadmap-4-1-1-work-item-4-mutation-red.out`, and
  `/tmp/test-mapsplice-roadmap-4-1-1-work-item-4-green-after-mutation.out`.
- Work item 4 deterministic gate logs:
  `/tmp/make-all-mapsplice-roadmap-4-1-1-work-item-4.out`,
  `/tmp/markdownlint-mapsplice-roadmap-4-1-1-work-item-4.out`, and
  `/tmp/nixie-mapsplice-roadmap-4-1-1-work-item-4-retry.out`.
- Work item 4 CodeRabbit attempt:
  `/tmp/coderabbit-mapsplice-roadmap-4-1-1-work-item-4.out` reported
  `deferred coderabbit review: no default network route visible in this
  sandbox`.
- Work item 5 Leta attempt:
  `/tmp/leta-mapsplice-roadmap-4-1-1-work-item-5-run-request.out` reported
  `Error: Failed to start daemon`.
- Work item 5 source audit used bounded branch-local windows from
  `sed -n '50,82p' src/lib.rs` and
  `sed -n '80,140p' src/roadmap/ops/mod.rs`.
- Work item 5 deterministic gate logs:
  `/tmp/make-all-mapsplice-roadmap-4-1-1-work-item-5.out`,
  `/tmp/markdownlint-mapsplice-roadmap-4-1-1-work-item-5.out`, and
  `/tmp/nixie-mapsplice-roadmap-4-1-1-work-item-5.out`.
- Work item 5 CodeRabbit attempt:
  `/tmp/coderabbit-mapsplice-roadmap-4-1-1-work-item-5.out` reported
  `deferred coderabbit review: no default network route visible in this
  sandbox`.
- Work item 6 deterministic gate logs:
  `/tmp/make-all-mapsplice-roadmap-4-1-1-work-item-6.out`,
  `/tmp/markdownlint-mapsplice-roadmap-4-1-1-work-item-6.out`, and
  `/tmp/nixie-mapsplice-roadmap-4-1-1-work-item-6-retry.out`.
- Work item 6 first `make nixie` attempt:
  `/tmp/nixie-mapsplice-roadmap-4-1-1-work-item-6.out` timed out while
  validating the pre-existing `docs/rstest-bdd-users-guide.md` Mermaid
  diagram; the immediate retry passed.
- Work item 6 CodeRabbit attempt:
  `/tmp/coderabbit-mapsplice-roadmap-4-1-1-work-item-6.out` reported
  `deferred coderabbit review: no default network route visible in this
  sandbox`.
- Fix round 1 pre-rebase status:
  `git status --short --branch` reported
  `## roadmap-4-1-1...origin/main [ahead 9, behind 2]`.
- Fix round 1 integration command:
  `git rebase origin/main` completed successfully with no conflicts.
- Fix round 1 post-rebase status:
  `git status --short --branch` reported
  `## roadmap-4-1-1...origin/main [ahead 9]`.
- Fix round 1 branch diff:
  `git diff --name-status origin/main..HEAD` no longer lists deletions for
  `docs/execplans/roadmap-3-1-3.md`, `docs/issues/audit-3.1.3.md`, or
  `tests/golden/format_gate.rs`.
- Fix round 1 format-gate verification:
  `tests/golden/mod.rs` contains `mod format_gate;` and
  `pub(crate) use format_gate::assert_gate_clean_rendered_output;`.
- Fix round 1 deterministic gate logs:
  `/tmp/make-all-mapsplice-roadmap-4-1-1-fix-round-1.out`,
  `/tmp/markdownlint-mapsplice-roadmap-4-1-1-fix-round-1.out`, and
  `/tmp/nixie-mapsplice-roadmap-4-1-1-fix-round-1.out`.
- Fix round 1 deterministic gate result:
  `make all` passed with 152 nextest tests and 7 doctests passing, including
  `golden::format_gate::tests::gate_clean_rendered_output_accepts_stable_markdown`
  and
  `golden::format_gate::tests::gate_clean_rendered_output_rejects_formatter_drift`;
  `make markdownlint` passed with 0 errors; `make nixie` validated all
  diagrams successfully.
- Fix round 1 CodeRabbit attempt:
  `/tmp/coderabbit-mapsplice-roadmap-4-1-1-fix-round-1.out` reported
  `deferred coderabbit review: no default network route visible in this
  sandbox`. This remains an open issue for the supervisor to relaunch or
  fallback-review when network access is available.
- Fix round 1 ExecPlan update gate logs:
  `/tmp/make-all-mapsplice-roadmap-4-1-1-fix-round-1-doc-update.out`,
  `/tmp/markdownlint-mapsplice-roadmap-4-1-1-fix-round-1-doc-update.out`, and
  `/tmp/nixie-mapsplice-roadmap-4-1-1-fix-round-1-doc-update.out`.
- Fix round 1 ExecPlan update CodeRabbit attempt:
  `/tmp/coderabbit-mapsplice-roadmap-4-1-1-fix-round-1-doc-update.out`
  reported
  `deferred coderabbit review: no default network route visible in this
  sandbox`.

## Interfaces and Dependencies

No public interface changes are planned.

The work must keep using these existing project interfaces:

```rust
pub fn run_from_args<I, T>(args: I) -> Result<RunOutcome>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone;

pub fn run_request(request: CliRequest) -> Result<RunOutcome>;

pub fn apply_command(
    roadmap: &mut RoadmapDocument,
    operation: RoadmapOperation,
    fragment: Option<RoadmapFragment>,
) -> Result<u64>;
```

The work must keep using the existing typed error enum in `src/error.rs`,
including `InvalidRoadmap`, `Markdown`, `LevelMismatch`,
`AppendLevelMismatch`, `AnchorNotFound`, `DanglingDependency`, and `Io`.

No new external libraries are allowed. Use the locked versions already in
`Cargo.lock`, especially `rstest` for fixtures and parameterized tests,
`serial_test` for serialized CLI-environment tests when needed, `tempfile` for
temporary workspaces, and `cap-std` / `camino` for capability-oriented
filesystem helpers.

## Revision Note

Round-3 revision for roadmap task 4.1.1. This revision resolves the remaining
design-review blocker by adding a separate durable
`f5_render_failure_in_place` golden case that reaches `render_roadmap`, returns
typed `MapspliceError::InvalidRoadmap`, emits no `RunOutcome::stdout`, and
leaves the in-place target byte-identical. It also updates the validation,
acceptance, and path-safe formatter commands so render failure is not proven
only by a temporary mutation.

Work item 1 implementation revision: the failure assertion vocabulary now
distinguishes append-specific level mismatches from anchor-level mismatches,
and the committed append mismatch fixture proves `--in-place` leaves the target
unchanged on that typed failure.

Work item 2 implementation revision: `apply_command` now has a direct
caller-state regression test for errors that occur after staged mutation and
before dependency rewrite completion.

Work item 3 implementation revision: the in-place golden matrix now exercises
target parse, fragment parse, fragment level, missing anchor, and
dependency-rewrite failures with `FailureOutput::InPlaceTargetUnchanged`. The
matrix is intentionally shared-boundary coverage rather than a cartesian
product, because the operations use the same parse, level validation, anchor
lookup, staging, render, and rewrite sequence documented above.

Work item 4 implementation revision: the committed render-failure fixture
uses an unsupported inline image in a task summary so parsing and append
application succeed but rendering fails. This keeps the render-failure branch
protected by a durable golden test rather than by mutation proof alone.

Work item 5 implementation revision: no production implementation gap was
found. The audit commit records the source-window evidence and leaves the
runtime code unchanged.

Work item 6 implementation revision: `docs/roadmap.md` now marks 4.1.1
complete, and this ExecPlan is closed with the final gate evidence and
deferred CodeRabbit review status. No behaviour documentation changed because
the implemented audit confirmed the existing fail-closed contract.

Fix round 1 revision: the branch was rebased onto `origin/main` to preserve
the already-landed 3.1.3 rendered-output gate work. The fix restores the
main-branch `format_gate` module and roadmap status relative to this branch,
then records green `make all`, `make markdownlint`, and `make nixie` evidence.
CodeRabbit review is deferred because the sandbox has no default network
route.
