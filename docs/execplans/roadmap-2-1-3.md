# Preserve sub-task nesting and indentation on render

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

## Purpose / big picture

Roadmap task 2.1.3 is complete when rendering a conformant roadmap with nested
addendum sub-tasks preserves their parent-list nesting and four-space
indentation. A reader should see a nested checklist item remain indented four
spaces beneath its parent task after a parse-render round trip or a structural
operation that does not target that sub-task block.

Planning-round-2 evidence on 2026-07-01 shows current branch `HEAD` at
`db53e09` and `origin/main` at `b502142`. The intentional branch delta before
implementation is this added ExecPlan at `docs/execplans/roadmap-2-1-3.md`;
there are no planned production-code or roadmap-status deltas yet. The existing
unit test `roadmap::render::render_tests::exact_nested_sub_task_round_trip`
passed in planning round 1. This plan therefore does not prescribe a
speculative renderer rewrite. It hardens the missing user-facing proof, records
the verified mechanism, and marks roadmap 2.1.3 complete only after the
repository gates pass. If execution finds that the focused evidence has
regressed, the tolerance below requires a minimal renderer fix in the same work
item as the failing regression test.

## Constraints

- Work only in `/home/leynos/Projects/mapsplice.worktrees/roadmap-2-1-3`.
- Do not edit the root/control worktree.
- Use absolute paths for every `apply_patch` hunk if an edit tool is not scoped
  by the assigned worktree.
- Treat `origin/main` as canonical and check branch skew before editing.
- Use Memtrace as the primary canonical-main search tool. First call
  `list_indexed_repositories`; proceed with repo id `mapsplice` only if that
  call confirms it. If the MCP host cancels or fails, record the exact failure
  and continue with bounded branch-local evidence.
- Use `leta` for branch-local symbol navigation and references when it is
  available. If `leta` fails to start or cannot register this worktree, record
  the exact failure and use precise local inspection for this task.
- Use `sem` for codebase history navigation and entity-level diff review
  instead of raw `git log` or `git blame`.
- Use `docs/mapsplice-design.md`, `docs/developers-guide.md`,
  `docs/users-guide.md`, `docs/contributing.md`,
  `docs/documentation-style-guide.md`, `docs/scripting-standards.md`,
  `AGENTS.md`, and `docs/roadmap.md` as source-of-truth documents.
- Follow en-GB Oxford spelling in prose, comments, and commit messages.
- Do not add a new external dependency for task 2.1.3.
- Do not redesign the roadmap grammar. This task implements the existing
  addenda contract in `docs/mapsplice-design.md` sections 4, 5, 6, and 8.
- Format only changed Markdown files with path-specific formatter commands.
  Do not run repository-global Markdown formatting such as `make fmt` or
  `mdformat-all`.
- Every test, lint, format check, and gate command must be logged with `tee` to
  a branch-specific `/tmp` file.
- Commit after each work item that changes files, and gate each commit.

## Tolerances (exception triggers)

- If `git branch --show-current` is not `roadmap-2-1-3`, stop before editing.
- Before implementation work starts, `sem diff --from origin/main --to HEAD
  --format json` may report only the intentional added ExecPlan at
  `docs/execplans/roadmap-2-1-3.md`. If it reports any source-code delta,
  roadmap-status delta, or other unexpected file, inspect it and update this
  plan before editing.
- If Memtrace or `leta` are unavailable, do not mark this plan blocked. Record
  the exact failure in `Surprises & Discoveries` and continue with bounded
  local evidence.
- If the exact nested sub-task round-trip test is absent, first add it back in
  `src/roadmap/render_tests.rs`; do not proceed using only containment tests.
- If the exact round-trip or the new integration regression fails, limit
  production changes to `src/roadmap/render.rs` and
  `src/roadmap/render_text.rs`. Stop for review before touching parser,
  renumbering, CLI, or public API code.
- If a renderer fix changes more than two production functions or more than
  one production module, stop and update the plan with the newly discovered
  mechanism before editing further.
- If any work item touches more than four files, split it before committing.
- If `make all` fails after two focused fix attempts, record the failing
  command and log path in `Decision Log` and stop for review.
- If formatter churn touches files outside the work item, park or discard it
  with a named stash following the required
  `df12-stash v1 task=2.1.3 kind=<discard|park|keep> reason="<short>"`
  format before proceeding.

## Risks

- Risk: Existing unit coverage can pass while the compiled binary still
  changes nested sub-task indentation during a real command.
  Severity: medium.
  Likelihood: medium.
  Mitigation: add a CLI-level regression that compares the preserved nested
  block exactly, not just by containment or ordering.

- Risk: A speculative renderer rewrite could break the already passing exact
  round-trip contract.
  Severity: high.
  Likelihood: medium.
  Mitigation: use the current mechanism unless a focused test fails; only then
  patch the smallest renderer surface.

