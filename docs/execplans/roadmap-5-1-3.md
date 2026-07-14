# Collapse duplicated lookup and rendering helpers

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

This plan is approved by the df12-build roadmap workflow and is being executed
work item by work item. This revision addresses the round-1 design-review
blocker about fail-closed dependency-rewrite metrics for failed in-place writes.

## Purpose / big picture

Roadmap task 5.1.3, "Collapse duplicated lookup and rendering helpers", reduces
drift after the parser strictness work. The task is a behaviour-preserving
refactor: users should see the same roadmap edits, diagnostics, rendered
Markdown, and observability metrics, while maintainers get one helper per
repeated concept.

The current duplication is concentrated in four small seams:

- phase lookup by number is repeated in phase insert, delete, and replace;
- fragment-level routing mixes the public `fragment_level` shim and the
  `RoadmapFragment::level` method;
- checkbox marker rendering is repeated for tasks, sub-tasks, and generic
  unordered task-list items;
- `run_request` records dependency rewrites in both output branches even
  though the count is branch-independent.

After this change, internal call sites should use a single helper or method for
each concept. Public behaviour and public library APIs remain unchanged.

## Constraints

- Work only inside
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-3`.
- Do not edit the root/control worktree.
- Treat `origin/main` as canonical and `docs/roadmap.md` as the roadmap source
  of truth.
- Implement only roadmap task 5.1.3 from `docs/roadmap.md`: "Single-source
  phase lookup, checkbox marker rendering, fragment-level routing, and
  dependency-rewrite recording where the current code repeats
  branch-independent logic."
- Preserve `docs/mapsplice-design.md` section 2, "Non-negotiable constraints":
  parsing remains mdast-based through the locked `markdown` crate and edits run
  through the roadmap model rather than raw-string surgery.
- Preserve `docs/mapsplice-design.md` section 4, "The roadmap grammar
  (normative reference)": phases, steps, tasks, and addendum sub-tasks keep the
  same grammar.
- Preserve `docs/mapsplice-design.md` section 5, "Fidelity guarantees",
  especially F1 content preservation, F2 minimal diff, F3 round-trip stability,
  F4 gate-clean output, and F5 fail-closed behaviour.
- Preserve `docs/mapsplice-design.md` section 6, "Functional and contract
  guarantees", especially C1 operations, C2 contiguous renumbering, C3
  dependency-reference rewriting, C4 first-class addendum sub-tasks, and C6
  output modes.
- Preserve `docs/mapsplice-design.md` section 8, "Fixture and test
  requirements": unit tests cover model and operation behaviour, behavioural
  tests cover CLI flows, and golden fixtures remain exact.
- Preserve `docs/developers-guide.md` section 2, "Architecture boundaries":
  `src/lib.rs` owns the application workflow and `src/roadmap` owns domain
  parsing, mutation, renumbering, and rendering.
- Preserve `docs/developers-guide.md` section 3, "Public library APIs": do not
  remove or rename public APIs, public error variants, or re-exports.
- Follow `docs/developers-guide.md` section 6, "Verification layers": use
  `rstest` unit tests for finite operation/rendering matrices; keep
  `rstest-bdd`, `proptest`, `trybuild`, and `insta` for surfaces that already
  own those behaviours.
- Follow `docs/users-guide.md`, "The roadmap shape `mapsplice` expects",
  "Output modes", and "Validation rules and failure cases".
- Follow `docs/documentation-style-guide.md`: prose must use en-GB Oxford
  spelling, sentence-case headings, fenced code languages, and 80-column
  wrapping.
- Do not add a new external dependency.
- Do not change accepted roadmap grammar, rendered output, CLI arguments,
  public metrics fields, or diagnostic text.
- Keep every Rust source file under 400 lines. Current line counts are
  `src/lib.rs` 259, `src/roadmap/ops/mod.rs` 316, `src/roadmap/render.rs` 379,
  `src/roadmap/model.rs` 256, `tests/roadmap_ops.rs` 399,
  `tests/roadmap_render.rs` 262, and `tests/roadmap_parse.rs` 217.
- Because `tests/roadmap_ops.rs` is already 399 lines, do not add new tests to
  that file unless lines are first moved or removed. Prefer colocated
  `#[cfg(test)]` unit tests in `src/roadmap/ops/mod.rs` or a new integration
  test module when extra operation coverage is needed.
- Use Red-Green-Refactor. Because this is a behaviour-preserving refactor, the
  red stage may be a mutation-style proof: temporarily change the helper or the
  expected assertion, confirm the focused test fails for the intended reason,
  and revert the temporary mutation before committing.
- Format only changed Markdown files. Do not run repository-global Markdown
  formatters such as `make fmt` or `mdformat-all`.
- Keep this ExecPlan current after every work item. Every work item therefore
  includes scoped Markdown formatting plus `make markdownlint` and `make nixie`.
- Run tests, lint, and formatting gates sequentially with `tee` logs under
  `/tmp`. Do not run test, lint, or format gates in parallel.
- Use the shared Cargo cache. Do not create an isolated Cargo cache.

## Tolerances

- Stop and escalate if implementation requires a public API signature change,
  a public re-export removal, a new public error variant, a new crate, or any
  change to accepted roadmap grammar.
- The planned implementation may touch only these files unless a focused test
  proves a necessary adjacent change: `docs/execplans/roadmap-5-1-3.md`,
  `docs/roadmap.md`, `src/lib.rs`, `src/roadmap/ops/mod.rs`,
  `src/roadmap/model.rs`, `src/roadmap/render.rs`, `tests/roadmap_parse.rs`,
  `tests/roadmap_render.rs`, `tests/roadmap_sub_tasks.rs`, and any new narrowly
  named integration test module created to keep existing test files under 400
  lines.
- Stop and escalate if implementation needs changes outside `src/lib.rs`,
  `src/roadmap/model.rs`, `src/roadmap/ops/mod.rs`, `src/roadmap/render.rs`,
  and focused tests, excluding the living ExecPlan and roadmap status update.
- Stop and escalate if exact rendered Markdown changes in any existing golden
  fixture or CLI behaviour test.
- Stop and escalate if dependency rewrite metrics change for stdout,
  successful in-place output, or failed in-place output. Failed in-place writes
  that occur after dependency rewriting must not increment
  `DEPENDENCY_REWRITES`.
- Stop and escalate if phase lookup helper extraction changes the
  `AnchorNotFound` anchor payload for insert, delete, or replace.
- Stop and escalate if the checkbox helper would need a public renderer API,
  broad trait abstraction, or new renderer module solely to satisfy this task.
- Stop and escalate if fragment-level routing consolidation would remove the
  public `fragment_level` compatibility function; internal call sites may
  standardize on `RoadmapFragment::level`, but public APIs must remain stable.
- Stop and escalate if the net production-code diff exceeds 120 lines or any
  Rust source file exceeds 400 lines after the change.
- Stop and escalate if the same focused test still fails after three
  implementation attempts.
