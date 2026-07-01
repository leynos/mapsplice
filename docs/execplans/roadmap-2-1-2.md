# Prove sub-tasks renumber with their parent

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

## Purpose / big picture

Roadmap task 2.1.2 is complete when an addendum sub-task tracks its parent
task through a structural renumber, and dependency references to that
sub-task are rewritten to the new fourth-level number. The observable success
case from `docs/roadmap.md` is precise: when parent task `8.2.3` moves to
`9.2.3`, sub-task `8.2.3.1` moves to `9.2.3.1`, and `Requires 8.2.3.1`
references become `Requires 9.2.3.1`.

Planning evidence shows that the production mechanism already exists on this
branch: `src/roadmap/ops/rewrite.rs::renumber_document` calls
`renumber_sub_tasks`, inserts each original sub-task anchor into
`RenumberPlan`, and `rewrite_task_entry` descends into
`rewrite_sub_task_entry` so dependency text inside sub-task summaries and
bodies is rewritten. The remaining delivery work is therefore not a broad
rewrite. It is to pin the exact roadmap success case with a behavioural test,
then mark only roadmap item 2.1.2 complete after the proof passes.

## Constraints

- Work only in `/home/leynos/Projects/mapsplice.worktrees/roadmap-2-1-2`.
- Do not edit the root/control worktree.
- Use absolute paths for every `apply_patch` hunk if this plan is implemented
  by an agent using that tool.
- Treat `origin/main` as canonical and re-check branch skew before editing.
- Use Memtrace as the primary canonical-main search tool. First call
  `list_indexed_repositories`; proceed with repo id `mapsplice` only if that
  call confirms it. If the MCP host cancels or fails, record the failure and
  verify branch-local facts with `leta`, `sem`, exact text search, and bounded
  file inspection.
- Use `leta` for branch-local symbol navigation and references when the daemon
  starts. If it fails, record the exact command and error before falling back
  to bounded local source inspection.
- Use `sem` for codebase history navigation and entity-level diff review.
- Use `docs/mapsplice-design.md`, `docs/developers-guide.md`,
  `docs/users-guide.md`, `docs/contributing.md`,
  `docs/documentation-style-guide.md`, `docs/scripting-standards.md`,
  `AGENTS.md`, and `docs/roadmap.md` as source-of-truth documents.
- Follow en-GB Oxford spelling in prose, comments, and commit messages.
- Do not add a new external dependency for roadmap task 2.1.2.
- Do not change public library APIs unless the exact behavioural test proves
  the current private implementation cannot satisfy the contract.
- Commit after each change and gate each commit.
- Format only changed Markdown files with path-safe commands. Do not run
  repository-global Markdown formatting such as `make fmt` or
  `mdformat-all`.
- Every test, lint, format check, and gate command must be logged with `tee`
  to a branch-specific `/tmp` file.

## Tolerances (exception triggers)

- If `sem diff --from origin/main --to HEAD --format json` reports an
  unexpected semantic code delta before implementation, inspect it and update
  this plan before editing.
- If Memtrace or `leta` remains unavailable, do not mark the plan blocked.
  Record the command failure in `Surprises & Discoveries` and continue with
  bounded local source, docs, and tests.
- If adding the exact success-case test requires touching more than
  `tests/roadmap_sub_tasks.rs` and one helper under `tests/support/`, stop and
  update this plan before editing production code.
- If the exact success-case test fails and the fix requires changing more than
  `src/roadmap/ops/rewrite.rs` plus the new test, stop and update this plan
  before broadening scope.
- If satisfying task 2.1.2 requires changing the accepted grammar in
  `docs/mapsplice-design.md` section 4 or `docs/users-guide.md` section
  "The roadmap shape `mapsplice` expects", stop and escalate.
- If a work item touches Rust code beyond tests, load `rust-router` again and
  follow the smallest routed skill before editing.
- If `make all` fails after two focused fix attempts, record the failing
  command and log path in `Decision Log` and stop for review.
- If formatter churn touches files outside the work item, park or discard it
  with a named stash following the df12 stash format before proceeding.

## Risks

- Risk: The current code appears to implement the required behaviour, but the
  existing tests do not mirror the exact roadmap success case.
  Severity: medium.
  Likelihood: high.
  Mitigation: work item 1 adds a named behavioural regression that exercises
  `8.2.3` to `9.2.3` and checks both the sub-task number and a reference to
  the sub-task.