- Risk: The `markdown` crate parses mdast with source positions but does not
  provide the full Markdown writer this fidelity contract needs.
  Severity: high.
  Likelihood: verified.
  Mitigation: keep using `markdown::to_mdast()` only for parsing and preserve
  rendering through mapsplice's deterministic renderer and original-block
  storage.

- Risk: Tooling required by the workflow can be unavailable in this sandbox.
  Severity: medium.
  Likelihood: observed.
  Mitigation: record Memtrace, Firecrawl, and Leta failures exactly and rely on
  locked local source, `sem`, focused tests, and repository gates as fallback
  evidence.

## Progress

- [x] (2026-07-01T10:58:56Z) Read `AGENTS.md` and confirmed the branch is
  `roadmap-2-1-3`, so this plan belongs at
  `docs/execplans/roadmap-2-1-3.md`.
- [x] (2026-07-01T10:58:56Z) Loaded `execplans`, `leta`, `sem`,
  `firecrawl-mcp`, `rust-router`, `rust-unit-testing`, `rust-errors`, and
  `rust-types-and-apis` for planning round 1.
- [x] (2026-07-01T11:34:00Z) Loaded `execplans`, `leta`, `sem`,
  `firecrawl-mcp`, `rust-router`, `rust-unit-testing`, `rust-errors`,
  `rust-types-and-apis`, and `en-gb-oxendict-style` for planning round 2.
- [x] (2026-07-01T10:58:56Z) Read the source-of-truth docs: `docs/roadmap.md`,
  `docs/mapsplice-design.md`, `docs/developers-guide.md`,
  `docs/users-guide.md`, `docs/contributing.md`,
  `docs/documentation-style-guide.md`, `docs/scripting-standards.md`, and
  `docs/execplans/initial-tool.md`.
- [x] (2026-07-01T10:58:56Z) Planning round 1 verified that
  `sem diff --from origin/main --to HEAD --format json` reported no semantic
  delta, and `HEAD` equalled `origin/main` at `b502142`.
- [x] (2026-07-01T11:34:00Z) Planning round 2 verified that `HEAD` is
  `db53e09`, `origin/main` is `b502142`, and the semantic branch delta is the
  intentional added ExecPlan at `docs/execplans/roadmap-2-1-3.md`.
- [x] (2026-07-01T11:34:00Z) Planning round 2 formatted and gated this
  ExecPlan revision with `mdtablefix docs/execplans/roadmap-2-1-3.md`,
  `markdownlint-cli2 --fix docs/execplans/roadmap-2-1-3.md`, `make all`,
  `make markdownlint`, and `make nixie`.
- [x] (2026-07-01T10:58:56Z) Verified focused baseline tests:
  `cargo test --workspace --all-targets --all-features
  exact_nested_sub_task_round_trip` passed, and
  `cargo test --workspace --all-targets --all-features --test roadmap_render
  render_preserves_task_body_and_sub_task_order` passed.
- [x] (2026-07-01T11:19:48Z) Work item 1 reconfirmed that the branch is
  `roadmap-2-1-3`, the semantic branch delta remains the intentional
  ExecPlan-only addition, the exact nested unit test passes, the current
  CLI-level ordering test passes, and the locked dependency/source facts still
  support the deterministic renderer mechanism.
- [x] (2026-07-01T11:56:25Z) Work item 1 deterministic gates passed on
  scrutineer rerun: `make all`, `make markdownlint`, and `make nixie`.
  CodeRabbit did not complete a review payload after the initial attempt and
  one retry; both attempts stopped at `connecting_to_review_service`, so the
  deferred review is recorded as an open issue for the supervisor.
- [x] (2026-07-01T12:12:48Z) Work item 2 added the CLI-level exact nested
  sub-task render regression in `tests/roadmap_render.rs`. The focused test
  passed before any production-code edit, so no renderer change was required.
  Deterministic gates passed after an import-order formatting fix and a
  transient `make nixie` retry. CodeRabbit again stalled at
  `connecting_to_review_service` and emitted no actionable findings. A
  post-plan-update gate rerun also passed after additional transient `make
  nixie` retries.
- [x] (2026-07-01T12:17:01Z) Work item 3 marked only roadmap task 2.1.3
  complete in `docs/roadmap.md` and updated this ExecPlan with final
  validation evidence. Adjacent tasks 2.1.2, 3.1.1, 3.1.2, and 3.1.3 remain
  incomplete.
- [x] (2026-07-01T12:33:47Z) Work item 3 deterministic gates passed:
  `make all`, `make markdownlint`, and `make nixie`. CodeRabbit again stalled
  at `connecting_to_review_service`, emitted no findings payload, and exited
  via interrupt status 130.

## Surprises & discoveries

- Memtrace `list_indexed_repositories` failed during planning with
  `user cancelled MCP tool call`. Canonical-main graph context was therefore
  unavailable in this planning round.
- `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-2-1-3`
  and `leta workspace info` failed with
  `Error: IO error: Read-only file system (os error 30)`. Branch-local
  verification used exact local inspection and focused tests.
