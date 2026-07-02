# Assemble grammar-surface and per-contract golden fixtures

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: IN PROGRESS

## Purpose / big picture

Roadmap task 3.1.1 is complete when `mapsplice` has a committed golden-fixture
corpus that proves every supported operation, every required grammar surface,
and every fidelity or contract guarantee that can be demonstrated with a
deterministic example. A maintainer should be able to inspect each fixture,
run the focused golden suite, and see raw Markdown bytes or typed failures
compared exactly.

This plan is a round-3 revision. Branch-local verification shows that the
operation fixtures and grammar-surface fixtures requested in the previous
draft already exist in `HEAD`. The remaining work is therefore the clean plan
handoff, the missing contract/output/failure fixtures, and the final roadmap
completion update. This plan does not implement roadmap task 3.1.2's generated
no-op round-trip property or roadmap task 3.1.3's rendered-output Markdown
stability sweep. It does add one scoped F4 fixture-and-formatter stability
proof for a representative rendered output so task 3.1.1 does not claim F4
coverage without a deterministic example.

## Constraints

- Work only in
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1`.
- Do not edit the root/control worktree at `/home/leynos/Projects/mapsplice`.
- Treat `origin/main` as canonical and the integration branch as `main`.
- Do not begin implementation of roadmap task 3.1.1 until this DRAFT plan is
  approved.
- Before any fixture implementation begins, this ExecPlan revision must be
  formatted, gated, committed, and the tree must be clean. This prevents the
  plan edit from being swept into an unrelated fixture commit.
- Use Memtrace as the primary canonical-main search and graph tool when it is
  available. If Memtrace is cancelled or unavailable, record the exact failure
  and continue with bounded branch-local evidence; do not mark the plan blocked
  for advisory-tool failure alone.
- Use `leta` for branch-local symbol navigation and verification when it is
  available. Fall back to precise file inspection only after recording the
  exact Leta failure.
- Use `sem` for entity-level history and diff inspection instead of raw
  `git log` or `git blame`.
- Follow `AGENTS.md`, `docs/mapsplice-design.md`,
  `docs/developers-guide.md`, `docs/users-guide.md`, `docs/roadmap.md`,
  `docs/documentation-style-guide.md`, `docs/scripting-standards.md`, and
  `docs/execplans/initial-tool.md`.
- Keep prose, comments, fixtures, and commit messages in en-GB Oxford spelling.
- Do not add an external dependency. If implementation appears to require one,
  stop and revise this plan with locked-source and official-documentation
  evidence for the dependency.
- Do not redesign the grammar, operation semantics, dependency-reference
  model, command-line interface, or public library API unless a red fixture
  exposes a real defect. Keep any defect fix in the same atomic work item as
  the fixture that proves it.
- Fixture files are committed test inputs and expected outputs, not generated
  artefacts.
- Format only Markdown files changed by the current work item. Do not run
  `make fmt`, `mdformat-all`, or any other repo-global formatter for this
  task.
- Every documentation-changing work item must run `make nixie` before commit.
  Plain `make nixie` currently has a known intermittent renderer timeout on
  unchanged documentation diagrams in this worktree. If the only failure is
  `diagram 1 timed out` in unchanged `docs/ortho-config-users-guide.md` or
  unchanged `docs/rstest-bdd-users-guide.md`, rerun the repository gate
  serially with the Makefile-supported renderer command override:

  ```bash
  NIXIE='nixie --no-sandbox --max-concurrency 1' make nixie
  ```

  Commit only if the plain gate or this serial repository-gate recovery passes.
  Any other `make nixie` failure is a real gate failure for the current work
  item and must be fixed before committing.
- Run tests, lints, and gates sequentially. Commands that may produce long
  output must use `set -o pipefail` and `tee` to a branch-specific file under
  `/tmp`.
- Commit after each work item, and gate each commit before moving on.

## Tolerances

- If `git branch --show-current` is not `roadmap-3-1-1`, stop before editing.
- If `git status --short` is not clean after work item 1, stop before fixture
  implementation.
- If a work item needs a public API signature change, stop and revise this
  plan.
- If a work item needs a new crate, stop and revise this plan with locked
  source and official-documentation evidence for that crate.
- If a focused test or repository gate still fails after two focused fix
  attempts, record the command, log path, and error in `Decision Log`, then
  stop for review.
- If one work item would touch more than six non-fixture Rust source files,
  split that work item before committing.
- If formatter churn touches files outside the current work item, park or
  discard it with a named stash:

  ```bash
  git stash push -m 'df12-stash v1 task=3.1.1 kind=discard reason="formatter churn"'
  ```

- If Memtrace, Firecrawl, `leta`, or another advisory tool is unavailable, do
  not mark this plan blocked. Record the exact failed command or tool result in
  `Surprises & Discoveries` and continue with bounded local evidence.
- If plain `make nixie` fails only with the known unchanged-document timeout
  described in `Constraints`, use the serial `NIXIE=... make nixie` recovery
  in the same work item. If the serial recovery also fails, stop before commit,
  record both log paths in `Decision Log`, and fix the actual failing diagram
  only if the log points to a document changed by the current work item.

## Risks

- Risk: The worktree already contains committed operation and grammar fixtures,
  so replaying the old work items would create duplicates or no-op commits.
  Severity: high. Likelihood: confirmed. Mitigation: mark those surfaces as
  verified existing coverage and keep the remaining work items scoped only to
  gaps.

- Risk: A table-driven harness can hide which fixture failed. Severity:
  medium. Likelihood: medium. Mitigation: every case must have a stable Rust
  test name, and assertion failures must include the case name plus the
  expected and actual output or typed error.

- Risk: Some guarantees are failure contracts rather than successful output
  contracts. Severity: medium. Likelihood: high. Mitigation: model those cases
  as typed-error expectations with unchanged target assertions, not as
  successful expected-output files.

- Risk: Future roadmap task 3.1.2 needs to enumerate conformant fixtures.
  Severity: medium. Likelihood: medium. Mitigation: store successful examples
  under `tests/fixtures/golden/<case-name>/` with `target.md`, optional
  `fragment.md`, and `expected.md`; keep failure fixtures explicitly marked in
  Rust metadata.

- Risk: `make nixie` can time out on unchanged repository documentation before
  a documentation-only commit can be gated. Severity: high. Likelihood:
  confirmed. Mitigation: first run plain `make nixie`; when it fails only with
  the known unchanged `diagram 1 timed out` case, rerun the repository gate
  through `NIXIE='nixie --no-sandbox --max-concurrency 1' make nixie`, a
  serial command verified in this worktree against the two known timeout
  documents.

## Progress

- [x] (2026-07-02T00:00:00Z) Confirmed the assigned worktree and branch:
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1` on
  `roadmap-3-1-1`.