- Risk: A broad production change could duplicate working renumber logic.
  Severity: medium.
  Likelihood: medium.
  Mitigation: no production code edit is planned unless the exact test fails;
  source inspection already shows the intended mechanism in
  `src/roadmap/ops/rewrite.rs`.

- Risk: A roadmap update could imply task 2.1.3 is complete.
  Severity: medium.
  Likelihood: medium.
  Mitigation: work item 2 marks only `docs/roadmap.md` task 2.1.2 complete.
  It must leave 2.1.3 unchecked.

- Risk: Advisory tools can fail in the sandbox.
  Severity: medium.
  Likelihood: observed.
  Mitigation: this plan records the Memtrace, Firecrawl, and `leta` failures
  already observed and permits bounded local fallback evidence.

## Progress

- [x] (2026-07-01T10:58:57Z) Read `AGENTS.md` and confirmed the branch is
  `roadmap-2-1-2`, so this plan belongs at
  `docs/execplans/roadmap-2-1-2.md`.
- [x] (2026-07-01T10:58:57Z) Loaded `execplans`, `leta`, `sem`,
  `rust-router`, `rust-unit-testing`, and `firecrawl-mcp` for this planning
  pass.
- [x] (2026-07-01T10:58:57Z) Read the source-of-truth docs needed for this
  task: `docs/roadmap.md`, `docs/mapsplice-design.md`,
  `docs/developers-guide.md`, `docs/users-guide.md`,
  `docs/contributing.md`, `docs/documentation-style-guide.md`,
  `docs/scripting-standards.md`, and `AGENTS.md`.
- [x] (2026-07-01T10:58:57Z) Verified branch-local code and tests around
  sub-task renumbering with bounded local inspection after Memtrace and `leta`
  were unavailable.
- [x] (2026-07-01T11:33:56Z) Work item 1: Added the exact
  `8.2.3` to `9.2.3` parent/sub-task renumber proof in
  `tests/roadmap_sub_tasks.rs`; the focused test and `make all` passed.
- [x] (2026-07-01T11:36:36Z) Committed work item 1 as
  `0796ff8b7b8a9a9e7756d910f158ae9bbf11fc08` after `make all`,
  `make markdownlint`, and a retried `make nixie` passed.
- [x] (2026-07-01T11:36:36Z) Work item 2: Marked only roadmap task 2.1.2
  complete after the proof gates passed; task 2.1.3 remains unchecked.
- [x] (2026-07-01T12:20:55Z) Work item 2 deterministic gates passed:
  `make all`, `make markdownlint`, and `make nixie`.
- [x] (2026-07-01T12:20:55Z) Final validation and retrospective recorded.

## Surprises & Discoveries

- Memtrace `list_indexed_repositories` returned `user cancelled MCP tool call`
  during planning. Canonical-main graph context was therefore unavailable in
  this planning session.
- Memtrace `list_indexed_repositories` again returned
  `user cancelled MCP tool call` during implementation, so canonical-main
  graph context remained unavailable.
- `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-2-1-2`
  succeeded, but `leta files docs` failed with `Error: Failed to start
  daemon`. Branch-local verification therefore used exact text search and
  bounded file inspection.
- `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-2-1-2`
  failed during implementation with
  `Error: IO error: Read-only file system (os error 30)`, and
  `leta grep "renumber_sub_tasks" "src/roadmap/ops/rewrite.rs"` failed with
  `Error: Failed to start daemon`. Branch-local verification continued with
  bounded local inspection.
- Firecrawl `firecrawl_extract` for official `rstest` and `serial_test` docs
  returned `user cancelled MCP tool call`. The plan does not rely on a new
  external API; it uses existing repository test attributes and local locked
  crate source as fallback evidence.
- `sem diff --from origin/main --to HEAD --format json` reported zero changes
  during planning, so no pre-existing branch delta was present before this
  ExecPlan.
- `tests/roadmap_sub_tasks.rs` already covers append renumbering, deleting a
  parent task, deleting a sub-task, dependency rewriting inside sub-task text,
  malformed parent rejection, and nested roadmap-list rejection.
- No existing test name or assertion exactly mirrors the roadmap success text:
  parent task `8.2.3` moving to `9.2.3`, sub-task `8.2.3.1` moving to
  `9.2.3.1`, and references to the sub-task being rewritten.
