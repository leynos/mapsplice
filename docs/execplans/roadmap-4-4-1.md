# Escape literal Markdown backslashes without losing text

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

Roadmap task 4.4.1 closes a renderer fidelity gap found by the post-4.2.2
audit: literal Markdown backslashes must survive a parse-render-parse cycle.
After this change, roadmap text such as a literal `\!` remains the same text
after rendering and reparsing, rather than being re-read as an escaped
exclamation mark with the backslash silently lost.

The observable success criteria are:

- focused renderer tests fail before the implementation for a literal
  backslash/exclamation case and pass after the minimal renderer change;
- a golden fixture containing literal backslash and exclamation-mark text is
  part of the round-trip corpus;
- rendered output remains stable under the house Markdown formatter and linter;
- `make all`, `make markdownlint`, and `make nixie` pass before the
  implementation branch is considered complete.

## Constraints

- Work exclusively in `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-1`.
  Never edit the root/control worktree.
- Preserve `docs/mapsplice-design.md` section 2: parsing remains mdast-based
  through the `markdown` crate, edits run against the roadmap intermediate
  representation, and there is no raw-string surgery.
- Preserve `docs/mapsplice-design.md` section 5, especially F1 content
  preservation, F3 round-trip stability, F4 gate-clean output, and F5 fail
  closed behaviour.
- Preserve `docs/mapsplice-design.md` section 8: every fixed defect must land
  with a fixture or test that fails before the fix and passes after it.
- Preserve `docs/users-guide.md` "The roadmap shape `mapsplice` expects" and
  "Output modes": the supported grammar and final-newline contract do not
  change.
- Preserve `docs/developers-guide.md` sections 2 and 6: the public workflow
  remains parse, apply, render, stdout-or-in-place, with verification layered
  through unit, behavioural, property, and golden tests.
- Do not add or upgrade dependencies. `Cargo.lock` currently resolves
  `markdown` to `1.0.0`, and this task must work with that locked version.
- Do not introduce a dependency on an external Markdown writer. The accepted
  initial plan in `docs/execplans/initial-tool.md` decided that the `markdown`
  crate is used for parsing/mdast access, while `mapsplice` owns a
  deterministic renderer for the supported roadmap subset.
- File-level size constraints still apply to Rust code: no single code file may
  exceed 400 lines. If adding tests would push a file over that limit, split
  the tests into a colocated module before implementation.
- Documentation prose, comments, and commit messages must use en-GB Oxford
  spelling.
- Do not run repo-global formatters for this task. Format only changed Markdown
  files with path-scoped commands, then run repository gates.
- Do not run formatters, lints, or tests in parallel. Use `tee` for long gate
  output under `/tmp`.

## Tolerances

- Scope: stop and escalate if the implementation needs production changes
  outside `src/roadmap/render_text.rs` and possibly local test-only imports in
  `src/roadmap/render_tests.rs`.
- Interface: stop and escalate if any public API signature, CLI syntax, or
  fixture harness public helper must change.
- Dependencies: stop and escalate if a new crate or a `markdown` version change
  appears necessary.
- Behaviour: stop and escalate if escaping a literal backslash or exclamation
  mark makes existing supported Markdown render as a different mdast structure.
- Tests: stop and escalate if the focused renderer/golden tests still fail
  after three implementation attempts.
- Test targeting: stop and correct the command if any focused Cargo command
  reports `running 0 tests`. A zero-test pass is not validation evidence.
- Formatter: stop and escalate if the house formatter changes the new rendered
  fixture after the renderer fix.
- Documentation: stop and escalate if implementation requires changing the
  documented roadmap grammar rather than preserving already accepted text.

## Risks

