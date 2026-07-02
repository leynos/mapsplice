# Assert gate-clean rendered output

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

Roadmap task 3.1.3 proves fidelity guarantee F4: every successful rendered
roadmap fixture is already accepted by the house Markdown gates and is stable
under the house formatter. A maintainer should be able to run the golden
fixture suite and trust that the Markdown emitted by `mapsplice` will not need
follow-up formatting.

The observable outcome is a failing test when rendered output would be changed
by the formatter or rejected by `markdownlint-cli2`, and a passing `make all`,
`make markdownlint`, and `make nixie` after the renderer and fixtures are made
gate-clean.

## Constraints

- Work only in `/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-3`.
- Do not edit the root/control worktree.
- Do not run `make fmt`, `mdformat-all`, or any repository-wide formatter.
  Format only changed Markdown files with path-scoped `mdtablefix` followed by
  path-scoped `markdownlint-cli2 --fix`.
- Do not set this plan to `BLOCKED` for Memtrace, Firecrawl, Leta, GrepAI, or
  sem advisory-tool failures. Record the failure and continue with bounded
  local evidence.
- Preserve public library API signatures unless a later approved revision of
  this plan explicitly changes that constraint.
- Keep all Rust modules under 400 lines and ensure every new Rust module starts
  with a `//!` module comment.
- Use `rstest` fixtures for shared setup. Do not mutate environment variables
  directly in tests.
- Do not introduce a new external dependency. Existing dev-dependencies
  `tempfile`, `rstest`, and `serial_test` are sufficient.
- A formatter-stability assertion must check actual rendered output, not merely
  fixture files under `tests/fixtures/golden`, because
  `.markdownlint-cli2.jsonc` ignores `tests/fixtures/golden/**/*.md`.

## Tolerances (exception triggers)

- Scope: stop and revise this plan if implementation needs changes outside
  `src/roadmap/render.rs`, `src/roadmap/render_table.rs`,
  `src/roadmap/render_text.rs`, `tests/golden/*`, `tests/roadmap_golden*`,
  `tests/fixtures/golden/**`, `tests/fixtures/reference_rewrite/**`, or
  `docs/roadmap.md`.
- Interface: stop if any public API in `src/lib.rs`, `src/roadmap/mod.rs`, or
  exported error type must change.
- Dependencies: stop if a new crate, npm package, binary, or system dependency
  is required.
- Formatter drift: stop if satisfying F4 would require weakening F1 content
  preservation for input that is already formatter-stable.
- Test iterations: stop if the focused golden fixture test still fails for a
  new reason after three implementation attempts.
- Advisory tools: if Memtrace, Leta, Firecrawl, or sem fails, record the exact
  command or tool result in `Surprises & Discoveries` and continue with local
  source, tests, and docs.

## Risks

- Risk: F1 exact preservation and F4 formatter stability can appear to conflict
  when existing fixtures contain Markdown that the house formatter rewrites.
  Severity: high. Likelihood: high. Mitigation: treat formatter-stable Markdown
  as the golden corpus invariant, then update renderer spacing only where the
  formatter probe proves current output is not stable.
- Risk: `make markdownlint` does not lint golden fixtures because of the ignore
  rule in `.markdownlint-cli2.jsonc`. Severity: high. Likelihood: confirmed.
  Mitigation: tests must copy actual rendered output into a temporary Markdown
  file outside the ignored fixture tree before linting and formatting it.
- Risk: shelling out to formatter binaries inside tests can produce vague
  failures. Severity: medium. Likelihood: medium. Mitigation: centralize
  command execution in one test helper that reports command, exit status,
  stdout, and stderr.
- Risk: `mdtablefix --renumber` may renumber ordinary ordered lists in rendered
  task bodies. Severity: medium. Likelihood: medium. Mitigation: add a fixture
  that exercises nested lists and assert the formatter is a no-op after the
  renderer uses the formatter-stable shape.

## Progress

- [x] (2026-07-02 10:12+02:00) Loaded `execplans`, `leta`,
  `rust-router`, `rust-unit-testing`, `rust-verification`, `rust-errors`,
  `sem`, and `firecrawl-mcp` skills for planning.
- [x] (2026-07-02 10:14+02:00) Confirmed branch leaf is
  `roadmap-3-1-3`, so this plan path is `docs/execplans/roadmap-3-1-3.md`.
- [x] (2026-07-02 10:18+02:00) Verified local documentation and source
  context for roadmap task 3.1.3, design guarantee F4, golden fixtures, and
  renderer entry points.