- `Cargo.toml` declares `rstest = "0.26.1"` and `serial_test = "3.2.0"`, while
  `Cargo.lock` resolves `rstest` to `0.26.1` and `serial_test` to `3.5.0`.
  Local source for `rstest` documents `#[rstest]`, `#[case]`, and
  `#[fixture]`; local source for locked `serial_test` documents
  `#[serial]` and keyed serial groups.
- `Makefile` target `all` expands to `check-fmt lint typecheck test`, so the
  required validation command includes the current typecheck gate.
- The new
  `insert_before_phase_moves_sub_task_and_rewrites_sub_task_references` test
  passed without production code changes, confirming the existing rewrite path
  already satisfies the roadmap success case.
- `coderabbit review --agent` did not reach analysis for work item 1. The
  initial attempt timed out with exit status 124, and retry 1 was interrupted
  while still connecting. Both logs contained only:
  `{"type":"status","phase":"connecting","status":"connecting_to_review_service"}`.
  No rate-limit, deferred-review, or actionable finding was emitted.
- `make nixie` initially timed out on an untouched Mermaid diagram in
  `docs/ortho-config-users-guide.md`; retry 1 passed without file changes.
- `coderabbit review --agent` did not reach analysis for work item 2. The log
  contained only:
  `{"type":"review_context","reviewType":"all","currentBranch":"roadmap-2-1-2","baseBranch":"main","workingDirectory":"/home/leynos/Projects/mapsplice.worktrees/roadmap-2-1-2"}`
  and
  `{"type":"status","phase":"connecting","status":"connecting_to_review_service"}`.
  No rate-limit, deferred-review, or actionable finding was emitted.

## Decision Log

- Decision: Treat task 2.1.2 as an audit-plus-proof change unless the exact
  success-case test fails.
  Rationale: The current source already renumbers sub-tasks through
  `renumber_sub_tasks` and rewrites their dependency text through
  `rewrite_sub_task_entry`; a focused test is the smallest missing evidence.
  Date/Author: 2026-07-01T10:58:57Z / Codex.

- Decision: Add a behavioural integration test instead of a lower-level unit
  test.
  Rationale: The roadmap success criterion is user-observable CLI output after
  a structural edit, and existing sub-task tests already use `run_from_args`
  plus `Workspace` fixtures for that layer.
  Date/Author: 2026-07-01T10:58:57Z / Codex.

- Decision: Do not add or change external dependencies.
  Rationale: The required mechanism is already expressible with existing
  domain types, `rstest`, and `serial_test`.
  Date/Author: 2026-07-01T10:58:57Z / Codex.

- Decision: Treat the work item 1 CodeRabbit pass as a documented deferred
  review issue rather than an implementation blocker.
  Rationale: Deterministic gates passed, but CodeRabbit never completed a
  review and emitted no actionable feedback or rate-limit backoff instruction;
  both attempts stalled at `connecting_to_review_service`.
  Date/Author: 2026-07-01T11:33:56Z / Codex.

- Decision: Treat the work item 2 CodeRabbit pass as the same documented
  deferred review issue.
  Rationale: The roadmap-only work item passed `make all`, `make
  markdownlint`, and `make nixie`; CodeRabbit again stalled at
  `connecting_to_review_service` without rate-limit guidance or findings.
  Date/Author: 2026-07-01T12:20:55Z / Codex.

## Outcomes & Retrospective

Work item 1 added the exact behavioural proof requested by roadmap task
2.1.2. The focused test command passed and `make all` passed at
`/tmp/make-all-wi1-mapsplice-roadmap-2-1-2.out`. The implementation did not
need production code changes. CodeRabbit review remains deferred because the
service stalled while connecting and produced no findings. Work item 1 was
committed as `0796ff8b7b8a9a9e7756d910f158ae9bbf11fc08`.

Work item 2 marked `docs/roadmap.md` task 2.1.2 complete and left task 2.1.3
incomplete, preserving the planned boundary between renumbering and render
indentation work. The work item 2 deterministic gate logs are:
`/tmp/make-all-wi2-mapsplice-roadmap-2-1-2.out`,
`/tmp/markdownlint-wi2-mapsplice-roadmap-2-1-2.out`, and
`/tmp/nixie-wi2-mapsplice-roadmap-2-1-2.out`. CodeRabbit review remains open
for supervisor decision because the service did not progress beyond
`connecting_to_review_service`.

The delivered behaviour now matches the roadmap task 2.1.2 success statement:
the committed regression proves `8.2.3` moves to `9.2.3`, sub-task
`8.2.3.1` moves to `9.2.3.1`, and `Requires 8.2.3.1` references are rewritten
to `Requires 9.2.3.1`.

