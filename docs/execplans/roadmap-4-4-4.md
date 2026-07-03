# Cover sub-task splice vector alignment

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

Roadmap task 4.4.4 is complete when the highest-risk addendum sub-task splice
invariant is directly covered by focused unit tests. A task stores each
addendum sub-task in `TaskEntry.sub_tasks` and also stores the original render
order in `TaskEntry.children`. Insert-before, insert-after, and replace
operations must mutate both vectors together. Success is observable when a
focused test in `src/roadmap/ops/sub_task.rs` fails if either vector is updated
without the corresponding `TaskChild::SubTask` entry, and passes through the
normal repository gates.

This plan is the second planning-round revision. It resolves the prior
design-review blocker about Work Item 3 Markdown formatting. Do not implement
it until the plan is approved.

## Constraints

- Work only in `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-4`.
- Preserve the public CLI and library APIs documented in
  `docs/developers-guide.md` section 3. If a public API signature must change,
  stop and escalate.
- Implement roadmap task 4.4.4 from `docs/roadmap.md` lines 252-258 only.
  Do not fold in roadmap task 5.1.1 parser consolidation or Finding 17 from
  `docs/issues/audit-4.2.2.md`.
- Preserve `docs/mapsplice-design.md` section 4 and section 6 contracts:
  addendum sub-tasks are first-class fourth-level items, successful operations
  renumber all levels contiguously, and addendum sub-task nesting is preserved.
- Use Red-Green-Refactor. First add a test that fails for the intended reason
  when `insert_sub_tasks` or `replace_sub_task` mutates only one vector; then
  keep or adjust production code only if the red test exposes a real defect.
- Keep changes atomic. Each work item below must be independently committable
  and gate-passable.
- Do not add a new external dependency. The locked `rstest` 0.26.1 crate
  already supports the table-driven unit-test shape needed here.
- Do not run repository-global Markdown formatting for this implementation.
  If Markdown files are changed, format only those paths and then run
  `make markdownlint` and `make nixie`.
- All command output used for gates must be captured with `tee` under `/tmp`.
  The log path pattern in this plan uses the branch leaf `roadmap-4-4-4`.

## Tolerances

- Scope: stop and escalate if the implementation needs production changes
  outside `src/roadmap/ops/sub_task.rs`, or test changes outside
  `src/roadmap/ops/sub_task.rs` and `tests/roadmap_sub_tasks.rs`.
- Interface: stop and escalate if any public type, function, or CLI syntax must
  change.
- Dependencies: stop and escalate if a new crate or feature flag is required.
- Behaviour: stop and escalate if an existing public sub-task operation changes
  rendered output in a way not required by `docs/mapsplice-design.md` F1-F5 and
  C1-C6.
- Iterations: stop and escalate if the same focused test still fails after
  three implementation attempts.
- Size: stop and escalate if the net production-code diff exceeds 80 lines or
  if `src/roadmap/ops/sub_task.rs` would exceed the 400-line file limit.
- Ambiguity: stop and escalate if unit-level vector alignment and CLI-level
  render behaviour point to different expected semantics.

## Risks

- Risk: the current code already satisfies the invariant and this task is
  test-only. Severity: low. Likelihood: high. Mitigation: keep Work Item 1 as a
  red test that proves it would catch a deliberate one-vector mutation, then
  commit the passing tests without production churn.

- Risk: table-driven internal tests duplicate existing CLI coverage.
  Severity: low. Likelihood: medium. Mitigation: make the internal assertion
  inspect `TaskEntry.sub_tasks` and `TaskEntry.children` identities directly.
  The existing CLI tests in `tests/roadmap_sub_tasks.rs` prove rendered
  behaviour, not the vector invariant.

- Risk: `TaskEntry.children` may include `TaskChild::Body` entries between
  sub-tasks, and a naive assertion that compares raw children to `sub_tasks`
  would reject valid task body content. Severity: medium. Likelihood: medium.
  Mitigation: the assertion helper must filter only `TaskChild::SubTask`
  identities and compare them with `task.sub_tasks.iter().map(|s| s.identity)`.

- Risk: `cargo test` output can be truncated by the agent environment.
  Severity: low. Likelihood: medium. Mitigation: every validation command in
  this plan pipes through `tee` to a `/tmp` log.

## Progress

- [x] (2026-07-03T03:29:55Z) Drafted the first-round plan from local docs,
  source inspection, locked crate source, and available tooling evidence.
