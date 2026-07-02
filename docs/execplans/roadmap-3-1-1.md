# Assemble grammar-surface and per-contract golden fixtures

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

## Purpose / big picture

Roadmap task 3.1.1 is complete when `mapsplice` has a committed
golden-fixture corpus that proves the supported roadmap grammar surface,
supported edit operations, and every fidelity or contract guarantee that can be
shown by deterministic examples. A maintainer should be able to inspect a
fixture directory, run the focused golden test, and see raw Markdown bytes or
typed failure outcomes compared exactly.

This is the first planning-round draft for task 3.1.1. It does not begin
fixture implementation. Branch-local evidence shows that `origin/main` already
contains operation, grammar-surface, and reference-rewrite fixtures, so the
implementation extends the existing harness and does not recreate proven cases.
Roadmap task 3.1.2's generated no-op round-trip property and roadmap task
3.1.3's exhaustive rendered-output Markdown gate sweep remain out of scope.

## Constraints

- Work only in `/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1`.
- Do not edit the root/control worktree at `/home/leynos/Projects/mapsplice`.
- Treat `origin/main` as canonical and the integration branch as `main`.
- Do not begin implementation until this DRAFT is approved by the controlling
  workflow.
- Follow `AGENTS.md`, `docs/mapsplice-design.md`,
  `docs/developers-guide.md`, `docs/users-guide.md`, `docs/roadmap.md`,
  `docs/documentation-style-guide.md`, `docs/scripting-standards.md`,
  `docs/contributing.md`, and `docs/execplans/initial-tool.md`.
- Keep prose, comments, fixture text, and commit messages in en-GB Oxford
  spelling.
- Use Memtrace first for canonical main-branch code search and graph context
  when the MCP server accepts calls. If it rejects or cancels a call, record the
  exact result and continue with bounded branch-local evidence.
- Use `leta` for branch-local symbol navigation when it works. If it cannot
  start or add the workspace, record the exact failure and fall back to precise
  file inspection for this task.
- Use `sem` for entity-level history and diff inspection instead of raw
  `git log` or `git blame`.
- Do not add a new external dependency. If a work item appears to require one,
  stop and revise this plan with locked-source and official-documentation
  evidence before implementation continues.
- Do not redesign the roadmap grammar, operation semantics,
  dependency-reference model, command-line interface, or public library API
  unless a red fixture exposes a real defect. Keep any defect fix in the same
  atomic work item as the fixture that proves it.
- Fixture files are committed test inputs and expected outputs, not generated
  artefacts.
- Format only Markdown files changed by the current work item. Do not run
  `make fmt`, `mdformat-all`, or any repository-global formatter for this task.
- Run tests, lints, and gates sequentially. Commands that may produce long
  output must use `set -o pipefail` and `tee` to a branch-specific file under
  `/tmp`.
- Commit after each work item, and gate each commit before moving on.

## Tolerances

- If `git branch --show-current` is not `roadmap-3-1-1`, stop before editing.
- If `git status --short` is not clean after the approved plan commit, stop
  before fixture implementation.
- If a work item needs a public API signature change, stop and revise this
  plan.
- If a work item needs a new crate, stop and revise this plan with
  locked-source and official-documentation evidence for that crate.
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
- If plain `make nixie` fails only with an unchanged-document renderer timeout,
  a serial repository-gate recovery may be used:

  ```bash
  NIXIE='nixie --no-sandbox --max-concurrency 1' make nixie
  ```

  If the serial recovery also fails, stop before commit and record both log
  paths in `Decision Log`.

## Risks

- Risk: Existing branch-local fixtures already cover part of the requested
  corpus, so blindly replaying the task could create duplicates. Severity:
  high. Likelihood: confirmed. Mitigation: preserve verified existing coverage
  and add only named missing cases.

- Risk: Some guarantees are failure contracts rather than successful output
  contracts. Severity: medium. Likelihood: high. Mitigation: model those cases
  as typed-error expectations with unchanged-target assertions, not as
  successful expected-output files.