- [x] (2026-07-02T03:05:04Z) Loaded the required skills used for this planning
  round: `execplans`, `leta`, `sem`, `firecrawl-mcp`, `rust-router`,
  `rust-unit-testing`, `rust-verification`, and `proptest`.
- [x] (2026-07-02T00:00:00Z) Read the source-of-truth documents listed in
  `Context and orientation`.
- [x] (2026-07-02T00:00:00Z) Confirmed the current dirty tree contains only
  `M docs/execplans/roadmap-3-1-1.md`.
- [x] (2026-07-02T00:00:00Z) Verified branch-local `tests/roadmap_golden.rs`
  already registers operation tests for `append_phase`, `insert_phase_before`,
  `insert_step_after`, `insert_task_before`, `insert_sub_task_after`,
  `delete_task`, `replace_step`, and `replace_sub_task`.
- [x] (2026-07-02T00:00:00Z) Verified branch-local
  `tests/fixtures/golden/{append_phase,insert_phase_before,insert_step_after,
  insert_task_before,insert_sub_task_after,delete_task,replace_step,
  replace_sub_task}` already contain fixture files.
- [x] (2026-07-02T00:00:00Z) Verified branch-local `tests/roadmap_golden.rs`
  already registers grammar-surface tests for `preamble_preserved`,
  `phase_step_task_surface`, `multi_line_task_body`, `nested_bullets`,
  `tables_preserved`, `code_blocks_preserved`, and `addendum_body_surface`.
- [x] (2026-07-02T00:00:00Z) Verified branch-local
  `tests/fixtures/golden/{preamble_preserved,phase_step_task_surface,
  multi_line_task_body,nested_bullets,tables_preserved,
  code_blocks_preserved,addendum_body_surface}` already contain fixture files.
- [x] (2026-07-02T00:00:00Z) Verified branch-local
  `tests/fixtures/reference_rewrite/` already contains
  `multi_id_requires.*.md` and `substring_non_match.*.md`, and
  `tests/roadmap_golden.rs` already registers both cases.
- [x] (2026-07-02T00:00:00Z) Verified locked local source for
  `markdown 1.0.0`, `rstest 0.26.1`, `rstest-bdd 0.5.0`,
  `proptest 1.11.0`, and `insta 1.48.0`.
- [x] (2026-07-02T03:05:04Z) Verified the previous review's current `make
  nixie` blocker: `/tmp/nixie-mapsplice-roadmap-3-1-1-review-round2.out`
  fails on unchanged `docs/ortho-config-users-guide.md` with
  `diagram 1 timed out`, while
  `/tmp/nixie-mapsplice-roadmap-3-1-1-plan.out` fails on unchanged
  `docs/rstest-bdd-users-guide.md` with the same timeout.
- [x] (2026-07-02T03:05:04Z) Verified the serial recovery command
  `nixie --no-sandbox --max-concurrency 1 docs/ortho-config-users-guide.md
  docs/rstest-bdd-users-guide.md` passes in
  `/tmp/nixie-mapsplice-roadmap-3-1-1-known-timeouts-serial.out`.
- [x] (2026-07-02T03:05:04Z) Revised the task 3.1.1 acceptance language and
  work item 2 to add a scoped F4 formatter-stability proof instead of
  deferring every F4 check to 3.1.3 or overclaiming full F4 completion.
- [x] (2026-07-02T03:05:04Z) Formatted this ExecPlan with `mdtablefix`, ran
  `markdownlint-cli2 --fix docs/execplans/roadmap-3-1-1.md`, passed repository
  `make markdownlint`, and passed the serial `NIXIE='nixie --no-sandbox
  --max-concurrency 1' make nixie` recovery gate.
- [x] (2026-07-02T11:43:00Z) Work item 1 approved by the automated workflow
  instruction and prepared for its gated plan-handoff commit.
- [x] (2026-07-02T11:55:00Z) Work item 1 deterministic gates passed for the
  final plan handoff state: `make all`, `make markdownlint`, and serial
  `NIXIE='nixie --no-sandbox --max-concurrency 1' make nixie` after plain
  `make nixie` hit the known unchanged
  `docs/rstest-bdd-users-guide.md` timeout.