- [x] (2026-07-02 10:24+02:00) Verified the exact house formatter sequence
  from `/home/leynos/.local/bin/mdformat-all`:
  `mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place`, then
  `markdownlint-cli2 --fix`.
- [x] (2026-07-02 10:26+02:00) Verified direct linting of
  `tests/fixtures/golden/.../expected.md` lints zero files because the
  repository markdownlint configuration ignores that tree.
- [x] (2026-07-02 13:45+02:00) Revised the plan after design review to require
  passing final gates before every commit, to use an existence-safe changed
  Markdown file set, and to tee-log path-scoped formatter runs.
- [x] (2026-07-02 10:51+02:00) Implemented work item 1: added
  `tests/golden/format_gate.rs`, wired it into the golden harness, and proved
  the helper accepts stable rendered Markdown while rejecting formatter drift.
- [x] (2026-07-02 11:03+02:00) Implemented work item 2: renderer task bodies
  now emit formatter-stable addendum indentation and block spacing; golden
  fixtures were normalized to gate-clean rendered bytes.
- [x] (2026-07-02 11:08+02:00) Implemented work item 3: enabled the
  render-output gate assertion across successful golden stdout and in-place
  target outputs, and added metadata coverage for the markdownlint golden
  fixture ignore.
- [x] (2026-07-02 11:10+02:00) Implemented work item 4: marked roadmap task
  3.1.3 complete, reviewed semantic diff output, and recorded final validation
  evidence.

## Surprises & discoveries

- Observation: Memtrace MCP was available through tool discovery, but
  `list_indexed_repositories` returned `user cancelled MCP tool call`.
  Evidence: first planning call to `mcp__memtrace.list_indexed_repositories`.
  Impact: the plan uses bounded local docs, Leta, and file inspection for
  branch-local evidence and records Memtrace unavailability as advisory-tool
  failure, not a blocker.
- Observation: Firecrawl MCP calls were cancelled by the host.
  Evidence: `mcp__firecrawl.firecrawl_search` returned
  `user cancelled MCP tool call`. Impact: official-doc evidence uses the web
  fallback and local installed source; this is not a product blocker.
