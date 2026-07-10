# Report Unresolved Dependency References

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

## Purpose / Big Picture

Roadmap task 4.1.2, "Report unresolved dependency references", closes a
fail-closed gap in the dependency-reference model. After this change, a valid
`Requires` dependency token such as `99.1.1` that does not resolve to a known
roadmap item after a splice is reported as a typed diagnostic instead of being
preserved silently. Invalid version-like text such as `1.4.0`, section
references such as `§2.1`, and incidental prose numbers remain preserved.

The observable outcome is that an edit containing a dangling `Requires`
reference fails with `MapspliceError::DanglingDependency`, the command reports
the dangling dependency anchor on standard error, standard output is empty, and
`--in-place` leaves the target file unchanged.

## Constraints

- Work only inside `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-1-2`.
- Do not edit the root/control worktree.
- Treat `origin/main` as the integration branch and `docs/roadmap.md` as the
  roadmap source of truth.
- Keep changes small enough that every work item is independently
  committable, gate-passable, and reviewable.
- Preserve the constrained roadmap grammar from `docs/users-guide.md` and
  `docs/mapsplice-design.md`.
- Preserve scoped rewriting from `docs/mapsplice-design.md` sections 6 and 7:
  non-`Requires` numbers, section references, version-like invalid tokens, and
  future `Blocks` clauses remain out of scope for this task.
- Keep public library errors typed. Public APIs must continue to return
  `MapspliceError`, per `docs/developers-guide.md` section 3 and `AGENTS.md`
  "Error Handling".
- Follow en-GB Oxford spelling in prose, comments, and commit messages.
- Use Red-Green-Refactor inside each work item. The final commit for each work
  item must be green; do not commit a red intermediate state.
- Format only changed Markdown files. Do not run repo-global Markdown
  formatters such as `make fmt` or `mdformat-all`.
- Run commands sequentially, with `tee` to `/tmp`; do not run tests, lint, or
  formatting checks in parallel.
- Use the shared Cargo cache. Do not create an isolated Cargo cache.

## Tolerances

- Stop and escalate if reporting dangling references requires a new external
  dependency, a public API signature change other than using the existing
  `MapspliceError::DanglingDependency` variant, or a change to the accepted
  roadmap grammar.
- The planned implementation may touch at most these twelve source, test, and
  documentation files:
  `docs/execplans/roadmap-4-1-2.md`, `docs/mapsplice-design.md`,
  `docs/roadmap.md`, `docs/users-guide.md`,
  `src/roadmap/ops/dependency_text.rs`, `src/roadmap/ops/rewrite.rs`,
  `tests/behaviour_cli.rs`, `tests/features/mapsplice.feature`,
  `tests/roadmap_ops.rs`, `tests/roadmap_properties.rs`,
  `tests/steps/cli_steps.rs`, and `tests/support/cli.rs`.
- Stop and escalate if implementation needs any thirteenth source, test, or
  documentation file, any file outside that planned surface, or more than 260
  net lines. This tolerance intentionally includes the files enumerated in the
  work items below; it is not an additional eight-file cap.
- Stop and escalate if preserving invalid or version-like tokens conflicts with
  reporting valid unresolved `Requires` anchors.
- Stop and escalate if a focused test still fails for the same reason after
  three implementation attempts.
- Stop and escalate if `make all` fails for an unrelated pre-existing failure
  that cannot be isolated with a focused command and a log.

## Risks

- Risk: `docs/mapsplice-design.md` section 7 currently says an unresolved
  dependency reference is left unchanged, while roadmap task 4.1.2 requires it
  to be reported.
  Severity: medium.
  Likelihood: high.
  Mitigation: update the design and users' guide in work item 4 so valid
  unresolved `Requires` references become a fail-closed diagnostic, while
  invalid tokens and incidental numbers remain preserved.

- Risk: making `dependency_text::rewrite_text_value` return an error directly
  could make low-level text scanning own operation-level policy.
  Severity: medium.
  Likelihood: medium.
  Mitigation: use a small internal report type that contains the rewritten
  value, rewrite count, and unresolved valid anchors; let
  `rewrite_dependencies` convert the first unresolved anchor into
  `MapspliceError::DanglingDependency`.

- Risk: a dangling reference in `--in-place` mode might be detected too late,
  after the file is rewritten.
  Severity: high.
  Likelihood: low.
  Mitigation: keep the existing staged clone flow in
  `src/roadmap/ops/mod.rs::apply_command`; detect dangling dependencies before
  `*roadmap = staged`, rendering, and filesystem writes.

- Risk: property-test behaviour could be misdescribed if the plan cites the
  caret floor in `Cargo.toml` instead of the locked crate.
  Severity: medium.
  Likelihood: medium.
  Mitigation: pin all proptest claims to locked `proptest` 1.11.0 in
  `Cargo.lock` and the local cargo registry source, and let the new project
  property test be the behavioural contract.

## Progress

- [x] (2026-07-01T15:18:33Z) Read `AGENTS.md`,
  `docs/mapsplice-design.md`, `docs/developers-guide.md`,
  `docs/users-guide.md`, `docs/contributing.md`,
  `docs/documentation-style-guide.md`, and the roadmap entry for 4.1.2.
- [x] (2026-07-01T15:18:33Z) Checked branch-local source and tests for
  dependency rewriting, error reporting, behavioural scenarios, and property
  tests.
- [x] (2026-07-01T15:18:33Z) Verified locked crate behaviour from local Cargo
  registry source after Firecrawl was unavailable.
- [x] (2026-07-01T16:06:00Z) Revised the draft after design review round 2:
  split internal reporting from API propagation, made the BDD failure
  in-place, fixed the target-unchanged assertion plan, and corrected the
  locked proptest version.
- [x] (2026-07-01T18:42:00Z) Revised the draft after design review round 4:
  replaced the contradictory eight-file tolerance with an explicit planned
  twelve-file surface and an escalation trigger for unplanned expansion.
- [x] (2026-07-01T16:02:15Z) Draft approved for implementation by the
  df12-build roadmap workflow.
- [x] (2026-07-01T16:17:23Z) Work item 1 complete: introduce internal
  dangling-reference reporting.
- [x] (2026-07-01T16:29:26Z) Work item 2 complete: propagate dangling
  references through the library operation boundary.
- [x] (2026-07-01T16:48:51Z) Work item 3 complete: add CLI behavioural
  in-place failure coverage.