## Context and orientation

`mapsplice` parses roadmap-shaped Markdown into a typed model, applies one
operation, renumbers affected items, rewrites dependency references, and
renders Markdown. The relevant roadmap structure has four levels:

- phase, such as `## 8. Phase title`;
- step, such as `### 8.2. Step title`;
- task, such as `- [ ] 8.2.3. Task title`;
- addendum sub-task, such as `- [ ] 8.2.3.1. Sub-task title`.

The normative documents are:

- `docs/mapsplice-design.md` section 4 for the accepted grammar;
- `docs/mapsplice-design.md` section 6 contracts C2, C3, and C4 for
  renumbering, reference rewriting, and addenda;
- `docs/mapsplice-design.md` section 7 for dependency-reference resolution;
- `docs/mapsplice-design.md` section 8 for fixture and test requirements;
- `docs/developers-guide.md` sections 2, 3, 6, and 7 for roadmap module
  boundaries, public API, verification layers, and gates;
- `docs/users-guide.md` section "The roadmap shape `mapsplice` expects" for
  user-facing fourth-level anchors;
- `docs/documentation-style-guide.md` section "Roadmap task writing
  guidelines" for roadmap item status and formatting;
- `docs/contributing.md` section "Development gates" and `AGENTS.md` section
  "Change Quality & Committing" for commit gates.

The relevant branch-local source files are:

- `src/roadmap/ops/rewrite.rs`, where `renumber_document`,
  `renumber_sub_tasks`, `rewrite_dependencies`, `rewrite_task_entry`, and
  `rewrite_sub_task_entry` live;
- `tests/roadmap_sub_tasks.rs`, where the new behavioural proof belongs;
- `tests/support/sub_tasks.rs`, which may hold a reusable fixture if the new
  success case is too large to keep readable in the test body;
- `docs/roadmap.md`, where task 2.1.2 is currently unchecked.

## Plan of work

### Work item 1: Add the exact parent/sub-task renumber proof

Documentation to read before editing:

- `docs/roadmap.md` task 2.1.2 for the exact success statement.
- `docs/mapsplice-design.md` section 6 contracts C2, C3, and C4.
- `docs/mapsplice-design.md` section 7 for dependency-reference resolution.
- `docs/developers-guide.md` sections 2, 3, 6, and 7.
- `AGENTS.md` sections "Rust Specific Guidance" and "Testing".

Skills to load before editing:

- `leta` for branch-local symbol navigation if the daemon works.
- `sem` for entity-level diff review.
- `rust-router`, then `rust-unit-testing`; no additional Rust skill is
  expected unless the test fails and production code must change.

Edit `tests/roadmap_sub_tasks.rs`. Add a new `#[rstest]` and
`#[serial_test::serial(cli_env)]` behavioural test named
`insert_before_phase_moves_sub_task_and_rewrites_sub_task_references`. The
test constructs a valid contiguous roadmap in which parent task `8.2.3` has
sub-task `8.2.3.1` and the parent summary contains `Requires 8.2.3.1.`.
Insert a phase before anchor `8` using the existing `PHASE_FRAGMENT` fixture.
Then assert all of these observable output facts:

- stdout contains `- [ ] 9.2.3. Parent task. Requires 9.2.3.1.`;
- stdout contains
  `- [ ] 9.2.3.1. Nested sub-task. Requires 9.2.3.`;
- stdout does not contain `Requires 8.2.3.1.`;
- stdout does not contain
  `- [ ] 8.2.3.1. Nested sub-task. Requires 8.2.3.`;
- the command succeeds through `run_from_args`.

Keep helper scope narrow. Prefer a small private function in
`tests/roadmap_sub_tasks.rs` if the fixture is only used by this test. Move it
to `tests/support/sub_tasks.rs` only if it materially improves readability.
The fixture must be a conformant roadmap with phases 1 through 8, phase 8
containing steps 8.1 and 8.2, and step 8.2 containing tasks 8.2.1, 8.2.2, and
8.2.3. This keeps the `8.2.3` to `9.2.3` movement literal without relying on
non-contiguous input.

Run the focused command and expect the new test to pass on this branch:

```bash
cargo test --workspace --all-targets --all-features \
  --test roadmap_sub_tasks \
  insert_before_phase_moves_sub_task_and_rewrites_sub_task_references \
  | tee /tmp/test-exact-subtask-renumber-mapsplice-roadmap-2-1-2.out
```