- Observation: In this planning revision, Leta workspace setup failed with
  `Error: IO error: Read-only file system (os error 30)` and `leta files docs`
  failed with `Error: Failed to start daemon`. Earlier planning evidence
  recorded intermittent `leta show` success for renderer symbols. Evidence:
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-3`
  and `leta files docs`. Impact: implementers must try Leta first for
  branch-local verification, record any exact failure, and then use bounded
  file inspection only for the affected symbols.
- Observation: `mdformat-all --help` is not a help command in this local
  wrapper; it ran the repository-wide formatter. Evidence: the wrapper source
  has no argument handling and always runs `with_all_md mdtablefix ...` then
  `with_all_md markdownlint-cli2 --fix`. Impact: the accidental churn was
  parked in a named discard stash for task 3.1.3 with reason
  `undo accidental mdformat-all help churn`. Implementers must not invoke
  `mdformat-all`.
- Observation: a temp-file formatter probe changed existing rendered expected
  outputs for tables, nested addendum items, code-block spacing, and multi-line
  paragraphs. Evidence: examples include
  `f4_formatter_stability_smoke/expected.md` table alignment,
  `addendum_body_surface/expected.md` nested checkbox indentation,
  `code_blocks_preserved/expected.md` blank lines around fences, and
  `multi_line_task_body/expected.md` paragraph joining. Impact: this task needs
  renderer changes plus fixture updates, not only a new assertion helper.
- Observation: During implementation, Memtrace remained unavailable at the
  repository discovery step. Evidence:
  `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`. Impact: branch-local verification used Leta
  where available and bounded file inspection after recording the advisory-tool
  failure.
- Observation: During implementation, Leta listed files successfully but symbol
  grep failed. Evidence:

  ```plaintext
  leta grep ".*" "tests/golden" -k function,method
  leta grep "render_(roadmap|task|sub_task|markdown_nodes|code_block|table)" "src/roadmap" -k function
  Error: Connection closed unexpectedly
  ```

  Impact: bounded source reads were used for the exact harness and renderer
  files named in this plan.
- Observation: The `scrutineer` sub-agent could not run deterministic gates.
  Evidence: sub-agent `019f2205-0657-72d2-969b-3737bcaf6377` errored with

  ```plaintext
  You've hit your usage limit for GPT-5.3-Codex-Spark. Switch to another model
  now, or try again at Jul 7th, 2026 12:20 PM.
  ```

  Impact: deterministic gates were run locally with tee logs; this is a
  verification-tooling failure, not a product blocker.
- Observation: CodeRabbit could not review work item 1 in this sandbox.
  Evidence:
  `/home/leynos/Projects/mapsplice.workshop/df12-build-20260629T235232Z-879541/bin/coderabbit-review-agent`
  returned

  ```json
  {
    "type": "status",
    "phase": "deferred",
    "status": "deferred coderabbit review: no default network route visible in this sandbox"
  }
  ```

  Impact: deterministic gates are green, but AI review remains deferred for
  supervisor follow-up.
- Observation: Work item 2 initially reproduced the known formatter drift in
  expected rendered fixtures, then a second probe after renderer and fixture
  changes produced no drift output. Evidence:
  `/tmp/formatter-drift-mapsplice-roadmap-3-1-3-after-item-2.out` was empty.
  Impact: the successful rendered corpus is now stable under the formatter
  sequence before the helper is wired into every case.
- Observation: The first `make nixie` run for work item 2 timed out rendering
  an unrelated diagram in `docs/ortho-config-users-guide.md`. Evidence:
  `/tmp/nixie-mapsplice-roadmap-3-1-3-work-item-2.out` recorded
  `diagram 1 timed out`. Impact: a sequential retry at
  `/tmp/nixie-mapsplice-roadmap-3-1-3-work-item-2-retry.out` passed, so this
  was treated as transient validation noise, not product debt.
- Observation: CodeRabbit could not review work item 2 in this sandbox.
  Evidence:
  `/home/leynos/Projects/mapsplice.workshop/df12-build-20260629T235232Z-879541/bin/coderabbit-review-agent`
  returned the same deferred network status as work item 1. Impact:
  deterministic gates are green, but AI review remains deferred for supervisor
  follow-up.
- Observation: CodeRabbit could not review work item 3 in this sandbox.
  Evidence:
  `/home/leynos/Projects/mapsplice.workshop/df12-build-20260629T235232Z-879541/bin/coderabbit-review-agent`
  returned the same deferred network status as work items 1 and 2. Impact:
  deterministic gates are green, but AI review remains deferred for supervisor
  follow-up.
- Observation: CodeRabbit could not review work item 4 in this sandbox.
  Evidence:
  `/home/leynos/Projects/mapsplice.workshop/df12-build-20260629T235232Z-879541/bin/coderabbit-review-agent`
  returned the same deferred network status as work items 1, 2, and 3. Impact:
  deterministic gates are green, but AI review remains deferred for supervisor
  follow-up.
- Observation: `sem diff --format json --file-exts .rs .md` completed before
  the final documentation commit and reported the expected changed Rust and
  Markdown entities only. Evidence:
  `/tmp/sem-diff-mapsplice-roadmap-3-1-3-work-item-4.out`. Impact: no
  additional semantic drift was found before final validation.
- Observation: Final `make nixie` validation hit the same transient Mermaid
  renderer timeout pattern as work item 2 before a sequential retry passed.
  Evidence: `/tmp/nixie-mapsplice-roadmap-3-1-3.out` timed out on
  `docs/rstest-bdd-users-guide.md`, and
  `/tmp/nixie-mapsplice-roadmap-3-1-3-retry.out` timed out on
  `docs/ortho-config-users-guide.md`; the passing retry is
  `/tmp/nixie-mapsplice-roadmap-3-1-3-retry-2.out`. Impact: final acceptance
  uses the passing retry and leaves the transient timeout evidence recorded.

## Decision log

- Decision: Assert F4 by copying actual rendered output to a temporary
  `rendered.md` outside ignored fixture directories, running the house
  formatter sequence there, byte-comparing the result, and then running
  `markdownlint-cli2` on the same temporary file. Rationale:
  `.markdownlint-cli2.jsonc` ignores `tests/fixtures/golden/**/*.md`, so direct
  fixture linting gives false confidence. The temp-file method checks rendered
  bytes using the real gate tools. Date/Author: 2026-07-02, planning agent.
- Decision: Reproduce the formatter sequence from the local `mdformat-all`
  wrapper without calling that wrapper. Rationale: `mdformat-all` is
  repository-wide and would reformat unrelated files; the task requires
  path-scoped formatting. Date/Author: 2026-07-02, planning agent.
- Decision: Do not add a new dependency for process assertions.
  Rationale: `std::process::Command`, existing `tempfile`, and the existing
  `TestResult` alias can express the checker with clear errors. Date/Author:
  2026-07-02, planning agent.
- Decision: Treat renderer output shape as the product surface to fix.
  Rationale: F4 is about rendered output; merely updating expected fixtures
  would leave `mapsplice` emitting bytes the formatter changes. Date/Author:
  2026-07-02, planning agent.
- Decision: Every work-item commit requires passing `make all`,
  `make markdownlint`, and `make nixie` first. Rationale: `AGENTS.md` says not
  to commit changes that fail quality gates, and this workflow requires each
  commit to be gated. A documented unrelated pre-existing failure is evidence
  for escalation, not permission to commit; only explicit human approval that
  changes this requirement may allow a different outcome. Date/Author:
  2026-07-02, planning revision agent.
- Decision: Path-scoped Markdown maintenance uses
  `git diff --name-only --diff-filter=ACMR -z -- '*.md'` and tee logs for the
  `mdtablefix` and `markdownlint-cli2 --fix` runs. Rationale: `ACMR` excludes
  deleted paths, so formatter arguments name files that still exist, and tee
  logs satisfy the repository command-output rule. Date/Author: 2026-07-02,
  planning revision agent.

## Outcomes & retrospective

No implementation has started. This first planning round produced the
self-contained plan and captured the tool failures and formatter behaviour that
an implementer must account for.

Work item 1 is implemented and validated. The helper uses a temporary rendered
Markdown file outside ignored golden fixture paths, applies
`mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place`, applies
`markdownlint-cli2 --fix`, byte-compares the formatted file against the
original rendered bytes, and then runs `markdownlint-cli2` without `--fix`.
Focused test evidence:
`cargo test --test roadmap_golden gate_clean -- --nocapture` passed with 2
tests. Full gate evidence: `make all` passed with 132 nextest tests and 7
doctests passed, 2 doctests ignored. CodeRabbit review is deferred because the
sandbox has no default network route.

Work item 2 is implemented and validated. The renderer now emits sub-task
markers with two-space nesting and preserves task-body block boundaries so
paragraphs, tables, and fenced code blocks remain stable under the house
formatter. Focused evidence: `cargo test --test roadmap_render -- --nocapture`
passed with 6 tests, and `cargo test --test roadmap_golden -- --nocapture`
passed with 38 tests. Full gate evidence: `make all` passed with 134 nextest
tests and 7 doctests passed, 2 doctests ignored; `make markdownlint` passed;
`make nixie` passed on retry after one unrelated diagram timeout. CodeRabbit
review is deferred because the sandbox has no default network route.

Work item 3 is implemented and validated. Successful golden outputs now run
through the render-output Markdown gate whether the rendered bytes are emitted
on stdout or written to the target in in-place mode, while expected failure
cases continue to assert target preservation without running the rendered
output gate. The metadata self-tests now document that direct linting of
`tests/fixtures/golden/**/*.md` is insufficient because the repository
markdownlint configuration ignores that tree. Focused evidence:
`cargo test --test roadmap_golden -- --nocapture` passed with 39 tests. Full
gate evidence: `make all` passed with 135 nextest tests and 7 doctests passed,
2 doctests ignored. CodeRabbit review is deferred because the sandbox has no
default network route.

Work item 4 is implemented and validated. Roadmap task 3.1.3 is marked
complete, the semantic diff was reviewed, and final gates passed at HEAD. Final
evidence: `make all` passed with 135 nextest tests and 7 doctests passed, 2
doctests ignored; `make markdownlint` passed with 35 files linted; `make nixie`
validated all Mermaid diagrams on the second sequential retry. CodeRabbit
review is deferred because the sandbox has no default network route.

## Context and orientation

The roadmap task is `docs/roadmap.md` task 3.1.3, "Assert gate-clean rendered
output". It requires rendered fixtures to pass the house Markdown gates and be
stable under the house formatter.

The design document defines the relevant guarantees:

- `docs/mapsplice-design.md` section 3 says the pipeline renders only the
  supported roadmap grammar.
- `docs/mapsplice-design.md` section 5 defines F1 content preservation, F3
  round-trip stability, and F4 gate-clean output.
- `docs/mapsplice-design.md` section 8 requires a golden-fixture corpus and
  says the round-trip property includes a second `mdformat-all` pass producing
  no diff.

The developers' guide defines the verification layers: `rstest` unit tests,
`rstest-bdd` behavioural tests, `proptest` properties, and `trybuild` plus
`insta` compatibility tests. This task belongs in the Rust integration golden
fixture layer, with small unit tests around the new gate helper.

The current renderer entry point is `src/roadmap/render.rs::render_roadmap`.
Leta showed that `render_roadmap` joins rendered blocks with blank lines and
ensures non-empty output ends in one final newline. It calls `render_tasks`,
`render_task`, `render_sub_task`, and `render_markdown_nodes`. The formatter
probe shows the most likely production edits are in:

- `src/roadmap/render.rs::render_task` and `render_sub_task` for nested
  addendum indentation and spacing between task-body blocks.
- `src/roadmap/render_text.rs::render_code_block` call sites or surrounding
  block joining for blank lines around fenced code blocks inside list items.
- `src/roadmap/render_table.rs::render_table` or preserved table handling for
  table alignment.

The golden fixture harness is in `tests/golden`. `tests/golden/runner.rs`
executes `mapsplice::run_from_args` for each `GoldenCase`, and
`tests/golden/assertions.rs` compares stdout or in-place target output against
expected strings. That is the correct place to enforce gate-clean rendered
output because it sees actual rendered bytes.

## Interfaces and dependencies

Use only existing local dependencies and tools:

- `tempfile = "3.27.0"` is already a dev-dependency and provides temporary
  directories for the render-output gate helper.
- `std::process::Command` runs `mdtablefix` and `markdownlint-cli2`.
- `mdtablefix 0.4.0` is installed at `/home/leynos/.cargo/bin/mdtablefix`.
  Local source and docs verified:
  `/home/leynos/Projects/mdtablefix/src/main.rs` lines 45-70 define the CLI
  flags; lines 86-118 show `--in-place` reads, processes, and writes files;
  `/home/leynos/Projects/mdtablefix/src/process.rs` lines 13-52 define the
  80-column wrap option and processing flags. Its README documents that
  `--wrap` reflows paragraphs and list items to 80 columns, `--renumber`
  rewrites ordered lists, `--breaks` standardizes thematic breaks, `--ellipsis`
  replaces `...`, `--fences` normalizes fenced code blocks, and `--in-place`
  modifies files.
- `markdownlint-cli2 0.22.1` with `markdownlint 0.40.0` is installed at
  `/home/leynos/.bun/bin/markdownlint-cli2`. Its local source
  `/home/leynos/.bun/install/global/node_modules/markdownlint-cli2/markdownlint-cli2.mjs`
  lines 203-223 process command arguments, lines 249-253 document `--fix`, and
  lines 749-765 apply fixable errors by reading the file, applying fixes, and
  writing it back. The official README at
  `https://github.com/DavidAnson/markdownlint-cli2` documents `--fix` and
  `--format`; web fallback verified the same command-line semantics.