- [ ] Work item 2: Add remaining per-contract fidelity and reference fixtures.
- [ ] Work item 3: Add output-mode and fail-closed fixtures.
- [ ] Work item 4: Mark roadmap task 3.1.1 complete.

## Surprises & Discoveries

- Memtrace `list_indexed_repositories` returned
  `user cancelled MCP tool call`. Canonical-main graph context was unavailable,
  so this planning round uses bounded branch-local evidence from documentation,
  local source inspection, `cargo tree`, and `sem`. This is not a blocker.
- Memtrace `list_indexed_repositories` returned
  `user cancelled MCP tool call` again during the implementation session, before
  work item 1. The exact retry failure matches the existing advisory-tool
  unavailability path and is not a blocker.
- `scrutineer` delegation for work item 1 deterministic gates failed before
  any gate ran: `You've hit your usage limit for GPT-5.3-Codex-Spark. Switch
  to another model now, or try again at Jul 7th, 2026 12:20 PM.` The same gate
  commands were run locally and passed.
- CodeRabbit review for work item 1 was deferred by the sandbox environment:
  `{"type":"status","phase":"deferred","status":"deferred coderabbit review: no
  default network route visible in this sandbox"}`. No CodeRabbit findings were
  available to action.
- `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1`
  returned `Error: IO error: Read-only file system (os error 30)` and
  `Error: Failed to start daemon`. Branch-local code verification therefore
  used precise inspection of known files. This is not a blocker.
- Firecrawl `firecrawl_scrape` for
  `https://docs.rs/markdown/1.0.0/markdown/fn.to_mdast.html` returned
  `user cancelled MCP tool call`. Official web documentation was unavailable,
  so this plan avoids web-only claims and pins load-bearing library behaviour
  to locked local source.
- The current branch-local harness already has a split golden-test surface in
  `tests/golden/{mod.rs,case.rs,workspace.rs,runner.rs,assertions.rs}`. It
  drives `run_from_args`, compares expected output as raw fixture bytes, and
  supports typed failure expectations for dangling dependencies, level
  mismatches, and missing anchors.
- The previous round-1 plan was stale against this worktree: operation and
  grammar-surface fixtures are already present in `HEAD`.
- Validation during this planning round ran `make all` and `make markdownlint`
  successfully, then `make nixie` twice. Both `make nixie` attempts processed
  `docs/execplans/roadmap-3-1-1.md` successfully but failed on the unchanged
  documentation diagrams described below.
- `/tmp/nixie-mapsplice-roadmap-3-1-1-review-round2.out` shows the current
  round-2 review blocker: unchanged `docs/ortho-config-users-guide.md` diagram
  1 timed out. `/tmp/nixie-mapsplice-roadmap-3-1-1-plan.out` shows a later
  unchanged `docs/rstest-bdd-users-guide.md` diagram 1 timeout. The serial
  command `nixie --no-sandbox --max-concurrency 1
  docs/ortho-config-users-guide.md docs/rstest-bdd-users-guide.md` passed, so
  every documentation-changing work item now has a concrete recovery path for
  this known timeout.

## Decision Log

- Decision: Treat operation and grammar-surface coverage as verified existing
  coverage, not remaining implementation work. Rationale: branch-local
  `tests/roadmap_golden.rs` and `tests/fixtures/golden/` already contain those
  named tests and fixtures; recreating them would be duplicate work or a no-op
  commit. Date/Author: 2026-07-02 / Codex.

- Decision: Add a dedicated plan-handoff work item before any fixture work.
  Rationale: the worktree starts with a dirty ExecPlan edit, and AGENTS.md
  requires committing each change after gates. The plan must not be mixed into
  a fixture commit. Date/Author: 2026-07-02 / Codex.

- Decision: Extend the existing split golden harness rather than replacing it.
  Rationale: `tests/golden/runner.rs::assert_golden_case` already drives
  `run_from_args`, dispatches success and failure expectations, and keeps case
  metadata explicit. Date/Author: 2026-07-02 / Codex.

- Decision: Store successful examples as Markdown files under
  `tests/fixtures/golden/<case-name>/`. Rationale:
  `docs/mapsplice-design.md` requires committed input-and-expected Markdown
  pairs compared exactly, and this layout is easy for later property work to
  enumerate. Date/Author: 2026-07-02 / Codex.

- Decision: Keep the existing reference-rewrite fixtures in
  `tests/fixtures/reference_rewrite/` and register them through the golden
  harness where they already prove a required C3 case. Rationale: moving
  existing fixtures would add churn without improving coverage. Date/Author:
  2026-07-02 / Codex.

- Decision: Use exact Markdown files, not new `insta` snapshots, for task
  3.1.1. Rationale: the design calls for exact fixture pairs; `insta` remains
  suitable for existing CLI help snapshots but is not the corpus mechanism for
  this task. Date/Author: 2026-07-02 / Codex.

- Decision: Prove process-level stdout semantics for fail-closed `--in-place`
  errors through a compiled-binary BDD scenario. Rationale: the golden harness
  observes `run_from_args` results and target bytes, while the BDD harness
  observes process stdout and stderr. Date/Author: 2026-07-02 / Codex.

- Decision: Add a scoped F4 fixture in task 3.1.1 and leave the exhaustive
  rendered-output sweep to task 3.1.3. Rationale: section 5 requires F4
  gate-clean output, and section 8 treats missing fixtures as unproven. A
  representative `f4_formatter_stability_smoke` case plus formatter no-diff
  validation proves task 3.1.1 has deterministic F4 evidence without claiming
  the full corpus sweep assigned to 3.1.3. Date/Author: 2026-07-02 / Codex.