- Do not treat advisory-tool unavailability as a blocker. Record the failed
  command or MCP call and continue with bounded local source, tests, and
  documentation evidence.

## Risks

- Risk: a phase lookup helper could accidentally change which anchor is
  reported for a missing phase. Severity: medium. Likelihood: medium.
  Mitigation: Work Item 1 pins insert, delete, and replace missing-phase errors
  before the helper extraction.
- Risk: checkbox rendering consolidation could change spacing around checklist
  markers. Severity: medium. Likelihood: medium. Mitigation: Work Item 1 pins
  task, sub-task, and generic list-item rendering before production changes.
- Risk: hoisting dependency-rewrite recording could record metrics before an
  in-place write succeeds. Severity: medium. Likelihood: medium. Mitigation:
  Work Item 1 adds a focused `#[serial]` regression test proving a failed
  in-place rewrite does not increment `DEPENDENCY_REWRITES`; Work Item 4 must
  place the single recording call strictly after the in-place `rewrite_utf8`
  call succeeds, never before the branch.
- Risk: operation and rendering refactors may appear too small to test.
  Severity: medium. Likelihood: medium. Mitigation: each work item includes a
  focused red proof, then existing broader gates and golden fixtures prove no
  public drift.
- Risk: advisory tools may remain unavailable in the implementation session.
  Severity: low. Likelihood: high. Mitigation: retry Memtrace and Leta before
  editing, record exact failures if they persist, and proceed with bounded
  local source inspection and tests.

## Progress

- [x] (2026-07-03T00:00:00Z) Confirmed the current branch is
  `roadmap-5-1-3`, so this plan is `docs/execplans/roadmap-5-1-3.md`.
- [x] (2026-07-03T00:00:00Z) Loaded planning and navigation skills:
  `execplans`, `leta`, `sem`, `firecrawl-mcp`, `rust-router`,
  `rust-types-and-apis`, `rust-unit-testing`, `rust-errors`,
  `en-gb-oxendict-style`, and `commit-message`.
- [x] (2026-07-03T00:00:00Z) Memtrace discovery failed with
  `mcp__memtrace.list_indexed_repositories -> user cancelled MCP tool call`.
  Per task instructions, this is recorded as a tooling failure and is not a
  product blocker.
- [x] (2026-07-03T08:27:07Z) Retried planning evidence for round 2:
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-3`
  succeeded, `leta show run_request` and related symbol reads succeeded, and
  `leta calls --from run_request --max-depth 2` failed with
  `Error: Failed to start daemon`. The plan uses successful `leta show`
  evidence plus `sem impact` fallback for call relationships.
- [x] (2026-07-03T08:27:07Z) Firecrawl official-doc verification failed with
  `mcp__firecrawl.firecrawl_scrape` for
  `https://docs.rs/rstest/0.26.1/rstest/attr.rstest.html` and
  `https://docs.rs/serial_test/3.5.0/serial_test/attr.serial.html` returning
  `user cancelled MCP tool call`. Locked local crate source is cited instead.
- [x] (2026-07-03T00:00:00Z) Used `sem diff --from origin/main --to HEAD` to
  confirm this planning branch has no code changes before the ExecPlan.
- [x] (2026-07-03T00:00:00Z) Used `sem blame` and `sem impact` on the relevant
  symbols in `src/lib.rs`, `src/roadmap/ops/mod.rs`, `src/roadmap/render.rs`,
  and `src/roadmap/model.rs`.
- [x] (2026-07-03T00:00:00Z) Drafted the first-round ExecPlan.
- [x] (2026-07-03T08:27:07Z) Revised the ExecPlan for planning round 2 to add
  an automated failed in-place dependency-rewrite metrics regression test and a
  strict Work Item 4 placement rule for the single metrics record call.
- [x] (2026-07-03T08:57:13Z) Work Item 1: pinned missing phase-anchor
  behaviour, checkbox marker rendering, successful output-mode metrics, and
  failed in-place dependency-rewrite metrics. Focused green logs:
  `/tmp/test-mapsplice-roadmap-5-1-3-item-1.out`,
  `/tmp/test-render-mapsplice-roadmap-5-1-3-item-1.out`, and
  `/tmp/test-failed-in-place-metrics-mapsplice-roadmap-5-1-3-item-1.out`.
  Deterministic gate logs: `/tmp/make-all-mapsplice-roadmap-5-1-3-item-1.out`,
  `/tmp/markdownlint-mapsplice-roadmap-5-1-3-item-1.out`, and
  `/tmp/nixie-mapsplice-roadmap-5-1-3-item-1.out`. CodeRabbit review was
  requested but deferred with
  `deferred coderabbit review: no default network route visible in this sandbox`
  in `/tmp/coderabbit-mapsplice-roadmap-5-1-3-item-1.out`.
- [x] (2026-07-03T09:04:53Z) Work Item 2: routed phase insert, delete, and
  replace through `find_phase_index`, and made internal append routing use
  `RoadmapFragment::level` directly. Focused green logs:
  `/tmp/test-lookup-mapsplice-roadmap-5-1-3-item-2.out` and
  `/tmp/test-fragment-level-mapsplice-roadmap-5-1-3-item-2.out`. `sem` and
  line-count logs: `/tmp/sem-impact-phase-mapsplice-roadmap-5-1-3-item-2.out`,
  `/tmp/sem-diff-mapsplice-roadmap-5-1-3-item-2.out`, and
  `/tmp/wc-mapsplice-roadmap-5-1-3-item-2.out`. Deterministic gate logs:
  `/tmp/make-all-mapsplice-roadmap-5-1-3-item-2.out`,
  `/tmp/markdownlint-mapsplice-roadmap-5-1-3-item-2.out`, and
  `/tmp/nixie-mapsplice-roadmap-5-1-3-item-2.out`. CodeRabbit review was
  requested but deferred with
  `deferred coderabbit review: no default network route visible in this sandbox`
  in `/tmp/coderabbit-mapsplice-roadmap-5-1-3-item-2.out`.
- [x] (2026-07-03T09:11:51Z) Work Item 3: added the private
  `checkbox_marker` helper and routed task, sub-task, and generic unordered
  list-item rendering through it. Focused green logs:
  `/tmp/test-render-mapsplice-roadmap-5-1-3-item-3.out`,
  `/tmp/test-sub-tasks-mapsplice-roadmap-5-1-3-item-3.out`, and
  `/tmp/test-golden-mapsplice-roadmap-5-1-3-item-3.out`. `sem` and line-count
  logs: `/tmp/sem-impact-render-mapsplice-roadmap-5-1-3-item-3.out`,
  `/tmp/sem-diff-mapsplice-roadmap-5-1-3-item-3.out`, and
  `/tmp/wc-mapsplice-roadmap-5-1-3-item-3.out`. Deterministic gate logs:
  `/tmp/make-all-mapsplice-roadmap-5-1-3-item-3.out`,
  `/tmp/markdownlint-mapsplice-roadmap-5-1-3-item-3.out`, and
  `/tmp/nixie-mapsplice-roadmap-5-1-3-item-3.out`. CodeRabbit review was
  requested but deferred with
  `deferred coderabbit review: no default network route visible in this sandbox`
  in `/tmp/coderabbit-mapsplice-roadmap-5-1-3-item-3.out`.
