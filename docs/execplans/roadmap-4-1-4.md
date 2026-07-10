# Fail Closed on Renderer Model-Invariant Breaches

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

## Purpose / Big Picture

Roadmap task 4.1.4, "Fail closed on renderer model-invariant breaches",
hardens the final rendering boundary without regressing supported sub-task
operations. A parsed task stores addendum sub-tasks in two coordinated fields:
`TaskEntry.sub_tasks` owns the `SubTaskEntry` values, and
`TaskEntry.children` stores the original child order with
`TaskChild::SubTask(ItemIdentity)` references. If a child reference points at a
missing sub-task, the renderer currently omits it silently.

This plan must first repair the supported public operation that can create that
inconsistency today: `mapsplice delete <4-level-anchor>` routes through
`src/roadmap/ops/mod.rs::delete_anchor` to
`src/roadmap/ops/sub_task.rs::delete_sub_task`, and `delete_sub_task` removes
from `task.sub_tasks` without removing the matching `TaskChild::SubTask` from
`task.children`. After that model-consistency fix is in place, the renderer can
fail closed for an orphaned child reference as defence in depth, while normal
sub-task delete continues to succeed.

Observable success is:

- a focused model test proves sub-task delete removes the deleted identity from
  both `sub_tasks` and `children`;
- a private renderer unit test proves a synthetically inconsistent model returns
  `MapspliceError::InvalidRoadmap` and no rendered roadmap bytes;
- the existing public golden test
  `tests::roadmap_golden::contracts::f5_render_failure_in_place` continues to
  prove render-time invalid-roadmap failures leave in-place targets unchanged;
- `make all`, `make markdownlint`, and `make nixie` pass.

## Constraints

- Work only inside
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-1-4`.
- Do not edit the root/control worktree.
- Treat `origin/main` as the integration branch and `docs/roadmap.md` as the
  roadmap source of truth.
- This is planning round 2. Do not begin implementation until this draft is
  approved by the df12-build roadmap workflow.
- Preserve `docs/mapsplice-design.md` section 2: parsing remains mdast-based,
  edits run against the roadmap model, output mode remains stdout by default
  with `--in-place` on success, level matching stays strict, and tests stay
  `rstest`/`rstest-bdd` based.
- Preserve `docs/mapsplice-design.md` section 4: addendum sub-tasks are the
  fourth structural level and command-line anchors address them directly.
- Preserve `docs/mapsplice-design.md` section 5, especially F1, F3, F4, and
  F5: successful output is faithful and gate-clean, and invalid or internally
  inconsistent input is rejected before output is produced.
- Preserve `docs/mapsplice-design.md` section 6, especially C1, C2, C4, and
  C6: `delete` remains supported for addendum sub-tasks, sub-tasks renumber
  with parents, and in-place writes occur only on success.
- Preserve `docs/developers-guide.md` sections 2 and 3:
  `src/roadmap` owns mutation, renumbering, and rendering, while public APIs
  return typed `MapspliceError` values.
- Preserve `docs/developers-guide.md` section 6: unit tests cover internal
  error behaviour, behavioural/golden tests cover output modes, and property
  tests are for generated input domains rather than one structural regression.
- Preserve `docs/users-guide.md` "The roadmap shape `mapsplice` expects",
  "`delete`", "Output modes", and "Validation rules and failure cases":
  `mapsplice delete <target> <anchor>` removes one addressed item, addendum
  sub-task anchors are public, failed validation leaves in-place targets
  unchanged, and no roadmap body is emitted on failure.
- Preserve the accepted initial-tool decisions in
  `docs/execplans/initial-tool.md`: constrained roadmap grammar,
  mdast-driven parsing, a roadmap intermediate representation, and a
  deterministic renderer rather than a general Markdown writer.
- Do not add external dependencies. Use the locked crates already in
  `Cargo.lock`.
- Keep every Rust source file under 400 lines and keep module-level `//!`
  comments in any new Rust module.
- Follow en-GB Oxford spelling in prose, comments, and commit messages.
- Use Red-Green-Refactor for code changes. Where an existing passing test
  already covers the public behaviour, record it as verification rather than
  forcing a no-op commit.
- Format only changed Markdown files. Do not run repository-global Markdown
  formatters such as `make fmt` or `mdformat-all`.
- Run tests, lint, and formatting gates sequentially with `tee` logs under
  `/tmp`. Do not run test, lint, or format gates in parallel.
- Use the shared Cargo cache. Do not create an isolated Cargo cache.

## Tolerances

- Stop and escalate if implementation requires a public API signature change,
  a new external dependency, a changed accepted roadmap grammar, or a CLI
  workflow rewrite outside the existing parse/apply/render/write order.
- The planned implementation may touch only these files:
  `docs/execplans/roadmap-4-1-4.md`, `docs/roadmap.md`,
  `src/roadmap/ops/sub_task.rs`, `src/roadmap/render.rs`,
  `src/roadmap/render_tests.rs`, and `tests/roadmap_sub_tasks.rs`.
- The verification checkpoint may read, but should not need to edit,
  `tests/roadmap_golden/contracts.rs` and
  `tests/fixtures/golden/f5_render_failure_in_place/{target.md,fragment.md}`.
  If those files have drifted and must be changed, make that a separate
  reviewed commit and update this plan before editing.