- Decision: Treat unchanged-document `make nixie` timeouts as a recoverable
  renderer scheduling issue only when the serial repository gate passes.
  Rationale: the Makefile exposes `NIXIE`, `nixie --help` documents
  `--max-concurrency`, and the serial command passed on both known timeout
  documents. This keeps documentation commits gateable while still requiring a
  Mermaid gate before every commit. Date/Author: 2026-07-02 / Codex.

- Decision: Run work item 1 gates locally after `scrutineer` quota exhaustion.
  Rationale: the task requires the deterministic gates before commit, and the
  fixed `scrutineer` role could not start because its model quota was
  exhausted. Local execution used the same commands and log paths requested of
  `scrutineer`. Date/Author: 2026-07-02 / Codex.

## Outcomes & Retrospective

Work item 1 is ready to commit as the gated plan-handoff revision. The
remaining outcome is three implementation commits: the missing contract
fixtures, the output-mode and fail-closed fixtures, and a documentation-only
completion commit that marks only roadmap task 3.1.1 complete. CodeRabbit is
recorded as deferred for this work item because the sandbox has no default
network route.

## Context and orientation

`mapsplice` edits constrained roadmap-shaped Markdown by parsing Markdown into
a roadmap model, applying one structural operation, renumbering affected items,
rewriting dependency references, and rendering Markdown. The source of truth
for command semantics and accepted grammar is `docs/users-guide.md`; the source
of truth for fidelity guarantees, contract guarantees, and fixture
requirements is `docs/mapsplice-design.md`.

Read these documents before implementation:

- `AGENTS.md`, especially lines 51-79 for quality gates and commits, lines
  112-249 for Rust and testing rules, and lines 251-260 for Markdown gates.
- `docs/roadmap.md` lines 82-112 for roadmap task 3.1.1 and the neighbouring
  3.1.2 and 3.1.3 scope boundaries.
- `docs/mapsplice-design.md` lines 86-101 for grammar, lines 103-149 for
  F1-F5 and C1-C6, lines 151-175 for the dependency-reference model, and lines
  177-210 for fixture requirements.
- `docs/users-guide.md` lines 24-64 for grammar and command overview, lines
  66-110 for the dependency-rewrite example, lines 112-181 for operation and
  output-mode semantics, and lines 205-218 for failure cases.
- `docs/developers-guide.md` lines 20-57 for architecture and public APIs,
  lines 84-105 for verification layers, and lines 107-131 for local gates.
- `docs/documentation-style-guide.md` lines 7-24 for spelling, lines 41-52 for
  Markdown rules, and lines 58-65 for formatting.
- `docs/scripting-standards.md` lines 1-6 for reproducible command practice.
- `docs/execplans/initial-tool.md` lines 80-100 for original constraints,
  lines 210-224 for deterministic rendering, and lines 423-490 for accepted
  decisions around constrained grammar, mdast parsing, rendering, and tests.

The implementation surfaces relevant to this task are:

- `tests/roadmap_golden.rs`, where named golden cases are registered.
- `tests/golden/case.rs`, which defines `GoldenCase`, `GoldenCommand`,
  `GoldenExpectation`, `SuccessOutput`, `FailureOutput`, and `ExpectedError`.
- `tests/golden/runner.rs`, which prepares a temporary workspace and dispatches
  success or failure assertions through `run_from_args`.
- `tests/golden/assertions.rs`, which compares stdout, target bytes, in-place
  writes, and typed `MapspliceError` variants.
- `tests/golden/workspace.rs`, which resolves fixture paths and reads expected
  output as raw fixture bytes.
- `tests/fixtures/reference_rewrite/`, the existing adversarial C3 reference
  fixtures from roadmap task 1.1.3.
- `tests/features/mapsplice.feature`, `tests/behaviour_cli.rs`, and
  `tests/steps/cli_steps.rs`, the compiled-binary BDD surface for process
  stdout, stderr, and target preservation.
- `src/lib.rs::run_from_args` and `src/lib.rs::run_request`, the library
  workflow entry points used by integration tests.
- `src/error.rs::MapspliceError`, the typed diagnostic surface for failure
  fixtures.
- `src/roadmap/ops/mod.rs::RoadmapOperation`, the domain operation surface.
- `src/roadmap/parse/mod.rs::parse_root`, which uses
  `markdown::to_mdast(markdown, &ParseOptions::gfm())`.
- `src/roadmap/render.rs::render_roadmap`, the deterministic renderer entry
  point that appends exactly one final newline to non-empty output.

## Research evidence

Memtrace and Firecrawl were attempted first, but both were cancelled by the
host session as recorded above. The following locked-source and branch-local
evidence is sufficient for an implementer to avoid unverified mechanisms:

- `cargo tree -i markdown`, `cargo tree -i rstest`,
  `cargo tree -i rstest-bdd`, `cargo tree -i proptest`, and
  `cargo tree -i insta` show `markdown v1.0.0`, `rstest v0.26.1`,
  `rstest-bdd v0.5.0`, `proptest v1.11.0`, and `insta v1.48.0`.
- `markdown-1.0.0/src/lib.rs` line 160 defines
  `pub fn to_mdast(value: &str, options: &ParseOptions)`.