- Firecrawl `firecrawl_scrape` for
  `https://docs.rs/markdown/1.0.0/markdown/fn.to_mdast.html` failed with
  `user cancelled MCP tool call`. The same failure occurred for
  `https://docs.rs/rstest/0.26.1/rstest/attr.rstest.html`. External API claims
  in this plan are therefore pinned to locked crate source in the local Cargo
  registry.
- After `leta workspace add` failed, `leta files src/roadmap` also failed with
  `Error: Failed to start daemon`. Branch-local verification used exact text
  search and bounded local source inspection for this planning revision.
- The locked `markdown` crate version is 1.0.0. Its local source exposes
  `markdown::to_mdast(value: &str, options: &ParseOptions)` in
  `src/lib.rs`, and its mdast nodes carry position data via `mdast.rs` and
  `unist::Position`. The crate source and crate-level docs describe parsing,
  HTML output, and mdast access, not a load-bearing Markdown writer API.
- The locked `insta` version is 1.48.0, but task 2.1.3 does not need it:
  exact `assert_eq!` string comparisons are clearer and avoid snapshot review
  workflow for a small Markdown block.
- The locked `rstest` version is 0.26.1 and supports `#[rstest]` plus named
  `#[case]` inputs. Use it for integration tests that vary fixtures.
- Existing render code uses `TaskChild` to replay task body and sub-task
  children in source order, `render_sub_task` to render a sub-task checklist
  item, and `indent_block(..., 4)` in `render_task` to nest the rendered
  sub-task under its parent.
- During implementation work item 1, Memtrace
  `mcp__memtrace.list_indexed_repositories` again returned
  `user cancelled MCP tool call`, so canonical-main graph context remained
  unavailable and branch-local evidence was bounded to `sem`, local source
  inspection, and focused tests.
- During implementation work item 1, `leta workspace add
  /home/leynos/Projects/mapsplice.worktrees/roadmap-2-1-3` failed with
  `Error: IO error: Read-only file system (os error 30)`, and `leta show
  render_task`, `leta show render_sub_task`, and `leta show indent_block`
  each failed with `Error: Failed to start daemon`.
- `make nixie` timed out once in scrutineer's first work item 1 gate run while
  rendering `docs/rstest-bdd-users-guide.md` diagram 1. A local retry and a
  scrutineer rerun both passed without documentation edits, so this was
  treated as transient gate flakiness rather than source drift.
- CodeRabbit work item 1 review attempt 1 exited with status 130 after
  logging only `review_context` and
  `{"type":"status","phase":"connecting","status":"connecting_to_review_service"}`.
  Retry 1 produced the same connection-phase output and no completion payload,
  rate-limit message, or actionable finding.
- During implementation work item 2, Memtrace `find_symbol` for `render_task`
  with repo id `mapsplice` returned `user cancelled MCP tool call`. Leta
  `show` commands for `render_task`, `render_sub_task`, and `indent_block`
  again failed with `Error: Failed to start daemon`.
- Work item 2 first deterministic gate run failed `make all` at
  `cargo fmt --all -- --check` because `tests/roadmap_render.rs` import order
  needed `use workspace_support::{TestResult, Workspace, workspace};`. The
  exact import-order fix made `make check-fmt` pass.
- Work item 2 `make nixie` timed out once on
  `docs/rstest-bdd-users-guide.md` diagram 1 during a scrutineer rerun. A
  local retry and final scrutineer rerun passed without edits, matching the
  work item 1 transient diagram-render behaviour.
- After the final work item 2 ExecPlan update, `make nixie` timed out once on
  `docs/ortho-config-users-guide.md` diagram 1 and once locally on
  `docs/rstest-bdd-users-guide.md` diagram 1. A second local retry and a final
  scrutineer rerun passed without edits, confirming repeated Mermaid-render
  flakiness rather than source drift.
- CodeRabbit work item 2 review again failed to reach a review payload. The
  initial command stalled at `connecting_to_review_service`; the scrutineer
  retry with a 120-second timeout exited 124 after logging the same
  connection-phase status and no rate-limit message.
- CodeRabbit work item 3 review failed the same way as earlier work items: it
  logged `review_context` and
  `{"type":"status","phase":"connecting","status":"connecting_to_review_service"}`
  before being interrupted with status 130. No rate-limit, deferred-review, or
  completed review-result message was emitted.

## Decision log

- Decision: Do not prescribe a renderer rewrite in the initial plan.
  Rationale: The current exact nested sub-task round-trip unit test passes on
  the assigned branch. Planning round 2 distinguishes the intentional
  plan-only branch delta from unexpected code deltas; rewriting a passing
  mechanism would add risk without evidence.
  Date/Author: 2026-07-01T10:58:56Z / planning agent.