- Stop and escalate if any additional source, test, fixture, or documentation
  file is needed, or if the net implementation exceeds 160 changed lines
  excluding living ExecPlan evidence.
- Stop and escalate if preserving public sub-task delete semantics requires
  changing `src/lib.rs::run_request`; the current source renders before
  writing in-place.
- Stop and escalate if a focused model, renderer, or golden test still fails
  for the same reason after three implementation attempts.
- Stop and escalate if `make all` fails for an unrelated pre-existing issue
  that cannot be isolated with a focused command and a log.
- Do not mark this plan blocked only because Memtrace, Leta, Firecrawl, Sem, or
  another advisory tool is unavailable. Record the failed command and continue
  with bounded local source and tests.

## Risks

- Risk: adding the renderer guard before fixing `delete_sub_task` would regress
  a supported public operation.
  Severity: high.
  Likelihood: high if the order is changed.
  Mitigation: Work Item 1 fixes `delete_sub_task` first and reruns the existing
  public sub-task delete regression before Work Item 2 adds the renderer guard.

- Risk: the model-consistency test could accidentally assert rendered output
  instead of the in-memory invariant.
  Severity: medium.
  Likelihood: medium.
  Mitigation: the red test must inspect `TaskEntry.children` and
  `TaskEntry.sub_tasks` after `apply_command`, before `render_roadmap` can mask
  the orphan.

- Risk: a renderer unit test may construct an impossible public state after
  Work Item 1.
  Severity: low.
  Likelihood: high by design.
  Mitigation: document it as defence-in-depth coverage. Build the inconsistent
  model by parsing valid input and then deleting only the `SubTaskEntry`; do
  not claim the public CLI can still create this state after Work Item 1.

- Risk: the existing golden F5 in-place test could be mistaken for new
  invariant coverage.
  Severity: medium.
  Likelihood: medium.
  Mitigation: Work Item 3 is verification only. It records that
  `f5_render_failure_in_place` already uses an unsupported inline image to
  trigger `InvalidRoadmap`, proving the public no-write/no-output contract but
  not the private missing-child invariant.

- Risk: advisory graph tooling may be unavailable in a sub-agent session.
  Severity: low.
  Likelihood: high.
  Mitigation: this planning round records exact failures and pins the plan to
  bounded local source windows, Leta where available, `sem diff`, and focused
  tests. Implementers should retry Memtrace and Leta before editing.

## Progress