- [x] (2026-07-01T17:01:53Z) Work item 4 complete: add generated coverage and
  reconcile documentation.
- [x] (2026-07-01T16:02:15Z) Work item 1 Red: focused
  `make test TEST_FLAGS='--workspace --all-targets --all-features
  dependency_reference'` failed as expected because the new unit tests read
  `DependencyRewriteReport` fields while `rewrite_text_value` still returned
  `(String, u64)`. Evidence:
  `/tmp/test-mapsplice-roadmap-4-1-2-4-1-2-item-1-red.out`.
- [x] (2026-07-01T16:02:15Z) Work item 1 Green: added
  `DependencyRewriteReport`, collected unresolved valid `Requires` anchors in
  `dependency_text`, and kept `rewrite.rs` discarding unresolved anchors so the
  public operation boundary still preserves them in this item. The focused
  dependency-reference command passed with 22 tests and 10 doctests. Evidence:
  `/tmp/test-mapsplice-roadmap-4-1-2-4-1-2-item-1.out`.
- [x] (2026-07-01T16:17:23Z) Work item 1 deterministic gates: after fixing
  Rust import formatting, scrutineer reported `make all`, `make markdownlint`,
  and `make nixie` green. Evidence:
  `/tmp/make-all-mapsplice-roadmap-4-1-2-4-1-2-item-1-rerun2.out`,
  `/tmp/markdownlint-mapsplice-roadmap-4-1-2-4-1-2-item-1-rerun2.out`, and
  `/tmp/nixie-mapsplice-roadmap-4-1-2-4-1-2-item-1-rerun2.out`.
- [x] (2026-07-01T16:17:23Z) Work item 1 CodeRabbit review was attempted by
  scrutineer but deferred because the command did not progress beyond service
  connection. Evidence: `/tmp/coderabbit-mapsplice-roadmap-4-1-2-item-1.out`
  contains `connecting_to_review_service`; scrutineer reported exit status
  `130` after interruption and no review payload.
- [x] (2026-07-01T16:20:30Z) Work item 2 Red: focused
  `make test TEST_FLAGS='--workspace --all-targets --all-features
  unresolved_dependency_reference_is_reported'` failed as expected because the
  append command still succeeded and preserved `Requires 99.1.1.`. Evidence:
  `/tmp/test-mapsplice-roadmap-4-1-2-4-1-2-item-2-red.out`.
- [x] (2026-07-01T16:20:30Z) Work item 2 Green: `rewrite_dependencies`
  aggregates unresolved valid dependency anchors during Markdown traversal and
  returns `MapspliceError::DanglingDependency` for the first deterministic
  unresolved anchor after traversal. The focused public API test passed, and
  the dependency-reference suite still passed with 22 tests and 10 doctests.
  Evidence: `/tmp/test-mapsplice-roadmap-4-1-2-4-1-2-item-2.out` and
  `/tmp/test-mapsplice-roadmap-4-1-2-4-1-2-item-2-dependency.out`.
- [x] (2026-07-01T16:23:39Z) Work item 2 Refactor: replaced the extra
  recursive traversal parameter with `DependencyRewriteContext` after Clippy
  reported `too_many_arguments`. The focused public API test remained green.
  Evidence:
  `/tmp/test-mapsplice-roadmap-4-1-2-4-1-2-item-2-rerun.out`.
- [x] (2026-07-01T16:29:26Z) Work item 2 deterministic gates: after replacing
  the wrong-variant `panic!` path with an error return, scrutineer reported
  `make all`, `make markdownlint`, and `make nixie` green. Evidence:
  `/tmp/make-all-mapsplice-roadmap-4-1-2-4-1-2-item-2-rerun2.out`,
  `/tmp/markdownlint-mapsplice-roadmap-4-1-2-4-1-2-item-2-rerun2.out`, and
  `/tmp/nixie-mapsplice-roadmap-4-1-2-4-1-2-item-2-rerun2.out`.
- [x] (2026-07-01T16:29:26Z) Work item 2 CodeRabbit review was attempted by
  scrutineer but deferred because the command again did not progress beyond
  service connection. Evidence:
  `/tmp/coderabbit-mapsplice-roadmap-4-1-2-item-2.out` contains
  `connecting_to_review_service`; scrutineer reported exit status `130` after
  interruption and no review payload.
- [x] (2026-07-01T16:32:44Z) Work item 3 Red: focused
  `make test TEST_FLAGS='--workspace --all-targets --all-features
  dangling_dependency'` failed as expected because the new BDD scenario had no
  step for `Given the target roadmap with a dangling dependency reference`.
  Evidence: `/tmp/test-mapsplice-roadmap-4-1-2-4-1-2-item-3-red.out`.
- [x] (2026-07-01T16:32:44Z) Work item 3 Green: added the dangling dependency
  CLI fixture, in-place append action, stderr assertion, and dynamic
  original-target tracking for unchanged-file checks. The focused
  dangling-dependency BDD test passed, and `tests/steps/cli_steps.rs` remained
  below the 400-line cap at 316 lines. Evidence:
  `/tmp/test-mapsplice-roadmap-4-1-2-4-1-2-item-3.out`.
- [x] (2026-07-01T16:48:51Z) Work item 3 deterministic gates: after replacing
  the `to_owned()` assignment with `clone_into`, scrutineer reported
  `make all` and `make markdownlint` green. `make nixie` timed out twice on
  an untouched `docs/ortho-config-users-guide.md` Mermaid diagram, then passed
  unchanged on the next retry. Evidence:
  `/tmp/make-all-mapsplice-roadmap-4-1-2-4-1-2-item-3-rerun.out`,
  `/tmp/markdownlint-mapsplice-roadmap-4-1-2-4-1-2-item-3-rerun.out`, and
  `/tmp/nixie-mapsplice-roadmap-4-1-2-4-1-2-item-3-rerun-nixie2.out`.
- [x] (2026-07-01T16:48:51Z) Work item 3 CodeRabbit review was attempted by
  scrutineer but deferred because the command again did not progress beyond
  service connection. Evidence:
  `/tmp/coderabbit-mapsplice-roadmap-4-1-2-item-3.out` contains
  `connecting_to_review_service`; scrutineer reported exit status `130` after
  interruption and no review payload.