- Decision: Treat the current branch delta as plan-only until implementation
  starts.
  Rationale: The design review identified stale baseline evidence. Current
  `HEAD` is `db53e09`, `origin/main` is `b502142`, and `sem diff --from
  origin/main --to HEAD --format json` shows the added ExecPlan. Work item 1
  now explicitly verifies that any extra source or roadmap-status delta is
  unexpected and must be resolved before implementation.
  Date/Author: 2026-07-01T11:34:00Z / planning agent.

- Decision: Prove the user-facing surface with an exact CLI-level regression
  before marking roadmap task 2.1.3 complete.
  Rationale: `docs/mapsplice-design.md` section 8 requires golden comparison
  for render fidelity end to end, and the current integration test checks
  ordering rather than exact block identity.
  Date/Author: 2026-07-01T10:58:56Z / planning agent.

- Decision: Use local deterministic rendering rather than any external
  Markdown writer.
  Rationale: `docs/execplans/initial-tool.md` decisions require a constrained
  roadmap renderer, and the locked `markdown` crate source verifies parsing and
  mdast support but no full Markdown writer API that satisfies F1, F3, and C4.
  Date/Author: 2026-07-01T10:58:56Z / planning agent.

- Decision: Proceed past work item 1 with a deferred CodeRabbit review issue.
  Rationale: Deterministic gates are green and CodeRabbit emitted no review
  payload on two attempts, only the connection-phase status
  `connecting_to_review_service`. There was no rate-limit backoff directive to
  obey and no actionable feedback to fix.
  Date/Author: 2026-07-01T11:56:25Z / implementation agent.

- Decision: Keep work item 2 test-only after the new CLI regression passed on
  the baseline.
  Rationale: The new exact block test proved the user-facing append workflow
  preserves nested sub-task indentation without changing renderer code. Editing
  production code after that pass would violate the plan's minimal-change
  direction.
  Date/Author: 2026-07-01T12:12:48Z / implementation agent.

- Decision: Mark only roadmap task 2.1.3 complete.
  Rationale: Work item 2 supplied the missing CLI-level proof and the final
  status work is scoped to the render-indentation task. The adjacent renumber
  and fixture-corpus tasks depend on separate roadmap work and remain
  unchecked.
  Date/Author: 2026-07-01T12:17:01Z / implementation agent.

- Decision: Leave CodeRabbit review deferred for the supervisor.
  Rationale: All deterministic gates pass and every CodeRabbit attempt across
  the three work items stalled at `connecting_to_review_service` without a
  rate-limit backoff directive or actionable review payload. There is no local
  code or documentation finding to address.
  Date/Author: 2026-07-01T12:33:47Z / implementation agent.

## Outcomes & retrospective

No implementation has started. Planning round 2 revised the draft to correct
stale baseline evidence and to make Markdown formatting and gates explicit in
work item 2 when the living ExecPlan is updated with new evidence.

Work item 1 completed the implementation-session evidence pass without source
changes. The current renderer mechanism remains the same: `TaskChild` preserves
task child order, `render_task` emits sub-tasks through `render_sub_task`, and
`indent_block(..., 4)` keeps the nested checklist item under the parent task.
The work item 1 deterministic gates are green. CodeRabbit review is deferred
because the review service did not advance beyond the connection phase.

Work item 2 added the exact CLI-level regression without production-code
changes. The new test passed immediately, proving the existing renderer already
preserves the full four-line nested parent task block byte-for-byte through
the compiled append workflow. Work item 2 deterministic gates are green after
the formatting fix and transient `make nixie` retry; CodeRabbit review is
deferred because the review service did not advance beyond the connection
phase.

Work item 3 completed the roadmap status update by checking only task 2.1.3 in
`docs/roadmap.md`. Final gate evidence is recorded below after path-specific
Markdown formatting and repository validation.
The ExecPlan is complete with all three work items committed or ready to
commit, deterministic gates green, and CodeRabbit review deferred because the
service connection did not complete.

## Context and orientation

Roadmap task 2.1.3 lives in `docs/roadmap.md` under "2.1. Renumber and render
addendum sub-tasks faithfully". It requires sub-tasks to render at the correct
nesting depth without breaking out of their parent list. The design sources
behind this are:

- `docs/mapsplice-design.md` section 4, "The roadmap grammar", which defines
  addendum sub-tasks as nested checklist items whose number extends the parent
  task by one level.
- `docs/mapsplice-design.md` section 5, "Fidelity guarantees", especially F1
  content preservation, F3 round-trip stability, and F4 gate-clean output.
- `docs/mapsplice-design.md` section 6, "Functional and contract guarantees",
  especially C4 addenda contract.
- `docs/mapsplice-design.md` section 8, "Fixture and test requirements",
  especially the required addendum render fidelity fixture.
- `docs/developers-guide.md` section 2, which places parsing, mutation,
  renumbering, and rendering under `src/roadmap`.
- `docs/developers-guide.md` section 6, which requires `rstest` unit tests,
  `rstest-bdd` behavioural tests, properties, `trybuild`, and `insta` where
  their shape fits.
- `docs/users-guide.md`, "The roadmap shape `mapsplice` expects", which
  documents addendum sub-tasks and fourth-level anchors for users.