- [x] (2026-07-03T09:19:40Z) Work Item 4: moved
  `record_dependency_rewrites` to one post-write success path in `run_request`,
  preserving failed in-place metric behaviour. Focused green logs:
  `/tmp/test-metrics-mapsplice-roadmap-5-1-3-item-4.out`,
  `/tmp/test-failed-in-place-metrics-mapsplice-roadmap-5-1-3-item-4.out`,
  `/tmp/test-in-place-mapsplice-roadmap-5-1-3-item-4.out`, and
  `/tmp/test-observability-mapsplice-roadmap-5-1-3-item-4.out`. `sem` and
  line-count logs:
  `/tmp/sem-impact-run-request-mapsplice-roadmap-5-1-3-item-4.out`,
  `/tmp/sem-diff-mapsplice-roadmap-5-1-3-item-4.out`, and
  `/tmp/wc-mapsplice-roadmap-5-1-3-item-4.out`. Deterministic gate logs:
  `/tmp/make-all-mapsplice-roadmap-5-1-3-item-4.out`,
  `/tmp/markdownlint-mapsplice-roadmap-5-1-3-item-4.out`, and
  `/tmp/nixie-mapsplice-roadmap-5-1-3-item-4.out`. CodeRabbit review was
  requested but deferred with
  `deferred coderabbit review: no default network route visible in this sandbox`
  in `/tmp/coderabbit-mapsplice-roadmap-5-1-3-item-4.out`.
- [x] (2026-07-03T09:23:08Z) Work Item 5: marked `docs/roadmap.md` task
  5.1.3 complete and finalized this ExecPlan. Final deterministic gate logs:
  `/tmp/make-all-mapsplice-roadmap-5-1-3-final.out`,
  `/tmp/markdownlint-mapsplice-roadmap-5-1-3-final.out`, and
  `/tmp/nixie-mapsplice-roadmap-5-1-3-final.out`. Final CodeRabbit review was
  requested but deferred with
  `deferred coderabbit review: no default network route visible in this sandbox`
  in `/tmp/coderabbit-mapsplice-roadmap-5-1-3-final.out`.

## Surprises & discoveries

- Observation: Memtrace and Firecrawl were unavailable in this planning
  session, and `leta calls` could not start its daemon, but `leta show` worked
  after workspace setup. Evidence: the exact failed calls are recorded in
  `Progress`; `leta show run_request` returned `src/lib.rs:95-191`. Impact: the
  plan is not blocked; it relies on successful `leta show`, bounded local
  source, locked crate source, and `sem` fallback evidence.
- Observation: the current in-place branch records dependency rewrites only
  after `rewrite_utf8(&request.target, &rendered)?` succeeds. Evidence:
  `src/lib.rs:172-181` shows `record_dependency_rewrites` at line 176, after
  the fallible write at line 175. Impact: the refactor must preserve this
  fail-closed metric behaviour with an automated failed-write test.
- Observation: `tests/roadmap_ops.rs` is already 399 lines. Evidence:
  `wc -l tests/roadmap_ops.rs` returned `399`. Impact: new operation tests
  should be colocated in source modules or placed in a new focused integration
  test module rather than appended to this file.
- Observation: `src/roadmap/render.rs` is 379 lines, leaving little room before
  the 400-line cap. Evidence: `wc -l src/roadmap/render.rs` returned `379`.
  Impact: the checkbox helper must reduce or only very narrowly increase the
  file; the work item must run `wc -l`.
- Observation: `leta workspace add` failed during implementation with
  `Error: IO error: Read-only file system (os error 30)`, but `leta show`
  continued to return symbol bodies for `run_request`, `insert_phases`,
  `delete_anchor`, `replace_anchor`, `append_fragment`, `render_task`,
  `render_sub_task`, and `render_list_item`. Evidence: the failed workspace
  command and successful symbol reads were observed before Work Item 1 edits.
  Impact: implementation used `leta show` where available and bounded local
  file inspection as the documented fallback.
- Observation: generic checked task-list rendering is accepted in preamble
  Markdown but not as an ordinary nested task-body item, where the parser
  treats checked nested items as sub-task candidates. Evidence: the Work Item 1
  red proof first failed with
  `expected a numbered sub-task prefix in "Ordinary checklist body."`; the
  final renderer matrix moved the generic checked list item to preamble
  Markdown. Impact: the test still pins `render_list_item` marker behaviour
  without broadening the roadmap grammar.
- Observation: CodeRabbit review could not run in this sandbox. Evidence:
  `/tmp/coderabbit-mapsplice-roadmap-5-1-3-item-1.out` contains
  `deferred coderabbit review: no default network route visible in this sandbox`.
  Impact: Work Item 1 carries a deferred-review open issue while deterministic
  gates remain green.
- Observation: `find_phase_index` has the expected narrow impact envelope.
  Evidence: `sem impact find_phase_index` reported direct dependents
  `insert_phases`, `delete_anchor`, and `replace_anchor`, with depth-2
  dependents `insert_fragment` and `apply_command`. Impact: Work Item 2 stayed
  within the planned operation-helper seam.
- Observation: `render.rs` line count decreased after extracting
  `checkbox_marker`. Evidence:
  `wc -l src/roadmap/render.rs tests/roadmap_render.rs` returned 375 and 302
  lines. Impact: Work Item 3 stayed comfortably under the 400-line cap while
  reducing repeated marker mapping.
- Observation: two focused Work Item 3 cargo test commands were accidentally
  launched together during local evidence collection. Evidence:
  `/tmp/test-golden-mapsplice-roadmap-5-1-3-item-3.out` recorded
  `Blocking waiting for file lock on build directory`, and both commands
  completed successfully after Cargo serialized access. Impact: no test
  evidence was invalidated, but all subsequent gates were run sequentially by
  `scrutineer`.
- Observation: the single `record_dependency_rewrites` call preserves the
  in-place failure boundary. Evidence: the Work Item 4 focused metrics tests
  passed, including
  `/tmp/test-failed-in-place-metrics-mapsplice-roadmap-5-1-3-item-4.out`, and
  `sem diff` reported only the `run_request` entity changed. Impact: the
  branch-independent success count is now single-sourced without changing the
  failed-write metric contract.

## Decision log

- Decision: keep this task to the four 5.1.3 concepts and do not absorb
  parser fragment-root work from 5.1.2 or mutation-invariant encapsulation from
  5.1.4. Rationale: `docs/roadmap.md` assigns those concerns to separate
  roadmap tasks with their own success criteria. Date/Author: 2026-07-03, Codex.