- [x] (2026-07-01T16:51:45Z) Work item 4 generated coverage: added a
  property that constructs valid positive anchors outside the generated
  two-phase roadmap and asserts `run_from_args` returns
  `MapspliceError::DanglingDependency`. The focused `dangling` command passed
  immediately because work items 2 and 3 already implemented the behaviour.
  Evidence: `/tmp/test-mapsplice-roadmap-4-1-2-4-1-2-item-4.out`.
- [x] (2026-07-01T16:51:45Z) Work item 4 documentation reconciliation:
  updated the design, users' guide, and roadmap so unresolved valid `Requires`
  references are documented as fail-closed diagnostics and 4.1.2 is ticked.
- [x] (2026-07-01T17:01:53Z) Work item 4 deterministic gates: scrutineer
  reported `make all` and `make markdownlint` green. `make nixie` timed out
  once on an untouched `docs/rstest-bdd-users-guide.md` Mermaid diagram, then
  passed unchanged on retry. Evidence:
  `/tmp/make-all-mapsplice-roadmap-4-1-2-4-1-2-item-4.out`,
  `/tmp/markdownlint-mapsplice-roadmap-4-1-2-4-1-2-item-4.out`, and
  `/tmp/nixie-mapsplice-roadmap-4-1-2-4-1-2-item-4-rerun.out`.
- [x] (2026-07-01T17:01:53Z) Work item 4 CodeRabbit review was attempted by
  scrutineer but deferred because the command again did not progress beyond
  service connection. Evidence:
  `/tmp/coderabbit-mapsplice-roadmap-4-1-2-item-4.out` contains
  `connecting_to_review_service`; scrutineer reported exit status `130` after
  interruption and no review payload.

## Surprises & Discoveries

- Observation: `src/error.rs` already contains
  `MapspliceError::DanglingDependency` with class `dangling_dependency`, and
  `src/roadmap/ops/mod.rs::apply_command` already stages edits on a clone.
  Evidence: branch-local inspection found the variant in `src/error.rs` and
  `apply_command` assigns `*roadmap = staged` only after
  `rewrite_dependencies` succeeds.
  Impact: implementation should activate the existing diagnostic instead of
  introducing a second error type or changing the public API shape.

- Observation: current branch-local tests still preserve unresolved valid
  dependency references.
  Evidence: `tests/roadmap_ops.rs::unresolved_dependency_reference_is_preserved`
  expects `Requires 99.1.1.` to survive an append; unit coverage in
  `src/roadmap/ops/dependency_text.rs` preserves the same case.
  Impact: implementation must replace the preservation expectation with
  diagnostic expectations at the correct layers.

- Observation: `tests/steps/cli_steps.rs::target_unchanged` compares only with
  `TARGET_TWO_PHASES`, but a dangling scenario needs a different target text.
  Evidence: branch-local inspection showed `target_unchanged` hard-codes
  `TARGET_TWO_PHASES`, while a dangling fixture must contain
  `Requires 99.1.1.`.
  Impact: work item 3 must make `CliState` remember the original target text
  on every `write_target`, so unchanged assertions compare against the actual
  fixture used by the scenario.

- Observation: advisory tools were unavailable in this planning session.
  Evidence: Memtrace `list_indexed_repositories` returned
  `user cancelled MCP tool call`; `leta workspace add
  /home/leynos/Projects/mapsplice.worktrees/roadmap-4-1-2` returned
  `Error: IO error: Read-only file system (os error 30)`; a retry with
  scratch state under `/tmp/mapsplice-leta-home` returned the same read-only
  filesystem error; Firecrawl `firecrawl_scrape` for the official docs pages
  for `proptest` 1.11.0, `thiserror` 2.0.18, `markdown` 1.0.0, and
  `rstest-bdd` 0.5.0 returned `user cancelled MCP tool call`.
  Impact: this plan uses bounded branch-local source inspection and Cargo
  registry source as fallback evidence. This is not a blocker.

- Observation: Memtrace repository discovery remained unavailable in the
  implementation session.
  Evidence: `list_indexed_repositories` returned `user cancelled MCP tool
  call` before work item 1 edits began.
  Impact: implementation continues with bounded branch-local Leta and file
  evidence, as permitted by this plan.

- Observation: Leta was partially available in the implementation session.
  Evidence: `leta workspace add
  /home/leynos/Projects/mapsplice.worktrees/roadmap-4-1-2` returned
  `Workspace already added`, and `leta show rewrite_text_value`,
  `leta show rewrite_dependencies`, `leta show rewrite_node`, and `leta show
  apply_command` succeeded. A later `leta show MapspliceError && leta refs
  DanglingDependency -n 2` command returned `Error: Failed to start daemon`.
  Impact: symbol navigation was used for the rewrite path; bounded file
  inspection was used only for the missed error enum context.

- Observation: CodeRabbit review was unavailable for work item 1.
  Evidence: `coderabbit review --agent` produced only
  `connecting_to_review_service` in
  `/tmp/coderabbit-mapsplice-roadmap-4-1-2-item-1.out`; scrutineer reported
  exit status `130` after interrupting the non-terminating run.
  Impact: this is recorded as a deferred-review open issue. Deterministic
  gates were green before the attempt, and no actionable review payload was
  returned.

- Observation: CodeRabbit review was unavailable for work item 2.
  Evidence: `coderabbit review --agent` again produced only
  `connecting_to_review_service` in
  `/tmp/coderabbit-mapsplice-roadmap-4-1-2-item-2.out`; scrutineer reported
  exit status `130` after interrupting the non-terminating run.
  Impact: this is recorded as a deferred-review open issue. Deterministic
  gates were green before the attempt, and no actionable review payload was
  returned.

- Observation: CodeRabbit review was unavailable for work item 3.
  Evidence: `coderabbit review --agent` again produced only
  `connecting_to_review_service` in
  `/tmp/coderabbit-mapsplice-roadmap-4-1-2-item-3.out`; scrutineer reported
  exit status `130` after interrupting the non-terminating run.
  Impact: this is recorded as a deferred-review open issue. Deterministic
  gates were green before the attempt, and no actionable review payload was
  returned.

- Observation: CodeRabbit review was unavailable for work item 4.
  Evidence: `coderabbit review --agent` again produced only
  `connecting_to_review_service` in
  `/tmp/coderabbit-mapsplice-roadmap-4-1-2-item-4.out`; scrutineer reported
  exit status `130` after interrupting the non-terminating run.
  Impact: this is recorded as a deferred-review open issue. Deterministic
  gates were green before the attempt, and no actionable review payload was
  returned.