- `docs/execplans/initial-tool.md` sections "Scope and grammar assumptions",
  "5. Render the supported roadmap grammar deterministically", and "Decision
  Log", which reject arbitrary Markdown surgery and establish a deterministic
  roadmap renderer.

The relevant implementation surface is small:

- `src/roadmap/model.rs` defines `TaskEntry`, `SubTaskEntry`, and `TaskChild`.
  `TaskEntry.children` preserves the original order of task body blocks and
  sub-task identities.
- `src/roadmap/parse/mod.rs` splits task children into body blocks and
  first-class sub-tasks through `split_task_children`, `parse_sub_task_list`,
  and `parse_sub_task_item`.
- `src/roadmap/render.rs` contains `render_task`, `render_sub_task`,
  `render_nested_body`, and `render_item_summary`.
- `src/roadmap/render_text.rs` contains `indent_block`, which supplies the
  four-space parent nesting.
- `src/roadmap/render_tests.rs` already contains
  `exact_nested_sub_task_round_trip`, which proves byte-identical direct
  parse-render output for one nested sub-task fixture.
- `tests/roadmap_render.rs` contains CLI-level rendering preservation tests.
  Its current sub-task test proves ordering only; this plan strengthens that
  surface.

## Plan of work

### Work item 1: Reconfirm render-fidelity mechanism and tool availability

This is an evidence and plan-update work item. It implements
`AGENTS.md` "Core tenets", `AGENTS.md` "Branches", `AGENTS.md` "Commands",
`docs/mapsplice-design.md` sections 4, 5, 6, and 8,
`docs/developers-guide.md` sections 2 and 6, and the initial-tool ExecPlan
decision to render only the supported roadmap grammar deterministically.

Read and cite these documents before editing: `AGENTS.md`, `docs/roadmap.md`,
`docs/mapsplice-design.md`, `docs/developers-guide.md`, `docs/users-guide.md`,
`docs/contributing.md`, `docs/documentation-style-guide.md`,
`docs/scripting-standards.md`, and `docs/execplans/initial-tool.md`.

Load these skills: `execplans`, `leta`, `sem`, `firecrawl-mcp`,
`rust-router`, `rust-unit-testing`, `rust-errors`, and
`rust-types-and-apis`. Use `rust-router` to confirm that
`rust-unit-testing` is the only Rust follow-on skill needed unless the focused
renderer tests fail and force production-code work.

Run the tool and baseline checks from the assigned worktree:

```sh
git branch --show-current
sem diff --from origin/main --to HEAD --format json \
  2>&1 | tee /tmp/sem-diff-mapsplice-roadmap-2-1-3.out
cargo test --workspace --all-targets --all-features \
  exact_nested_sub_task_round_trip \
  2>&1 | tee /tmp/cargo-test-exact-nested-sub-task-round-trip-mapsplice-roadmap-2-1-3.out
cargo test --workspace --all-targets --all-features --test roadmap_render \
  render_preserves_task_body_and_sub_task_order \
  2>&1 | tee /tmp/cargo-test-roadmap-render-order-mapsplice-roadmap-2-1-3.out
```

Expected results: the branch is `roadmap-2-1-3`; `sem diff` reports only the
intentional added ExecPlan before implementation; the exact unit test passes;
and the current CLI ordering test passes. Any source-code or roadmap-status
delta before work item 2 is unexpected and must be investigated before editing.
If Memtrace, Firecrawl, or Leta fail again, record the exact command and error
in this ExecPlan and continue.

External-library research for this item is already scoped. Recheck the locked
versions in `Cargo.lock`. For `markdown` 1.0.0, inspect
`~/.cargo/registry/src/index.crates.io-*/markdown-1.0.0/src/lib.rs` and
`src/mdast.rs`: rely on `to_mdast` and mdast position data only. Do not rely on
an external Markdown writer. For `rstest` 0.26.1, inspect its README enough to
confirm `#[rstest]` remains available. For `insta` 1.48.0, record that this
task deliberately avoids snapshot review because exact string equality is
load-bearing and small.

Update only `docs/execplans/roadmap-2-1-3.md` with any new evidence. Format
and gate the changed Markdown file:

```sh
mdtablefix docs/execplans/roadmap-2-1-3.md \
  2>&1 | tee /tmp/mdtablefix-execplan-mapsplice-roadmap-2-1-3.out
markdownlint-cli2 --fix docs/execplans/roadmap-2-1-3.md \
  2>&1 | tee /tmp/markdownlint-fix-execplan-mapsplice-roadmap-2-1-3.out
make all 2>&1 | tee /tmp/make-all-wi1-mapsplice-roadmap-2-1-3.out
make markdownlint 2>&1 | tee /tmp/make-markdownlint-wi1-mapsplice-roadmap-2-1-3.out
make nixie 2>&1 | tee /tmp/make-nixie-wi1-mapsplice-roadmap-2-1-3.out
```