- [x] (2026-07-02T17:17:37Z) Confirmed worktree
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-1-4` and branch
  `roadmap-4-1-4`.
- [x] (2026-07-02T17:17:37Z) Loaded the required `execplans`, `leta`,
  `memtrace-first`, `sem`, `firecrawl-mcp`, `rust-router`, `rust-errors`,
  `rust-unit-testing`, `rust-verification`, and `proptest` skills.
- [x] (2026-07-02T17:17:37Z) Retried Memtrace first; the MCP call
  `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`.
- [x] (2026-07-02T17:17:37Z) Used Leta for branch-local symbol checks:
  `leta show delete_sub_task`, `leta show delete_anchor`,
  `leta show sub_task.rs:replace_sub_task`,
  `leta show sub_task.rs:insert_sub_tasks`, `leta show render_task`,
  `leta show render_roadmap`, `leta show run_request`,
  `leta show f5_render_failure_in_place`, and
  `leta show delete_sub_task_renumbers_later_sub_tasks`.
- [x] (2026-07-02T17:17:37Z) Recorded that a later Leta reference/call-graph
  command failed with `Error: Failed to start daemon`; direct `leta show` and
  `leta grep` output plus bounded local source windows were used as fallback
  evidence.
- [x] (2026-07-02T17:17:37Z) Reviewed governing docs:
  `AGENTS.md`, `docs/mapsplice-design.md`, `docs/developers-guide.md`,
  `docs/users-guide.md`, `docs/roadmap.md`,
  `docs/documentation-style-guide.md`, and
  `docs/execplans/initial-tool.md`.
- [x] (2026-07-02T17:17:37Z) Verified the review blockers against current
  branch-local source and tests.
- [x] (2026-07-02T17:17:37Z) Revised this plan for round 2.
- [x] (2026-07-02T17:17:37Z) Formatted only
  `docs/execplans/roadmap-4-1-4.md`, then validated the documentation change
  with `make markdownlint` and `make nixie`.
- [x] (2026-07-02T17:34:10Z) Implementation approval was supplied by the
  df12-build roadmap workflow prompt for this execution round.
- [x] (2026-07-02T17:34:10Z) Work Item 1 restored sub-task delete model
  consistency. The focused red test
  `delete_sub_task_removes_matching_child_reference` failed because
  `TaskEntry.children` retained the deleted identity, then passed after
  `delete_sub_task` removed the matching child reference. The public
  `delete_sub_task_renumbers_later_sub_tasks` regression also passed.
- [x] (2026-07-02T17:34:10Z) Scrutineer ran the Work Item 1 deterministic
  gate. After fixing Clippy feedback in the new test, `make all` passed with
  evidence in `/tmp/make-all-mapsplice-roadmap-4-1-4.out`.
- [x] (2026-07-02T17:34:10Z) Scrutineer ran the Work Item 1 CodeRabbit command.
  The review deferred with no findings because the sandbox has no default
  network route; evidence is in
  `/tmp/coderabbit-delete-model-mapsplice-roadmap-4-1-4.out`.
- [x] (2026-07-02T17:39:50Z) Work Item 2 added renderer
  defence-in-depth for orphaned sub-task children. The focused red test
  `render_fails_when_task_child_references_missing_sub_task` failed because
  rendering silently omitted the missing sub-task child, then passed after
  `render_task` returned `MapspliceError::InvalidRoadmap` with the pinned
  parent-task and missing-child diagnostic. The public
  `delete_sub_task_renumbers_later_sub_tasks` regression still passed.
- [x] (2026-07-02T17:39:50Z) Scrutineer ran the Work Item 2 deterministic
  gate. `make all` passed with evidence in
  `/tmp/make-all-renderer-invariant-mapsplice-roadmap-4-1-4.out`.
- [x] (2026-07-02T17:39:50Z) Scrutineer ran the Work Item 2 CodeRabbit command.
  The review deferred with no findings because the sandbox has no default
  network route; evidence is in
  `/tmp/coderabbit-renderer-invariant-mapsplice-roadmap-4-1-4.out`.
- [x] (2026-07-02T17:43:35Z) Work Item 3 verified existing public in-place
  render-failure coverage. `cargo test --test roadmap_golden
  f5_render_failure_in_place` passed, confirming the existing
  `InvalidRoadmap` render-failure fixture leaves the in-place target
  unchanged without source or fixture edits.
- [x] (2026-07-02T17:43:35Z) Scrutineer ran the Work Item 3 deterministic
  gate. `make all` passed with evidence in
  `/tmp/make-all-golden-verification-mapsplice-roadmap-4-1-4.out`.
- [x] (2026-07-02T17:43:35Z) Scrutineer ran the Work Item 3 CodeRabbit command.
  The review deferred with no findings because the sandbox has no default
  network route; evidence is in
  `/tmp/coderabbit-golden-verification-mapsplice-roadmap-4-1-4.out`.
- [x] (2026-07-02T17:46:00Z) Work Item 4 marked
  `docs/roadmap.md` task 4.1.4 complete. Final validation, CodeRabbit
  review, and close-out evidence were completed in this close-out round.
- [x] (2026-07-02T17:49:18Z) Final deterministic gates passed. `make all`
  passed in `/tmp/final-make-all-mapsplice-roadmap-4-1-4.out`,
  `make markdownlint` passed in
  `/tmp/final-markdownlint-mapsplice-roadmap-4-1-4.out`, and `make nixie`
  passed on retry in `/tmp/final-nixie-retry-mapsplice-roadmap-4-1-4.out`.
- [x] (2026-07-02T17:49:18Z) Scrutineer ran the final CodeRabbit command. The
  review deferred with no findings because the sandbox has no default network
  route; evidence is in `/tmp/coderabbit-final-mapsplice-roadmap-4-1-4.out`.

## Surprises & Discoveries

- Observation: Memtrace MCP tools were exposed but unusable in this host
  session.
  Evidence: `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`.
  Impact: Memtrace canonical main-branch graph context could not be used in
  this planning round. The plan records a retry step before implementation and
  uses bounded branch-local evidence for the draft.

- Observation: Firecrawl was exposed but unusable in this host session.
  Evidence: `mcp__firecrawl.firecrawl_scrape` for
  `https://docs.rs/thiserror/2.0.18/thiserror/` returned
  `user cancelled MCP tool call`.
  Impact: official web documentation could not be retrieved through
  Firecrawl. This plan therefore avoids any new external-library behaviour and
  cites local locked crate source for the already-used `thiserror` and
  `rstest` APIs.

- Observation: Leta direct symbol lookup worked, but a later reference/call
  graph command failed.
  Evidence: `leta workspace add
  /home/leynos/Projects/mapsplice.worktrees/roadmap-4-1-4` succeeded, and
  `leta show` returned source for key symbols; `leta refs delete_sub_task &&
  leta refs render_task && leta calls --to delete_sub_task && leta calls --to
  render_task` failed with `Error: Failed to start daemon`.
  Impact: branch-local verification used successful Leta symbol output and
  bounded source windows for exact line evidence.

- Observation: the reviewer's primary blocker is valid in current source.
  Evidence: `src/roadmap/ops/sub_task.rs::delete_sub_task` lines 31-35 remove
  from `task.sub_tasks` only. `src/roadmap/ops/mod.rs::delete_anchor` lines
  194-217 routes `RoadmapAnchor::SubTask` to `delete_sub_task`. In contrast,
  `insert_sub_tasks` and `replace_sub_task` splice both `task.sub_tasks` and
  `task.children`.
  Impact: the first implementation item must fix `delete_sub_task`; otherwise
  the renderer guard would break supported `mapsplice delete <sub-task>`.