- Observation: `make nixie` intermittently times out on untouched guide
  diagrams.
  Evidence: during work items 2, 3, and 4, `nixie` timeouts alternated between
  `docs/rstest-bdd-users-guide.md` and `docs/ortho-config-users-guide.md`, but
  retries passed without file changes.
  Impact: untouched diagram timeouts are treated as transient gate noise when
  an immediate retry passes and no planned file needs to change.

## Decision Log

- Decision: Treat roadmap task 4.1.2 as the newer contract for valid
  unresolved `Requires` anchors, and update design and user documentation
  during this task.
  Rationale: `docs/roadmap.md` explicitly requires surfacing dangling
  dependencies; keeping the older "left unchanged" sentence would make the
  implementation and documentation disagree.
  Date/Author: 2026-07-01T15:18:33Z / planning agent.

- Decision: Keep `Blocks` out of scope.
  Rationale: `docs/mapsplice-design.md` section 7 says only `Requires` is a
  dependency context today and `Blocks` is a future grammar extension; current
  tests assert `Blocks 2.1.1.` is ignored.
  Date/Author: 2026-07-01T15:18:33Z / planning agent.

- Decision: Split internal text reporting from library-boundary propagation.
  Rationale: work item 1 must be atomic and gate-passable by testing only the
  private/internal rewrite report; work item 2 then adds the API-level
  `MapspliceError::DanglingDependency` diagnostic and propagation through
  `rewrite_dependencies` and `apply_command`.
  Date/Author: 2026-07-01T16:06:00Z / planning agent.

- Decision: The BDD diagnostic must run the dangling command with
  `--in-place`.
  Rationale: the user-visible contract includes write safety, and a
  stdout-mode failure alone does not prove the target file is protected.
  Date/Author: 2026-07-01T16:06:00Z / planning agent.

- Decision: Pin proptest research to locked `proptest` 1.11.0, not the
  `Cargo.toml` caret floor `1.9.0`.
  Rationale: `Cargo.lock` resolves `proptest` 1.11.0, and load-bearing
  behaviour must match the actual locked source used by tests.
  Date/Author: 2026-07-01T16:06:00Z / planning agent.

- Decision: Set the autonomous scope tolerance to the explicit planned
  twelve-file surface rather than an eight-file cap.
  Rationale: the work items already require twelve named files to cover unit,
  library, CLI behavioural, property, design, user-guide, roadmap, and living
  ExecPlan evidence. An eight-file tolerance would force an implementer either
  to violate the plan or stop before completing the required coverage. The new
  tolerance preserves bounded scope by escalating on any thirteenth file or
  unplanned file.
  Date/Author: 2026-07-01T18:42:00Z / planning agent.

## Outcomes & Retrospective

Roadmap task 4.1.2 is implemented across four gate-passable commits:
`7f5a343 Report internal dangling references`, `1310063 Report dangling
dependencies`, `5ee826b Cover dangling CLI failures`, and the final work item
4 commit. Valid unresolved `Requires` anchors now surface as
`MapspliceError::DanglingDependency`; the CLI reports the dangling anchor on
standard error, emits no standard output on failure, and preserves the target
in `--in-place` mode.

The property suite now generates valid positive dangling `Requires` anchors
and asserts the typed error. Existing coverage still preserves invalid tokens,
incidental numeric text, section references, semantic versions, and ignored
`Blocks` clauses. The design, users' guide, and roadmap now agree that valid
unresolved dependency references fail closed.

Deterministic gates were green for each work item before commit. CodeRabbit
was attempted once per work item after deterministic gates were green, but all
four runs stalled at `connecting_to_review_service` and were interrupted with
exit status `130`; no actionable review payload was returned.

## Context and Orientation

The relevant pipeline is:

1. `src/lib.rs::run_request` parses command-line input, parses the target and
   optional fragment, calls `apply_command`, renders the roadmap, then writes
   standard output or performs an in-place rewrite.
2. `src/roadmap/ops/mod.rs::apply_command` clones the input roadmap into
   `staged`, applies append, insert, delete, or replace to that clone,
   renumbers it with `renumber_document`, rewrites dependency text with
   `rewrite_dependencies`, and assigns `staged` back only on success.
3. `src/roadmap/ops/rewrite.rs::rewrite_dependencies` walks all
   `MarkdownNodes`, calls `dependency_text::rewrite_text_value` for every
   `markdown::mdast::Node::Text`, clears preserved source snippets when a
   rewrite occurred, and returns a rewrite count.
4. `src/roadmap/ops/dependency_text.rs` classifies anchor-shaped text. It
   distinguishes `Reference`, `InvalidDependencyToken`, and
   `NotDependencyReference`. A `Reference` is a valid `Requires` anchor token;
   today, if the renumber plan cannot resolve it, the token is preserved.
5. `src/error.rs` already provides `MapspliceError::DanglingDependency {
   anchor: RoadmapAnchor }` and the diagnostic class `dangling_dependency`.

Important terms:

- A dependency reference is a valid roadmap anchor token inside a supported
  dependency context, currently `Requires`.
- A dangling dependency is a dependency reference that is valid anchor syntax
  but does not resolve to any item in the renumber plan after the edit.
- An invalid dependency token is text in a `Requires` clause that looks
  number-like but is not a canonical positive roadmap anchor, for example
  `1.4.0`. This task must preserve such text.

## Research Notes

- `thiserror` 2.0.18 supports deriving `std::error::Error`, generating
  `Display` from `#[error("...")]`, and interpolating named fields such as
  `{anchor}`. Verified in the locked source at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-2.0.18/src/lib.rs`,
  especially lines 38-56. Official docs retrieval via Firecrawl was attempted
  for `https://docs.rs/thiserror/2.0.18/thiserror/` and returned
  `user cancelled MCP tool call`; the same result was observed for official
  docs pages for `markdown` 1.0.0 and `rstest-bdd` 0.5.0.