- Risk: A table-driven harness can obscure which fixture failed. Severity:
  medium. Likelihood: medium. Mitigation: every added case must have a stable
  Rust test function name, and assertion failures must include the case name.

- Risk: Task 3.1.2 needs to enumerate conformant fixtures later. Severity:
  medium. Likelihood: medium. Mitigation: store successful examples under
  `tests/fixtures/golden/<case-name>/` with `target.md`, optional
  `fragment.md`, and `expected.md`; keep failure fixtures explicitly marked in
  Rust metadata.

- Risk: Official documentation retrieval for external crates may be unavailable
  in the agent session. Severity: medium. Likelihood: confirmed for this
  planning round. Mitigation: do not rely on web-only crate claims; pin
  load-bearing behaviour to locked local source and existing repository code.

## Progress

- [x] (2026-07-02T06:06:18Z) Confirmed the assigned worktree and branch:
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1` on
  `roadmap-3-1-1`.
- [x] (2026-07-02T06:06:18Z) Loaded the required planning, navigation, Rust,
  CLI and prose skills used in this round: `execplans`, `leta`, `sem`,
  `memtrace-first`, `firecrawl-mcp`, `rust-router`, `rust-unit-testing`,
  `rust-errors`, `rust-verification`, `proptest`, `domain-cli-and-daemons`, and
  `en-gb-oxendict-style`.
- [x] (2026-07-02T06:06:18Z) Read the source-of-truth documents listed in
  `Context and orientation`.
- [x] (2026-07-02T06:06:18Z) Attempted Memtrace, Leta, and Firecrawl; their
  exact failures are recorded in `Surprises & Discoveries`.
- [x] (2026-07-02T06:06:18Z) Verified branch-local existing golden cases,
  fixture layout, BDD surface, Makefile gates, locked dependency versions, and
  the specific locked crate symbols recorded in `Research evidence`.
- [x] (2026-07-02T06:06:18Z) Rewrote this ExecPlan as a clean first-round
  DRAFT.
- [x] (2026-07-02T10:10:00Z) Work item 1: formatted the ExecPlan, ran
  `make all`, `make markdownlint`, and `make nixie` with tee logs under
  `/tmp`, recorded the unavailable `scrutineer` and deferred CodeRabbit
  results, and prepared the plan handoff commit.
- [x] (2026-07-02T10:27:00Z) Work item 2: added eight golden cases for F1,
  F2, F3/C5, F4, C2, C3, and C4 coverage. The red focused run failed only
  because the new fixture files were absent; after adding fixtures, the
  focused golden suite passed with 31 tests, the F4 copied-file formatter diff
  was empty, and `make all`, `make markdownlint`, and `make nixie` passed.
- [x] (2026-07-02T11:11:00Z) Work item 3: added C6 stdout and in-place
  success fixtures, F5 invalid-roadmap, level-mismatch, and missing-anchor
  failure fixtures, and the compiled-binary missing-anchor `--in-place` BDD
  scenario. The red focused run failed only for the absent new fixtures; after
  adding them, the focused golden suite passed with 36 tests, the focused BDD
  scenario passed, and `make all`, `make markdownlint`, and `make nixie`
  passed.
- [x] (2026-07-02T11:45:00Z) Work item 4: marked only roadmap task 3.1.1
  complete, ran the completion gates, and recorded final evidence. `make all`
  and `make markdownlint` passed; plain `make nixie` hit an unchanged
  Mermaid-renderer timeout, so the documented serial recovery was run after
  warming the existing diagram docs and then passed.

## Surprises & Discoveries

- Memtrace `list_indexed_repositories` returned
  `user cancelled MCP tool call`. Canonical-main graph context was unavailable,
  so this planning round uses bounded branch-local evidence from documentation,
  local source inspection, `cargo tree`, and `sem`. This is not a blocker.
- `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1`
  failed with `Error: IO error: Read-only file system (os error 30)`. Retrying
  `leta files` without adding the workspace failed with
  `Error: Failed to start daemon`. Branch-local verification therefore used
  precise inspection of known files. This is not a blocker.
- Firecrawl `firecrawl_scrape` for docs.rs pages for `markdown`, `rstest`, and
  `proptest` returned `user cancelled MCP tool call`. Official web
  documentation was unavailable, so this plan avoids web-only crate claims and
  pins load-bearing library behaviour to locked local source.
- `sem diff --from origin/main --to HEAD --format json` returned zero semantic
  changes before this plan edit.
- The current branch-local harness already has a split golden-test surface in
  `tests/golden/{mod.rs,case.rs,workspace.rs,runner.rs,assertions.rs}`. It
  drives `run_from_args`, compares expected output as raw fixture text, and
  supports typed failure expectations for dangling dependencies, level
  mismatches, and missing anchors.
- `scrutineer` could not start for the work item 1 gate pass because its fixed
  model quota was exhausted: `You've hit your usage limit for
  GPT-5.3-Codex-Spark. Switch to another model now, or try again at Jul 7th,
  2026 12:20 PM.` The same deterministic gates were run locally with the
  planned tee logs.
- CodeRabbit review for work item 1 was deferred by the local wrapper with
  `deferred coderabbit review: no default network route visible in this
  sandbox`. This is an open review issue for the supervisor rather than a
  deterministic gate failure.
- CodeRabbit review for work item 2 returned the same deferred status:
  `deferred coderabbit review: no default network route visible in this
  sandbox`. The deterministic gates passed; the review remains an open
  supervisor issue.
- CodeRabbit review for work item 3 returned the same deferred status:
  `deferred coderabbit review: no default network route visible in this
  sandbox`. The deterministic gates passed; the review remains an open
  supervisor issue.
- Plain `make nixie` for work item 4 timed out while rendering an unchanged
  Mermaid diagram in `docs/rstest-bdd-users-guide.md`. The same diagram passed
  when validated directly, and the documented serial recovery
  `NIXIE='nixie --max-concurrency 1' make nixie` then passed for the full
  repository.
- CodeRabbit review for work item 4 returned the same deferred status:
  `deferred coderabbit review: no default network route visible in this
  sandbox`. The deterministic gates passed; the review remains an open
  supervisor issue.

## Decision Log

- Decision: Keep this plan in `DRAFT` and require explicit approval before
  implementation. Rationale: the `execplans` skill requires a draft phase and
  the task says this is the first planning round. Date/Author: 2026-07-02 /
  Codex.

- Decision: Extend the existing split golden harness rather than replacing it.
  Rationale: `tests/golden/runner.rs::assert_golden_case` already drives
  `run_from_args`, dispatches success and failure expectations, and keeps case
  metadata explicit. Date/Author: 2026-07-02 / Codex.

- Decision: Use exact Markdown files, not new `insta` snapshots, for task
  3.1.1. Rationale: `docs/mapsplice-design.md`, section 8, requires exact
  input-and-expected Markdown pairs; `insta` remains suitable for existing CLI
  help snapshots but is not the corpus mechanism for this task. Date/Author:
  2026-07-02 / Codex.

- Decision: Keep existing reference-rewrite fixtures in
  `tests/fixtures/reference_rewrite/`. Rationale: they already prove required
  C3 adversarial classes, and moving them would add churn without improving
  coverage. Date/Author: 2026-07-02 / Codex.

- Decision: Add a scoped F4 formatter-stability smoke fixture in task 3.1.1 and
  leave the full rendered-output sweep to task 3.1.3. Rationale:
  `docs/mapsplice-design.md`, sections 5 and 8, require F4 evidence, while
  `docs/roadmap.md` assigns the exhaustive Markdown-gate sweep to 3.1.3.
  Date/Author: 2026-07-02 / Codex.

- Decision: Prove process-level stdout and stderr semantics for fail-closed
  `--in-place` errors through the compiled-binary BDD harness. Rationale: the
  golden harness observes `run_from_args` results and target bytes, while the
  BDD harness observes real process stdout and stderr. Date/Author:
  2026-07-02 / Codex.

- Decision: Continue work item execution after the `scrutineer` quota failure
  by running the exact deterministic gates locally. Rationale: the delegated
  gate runner was unavailable for quota reasons, while the repository commands
  were available in the assigned worktree and produced the required tee logs.
  Date/Author: 2026-07-02 / Codex.

- Decision: Put the F4 formatter-stability smoke table and code block in the
  preamble, and the nested list in a task body. Rationale: renderer output that
  places a preamble list before another preamble block needs an extra separator
  that the Markdown fixer collapses. Splitting the constructs keeps the
  rendered fixture representative, golden-comparable, and stable under
  `mdtablefix` plus `markdownlint-cli2 --fix`. Date/Author: 2026-07-02 /
  Codex.

- Decision: Move the contract golden tests into
  `tests/roadmap_golden/contracts.rs` while keeping the original operation and
  grammar-surface tests in `tests/roadmap_golden.rs`. Rationale: adding C6 and
  F5 cases would push the original integration-test file beyond the
  repository's 400-line file-size limit; the split keeps fixture groups
  discoverable without changing harness behaviour. Date/Author: 2026-07-02 /
  Codex.

- Decision: Accept the documented serial `nixie` recovery for work item 4
  after pre-validating the unchanged diagram-bearing documents directly.
  Rationale: the failing diagram was unchanged by this task and direct
  validation proved the source was valid; the final serial `make nixie` run
  passed and produced full-repository gate evidence. Date/Author: 2026-07-02 /
  Codex.

## Addenda

- [ ] 3.1.1.1. Document exact fixture EOF whitespace policy.
  - Source: review:3.1.1 (low).
  - Scope: Document when raw-byte golden fixtures may preserve EOF whitespace
    and how maintainers distinguish intentional fixture fidelity from
    accidental whitespace churn. Lightweight addendum pass.
- [ ] 3.1.1.2. Consolidate the golden fixture harness.
  - Source: audit:3.1.1 (medium).
  - Scope: Refactor the golden fixture case construction into a parameterized
    harness that preserves named cases while reducing repeated helper and
    assertion edits. Lightweight addendum pass.

## Outcomes & Retrospective

Roadmap task 3.1.1 is complete. The fixture corpus now includes eight
per-contract fidelity/reference cases and five output-mode or fail-closed
cases, plus the compiled-binary missing-anchor `--in-place` scenario. The
roadmap marks only task 3.1.1 complete; 3.1.2 and 3.1.3 remain open for the
round-trip property and rendered-output gate sweep.

## Context and orientation

`mapsplice` edits constrained roadmap-shaped Markdown by parsing Markdown into
a roadmap model, applying one structural operation, renumbering affected items,
rewriting dependency references, and rendering Markdown. The source of truth
for command semantics and accepted grammar is `docs/users-guide.md`; the source
of truth for fidelity guarantees, contract guarantees, and fixture
requirements is `docs/mapsplice-design.md`.

Read these documents before implementation:

- `AGENTS.md`: core workflow, commits after each change, Rust testing rules,
  Markdown gates, and path-safe formatting.
- `docs/roadmap.md`: section 3, especially task 3.1.1 and the boundaries with
  tasks 3.1.2 and 3.1.3.
- `docs/mapsplice-design.md`: sections 2, 4, 5, 6, 7, 8, and 9.
- `docs/users-guide.md`: `The roadmap shape mapsplice expects`, `Command
  overview`, `Command details`, `Output modes`, and `Validation rules and
  failure cases`.
- `docs/developers-guide.md`: sections 1, 2, 3, 6, and 7.
- `docs/contributing.md`: `Development gates`.
- `docs/documentation-style-guide.md`: `Spelling`, `Markdown rules`, and
  `Formatting`.
- `docs/scripting-standards.md`: `Language and runtime`, for reproducible
  command practice if any helper script becomes necessary.
- `docs/execplans/initial-tool.md`: `Scope and grammar assumptions`,
  `Constraints`, `Proposed implementation`, and `Decision Log`, especially the
  accepted decisions to use mdast parsing, a constrained grammar, deterministic
  rendering, `rstest`, and `rstest-bdd`.

The implementation surfaces relevant to this task are:

- `tests/roadmap_golden.rs`, where named golden cases are registered.
- `tests/golden/case.rs`, which defines `GoldenCase`, `GoldenCommand`,
  `GoldenExpectation`, `SuccessOutput`, `FailureOutput`, and `ExpectedError`.
- `tests/golden/runner.rs`, which prepares a temporary workspace and dispatches
  success or failure assertions through `run_from_args`.
- `tests/golden/assertions.rs`, which compares stdout, target bytes, in-place
  writes, and typed `MapspliceError` variants.
- `tests/golden/workspace.rs`, which resolves fixture paths and reads expected
  output as raw fixture text.
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

External library claims are pinned to locked local source and current
repository code. Official web documentation retrieval was attempted through
Firecrawl and cancelled by the host session, so the plan does not depend on
unstated web-only behaviour.

- `cargo tree -i markdown`, `cargo tree -i rstest`,
  `cargo tree -i rstest-bdd`, `cargo tree -i proptest`, and
  `cargo tree -i insta` show `markdown v1.0.0`, `rstest v0.26.1`,
  `rstest-bdd v0.5.0`, `proptest v1.11.0`, and `insta v1.48.0`.
- `Cargo.lock` pins the same locked versions for those crates.
- `markdown-1.0.0/src/lib.rs` defines
  `pub fn to_mdast(value: &str, options: &ParseOptions)`.
- `markdown-1.0.0/src/configuration.rs` defines `ParseOptions::gfm()` and
  `Constructs::gfm()`; its tests assert that GFM enables
  `gfm_autolink_literal`, and the same source sets `gfm_table` and
  `gfm_task_list_item` to true. `src/roadmap/parse/mod.rs::parse_root` already
  uses `to_mdast(markdown, &ParseOptions::gfm())`. Fixtures may rely on GFM
  task lists and tables being parsed by the existing code path; they must not
  rely on `markdown` for exact Markdown rendering.
- `rstest-0.26.1/src/lib.rs` documents and re-exports `#[rstest]` and
  `#[fixture]`. New tests should follow the existing `#[fixture]` and
  `#[rstest]` style in `tests/roadmap_golden.rs`.