- Decision: preserve the public `fragment_level` function as a compatibility
  shim while standardizing internal routing on `RoadmapFragment::level`.
  Rationale: `docs/developers-guide.md` section 3 requires public API
  stability, and the existing integration tests import `fragment_level`.
  Date/Author: 2026-07-03, Codex.
- Decision: use concrete private helper functions rather than traits or
  generics for the repeated lookup/rendering logic. Rationale:
  `rust-types-and-apis` favours concrete types until abstraction pressure is
  real, and the repeated code is finite and local. Date/Author: 2026-07-03,
  Codex.
- Decision: use focused `rstest` unit or integration tests for the finite
  behaviour matrix and keep property testing in reserve. Rationale:
  `docs/developers-guide.md` section 6 routes finite parser/model/render
  behaviour to unit tests, while existing property tests already cover
  generated dependency rewrites. Date/Author: 2026-07-03, Codex.
- Decision: do not rely on a new external library or on undocumented upstream
  behaviour. Rationale: this task is internal refactoring and the only external
  facts needed are existing `markdown` mdast fields and `rstest` case syntax,
  both pinned to locked local source. Date/Author: 2026-07-03, Codex.
- Decision: pin the failed in-place dependency-rewrite metric path before
  moving the record call. Rationale: `DEPENDENCY_REWRITES` is a process-global
  atomic counter, so a hoist before `rewrite_utf8` could silently change
  failure metrics while all success-only tests still pass. Date/Author:
  2026-07-03, Codex.
- Decision: model the failed in-place write regression with a capability-scoped
  temporary directory whose permissions are changed through
  `cap_std::fs_utf8::Dir`. Rationale: this keeps the test within the
  repository's capability-filesystem policy while forcing `rewrite_utf8` to
  fail after dependency rewriting is computed. Date/Author:
  2026-07-03T08:57:13Z, Codex.

## Outcomes & retrospective

All five work items are complete. Work Item 1 added regression tests only,
leaving production code untouched. The new tests pin missing phase-anchor
payloads for insert, delete, and replace; checkbox marker rendering for tasks,
sub-tasks, and generic checked list items; matching dependency-rewrite metrics
for stdout and in-place success; and unchanged dependency-rewrite metrics when
an in-place write fails after dependency rewriting has been computed.

Work Item 2 added the private `find_phase_index` helper in
`src/roadmap/ops/mod.rs`, routed phase insert, delete, and replace through it,
and removed the internal `fragment_level` wrapper call from `append_fragment`.
The public `fragment_level` compatibility wrapper remains unchanged.

Work Item 3 added the private `checkbox_marker` helper in
`src/roadmap/render.rs` and routed task, sub-task, and generic unordered
list-item marker rendering through it. Exact rendering remains unchanged under
the focused renderer test, sub-task suite, and golden fixture suite.

Work Item 4 moved dependency-rewrite metric recording in `src/lib.rs` so
`run_request` records the count once after rendering and after any required
in-place write has succeeded. The stdout success, in-place success, failed
in-place, in-place CLI, and observability tests all pass unchanged.

The Work Item 1 red proof temporarily expected the checked task marker to
render as `- [ ] 1.1.1. Checked task.`. The focused renderer command failed in
`/tmp/test-red-render-mapsplice-roadmap-5-1-3-item-1.out`, and the temporary
mutation was reverted before the green focused run.

Deterministic gates are green through final close-out. CodeRabbit did not
return a review for any work item because the sandbox has no default network
route, so those reviews remain deferred for a network-capable environment.
`docs/roadmap.md` now marks task 5.1.3 complete.

## Context and orientation

The application boundary lives in `src/lib.rs`. `run_request` reads the target
roadmap, parses the optional fragment, applies the roadmap operation, renders
the result, and either writes in place or returns stdout. Before Work Item 4 it
called `observability::record_dependency_rewrites(dependency_rewrites)` in both
the in-place and stdout branches; the final implementation records that count
once after any required in-place write succeeds.

Roadmap mutation lives in `src/roadmap/ops/mod.rs`. `insert_phases`,
`delete_anchor`, and `replace_anchor` all search `roadmap.phases` by phase
number with the same `iter().position(...)` pattern. The step and task paths
already have `find_step_parent_mut` and `find_task_parent_mut` helpers, so
phase lookup is the outlier.

Fragment-level routing is split between `RoadmapFragment::level` in
`src/roadmap/model.rs` and the public `fragment_level` wrapper. Internal
operation code currently uses both forms. This plan keeps the public wrapper
for compatibility but makes internal routing consistently use the method.

Rendering lives in `src/roadmap/render.rs`. `render_task` and `render_sub_task`
each map `Option<bool>` to `"[x] "`, `"[ ] "`, or `""`. `render_list_item`
repeats the same marker mapping while adding the leading `"- "` prefix for
generic unordered list items. A small private helper can own the checkbox
marker string while callers keep their own list prefix.

The relevant tests are:

- `tests/roadmap_parse.rs` for public fragment-level queries and parse
  diagnostics;
- `tests/roadmap_render.rs` for renderer behaviour and preservation;
- `tests/roadmap_sub_tasks.rs` for sub-task ordering and nested rendering;
- `tests/roadmap_ops.rs` for CLI operation behaviour, although this file is
  already at the line cap and should not receive new tests.

## Research and verified facts

Memtrace canonical search was requested first but the MCP host returned
`user cancelled MCP tool call`. The repository id `mapsplice` could not be
confirmed by `list_indexed_repositories` in this session. This plan therefore
uses branch-local source and `sem` evidence as allowed by the task.

Leta was requested before source exploration. In round 2,
`leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-3`
succeeded and `leta show` returned bounded symbol definitions. `leta calls`
failed with `Error: Failed to start daemon`, so call-relationship evidence uses
`sem impact` fallback.

Firecrawl was requested for official docs.rs verification but the host returned
`user cancelled MCP tool call`. The plan does not rely on network-only API
claims; it cites locked crate source from the local Cargo registry.

Load-bearing local source evidence:

- `src/roadmap/ops/mod.rs:102-164` contains `append_fragment`,
  `insert_fragment`, and `insert_phases`; `append_fragment` uses
  `fragment_level`, while `insert_fragment` uses `.level()`.
- `src/roadmap/ops/mod.rs:194-257` repeats phase lookup in the phase arms of
  `delete_anchor` and `replace_anchor`.
- `src/roadmap/render.rs:70-75` and `src/roadmap/render.rs:105-110` repeat
  task and sub-task checkbox marker mapping.
- `src/roadmap/render.rs:262-270` repeats checkbox mapping for generic
  unordered list items.
- `src/lib.rs:172-181` records dependency rewrites in both terminal branches
  of `run_request`.
- `src/lib.rs:172-181` records dependency rewrites in the in-place branch only
  after `rewrite_utf8(&request.target, &rendered)?` succeeds, so failed
  in-place writes currently record no dependency-rewrite count.
- `src/roadmap/model.rs:212-227` defines `RoadmapFragment::level` and the
  public `fragment_level` wrapper.