If it fails, do not invent a different mechanism. Inspect
`src/roadmap/ops/rewrite.rs::renumber_document`,
`src/roadmap/ops/rewrite.rs::renumber_sub_tasks`,
`src/roadmap/ops/rewrite.rs::rewrite_task_entry`, and
`src/roadmap/ops/rewrite.rs::rewrite_sub_task_entry`. Apply the smallest fix
in `src/roadmap/ops/rewrite.rs` that makes the plan insert fourth-level
anchors and rewrite sub-task dependency text. Then rerun the same focused
command.

After the focused test passes, run the full code gate:

```bash
make all | tee /tmp/make-all-wi1-mapsplice-roadmap-2-1-2.out
```

Review the semantic diff before committing:

```bash
sem diff --format json | tee /tmp/sem-diff-wi1-mapsplice-roadmap-2-1-2.out
```

Commit only the test, and any strictly necessary production fix, with an
imperative subject such as `Prove sub-task parent renumbering`.

### Work item 2: Mark roadmap task 2.1.2 complete after proof gates pass

Documentation to read before editing:

- `docs/roadmap.md` section 2.1.
- `docs/documentation-style-guide.md` section "Roadmap task writing
  guidelines".
- `docs/contributing.md` section "Development gates".
- `AGENTS.md` sections "Documentation Maintenance" and "Markdown Guidance".

Skills to load before editing:

- `execplans` because this plan must be updated as a living document.
- `sem` for reviewing the Markdown entity-level change.
- `en-gb-oxendict-style` if any new prose is added beyond the checkbox
  status change.

Edit `docs/roadmap.md` only after work item 1 is committed and gated. Change
task 2.1.2 from unchecked to checked. Do not check task 2.1.3, and do not
claim the nesting/indentation task is complete.

Update this ExecPlan's `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` with the work item 1 commit hash and gate log
paths. Format only the Markdown files touched by this work item:

```bash
mdtablefix docs/roadmap.md docs/execplans/roadmap-2-1-2.md \
  | tee /tmp/mdtablefix-wi2-mapsplice-roadmap-2-1-2.out
markdownlint-cli2 --fix docs/roadmap.md docs/execplans/roadmap-2-1-2.md \
  | tee /tmp/markdownlint-fix-wi2-mapsplice-roadmap-2-1-2.out
```

Then run the required validation gates:

```bash
make all | tee /tmp/make-all-wi2-mapsplice-roadmap-2-1-2.out
make markdownlint | tee /tmp/markdownlint-wi2-mapsplice-roadmap-2-1-2.out
make nixie | tee /tmp/nixie-wi2-mapsplice-roadmap-2-1-2.out
```

Review the semantic diff before committing:

```bash
sem diff --format json | tee /tmp/sem-diff-wi2-mapsplice-roadmap-2-1-2.out
```

Commit the roadmap and ExecPlan updates with an imperative subject such as
`Mark sub-task renumbering complete`.

## Concrete steps

1. Start in the assigned worktree:

   ```bash
   cd /home/leynos/Projects/mapsplice.worktrees/roadmap-2-1-2
   git branch --show-current
   ```

   Expected output:

   ```plaintext
   roadmap-2-1-2
   ```

2. Re-run advisory-tool orientation. If Memtrace or `leta` fails again, append
   the exact failure to `Surprises & Discoveries` and continue:

   ```bash
   sem diff --from origin/main --to HEAD --format json \
     | tee /tmp/sem-baseline-mapsplice-roadmap-2-1-2.out
   leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-2-1-2 \
     | tee /tmp/leta-workspace-mapsplice-roadmap-2-1-2.out
   leta grep "renumber_sub_tasks" "src/roadmap/ops/rewrite.rs" \
     | tee /tmp/leta-renumber-subtasks-mapsplice-roadmap-2-1-2.out
   ```

3. Complete work item 1 exactly as written. Do not update `docs/roadmap.md`
   until the exact behavioural proof is committed.

4. Complete work item 2 exactly as written. Keep Markdown formatting
   path-safe and run `make all`, `make markdownlint`, and `make nixie`.

5. Before returning, run:

   ```bash
   git status --short | tee /tmp/git-status-final-mapsplice-roadmap-2-1-2.out
   git log --oneline -2 | tee /tmp/git-log-final-mapsplice-roadmap-2-1-2.out
   ```

   Expected final status is clean.

## Validation and acceptance