- Observation: the reviewer's secondary blocker is valid in current tests.
  Evidence: `tests/roadmap_golden/contracts.rs::f5_render_failure_in_place`
  already asserts `ExpectedError::InvalidRoadmap` and
  `FailureOutput::InPlaceTargetUnchanged`; the fixture target contains
  `![unsupported](unsupported.png)`, which triggers render-time invalid
  roadmap handling independently of the missing-child invariant.
  Impact: the golden F5 case is verification, not new coverage, and this plan
  no longer mandates a no-op commit for it.

- Observation: `sem diff --format json` reported a clean worktree before this
  round-2 plan edit.
  Evidence: the summary was `fileCount:0` and `total:0`.
  Impact: the plan revision started from a clean branch.

- Observation: in the implementation session, Leta could not initialize the
  worktree.
  Evidence: `leta workspace add
  /home/leynos/Projects/mapsplice.worktrees/roadmap-4-1-4` returned
  `Error: IO error: Read-only file system (os error 30)`, and later
  `leta show delete_sub_task` returned `Error: Failed to start daemon`.
  Impact: branch-local verification used bounded source inspection and focused
  tests, as permitted by the plan's advisory-tool fallback.

- Observation: CodeRabbit could not run in this sandbox.
  Evidence: the review command returned
  `deferred coderabbit review: no default network route visible in this
  sandbox`.
  Impact: Work Items 1 through 4 have no actionable AI-review findings, but
  the deferred CodeRabbit reviews remain an open issue for a network-enabled
  supervisor or relaunch environment.

- Observation: final Mermaid validation had one transient timeout on an
  untouched document, then passed without edits.
  Evidence: `/tmp/final-nixie-mapsplice-roadmap-4-1-4.out` timed out rendering
  `docs/ortho-config-users-guide.md` diagram 1. A focused retry, captured in
  `/tmp/final-nixie-retry-mapsplice-roadmap-4-1-4.out`, passed all diagrams.
  Impact: no out-of-scope documentation edit was made. The final gate state is
  green.

## Decision Log

- Decision: fix the public sub-task delete model inconsistency before adding
  the renderer fail-closed guard.
  Rationale: `docs/mapsplice-design.md` C1 and C4 plus `docs/users-guide.md`
  document addendum sub-tasks as first-class, addressable items. Failing closed
  in the renderer without fixing `delete_sub_task` would turn a supported
  operation into `InvalidRoadmap`.
  Date/Author: 2026-07-02, planning agent.

- Decision: keep the renderer guard as defence-in-depth reachable through a
  private unit test, not through the public CLI after Work Item 1.
  Rationale: after `delete_sub_task` removes the matching child reference, the
  missing-child inconsistency should no longer be reachable through normal
  `mapsplice delete <sub-task>`. The renderer still must fail closed if future
  code constructs an inconsistent model.
  Date/Author: 2026-07-02, planning agent.

- Decision: use `MapspliceError::InvalidRoadmap` for the renderer invariant
  breach.
  Rationale: `docs/developers-guide.md` section 3 requires typed public errors,
  `src/error.rs` already exposes `InvalidRoadmap { message }`, and
  `rust-errors` favours typed recoverable errors over panics at library
  boundaries.
  Date/Author: 2026-07-02, planning agent.

- Decision: treat `f5_render_failure_in_place` as an existing public contract
  verification checkpoint, not a required change.
  Rationale: the test and fixture already exist and already assert the
  in-place target remains unchanged on `InvalidRoadmap`. Requiring a commit
  there would mislabel existing coverage as new work.
  Date/Author: 2026-07-02, planning agent.

- Decision: do not add a property test for this task.
  Rationale: `rust-verification` and `proptest` route property testing to
  generated input domains and algebraic properties. This work has two concrete
  structural regressions with direct unit and behavioural coverage.
  Date/Author: 2026-07-02, planning agent.

## Outcomes & Retrospective

All work items are complete. `delete_sub_task` now removes the owned
`SubTaskEntry` and its matching `TaskChild::SubTask` child-order reference in
the same mutation. `render_task` now fails closed with
`MapspliceError::InvalidRoadmap` when task child ordering references a missing
sub-task identity. The new model-level and renderer-level tests pin those
invariants before public output can mask them, and the public sub-task delete
regression still passes. The existing F5 in-place render-failure golden test
passes without fixture edits, confirming public no-write-on-render-failure
coverage remains intact. `docs/roadmap.md` marks 4.1.4 complete. Final
`make all`, `make markdownlint`, and `make nixie` are green. CodeRabbit review
is deferred by sandbox networking rather than a review finding.

## Context and Orientation

`mapsplice` parses roadmap-shaped Markdown into a domain model, applies one
structural operation, renumbers affected items, rewrites dependency references,
and renders the supported grammar. The relevant pipeline is documented in
`docs/mapsplice-design.md` section 3 and implemented by
`src/lib.rs::run_request`: read target, parse, load fragment, apply operation,
render, and only then write in-place.

The relevant files are:

- `src/roadmap/model.rs`: defines `TaskEntry`, `SubTaskEntry`, and
  `TaskChild`. `TaskEntry.sub_tasks` stores first-class sub-tasks, while
  `TaskEntry.children` preserves their order relative to body blocks.
- `src/roadmap/ops/mod.rs`: defines `RoadmapOperation`, `apply_command`, and
  `delete_anchor`. `delete_anchor` dispatches sub-task deletes to
  `delete_sub_task`.