- `markdown-1.0.0/src/configuration.rs` lines 1269-1279 state that
  `ParseOptions::gfm()` enables GitHub Flavoured Markdown constructs including
  tables and task lists. `src/roadmap/parse/mod.rs` lines 373-381 already use
  `to_mdast(markdown, &ParseOptions::gfm())`. Fixtures may rely on GFM task
  lists and tables being parsed by the existing code path; they must not rely
  on `markdown` for exact Markdown rendering.
- `rstest-0.26.1/src/lib.rs` line 571 re-exports
  `rstest_macros::fixture`. New tests should follow the existing `#[fixture]`
  and `#[rstest]` style.
- `rstest-bdd-macros-0.5.0/src/lib.rs` lines 75-101 define the `given` and
  `when` macro entry points; lines 123-125 define `then`; lines 205-210
  define the `scenarios` macro used for feature-file discovery. Use the existing
  compiled-binary BDD harness style.
- `proptest-1.11.0/src/sugar.rs` line 624 defines `prop_compose!`. Task 3.1.1
  must not add task 3.1.2's property, but fixture layout should remain easy for
  that later property to enumerate.
- `insta-1.48.0/src/macros.rs` line 463 defines `assert_snapshot!`. This task
  deliberately avoids new snapshots because exact Markdown fixture files are
  the design requirement.
- `tests/golden/workspace.rs::expected_output` returns raw fixture text, so
  golden expected output is compared as raw bytes after UTF-8 loading.
- `Makefile` line 26 defines `all: check-fmt lint typecheck test`, so
  `make all` includes the `typecheck` target on current `origin/main`.
- `Makefile` lines 21 and 63 route `make nixie` through the `NIXIE` variable
  and pass `--no-sandbox`. `nixie --help` documents `--max-concurrency`.
  `nixie --no-sandbox --max-concurrency 1 docs/ortho-config-users-guide.md
  docs/rstest-bdd-users-guide.md` passed in this worktree, so the serial
  `NIXIE=... make nixie` recovery below is an observed repository-gate path for
  the known unchanged-document timeout.
- `src/roadmap/render.rs` lines 49-54 join rendered blocks and append exactly
  one final newline to non-empty output.

## Verified existing coverage

The following surfaces are already present in this worktree and are not future
work items:

- Operation fixtures and tests:
  `append_phase`, `insert_phase_before`, `insert_step_after`,
  `insert_task_before`, `insert_sub_task_after`, `delete_task`, `replace_step`,
  and `replace_sub_task`.
- Grammar-surface fixtures and tests:
  `preamble_preserved`, `phase_step_task_surface`, `multi_line_task_body`,
  `nested_bullets`, `tables_preserved`, `code_blocks_preserved`, and
  `addendum_body_surface`.
- Existing C3 reference-rewrite adversarial fixtures and tests:
  `section_reference`, `version_quantity`,
  `section_reference_outside_requires`, `substring_non_match`, and
  `multi_id_requires`.

An implementer must not recreate these cases. If a later gate exposes a defect
in one of them, fix that defect in the smallest related work item and record
the reason in `Decision Log`.

## Plan of work

### Work item 1: Approve, format, gate, and commit this ExecPlan revision

This item implements `AGENTS.md` lines 51-79 for atomic commits and quality
gates, `AGENTS.md` lines 251-260 for Markdown validation, and the plan
transition constraint above. It exists solely to keep the approved plan edit
out of later fixture commits.

Skills to load: `execplans`, `sem`, and `en-gb-oxendict-style`.

After this DRAFT is approved, format only this ExecPlan file, run the
documentation gates and repository gate, inspect the semantic diff, and commit
the plan revision. Do not add fixtures, edit tests, or mark roadmap items
complete in this work item.

Tests to add or update: none. This is a documentation-only plan commit.

Validation for this work item:

```bash
set -o pipefail
mdtablefix docs/execplans/roadmap-3-1-1.md \
  2>&1 | tee /tmp/mdtablefix-mapsplice-roadmap-3-1-1-plan.out
markdownlint-cli2 --fix docs/execplans/roadmap-3-1-1.md \
  2>&1 | tee /tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-plan.out
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-3-1-1-plan.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-plan.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-plan.out
NIXIE='nixie --no-sandbox --max-concurrency 1' make nixie \
  2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-plan-serial.out
sem diff --format json 2>&1 | tee /tmp/sem-mapsplice-roadmap-3-1-1-plan.out
git status --short
```

Run the serial `NIXIE=... make nixie` command only if plain `make nixie` fails
solely with the known unchanged-document timeout. Commit only after plain
`make nixie` or the serial recovery passes and `git status --short` shows only
the intended ExecPlan change before staging.

### Work item 2: Add remaining per-contract fidelity and reference fixtures

This item implements `docs/mapsplice-design.md` lines 103-175 and 190-210,
`docs/roadmap.md` lines 92-100, and `docs/developers-guide.md` lines 99-105.
It also preserves the reference-context limit from `docs/mapsplice-design.md`
lines 224-226.

Skills to load: `leta`, `rust-router`, `rust-unit-testing`,
`rust-verification`, `proptest`, `rust-errors`, `sem`, and
`en-gb-oxendict-style`.

Start from a clean tree after work item 1. Keep the existing
`tests/fixtures/reference_rewrite/` cases where they already prove required C3
adversarial classes. Add or register only the missing cases below:

- `f1_minimal_untouched_content/` proves unrelated text, formatting, tables,
  and code blocks remain unchanged while an operation changes a separate item.
- `f2_minimal_renumber_diff/` proves the only successful-output changes are
  the addressed edit, deterministic renumbering, and dependency-reference
  rewrites.