- [x] (2026-07-03T03:41:55Z) Revised Work Item 3 so scoped Markdown formatting
  uses the repository-default `MARKDOWN_FORMAT_FLAGS`, preserving the
  load-bearing `--wrap` flag needed for `make markdownlint`.
- [x] (2026-07-03T03:58:13Z) Work Item 1: Added focused insert-before and
  insert-after sub-task alignment unit tests. The deliberate one-vector
  mutation failed the new test as intended, the restored code passed the
  focused test, and `make all` passed via `scrutineer`.
- [x] (2026-07-03T04:06:37Z) Work Item 2: Added focused replace-first and
  replace-last-with-multiple sub-task alignment unit tests. The deliberate
  one-vector mutation failed the new test as intended, the restored code passed
  the focused unit test, `cargo test roadmap::ops::sub_task`,
  `cargo test --test roadmap_sub_tasks`, and `make all` via `scrutineer`. No
  production fix was required.
- [x] (2026-07-03T04:09:56Z) Work Item 3: Marked roadmap task 4.4.4 complete,
  ran the final deterministic gates, recorded the final CodeRabbit deferral,
  and closed this ExecPlan.

## Surprises & discoveries

- Observation: Memtrace was unavailable in this planning session.
  Evidence: `mcp__memtrace__list_indexed_repositories` returned
  `user cancelled MCP tool call`. Impact: this plan records bounded
  branch-local evidence instead. This is not a product blocker under the task
  instructions.

- Observation: Leta could not start in this sandbox.
  Evidence:
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-4`
  returned `Error: IO error: Read-only file system (os error 30)`. Retrying with
  `XDG_DATA_HOME` and `XDG_CONFIG_HOME` under `/tmp` returned the same error.
  Impact: code orientation used exact local source inspection. The implementer
  should retry Leta before editing and record the result here.

- Observation: Firecrawl was unavailable for official docs in this planning
  session. Evidence:
  `mcp__firecrawl__firecrawl_scrape https://docs.rs/rstest/0.26.1/rstest/attr.rstest.html`
  returned `user cancelled MCP tool call`. Impact: `rstest` behaviour was
  verified against the locked crate source and README in the local Cargo
  registry. The plan does not rely on unverified network-only behaviour.

- Observation: the working tree had no semantic diff from `origin/main` before
  this plan was written. Evidence:
  `sem diff --from origin/main --to HEAD --file-exts .rs .md --format json`
  reported zero changed files and zero changed entities. Impact: branch-local
  evidence matched the current branch state before the planning document was
  added.

- Observation: Leta was available in this implementation session for symbol
  discovery and source display, but `leta refs` could not start the daemon.
  Evidence:
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-4`
  succeeded; `leta grep` for the target sub-task helpers and model types under
  `src/roadmap` located the expected symbols; `leta refs insert_sub_tasks -n 2`
  and `leta refs 'sub_task.rs:replace_sub_task' -n 2` both returned
  `Error: Failed to start daemon`. Impact: branch-local verification used
  successful `leta grep` and `leta show` output plus bounded source inspection.

- Observation: Memtrace remained unavailable in this implementation session.
  Evidence: `mcp__memtrace__list_indexed_repositories` returned
  `user cancelled MCP tool call`. Impact: the implementation continued with
  bounded branch-local evidence, as allowed by the task instructions.

- Observation: the Work Item 1 CodeRabbit pass was deferred by the sandbox
  environment rather than by a code finding. Evidence:
  `coderabbit-review-agent` exited 124 and wrote
  `deferred coderabbit review: no default network route visible in this sandbox`
  to `/tmp/coderabbit-sub-task-insert-alignment-mapsplice-roadmap-4-4-4.out`.
  Impact: there were no actionable CodeRabbit findings to address for Work Item
  1; the deferred review remains an open issue for the supervisor.

- Observation: the Work Item 2 CodeRabbit pass was deferred by the same sandbox
  network state. Evidence: `coderabbit-review-agent` exited 124 and wrote
  `deferred coderabbit review: no default network route visible in this sandbox`
  to `/tmp/coderabbit-sub-task-replace-alignment-mapsplice-roadmap-4-4-4.out`.
  Impact: there were no actionable CodeRabbit findings to address for Work Item
  2; the deferred review remains an open issue for the supervisor.

- Observation: the final Work Item 3 CodeRabbit pass was also deferred by the
  sandbox network state. Evidence: `coderabbit-review-agent` wrote
  `deferred coderabbit review: no default network route visible in this sandbox`
  to `/tmp/final-coderabbit-mapsplice-roadmap-4-4-4.out`. Impact:
  deterministic gates passed, but the final AI review remains an open issue for
  the supervisor.

## Decision log

- Decision: implement 4.4.4 as focused internal unit coverage in
  `src/roadmap/ops/sub_task.rs`, with a lightweight CLI backstop only if the
  direct unit tests expose a user-visible gap. Rationale: `docs/roadmap.md`
  lines 252-258 specifically names unit tests that fail when `sub_tasks` and
  `children` diverge. `docs/developers-guide.md` section 6 says `rstest` unit
  tests cover splice behaviour, while BDD covers compiled binary workflows.
  Date/Author: 2026-07-03, Codex.

- Decision: use locked `rstest` 0.26.1 for parameterized cases instead of
  adding or changing test dependencies. Rationale: `Cargo.toml` already pins
  `rstest = "0.26.1"`, and `Cargo.lock` records `rstest` 0.26.1 plus
  `rstest_macros` 0.26.1. The locked README at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rstest-0.26.1/README.md`
  documents `#[rstest]`, `#[case]`, fixtures, and independent generated tests.
  Date/Author: 2026-07-03, Codex.