- `rstest-bdd` 0.5.0 and `rstest-bdd-macros` 0.5.0 support the existing
  `#[scenario]`, `#[given]`, `#[when]`, and `#[then]` wiring used by
  `tests/behaviour_cli.rs` and `tests/steps/cli_steps.rs`. Verified in locked
  sources
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rstest-bdd-0.5.0/src/lib.rs`
  and
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rstest-bdd-macros-0.5.0/src/lib.rs`.
- `Cargo.lock` resolves `proptest` 1.11.0 while `Cargo.toml` declares the caret
  floor `proptest = "1.9.0"`. Locked `proptest` 1.11.0 exposes `pub mod
  prelude`, the `proptest!` macro, `prop_assert!`, `prop_assert_eq!`,
  `Config::with_cases`, default `cases: 256`, `PROPTEST_CASES`, and default
  failure persistence via `FileFailurePersistence::SourceParallel(
  "proptest-regressions")`. Verified in
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/lib.rs`,
  `src/prelude.rs`, `src/sugar.rs`, `src/test_runner/config.rs`, and
  `src/test_runner/failure_persistence/file.rs`. Official docs retrieval via
  Firecrawl was attempted and unavailable, so the new project property test is
  the behavioural pin for this task.
- `markdown` 1.0.0 exposes `markdown::mdast::Node::Text(Text)`, and `Text` has
  a public mutable `value: String`, which is the current rewrite surface.
  Verified in locked source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/mdast.rs`.
- `Makefile` target `all` runs `check-fmt`, `lint`, `typecheck`, and `test`.
  `make markdownlint` runs `markdownlint-cli2 '**/*.md'`; `make nixie` runs
  `nixie --no-sandbox`.

## Plan of Work

### Work Item 1: Introduce Internal Dangling-Reference Reporting

Docs to read before starting:
`docs/mapsplice-design.md` sections 5, 6, 7, and 8;
`docs/developers-guide.md` sections 3 and 6; `AGENTS.md` "Rust Specific
Guidance", "Testing", and "Error Handling".

Skills to load before starting: `leta`, `rust-router`, `rust-errors`,
`rust-unit-testing`, and `en-gb-oxendict-style`. If source-code search or
graph tools are available, start with Memtrace `list_indexed_repositories`,
`find_code`, `find_symbol`, `get_symbol_context`, and `get_impact` for
`rewrite_text_value`, `rewrite_dependencies`, and `MapspliceError`.

Implement Red-Green-Refactor in one gate-passable commit:

1. Red: in `src/roadmap/ops/dependency_text.rs`, replace the unit expectation
   named
   `dependency_reference_preserves_unresolved_valid_reference` with an
   internal-report expectation. The red test calls `rewrite_text_value` with
   `Requires 99.1.1.` and an empty `RenumberPlan`, and expects:
   the returned value is still `Requires 99.1.1.`, `rewrite_count == 0`, and
   the report contains exactly the parsed anchor `99.1.1`.
2. Red: keep or add unit coverage proving `Requires 1.4.0`, section
   references, prose numbers, and `Blocks 2.1.1.` are not returned as
   unresolved references.
3. Green: change `rewrite_text_value` to return a small internal report with
   the rewritten text, rewrite count, and unresolved valid anchors. Do not
   return `MapspliceError` from this low-level scanner.
4. Refactor: keep the report type private to `dependency_text` unless
   `rewrite.rs` needs field access. Prefer a struct such as:

```rust
pub(super) struct DependencyRewriteReport {
    pub(super) value: String,
    pub(super) rewrite_count: u64,
    pub(super) unresolved: Vec<RoadmapAnchor>,
}
```

Focused tests for this item:

```bash
branch=$(git branch --show-current)
safe_branch=${branch//\//-}
make test TEST_FLAGS='--workspace --all-targets --all-features dependency_reference' \
  2>&1 | tee "/tmp/test-mapsplice-${safe_branch}-4-1-2-item-1.out"
test "${PIPESTATUS[0]}" -eq 0
```

Before committing this work item, update this ExecPlan with the Red, Green,
Refactor, and commit evidence, then run the exact Markdown handling required
because `docs/execplans/roadmap-4-1-2.md` changed:

```bash
branch=$(git branch --show-current)
safe_branch=${branch//\//-}

mdtablefix docs/execplans/roadmap-4-1-2.md \
  2>&1 | tee "/tmp/mdtablefix-mapsplice-${safe_branch}-4-1-2-item-1-plan.out"
test "${PIPESTATUS[0]}" -eq 0

markdownlint-cli2 --fix docs/execplans/roadmap-4-1-2.md \
  2>&1 | tee "/tmp/markdownlint-fix-mapsplice-${safe_branch}-4-1-2-item-1-plan.out"
test "${PIPESTATUS[0]}" -eq 0
```

Then run the full pre-commit gates for this code/test commit:

```bash
branch=$(git branch --show-current)
safe_branch=${branch//\//-}

make all 2>&1 | tee "/tmp/make-all-mapsplice-${safe_branch}-4-1-2-item-1.out"
test "${PIPESTATUS[0]}" -eq 0

make markdownlint 2>&1 | tee "/tmp/markdownlint-mapsplice-${safe_branch}-4-1-2-item-1.out"
test "${PIPESTATUS[0]}" -eq 0

make nixie 2>&1 | tee "/tmp/nixie-mapsplice-${safe_branch}-4-1-2-item-1.out"
test "${PIPESTATUS[0]}" -eq 0
```

Acceptance for this item:

- The internal scanner reports valid unresolved `Requires 99.1.1` anchors.
- Invalid/version-like `Requires 1.4.0` unit coverage still proves
  preservation and no unresolved report entry.
- The public API-level preservation test is not changed in this item.

### Work Item 2: Propagate Dangling References Through the Library Boundary

Docs to read before starting:
`docs/mapsplice-design.md` sections 5, 6, and 7;
`docs/developers-guide.md` sections 2, 3, and 6; `AGENTS.md` "Error Handling".

Skills to load before starting: `leta`, `rust-router`, `rust-errors`,
`rust-unit-testing`, and `en-gb-oxendict-style`. Use `sem diff` before commit
to review the entity-level change.

Implement Red-Green-Refactor in one gate-passable commit:

1. Red: replace
   `tests/roadmap_ops.rs::unresolved_dependency_reference_is_preserved` with
   `unresolved_dependency_reference_is_reported`. The test writes a roadmap
   containing `Requires 99.1.1.`, runs an append that reaches dependency
   rewriting, and expects `MapspliceError::DanglingDependency { anchor }` where
   `anchor.to_string() == "99.1.1"`.