- Risk: escaping all punctuation could create unnecessary churn or alter
  formatter-stable input. Severity: medium. Likelihood: medium. Mitigation: the
  plan scopes the implementation to the two audited load-bearing omissions, `\`
  and `!`, and relies on existing tests for the broader escape set.
- Risk: an exclamation mark is not lossy by itself, so a weak test could pass
  without proving the image-syntax hazard. Severity: medium. Likelihood:
  medium. Mitigation: include an end-to-end case where a literal exclamation
  mark is adjacent to link syntax, so rendering `![...]` would be reparsed as
  an image or rejected by the supported renderer.
- Risk: golden fixtures under `tests/fixtures/golden/**/*.md` are ignored by
  repository markdownlint configuration. Severity: medium. Likelihood: high.
  Mitigation: use the existing golden render-output gate
  `tests/golden/format_gate.rs`, which writes rendered bytes to a temporary
  Markdown path and runs the real `mdtablefix` plus `markdownlint-cli2` tools.
- Risk: advisory search tools may be unavailable in an implementation session.
  Severity: low. Likelihood: medium. Mitigation: this plan records the exact
  failures and gives bounded local source and documentation paths that are
  sufficient to implement the task.

## Progress

- [x] (2026-07-03 00:00+02:00) Read `AGENTS.md`, confirmed the branch is
  `roadmap-4-4-1`, and confirmed the plan path is
  `docs/execplans/roadmap-4-4-1.md`.
- [x] (2026-07-03 05:43+02:00) Loaded the `execplans`, `leta`,
  `memtrace-first`, `rust-router`, `rust-unit-testing`, `rust-verification`,
  `proptest`, `rust-errors`, and `firecrawl-mcp` skills needed for planning and
  signposting.
- [x] (2026-07-03 00:00+02:00) Read the governing documents:
  `docs/roadmap.md`, `docs/mapsplice-design.md`, `docs/developers-guide.md`,
  `docs/users-guide.md`, `docs/contributing.md`,
  `docs/documentation-style-guide.md`, `docs/issues/audit-4.2.2.md`, and the
  accepted initial ExecPlan.
- [x] (2026-07-03 00:00+02:00) Verified the locked `markdown` version from
  `Cargo.lock` and local registry source.
- [x] (2026-07-03 06:18+02:00) Revised the plan after design-review round 3
  so each code-producing work item runs the full Rust commit gates before its
  commit.
- [x] (2026-07-03 06:23+02:00) Work item 1: fixed inline literal
  punctuation escaping. Red evidence:
  `/tmp/test-render-literal-escape-mapsplice-roadmap-4-4-1-red.out` reported 3
  failing literal tests. Green evidence:
  `/tmp/test-render-literal-escape-mapsplice-roadmap-4-4-1-green.out` passed 3
  tests, `/tmp/test-lib-mapsplice-roadmap-4-4-1.out` passed 39 library tests,
  and scrutineer reported `make all` green in
  `/tmp/all-mapsplice-roadmap-4-4-1-wi1-rerun.out`.
- [x] (2026-07-03 06:30+02:00) Work item 2: added golden round-trip and
  formatter-stability coverage. Red evidence:
  `/tmp/test-golden-literal-escape-mapsplice-roadmap-4-4-1-red.out` failed one
  focused golden test before the renderer escapes were restored. Green evidence:
  `/tmp/test-golden-literal-escape-mapsplice-roadmap-4-4-1-green.out` passed
  one focused golden test,
  `/tmp/test-roadmap-golden-mapsplice-roadmap-4-4-1.out` passed 64 golden tests,
  `/tmp/test-round-trip-fixture-list-mapsplice-roadmap-4-4-1.out` passed one
  required-surface test, and
  `/tmp/test-round-trip-fixtures-mapsplice-roadmap-4-4-1.out` passed the
  conformant round-trip corpus. Scrutineer then reported `make all`,
  `make markdownlint`, and `make nixie` green in the work-item 2 logs.
- [x] (2026-07-03 06:36+02:00) Work item 3: marked roadmap task 4.4.1
  complete and recorded final validation evidence. Scrutineer reported
  `make all`, `make markdownlint`, and `make nixie` green in
  `/tmp/all-mapsplice-roadmap-4-4-1-wi3.out`,
  `/tmp/markdownlint-mapsplice-roadmap-4-4-1-wi3.out`, and
  `/tmp/nixie-mapsplice-roadmap-4-4-1-wi3.out`.

## Surprises & discoveries

- Observation: Memtrace was unavailable in this planning session.
  Evidence: `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`. Impact: canonical main-branch graph context
  could not be gathered through Memtrace; this is recorded as advisory-tool
  unavailability, not a product blocker.
- Observation: Leta was available for branch-local verification in this
  planning round. Evidence:
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-1`
  returned either
  `Added workspace: /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-1` or
  `Workspace already added: /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-1`,
  and `leta show` located `escape_markdown`, `is_markdown_metacharacter`,
  `render_inline_node`,
  `noop_round_trip_property_holds_for_all_conformant_fixtures`, and
  `REQUIRED_ROUND_TRIP_SURFACES`. Impact: this revision verifies branch-local
  unit-test placement directly instead of relying on inferred test binaries.
- Observation: Firecrawl was unavailable for official docs scraping.
  Evidence: `mcp__firecrawl.firecrawl_scrape` for the docs.rs `markdown` 1.0.0
  `to_mdast` page returned `user cancelled MCP tool call`. Impact: load-bearing
  library behaviour is pinned to the locked crate source under the local Cargo
  registry, and implementers should retry docs.rs access if their session
  allows it. This does not block implementation because the exact locked source
  is available locally.
