# Represent addendum sub-tasks in the roadmap model

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: DRAFT

## Purpose / big picture

Roadmap task 2.1.1 is complete only when addendum sub-tasks are represented as
first-class roadmap model items and a no-op render of a task with nested
`8.2.3.1`-style sub-tasks is byte-identical to the source. The parser must
expose ordered fourth-level sub-tasks owned by their parent task, and the
renderer must preserve their Markdown nesting and indentation exactly for a
conformant no-op round trip.

This is planning round 3. Prior review found that named symbols and containment
tests are not enough evidence for the task's stated success condition. The
current visible tests include structural sub-task tests and order assertions,
but `tests/roadmap_render.rs::render_preserves_task_body_and_sub_task_order`
only checks containment/order and its expected marker contains escaped
`sub\-task`, so it does not prove source-byte identity. This plan therefore
makes an exact nested sub-task round-trip fixture a blocking audit result. If no
existing exact fixture proves the behaviour, the implementer must record red
evidence in this ExecPlan, then land the smallest regression test and any
required implementation fix together in one green, gated commit.

## Constraints

- Work only in `/home/leynos/Projects/mapsplice.worktrees/roadmap-2-1-1`.
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
  starts. Use exact text search only for Markdown, command output, literal
  string checks, and daemon failure fallback evidence.
- Use `sem` for codebase history navigation and entity-level diff review.
- Use `docs/mapsplice-design.md`, `docs/developers-guide.md`,
  `docs/users-guide.md`, `docs/contributing.md`,
  `docs/documentation-style-guide.md`, `docs/scripting-standards.md`,
  `AGENTS.md`, and `docs/roadmap.md` as source-of-truth documents.
- Follow en-GB Oxford spelling in prose, comments, and commit messages.
- Do not add a new external dependency for task 2.1.1.
- Do not make a standalone committed red-test work item. If a missing
  regression is found, record the red evidence here, then commit the test and
  implementation together once green.
- Commit after each file change and gate each commit. If this ExecPlan is
  edited during an audit or implementation work item, format the touched
  Markdown file, run the applicable gates, and commit that edit before moving
  on.
- Format only changed Markdown files with path-safe commands. Do not run
  repository-global Markdown formatting such as `make fmt` or `mdformat-all`.
- Every test, lint, format check, and gate command must be logged with `tee` to
  a branch-specific `/tmp` file.

## Tolerances (exception triggers)

- If `sem diff --from origin/main --to HEAD --format json` reports an
  unexpected semantic code delta before implementation, inspect it and update
  this plan before editing.
- If the existing sub-task symbols are absent in the worktree, stop and rewrite
  this plan around the newly observed state.
- If no exact nested sub-task round-trip test exists and adding one requires
  more than one production module change, record the evidence and stop for
  review before broadening scope.
- If reconciling documentation would require changing the accepted grammar in
  `docs/mapsplice-design.md` section 4, stop and escalate; task 2.1.1 should
  update stale prose to match the accepted design, not redesign the grammar.
- If a work item touches Rust code, load the relevant Rust follow-on skills
  routed by `rust-router` before editing.
- If a work item touches more than six files, split it before implementation.
- If `make all` fails after two focused fix attempts, record the failing
  command and log path in the Decision Log and stop for review.
- If formatter churn touches files outside the work item, park or discard it
  with a named stash following the df12 stash format before proceeding.

## Risks

- Risk: The roadmap task text may be partly behind the implementation, so a
  stale plan could ask for duplicate model work.
  Severity: high.
  Likelihood: already observed.
  Mitigation: work item 1 starts with `sem`, Memtrace when available, and
  `leta` evidence proving the current model surface before any code edit.

- Risk: The code may expose sub-task symbols but still fail the exact
  byte-identity success condition.
  Severity: high.
  Likelihood: high, because current visible tests assert containment/order and
  include escaped `sub\-task` expected strings.
  Mitigation: work item 1 requires an exact existing fixture or explicit red
  evidence, and work item 2 combines the missing regression and fix in one
  green commit if needed.