- `rstest-bdd-0.5.0/src/lib.rs` exposes the runtime step registry and
  `StepContext`; `rstest-bdd-macros-0.5.0` supplies the `given`, `when`,
  `then`, and `scenario` macros already used by `tests/behaviour_cli.rs` and
  `tests/steps/cli_steps.rs`.
- `proptest-1.11.0/src/sugar.rs` defines the `proptest!` macro. Task 3.1.1
  must not add task 3.1.2's property, but fixture layout should remain easy for
  that later property to enumerate.
- `insta-1.48.0/src/macros.rs` defines snapshot assertion macros. This task
  deliberately avoids new snapshots because exact Markdown fixture files are
  the design requirement.
- `tests/golden/workspace.rs::expected_output` returns raw fixture text, so
  expected output is compared exactly after UTF-8 loading.
- `Makefile` defines `all: check-fmt lint typecheck test`, so `make all`
  includes `typecheck` on current `origin/main`.
- `Makefile` routes `make nixie` through the `NIXIE` variable and passes
  `--no-sandbox`; the documented timeout recovery uses that variable to add
  `--max-concurrency 1`.
- `src/roadmap/render.rs::render_roadmap` joins rendered blocks and appends
  exactly one final newline to non-empty output.

## Verified existing coverage

The following surfaces are already present in this worktree and are not future
work items:

- Operation fixtures and tests: `append_phase`, `insert_phase_before`,
  `insert_step_after`, `insert_task_before`, `insert_sub_task_after`,
  `delete_task`, `replace_step`, and `replace_sub_task`.
- Grammar-surface fixtures and tests: `preamble_preserved`,
  `phase_step_task_surface`, `multi_line_task_body`, `nested_bullets`,
  `tables_preserved`, `code_blocks_preserved`, and `addendum_body_surface`.
- Existing C3 reference-rewrite adversarial fixtures and tests:
  `section_reference`, `version_quantity`,
  `section_reference_outside_requires`, `substring_non_match`, and
  `multi_id_requires`.

An implementer must not recreate these cases. If a later gate exposes a defect
in one of them, fix that defect in the smallest related work item and record
the reason in `Decision Log`.

## Plan of work

### Work item 1: Format, gate, review, and commit this ExecPlan revision

This item implements `AGENTS.md` `Plans`, `Commands`, `Change Quality &
Committing`, and `Markdown Guidance`; `docs/developers-guide.md` section 7;
`docs/contributing.md` `Development gates`; and the `execplans` approval gate.
It exists solely to keep the approved plan edit out of later fixture commits.

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
sem diff --format json 2>&1 | tee /tmp/sem-mapsplice-roadmap-3-1-1-plan.out
git status --short
```

Run `NIXIE='nixie --no-sandbox --max-concurrency 1' make nixie` only if plain
`make nixie` fails solely with an unchanged-document timeout. Commit only after
plain `make nixie` or the serial recovery passes and `git status --short`
shows only the intended ExecPlan change before staging.

### Work item 2: Add remaining per-contract fidelity and reference fixtures

This item implements `docs/mapsplice-design.md` sections 5, 6, 7, and 8;
`docs/roadmap.md` task 3.1.1; `docs/developers-guide.md` sections 2, 3, and 6;
and `docs/execplans/initial-tool.md` `Proposed implementation` items 2, 4, 5,
and 7.

Skills to load: `leta`, `rust-router`, `rust-unit-testing`, `rust-errors`,
`rust-verification`, `proptest`, `sem`, and `en-gb-oxendict-style`.

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
  the copy, and `diff -u` proves the formatter made no changes.
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
sem diff --format json 2>&1 | tee /tmp/sem-mapsplice-roadmap-3-1-1-contracts.out
```