- Observation: the literal escaping tests named in work items 1 and 2 belong
  to the library crate, not the `tests/roadmap_render.rs` integration binary.
  Evidence: `src/roadmap/render.rs` mounts
  `#[path = "render_tests.rs"] mod render_tests;`; `leta show` located
  `noop_round_trip_property_holds_for_all_conformant_fixtures` and
  `REQUIRED_ROUND_TRIP_SURFACES` in `src/roadmap/render_tests.rs`, while
  `tests/roadmap_render.rs` is a separate integration target. Impact: focused
  commands for those tests must use `cargo test --lib <filter>`, otherwise
  Cargo can filter to zero tests and exit successfully.
- Observation: the required work-item 1 CodeRabbit pass could not reach the
  network from this sandbox. Evidence: scrutineer wrote
  `/tmp/roadmap-4-4-1-wi1-coderabbit.out`, whose output was:

  ```json
  {"type":"status","phase":"deferred","status":"deferred coderabbit review: no default network route visible in this sandbox"}
  ```

  Impact: there were no actionable CodeRabbit findings to address locally, but
  the AI review remains deferred for supervisor follow-up.
- Observation: the first work-item 2 fixture spelling was not gate-clean after
  rendered-output formatting. Evidence: the focused green test first failed
  with Markdown lint errors MD041 and MD059, then a formatter probe in
  `/tmp/formatter-diff-literal-escape-mapsplice-roadmap-4-4-1.out` showed that
  the longer descriptive link text wrapped across two lines. Impact: the final
  fixture includes an H1 and the shorter descriptive link text `example docs`,
  preserving the `![...]` image-syntax boundary while satisfying F4 gate-clean
  output.
- Observation: the required work-item 2 CodeRabbit pass could not reach the
  network from this sandbox. Evidence: scrutineer wrote
  `/tmp/roadmap-4-4-1-wi2-coderabbit.out`, whose output was:

  ```json
  {"type":"status","phase":"deferred","status":"deferred coderabbit review: no default network route visible in this sandbox"}
  ```

  Impact: there were no actionable CodeRabbit findings to address locally, but
  the AI review remains deferred for supervisor follow-up.
- Observation: the required work-item 3 CodeRabbit pass could not reach the
  network from this sandbox. Evidence: scrutineer wrote
  `/tmp/coderabbit-mapsplice-roadmap-4-4-1-wi3-coderabbit.out`, whose output
  was:

  ```json
  {"type":"status","phase":"deferred","status":"deferred coderabbit review: no default network route visible in this sandbox"}
  ```

  Impact: there were no actionable CodeRabbit findings to address locally, but
  the AI review remains deferred for supervisor follow-up.

## Decision log

- Decision: implement this task by extending the existing inline-text escaping
  helper, not by adding a Markdown writer or post-processing rendered strings.
  Rationale: `docs/mapsplice-design.md` section 2 requires mdast parsing and a
  roadmap model, while `docs/execplans/initial-tool.md` records the accepted
  decision to own a deterministic renderer for the supported roadmap subset.
  Date/Author: 2026-07-03 / planning agent.