2. Green: update `src/roadmap/ops/rewrite.rs::rewrite_node`,
   `rewrite_nodes`, `rewrite_markdown_nodes`, and `rewrite_dependencies` to
   aggregate unresolved anchors from the internal report. Convert the first
   deterministic unresolved anchor into
   `MapspliceError::DanglingDependency { anchor }`.
3. Green: keep `src/roadmap/ops/mod.rs::apply_command` staged. It must still
   assign `*roadmap = staged` only after `rewrite_dependencies` succeeds.
4. Refactor: preserve the current clearing behaviour for original Markdown
   snippets. Call `markdown.clear_original_blocks()` only when text actually
   changed, not merely because an unresolved reference was detected.

Focused tests for this item:

```bash
branch=$(git branch --show-current)
safe_branch=${branch//\//-}
make test TEST_FLAGS='--workspace --all-targets --all-features unresolved_dependency_reference_is_reported' \
  2>&1 | tee "/tmp/test-mapsplice-${safe_branch}-4-1-2-item-2.out"
test "${PIPESTATUS[0]}" -eq 0

make test TEST_FLAGS='--workspace --all-targets --all-features dependency_reference' \
  2>&1 | tee "/tmp/test-mapsplice-${safe_branch}-4-1-2-item-2-dependency.out"
test "${PIPESTATUS[0]}" -eq 0
```

Before committing this work item, update this ExecPlan with the Red, Green,
Refactor, and commit evidence, then run the exact Markdown handling required
because `docs/execplans/roadmap-4-1-2.md` changed:

```bash
branch=$(git branch --show-current)
safe_branch=${branch//\//-}

mdtablefix docs/execplans/roadmap-4-1-2.md \
  2>&1 | tee "/tmp/mdtablefix-mapsplice-${safe_branch}-4-1-2-item-2-plan.out"
test "${PIPESTATUS[0]}" -eq 0

markdownlint-cli2 --fix docs/execplans/roadmap-4-1-2.md \
  2>&1 | tee "/tmp/markdownlint-fix-mapsplice-${safe_branch}-4-1-2-item-2-plan.out"
test "${PIPESTATUS[0]}" -eq 0
```

Then run the full pre-commit gates for this code/test commit:

```bash
branch=$(git branch --show-current)
safe_branch=${branch//\//-}

make all 2>&1 | tee "/tmp/make-all-mapsplice-${safe_branch}-4-1-2-item-2.out"
test "${PIPESTATUS[0]}" -eq 0

make markdownlint 2>&1 | tee "/tmp/markdownlint-mapsplice-${safe_branch}-4-1-2-item-2.out"
test "${PIPESTATUS[0]}" -eq 0

make nixie 2>&1 | tee "/tmp/nixie-mapsplice-${safe_branch}-4-1-2-item-2.out"
test "${PIPESTATUS[0]}" -eq 0
```

Acceptance for this item:

- Valid unresolved `Requires 99.1.1` is reported as
  `MapspliceError::DanglingDependency`.
- Mapped references still rewrite and increment the dependency rewrite counter.
- Invalid `Requires 1.4.0`, section references, prose numbers, and `Blocks`
  clauses remain preserved.
- `apply_command` still applies changes only after dependency rewriting
  succeeds.

### Work Item 3: Add CLI Behavioural In-Place Failure Coverage

Docs to read before starting:
`docs/users-guide.md` "Output modes" and "Validation rules and failure cases";
`docs/mapsplice-design.md` sections 5, 6, and 7;
`docs/developers-guide.md` section 6; `AGENTS.md` "Testing".

Skills to load before starting: `leta`, `rust-router`, `rust-errors`,
`rust-unit-testing`, `domain-cli-and-daemons`, and `en-gb-oxendict-style`.
Use `sem diff` before commit to review the entity-level change.

Implement Red-Green-Refactor in one gate-passable commit:

1. Red: add this scenario to `tests/features/mapsplice.feature`:

```gherkin
Scenario: Dangling dependency reference fails in place without rewriting target
  Given the target roadmap with a dangling dependency reference
  And the phase fragment roadmap
  When I try to append the phase fragment in place
  Then the command fails
  And stdout is empty
  And stderr mentions dangling dependency anchor 99.1.1
  And the target file remains unchanged
```

1. Red: add the matching `#[scenario]` function in
   `tests/behaviour_cli.rs`.
2. Green: add a `TARGET_DANGLING_DEPENDENCY` fixture constant to
   `tests/support/cli.rs`, and add a `#[given]` step that writes it.
3. Green: update `tests/steps/cli_steps.rs::CliState` so it stores
   `original_target: String`. Initialize it empty, set it in
   `CliState::write_target` whenever a fixture is written, and change
   `target_unchanged` to compare `read_target()` with `original_target`.
   This fixes the existing hard-coded comparison with `TARGET_TWO_PHASES`.
4. Green: add `#[when("I try to append the phase fragment in place")]` that
   runs `["--in-place", "append", target, fragment]`.
5. Green: add `#[then("stderr mentions dangling dependency anchor 99.1.1")]`
   that checks standard error contains
   `dependency anchor \`99.1.1\` was not found in the target roadmap`.
6. Refactor: keep all new steps in the existing CLI step module. Do not create
   a new support layer unless the existing module crosses the 400-line file
   limit from `AGENTS.md`.

Focused tests for this item:

```bash
branch=$(git branch --show-current)
safe_branch=${branch//\//-}
make test TEST_FLAGS='--workspace --all-targets --all-features dangling_dependency' \
  2>&1 | tee "/tmp/test-mapsplice-${safe_branch}-4-1-2-item-3.out"
test "${PIPESTATUS[0]}" -eq 0
```

Before committing this work item, update this ExecPlan with the Red, Green,
Refactor, and commit evidence, then run the exact Markdown handling required
because `docs/execplans/roadmap-4-1-2.md` changed:

```bash
branch=$(git branch --show-current)
safe_branch=${branch//\//-}

mdtablefix docs/execplans/roadmap-4-1-2.md \
  2>&1 | tee "/tmp/mdtablefix-mapsplice-${safe_branch}-4-1-2-item-3-plan.out"
test "${PIPESTATUS[0]}" -eq 0

markdownlint-cli2 --fix docs/execplans/roadmap-4-1-2.md \
  2>&1 | tee "/tmp/markdownlint-fix-mapsplice-${safe_branch}-4-1-2-item-3-plan.out"
test "${PIPESTATUS[0]}" -eq 0
```