- `f3_c5_identity_replace/` replaces a task with byte-identical task Markdown
  and proves byte-identical stdout through `SuccessOutput::OriginalTargetStdout`.
- `f4_formatter_stability_smoke/` uses a rendered output that includes a table,
  nested list, code block, and final newline. Its golden test proves rendered
  stdout equals `expected.md`; the validation command below copies that
  `expected.md` to `/tmp`, runs `mdtablefix` and `markdownlint-cli2 --fix` on
  the copy, and `diff -u` proves the formatter made no changes. This is the
  scoped deterministic F4 proof for 3.1.1; the exhaustive rendered-output
  stability sweep remains 3.1.3.
- `c2_contiguous_renumber/` proves phase, step, task, and addendum numbers are
  contiguous after an insertion.
- `c4_addendum_renumber/` proves `8.2.3.1` tracks its parent task when the
  parent renumbers.
- `c4_addendum_render_fidelity/` proves addendum nesting and indentation are
  preserved on render.
- `c3_dangling_requires_failure/` proves an unresolved valid anchor in a
  `Requires` clause fails closed with `ExpectedError::DanglingDependency` and
  leaves the target unchanged. This failure case has no `expected.md`.

Do not recreate `multi_id_requires` or `substring_non_match`; they are already
registered from `tests/fixtures/reference_rewrite/`.

Tests to add or update: named golden tests in `tests/roadmap_golden.rs`, plus
metadata tests only if a new metadata helper is needed. Do not add task 3.1.2's
generated no-op property, snapshots, or compiled-binary BDD tests in this work
item.

Validation for this work item:

```bash
set -o pipefail
cargo test --workspace --all-targets --all-features --test roadmap_golden \
  2>&1 | tee /tmp/test-mapsplice-roadmap-3-1-1-contracts.out
cp tests/fixtures/golden/f4_formatter_stability_smoke/expected.md \
  /tmp/f4-formatter-stability-smoke-mapsplice-roadmap-3-1-1.md
mdtablefix /tmp/f4-formatter-stability-smoke-mapsplice-roadmap-3-1-1.md \
  2>&1 | tee /tmp/mdtablefix-f4-mapsplice-roadmap-3-1-1-contracts.out
markdownlint-cli2 --fix /tmp/f4-formatter-stability-smoke-mapsplice-roadmap-3-1-1.md \
  2>&1 | tee /tmp/markdownlint-fix-f4-mapsplice-roadmap-3-1-1-contracts.out
diff -u tests/fixtures/golden/f4_formatter_stability_smoke/expected.md \
  /tmp/f4-formatter-stability-smoke-mapsplice-roadmap-3-1-1.md \
  2>&1 | tee /tmp/diff-f4-mapsplice-roadmap-3-1-1-contracts.out
git diff --name-only -z --diff-filter=ACMRT HEAD -- '*.md' \
  | xargs -0 -r mdtablefix \
  2>&1 | tee /tmp/mdtablefix-mapsplice-roadmap-3-1-1-contracts.out
git diff --name-only -z --diff-filter=ACMRT HEAD -- '*.md' \
  | xargs -0 -r markdownlint-cli2 --fix \
  2>&1 | tee /tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-contracts.out
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-3-1-1-contracts.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-contracts.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-contracts.out
NIXIE='nixie --no-sandbox --max-concurrency 1' make nixie \
  2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-contracts-serial.out
sem diff --format json 2>&1 | tee /tmp/sem-mapsplice-roadmap-3-1-1-contracts.out
```

Run the serial `NIXIE=... make nixie` command only if plain `make nixie` fails
solely with the known unchanged-document timeout. Commit only after all focused
tests, formatters, `make all`, `make markdownlint`, and either plain `make
nixie` or the serial recovery pass.

### Work item 3: Add output-mode and fail-closed fixtures

This item implements `docs/mapsplice-design.md` lines 122-149 and 170-175,
`docs/users-guide.md` lines 160-181 and 205-218, and
`docs/developers-guide.md` lines 20-57 and 84-105.

Skills to load: `leta`, `rust-router`, `rust-unit-testing`, `rust-errors`,
`domain-cli-and-daemons`, `sem`, and `en-gb-oxendict-style`.

Add these golden cases:

- `c6_stdout_target_unchanged/` uses a successful stdout operation and proves
  stdout contains raw expected bytes while the temporary target remains equal
  to original target bytes.
- `c6_in_place_success/` uses `--in-place`, proves stdout is absent,
  `RunOutcome::written_path` is the target, and target bytes equal
  `expected.md`.
- `f5_malformed_grammar_failure/` proves malformed input returns a typed
  `MapspliceError` and leaves the target unchanged.
- `f5_level_mismatch_failure/` proves a fragment at the wrong structural level
  fails before output and leaves the target unchanged.
- `f5_missing_anchor_in_place_failure/` uses `--in-place`, proves a missing
  anchor returns `MapspliceError::AnchorNotFound` through the golden harness,
  and leaves the target unchanged.

Add one compiled-binary behaviour-driven development scenario for the C6/F5
process contract that the golden harness cannot observe:

```gherkin
Scenario: Missing anchor fails in place without rewriting target
  Given the target roadmap with two phases
  When I try to delete missing phase 99 in place
  Then the command fails
  And stdout is empty
  And stderr mentions that anchor 99 was not found
  And the target file remains unchanged
```

In `tests/behaviour_cli.rs`, add the matching scenario function. In
`tests/steps/cli_steps.rs`, add only the missing `when` step that runs
`["--in-place", "delete", target.as_str(), "99"]`; reuse existing assertion
steps where they already exist.