- Decision: add `\` and `!` to the renderer's Markdown metacharacter predicate
  and leave broader leading punctuation handling out of this task. Rationale:
  roadmap task 4.4.1 specifically requires literal backslash and
  exclamation-mark coverage. The audit mentions leading `-` and `.` only as a
  consideration; no current source evidence shows text loss for those cases in
  this task's surface. Date/Author: 2026-07-03 / planning agent.
- Decision: prove the failure at two levels: a focused renderer unit test for
  `escape_markdown`, and an end-to-end golden fixture that exercises parse,
  render, reparse, and formatter stability. Rationale: a unit test isolates the
  minimal fix, while the golden fixture satisfies the design-document fixture
  discipline and prevents a later integration regression. Date/Author:
  2026-07-03 / planning agent.
- Decision: keep work item 1 production code limited to
  `is_markdown_metacharacter` and escape only literal backslash plus
  exclamation mark beyond the existing set. Rationale: the focused red tests
  demonstrated the audited text-loss and image-syntax hazards, and the green
  implementation required no parser, model, public API, or dependency changes.
  Date/Author: 2026-07-03 06:23+02:00 / implementation agent.
- Decision: use `\![example docs](https://example.com)` in the work-item 2
  golden fixture instead of the draft-plan placeholder
  `\![link](https://example.com)`. Rationale: `link` violates the repository
  Markdown lint rule for descriptive link text, while `example docs` still
  exercises the same exclamation-mark adjacency that would become image syntax
  if the renderer emitted `!` raw. Date/Author: 2026-07-03 06:30+02:00 /
  implementation agent.
- Decision: mark roadmap task 4.4.1 complete after the final deterministic
  gates passed, while carrying CodeRabbit as a deferred open review issue.
  Rationale: every local deterministic gate is green and all actionable local
  work is complete; the CodeRabbit agent could not start because the sandbox
  has no default network route, so no actionable review feedback exists to
  address in this worktree. Date/Author: 2026-07-03 06:36+02:00 /
  implementation agent.

## Outcomes & retrospective

Work item 1 is complete. `escape_markdown` now renders literal `\` and `!` with
explicit escapes so a text value such as `\!` survives parse, render, and
reparse without losing the backslash or becoming image syntax. The focused red
command ran 3 tests and failed for the intended reasons before the production
change. The green focused command passed 3 tests, the library suite passed 39
tests, and scrutineer reported `make all` green for
`/tmp/all-mapsplice-roadmap-4-4-1-wi1-rerun.out`.

Work item 2 is complete. The golden fixture
`tests/fixtures/golden/literal_backslash_escape/` now exercises the literal
backslash-plus-exclamation sequence and an escaped exclamation mark adjacent to
inline link syntax. It is included in both the golden operation suite and the
required conformant round-trip surface list. The focused red command failed for
the intended renderer difference, the green focused command passed, the full
golden suite passed, the fixture-list and round-trip corpus checks passed, and
scrutineer reported `make all`, `make markdownlint`, and `make nixie` green for
work item 2.

Work item 3 is complete. `docs/roadmap.md` now marks roadmap task 4.4.1 done.
The final deterministic gates passed through scrutineer with `make all`,
`make markdownlint`, and `make nixie` green. The only remaining issue is the
deferred CodeRabbit review caused by the sandbox lacking a default network
route.

## Context and orientation

The relevant roadmap task is `docs/roadmap.md` section 4.4.1, "Escape literal
Markdown backslashes without losing text." It requires literal backslash and
exclamation-mark cases in renderer round-trip coverage so text nodes cannot be
re-read with characters silently dropped.

The governing audit is `docs/issues/audit-4.2.2.md`, finding 4. It identifies
`src/roadmap/render_text.rs::escape_markdown` and
`src/roadmap/render_text.rs::is_markdown_metacharacter` as the relevant
implementation surface. The current helper prefixes a backslash before selected
Markdown metacharacters, but the current predicate omits `\` and `!`.

The renderer is in `src/roadmap/render.rs`. Inline text nodes reach
`escape_markdown` through `render_inline_node`, which matches
`Node::Text(text)` and calls `escape_markdown(&text.value)`. The parser is in
`src/roadmap/parse/mod.rs`, where `parse_markdown_root` uses
`markdown::to_mdast(markdown, &ParseOptions::gfm())`.

The locked dependency is `markdown` 1.0.0. `Cargo.lock` lines 942-949 identify
the package as `markdown` version `1.0.0`. The local locked source under
`/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src`
is the load-bearing dependency evidence:

- `src/lib.rs` lines 150-164 define
  `pub fn to_mdast(value: &str, options: &ParseOptions) -> Result<mdast::Node, message::Message>`.
- `src/mdast.rs` lines 827-842 define
  `Text { value: String, position: Option<Position> }`.
- `src/construct/character_escape.rs` lines 1-17 define character escapes as a
  backslash followed by ASCII punctuation.
- `src/construct/character_escape.rs` lines 73-84 accept ASCII punctuation
  ranges including `!` and `\`.

Because `\` and `!` are ASCII punctuation in the locked parser, a rendered text
sequence `\!` is interpreted as an escaped exclamation mark on reparse and
loses the literal backslash. Rendering the same text value as `\\\!` preserves
the literal backslash and the literal exclamation mark through
`markdown::to_mdast`.

## Plan of work

### Work item 1: Fix inline literal punctuation escaping

This work item is one atomic code-and-test commit. It implements the minimal
renderer fix and pins the behaviour with focused unit tests. It implements
`docs/roadmap.md` task 4.4.1, `docs/mapsplice-design.md` sections 2, 5, and 8,
`docs/developers-guide.md` sections 2 and 6, and `AGENTS.md` Rust testing and
error-handling guidance.

Read before editing:

- `AGENTS.md`
- `docs/mapsplice-design.md` sections 2, 5, and 8
- `docs/developers-guide.md` sections 2 and 6
- `docs/issues/audit-4.2.2.md` finding 4
- `src/roadmap/render_text.rs`
- `src/roadmap/render.rs`
- `src/roadmap/render_tests.rs`
- locked `markdown` 1.0.0 source files listed in "Context and orientation"

Skills to load:

- `leta` for branch-local symbol navigation, falling back to bounded local
  source inspection if the workspace cannot be added.
- `memtrace-first` plus Memtrace graph tools for canonical main-branch search,
  falling back to bounded local evidence if the MCP call is rejected or
  cancelled.
- `rust-router`, then `rust-unit-testing` for focused renderer tests and
  `rust-errors` only if a typed renderer error boundary unexpectedly changes.

Test-first steps:

1. Add a focused `rstest` parameterized unit test in
   `src/roadmap/render_tests.rs` for `super::text::escape_markdown`. Include at
   least these cases, and name the test with `literal` so the focused library
   command below cannot filter past it:
   - input text value `literal \! marker` renders as Markdown source
     `literal \\\! marker`;
   - input text value `bang ! before link` renders with `\!` so a later inline
     link cannot combine into image syntax.
2. Add an end-to-end parser/renderer unit test in the same file. The source
   should contain a roadmap task with escaped source spelling that parses to a
   text value containing a literal backslash immediately followed by a literal
   exclamation mark. Use canonical Markdown source spelling equivalent to:

   ```markdown
   - [ ] 1.1.1. Preserve \\\! marker and \![link](https://example.com).
   ```

   The assertion must parse, render, parse the rendered output again, render
   again, and assert the rendered bytes are stable.
3. Run the focused red command before production-code changes:

   ```bash
   cargo test --lib literal -- --nocapture \
     2>&1 | tee /tmp/test-render-literal-escape-mapsplice-roadmap-4-4-1-red.out
   ```

   The expected failure is an assertion showing that rendered Markdown contains
   an unescaped backslash/exclamation sequence or the second render drops the
   literal backslash. The command must run at least one test; `running 0 tests`
   is a targeting error, not a red pass.
4. In `src/roadmap/render_text.rs`, update
   `is_markdown_metacharacter` so it returns `true` for `\\` and `!`. Keep the
   change local to the predicate and do not redesign escaping.
5. Run the focused green command:

   ```bash
   cargo test --lib literal -- --nocapture \
     2>&1 | tee /tmp/test-render-literal-escape-mapsplice-roadmap-4-4-1-green.out
   ```

   Expected result: the new focused tests pass, and Cargo reports at least one
   test was run.
6. Run the local renderer suite:

   ```bash
   cargo test --lib -- --nocapture \
     2>&1 | tee /tmp/test-lib-mapsplice-roadmap-4-4-1.out
   ```

   Expected result: all library tests, including the renderer unit-test module,
   pass.
7. Run the full Rust commit gates for this code commit, sequentially:

   ```bash
   make check-fmt \
     2>&1 | tee /tmp/check-fmt-mapsplice-roadmap-4-4-1-wi1.out
   make lint \
     2>&1 | tee /tmp/lint-mapsplice-roadmap-4-4-1-wi1.out
   make typecheck \
     2>&1 | tee /tmp/typecheck-mapsplice-roadmap-4-4-1-wi1.out
   make test \
     2>&1 | tee /tmp/test-mapsplice-roadmap-4-4-1-wi1.out
   ```

   Expected result: all four commands exit successfully. These gates are not
   deferrable to work item 3; this work item changes production and test Rust
   code, so the commit must be format-clean, lint-clean, type-clean, and
   workspace-test-clean before it is made.
8. Commit only this work item after its focused gates and full Rust commit
   gates pass.

### Work item 2: Add golden round-trip and formatter-stability coverage

This work item is one atomic fixture-and-harness commit after work item 1 has
passed. It proves the behaviour through the golden corpus and the existing
house formatter gate. It implements `docs/roadmap.md` task 4.4.1,
`docs/mapsplice-design.md` sections 5 and 8, `docs/developers-guide.md` section
6, and `AGENTS.md` testing guidance.

Read before editing:

- `tests/roadmap_golden.rs`
- `tests/golden/format_gate.rs`
- `tests/golden/case.rs`
- `tests/fixtures/golden/`
- `src/roadmap/render_tests.rs`
- `docs/execplans/roadmap-3-1-3.md` for the rendered-output gate rationale
- `docs/execplans/roadmap-3-1-4.md` for the formatter-boundary normalization
  boundary

Skills to load:

- `rust-router`, then `rust-unit-testing` for golden harness integration.
- `rust-verification`; choose no additional deep verification tool unless a
  generated property is needed. `proptest` is already available in the project,
  but this task should not add a new generated strategy unless the named
  fixture exposes more than one concrete escape family.

Steps:

1. Add a new golden fixture directory such as
   `tests/fixtures/golden/literal_backslash_escape/`.
2. Add `target.md`, `fragment.md`, and `expected.md` for a concrete insert
   operation using `GoldenCommand::InsertAfter { anchor: "1.1.1" }`. Keep the
   literal text on the untouched first task, insert one ordinary second task,
   and avoid unrelated renumber churn beyond the inserted task. The untouched
   target and expected text must include this canonical Markdown source
   spelling:

   ```markdown
   - [ ] 1.1.1. Preserve \\\! marker and \![link](https://example.com).
   ```

   The fixture must include:

   - a literal backslash immediately followed by an exclamation mark in task
     text;
   - an exclamation mark adjacent to link syntax or a similar inline boundary
     that would become image syntax if `!` were emitted raw;
   - otherwise ordinary, gate-clean roadmap Markdown.
3. Add a named test in `tests/roadmap_golden.rs` that calls
   `golden_success_case` or `golden_success_output_case` for the new fixture.
   The test name should include `literal_backslash` or `literal_escape` so the
   focused command can filter it.
4. Add the new fixture target path to
   `src/roadmap/render_tests.rs::REQUIRED_ROUND_TRIP_SURFACES` so the no-op
   round-trip fixture list must continue to include this defect class.
5. Run the focused red command by temporarily reverting the production change
   from work item 1 or by applying the test before the fix when implementing
   both items in a single local red-green sequence:

   ```bash
   cargo test --test roadmap_golden literal_backslash -- --nocapture \
     2>&1 | tee /tmp/test-golden-literal-escape-mapsplice-roadmap-4-4-1-red.out
   ```

   Expected failure before the renderer fix: the golden actual output or
   formatter gate differs because the literal backslash is not preserved.
6. With the work item 1 fix present, run:

   ```bash
   cargo test --test roadmap_golden literal_backslash -- --nocapture \
     2>&1 | tee /tmp/test-golden-literal-escape-mapsplice-roadmap-4-4-1-green.out
   cargo test --test roadmap_golden -- --nocapture \
     2>&1 | tee /tmp/test-roadmap-golden-mapsplice-roadmap-4-4-1.out
   cargo test --lib round_trip_fixture_list_covers_required_surfaces -- --nocapture \
     2>&1 | tee /tmp/test-round-trip-fixture-list-mapsplice-roadmap-4-4-1.out
   cargo test --lib noop_round_trip_property_holds_for_all_conformant_fixtures -- --nocapture \
     2>&1 | tee /tmp/test-round-trip-fixtures-mapsplice-roadmap-4-4-1.out
   ```

   Expected result: the new golden test, the full golden suite, and the
   conformant fixture list and round-trip property pass. The two fixture-list
   commands use `--lib` because those tests are compiled into the library crate
   from `src/roadmap/render_tests.rs`, not into the `roadmap_render`
   integration test binary. Each focused command must report at least one test;
   `running 0 tests` is a targeting error.
7. Run the full Rust commit gates for this fixture-and-test commit,
   sequentially:

   ```bash
   make check-fmt \
     2>&1 | tee /tmp/check-fmt-mapsplice-roadmap-4-4-1-wi2.out
   make lint \
     2>&1 | tee /tmp/lint-mapsplice-roadmap-4-4-1-wi2.out
   make typecheck \
     2>&1 | tee /tmp/typecheck-mapsplice-roadmap-4-4-1-wi2.out
   make test \
     2>&1 | tee /tmp/test-mapsplice-roadmap-4-4-1-wi2.out
   ```

   Expected result: all four commands exit successfully. These gates are not
   deferrable to work item 3; this work item changes Rust test code and
   Markdown fixture inputs, so the commit must be format-clean, lint-clean,
   type-clean, and workspace-test-clean before it is made.
8. Commit only this work item after its focused gates and full Rust commit
   gates pass.

### Work item 3: Record completion and gate evidence

This work item is one atomic documentation commit after work items 1 and 2 have
passed. It marks the roadmap task complete and records final validation in this
ExecPlan. It implements `docs/roadmap.md` task tracking, `AGENTS.md`
documentation maintenance, and `docs/documentation-style-guide.md` Markdown
rules.

Read before editing:

- `docs/roadmap.md` section 4.4.1
- this ExecPlan
- `docs/developers-guide.md` section 7
- `docs/documentation-style-guide.md`

Skills to load:

- `execplans` for living-plan updates.
- `changelog` only if repository convention or reviewer instruction asks for a
  changelog entry; no changelog update is expected for this internal roadmap
  task.

Steps:

1. Change `docs/roadmap.md` task 4.4.1 from `[ ]` to `[x]` only after the full
   validation commands below have passed.
2. Update this ExecPlan's `Progress`, `Surprises & Discoveries`,
   `Decision Log`, and `Outcomes & Retrospective` with the actual focused and
   repository gate evidence.
3. Format only changed Markdown files with path-scoped commands. If both
   `docs/roadmap.md` and this plan changed, run:

   ```bash
   mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place \
     docs/roadmap.md docs/execplans/roadmap-4-4-1.md \
     2>&1 | tee /tmp/mdtablefix-mapsplice-roadmap-4-4-1-docs.out
   markdownlint-cli2 --fix --no-globs -- \
     docs/roadmap.md docs/execplans/roadmap-4-4-1.md \
     2>&1 | tee /tmp/markdownfmt-mapsplice-roadmap-4-4-1-docs.out
   ```

   If only this plan changed, run:

   ```bash
   mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place \
     docs/execplans/roadmap-4-4-1.md \
     2>&1 | tee /tmp/mdtablefix-mapsplice-roadmap-4-4-1-plan.out
   markdownlint-cli2 --fix --no-globs -- docs/execplans/roadmap-4-4-1.md \
     2>&1 | tee /tmp/markdownfmt-mapsplice-roadmap-4-4-1-plan.out
   ```

4. Run the final gates, sequentially:

   ```bash
   make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-4-4-1.out
   make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-4-4-1.out
   make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-4-4-1.out
   ```

   Expected result: all commands exit successfully. `make all` includes
   `check-fmt`, `lint`, `typecheck`, and `test` on current `origin/main`.
5. Commit the documentation update only after the formatter and final gates
   pass.

## Concrete steps

All commands run from:

```bash
cd /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-1
```

Before starting implementation, re-check branch and worktree cleanliness:

```bash
git branch --show-current
git status --short
```

Expected branch:

```plaintext
roadmap-4-4-1
```

If `git status --short` shows unrelated changes, do not revert them. Work
around them if unrelated, or stop only if they directly block this task.

Use Red-Green-Refactor inside work items 1 and 2. Red failures are temporary
local evidence and should not be committed as failing states. Each committed
code or test work item must pass its focused gates and the full Rust commit
gates (`make check-fmt`, `make lint`, `make typecheck`, and `make test`) before
the commit is made. Do not defer rustfmt, Clippy, typecheck, or workspace-test
failures from work item 1 or work item 2 to work item 3.

After every work item commit, run at least the focused gates listed for that
work item if follow-up inspection is needed. After the final documentation
update, run the full validation gates listed in work item 3.

## Validation and acceptance

Red evidence:

- `cargo test --lib literal -- --nocapture` fails before the
  renderer fix because literal backslash/exclamation text is not preserved.
- `cargo test --test roadmap_golden literal_backslash -- --nocapture` fails
  before the renderer fix if the golden fixture is introduced first.

Green evidence:

- `cargo test --lib literal -- --nocapture` passes after adding `\` and `!` to
  `is_markdown_metacharacter`.
- Work item 1 then passes `make check-fmt`, `make lint`, `make typecheck`, and
  `make test` before its production-code commit.
- `cargo test --test roadmap_golden literal_backslash -- --nocapture` passes
  after the renderer fix.
- `cargo test --test roadmap_golden -- --nocapture` passes.
- `cargo test --lib round_trip_fixture_list_covers_required_surfaces -- --nocapture`
  passes and proves the required-surface list includes the new
  literal-backslash fixture path.
- The library test
  `noop_round_trip_property_holds_for_all_conformant_fixtures` passes with the
  command listed in work item 2 and exercises the conformant round-trip corpus,
  including the new literal-backslash fixture path.
- Work item 2 then passes `make check-fmt`, `make lint`, `make typecheck`, and
  `make test` before its fixture-and-test commit.

Final repository gates:

```bash
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-4-4-1.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-4-4-1.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-4-4-1.out
```

Acceptance is met when:

- rendered roadmap text containing literal `\` and `!` round-trips through
  `parse_roadmap` and `render_roadmap` without losing either character;
- an exclamation mark adjacent to inline link syntax cannot accidentally create
  image syntax on reparse;
- the new golden fixture is part of the conformant round-trip fixture corpus;
- the full final gates pass.

No snapshot update is expected. No end-to-end CLI scenario is required beyond
the golden fixture because the existing golden harness drives the compiled CLI
workflow and checks stdout or in-place output.

## Idempotence and recovery

The focused tests and repository gates are safe to rerun. Temporary files are
written under `/tmp` or test-owned temporary directories.

If a formatter command changes files outside the named documentation paths, do
not commit that churn. Revert only the unrelated formatter changes, or park
them with a named stash:

```bash
git stash push -m 'df12-stash v1 task=roadmap-4-4-1 kind=discard reason="unrelated formatter churn"' -- <paths>
```

If a focused gate fails, read the corresponding `/tmp/...roadmap-4-4-1...out`
log before rerunning. Rerun only after applying a fix.

## Artefacts and notes

Planning evidence gathered in this worktree:

```plaintext
git branch --show-current
roadmap-4-4-1

leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-1
Workspace already added: /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-1

leta show noop_round_trip_property_holds_for_all_conformant_fixtures
src/roadmap/render_tests.rs:93-129

mcp__memtrace.list_indexed_repositories
user cancelled MCP tool call

mcp__firecrawl.firecrawl_scrape https://docs.rs/markdown/1.0.0/markdown/fn.to_mdast.html
user cancelled MCP tool call
```

Locked dependency evidence:

```plaintext
Cargo.lock:943 name = "markdown"
Cargo.lock:944 version = "1.0.0"
Cargo.lock:946 checksum = "a5cab8f2cadc416a82d2e783a1946388b31654d391d1c7d92cc1f03e295b1deb"
```

Current renderer evidence:

```plaintext
src/roadmap/render_text.rs:17 escape_markdown(value: &str) -> String
src/roadmap/render_text.rs:74 is_markdown_metacharacter(character: char) -> bool
src/roadmap/render.rs:308 Node::Text(text) => Ok(escape_markdown(&text.value))
```

## Interfaces and dependencies

The production interface should remain:

```rust
pub(super) fn escape_markdown(value: &str) -> String
```

The implementation should remain:

```rust
const fn is_markdown_metacharacter(character: char) -> bool
```

At the end of work item 1, `is_markdown_metacharacter` must treat `\\` and `!`
as Markdown metacharacters. Do not change `render_inline_node`,
`render_roadmap`, public parser APIs, CLI APIs, or `MarkdownNodes`.

No dependency changes are permitted. The locked `markdown` 1.0.0 parser is the
oracle for reparse behaviour; `mapsplice` remains responsible for rendering the
supported roadmap subset.

## Revision note

2026-07-03 05:43+02:00: revised the draft after design-review round 2. The
focused commands for `src/roadmap/render_tests.rs` now use
`cargo test --lib <filter>` instead of the `roadmap_render` integration binary,
and work item 2 now separately runs
`round_trip_fixture_list_covers_required_surfaces` and
`noop_round_trip_property_holds_for_all_conformant_fixtures` through the
library crate. The Markdown formatting instructions were also narrowed to the
direct path-scoped `mdtablefix` then `markdownlint-cli2 --fix` sequence for
only the changed files. The remaining work is unchanged: implement red tests,
make the minimal renderer fix, add fixture coverage, then run the final
repository gates.

2026-07-03 06:18+02:00: revised the draft after design-review round 3. Work
items 1 and 2 now each run `make check-fmt`, `make lint`, `make typecheck`, and
`make test` before their commits, after their focused red/green evidence. This
resolves the intermediate-gating gap: rustfmt, Clippy, typecheck, and workspace
test failures introduced by either atomic code/test commit must surface before
that commit is made rather than being deferred to the final documentation work
item.

2026-07-03 06:23+02:00: updated the plan during work-item 1 implementation. The
status is now `IN PROGRESS`; work item 1 records its red, green, library, and
`make all` evidence; the living sections record the scoped renderer decision
and the deferred CodeRabbit review caused by the sandbox lacking a default
network route. Work item 2 remains next and unchanged.

2026-07-03 06:30+02:00: updated the plan during work-item 2 implementation.
Work item 2 now records focused red/green golden evidence, full golden-suite
evidence, fixture-list and round-trip corpus evidence, and the green scrutineer
gate logs. The living sections also record the gate-clean fixture wording
adjustment and the second deferred CodeRabbit review caused by the sandbox
lacking a default network route. Work item 3 remains next.

2026-07-03 06:36+02:00: updated the plan during work-item 3 implementation. The
status is now `COMPLETE`; work item 3 records the roadmap checkbox update and
final green `make all`, `make markdownlint`, and `make nixie` logs. The living
sections record the third deferred CodeRabbit review caused by the sandbox
lacking a default network route. No implementation work remains beyond external
review follow-up if network access becomes available.