- `Cargo.lock:943-946` resolves `markdown` to version 1.0.0.
- `Cargo.lock:1363-1366` resolves `rstest` to version 0.26.1.
- `Cargo.lock:1635-1646` resolves `serial_test` to version 3.5.0.
- Locked source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/mdast.rs:704-718`
  defines `ListItem.children` and `ListItem.checked: Option<bool>`.
- Locked source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rstest-0.26.1/src/lib.rs:164-190`
  documents `#[rstest]` with `#[case]` table tests.
- Locked source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rstest-0.26.1/src/lib.rs:805-815`
  documents named cases such as `#[case::zero_base_case(...)]`.
- Locked source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serial_test-3.5.0/src/lib.rs:6-31`
  documents `#[serial]`, keyed serial groups, and the guarantee that tests with
  `#[serial]` execute serially, with no ordering guarantee.

`sem impact run_request` reported dependencies on `operation_from_command`,
`read_utf8`, `load_fragment`, `render_roadmap`, `rewrite_utf8`,
`record_dependency_rewrites`, `record_in_place_rewrite`, and `RunOutcome`, and
reported no other affected entities.
`sem diff --from origin/main --to HEAD --format json` reported the current
planning-only ExecPlan change and no code changes.

## Interfaces and dependencies

No new external dependency is allowed or needed.

Use these internal helper shapes unless the implementation discovers a tighter
name that preserves the same boundaries:

```rust
fn find_phase_index(roadmap: &RoadmapDocument, target: PhaseNumber) -> Result<usize>;
```

The helper belongs in `src/roadmap/ops/mod.rs` near `find_step_parent_mut`. It
returns the index used by `Vec::splice` or `Vec::remove` and must preserve
`MapspliceError::AnchorNotFound` with the phase anchor payload.

```rust
const fn checkbox_marker(checked: Option<bool>) -> &'static str;
```

The helper belongs in `src/roadmap/render.rs`. It returns the marker fragment
without the leading list bullet: `"[x] "`, `"[ ] "`, or `""`. `render_task` and
`render_sub_task` interpolate it after their own `"- "` prefix.
`render_list_item` keeps ownership of ordered-list ordinals and the leading
`"- "` prefix for unordered lists.

For fragment-level routing, prefer the existing method:

```rust
impl RoadmapFragment {
    pub const fn level(&self) -> RoadmapItemLevel;
}
```

Keep the public wrapper as:

```rust
pub const fn fragment_level(fragment: &RoadmapFragment) -> RoadmapItemLevel {
    fragment.level()
}
```

Do not remove the wrapper during this task.

For metrics recording, preserve `observability::record_dependency_rewrites`.
The acceptable refactor is to compute and render first, then record the
branch-independent dependency rewrite count once at a point that preserves
current success/failure semantics.

## Plan of work

### Work Item 1: Pin unchanged lookup, rendering, and metric behaviour

Docs and skills to read before editing: `AGENTS.md`; `docs/roadmap.md` section
5.1.3; `docs/mapsplice-design.md` sections 2, 4, 5, 6, and 8;
`docs/developers-guide.md` sections 2, 3, and 6; `docs/users-guide.md`, "The
roadmap shape `mapsplice` expects", "Output modes", and "Validation rules and
failure cases"; `docs/documentation-style-guide.md`; `execplans`; `leta`;
`rust-router`; `rust-unit-testing`; `rust-errors`; `rust-types-and-apis`; `sem`;
`en-gb-oxendict-style`; `commit-message`.

Add focused tests that pass on current behaviour before production refactors:

- Phase lookup: pin missing phase anchors for insert, delete, and replace in a
  new focused integration test module, `tests/roadmap_lookup_rendering.rs`.
  Because `tests/roadmap_ops.rs` is at 399 lines, do not append these tests
  there. Use existing public APIs where practical and assert
  `MapspliceError::AnchorNotFound` with the expected phase anchor.
- Fragment-level routing: keep the existing
  `parse_fragment_detects_supported_level` coverage in
  `tests/roadmap_parse.rs`; add no duplicate test unless the implementation
  changes the public wrapper.
- Checkbox markers: add a focused `rstest` matrix for checked task,
  unchecked task, checked sub-task, unchecked sub-task, and ordinary unordered
  nested list item rendering. Prefer `tests/roadmap_render.rs` unless the file
  approaches 400 lines after the edit.
- Metrics: add a serial test that compares `metrics_snapshot` before and after
  a stdout operation that rewrites one dependency and an in-place operation
  that rewrites one dependency. The dependency rewrite counter must increase by
  the same count in both modes; the in-place counter must increase only for the
  in-place mode.
- Failed in-place metrics: add a focused `#[serial_test::serial(cli_env)]`
  test in `tests/roadmap_lookup_rendering.rs` that runs an in-place operation
  which rewrites at least one dependency during the roadmap operation but makes
  `rewrite_utf8` fail, then asserts `metrics_snapshot().dependency_rewrites` is
  unchanged. One path-safe way is to make the target file read-only, or to
  place the target in a directory whose permissions cause the sibling temporary
  write or rename in `src/fs.rs::rewrite_utf8` to fail. The test must restore
  permissions before the temporary directory is dropped. This test pins
  `docs/mapsplice-design.md` F5 fail-closed behaviour and the acceptance
  criterion that `run_request` records without changing failure metrics.

The red proof for this item is a temporary assertion mutation, such as
expecting a wrong checkbox marker or wrong dependency-rewrite delta. Run the
focused test, confirm it fails for the intended reason, then revert the
temporary mutation before committing.

This item is independently committable because it adds behaviour locks without
production-code changes.

Validation for this item:

```bash
cargo test --test roadmap_lookup_rendering -- --nocapture 2>&1 | tee /tmp/test-mapsplice-roadmap-5-1-3-item-1.out
cargo test --test roadmap_render checkbox -- --nocapture 2>&1 | tee /tmp/test-render-mapsplice-roadmap-5-1-3-item-1.out
RUST_TEST_THREADS=1 cargo test --test roadmap_lookup_rendering failed_in_place -- --nocapture 2>&1 | tee /tmp/test-failed-in-place-metrics-mapsplice-roadmap-5-1-3-item-1.out
wc -l src/lib.rs src/roadmap/ops/mod.rs src/roadmap/render.rs \
  tests/roadmap_lookup_rendering.rs tests/roadmap_render.rs \
  2>&1 | tee /tmp/wc-mapsplice-roadmap-5-1-3-item-1.out
MARKDOWN_PATHS='docs/execplans/roadmap-5-1-3.md' make markdownfmt 2>&1 | tee /tmp/markdownfmt-mapsplice-roadmap-5-1-3-item-1.out
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-5-1-3-item-1.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-3-item-1.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-3-item-1.out
```

Commit after all gates pass.

### Work Item 2: Single-source phase lookup and fragment-level routing

Docs and skills to read before editing: all Work Item 1 docs and skills, plus
the `rust-types-and-apis` helper-shape guidance.