Commit message: `Plan sub-task render fidelity`.

### Work item 2: Add CLI-level exact nested sub-task render regression

This is a test-first coverage work item. It implements
`docs/mapsplice-design.md` section 5 F1, F3, and F4, section 6 C4, and section
8's addendum render fidelity fixture requirement. It also implements
`docs/developers-guide.md` section 6 by adding `rstest` integration coverage
for the compiled workflow surface.

Read `tests/roadmap_render.rs`, `tests/support/roadmap_workspace.rs`,
`src/roadmap/render.rs`, and `src/roadmap/render_text.rs`. Use `leta show` and
`leta refs` for `render_task`, `render_sub_task`, and `indent_block` if Leta
is available; otherwise use bounded local inspection and record the fallback.

Add a new `#[rstest]` test in `tests/roadmap_render.rs` named
`render_preserves_nested_sub_task_block_exactly`. The test should write a
target roadmap containing this exact parent task block:

```markdown
- [ ] 1.1.1. Parent task.
    Body before.
    - [ ] 1.1.1.1. Nested sub-task.
    Body after.
```

Then run `mapsplice append` with `PHASE_FRAGMENT`, capture stdout, and assert
that stdout contains that full four-line block byte-for-byte. Keep the existing
ordering test unless the new exact test fully subsumes it and deleting it is
the only way to avoid duplicate coverage. Prefer keeping it to minimise churn.

Run the new test before any production-code edit:

```sh
cargo test --workspace --all-targets --all-features --test roadmap_render \
  render_preserves_nested_sub_task_block_exactly \
  2>&1 | tee /tmp/cargo-test-exact-cli-nested-sub-task-mapsplice-roadmap-2-1-3.out
```

Expected result on the planning baseline: the new test passes without
production-code changes. If it fails, keep the failure transcript as red
evidence and make the smallest renderer change in `src/roadmap/render.rs` or
`src/roadmap/render_text.rs` to preserve the exact block. Rerun the same
focused test until it passes. Do not touch parser, renumbering, CLI, or public
API code for this roadmap item.

If production code changed, run:

```sh
cargo test --workspace --all-targets --all-features \
  exact_nested_sub_task_round_trip \
  2>&1 | tee /tmp/cargo-test-exact-nested-sub-task-round-trip-wi2-mapsplice-roadmap-2-1-3.out
cargo test --workspace --all-targets --all-features --test roadmap_render \
  2>&1 | tee /tmp/cargo-test-roadmap-render-wi2-mapsplice-roadmap-2-1-3.out
```

Because this work item must update the living ExecPlan with the red or baseline
test evidence before committing, format and gate the changed Markdown path
before the work-item commit:

```sh
mdtablefix docs/execplans/roadmap-2-1-3.md \
  2>&1 | tee /tmp/mdtablefix-execplan-wi2-mapsplice-roadmap-2-1-3.out
markdownlint-cli2 --fix docs/execplans/roadmap-2-1-3.md \
  2>&1 | tee /tmp/markdownlint-fix-execplan-wi2-mapsplice-roadmap-2-1-3.out
```

Then run the required gates:

```sh
make all 2>&1 | tee /tmp/make-all-wi2-mapsplice-roadmap-2-1-3.out
make markdownlint 2>&1 | tee /tmp/make-markdownlint-wi2-mapsplice-roadmap-2-1-3.out
make nixie 2>&1 | tee /tmp/make-nixie-wi2-mapsplice-roadmap-2-1-3.out
```

Commit message if this is test-only: `Prove sub-task render indentation`.
Commit message if a renderer fix was required: `Preserve sub-task render indentation`.

### Work item 3: Update roadmap status and final validation evidence

This is the documentation status work item. It implements
`docs/roadmap.md` task 2.1.3, `docs/mapsplice-design.md` F4 gate-clean output,
`AGENTS.md` "Markdown Guidance", and
`docs/documentation-style-guide.md` spelling, headings, wrapping, and Markdown
rules.

Update `docs/roadmap.md` to mark only task 2.1.3 as complete after work item 2
passes. Do not mark 2.1.2, 3.1.1, 3.1.2, or 3.1.3 complete from this work
item. Update this ExecPlan's `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` with the final validation
evidence.

Format only the changed Markdown files. At this point both paths definitely
exist:

```sh
mdtablefix docs/execplans/roadmap-2-1-3.md docs/roadmap.md \
  2>&1 | tee /tmp/mdtablefix-docs-mapsplice-roadmap-2-1-3.out
markdownlint-cli2 --fix docs/execplans/roadmap-2-1-3.md docs/roadmap.md \
  2>&1 | tee /tmp/markdownlint-fix-docs-mapsplice-roadmap-2-1-3.out
```

Run final validation:

```sh
make all 2>&1 | tee /tmp/make-all-final-mapsplice-roadmap-2-1-3.out
make markdownlint 2>&1 | tee /tmp/make-markdownlint-final-mapsplice-roadmap-2-1-3.out
make nixie 2>&1 | tee /tmp/make-nixie-final-mapsplice-roadmap-2-1-3.out
```