- The local `/home/leynos/.local/bin/mdformat-all` wrapper lines 30-31 proves
  the repository formatter order:
  `mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place`, then
  `markdownlint-cli2 --fix`.

Firecrawl could not be used because the MCP call was cancelled. The implementer
does not need to choose between formatter alternatives; the plan pins the local
locked behaviour above.

## Plan of work

### Work item 1: Add a render-output Markdown gate helper

Purpose: create one reusable assertion that checks actual rendered bytes with
the real house formatter and linter.

Documentation and skills to read:

- `AGENTS.md`, Rust Specific Guidance and Testing.
- `docs/mapsplice-design.md` sections 5 and 8.
- `docs/developers-guide.md` sections 6 and 7.
- Skills: `leta`, `rust-router`, `rust-unit-testing`, `rust-errors`.

Edits:

- Add `tests/golden/format_gate.rs` with a module-level `//!` comment.
- In `tests/golden/mod.rs`, add `mod format_gate;` and export only the helper
  needed by `assertions.rs`.
- Implement
  `assert_gate_clean_rendered_output(name: &str, rendered: &str) -> TestResult`.
- The helper writes `rendered` to `tempfile::tempdir()/rendered.md`, reads the
  original bytes back, runs:

```bash
mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place <temp>/rendered.md
markdownlint-cli2 --fix <temp>/rendered.md
markdownlint-cli2 <temp>/rendered.md
```