Run `NIXIE='nixie --no-sandbox --max-concurrency 1' make nixie` only if plain
`make nixie` fails solely with an unchanged-document timeout. Commit only after
all focused tests, formatters, `make all`, `make markdownlint`, and either
plain `make nixie` or the serial recovery pass.

### Work item 3: Add output-mode and fail-closed fixtures

This item implements `docs/mapsplice-design.md` sections 5 and 6;
`docs/users-guide.md` `Output modes` and `Validation rules and failure cases`;
`docs/developers-guide.md` sections 2, 3, and 6; and
`docs/execplans/initial-tool.md` `Proposed implementation` items 6 and 7.

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
sem diff --format json 2>&1 | tee /tmp/sem-mapsplice-roadmap-3-1-1-output-failure.out
```

Run `NIXIE='nixie --no-sandbox --max-concurrency 1' make nixie` only if plain
`make nixie` fails solely with an unchanged-document timeout. Commit only after
all focused tests, formatters, `make all`, `make markdownlint`, and either
plain `make nixie` or the serial recovery pass.

### Work item 4: Mark roadmap task 3.1.1 complete

This item implements the documentation completion for `docs/roadmap.md` task
3.1.1 after work items 1-3 have passed. It follows `AGENTS.md` `Documentation
Maintenance`, `AGENTS.md` `Markdown Guidance`, and `docs/developers-guide.md`
section 7.

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
sem diff --format json 2>&1 | tee /tmp/sem-mapsplice-roadmap-3-1-1-complete.out
```