- Decision: do not add property tests for this roadmap item.
  Rationale: the failure mode is a small structural invariant over three known
  operations and boundary ordinals. `rust-verification` routes broad generated
  input gaps to `proptest`, but `rust-unit-testing` is the smaller useful tool
  for a concrete table of splice cases. The developers' guide already reserves
  existing proptests for anchor round-trips and dependency rewrites.
  Date/Author: 2026-07-03, Codex.

- Decision: Work Item 3 must call `make markdownfmt` without overriding
  `MARKDOWN_FORMAT_FLAGS`. Rationale: `Makefile` line 23 defines the default
  `mdtablefix` flags as
  `--wrap --renumber --breaks --ellipsis --fences --in-place`, and
  `.markdownlint-cli2.jsonc` lines 5-7 enforce MD013 at 80 columns. MD013 is
  not reliably auto-fixable by `markdownlint-cli2 --fix`, so dropping `--wrap`
  can make the plan fail its own `make markdownlint` gate. Date/Author:
  2026-07-03, Codex.

## Outcomes & retrospective

Roadmap task 4.4.4 is implemented and marked complete in `docs/roadmap.md`. The
focused tests added are
`sub_task_insert_keeps_structural_and_child_vectors_aligned` and
`sub_task_replace_keeps_structural_and_child_vectors_aligned`. The insert test
covers insert-before first and insert-after last boundary ordinals. The replace
test covers replacing the first sub-task and replacing the last sub-task with a
multi-item fragment. Both tests assert rendered sub-task numbers and direct
alignment between `TaskEntry.sub_tasks` identities and filtered
`TaskChild::SubTask` identities.

No production code changed. The existing `insert_sub_tasks` and
`replace_sub_task` implementations already spliced the structural vector and
the child-order vector together. The deliberate red mutations for both helpers
failed the new tests for the intended reason.

Validation evidence:

- `/tmp/green-sub-task-insert-alignment-mapsplice-roadmap-4-4-4.out`
- `/tmp/green-sub-task-replace-alignment-mapsplice-roadmap-4-4-4.out`
- `/tmp/unit-sub-task-ops-mapsplice-roadmap-4-4-4.out`
- `/tmp/integration-sub-task-mapsplice-roadmap-4-4-4.out`
- `/tmp/final-make-all-mapsplice-roadmap-4-4-4.out`
- `/tmp/final-markdownlint-mapsplice-roadmap-4-4-4.out`
- `/tmp/final-nixie-mapsplice-roadmap-4-4-4.out`

Lesson for roadmap task 5.1.1: unit coverage around model-vector invariants is
valuable even when CLI behaviour is already covered, because the direct
identity assertions catch internal divergence before render output obscures the
cause.

## Context and orientation

`mapsplice` edits roadmap-shaped Markdown. The relevant grammar is in
`docs/mapsplice-design.md` section 4 and `docs/users-guide.md` "The roadmap
shape `mapsplice` expects": phases, steps, tasks, and addendum sub-tasks are
addressable structural levels. An addendum sub-task is a nested checklist item
whose number extends its parent task by one level, such as `8.2.3.1`.