Tests to add or update: named golden tests, metadata tests if a new typed
failure is required, the compiled-binary BDD scenario named
`missing_anchor_in_place`, and focused unit tests for any new harness failure
assertion. Do not add property or snapshot tests in this work item.

Validation for this work item:

```bash
set -o pipefail
cargo test --workspace --all-targets --all-features --test roadmap_golden \
  2>&1 | tee /tmp/test-mapsplice-roadmap-3-1-1-output-failure.out
cargo test --workspace --all-targets --all-features --test behaviour_cli \
  missing_anchor_in_place \
  2>&1 | tee /tmp/test-mapsplice-roadmap-3-1-1-missing-anchor-cli.out
git diff --name-only -z --diff-filter=ACMRT HEAD -- '*.md' \
  | xargs -0 -r mdtablefix \
  2>&1 | tee /tmp/mdtablefix-mapsplice-roadmap-3-1-1-output-failure.out
git diff --name-only -z --diff-filter=ACMRT HEAD -- '*.md' \
  | xargs -0 -r markdownlint-cli2 --fix \
  2>&1 | tee /tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-output-failure.out
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-3-1-1-output-failure.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-output-failure.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-output-failure.out
NIXIE='nixie --no-sandbox --max-concurrency 1' make nixie \
  2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-output-failure-serial.out
sem diff --format json 2>&1 | tee /tmp/sem-mapsplice-roadmap-3-1-1-output-failure.out
```

Run the serial `NIXIE=... make nixie` command only if plain `make nixie` fails
solely with the known unchanged-document timeout. Commit only after all focused
tests, formatters, `make all`, `make markdownlint`, and either plain `make
nixie` or the serial recovery pass.

### Work item 4: Mark roadmap task 3.1.1 complete

This item implements the documentation completion for `docs/roadmap.md` lines
92-100 after work items 1-3 have passed. It follows `AGENTS.md` lines 36-49
for documentation maintenance and `docs/developers-guide.md` lines 107-131 for
local gates.

Skills to load: `execplans`, `sem`, and `en-gb-oxendict-style`.

After the contract and output/failure fixtures are committed and gated, update
`docs/roadmap.md` to mark only task 3.1.1 complete. Update this ExecPlan's
`Progress`, `Decision Log`, and `Outcomes & Retrospective` with fixture counts
and final gate evidence. Do not mark 3.1.2 or 3.1.3 complete.

Tests to add or update: no new Rust tests, property tests, snapshots, or BDD
tests. This is a documentation-only completion commit.

Validation for this work item:

```bash
set -o pipefail
git diff --name-only -z --diff-filter=ACMRT HEAD -- '*.md' \
  | xargs -0 -r mdtablefix \
  2>&1 | tee /tmp/mdtablefix-mapsplice-roadmap-3-1-1-complete.out
git diff --name-only -z --diff-filter=ACMRT HEAD -- '*.md' \
  | xargs -0 -r markdownlint-cli2 --fix \
  2>&1 | tee /tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-complete.out
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-3-1-1-complete.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-complete.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-complete.out
NIXIE='nixie --no-sandbox --max-concurrency 1' make nixie \
  2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-complete-serial.out
sem diff --format json 2>&1 | tee /tmp/sem-mapsplice-roadmap-3-1-1-complete.out
```

Run the serial `NIXIE=... make nixie` command only if plain `make nixie` fails
solely with the known unchanged-document timeout. Commit only after `make all`,
`make markdownlint`, and either plain `make nixie` or the serial recovery pass.

## Concrete steps

1. Start each implementation session from the assigned worktree:

   ```bash
   cd /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1
   git branch --show-current
   git status --short
   sem diff --from origin/main --to HEAD --format json
   ```

   Expected branch output is `roadmap-3-1-1`. Stop if the branch differs.

2. Complete work item 1 before any fixture implementation. Its acceptance
   condition is a committed plan revision and a clean tree.

3. For each later work item, load the skills listed in that item and re-read
   the named documentation sections.

4. Add the red test or fixture first, run the focused test command, and record
   the expected failure in this ExecPlan's `Progress`.

5. Make the smallest implementation or fixture change needed for the focused
   test to pass.

6. Run the work item's changed-Markdown formatter commands and full gates
   exactly as listed.

7. Review entity-level changes with `sem diff --format json`, update this
   ExecPlan's living sections, and commit the atomic work item.

8. Repeat for the next work item. Do not skip gates between commits.

## Validation and acceptance

The final acceptance criteria for roadmap task 3.1.1 are:

- The plan revision is committed before fixture implementation begins, and the
  implementation starts from a clean tree.
- `tests/roadmap_golden.rs` contains named golden tests for every operation,
  every required grammar surface, the example-expressible fidelity guarantees
  F1, F2, F3, and F5, the scoped F4 formatter-stability smoke proof described
  in work item 2, every example-expressible contract guarantee C1-C6, and every
  required adversarial fixture class in `docs/mapsplice-design.md` section 8.
- Successful cases are committed as explicit input-and-expected Markdown files
  under `tests/fixtures/golden/<case-name>/`, or remain covered under
  `tests/fixtures/reference_rewrite/` where those fixtures already exist.
- The `f4_formatter_stability_smoke` successful fixture renders exactly to
  `expected.md`, and a copied `expected.md` is unchanged after `mdtablefix` and
  `markdownlint-cli2 --fix`. This is task 3.1.1's deterministic F4 example;
  task 3.1.3 remains responsible for sweeping all rendered fixtures through the
  house formatter and Markdown gates.