- The helper then byte-compares the post-fix file with the original. If bytes
  differ, return an error whose text includes the fixture name and a clear
  message such as `formatter changed rendered output`.
- Do not use shell pipelines in Rust tests. Use `std::process::Command` with
  explicit arguments so paths are passed safely.

Tests:

- Unit test: a stable Markdown document beginning with `# Roadmap` and an
  already aligned table passes the helper.
- Unit test: an unstable Markdown document with an unaligned table returns an
  error containing `formatter changed rendered output`.
- No behavioural, property, snapshot, or end-to-end test is needed in this
  work item because it only introduces the reusable checker. The later work
  item wires it into the behavioural golden corpus.

Focused validation:

```bash
cargo test --test roadmap_golden gate_clean -- --nocapture 2>&1 | tee /tmp/test-mapsplice-roadmap-3-1-3-work-item-1.out
```

Commit after this work item only if the focused test passes and `make all`,
`make markdownlint`, and `make nixie` pass. If any required gate fails, do not
commit; record the failure in this plan and escalate for explicit human
approval before changing this requirement.

### Work item 2: Make renderer output formatter-stable

Purpose: change renderer output shape so the existing successful rendered
fixtures are accepted by the helper from work item 1.

Documentation and skills to read:

- `docs/mapsplice-design.md` section 5, especially F1, F3, F4, and F5.
- `docs/mapsplice-design.md` section 8, especially fixture EOF whitespace and
  required coverage.