In `src/roadmap/ops/mod.rs`, add a private `find_phase_index` helper near
`find_step_parent_mut`. Route phase insert, phase delete, and phase replace
through that helper. The helper should accept `&RoadmapDocument` because the
callers only need an index; mutable borrowing remains local to the splice or
remove operation.

Also remove the internal `fragment_level` import from `src/roadmap/ops/mod.rs`
and call `fragment_document.level()` in `append_fragment`, matching
`insert_fragment` and `replace_anchor`. Keep
`src/roadmap/model.rs::fragment_level` as the public compatibility wrapper.

Do not change operation matching, fragment level validation, error variants, or
public exports. Run `sem impact find_phase_index` after adding the helper and
`sem diff --format json` before committing to confirm the entity-level change
is narrow.

This item is independently committable because Work Item 1 already pins the
missing-anchor and fragment-level behaviour.

Validation for this item:

```bash
cargo test --test roadmap_lookup_rendering phase -- --nocapture 2>&1 | tee /tmp/test-lookup-mapsplice-roadmap-5-1-3-item-2.out
cargo test --test roadmap_parse parse_fragment_detects_supported_level -- --nocapture 2>&1 | tee /tmp/test-fragment-level-mapsplice-roadmap-5-1-3-item-2.out
sem impact find_phase_index 2>&1 | tee /tmp/sem-impact-phase-mapsplice-roadmap-5-1-3-item-2.out
sem diff --format json 2>&1 | tee /tmp/sem-diff-mapsplice-roadmap-5-1-3-item-2.out
wc -l src/roadmap/ops/mod.rs src/roadmap/model.rs 2>&1 | tee /tmp/wc-mapsplice-roadmap-5-1-3-item-2.out
MARKDOWN_PATHS='docs/execplans/roadmap-5-1-3.md' make markdownfmt 2>&1 | tee /tmp/markdownfmt-mapsplice-roadmap-5-1-3-item-2.out
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-5-1-3-item-2.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-3-item-2.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-3-item-2.out
```

Commit after all gates pass.

### Work Item 3: Single-source checkbox marker rendering

Docs and skills to read before editing: all Work Item 1 docs and skills, plus
`rust-types-and-apis` for the private helper boundary and `rust-unit-testing`
for table-test shape.

In `src/roadmap/render.rs`, add a private `const fn checkbox_marker` that maps
`Option<bool>` to the marker fragment. Replace the repeated matches in
`render_task`, `render_sub_task`, and the unordered-list branch of
`render_list_item`. Keep `render_list_item` responsible for ordered-list
ordinal prefixes and for adding the leading `"- "` to unordered list items.

Do not alter `render_item_summary`, `render_nested_body`,
`render_markdown_nodes`, or preservation behaviour. Run the focused renderer
tests and the golden fixture suite because checkbox spacing is part of the
exact Markdown contract.

This item is independently committable because Work Item 1 already pins marker
output before the helper extraction.

Validation for this item:

```bash
cargo test --test roadmap_render checkbox -- --nocapture 2>&1 | tee /tmp/test-render-mapsplice-roadmap-5-1-3-item-3.out
cargo test --test roadmap_sub_tasks -- --nocapture 2>&1 | tee /tmp/test-sub-tasks-mapsplice-roadmap-5-1-3-item-3.out
cargo test --test roadmap_golden -- --nocapture 2>&1 | tee /tmp/test-golden-mapsplice-roadmap-5-1-3-item-3.out
sem impact render_task 2>&1 | tee /tmp/sem-impact-render-mapsplice-roadmap-5-1-3-item-3.out
sem diff --format json 2>&1 | tee /tmp/sem-diff-mapsplice-roadmap-5-1-3-item-3.out
wc -l src/roadmap/render.rs tests/roadmap_render.rs 2>&1 | tee /tmp/wc-mapsplice-roadmap-5-1-3-item-3.out
MARKDOWN_PATHS='docs/execplans/roadmap-5-1-3.md' make markdownfmt 2>&1 | tee /tmp/markdownfmt-mapsplice-roadmap-5-1-3-item-3.out
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-5-1-3-item-3.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-3-item-3.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-3-item-3.out
```

Commit after all gates pass.

### Work Item 4: Single-source dependency-rewrite metric recording

Docs and skills to read before editing: `AGENTS.md`; `docs/roadmap.md` section
5.1.3; `docs/mapsplice-design.md` sections 5 and 6; `docs/developers-guide.md`
sections 2, 5, and 6; `docs/users-guide.md`, "Output modes"; `execplans`;
`leta`; `rust-router`; `rust-errors`; `rust-unit-testing`; `sem`;
`en-gb-oxendict-style`; `commit-message`.

In `src/lib.rs::run_request`, remove the duplicated
`observability::record_dependency_rewrites(dependency_rewrites)` calls from the
terminal branches. Record the count once, but place that single call strictly
after the in-place write branch has completed any required
`rewrite_utf8(&request.target, &rendered)?` successfully. Never record once
after render but before the output-mode branch; that would break the
fail-closed failure-metric contract pinned by Work Item 1.

The required semantics are:

- stdout success records after render succeeds;
- in-place success records after `rewrite_utf8` succeeds;
- failed render or failed in-place rewrite does not record a dependency rewrite
  count, even when `apply_command_inner` already computed a non-zero rewrite
  count.

One acceptable shape is to keep the in-place write inside the branch, then call
`record_dependency_rewrites` once after any required write succeeds and before
constructing the `RunOutcome`. If that shape complicates ownership of
`request.target`, use a small private helper or local `outcome` variable, but
do not add a public API.

The success metric test and failed in-place metric test from Work Item 1 must
pass unchanged. Also preserve the existing observability unit tests and
in-place CLI tests.

This item is independently committable because it changes only
branch-independent recording location while tests pin both output modes.

Validation for this item:

```bash
cargo test --test roadmap_lookup_rendering metrics -- --nocapture 2>&1 | tee /tmp/test-metrics-mapsplice-roadmap-5-1-3-item-4.out
RUST_TEST_THREADS=1 cargo test --test roadmap_lookup_rendering failed_in_place -- --nocapture 2>&1 | tee /tmp/test-failed-in-place-metrics-mapsplice-roadmap-5-1-3-item-4.out
cargo test --test roadmap_ops in_place -- --nocapture 2>&1 | tee /tmp/test-in-place-mapsplice-roadmap-5-1-3-item-4.out
cargo test observability -- --nocapture 2>&1 | tee /tmp/test-observability-mapsplice-roadmap-5-1-3-item-4.out
sem impact run_request 2>&1 | tee /tmp/sem-impact-run-request-mapsplice-roadmap-5-1-3-item-4.out
sem diff --format json 2>&1 | tee /tmp/sem-diff-mapsplice-roadmap-5-1-3-item-4.out
wc -l src/lib.rs 2>&1 | tee /tmp/wc-mapsplice-roadmap-5-1-3-item-4.out
MARKDOWN_PATHS='docs/execplans/roadmap-5-1-3.md' make markdownfmt 2>&1 | tee /tmp/markdownfmt-mapsplice-roadmap-5-1-3-item-4.out
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-5-1-3-item-4.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-3-item-4.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-3-item-4.out
```