- Failure cases assert typed `MapspliceError` variants and unchanged target
  bytes.
- The missing-anchor `--in-place` failure has compiled-binary BDD coverage
  proving the command fails, stdout is empty, stderr reports the missing
  anchor, and the target file remains unchanged.
- Expected output is compared as raw fixture bytes.
- Rendered non-empty Markdown ends in exactly one final newline.
- `docs/roadmap.md` marks only 3.1.1 complete after the corpus and gates are
  complete.
- The final committed state passes:

  ```bash
  set -o pipefail
  make all 2>&1 | tee /tmp/all-mapsplice-roadmap-3-1-1-final.out
  make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-final.out
  make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-final.out
  NIXIE='nixie --no-sandbox --max-concurrency 1' make nixie \
    2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-final-serial.out
  ```

  The serial `NIXIE=... make nixie` final command is conditional: run it only
  when plain `make nixie` fails solely with the known unchanged-document
  timeout. The final state is acceptable only when plain `make nixie` or the
  serial repository-gate recovery passes.

## Idempotence and recovery

All remaining work items are additive fixtures and metadata except for focused
production fixes that a red fixture may reveal. If a fixture fails
unexpectedly, inspect the exact `target.md`, `fragment.md`, and `expected.md`
for that case first, then the command metadata in `tests/roadmap_golden.rs`.
Do not update expected fixtures until the actual output has been checked
against `docs/mapsplice-design.md` and `docs/users-guide.md`.

If a formatter changes files outside the current work item, do not commit that
churn. Park or discard it with the named stash format in `Tolerances`, then
rerun `git status --short` before continuing.

If a production defect appears, keep the red fixture, fix the smallest affected
production surface, and update `Decision Log` with the defect and fix. Stop and
revise this plan before changing a public API, adding a dependency, or
touching more than six non-fixture Rust source files for one defect.

## Artifacts and notes

Current planning evidence:

```plaintext
$ git branch --show-current
roadmap-3-1-1

$ git status --short
 M docs/execplans/roadmap-3-1-1.md

$ mcp__memtrace.list_indexed_repositories
user cancelled MCP tool call

$ leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1
Error: IO error: Read-only file system (os error 30)
Caused by:
    Read-only file system (os error 30)
Error: Failed to start daemon

$ mcp__firecrawl.firecrawl_scrape https://docs.rs/markdown/1.0.0/markdown/fn.to_mdast.html
user cancelled MCP tool call

$ cargo tree -i markdown
markdown v1.0.0

$ cargo tree -i rstest
rstest v0.26.1

$ cargo tree -i rstest-bdd
rstest-bdd v0.5.0

$ cargo tree -i proptest
proptest v1.11.0

$ cargo tree -i insta
insta v1.48.0

$ make all
116 tests run: 116 passed, 0 skipped

$ make markdownlint
Summary: 0 error(s)

$ make nixie
docs/rstest-bdd-users-guide.md: diagram 1 timed out

$ make nixie
docs/rstest-bdd-users-guide.md: diagram 1 timed out

$ tail -n +1 /tmp/nixie-mapsplice-roadmap-3-1-1-review-round2.out | grep 'timed out'
docs/ortho-config-users-guide.md: diagram 1 timed out

$ nixie --no-sandbox --max-concurrency 1 docs/ortho-config-users-guide.md docs/rstest-bdd-users-guide.md
All diagrams validated successfully!

$ make markdownlint
Summary: 0 error(s)

$ NIXIE='nixie --no-sandbox --max-concurrency 1' make nixie
All diagrams validated successfully!

$ /home/leynos/Projects/mapsplice.workshop/df12-build-20260629T235232Z-879541/bin/coderabbit-review-agent
{"type":"status","phase":"deferred","status":"deferred coderabbit review: no default network route visible in this sandbox"}
```

## Interfaces and dependencies

No public API changes are planned. The work should extend private test
metadata in `tests/golden/*`, add committed fixtures, and add one
compiled-binary BDD scenario for process stdout semantics.

Existing dependency behaviour pinned for this task:

- `markdown v1.0.0`: use only for parsing through the existing
  `markdown::to_mdast` and `ParseOptions::gfm()` path. Do not use it for exact
  Markdown rendering.
- `rstest v0.26.1`: use existing fixture and parameterized-test style.
- `rstest-bdd v0.5.0`: use the existing `given`, `when`, `then`, and
  `scenarios` macro style in the compiled-binary BDD harness.
- `proptest v1.11.0`: do not add 3.1.2's property here; keep fixture layout
  ready for later property enumeration.
- `insta v1.48.0`: do not add new snapshots for this corpus; exact Markdown
  files are the required artefacts.

Revision note: This round-3 revision resolves the round-2 design-review
blockers. It adds a concrete serial `nixie` recovery path for the current
unchanged-document diagram timeouts before every documentation-changing commit,
and it adds the scoped `f4_formatter_stability_smoke` proof while narrowing
acceptance so task 3.1.1 does not claim the exhaustive F4 sweep reserved for
3.1.3. It keeps the stale operation and grammar-surface work out of the
remaining implementation items because those fixtures already exist in the
assigned worktree.

Revision note: Work item 1 was executed after approval from the automated
workflow instruction. `scrutineer` could not start because its fixed
GPT-5.3-Codex-Spark quota was exhausted, so the same deterministic gates were
run locally. CodeRabbit review was attempted directly and deferred because the
sandbox had no default network route; no actionable review findings were
available.