- Risk: Public documentation remains inconsistent after the code accepts
  fourth-level anchors.
  Severity: high.
  Likelihood: high.
  Mitigation: work item 3 updates `docs/developers-guide.md`,
  `docs/users-guide.md`, `docs/mapsplice-design.md`, and `docs/roadmap.md`
  after exact behaviour is proven.

- Risk: A documentation update could accidentally claim that roadmap tasks
  2.1.2 or 2.1.3 are complete.
  Severity: medium.
  Likelihood: medium.
  Mitigation: mark or describe only 2.1.1 unless a separate audited task
  explicitly authorizes later roadmap items.

- Risk: Memtrace, Firecrawl, or `leta` daemons can be unavailable in the
  sandbox.
  Severity: medium.
  Likelihood: observed.
  Mitigation: record failures with command transcripts and use locked local
  source, official docs URLs, `sem`, bounded file inspection, exact text search,
  and repository gates as fallback evidence.

## Progress

- [x] (2026-06-30) Read `AGENTS.md` and confirmed the branch is
  `roadmap-2-1-1`, so this plan belongs at
  `docs/execplans/roadmap-2-1-1.md`.
- [x] (2026-06-30) Loaded `execplans`, `leta`, `rust-router`, `sem`,
  `firecrawl-mcp`, and `en-gb-oxendict-style` for this planning pass.
- [x] (2026-06-30) Read the existing round-2 plan and the visible source docs:
  `docs/roadmap.md`, `docs/mapsplice-design.md`,
  `docs/developers-guide.md`, `docs/users-guide.md`, and `Makefile`.
- [x] (2026-06-30) Verified that `docs/roadmap.md` task 2.1.1 success requires
  first-class sub-task items and source-byte-identical render output for nested
  `8.2.3.1`-style items.
- [x] (2026-06-30) Verified that current visible tests include structural
  sub-task tests but do not, from inspection alone, prove exact no-op nested
  sub-task round-trip identity.
- [ ] Work item 1: Audit first-class sub-task model and exact round-trip proof.
- [ ] Work item 2: Add the exact round-trip regression and minimal fix if the
  audit finds a gap.
- [ ] Work item 3: Reconcile documentation and roadmap status before final
  gates.

## Surprises & Discoveries

- `docs/roadmap.md` task 2.1.1 explicitly says success requires parsing a task
  with nested `8.2.3.1`-style items into first-class sub-task items and
  rendering them byte-identically to the source.
- `tests/roadmap_render.rs::render_preserves_task_body_and_sub_task_order`
  asserts that body text and a nested sub-task marker appear in order after an
  append command, but it does not assert the full no-op output equals the input.
  Its expected marker is `Nested sub\\-task`, which is evidence that renderer
  escaping may differ from source bytes.
- `tests/roadmap_sub_tasks.rs` covers structural parsing, renumbering,
  dependency rewriting, deletion, and malformed nested-list rejection, but the
  visible assertions are containment checks rather than exact no-op fixture
  comparisons.
- `docs/developers-guide.md` section 3 still says `parse_anchor` accepts only
  `8`, `8.2`, and `8.2.3`; this conflicts with current fourth-level anchor
  support.
- `docs/users-guide.md` still describes only phase, step, and task anchors; it
  must describe addendum sub-tasks and fourth-level anchors if the exact audit
  proves the code accepts and preserves them.
- `docs/mapsplice-design.md` section 9 still records D2 as "Addenda not
  modelled". If the exact audit passes or is fixed, this stale divergence must
  be removed or narrowed without implying follow-on tasks are complete.
- Memtrace `list_indexed_repositories` and Firecrawl `firecrawl_scrape` calls
  were cancelled by the MCP host during planning. Retry them during
  implementation; if they remain unavailable, preserve the cancellation
  transcripts in the plan and rely on locked local source plus official docs
  URLs for the fallback library notes.
- `leta workspace add` failed with a read-only filesystem error during
  planning. Retry `leta` commands from the assigned worktree before code edits;
  if the daemon still cannot start, record the failure and use exact local
  inspection for branch-local evidence.