Commit after all gates pass.

### Work Item 5: Close roadmap task 5.1.3 and finalize documentation

Docs and skills to read before editing: `docs/roadmap.md` section 5.1.3;
`docs/documentation-style-guide.md`; `AGENTS.md`; `execplans`;
`en-gb-oxendict-style`; `sem`; `commit-message`.

Mark `docs/roadmap.md` task 5.1.3 complete only after Work Items 1-4 have
passed and have been committed. Update this ExecPlan's `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` with
the actual validation logs, advisory-tool status, line-count evidence, and any
deviations. Set `Status: COMPLETE` only after all final gates pass.

Format only the changed Markdown paths. If only this ExecPlan and the roadmap
changed, run the exact command below. If another Markdown file was changed in
this work item, add only that existing path to `MARKDOWN_PATHS`.

```bash
MARKDOWN_PATHS='docs/execplans/roadmap-5-1-3.md docs/roadmap.md' make markdownfmt 2>&1 | tee /tmp/markdownfmt-mapsplice-roadmap-5-1-3-final.out
```

Then run final gates:

```bash
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-5-1-3-final.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-3-final.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-3-final.out
```

Commit after all gates pass.

## Concrete steps

1. Start each implementation session in the assigned worktree:

   ```bash
   cd /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-3
   git branch --show-current
   git status --short
   ```

   Expected branch output:

   ```plaintext
   roadmap-5-1-3
   ```

2. Retry advisory tools before editing:

   ```bash
   leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-3
   sem diff --from origin/main --to HEAD --format json
   sem blame src/roadmap/ops/mod.rs
   ```

   Also retry Memtrace MCP with `list_indexed_repositories`. If Memtrace still
   returns `user cancelled MCP tool call`, record it in this plan and continue
   with local evidence.

3. Perform Work Items 1-5 in order. Do not combine work-item commits.

4. Use file-based commit messages:

   ```bash
   COMMIT_MSG_DIR="$(mktemp -d)"
   ${EDITOR:-vi} "$COMMIT_MSG_DIR/COMMIT_MSG.md"
   git commit -F "$COMMIT_MSG_DIR/COMMIT_MSG.md"
   rm -rf "$COMMIT_MSG_DIR"
   ```

5. Keep `docs/execplans/roadmap-5-1-3.md` current after every work item.

## Validation and acceptance

The accepted end state is:

- phase insert, delete, and replace use one phase lookup helper;
- internal fragment-level routing uses `RoadmapFragment::level`, while the
  public `fragment_level` wrapper remains available and tested;
- task, sub-task, and generic unordered list rendering use one checkbox marker
  helper;
- `run_request` records dependency rewrite counts through one
  branch-independent path without changing success or failure metrics,
  including the failed in-place write path;
- public CLI output, rendered Markdown, diagnostics, and public APIs are
  unchanged;
- `docs/roadmap.md` marks task 5.1.3 complete only after the code gates pass;
- all changed Rust source files remain under 400 lines;
- `make all`, `make markdownlint`, and `make nixie` pass.

Red-Green-Refactor evidence required by this plan:

- Work Item 1 records the red mutation and green focused test log paths.
- Work Item 1 records a focused failed in-place dependency-rewrite metric test
  that fails if `record_dependency_rewrites` is moved before `rewrite_utf8`
  succeeds.
- Work Items 2-4 each record the focused green test log, `sem diff`, line
  count evidence, and full gate logs.
- Work Item 5 records final `make all`, `make markdownlint`, and `make nixie`
  logs.

Quality criteria:

- Tests: focused tests listed per work item, existing behavioural and golden
  suites through `make all`.
- Lint/typecheck: `make all`, which includes `check-fmt`, `lint`,
  `typecheck`, and `test` on current `origin/main`.
- Documentation: `make markdownlint` and `make nixie` for Markdown changes.
- Performance: no benchmark required; helper extraction must not introduce
  extra allocations in the phase lookup path beyond existing iterator scans.
- Security: no new filesystem, network, or unsafe code surface.

Quality method:

```bash
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-5-1-3-final.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-3-final.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-3-final.out
```

## Idempotence and recovery

All planned edits are ordinary source and Markdown edits. If a focused test
fails after a temporary red mutation, revert only the temporary mutation and
rerun the focused command. If a production refactor fails, use `sem diff` and
`git diff` to inspect the current worktree, then repair forward without
resetting unrelated user work.

If a scoped Markdown formatter changes files outside the named
`MARKDOWN_PATHS`, stop and inspect before committing. If unrelated formatter
churn must be parked, use a named stash only:

```bash
git stash push -m 'df12-stash v1 task=5.1.3 kind=discard reason="unrelated formatter churn"' -- <paths>
```

Do not use a bare `git stash`.

## Artefacts and notes

Planning artefacts gathered in this round:

```plaintext
mcp__memtrace.list_indexed_repositories -> user cancelled MCP tool call
leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-3 -> Added workspace
leta calls --from run_request --max-depth 2 -> Error: Failed to start daemon
mcp__firecrawl.firecrawl_scrape https://docs.rs/rstest/0.26.1/rstest/attr.rstest.html
  -> user cancelled MCP tool call
mcp__firecrawl.firecrawl_scrape https://docs.rs/serial_test/3.5.0/serial_test/attr.serial.html
  -> user cancelled MCP tool call
sem impact run_request -> no entities affected by changes to this entity
```

Relevant file line counts before implementation:

```plaintext
259 src/lib.rs
316 src/roadmap/ops/mod.rs
379 src/roadmap/render.rs
256 src/roadmap/model.rs
399 tests/roadmap_ops.rs
262 tests/roadmap_render.rs
217 tests/roadmap_parse.rs
```

Work Item 1 implementation artefacts:

```plaintext
cargo test --test roadmap_render checkbox -- --nocapture
  -> /tmp/test-render-mapsplice-roadmap-5-1-3-item-1.out, 5 passed
cargo test --test roadmap_lookup_rendering -- --nocapture
  -> /tmp/test-mapsplice-roadmap-5-1-3-item-1.out, 5 passed
RUST_TEST_THREADS=1 cargo test --test roadmap_lookup_rendering failed_in_place -- --nocapture
  -> /tmp/test-failed-in-place-metrics-mapsplice-roadmap-5-1-3-item-1.out, 1 passed
wc -l src/lib.rs src/roadmap/ops/mod.rs src/roadmap/render.rs \
  tests/roadmap_lookup_rendering.rs tests/roadmap_render.rs
  -> 259, 316, 379, 197, and 302 lines respectively
make all
  -> /tmp/make-all-mapsplice-roadmap-5-1-3-item-1.out, 229 tests passed
make markdownlint
  -> /tmp/markdownlint-mapsplice-roadmap-5-1-3-item-1.out, 0 errors
make nixie
  -> /tmp/nixie-mapsplice-roadmap-5-1-3-item-1.out, passed
coderabbit-review-agent
  -> /tmp/coderabbit-mapsplice-roadmap-5-1-3-item-1.out, deferred:
     no default network route visible in this sandbox
```