- `src/roadmap/ops/sub_task.rs`: defines sub-task insert, delete, replace, and
  helper lookup functions. Insert and replace already keep `children` and
  `sub_tasks` aligned; delete does not.
- `src/roadmap/render.rs`: defines `render_roadmap`, `render_tasks`, and
  `render_task`. The current `render_task` branch for
  `TaskChild::SubTask(identity)` silently skips the child when no matching
  `SubTaskEntry` exists.
- `src/roadmap/render_tests.rs`: private renderer tests live beside the
  renderer and can construct inconsistent model values without widening public
  APIs.
- `tests/roadmap_sub_tasks.rs`: public CLI sub-task operation tests, including
  `delete_sub_task_renumbers_later_sub_tasks`.
- `tests/roadmap_golden/contracts.rs`: golden contract tests. The existing
  `f5_render_failure_in_place` test proves an in-place render failure keeps the
  target unchanged.

Locked-library research is pinned as follows:

- `thiserror` is declared as `thiserror = "2.0.18"` in `Cargo.toml` and locked
  to `2.0.18` in `Cargo.lock`. Local locked source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-2.0.18/src/lib.rs`
  documents `#[derive(Error, Debug)]` and `#[error("...")]` display messages.
  `src/error.rs` already uses this for `MapspliceError`; no new `thiserror`
  feature is required.
- `rstest` is declared as `rstest = "0.26.1"` in `Cargo.toml` and locked to
  `0.26.1` in `Cargo.lock`. Local locked source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rstest-0.26.1/src/lib.rs`
  documents `#[rstest]`, `#[fixture]`, fixture injection, and parameterized
  cases. The planned tests use existing repository `rstest` patterns.
- Firecrawl could not retrieve official docs in this session. Because no work
  item requires new dependency behaviour, local locked source plus existing
  repository usage is sufficient for this plan.

## Plan of Work

### Work Item 1: Restore sub-task delete model consistency

Documentation to read before this work item:
`AGENTS.md` "Rust Specific Guidance", "Testing", and "Error Handling";
`docs/mapsplice-design.md` sections 2, 4, 5, 6, and 8;
`docs/developers-guide.md` sections 2, 3, and 6;
`docs/users-guide.md` "The roadmap shape `mapsplice` expects", "`delete`",
and "Output modes"; `docs/roadmap.md` task 4.1.4.

Skills to load before editing: `memtrace-first`, `leta`, `sem`,
`rust-router`, `rust-errors`, and `rust-unit-testing`. Retry Memtrace
`list_indexed_repositories` and, if `repo_id: "mapsplice"` is available, use
`find_symbol` for `delete_sub_task`, `delete_anchor`, `find_sub_task_child_index`,
and `apply_command`, then use `get_symbol_context`, `get_impact`, and
`get_timeline` before editing. If Memtrace or Leta fails with the host-session
errors recorded above, continue with bounded local source inspection and record
the failure in this plan.

Red test:

1. Add a small unit test in `src/roadmap/ops/sub_task.rs` under
   `#[cfg(test)]`, for example
   `delete_sub_task_removes_matching_child_reference`.
2. Parse a minimal roadmap with one task and two nested sub-tasks, then call
   `apply_command` or the private `delete_sub_task` path to delete
   `1.1.1.1`.
3. Inspect the parent task before rendering. Assert:
   `task.sub_tasks.len() == 1`; `task.children` contains exactly one
   `TaskChild::SubTask`; and that remaining child identity equals the remaining
   `SubTaskEntry.identity`.
4. Run the focused red command and expect it to fail because the current
   implementation leaves the deleted identity in `task.children`:

   ```bash
   cargo test delete_sub_task_removes_matching_child_reference \
     | tee /tmp/red-delete-child-order-mapsplice-roadmap-4-1-4.out
   ```

Green implementation:

1. In `src/roadmap/ops/sub_task.rs::delete_sub_task`, mirror the existing
   insert/replace pattern. Capture the target sub-task identity with
   `sub_task_identity(task, sub_task_index)?`, find the child position with
   `find_sub_task_child_index(task, target_identity)?`, remove the
   `SubTaskEntry`, and remove the matching `TaskChild::SubTask`.
2. Do not change renumbering, parser behaviour, renderer behaviour, or public
   APIs in this commit.
3. Rerun the focused command and expect the new test to pass:

   ```bash
   cargo test delete_sub_task_removes_matching_child_reference \
     | tee /tmp/green-delete-child-order-mapsplice-roadmap-4-1-4.out
   ```

Regression validation:

```bash
cargo test --test roadmap_sub_tasks delete_sub_task_renumbers_later_sub_tasks \
  | tee /tmp/sub-task-delete-regression-mapsplice-roadmap-4-1-4.out
make all | tee /tmp/make-all-delete-model-mapsplice-roadmap-4-1-4.out
```

If this work item updates this ExecPlan with evidence before committing,
format the changed plan file and run Markdown gates:

```bash
mdtablefix docs/execplans/roadmap-4-1-4.md
markdownlint-cli2 --fix docs/execplans/roadmap-4-1-4.md
make markdownlint | tee /tmp/markdownlint-delete-model-mapsplice-roadmap-4-1-4.out
make nixie | tee /tmp/nixie-delete-model-mapsplice-roadmap-4-1-4.out
```

Commit only after the focused tests, `make all`, and any required Markdown
gates pass.

### Work Item 2: Add renderer defence-in-depth for orphaned sub-task children

Documentation to read before this work item:
`docs/roadmap.md` task 4.1.4; `docs/mapsplice-design.md` sections 5 F5 and 6
C4/C6; `docs/developers-guide.md` sections 2, 3, and 6; `docs/users-guide.md`
"Validation rules and failure cases"; `AGENTS.md` "Error Handling".

Skills to load before editing: `memtrace-first`, `leta`, `sem`,
`rust-router`, `rust-errors`, and `rust-unit-testing`. Retry Memtrace for
`render_task`, `render_roadmap`, and `MapspliceError`; use
`get_symbol_context`, `get_impact`, and `get_timeline` before editing if
available.

Red test:

1. Add a private renderer unit test to `src/roadmap/render_tests.rs`, for
   example `render_fails_when_task_child_references_missing_sub_task`.
2. Parse a valid minimal roadmap containing a parent task and one nested
   sub-task.
3. Mutate the parsed model by removing only the `SubTaskEntry` from
   `roadmap.phases[0].steps[0].tasks[0].sub_tasks`, leaving the existing
   `TaskChild::SubTask` in `children`.
4. Call `render_roadmap(&roadmap)` and assert it returns
   `MapspliceError::InvalidRoadmap { message }`.
5. Assert the message identifies the parent task and missing child context. A
   stable proposed message is:

   ```plaintext
   task `1.1.1` child ordering references missing sub-task `1.1.1.1`
   ```

Run the focused red command and expect it to fail before production changes
because `render_task` currently returns `Ok` after silently skipping the child:

```bash
cargo test render_fails_when_task_child_references_missing_sub_task \
  | tee /tmp/red-renderer-invariant-mapsplice-roadmap-4-1-4.out
```

Green implementation:

1. In `src/roadmap/render.rs::render_task`, replace the `if let Some(...)`
   branch for `TaskChild::SubTask(identity)` with a fallible lookup.
2. Prefer a tiny private helper if it improves clarity, for example
   `find_sub_task_for_child(task: &TaskEntry, identity: ItemIdentity) ->
   Result<&SubTaskEntry>`.
3. On a missing identity, return `MapspliceError::InvalidRoadmap { message }`.
   The diagnostic must include `task.number` and the missing child reference.
   If deriving the original anchor from `ItemIdentity` is clearer than using a
   rendered sub-task number that no longer exists, include that anchor in the
   message and update the test to pin the exact string.
4. Do not change `render_roadmap`, `render_tasks`, public APIs, parser shape,
   or sub-task operation logic in this commit.

Run the focused green command and expect the new test to pass:

```bash
cargo test render_fails_when_task_child_references_missing_sub_task \
  | tee /tmp/green-renderer-invariant-mapsplice-roadmap-4-1-4.out
```

Then rerun the public delete regression to prove Work Item 1 prevents the new
guard from breaking supported sub-task deletes:

```bash
cargo test --test roadmap_sub_tasks delete_sub_task_renumbers_later_sub_tasks \
  | tee /tmp/post-renderer-sub-task-delete-mapsplice-roadmap-4-1-4.out
make all | tee /tmp/make-all-renderer-invariant-mapsplice-roadmap-4-1-4.out
```

If this work item updates this ExecPlan with evidence before committing,
format the changed plan file and run Markdown gates:

```bash
mdtablefix docs/execplans/roadmap-4-1-4.md
markdownlint-cli2 --fix docs/execplans/roadmap-4-1-4.md
make markdownlint | tee /tmp/markdownlint-renderer-invariant-mapsplice-roadmap-4-1-4.out
make nixie | tee /tmp/nixie-renderer-invariant-mapsplice-roadmap-4-1-4.out
```

Commit only after the focused tests, `make all`, and any required Markdown
gates pass.

### Work Item 3: Verify existing public in-place render-failure coverage

Documentation to read before this work item:
`docs/mapsplice-design.md` section 5 F5 and section 6 C6,
`docs/users-guide.md` "Output modes" and "Validation rules and failure cases",
`docs/developers-guide.md` section 6, `docs/roadmap.md` task 4.1.4, and
`AGENTS.md` "Testing".

Skills to load before verification: `rust-router`, `rust-errors`, and
`rust-unit-testing`.

This is a verification checkpoint, not a planned code change. Do not add a new
golden fixture merely to exercise the missing-child invariant publicly; after
Work Item 1, that inconsistency is not reachable through the public CLI. The
existing test `tests/roadmap_golden/contracts.rs::f5_render_failure_in_place`
already asserts `ExpectedError::InvalidRoadmap` and
`FailureOutput::InPlaceTargetUnchanged`, and its existing `target.md` fixture
contains an unsupported inline image that produces a render-time
`InvalidRoadmap`.

Run:

```bash
cargo test --test roadmap_golden f5_render_failure_in_place \
  | tee /tmp/golden-render-failure-mapsplice-roadmap-4-1-4.out
make all | tee /tmp/make-all-golden-verification-mapsplice-roadmap-4-1-4.out
```