## Decision Log

- Decision: Treat exact nested sub-task round-trip identity as a blocking
  acceptance criterion, not an inferred consequence of symbol presence.
  Rationale: roadmap task 2.1.1 names byte-identical rendering as success, and
  current visible tests do not prove that property.
  Date/Author: 2026-06-30, planning agent.

- Decision: Do not include a standalone red-test commit.
  Rationale: the standing instruction forbids standalone committed red-test
  work items. If the audit finds no exact fixture, the plan records red
  evidence first, then combines the regression and implementation in one green,
  gated commit.
  Date/Author: 2026-06-30, planning agent.

- Decision: Put all final ExecPlan and roadmap status edits before path-scoped
  Markdown formatting and before `make all`, `make markdownlint`, and
  `make nixie`.
  Rationale: a status edit after final gates would be an ungated Markdown
  change.
  Date/Author: 2026-06-30, planning agent.

- Decision: Do not rely on `markdown 1.0.0` for source-byte-preserving render
  output.
  Rationale: the locked crate source confirms `markdown::to_mdast` parses into
  mdast nodes and exposes list/list-item structure, but the project renderer is
  responsible for emitted bytes. Any byte-identity claim must be pinned by an
  exact fixture test, not by an assumption about the external parser.
  Date/Author: 2026-06-30, planning agent.

## Context and orientation

The design document defines the roadmap grammar in section 4. Phases are
level-2 headings such as `## 8.`, steps are level-3 headings such as
`### 8.2.`, and tasks are checklist list items beginning with numbers such as
`- [ ] 8.2.3.`. A nested list item whose number extends the parent task by one
level, for example `- [ ] 8.2.3.1.`, is an addendum sub-task. Section 6 defines
C4, the addenda contract: addendum sub-tasks are first-class items, renumber
with their parent, and preserve Markdown nesting and indentation on render.
Section 8 says a guarantee without a golden fixture is unproven.

The developers' guide identifies `src/roadmap` as the domain boundary for
parsing, mutation, renumbering, and rendering. It also says new roadmap fields
should prefer typed domain values over raw parser or adapter types.

Branch-local reconnaissance during planning found these relevant files and
symbols by exact local inspection:

- `src/roadmap/anchor.rs`: `SubTaskNumber`, `RoadmapAnchor::SubTask`, and
  `parse_anchor`.
- `src/roadmap/model.rs`: `TaskEntry.sub_tasks`, `TaskChild`, and
  `SubTaskEntry`.
- `src/roadmap/parse/mod.rs`: parser support for sub-task lists and validation.
- `src/roadmap/render.rs`: `render_task` and `render_sub_task`.
- `tests/roadmap_parse.rs`: fourth-level `parse_anchor` cases.
- `tests/roadmap_render.rs`: render preservation tests, including the
  non-exact `render_preserves_task_body_and_sub_task_order`.
- `tests/roadmap_sub_tasks.rs`: structural parser, renumber, delete,
  dependency-rewrite, and malformed sub-task tests.

The locked external parser dependency is `markdown 1.0.0` in `Cargo.lock`.
Local source
`/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/lib.rs`
defines `to_mdast(value: &str, options: &ParseOptions) -> Result<mdast::Node,
message::Message>`. Local source
`/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/mdast.rs`
defines `Node::List(List)`, `List { children, ordered, start, spread }`, and
`ListItem { children, spread, checked }`. Official docs URLs to verify when
Firecrawl or browser access is available are
`https://docs.rs/markdown/1.0.0/markdown/fn.to_mdast.html` and
`https://docs.rs/markdown/1.0.0/markdown/mdast/index.html`.

## Plan of work

### Work item 1: Audit first-class sub-task model and exact round-trip proof

Documentation to read: `AGENTS.md`; `docs/roadmap.md` task 2.1.1;
`docs/mapsplice-design.md` sections 4, 6, 8, and 9;
`docs/developers-guide.md` sections 2, 3, and 6; `docs/contributing.md`.