- `docs/users-guide.md` Output modes.
- Skills: `leta`, `rust-router`, `rust-unit-testing`, `rust-errors`.

Edits:

- Use `leta show render_task`, `leta show render_sub_task`,
  `leta show render_markdown_nodes`, `leta show render_code_block`, and
  `leta show render_table` before editing. If Leta fails, record the exact
  failure in this plan and use bounded file reads of those symbols.
- Update `src/roadmap/render.rs` so task body blocks, addendum sub-tasks, and
  nested Markdown are emitted in the same shape produced by the verified
  formatter sequence:
  - nested task-list items under a roadmap task use the formatter-stable
    indentation proven by `mdtablefix`,
  - fenced code blocks inside list items are separated by blank lines in the
    shape the formatter leaves unchanged,
  - separate task-body paragraphs are rendered so the formatter does not join
    them unexpectedly.
- Update `src/roadmap/render_table.rs` only if the rendered table branch is
  responsible for unaligned table output. If table text is coming from
  preserved original blocks, prefer fixture or parser-preservation changes in
  this work item only when doing so does not weaken F1 for formatter-stable
  input.
- Do not change CLI output modes, parser acceptance rules, or public APIs.

Tests:

- Update existing golden expected fixtures only where the renderer output has
  intentionally changed to formatter-stable Markdown.
- Add or update focused unit tests in `tests/roadmap_render.rs` for:
  - nested addendum indentation under a task,
  - fenced code-block spacing inside a task body,
  - multi-paragraph task body rendering.
- Add no property test in this work item unless the render change touches a
  pure helper whose invariant can be generated cheaply. If it does, use
  `proptest` only for the pure helper and keep generated inputs valid.
- No snapshot or e2e test is needed beyond the golden fixture corpus because
  the changed behaviour is rendered Markdown bytes.

Focused validation:

```bash
cargo test --test roadmap_render -- --nocapture 2>&1 | tee /tmp/test-mapsplice-roadmap-3-1-3-render.out
cargo test --test roadmap_golden -- --nocapture 2>&1 | tee /tmp/test-mapsplice-roadmap-3-1-3-golden.out
```

Commit after this work item only if the focused tests pass and `make all`,
`make markdownlint`, and `make nixie` pass. If any required gate fails, do not
commit; record the failure in this plan and escalate for explicit human
approval before changing this requirement.

### Work item 3: Enforce gate-clean output for every successful golden case

Purpose: make F4 a permanent regression test across rendered fixture outputs.

Documentation and skills to read:

- `docs/roadmap.md` task 3.1.3.
- `docs/mapsplice-design.md` sections 5 and 8.
- `docs/developers-guide.md` section 6.
- Skills: `leta`, `rust-router`, `rust-unit-testing`, `rust-errors`.

Edits:

- In `tests/golden/assertions.rs`, call
  `assert_gate_clean_rendered_output` on every successful rendered output:
  - stdout for `SuccessOutput::Stdout`,
  - stdout for `SuccessOutput::StdoutTargetUnchanged`,
  - target contents for `SuccessOutput::InPlaceSuccess`,
  - stdout for `SuccessOutput::OriginalTargetStdout`.
- Do not run the gate helper for expected failure cases; F5 asserts no partial
  output and target preservation there.
- Add a metadata test in `tests/golden/metadata_tests.rs` documenting that
  direct linting of golden fixture paths is not sufficient because repository
  markdownlint ignores that tree. This can be a pure test over configuration
  text or a short helper test that asserts the gate helper uses a temp file
  outside `tests/fixtures/golden`.

Tests:

- Behavioural golden test: the entire `tests/roadmap_golden.rs` integration
  suite passes with the gate helper enabled.
- Unit tests from work item 1 continue to prove the helper catches unstable
  output.
- No additional property or snapshot test is required because this work item
  broadens an existing behavioural corpus assertion.

Focused validation:

```bash
cargo test --test roadmap_golden -- --nocapture 2>&1 | tee /tmp/test-mapsplice-roadmap-3-1-3-gate-corpus.out
```

Commit after this work item only if the focused test passes and `make all`,
`make markdownlint`, and `make nixie` pass. If any required gate fails, do not
commit; record the failure in this plan and escalate for explicit human
approval before changing this requirement.

### Work item 4: Update roadmap and final validation evidence

Purpose: mark the roadmap task complete and leave a clear validation trail.

Documentation and skills to read:

- `docs/roadmap.md` task 3.1.3.
- `docs/documentation-style-guide.md` Markdown conventions.
- `AGENTS.md` Markdown Guidance and Change Quality.
- Skills: `execplans`, `sem`.

Edits:

- Change `docs/roadmap.md` task 3.1.3 from unchecked to checked only after
  work items 1-3 pass.
- Update this ExecPlan `Progress`, `Surprises & Discoveries`, `Decision Log`,
  and `Outcomes & Retrospective` with exact evidence and any deviations.
- Use `sem diff --format json --file-exts .rs .md` before the final commit to
  review entity-level changes. If sem creates local cache files, remove or park
  them with a named discard stash before committing.

Tests:

- No new tests are added in this work item. Its purpose is documentation and
  validation evidence.

Focused validation:

```bash
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-3-1-3.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-3.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-3.out
```

Commit after this work item only if all validation commands pass. If any gate
failure appears to be unrelated pre-existing debt, document the exact file,
line, command, and log path in this plan, leave the change uncommitted, and
escalate for explicit human approval before changing the gate requirement.

## Concrete steps

1. Start from the assigned worktree:

```bash
cd /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-3
git branch --show-current
```

Expected branch:

```plaintext
roadmap-3-1-3
```

1. Before each code edit, inspect the symbols with Leta:

```bash
leta show render_roadmap
leta show render_task
leta show render_sub_task
leta show render_markdown_nodes
leta show render_code_block
leta show render_table
```

If a Leta command fails, append the exact failure to `Surprises & Discoveries`
and continue with bounded source reads of the same symbols.

1. For formatter research or test debugging, use temporary files only:

```bash
set -o pipefail
tmp=$(mktemp -d /tmp/mapsplice-render-gate-XXXXXX)
cp tests/fixtures/golden/f4_formatter_stability_smoke/expected.md "$tmp/rendered.md"
mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place "$tmp/rendered.md" \
  2>&1 | tee /tmp/mdtablefix-probe-mapsplice-roadmap-3-1-3.out
markdownlint-cli2 --fix "$tmp/rendered.md" \
  2>&1 | tee /tmp/markdownlint-fix-probe-mapsplice-roadmap-3-1-3.out
diff -u tests/fixtures/golden/f4_formatter_stability_smoke/expected.md "$tmp/rendered.md"
rm -rf "$tmp"
```

1. After each work item, format only changed Markdown files. The `ACMR` diff
filter excludes deleted files, so every formatter argument names a path that
still exists in the worktree. The formatter and fixer runs are tee-logged with
deterministic `/tmp` paths for this branch:

```bash
set -o pipefail
changed_md=/tmp/changed-md-mapsplice-roadmap-3-1-3.list
git diff --name-only --diff-filter=ACMR -z -- '*.md' > "$changed_md"
if [ -s "$changed_md" ]; then
  xargs -0 --no-run-if-empty mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place \
    < "$changed_md" 2>&1 | tee /tmp/mdtablefix-mapsplice-roadmap-3-1-3.out
  xargs -0 --no-run-if-empty markdownlint-cli2 --fix \
    < "$changed_md" 2>&1 | tee /tmp/markdownlint-fix-mapsplice-roadmap-3-1-3.out
fi
```

1. Run focused validation for the work item, then the full acceptance commands
before committing.

## Validation and acceptance

The final validation commands are:

```bash
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-3-1-3.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-3.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-3.out
```

`make all` includes `check-fmt`, `lint`, `typecheck`, and `test` on current
`origin/main` policy. `make markdownlint` and `make nixie` are required because
this plan and the roadmap are Markdown documents.

Acceptance criteria:

- The new render-output gate helper fails on an intentionally unstable rendered
  Markdown sample and passes on a stable sample.
- Every successful golden fixture checks actual stdout or in-place target bytes
  with the helper.
- Running `cargo test --test roadmap_golden` fails before renderer fixes for
  the expected formatter-stability reason and passes after the minimal renderer
  and fixture updates.
- Running the final validation commands above passes before every commit.
- `docs/roadmap.md` marks 3.1.3 complete only after the test and gate evidence
  exists.

Red-Green-Refactor evidence to record during implementation:

- Red: add or enable the gate assertion against existing rendered output and
  run `cargo test --test roadmap_golden`. The expected failure is a
  formatter-stability error for a named fixture, not an unrelated panic.
- Green: update the renderer and expected fixtures so the same command passes.
- Refactor: simplify helper names or renderer helpers only after the focused
  tests pass, then rerun `cargo test --test roadmap_golden` and `make all`.

## Idempotence and recovery

All formatter probes must operate on files under `/tmp` and can be rerun
safely. The render-output helper creates and deletes its own temporary
directory through `tempfile`.

If a path-scoped Markdown formatter command changes files outside the intended
work item, stop and inspect `git status --short`. If the churn is accidental
and was created by this implementation, park it with a named discard stash:

```bash
git stash push -u -m 'df12-stash v1 task=3.1.3 kind=discard reason="formatter churn"' -- <paths>
```

Do not use a bare `git stash`.

If final `make all`, `make markdownlint`, or `make nixie` fails on unrelated
pre-existing files, do not repair unrelated files in this task. Record the
exact command, log path, file, and line evidence in this plan and final output.
Do not commit while any required gate is failing unless a human explicitly
approves changing the gate requirement for this task.

## Artifacts and notes

Verified local source and command evidence:

- `Makefile` line 26 defines `make all` as `check-fmt lint typecheck test`.
- `Makefile` lines 59-63 define `make markdownlint` and `make nixie`.
- `.markdownlint-cli2.jsonc` lines 13-19 ignore
  `tests/fixtures/golden/**/*.md`.
- `/home/leynos/.local/bin/mdformat-all` lines 30-31 run the house formatter
  sequence globally.
- `markdownlint-cli2 --help` reported version `0.22.1` with
  `markdownlint 0.40.0` and documented that `--fix` updates files.
- `mdtablefix --help` reported the relevant flags:
  `--wrap`, `--renumber`, `--breaks`, `--ellipsis`, `--fences`, and
  `--in-place`.
- Direct
  `markdownlint-cli2 tests/fixtures/golden/f4_formatter_stability_smoke/expected.md`
  linted zero files because of the ignore rule. A copied temp file under
  `/tmp` linted one file.

Known initial formatter-probe failures to expect in the Red phase include:

- `tests/fixtures/golden/addendum_body_surface/expected.md`
- `tests/fixtures/golden/c2_contiguous_renumber/expected.md`
- `tests/fixtures/golden/c4_addendum_render_fidelity/expected.md`
- `tests/fixtures/golden/c4_addendum_renumber/expected.md`
- `tests/fixtures/golden/code_blocks_preserved/expected.md`
- `tests/fixtures/golden/f1_minimal_untouched_content/expected.md`
- `tests/fixtures/golden/f4_formatter_stability_smoke/expected.md`
- `tests/fixtures/golden/insert_sub_task_after/expected.md`
- `tests/fixtures/golden/multi_line_task_body/expected.md`
- `tests/fixtures/golden/preamble_preserved/expected.md`
- `tests/fixtures/golden/replace_sub_task/expected.md`
- `tests/fixtures/golden/tables_preserved/expected.md`

## Revision notes

- 2026-07-02: Initial planning round. Created the plan, pinned formatter and
  linter behaviour, recorded advisory-tool failures, and decomposed roadmap
  task 3.1.3 into four independently committable work items.
- 2026-07-02: Planning round 2. Resolved design-review blockers by removing
  permission to commit with documented pre-existing gate failures, replacing
  the changed-Markdown formatter recipe with an `ACMR` existence-safe file set,
  and tee-logging both path-scoped formatter commands to deterministic `/tmp`
  logs.
- 2026-07-02: Implementation work item 1. Added the rendered-output Markdown
  gate helper, recorded advisory-tool and review-tool failures, and captured
  focused and full deterministic gate evidence.
- 2026-07-02: Implementation work item 2. Updated renderer spacing and
  formatter-stable fixture bytes, recorded the transient `nixie` timeout, and
  captured focused and full deterministic gate evidence.