The model in `src/roadmap/model.rs` stores sub-task state in two coordinated
places. `TaskEntry.sub_tasks` is the structural vector of first-class
`SubTaskEntry` values. `TaskEntry.children` preserves render order as
`TaskChild::Body(MarkdownNodes)` and `TaskChild::SubTask(ItemIdentity)`.
Renderer code later uses `children` to replay body blocks and sub-tasks in
source order, so a sub-task operation that updates only one vector can produce
invalid model state or incorrect output.

The target code is `src/roadmap/ops/sub_task.rs`. Current source lines 13-28
define `insert_sub_tasks`: it finds the target sub-task in `sub_tasks`, finds
the corresponding child identity in `children`, builds `new_children` from the
fragment sub-tasks, and splices both vectors at the before or after index.
Lines 40-53 define `replace_sub_task`: it finds the same pair of indices and
splices both vectors over the targeted element. Lines 105-159 contain the
colocated test module, which currently tests only delete alignment.

`docs/issues/audit-4.2.2.md` Finding 16 identifies the gap: insert and replace
splice both the structural `sub_tasks` vector and the parallel `children`
vector, but the colocated test module only exercises delete. The roadmap item
4.4.4 is the accepted work item for closing that gap.

Tooling evidence gathered before this plan:

- Memtrace: unavailable, exact result `user cancelled MCP tool call`.
- Leta: unavailable, exact results `Read-only file system (os error 30)` and
  `Failed to start daemon`.
- Sem: available;
  `sem diff --from origin/main --to HEAD --file-exts .rs .md --format json`
  showed no branch changes before this plan file was created.
- Firecrawl: unavailable, exact result `user cancelled MCP tool call`.
- Locked crate source: available for `rstest` 0.26.1 under the shared Cargo
  registry.

## Plan of work

### Work Item 1: Add focused sub-task insert alignment unit tests

Read first:

- `docs/roadmap.md` lines 252-258 for the task scope.
- `docs/mapsplice-design.md` sections 4, 5, and 6 for the roadmap grammar,
  fidelity guarantees, and addenda contract.
- `docs/developers-guide.md` sections 2 and 6 for domain boundaries and test
  layers.
- `src/roadmap/ops/sub_task.rs` lines 13-28 and 105-159 for the implementation
  and colocated delete test.
- Locked `rstest` 0.26.1 README lines documenting `#[rstest]` and `#[case]`
  under the Cargo registry path named above.
- `docs/execplans/initial-tool.md` lines 180-190 and 246-255 for the accepted
  ADR-style direction that `insert` and `replace` operate on the roadmap model
  and mutation coverage belongs in `rstest` unit tests.

Load or follow these skills:

- `leta` for branch-local symbol navigation. If it fails, record the exact
  failure and use bounded source inspection.
- `rust-router`, then `rust-unit-testing`. `rust-verification` should be used
  only to confirm that unit tests, not property tests, are the right level.
- `sem` before committing to review the entity-level diff.

Implementation:

1. In `src/roadmap/ops/sub_task.rs`, extend the existing `#[cfg(test)]` module.
   Import `rstest::rstest` and any already-local model types needed by the test
   helper. Keep production code unchanged in this work item.
2. Add a fixture string with three sub-tasks and at least one non-sub-task body
   block under the parent task. The body block is required so the assertion
   proves only `TaskChild::SubTask` order, not raw `children` equality.
3. Add a small helper such as
   `sub_task_child_identities(task: &TaskEntry) -> Vec<ItemIdentity>` that
   filters `task.children` to `TaskChild::SubTask` identities. Add a companion
   assertion helper that compares those identities with
   `task.sub_tasks.iter().map(|sub_task| sub_task.identity)`.
4. Add a table-driven `#[rstest]` unit test for insert-before and insert-after
   boundary cases. The cases must include:
   - inserting before the first sub-task, target anchor `1.1.1.1`;
   - inserting after the last sub-task, target anchor `1.1.1.3`.
5. Each case must parse the target and a sub-task fragment with
   `parse_roadmap` and `parse_fragment`, call `apply_command` with
   `RoadmapOperation::Insert { anchor, after }`, then inspect the parent task
   directly.
6. Assert both the rendered sub-task numbers and the identity alignment. The
   direct invariant is:
   `sub_task_child_identities(task) == task.sub_tasks.iter().map(|s| s.identity).collect::<Vec<_>>()`.