Skills to load: `execplans`, `leta`, `sem`, `rust-router`,
`rust-types-and-apis`, `rust-unit-testing`, `firecrawl-mcp`, and
`en-gb-oxendict-style`. If generated tests become necessary, also load
`rust-verification` and `proptest` before designing them.

First, confirm the branch and canonical context:

```bash
cd /home/leynos/Projects/mapsplice.worktrees/roadmap-2-1-1
git branch --show-current \
  2>&1 | tee /tmp/branch-mapsplice-roadmap-2-1-1.out
sem diff --from origin/main --to HEAD --format json \
  2>&1 | tee /tmp/sem-diff-mapsplice-roadmap-2-1-1.out
```

Then call Memtrace `list_indexed_repositories`. If it confirms repo id
`mapsplice`, use `find_symbol` or `find_code` for `SubTaskNumber`,
`RoadmapAnchor`, `TaskEntry`, `SubTaskEntry`, `render_sub_task`, and
`parse_anchor`. For any symbol that needs editing, call `get_symbol_context`,
`get_impact`, and `get_timeline` before changing it. If the MCP host cancels
again, record the cancellation in `Surprises & Discoveries` and continue with
branch-local evidence.

Verify the branch-local symbols and references with `leta` if available:

```bash
leta grep 'SubTask|sub_task|sub-task' 'src/roadmap|tests' --head 200 \
  2>&1 | tee /tmp/leta-subtask-surface-mapsplice-roadmap-2-1-1.out
leta show TaskEntry \
  2>&1 | tee /tmp/leta-taskentry-mapsplice-roadmap-2-1-1.out
leta show RoadmapAnchor \
  2>&1 | tee /tmp/leta-roadmap-anchor-mapsplice-roadmap-2-1-1.out
leta show parse_anchor \
  2>&1 | tee /tmp/leta-parse-anchor-mapsplice-roadmap-2-1-1.out
```

If `leta` cannot start, record the failure and use bounded file inspection for
the named files in `Context and orientation`.

Next, audit for an exact no-op nested sub-task fixture. Search for tests that
parse or run the CLI on an input containing a nested `8.2.3.1`-style checklist
item and assert equality of the full rendered output against the full source.
Containment checks, order checks, symbol presence, or escaped-marker assertions
do not satisfy this audit.

Focused commands:

```bash
cargo test --workspace --all-targets --all-features roadmap_sub_tasks \
  2>&1 | tee /tmp/test-roadmap-sub-tasks-mapsplice-roadmap-2-1-1.out
cargo test --workspace --all-targets --all-features render_preserves_task_body_and_sub_task_order \
  2>&1 | tee /tmp/test-render-subtask-order-mapsplice-roadmap-2-1-1.out
```

Acceptance for this work item:

- The model audit confirms `TaskEntry` owns ordered first-class
  `SubTaskEntry` values, or this plan is rewritten around the missing model.
- An existing exact nested sub-task no-op fixture is identified by test name and
  passing command, or the absence of such a fixture is recorded here as red
  evidence before work item 2 begins.
- If this ExecPlan is edited to record evidence, run:

  ```bash
  mdtablefix docs/execplans/roadmap-2-1-1.md \
    2>&1 | tee /tmp/mdtablefix-execplan-audit-mapsplice-roadmap-2-1-1.out
  markdownlint-cli2 --fix docs/execplans/roadmap-2-1-1.md \
    2>&1 | tee /tmp/markdownlint-fix-execplan-audit-mapsplice-roadmap-2-1-1.out
  make all 2>&1 | tee /tmp/make-all-audit-mapsplice-roadmap-2-1-1.out
  make markdownlint \
    2>&1 | tee /tmp/make-markdownlint-audit-mapsplice-roadmap-2-1-1.out
  make nixie 2>&1 | tee /tmp/make-nixie-audit-mapsplice-roadmap-2-1-1.out
  git add docs/execplans/roadmap-2-1-1.md
  git commit -m "Record sub-task round-trip audit"
  ```