Then run the full pre-commit gates for this code/test commit:

```bash
branch=$(git branch --show-current)
safe_branch=${branch//\//-}

make all 2>&1 | tee "/tmp/make-all-mapsplice-${safe_branch}-4-1-2-item-3.out"
test "${PIPESTATUS[0]}" -eq 0

make markdownlint 2>&1 | tee "/tmp/markdownlint-mapsplice-${safe_branch}-4-1-2-item-3.out"
test "${PIPESTATUS[0]}" -eq 0

make nixie 2>&1 | tee "/tmp/nixie-mapsplice-${safe_branch}-4-1-2-item-3.out"
test "${PIPESTATUS[0]}" -eq 0
```

Acceptance for this item:

- The compiled binary reports the dangling dependency on standard error.
- Standard output is empty on failure.
- The target file is unchanged in explicit `--in-place` mode.
- Existing missing-anchor and level-mismatch diagnostics still pass.

### Work Item 4: Add Generated Coverage and Reconcile Documentation

Docs to read before starting:
`docs/mapsplice-design.md` sections 7, 8, 9, and 10;
`docs/users-guide.md` "Worked example", "Output modes", and
"Validation rules and failure cases"; `docs/developers-guide.md` section 6;
`docs/roadmap.md` task 4.1.2; `docs/documentation-style-guide.md`.

Skills to load before starting: `leta`, `rust-router`, `rust-errors`,
`rust-unit-testing`, `rust-verification`, `proptest`, and
`en-gb-oxendict-style`. Use locked `proptest` 1.11.0 source and project tests
as the behavioural authority for proptest features if Firecrawl remains
unavailable.

Implement Red-Green-Refactor in one gate-passable commit:

1. Red: add a property to `tests/roadmap_properties.rs` that generates valid
   positive anchors not present in the generated roadmap, places them in
   `Requires`, and expects `run_from_args` to return
   `MapspliceError::DanglingDependency`. Construct valid anchors directly;
   do not use `prop_assume!` to filter a broad invalid domain.
2. Green: if the property exposes gaps in the previous work items, fix only
   those gaps in `dependency_text` or `rewrite.rs`.
3. Refactor: update `docs/mapsplice-design.md` section 7 so unresolved valid
   dependency references are reported instead of left unchanged. Update
   `docs/users-guide.md` validation rules to mention dangling `Requires`
   diagnostics and `--in-place` write safety on failure. Update
   `docs/roadmap.md` to tick 4.1.2 only after all gates below pass.
4. Refactor: preserve existing properties proving invalid dependency tokens,
   incidental numeric tokens, and scoped references remain unchanged beside
   mapped `Requires` references.

Focused tests and Markdown formatting for this item:

```bash
branch=$(git branch --show-current)
safe_branch=${branch//\//-}
make test TEST_FLAGS='--workspace --all-targets --all-features dangling' \
  2>&1 | tee "/tmp/test-mapsplice-${safe_branch}-4-1-2-item-4.out"
test "${PIPESTATUS[0]}" -eq 0

mdtablefix docs/mapsplice-design.md docs/users-guide.md docs/roadmap.md \
  2>&1 | tee "/tmp/mdtablefix-mapsplice-${safe_branch}-4-1-2-item-4.out"
test "${PIPESTATUS[0]}" -eq 0

markdownlint-cli2 --fix docs/mapsplice-design.md docs/users-guide.md docs/roadmap.md \
  2>&1 | tee "/tmp/markdownlint-fix-mapsplice-${safe_branch}-4-1-2-item-4.out"
test "${PIPESTATUS[0]}" -eq 0

mdtablefix docs/execplans/roadmap-4-1-2.md \
  2>&1 | tee "/tmp/mdtablefix-mapsplice-${safe_branch}-4-1-2-item-4-plan.out"
test "${PIPESTATUS[0]}" -eq 0

markdownlint-cli2 --fix docs/execplans/roadmap-4-1-2.md \
  2>&1 | tee "/tmp/markdownlint-fix-mapsplice-${safe_branch}-4-1-2-item-4-plan.out"
test "${PIPESTATUS[0]}" -eq 0
```

Then run the full pre-commit gates for this code, property-test, and Markdown
commit:

```bash
branch=$(git branch --show-current)
safe_branch=${branch//\//-}

make all 2>&1 | tee "/tmp/make-all-mapsplice-${safe_branch}-4-1-2-item-4.out"
test "${PIPESTATUS[0]}" -eq 0

make markdownlint 2>&1 | tee "/tmp/markdownlint-mapsplice-${safe_branch}-4-1-2-item-4.out"
test "${PIPESTATUS[0]}" -eq 0

make nixie 2>&1 | tee "/tmp/nixie-mapsplice-${safe_branch}-4-1-2-item-4.out"
test "${PIPESTATUS[0]}" -eq 0
```

Acceptance for this item:

- Generated valid dangling `Requires` anchors fail with
  `MapspliceError::DanglingDependency`.
- Generated invalid tokens and incidental numbers remain preserved.
- Design, users' guide, and roadmap text agree on the new contract.
- The roadmap task is ticked only after focused tests and full gates pass.

## Concrete Steps

All commands run from
`/home/leynos/Projects/mapsplice.worktrees/roadmap-4-1-2`.

Before implementing any work item:

```bash
git branch --show-current
git status --short --branch
```

Expected branch output:

```plaintext
roadmap-4-1-2
```

For every work item, follow this commit discipline:

1. Add the red test or BDD scenario first and run the focused command listed
   in the work item. Record the expected failing evidence in this ExecPlan, but
   do not commit the red state.
2. Make the smallest production or fixture change that turns the focused
   command green. Rerun the focused command and record the passing evidence in
   this ExecPlan.
3. Perform any refactor inside that work item. Rerun the focused command and
   record the passing evidence in this ExecPlan.
4. Because the ExecPlan progress evidence changes
   `docs/execplans/roadmap-4-1-2.md`, run `mdtablefix` and
   `markdownlint-cli2 --fix` on that exact file before every work-item commit.
   If the work item also changes `docs/mapsplice-design.md`,
   `docs/users-guide.md`, or `docs/roadmap.md`, run the direct Markdown
   formatter commands listed in work item 4 on those exact files too.