Work Item 2 implementation artefacts:

```plaintext
cargo test --test roadmap_lookup_rendering phase -- --nocapture
  -> /tmp/test-lookup-mapsplice-roadmap-5-1-3-item-2.out, 3 passed
cargo test --test roadmap_parse parse_fragment_detects_supported_level -- --nocapture
  -> /tmp/test-fragment-level-mapsplice-roadmap-5-1-3-item-2.out, 4 passed
sem impact find_phase_index
  -> /tmp/sem-impact-phase-mapsplice-roadmap-5-1-3-item-2.out, direct
     dependents insert_phases, delete_anchor, and replace_anchor
sem diff --format json
  -> /tmp/sem-diff-mapsplice-roadmap-5-1-3-item-2.out, one helper added and
     four operation functions modified
wc -l src/roadmap/ops/mod.rs src/roadmap/model.rs
  -> 311 and 256 lines respectively
make all
  -> /tmp/make-all-mapsplice-roadmap-5-1-3-item-2.out, passed
make markdownlint
  -> /tmp/markdownlint-mapsplice-roadmap-5-1-3-item-2.out, 0 errors
make nixie
  -> /tmp/nixie-mapsplice-roadmap-5-1-3-item-2.out, passed
coderabbit-review-agent
  -> /tmp/coderabbit-mapsplice-roadmap-5-1-3-item-2.out, deferred:
     no default network route visible in this sandbox
```

Work Item 3 implementation artefacts:

```plaintext
cargo test --test roadmap_render checkbox -- --nocapture
  -> /tmp/test-render-mapsplice-roadmap-5-1-3-item-3.out, 5 passed
cargo test --test roadmap_sub_tasks -- --nocapture
  -> /tmp/test-sub-tasks-mapsplice-roadmap-5-1-3-item-3.out, 9 passed
cargo test --test roadmap_golden -- --nocapture
  -> /tmp/test-golden-mapsplice-roadmap-5-1-3-item-3.out, 64 passed
sem impact render_task
  -> /tmp/sem-impact-render-mapsplice-roadmap-5-1-3-item-3.out, direct
     dependent render_tasks and depth-2 dependent render_roadmap
sem diff --format json
  -> /tmp/sem-diff-mapsplice-roadmap-5-1-3-item-3.out, checkbox_marker added
     and three renderer functions modified
wc -l src/roadmap/render.rs tests/roadmap_render.rs
  -> 375 and 302 lines respectively
make all
  -> /tmp/make-all-mapsplice-roadmap-5-1-3-item-3.out, passed
make markdownlint
  -> /tmp/markdownlint-mapsplice-roadmap-5-1-3-item-3.out, 0 errors
make nixie
  -> /tmp/nixie-mapsplice-roadmap-5-1-3-item-3.out, passed
coderabbit-review-agent
  -> /tmp/coderabbit-mapsplice-roadmap-5-1-3-item-3.out, deferred:
     no default network route visible in this sandbox
```

Work Item 4 implementation artefacts:

```plaintext
cargo test --test roadmap_lookup_rendering metrics -- --nocapture
  -> /tmp/test-metrics-mapsplice-roadmap-5-1-3-item-4.out, 1 passed
RUST_TEST_THREADS=1 cargo test --test roadmap_lookup_rendering failed_in_place -- --nocapture
  -> /tmp/test-failed-in-place-metrics-mapsplice-roadmap-5-1-3-item-4.out, 1 passed
cargo test --test roadmap_ops in_place -- --nocapture
  -> /tmp/test-in-place-mapsplice-roadmap-5-1-3-item-4.out, 1 passed
cargo test observability -- --nocapture
  -> /tmp/test-observability-mapsplice-roadmap-5-1-3-item-4.out, passed
sem impact run_request
  -> /tmp/sem-impact-run-request-mapsplice-roadmap-5-1-3-item-4.out, direct
     dependent failed_in_place_rewrite_does_not_record_dependency_rewrites
sem diff --format json
  -> /tmp/sem-diff-mapsplice-roadmap-5-1-3-item-4.out, run_request modified
wc -l src/lib.rs
  -> 261 lines
make all
  -> /tmp/make-all-mapsplice-roadmap-5-1-3-item-4.out, passed
make markdownlint
  -> /tmp/markdownlint-mapsplice-roadmap-5-1-3-item-4.out, 0 errors
make nixie
  -> /tmp/nixie-mapsplice-roadmap-5-1-3-item-4.out, passed
coderabbit-review-agent
  -> /tmp/coderabbit-mapsplice-roadmap-5-1-3-item-4.out, deferred:
     no default network route visible in this sandbox
```

Work Item 5 final artefacts:

```plaintext
MARKDOWN_PATHS='docs/execplans/roadmap-5-1-3.md docs/roadmap.md' make markdownfmt
  -> /tmp/markdownfmt-mapsplice-roadmap-5-1-3-final.out, 0 errors
make all
  -> /tmp/make-all-mapsplice-roadmap-5-1-3-final.out, passed
make markdownlint
  -> /tmp/markdownlint-mapsplice-roadmap-5-1-3-final.out, 0 errors
make nixie
  -> /tmp/nixie-mapsplice-roadmap-5-1-3-final.out, passed
coderabbit-review-agent
  -> /tmp/coderabbit-mapsplice-roadmap-5-1-3-final.out, deferred:
     no default network route visible in this sandbox
```

## Revision note

- Initial planning round created this ExecPlan for roadmap task 5.1.3.
- Round 2 resolved the design-review blocker by adding a focused failed
  in-place dependency-rewrite metrics test to Work Item 1 and by making Work
  Item 4 place the single `record_dependency_rewrites` call strictly after a
  successful in-place `rewrite_utf8`.
- The plan records Memtrace and Firecrawl unavailability, partial Leta
  availability, and proceeds with bounded `leta show`, local source, locked
  crate source, and `sem` evidence.
- Work Item 1 implemented the behaviour locks and recorded the deferred
  CodeRabbit review caused by missing sandbox network routing.
- Work Item 2 implemented the phase lookup helper and internal fragment-level
  routing cleanup. CodeRabbit review was again deferred by missing sandbox
  network routing.
- Work Item 3 implemented the checkbox marker helper and recorded the local
  evidence-collection command-launch deviation. CodeRabbit review was again
  deferred by missing sandbox network routing.
- Work Item 4 implemented single-source dependency-rewrite metric recording in
  `run_request`. CodeRabbit review was again deferred by missing sandbox
  network routing.
- Work Item 5 marked roadmap task 5.1.3 complete and set this ExecPlan to
  COMPLETE after final deterministic gates. CodeRabbit review was again
  deferred by missing sandbox network routing.