If the audit produces no file edits, there is no commit for this work item; the
plan explicitly permits that no-change outcome. Do not edit this ExecPlan and
then continue without formatting, gating, and committing the edit.

### Work item 2: Add the exact round-trip regression and minimal fix if needed

Documentation to read: `AGENTS.md`; `docs/mapsplice-design.md` sections 5, 6,
and 8; `docs/developers-guide.md` sections 2 and 6;
`docs/rust-testing-with-rstest-fixtures.md`; `docs/contributing.md`.

Skills to load: `execplans`, `leta`, `sem`, `rust-router`,
`rust-unit-testing`, `rust-types-and-apis`, and `en-gb-oxendict-style`. If the
fix touches renderer error handling, also load `rust-errors`.

Skip this work item only if work item 1 identified and ran an existing exact
nested sub-task no-op fixture. Otherwise, add the smallest exact regression.
The preferred shape is a named `rstest` in `tests/roadmap_render.rs` or
`tests/roadmap_sub_tasks.rs` that uses a source string containing:

```markdown
# Example

## 8. Phase one

### 8.2. Step one

- [ ] 8.2.3. Parent task.
    Body before.
    - [ ] 8.2.3.1. Nested sub-task.
    Body after.
```

The test must perform a no-op render path that exercises parser plus renderer
and then assert `assert_eq!(rendered, source)`. If the current public API has no
direct no-op render helper, use the narrowest existing CLI or library path that
can prove a no-op render. If no such path exists, add a small test-only helper
or public-neutral internal helper rather than broadening the CLI contract.

Red evidence before implementation:

```bash
cargo test --workspace --all-targets --all-features exact_nested_sub_task_round_trip \
  2>&1 | tee /tmp/red-exact-subtask-roundtrip-mapsplice-roadmap-2-1-1.out
```

Expected red result: the new exact test fails because rendered output is not
byte-identical, or because no suitable no-op render path exists. Record the
short failure excerpt in this ExecPlan before applying the implementation fix.

Make the smallest code change required to satisfy the exact fixture. Likely
surfaces, to be verified with `leta` and Memtrace before editing, are
`src/roadmap/render.rs::render_task`, `src/roadmap/render.rs::render_sub_task`,
and the parse/render value objects in `src/roadmap/model.rs`. Do not change
the accepted grammar unless section 4 of the design document is wrong and the
task is escalated.

Green and commit gates:

```bash
cargo test --workspace --all-targets --all-features exact_nested_sub_task_round_trip \
  2>&1 | tee /tmp/green-exact-subtask-roundtrip-mapsplice-roadmap-2-1-1.out
cargo test --workspace --all-targets --all-features roadmap_sub_tasks \
  2>&1 | tee /tmp/test-roadmap-sub-tasks-green-mapsplice-roadmap-2-1-1.out
make all 2>&1 | tee /tmp/make-all-roundtrip-mapsplice-roadmap-2-1-1.out
git diff --check \
  2>&1 | tee /tmp/git-diff-check-roundtrip-mapsplice-roadmap-2-1-1.out
git add tests/roadmap_render.rs tests/roadmap_sub_tasks.rs src/roadmap/render.rs src/roadmap/model.rs
git commit -m "Prove nested sub-task round trips"
```

Before staging, remove any file from `git add` that was not edited. If the
ExecPlan was edited to record red/green evidence in this work item, format and
gate `docs/execplans/roadmap-2-1-1.md` before the same commit, or make a
separate plan-evidence commit after rerunning the Markdown gates.

### Work item 3: Reconcile documentation and roadmap status before final gates

Documentation to read: `docs/mapsplice-design.md` sections 4, 6, 8, and 9;
`docs/users-guide.md`; `docs/developers-guide.md` sections 1, 2, 3, and 6;
`docs/documentation-style-guide.md`; `docs/scripting-standards.md`;
`docs/roadmap.md` task 2.1.1; this ExecPlan's validation section.

Skills to load: `execplans`, `sem`, `leta`, `code-review`,
`en-gb-oxendict-style`, and `commit-message`. Rust follow-on skills are not
needed if this work item remains documentation-only.