5. Run `sem diff` to inspect the entity-level change.
6. Run the full code gate `make all` before committing every work item,
   including work items 1, 2, and 3. Do not defer this gate to the final
   commit.
7. Run `make markdownlint` and `make nixie` before committing every work item
   whose commit includes this ExecPlan or any other Markdown file.
8. Commit the single logical unit only after all required gates pass, using an
   imperative en-GB Oxford commit message.

The full gate commands are:

```bash
branch=$(git branch --show-current)
safe_branch=${branch//\//-}

make all 2>&1 | tee "/tmp/make-all-mapsplice-${safe_branch}.out"
test "${PIPESTATUS[0]}" -eq 0

make markdownlint 2>&1 | tee "/tmp/markdownlint-mapsplice-${safe_branch}.out"
test "${PIPESTATUS[0]}" -eq 0

make nixie 2>&1 | tee "/tmp/nixie-mapsplice-${safe_branch}.out"
test "${PIPESTATUS[0]}" -eq 0
```

Format only changed Markdown files with `mdtablefix <changed-md-files>` and
`markdownlint-cli2 --fix <changed-md-files>`. Every direct formatter path
listed in this ExecPlan exists today at the point where the command is used.

## Validation and Acceptance

The final implementation is accepted when:

- `make all` passes.
- `make markdownlint` passes.
- `make nixie` passes.
- The new internal unit test proves `Requires 99.1.1.` is reported in the
  internal unresolved-anchor report.
- The new library/API test proves `Requires 99.1.1.` returns
  `MapspliceError::DanglingDependency`.
- The new BDD scenario proves the compiled CLI reports the dangling anchor on
  standard error, emits no standard output, and leaves the target unchanged in
  `--in-place` mode.
- The new property proves generated valid unresolved dependency anchors report
  dangling dependencies.
- Existing tests still prove scoped preservation for invalid tokens, section
  references, semantic versions, prose numbers, addendum references, and
  ignored `Blocks` clauses.

Red-Green-Refactor evidence must be recorded in this plan as implementation
proceeds. Each work item records:

- the red command and expected failure;
- the green command and pass;
- any refactor and rerun command;
- the commit hash after gates pass.

## Idempotence and Recovery

All implementation steps are source edits and tests; they are safe to rerun.
If a focused test fails after a partial edit, inspect the `/tmp` log named by
that work item and continue in the same worktree. Do not stash unless required
to park unrelated generated churn, and if stashing is required use:

```bash
git stash push -u -m 'df12-stash v1 task=4.1.2 kind=discard reason="park-unrelated-churn"'
```

If a formatter changes unrelated Markdown, use `sem diff` and `git diff` to
identify it, then park that unrelated churn with the named discard stash above
or revert only the unrelated formatter output with a path-specific safe
operation that does not touch user work.

## Artifacts and Notes

Planning evidence:

```plaintext
Memtrace: list_indexed_repositories -> user cancelled MCP tool call.
Leta: workspace add -> Error: IO error: Read-only file system (os error 30).
Leta scratch retry: Error: IO error: Read-only file system (os error 30).
Firecrawl: firecrawl_scrape docs.rs proptest 1.11.0 -> user cancelled MCP tool call.
Firecrawl: firecrawl_scrape docs.rs thiserror 2.0.18 -> user cancelled MCP tool call.
Firecrawl: firecrawl_scrape docs.rs markdown 1.0.0 -> user cancelled MCP tool call.
Firecrawl: firecrawl_scrape docs.rs rstest-bdd 0.5.0 -> user cancelled MCP tool call.
sem diff before plan edit -> no changed entities.
Branch: roadmap-4-1-2.
Existing contradictory test: tests/roadmap_ops.rs::unresolved_dependency_reference_is_preserved.
Existing hard-coded BDD assertion: tests/steps/cli_steps.rs::target_unchanged.
Existing diagnostic type: src/error.rs::MapspliceError::DanglingDependency.
Locked proptest version: Cargo.lock -> proptest 1.11.0.
Planned implementation surface: 12 files; escalate on any unplanned thirteenth source, test, or documentation file.
```

## Interfaces and Dependencies

No new external dependencies are required.

Internal interfaces expected after work item 1:

```rust
pub(super) struct DependencyRewriteReport {
    pub(super) value: String,
    pub(super) rewrite_count: u64,
    pub(super) unresolved: Vec<RoadmapAnchor>,
}
```

The exact struct name may stay private to
`src/roadmap/ops/dependency_text.rs`, but the behaviour is fixed:

- mapped valid dependency references rewrite and increment `rewrite_count`;
- valid dependency references with no source-local or unique cross-source
  mapping are returned in `unresolved`;
- invalid dependency tokens and non-dependency references are copied unchanged
  and do not appear in `unresolved`;
- `src/roadmap/ops/rewrite.rs::rewrite_dependencies` converts the first
  deterministic unresolved anchor into
  `MapspliceError::DanglingDependency { anchor }`;
- `src/roadmap/ops/mod.rs::apply_command` continues to stage all edits and
  publishes the staged roadmap only after dependency rewriting succeeds.

Revision note: revised after design review round 2. The plan now makes work
item 1 internal and independently gate-passable, moves API-level propagation
to work item 2, requires an explicit `--in-place` BDD failure scenario in work
item 3, fixes the target-unchanged assertion by recording original target
contents, and corrects proptest research from the stale `1.9.0` claim to the
locked `1.11.0` source and project tests.

Revision note: revised after design review round 3. The plan now requires
`make all` before every work-item commit, including the code/test commits for
work items 1, 2, and 3, instead of deferring the full code gate to the final
commit. It also specifies exact per-commit Markdown handling whenever
`docs/execplans/roadmap-4-1-2.md` is updated with Red-Green-Refactor evidence:
run `mdtablefix docs/execplans/roadmap-4-1-2.md`, then
`markdownlint-cli2 --fix docs/execplans/roadmap-4-1-2.md`, then the Markdown
gates `make markdownlint` and `make nixie` before committing.

Revision note: revised after design review round 4. The plan no longer
contains the contradictory tolerance that stopped at more than eight changed
source, test, or documentation files while enumerating twelve required files.
The scope tolerance now names those twelve planned files and escalates only if
implementation needs a thirteenth file, an unplanned file, or more than 260
net lines. This keeps the plan implementable as written while preserving a
clear expansion trigger.