Run `NIXIE='nixie --no-sandbox --max-concurrency 1' make nixie` only if plain
`make nixie` fails solely with an unchanged-document timeout. Commit only after
`make all`, `make markdownlint`, and either plain `make nixie` or the serial
recovery pass.

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
  `markdownlint-cli2 --fix`.
- Failure cases assert typed `MapspliceError` variants and unchanged target
  bytes.
- The missing-anchor `--in-place` failure has compiled-binary BDD coverage
  proving the command fails, stdout is empty, stderr reports the missing
  anchor, and the target file remains unchanged.
- Expected output is compared as raw fixture text.
- Rendered non-empty Markdown ends in exactly one final newline.
- `docs/roadmap.md` marks only 3.1.1 complete after the corpus and gates are
  complete.
- The final committed state passes:

  ```bash
  set -o pipefail
  make all 2>&1 | tee /tmp/all-mapsplice-roadmap-3-1-1-final.out
  make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-final.out
  make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-final.out
  ```

  Run `NIXIE='nixie --no-sandbox --max-concurrency 1' make nixie` only if
  plain `make nixie` fails solely with an unchanged-document timeout.

## Idempotence and recovery

The fixture work is additive and can be retried safely from a clean tree. If a
focused fixture fails, inspect the failing case before editing production code.
If formatter churn touches unrelated files, use the named stash command in
`Tolerances` and continue only with the intended paths. If a gate fails on a
document or Rust file changed by the current work item, fix that file before
committing. If a gate fails on an unchanged file, record the log path and
confirm whether the serial `make nixie` recovery is the documented path.