Update `docs/developers-guide.md` section 3 so `parse_anchor` is documented as
accepting canonical positive anchors `8`, `8.2`, `8.2.3`, and `8.2.3.1`, and
so the public API section mentions fourth-level sub-task anchors only to the
extent the current exported API exposes them.

Update `docs/users-guide.md` so the roadmap shape and anchor list describe
addendum sub-tasks as nested numbered checklist items under tasks. The guide
must distinguish ordinary nested bullets, which remain body Markdown, from
addendum sub-tasks, which are fourth-level roadmap items.

Update `docs/mapsplice-design.md` section 9 so it no longer falsely says
addenda are not modelled after work item 1 or 2 proves first-class modelling
and exact render preservation. If follow-on tasks 2.1.2 or 2.1.3 remain
incomplete after the audit, narrow the divergence to that exact remaining
behaviour instead of making a blanket D2 claim.

Update `docs/roadmap.md` only for task 2.1.1. Mark it complete only after the
exact round-trip audit or regression, documentation edits, and gates are ready
to run. Do not mark 2.1.2 or 2.1.3 complete from this plan unless the user
explicitly retargets the task.

Update this ExecPlan's `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` before formatting and before all final gates.
This sequencing is mandatory: no final plan-status or roadmap-status edit may
be made after the final gate run unless the relevant Markdown formatting and
all gates are rerun.

Path-safe formatting for the Markdown files this work item edits:

```bash
mdtablefix \
  docs/execplans/roadmap-2-1-1.md \
  docs/developers-guide.md \
  docs/users-guide.md \
  docs/mapsplice-design.md \
  docs/roadmap.md \
  2>&1 | tee /tmp/mdtablefix-docs-mapsplice-roadmap-2-1-1.out
markdownlint-cli2 --fix \
  docs/execplans/roadmap-2-1-1.md \
  docs/developers-guide.md \
  docs/users-guide.md \
  docs/mapsplice-design.md \
  docs/roadmap.md \
  2>&1 | tee /tmp/markdownlint-fix-docs-mapsplice-roadmap-2-1-1.out
```

All listed paths exist in the repository. If this work item does not edit one
of them, remove that path from the formatter command before running it. Do not
include optional files or deleted files in formatter commands.

Final review and gates, after every status edit is complete:

```bash
sem diff --format json \
  2>&1 | tee /tmp/sem-diff-final-mapsplice-roadmap-2-1-1.out
git diff --check \
  2>&1 | tee /tmp/git-diff-check-final-mapsplice-roadmap-2-1-1.out
make all 2>&1 | tee /tmp/make-all-final-mapsplice-roadmap-2-1-1.out
make markdownlint \
  2>&1 | tee /tmp/make-markdownlint-final-mapsplice-roadmap-2-1-1.out
make nixie 2>&1 | tee /tmp/make-nixie-final-mapsplice-roadmap-2-1-1.out
git status --short \
  2>&1 | tee /tmp/git-status-final-mapsplice-roadmap-2-1-1.out
```

Commit this work item only after the gates pass. A suitable subject is:

```plaintext
Document addendum sub-task support
```

## Validation and acceptance

Roadmap task 2.1.1 is accepted when all of these are true:

- `sem diff --from origin/main --to HEAD --format json` has been inspected, so
  the implementer knows whether the branch is still code-identical to main or
  contains only the intended task delta.
- The code surface contains `SubTaskNumber`, `RoadmapAnchor::SubTask`,
  `TaskEntry.sub_tasks`, `TaskChild`, `SubTaskEntry`, parser support, renderer
  support, and named sub-task tests.
- An exact no-op fixture for a task with nested `8.2.3.1`-style sub-tasks
  passes and asserts full rendered output equals full source bytes. A
  containment-only or order-only assertion does not satisfy this criterion.
- If that exact fixture did not exist before implementation, this ExecPlan
  contains the red failure excerpt and the combined regression/fix commit
  passes the focused green command.