Red validation:

Before accepting the test as useful, deliberately introduce a temporary local
mutation that removes only the `task.children.splice(...)` call from
`insert_sub_tasks`, run the focused test, and confirm failure. Restore the
production code immediately after observing red; do not commit the deliberate
mutation.

```bash
cargo test sub_task_insert_keeps_structural_and_child_vectors_aligned \
  2>&1 | tee /tmp/red-sub-task-insert-alignment-mapsplice-roadmap-4-4-4.out
```

Expected red signal: the new test fails because the child identity sequence no
longer matches the structural `sub_tasks` identities after insert-before or
insert-after.

Green validation:

```bash
cargo test sub_task_insert_keeps_structural_and_child_vectors_aligned \
  2>&1 | tee /tmp/green-sub-task-insert-alignment-mapsplice-roadmap-4-4-4.out
```

Expected green signal: all generated insert cases pass. Then run the commit
gates:

```bash
make all 2>&1 | tee /tmp/make-all-sub-task-insert-alignment-mapsplice-roadmap-4-4-4.out
```

Review before commit:

```bash
sem diff --file-exts .rs
git diff -- src/roadmap/ops/sub_task.rs
```

Commit message:

```text
Prove sub-task insert alignment

Add focused unit coverage for insert-before and insert-after sub-task
operations so the structural sub-task vector and render-order child vector
must stay aligned.
```

### Work Item 2: Add focused sub-task replace alignment unit tests

Read first:

- The same design and guide sections as Work Item 1.
- `src/roadmap/ops/sub_task.rs` lines 40-53 for `replace_sub_task`.
- `docs/issues/audit-4.2.2.md` Finding 16 for the replace test-gap rationale.
- `docs/execplans/initial-tool.md` lines 180-190 and 246-255 for the accepted
  ADR-style direction that `replace` swaps the addressed model item and that
  mutation coverage belongs in `rstest` unit tests.

Load or follow these skills:

- `leta` for symbol navigation, with the same fallback rule if unavailable.
- `rust-router`, then `rust-unit-testing`.
- `sem` before committing to review the entity-level diff.

Implementation:

1. In the same test module, add table-driven replace cases that reuse the
   target fixture and identity-alignment helpers from Work Item 1.
2. Cover boundary ordinals by replacing the first sub-task, target anchor
   `1.1.1.1`, and replacing the last sub-task, target anchor `1.1.1.3`.
3. Include a multi-sub-task fragment in at least one replace case so
   `replace_sub_task` must replace one child reference with more than one
   `TaskChild::SubTask` entry. This pins the actual splice behaviour on both
   vectors.
4. Assert that the `sub_tasks` vector contains the expected replacement titles
   or numbers after renumbering, and assert that the filtered child identities
   exactly match the structural sub-task identities.
5. If either Work Item 1 or Work Item 2 exposes a production defect, make the
   smallest fix in `insert_sub_tasks`, `replace_sub_task`, or a private helper
   in `src/roadmap/ops/sub_task.rs`. Do not refactor parser or renderer code in
   this work item.

Red validation:

Before accepting the replace test as useful, deliberately introduce a temporary
local mutation that removes only the `task.children.splice(...)` call from
`replace_sub_task`, run the focused test, and confirm failure. Restore the
production code immediately after observing red; do not commit the deliberate
mutation.

```bash
cargo test sub_task_replace_keeps_structural_and_child_vectors_aligned \
  2>&1 | tee /tmp/red-sub-task-replace-alignment-mapsplice-roadmap-4-4-4.out
```

Expected red signal: the new test fails because replace leaves stale child
identity entries or misses new replacement identities.

Green validation:

```bash
cargo test sub_task_replace_keeps_structural_and_child_vectors_aligned \
  2>&1 | tee /tmp/green-sub-task-replace-alignment-mapsplice-roadmap-4-4-4.out
```

Then run the wider sub-task-focused tests and full repository gate:

```bash
cargo test roadmap::ops::sub_task 2>&1 | tee /tmp/unit-sub-task-ops-mapsplice-roadmap-4-4-4.out
cargo test --test roadmap_sub_tasks 2>&1 | tee /tmp/integration-sub-task-mapsplice-roadmap-4-4-4.out
make all 2>&1 | tee /tmp/make-all-sub-task-replace-alignment-mapsplice-roadmap-4-4-4.out
```