Commit message: `Mark sub-task render fidelity complete`.

## Concrete steps

Start every implementation session in the assigned worktree:

```sh
cd /home/leynos/Projects/mapsplice.worktrees/roadmap-2-1-3
git branch --show-current
```

The branch command must print:

```plaintext
roadmap-2-1-3
```

Then execute the work items in order. Each work item must update this ExecPlan
before committing when new evidence, surprises, or deviations occur. Do not
start implementation until the plan is approved.

## Validation and acceptance

Acceptance for task 2.1.3 is observable when all of the following are true:

- `src/roadmap/render_tests.rs::exact_nested_sub_task_round_trip` passes and
  compares direct parse-render output byte-for-byte.
- `tests/roadmap_render.rs::render_preserves_nested_sub_task_block_exactly`
  passes and proves the CLI workflow preserves the full nested parent task
  block byte-for-byte.
- `docs/roadmap.md` marks only roadmap task 2.1.3 complete for this work.
- `make all` passes. This includes `check-fmt`, `lint`, `typecheck`, and
  `test` on current `origin/main`.
- Because Markdown files change, `make markdownlint` and `make nixie` pass.

Red-Green-Refactor evidence is conditional because planning evidence shows the
renderer behaviour already passes at the unit level. The required method is:

- Red or baseline command:
  `cargo test --workspace --all-targets --all-features --test roadmap_render
  render_preserves_nested_sub_task_block_exactly`. If the newly added test
  passes before production-code edits, record that as baseline evidence and do
  not alter production code. If it fails, record the failure as red evidence.
- Green command:
  rerun the same focused integration test after the minimal renderer fix, if a
  fix was required.
- Refactor command:
  rerun `cargo test --workspace --all-targets --all-features
  exact_nested_sub_task_round_trip`,
  `cargo test --workspace --all-targets --all-features --test roadmap_render`,
  and `make all`.

The final Markdown validation commands are:

```sh
make markdownlint 2>&1 | tee /tmp/make-markdownlint-final-mapsplice-roadmap-2-1-3.out
make nixie 2>&1 | tee /tmp/make-nixie-final-mapsplice-roadmap-2-1-3.out
```

## Idempotence and recovery

All work items are safe to rerun. The focused tests overwrite only temporary
test workspaces created by the test harness. Formatter commands name only files
that exist and are intentionally edited by this plan. If a command fails, read
the corresponding `/tmp/*mapsplice-roadmap-2-1-3.out` log before retrying.

Do not use an unnamed stash. If unrelated formatter churn appears, park it with
a named stash matching:

```sh
git stash push -m 'df12-stash v1 task=2.1.3 kind=discard reason="formatter churn"' -- <paths>
```

If a renderer fix is attempted and proves too broad, leave the failing test and
partial production changes uncommitted, update `Decision Log` with the exact
tolerance breached, and stop for review.

## Artifacts and notes

Planning pass evidence:

```plaintext
git branch --show-current
roadmap-2-1-3

git rev-parse --short HEAD
db53e09

git rev-parse --short origin/main
b502142

sem diff --from origin/main --to HEAD --format json
{"summary":{"fileCount":1,"added":20,"modified":0,"deleted":0,"moved":0,"renamed":0,"reordered":0,"orphan":0,"total":20},"changes":[{"filePath":"docs/execplans/roadmap-2-1-3.md","changeType":"added"}]}

cargo test --workspace --all-targets --all-features exact_nested_sub_task_round_trip
test roadmap::render::render_tests::exact_nested_sub_task_round_trip ... ok

cargo test --workspace --all-targets --all-features --test roadmap_render render_preserves_task_body_and_sub_task_order
test render_preserves_task_body_and_sub_task_order ... ok
```

Tool failures to preserve:

```plaintext
Memtrace list_indexed_repositories
user cancelled MCP tool call

leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-2-1-3
Error: IO error: Read-only file system (os error 30)

Firecrawl firecrawl_scrape https://docs.rs/markdown/1.0.0/markdown/fn.to_mdast.html
user cancelled MCP tool call

Firecrawl firecrawl_scrape https://docs.rs/rstest/0.26.1/rstest/attr.rstest.html
user cancelled MCP tool call

leta files src/roadmap
Error: Failed to start daemon
```

Planning round 2 validation:

```plaintext
mdtablefix docs/execplans/roadmap-2-1-3.md
completed with exit code 0

markdownlint-cli2 --fix docs/execplans/roadmap-2-1-3.md
Summary: 0 error(s)

make all
Summary: 62 tests run: 62 passed, 0 skipped
Doc-tests mapsplice: 8 passed; 0 failed; 2 ignored

make markdownlint
Summary: 0 error(s)

make nixie
All diagrams validated successfully.
```

Work item 1 implementation evidence:

```plaintext
git branch --show-current
roadmap-2-1-3

sem diff --from origin/main --to HEAD --format json
{"summary":{"fileCount":1,"added":20,"modified":0,"deleted":0,"moved":0,"renamed":0,"reordered":0,"orphan":0,"total":20},"changes":[{"filePath":"docs/execplans/roadmap-2-1-3.md","changeType":"added"}]}

cargo test --workspace --all-targets --all-features exact_nested_sub_task_round_trip
test roadmap::render::render_tests::exact_nested_sub_task_round_trip ... ok

cargo test --workspace --all-targets --all-features --test roadmap_render render_preserves_task_body_and_sub_task_order
test render_preserves_task_body_and_sub_task_order ... ok

make all
passed on scrutineer rerun

make markdownlint
passed on scrutineer rerun

make nixie
passed on local retry and scrutineer rerun

coderabbit review --agent
attempt 1: exit status 130 after connecting_to_review_service
retry 1: no review payload; still connecting_to_review_service

cargo test --workspace --all-targets --all-features --test roadmap_render render_preserves_nested_sub_task_block_exactly
test render_preserves_nested_sub_task_block_exactly ... ok

make all
passed on final scrutineer rerun

make markdownlint
passed on final scrutineer rerun

make nixie
passed on local retry and final scrutineer rerun

make all
passed on post-plan final scrutineer rerun

make markdownlint
passed on post-plan final scrutineer rerun

make nixie
passed on second local retry and post-plan final scrutineer rerun

coderabbit review --agent
attempt 1: no review payload; connecting_to_review_service
retry 1: timeout 124 after connecting_to_review_service

docs/roadmap.md
task 2.1.3 marked complete; tasks 2.1.2, 3.1.1, 3.1.2, and 3.1.3 remain unchecked

make all
passed for work item 3 final validation

make markdownlint
passed for work item 3 final validation

make nixie
passed for work item 3 final validation

coderabbit review --agent
attempt 1: interrupt 130 after connecting_to_review_service
```

## Interfaces and dependencies

No new public interfaces or external dependencies are planned.

Pinned dependency facts:

- `markdown` is locked to 1.0.0 in `Cargo.lock`. Use it only through
  `markdown::to_mdast(value: &str, options: &ParseOptions)` and mdast node
  data. Do not depend on an external Markdown writer for this task.
- `rstest` is locked to 0.26.1. Use `#[rstest]` for the new integration
  regression.
- `rstest-bdd` is locked to 0.5.0. No new BDD scenario is required because this
  task is a render-fidelity regression on an existing command path, not a new
  user command.
- `insta` resolves to 1.48.0 in `Cargo.lock`, but it is intentionally not used
  for the new proof because the expected Markdown block is small and exact
  `assert_eq!` or `contains` with a full literal gives clearer failure output.

Implementation must preserve these internal interfaces:

```rust
fn render_task(task: &TaskEntry) -> Result<String>;
fn render_sub_task(sub_task: &SubTaskEntry) -> Result<String>;
fn render_nested_body(markdown: &MarkdownNodes, indent: usize) -> Result<Vec<String>>;
pub(super) fn indent_block(block: &str, spaces: usize) -> String;
```

If any of these signatures must change, stop and update this plan before
editing.

## Revision note

Initial draft for the first planning round. It records the verified branch
state, the current passing exact unit evidence, the unavailable advisory tools,
and an implementation sequence that hardens CLI-level proof before updating the
roadmap status.

Planning round 2 revision: corrected stale baseline evidence by distinguishing
current `HEAD` `db53e09` from `origin/main` `b502142`, recorded that the
intentional branch delta is the added ExecPlan, added the repeated Firecrawl and
Leta failures, and updated work item 2 so any ExecPlan evidence update is
followed by path-specific Markdown formatting plus `make markdownlint` and
`make nixie` before commit.

Work item 1 implementation revision: changed the plan status to in progress,
recorded current Memtrace and Leta failures, ticked the first work item, and
added the focused baseline transcripts that justify keeping production code
unchanged for this evidence-only item.

Work item 1 gate revision: recorded the deterministic gate rerun success, the
transient `make nixie` timeout and successful retries, and the deferred
CodeRabbit review issue caused by repeated connection-phase stalls.

Work item 2 evidence revision: recorded the new CLI-level exact nested
sub-task block regression and its baseline pass without production-code edits.

Work item 2 gate revision: recorded the rustfmt import-order fix, the final
deterministic gate success, the transient `make nixie` timeout and retry, and
the deferred CodeRabbit review caused by another connection-phase stall.

Work item 2 post-plan gate revision: recorded the additional `make nixie`
timeouts on unchanged Mermaid diagrams and the final successful local and
scrutineer reruns before committing the work item.

Work item 3 status revision: marked the ExecPlan complete, checked only
roadmap task 2.1.3, and recorded that adjacent roadmap tasks remain
incomplete.

Work item 3 review revision: recorded the final deterministic gate success and
the third deferred CodeRabbit review caused by the same connection-phase
stall.