- `docs/developers-guide.md` documents fourth-level anchors in `parse_anchor`.
- `docs/users-guide.md` documents addendum sub-tasks and the fourth-level
  anchor form.
- `docs/mapsplice-design.md` no longer contains the stale blanket claim that
  addenda are not modelled once the exact audit or fix proves they are.
- `docs/roadmap.md` marks only task 2.1.1 complete, if and only if the audit,
  exact fixture, docs, and gates pass.
- All command evidence for tests, lints, format checks, and gates is captured
  with `tee` in `/tmp`.
- Final status edits are complete before the final gate sequence.
- Final gates pass:

  ```bash
  make all 2>&1 | tee /tmp/make-all-final-mapsplice-roadmap-2-1-1.out
  make markdownlint \
    2>&1 | tee /tmp/make-markdownlint-final-mapsplice-roadmap-2-1-1.out
  make nixie 2>&1 | tee /tmp/make-nixie-final-mapsplice-roadmap-2-1-1.out
  ```

## Idempotence and recovery

All edits are ordinary source or documentation edits and can be retried after a
failed focused command. If a formatter modifies files outside the current work
item, do not commit that churn. Park or discard it with a named stash such as:

```bash
git stash push -u -m 'df12-stash v1 task=2.1.1 kind=discard reason="formatter-churn"'
```

Never use a bare `git stash`. Do not run `git reset --hard` or
`git checkout --` unless explicitly instructed by the user.

If a red test is recorded in this plan, remove any temporary expected-failure
marker before committing the passing implementation. The committed state must
be green.

## Artifacts and notes

Locked-library research completed during planning:

- `Cargo.lock` pins `markdown` to version `1.0.0`.
- Local source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/lib.rs`
  defines `to_mdast(value: &str, options: &ParseOptions)` returning
  `mdast::Node`.
- Local source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/mdast.rs`
  defines `Node::List(List)`, `List` fields `children`, `ordered`, `start`,
  and `spread`, and `ListItem` fields `children`, `spread`, and `checked`.
- Official docs to verify when web tooling is available:
  `https://docs.rs/markdown/1.0.0/markdown/fn.to_mdast.html` and
  `https://docs.rs/markdown/1.0.0/markdown/mdast/index.html`.

Branch-local evidence collected during planning:

- `docs/roadmap.md` lines for task 2.1.1 require first-class sub-task items and
  byte-identical rendering.
- `tests/roadmap_render.rs` contains `render_preserves_task_body_and_sub_task_order`,
  which checks containment/order and expects escaped `sub\\-task`; it is not an
  exact source-byte identity assertion.
- `tests/roadmap_sub_tasks.rs` contains structural parser, renumber, delete,
  dependency-rewrite, and malformed nested-list tests.
- `docs/developers-guide.md`, `docs/users-guide.md`, and
  `docs/mapsplice-design.md` contain stale three-level or D2 prose that must be
  reconciled only after exact behaviour is proven.

## Outcomes & Retrospective

No implementation has begun. This round revises the plan so task 2.1.1 cannot
be marked complete by symbol presence alone. The plan now requires exact nested
sub-task no-op byte identity, commits every ExecPlan edit after formatting and
gates, and places final status edits before final validation.

## Revision note

Round 3 resolves the design-review blockers:

1. The plan now treats exact nested `8.2.3.1`-style no-op round-trip identity
   as a blocking acceptance criterion. Work item 1 must find and run an exact
   existing fixture or record the missing fixture as red evidence; work item 2
   then combines the regression and any required implementation fix in one
   green, gated commit.
2. Work item 1 no longer permits editing the ExecPlan and continuing without a
   commit. It says that if no file is edited, there is no commit; if the
   ExecPlan is edited to record evidence, it must be path-formatted, gated, and
   committed before moving on.
3. Work item 3 now requires final ExecPlan and roadmap status edits before
   path-scoped Markdown formatting and before `make all`, `make markdownlint`,
   and `make nixie`. Any later status edit requires rerunning the relevant
   formatting and gates.