Review before commit:

```bash
sem diff --file-exts .rs
git diff -- src/roadmap/ops/sub_task.rs tests/roadmap_sub_tasks.rs
```

Commit message:

```text
Prove sub-task replace alignment

Add focused replace coverage so replacing a boundary addendum sub-task keeps
the structural vector and task child ordering in sync, including multi-item
replacement fragments.
```

If a production fix was necessary, use this subject instead:

```text
Keep sub-task replace vectors aligned
```

### Work Item 3: Mark roadmap task 4.4.4 complete and run final gates

Read first:

- `docs/roadmap.md` lines 252-258.
- `docs/documentation-style-guide.md`.
- `AGENTS.md` Markdown guidance and committing rules.
- `Makefile` lines 22-24 and 70-81 for the repository-default Markdown
  formatter flags and Markdown lint targets.
- `.markdownlint-cli2.jsonc` lines 5-7 for the MD013 80-column lint rule.
- `docs/execplans/initial-tool.md` lines 80-100 for the accepted quality and
  documentation constraints that still apply to roadmap maintenance.

Load or follow these skills:

- `execplans` to keep this plan current while closing the work.
- `sem` to review the documentation entity diff before committing.

Implementation:

1. Update this ExecPlan's `Progress`, `Surprises & Discoveries`,
   `Decision log`, and `Outcomes & retrospective` sections with the actual test
   names, whether production code changed, and the validation logs.
2. Update `docs/roadmap.md` by changing only task 4.4.4 from `[ ]` to `[x]`.
   Do not edit neighbouring roadmap items.
3. Format only changed Markdown files. If only `docs/roadmap.md` and this plan
   changed, run the scoped Markdown formatter with the repository-default
   `MARKDOWN_FORMAT_FLAGS`:

```bash
MARKDOWN_PATHS='docs/roadmap.md docs/execplans/roadmap-4-4-4.md' \
make markdownfmt 2>&1 | tee /tmp/markdownfmt-roadmap-4-4-4-mapsplice-roadmap-4-4-4.out
```

If a listed Markdown path does not exist, remove it from `MARKDOWN_PATHS`
instead of running the command with a stale path. Do not override
`MARKDOWN_FORMAT_FLAGS` in this work item unless the plan is revised with proof
that the reduced flag set keeps the edited files MD013-clean.

Validation:

```bash
make all 2>&1 | tee /tmp/final-make-all-mapsplice-roadmap-4-4-4.out
make markdownlint 2>&1 | tee /tmp/final-markdownlint-mapsplice-roadmap-4-4-4.out
make nixie 2>&1 | tee /tmp/final-nixie-mapsplice-roadmap-4-4-4.out
```

Review before commit:

```bash
sem diff --file-exts .rs .md
git diff -- docs/roadmap.md docs/execplans/roadmap-4-4-4.md
```

Commit message:

```text
Mark sub-task alignment covered

Record roadmap task 4.4.4 as complete after focused unit tests prove
insert and replace keep sub-task model vectors aligned.
```

## Concrete steps

All commands run from:

```bash
cd /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-4
```

Before Work Item 1, retry the advisory tools and update
`Surprises & Discoveries` with exact results:

```bash
git branch --show-current
leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-4
leta grep 'insert_sub_tasks|replace_sub_task|delete_sub_task|TaskEntry|TaskChild' 'src/roadmap' --head 100
sem diff --from origin/main --to HEAD --file-exts .rs .md --format json
```

If Memtrace is available to the implementing agent, use repo id `mapsplice`
after `list_indexed_repositories` confirms it, then run:

```text
list_communities(repo_id="mapsplice", branch="main", limit=20)
find_code(repo_id="mapsplice", view="committed", query="sub-task splice vector alignment")
find_symbol(repo_id="mapsplice", name="insert_sub_tasks", file_path="src/roadmap/ops/sub_task.rs")
get_symbol_context(repo_id="mapsplice", symbol="insert_sub_tasks", file_path="src/roadmap/ops/sub_task.rs", view="committed")
get_impact(repo_id="mapsplice", target="insert_sub_tasks", direction="both", depth=3)
get_timeline(repo_id="mapsplice", file_path="src/roadmap/ops/sub_task.rs", scope_path="insert_sub_tasks")
```