Acceptance is behavioural. After work item 1, the repository has a named
integration test proving that an edit moving task `8.2.3` to `9.2.3` also
moves sub-task `8.2.3.1` to `9.2.3.1` and rewrites a dependency reference to
the sub-task. After work item 2, `docs/roadmap.md` marks task 2.1.2 complete
and leaves task 2.1.3 incomplete.

Because branch-local source inspection shows the production mechanism already
exists, this plan does not require a standalone failing red test before a
production edit. The proof test is expected to pass without production changes.
If it fails, the failing focused command becomes the red evidence, and the
green change must be limited to `src/roadmap/ops/rewrite.rs` unless this plan
is revised.

Quality criteria:

- Tests: the new
  `insert_before_phase_moves_sub_task_and_rewrites_sub_task_references`
  integration test passes, and `make all` passes.
- Lint/typecheck: `make all` passes, including `check-fmt`, `lint`,
  `typecheck`, and `test`.
- Markdown: for the roadmap completion commit, `make markdownlint` and
  `make nixie` pass after path-scoped Markdown formatting.
- Scope: no public API change and no new dependency.

## Idempotence and recovery

The planned test and roadmap edits are safe to rerun. If a focused test fails,
leave the failure visible in this ExecPlan and do not update the roadmap status
until the proof passes. If a Markdown formatter modifies files outside
`docs/roadmap.md` or `docs/execplans/roadmap-2-1-2.md`, inspect the diff and
park or discard unrelated churn with a named stash:

```bash
git stash push -m \
  'df12-stash v1 task=2.1.2 kind=discard reason="unrelated formatter churn"' \
  -- <paths>
```

Do not use a bare `git stash`. If `make nixie` fails on an untouched Mermaid
diagram, record the exact log path and retry once. If it fails twice with the
same untouched diagram and all code gates pass, stop and ask for review rather
than marking the work complete silently.

## Artifacts and notes

Planning artifacts:

```plaintext
Memtrace list_indexed_repositories: user cancelled MCP tool call
leta files docs: Error: Failed to start daemon
Firecrawl official-doc extraction: user cancelled MCP tool call
sem diff --from origin/main --to HEAD --format json: zero changes
```

Local dependency evidence:

- `Cargo.lock` resolves `rstest` to `0.26.1`; local source documents
  `#[rstest]`, `#[case]`, and `#[fixture]`.
- `Cargo.lock` resolves `serial_test` to `3.5.0`; local source documents
  `#[serial]` and keyed serial groups.
- `Cargo.lock` resolves `markdown` to `1.0.0`; local source exports `mdast`
  nodes used by the parse and rewrite implementation.

## Interfaces and dependencies

No public interface change is planned.

Use the existing test surface:

```rust
#[rstest]
#[serial_test::serial(cli_env)]
fn insert_before_phase_moves_sub_task_and_rewrites_sub_task_references(
    workspace: TestResult<Workspace>,
) -> TestResult {
    // Test body described in work item 1.
}
```

Use the existing production interfaces only if the exact proof fails:

```rust
pub(super) fn renumber_document(roadmap: &mut RoadmapDocument) -> Result<RenumberPlan>;
fn renumber_sub_tasks(
    task: &mut TaskEntry,
    new_task: TaskNumber,
    plan: &mut RenumberPlan,
) -> Result<()>;
pub(super) fn rewrite_dependencies(
    roadmap: &mut RoadmapDocument,
    plan: &RenumberPlan,
) -> Result<u64>;
```

No new crate, command-line option, environment variable, snapshot format, or
configuration key should be introduced for roadmap task 2.1.2.

## Revision note

2026-07-01T11:33:56Z: Work item 1 changed this plan from draft execution
state to in-progress delivery, recorded the exact behavioural proof and gate
evidence, and documented CodeRabbit as a deferred review issue because both
attempts stalled while connecting to the review service. Remaining work is to
mark only roadmap task 2.1.2 complete, re-run the gates, and commit the final
roadmap update.

2026-07-01T11:36:36Z: Work item 2 marked only roadmap task 2.1.2 complete and
recorded the work item 1 commit hash and gate evidence. Remaining work is the
required deterministic gate, CodeRabbit attempt, final ExecPlan closure, and
atomic commit for the roadmap update.

2026-07-01T12:20:55Z: Final ExecPlan closure recorded work item 2 gate
evidence and the CodeRabbit service-connection issue. No further work items
remain in this plan; the only open issue is the deferred CodeRabbit review.