## Artifacts and notes

The following transcripts are the planning-round evidence to preserve:

```plaintext
$ git branch --show-current
roadmap-3-1-1

$ mcp__memtrace.list_indexed_repositories
user cancelled MCP tool call

$ leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1
Error: IO error: Read-only file system (os error 30)

$ leta files
Error: Failed to start daemon

$ mcp__firecrawl.firecrawl_scrape \
  https://docs.rs/markdown/1.0.0/markdown/fn.to_mdast.html
user cancelled MCP tool call

$ sem diff --from origin/main --to HEAD --format json
{"summary":{"fileCount":0,"added":0,"modified":0,"deleted":0,"moved":0,"renamed":0,"reordered":0,"orphan":0,"total":0},"changes":[]}
```

## Interfaces and dependencies

Use the existing public library and test-harness interfaces:

```rust
pub fn run_from_args<I, T>(args: I) -> Result<RunOutcome>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone;
```

```rust
pub(crate) struct GoldenCase {
    pub(crate) name: &'static str,
    pub(crate) command: GoldenCommand,
    pub(crate) target: FixturePath,
    pub(crate) fragment: Option<FixturePath>,
    pub(crate) expectation: GoldenExpectation,
}
```

```rust
pub(crate) enum GoldenExpectation {
    Success { output: SuccessOutput },
    Failure { error: ExpectedError, output: FailureOutput },
}
```

Do not introduce new dependencies. Use the locked versions already present in
`Cargo.lock`: `markdown 1.0.0`, `rstest 0.26.1`, `rstest-bdd 0.5.0`,
`rstest-bdd-macros 0.5.0`, `proptest 1.11.0`, and `insta 1.48.0`. Do not add
new `insta` snapshots for this task.

## Revision note

2026-07-02 work item 1 execution: moved the plan to `IN PROGRESS`, recorded
the successful deterministic gate evidence, noted the unavailable `scrutineer`
and deferred CodeRabbit review, and marked the plan handoff work item complete.

2026-07-02 work item 2 execution: added eight per-contract golden cases,
recorded red and green fixture evidence, documented the F4 fixture layout
decision, and noted that CodeRabbit review was deferred by the sandbox network
state.

2026-07-02 work item 3 execution: added five output-mode or fail-closed golden
cases, added the compiled-binary missing-anchor `--in-place` BDD scenario,
split contract tests into `tests/roadmap_golden/contracts.rs`, and noted that
CodeRabbit review was deferred by the sandbox network state.

2026-07-02 work item 4 execution: marked roadmap task 3.1.1 complete, recorded
final gate evidence including the successful serial `nixie` recovery, and
noted that CodeRabbit review was deferred by the sandbox network state.

2026-07-02 planning round 1: reset the plan to `DRAFT`, removed prior
implementation-session status from the plan narrative, and preserved only
tooling and repository evidence that is useful before implementation. This
keeps the next step as explicit approval rather than fixture work.