Expected result: both commands pass with no source or fixture edits. If this
checkpoint requires only verification, record the pass in `Progress` and do
not create a commit. If fixture or test drift makes an edit necessary, update
this plan first, format only the changed Markdown fixture paths that exist,
and commit that change separately after the gates.

### Work Item 4: Close roadmap task 4.1.4 and finalize plan evidence

Documentation to read before this work item:
`docs/roadmap.md` section 4.1.4, `docs/mapsplice-design.md` sections 5 and 8,
`docs/developers-guide.md` sections 6 and 7,
`docs/documentation-style-guide.md`, and `AGENTS.md` "Markdown Guidance".

Skills to load before editing: `execplans`, `sem`, and `rust-router` only if
code evidence needs interpretation. Use `sem diff --format json` to record the
entity-level change summary before the final commit.

Update living documents:

1. Mark `docs/roadmap.md` task 4.1.4 complete only after Work Items 1 through
   3 have passed their gates.
2. Update this ExecPlan's `Progress`, `Surprises & Discoveries`,
   `Decision Log`, and `Outcomes & Retrospective` with concise evidence:
   focused red failures, focused green passes, golden verification,
   `make all`, `make markdownlint`, and `make nixie`.
3. Set `Status: COMPLETE` only after every required gate passes and the final
   commit is ready.

Format only the changed Markdown files. These paths exist at this point:

```bash
mdtablefix docs/roadmap.md docs/execplans/roadmap-4-1-4.md
markdownlint-cli2 --fix docs/roadmap.md docs/execplans/roadmap-4-1-4.md
```

Run final validation:

```bash
make all | tee /tmp/final-make-all-mapsplice-roadmap-4-1-4.out
make markdownlint | tee /tmp/final-markdownlint-mapsplice-roadmap-4-1-4.out
make nixie | tee /tmp/final-nixie-mapsplice-roadmap-4-1-4.out
```

Commit the roadmap and ExecPlan update only after all three gates pass.

## Concrete Steps

1. From the repository root, confirm the worktree and branch:

   ```bash
   cd /home/leynos/Projects/mapsplice.worktrees/roadmap-4-1-4
   git branch --show-current
   ```

   Expected output:

   ```plaintext
   roadmap-4-1-4
   ```

2. Retry advisory tooling before code edits:

   ```bash
   leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-1-4
   sem diff --format json
   ```

   Also call Memtrace MCP `list_indexed_repositories`. If `mapsplice` is
   listed, use Memtrace for the target symbols before editing. If the tools
   fail as recorded in this plan, add a short dated note under
   `Surprises & Discoveries` and continue.

3. Implement Work Item 1 with Red-Green-Refactor, then commit after gates.

4. Implement Work Item 2 with Red-Green-Refactor, then commit after gates.

5. Execute Work Item 3 as verification. Do not commit if there are no changes.

6. Implement Work Item 4, run final gates, and commit it.

7. Before each commit, inspect the semantic diff:

   ```bash
   sem diff --format json
   git diff --check
   git status --short
   ```

   Do not commit unrelated formatter churn or generated artefacts.

## Validation and Acceptance

The required final validation commands are:

```bash
make all | tee /tmp/final-make-all-mapsplice-roadmap-4-1-4.out
make markdownlint | tee /tmp/final-markdownlint-mapsplice-roadmap-4-1-4.out
make nixie | tee /tmp/final-nixie-mapsplice-roadmap-4-1-4.out
```

`make all` is required because it includes `check-fmt`, `lint`, `typecheck`,
and `test` on current `origin/main`. `make markdownlint` and `make nixie` are
required because this plan and `docs/roadmap.md` are Markdown changes.

Acceptance criteria:

- The new model-consistency test fails before the Work Item 1 production change
  because `delete_sub_task` leaves a deleted sub-task identity in
  `TaskEntry.children`.
- The same model-consistency test passes after `delete_sub_task` removes the
  matching child reference.
- The existing public test
  `tests/roadmap_sub_tasks.rs::delete_sub_task_renumbers_later_sub_tasks`
  passes before and after the renderer guard, proving
  `mapsplice delete <sub-task>` remains supported.
- The new renderer unit test fails before the Work Item 2 production change
  because the renderer silently omits a missing sub-task child reference.
- The same renderer unit test passes after `render_task` returns
  `MapspliceError::InvalidRoadmap` for the missing child.
- `cargo test --test roadmap_golden f5_render_failure_in_place` passes and
  proves a render-time invalid-roadmap failure leaves the in-place target
  unchanged.
- `make all`, `make markdownlint`, and `make nixie` pass.
- `docs/roadmap.md` marks task 4.1.4 complete only after the tests and gates
  pass.

## Idempotence and Recovery

The implementation steps are idempotent when applied to a clean branch. If a
focused test fails unexpectedly, read the corresponding `/tmp` log before
rerunning. If a formatter changes files outside the stated work item paths, do
not commit the churn; either revert only the unrelated formatter changes or
park them with a named stash:

```bash
git stash push -m 'df12-stash v1 task=roadmap-4-1-4 kind=discard reason="unrelated formatter churn"' -- <paths>
```