Repeat the same `find_symbol`, `get_symbol_context`, `get_impact`, and
`get_timeline` sequence for `replace_sub_task`. If any Memtrace call is
unavailable, record the exact failure and continue with bounded local source
inspection.

Execute Work Items 1, 2, and 3 in order. Do not start the next work item until
the current work item's focused test and gate pass, and the commit is made.

## Validation and acceptance

Done means:

- `src/roadmap/ops/sub_task.rs` has focused unit coverage for insert-before,
  insert-after, and replace boundary cases.
- The tests inspect the internal model and fail when `TaskEntry.sub_tasks` and
  filtered `TaskChild::SubTask` identities diverge.
- Any production change, if required, is confined to
  `src/roadmap/ops/sub_task.rs` and preserves public CLI/library behaviour.
- `docs/roadmap.md` marks task 4.4.4 complete after tests and gates pass.
- Final validation commands pass:

```bash
make all 2>&1 | tee /tmp/final-make-all-mapsplice-roadmap-4-4-4.out
make markdownlint 2>&1 | tee /tmp/final-markdownlint-mapsplice-roadmap-4-4-4.out
make nixie 2>&1 | tee /tmp/final-nixie-mapsplice-roadmap-4-4-4.out
```

Expected final success signals:

```text
make all
# exits 0 after check-fmt, lint, typecheck, and test

make markdownlint
# exits 0

make nixie
# exits 0
```

No snapshot, BDD, e2e, CrossHair, Kani, Verus, or mutation-test artefact is
required for this task. The required proof is unit-level model invariant
coverage. `cargo-mutants` may be useful as a follow-up, but it is outside this
roadmap item's acceptance scope unless a reviewer requests it.

## Idempotence and recovery

The test additions are additive and can be rerun safely. If the deliberate red
mutation is applied during Work Item 1 or Work Item 2, restore it before any
commit by editing only the touched function back to the pre-mutation state. Use
`git diff -- src/roadmap/ops/sub_task.rs` to verify the deliberate mutation is
gone.

Do not use a bare `git stash`. If unrelated formatter or build churn appears
and must be parked, use a named stash:

```bash
git stash push -m 'df12-stash v1 task=4.4.4 kind=discard reason="park unrelated formatter churn"'
```

If a gate fails, inspect the corresponding `/tmp` log first. Re-run a gate only
after applying a fix. If `/tmp` or the working filesystem fills up, stop and
escalate.

## Artifacts and notes

Current planning evidence:

```text
branch: roadmap-4-4-4
Memtrace: mcp__memtrace__list_indexed_repositories -> user cancelled MCP tool call
Leta workspace add: Error: IO error: Read-only file system (os error 30)
Leta retry with /tmp XDG dirs: Error: IO error: Read-only file system (os error 30)
Firecrawl docs.rs scrape: user cancelled MCP tool call
sem diff origin/main..HEAD before plan: zero files, zero entities
```

Relevant source anchors:

```text
docs/roadmap.md:252-258
docs/mapsplice-design.md:88-158
docs/developers-guide.md:84-123
src/roadmap/ops/sub_task.rs:13-28
src/roadmap/ops/sub_task.rs:40-53
src/roadmap/ops/sub_task.rs:105-159
tests/roadmap_sub_tasks.rs:61-223
```

## Interfaces and dependencies

Do not introduce new public interfaces. The implementation should use the
existing internal interfaces:

```rust
apply_command(&mut roadmap, RoadmapOperation::Insert { anchor, after }, Some(fragment))?;
apply_command(&mut roadmap, RoadmapOperation::Replace { anchor }, Some(fragment))?;
parse_roadmap(source)?;
parse_fragment(source)?;
parse_anchor("1.1.1.1")?;
```

The only external test API this plan relies on is locked `rstest` 0.26.1:

- `Cargo.toml` declares `rstest = "0.26.1"`.
- `Cargo.lock` resolves `rstest` 0.26.1 and `rstest_macros` 0.26.1.
- The locked README in the local Cargo registry documents `#[rstest]`,
  `#[case(...)]`, and fixture injection. This supports the planned table-driven
  unit tests without new dependencies.

Revision note: second-round ExecPlan revision for roadmap task 4.4.4. It fixes
the Work Item 3 formatter command by removing the
`MARKDOWN_FORMAT_FLAGS='--in-place'` override, preserving the repository default
`--wrap` flag so the subsequent `make markdownlint` gate is executable as
written. Implementation remains pending plan approval.