Do not use a bare `git stash`. Do not use `git reset --hard` or
`git checkout --` unless explicitly instructed.

If `make all` fails for an unrelated pre-existing issue, keep the focused test
logs, record the failure under `Surprises & Discoveries`, and escalate only if
the failure cannot be isolated from this task.

## Artifacts and Notes

Planning evidence:

```plaintext
Branch: roadmap-4-1-4
Memtrace: list_indexed_repositories -> user cancelled MCP tool call
Firecrawl: docs.rs thiserror scrape -> user cancelled MCP tool call
Leta: workspace add -> Added workspace
Leta: direct show commands -> source returned for delete_sub_task, render_task, run_request, and tests
Leta: refs/calls command -> Error: Failed to start daemon
Sem: sem diff --format json -> no changes before plan revision
Implementation Leta: workspace add -> Error: IO error: Read-only file system (os error 30)
Implementation Memtrace: list_indexed_repositories -> user cancelled MCP tool call
Work Item 1 red: cargo test delete_sub_task_removes_matching_child_reference -> failed as expected
Work Item 1 green: cargo test delete_sub_task_removes_matching_child_reference -> passed
Work Item 1 regression: cargo test --test roadmap_sub_tasks delete_sub_task_renumbers_later_sub_tasks -> passed
Work Item 1 gate: make all -> passed in /tmp/make-all-mapsplice-roadmap-4-1-4.out
Work Item 1 CodeRabbit: coderabbit-review-agent -> deferred, no default network route
Work Item 2 red: cargo test render_fails_when_task_child_references_missing_sub_task -> failed as expected
Work Item 2 green: cargo test render_fails_when_task_child_references_missing_sub_task -> passed
Work Item 2 regression: cargo test --test roadmap_sub_tasks delete_sub_task_renumbers_later_sub_tasks -> passed
Work Item 2 gate: make all -> passed in /tmp/make-all-renderer-invariant-mapsplice-roadmap-4-1-4.out
Work Item 2 CodeRabbit: coderabbit-review-agent -> deferred, no default network route
Work Item 3 golden: cargo test --test roadmap_golden f5_render_failure_in_place -> passed
Work Item 3 gate: make all -> passed in /tmp/make-all-golden-verification-mapsplice-roadmap-4-1-4.out
Work Item 3 CodeRabbit: coderabbit-review-agent -> deferred, no default network route
Work Item 4 roadmap: docs/roadmap.md 4.1.4 -> marked complete
Final make all: make all -> passed in /tmp/final-make-all-mapsplice-roadmap-4-1-4.out
Final markdownlint: make markdownlint -> passed in /tmp/final-markdownlint-mapsplice-roadmap-4-1-4.out
Final nixie: make nixie -> transient timeout, retry passed in /tmp/final-nixie-retry-mapsplice-roadmap-4-1-4.out
Final CodeRabbit: coderabbit-review-agent -> deferred, no default network route
```

Current public inconsistency:

```rust
pub(super) fn delete_sub_task(roadmap: &mut RoadmapDocument, target: SubTaskNumber) -> Result<()> {
    let (task, sub_task_index) = find_sub_task_parent_mut(roadmap, target)?;
    task.sub_tasks.remove(sub_task_index);
    Ok(())
}
```

The implementation must make this function remove the corresponding
`TaskChild::SubTask` as well.

Current renderer silent omission:

```rust
TaskChild::SubTask(identity) => {
    if let Some(sub_task) = task
        .sub_tasks
        .iter()
        .find(|sub_task| sub_task.identity == *identity)
    {
        parts.push(render_sub_task(sub_task, 2)?);
    }
}
```

The implementation must replace this branch with a fallible lookup after
Work Item 1 is complete.

## Interfaces and Dependencies

At the end of the implementation, the public renderer interface remains:

```rust
pub fn render_roadmap(roadmap: &RoadmapDocument) -> Result<String>
```

No public function signature changes. No new dependency is added.

The load-bearing operation behaviour is:

```rust
pub(super) fn delete_sub_task(roadmap: &mut RoadmapDocument, target: SubTaskNumber) -> Result<()>
```

When it succeeds, both `task.sub_tasks` and `task.children` no longer contain
the deleted sub-task identity.

The load-bearing renderer behaviour is:

```rust
fn render_task(task: &TaskEntry) -> Result<String>
```

When `task.children` contains `TaskChild::SubTask(identity)` and no matching
`SubTaskEntry` exists in `task.sub_tasks`, `render_task` returns
`Err(MapspliceError::InvalidRoadmap { message })`. The message must include
the parent task number and the missing child reference context.

The public workflow keeps this existing ordering in `src/lib.rs::run_request`:
parse, apply operation, render, then write in-place only after rendering
succeeds.

## Revision Note

2026-07-02: Round-2 revision after design review. The plan now fixes
`delete_sub_task` first so public sub-task deletes cannot leave orphaned child
references, treats the renderer guard as defence-in-depth, and reframes the
existing `f5_render_failure_in_place` golden test as verification rather than
new required coverage. The allowed file list, tolerances, risks, decision log,
work items, and validation commands were updated to match that sequence.
