# Assemble grammar-surface and per-contract golden fixtures

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: IN PROGRESS

## Purpose / big picture

Roadmap task 3.1.1 is complete when `mapsplice` has a committed golden-fixture
corpus that proves the supported roadmap grammar, each structural operation,
and each fidelity or contract guarantee that can be expressed with a
deterministic example. A maintainer should be able to inspect the fixture
directories, see the exact command each case runs, and run the focused golden
test suite to compare Markdown bytes or typed failure outcomes exactly.

This plan does not implement roadmap task 3.1.2's generated no-op property test
or roadmap task 3.1.3's rendered-output Markdown stability sweep. It does
prepare the fixture layout so those later tasks can enumerate conformant
fixtures without redesigning the harness.

## Constraints

- Work only in
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1`.
- Do not edit the root/control worktree at `/home/leynos/Projects/mapsplice`.
- Treat `origin/main` as canonical and the integration branch as `main`.
- Use Memtrace as the primary canonical-main search and graph tool when it is
  available. If it is cancelled or unavailable, record the exact failure and
  continue with bounded branch-local evidence; do not mark this plan blocked
  solely because Memtrace is unavailable.
- Use `leta` for branch-local symbol navigation when it is available. If its
  daemon or workspace tooling fails, record the exact failure and continue with
  precise local file inspection.
- Use `sem` for entity-level history and diff inspection instead of raw
  `git log` or `git blame`.
- Follow the source-of-truth documents named in `Context and orientation`,
  especially `docs/mapsplice-design.md`, `docs/developers-guide.md`,
  `docs/users-guide.md`, `docs/roadmap.md`, and `AGENTS.md`.
- Keep prose, comments, and commit messages in en-GB Oxford spelling.
- Do not add a new external dependency for this task. If implementation
  appears to require one, stop and revise this plan with locked-source and
  official-documentation evidence.
- Do not redesign the grammar, operation semantics, dependency-reference
  model, command-line interface, or public library API unless a red fixture
  exposes a real defect. If that happens, keep the defect fix in the same
  atomic work item as the fixture that proves it.
- Fixture files are committed test inputs, not generated artefacts.
- Format only files changed by the current work item. Use path-specific
  `mdtablefix ... <files>` followed by `markdownlint-cli2 --fix <files>` for
  Markdown touched in that item. Do not run `make fmt` or `mdformat-all` for
  this task.
- Run tests, lints, and gates sequentially. Every command that may produce long
  output must be logged through `tee` to a branch-specific `/tmp` file, with
  `set -o pipefail` before pipelines.
- Commit after each implemented work item, and gate each commit before moving
  on.

## Tolerances

- If `git branch --show-current` is not `roadmap-3-1-1`, stop before editing.
- If a work item needs a public API signature change, stop and revise this plan
  before editing that API.
- If a work item needs a new crate, stop and revise this plan with locked
  source and official-documentation evidence for that crate.
- If a focused test or repository gate still fails after two focused fix
  attempts, record the command, log path, and error in `Decision Log`, then
  stop for review.
- If a work item would touch more than six non-fixture Rust source files, split
  the work item before committing.
- If formatter churn touches files outside the current work item, park or
  discard it with a named stash:

  ```bash
  git stash push -m 'df12-stash v1 task=3.1.1 kind=discard reason="formatter churn"'
  ```

- If Memtrace, Firecrawl, `leta`, or another advisory tool is unavailable, do
  not mark this plan blocked. Record the exact failed command or tool result in
  `Surprises & Discoveries` and continue with bounded local evidence.

## Risks

- Risk: Adding every required fixture in one commit could create an
  unreviewable diff. Severity: medium. Likelihood: high. Mitigation: split the
  corpus into independently committable work items for harness bytes,
  successful operations, grammar-surface preservation, adversarial contracts,
  output modes, and roadmap completion.

- Risk: A table-driven harness can hide which fixture failed.
  Severity: medium. Likelihood: medium. Mitigation: give every case a stable
  Rust test name and include the case name, command shape, expected output, and
  actual output in assertion failures.

- Risk: The current golden harness normalizes expected output by stripping one
  final newline, so it cannot prove byte identity. Severity: high. Likelihood:
  verified. Mitigation: first establish raw fixture-byte comparison and the
  renderer's canonical final-newline contract before adding identity or
  output-mode fixtures.

- Risk: Some design guarantees are fail-closed cases rather than successful
  output cases. Severity: medium. Likelihood: high. Mitigation: represent those
  cases with typed-error expectations, empty stdout, and unchanged target
  assertions rather than inventing successful output.

- Risk: Later roadmap task 3.1.2 may need to enumerate all conformant fixtures.
  Severity: medium. Likelihood: medium. Mitigation: store successful golden
  cases under `tests/fixtures/golden/<case-name>/` with `target.md`, optional
  `fragment.md`, and `expected.md`, and keep failure cases explicitly marked in
  Rust metadata.

## Progress

- [x] (2026-07-02T00:00:00Z) Confirmed the assigned worktree and branch:
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1` on `roadmap-3-1-1`.
- [x] (2026-07-02T00:00:00Z) Loaded the required planning and navigation
  skills: `execplans`, `leta`, `sem`, and `firecrawl-mcp`.
- [x] (2026-07-02T00:00:00Z) Loaded the Rust planning skills needed for this
  fixture/testing task: `rust-router`, `rust-unit-testing`, `rust-verification`,
  `proptest`, `rust-errors`, and `domain-cli-and-daemons`.
- [x] (2026-07-02T00:00:00Z) Read the source-of-truth documents listed in
  `Context and orientation`.
- [x] (2026-07-02T00:00:00Z) Verified
  `sem diff --from origin/main --to HEAD --format json` reported no semantic
  branch delta before this planning edit.
- [x] (2026-07-02T00:00:00Z) Verified locked local source for `markdown`
  1.0.0, `rstest` 0.26.1, `proptest` 1.11.0, and `insta` 1.48.0.
- [x] (2026-07-02T00:00:00Z) Revised this plan for planning round 2:
  work item 4 now keeps `c3_requires_list_rewrite` and `c3_substring_non_match`
  in the existing `tests/fixtures/reference_rewrite/` corpus, and the formatter
  command lists no longer name new `tests/fixtures/golden/` paths for those
  cases.
- [x] (2026-07-02T00:20:00Z) Revised this plan for planning round 3:
  work item 1 now requires splitting `tests/golden/mod.rs` before adding
  golden-harness metadata, and work item 5 now requires a compiled-binary BDD
  scenario for missing-anchor `--in-place` stdout and target-preservation
  semantics.
- [x] (2026-07-02T00:26:11Z) Work item 1: Established raw-byte golden
  comparisons and the renderer newline contract. Split the golden harness into
  `case`, `workspace`, `runner`, and `assertions` modules; removed expected
  output newline stripping; added the `OriginalTargetStdout` expectation shape;
  and documented the canonical one-final-newline renderer normalization.
- [x] (2026-07-02T01:53:00Z) Work item 1 review follow-up: host-side
  CodeRabbit review completed after the sandboxed task-agent invocation hung
  at `connecting_to_review_service`. Addressed the critical typed-error
  finding and subsequent low-severity harness clarity findings; final local
  gates passed.
- [ ] Work item 2: Add successful operation golden fixtures.
- [ ] Work item 3: Add grammar-surface preservation fixtures.
- [ ] Work item 4: Add fidelity and dependency-contract fixtures.
- [ ] Work item 5: Add output-mode and fail-closed golden fixtures.
- [ ] Work item 6: Mark roadmap 3.1.1 complete after gates.

## Surprises & discoveries

- Memtrace `list_indexed_repositories` returned
  `user cancelled MCP tool call`. Canonical-main graph context was unavailable,
  so this plan uses bounded branch-local evidence from documentation, source
  inspection, `cargo tree`, and `sem`. This is not a blocker.
- `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1`
  reported that the
  workspace was already added, and `leta files` succeeded. Branch-local symbol
  navigation is available for implementation; exact local source inspection in
  this planning round was limited to known documentation and test-harness paths.
- Firecrawl `firecrawl_scrape` calls for the official docs.rs pages for
  `markdown::to_mdast`, `markdown::ParseOptions::gfm`, `rstest::fixture`, and
  `insta::assert_snapshot` returned `user cancelled MCP tool call`. Official
  web documentation was unavailable, so this plan does not depend on unverified
  web-only claims; load-bearing library behaviour is pinned to locked local
  source.
- `cargo tree -i` shows caret resolution to locked versions that differ from
  some manifest minimums: `markdown v1.0.0`, `rstest v0.26.1`,
  `proptest v1.11.0`, and `insta v1.48.0`.
- The existing committed corpus under `tests/fixtures/reference_rewrite/`
  covers section-reference preservation, version/quantity preservation,
  section-reference-outside-`Requires`, substring non-match, and multi-id
  `Requires` lists. It does not yet cover the complete operation,
  grammar-surface, addendum, output-mode, and fail-closed corpus required by
  roadmap task 3.1.1.
- `tests/golden/mod.rs` is already 358 lines. AGENTS.md caps every code file
  at 400 lines, so work item 1 must split the golden harness before adding raw
  byte, identity, or typed-error metadata.
- `src/lib.rs::run_from_args` returns `Err(MapspliceError)` on failure and
  therefore cannot observe process stdout. Existing BDD coverage proves
  stdout-empty behaviour for dangling dependency `--in-place` failure, but the
  current missing-anchor BDD scenario is not in-place and does not assert
  stdout or target preservation. Work item 5 must add compiled-binary coverage
  for that CLI contract.
- During implementation, a fresh Memtrace
  `mcp__memtrace.list_indexed_repositories` call returned
  `user cancelled MCP tool call`. This confirms canonical-main graph context
  remained unavailable and branch-local evidence remained necessary.
- During implementation, `leta show render_roadmap` and a follow-up
  `leta grep` both returned `Error: Failed to start daemon` after earlier Leta
  searches had succeeded. Local inspection was used only for known files named
  in this plan.
- The requested `scrutineer` sub-agent gate delegation failed before running
  commands with:
  `You've hit your usage limit for GPT-5.3-Codex-Spark. Switch to another
  model now, or try again at Jul 7th, 2026 12:20 PM.`
  Deterministic gates were run locally instead and logged to the planned
  `/tmp` paths.
- `coderabbit review --agent` was invoked for work item 1 after deterministic
  gates passed. It emitted review context and `connecting_to_review_service`,
  then produced no further output for roughly 20 minutes. The local process was
  interrupted with exit 130 and the deferred review is tracked as an open issue.

## Decision log

- Decision: Use the existing `tests/golden/mod.rs` harness and extend it
  through explicit case metadata rather than replacing it. Rationale: The
  current harness already drives `run_from_args`, supports command shapes, and
  has typed success/failure expectations. Replacing it would add risk before
  the missing corpus exists. Date/Author: 2026-07-02 / Codex.

- Decision: Store new successful cases under
  `tests/fixtures/golden/<case-name>/` with `target.md`, optional
  `fragment.md`, and `expected.md`. Rationale: This shape keeps input and
  expected Markdown reviewable and gives later property work a clear corpus to
  enumerate. Date/Author: 2026-07-02 / Codex.

- Decision: Use exact Markdown files, not new `insta` snapshots, for roadmap
  task 3.1.1. Rationale: `docs/mapsplice-design.md` section 8 requires
  committed input-and-expected Markdown pairs compared exactly. `insta` remains
  appropriate for existing CLI help snapshots, but not for this corpus.
  Date/Author: 2026-07-02 / Codex.

- Decision: Establish raw-byte expected comparison and a canonical
  final-newline renderer contract before adding F3/C5 and C6 fixtures.
  Rationale: `tests/golden/mod.rs::expected_output` currently strips one final
  newline before comparison, while `src/roadmap/render.rs::render_roadmap`
  returns `blocks.join("\n\n")` without a final newline. Raw identity and
  formatter-stable Markdown cannot be proved until this is fixed. Date/Author:
  2026-07-02 / Codex.

- Decision: Cover F3/C5 in this task with an exact identity replacement
  fixture, then leave generated no-op coverage to roadmap task 3.1.2.
  Rationale: There is no public no-op CLI command. Replacing an item with
  byte-identical content exercises the parse, operation, renumber, rewrite, and
  render pipeline and must produce byte-identical output. Date/Author:
  2026-07-02 / Codex.

- Decision: Split `tests/golden/mod.rs` before extending the golden harness.
  Rationale: The file is 358 lines, while AGENTS.md caps code files at 400
  lines. Raw-byte comparison, identity output, and richer typed failure
  metadata would otherwise risk violating the file-size rule during the first
  implementation commit. Date/Author: 2026-07-02 / Codex.

- Decision: Prove missing-anchor `--in-place` stdout behaviour through a
  compiled-binary BDD scenario, not through the golden harness alone.
  Rationale: The golden harness uses `run_from_args`, which returns
  `Err(MapspliceError)` and exposes no process stdout on failure. The BDD
  harness executes `CARGO_BIN_EXE_mapsplice`, so it can assert that the CLI
  emits no roadmap body and leaves the target unchanged. Date/Author:
  2026-07-02 / Codex.

## Outcomes & retrospective

Work item 1 is implemented. `tests/golden/mod.rs` is now a 33-line module hub,
and the moved harness modules are all below 150 lines. The golden harness
compares raw fixture bytes, so later F3/C5 and C6 fixtures can prove identity
without hidden newline normalization. The renderer now normalizes every
non-empty rendered roadmap to exactly one final newline, and
`docs/mapsplice-design.md` plus `docs/users-guide.md` describe that contract.

Focused evidence for work item 1:

- `cargo test --workspace --all-targets --all-features --test roadmap_golden`
  passed and logged to `/tmp/test-mapsplice-roadmap-3-1-1-raw-bytes.out`.
- `cargo test --workspace --all-targets --all-features roadmap::render`
  passed and logged to `/tmp/test-mapsplice-roadmap-3-1-1-render-newline.out`.
- `wc -l tests/golden/mod.rs tests/golden/case.rs
  tests/golden/workspace.rs tests/golden/runner.rs
  tests/golden/assertions.rs tests/golden/metadata_tests.rs`
  logged line counts to
  `/tmp/wc-mapsplice-roadmap-3-1-1-golden-split.out`.
- `make all`, `make markdownlint`, and `make nixie` passed and logged to
  `/tmp/all-mapsplice-roadmap-3-1-1-raw-bytes.out`,
  `/tmp/markdownlint-mapsplice-roadmap-3-1-1-raw-bytes.out`, and
  `/tmp/nixie-mapsplice-roadmap-3-1-1-raw-bytes.out`.

Review recovery evidence for work item 1:

- Host-side `coderabbit review --agent --type uncommitted` completed and logged
  to `/tmp/coderabbit-mapsplice-roadmap-3-1-1-recovery-preserve.out`; the
  critical typed-error finding was fixed.
- Follow-up host-side CodeRabbit reviews logged to
  `/tmp/coderabbit-mapsplice-roadmap-3-1-1-review-fixes.out` and
  `/tmp/coderabbit-mapsplice-roadmap-3-1-1-final-preserve.out`; the remaining
  low-severity harness clarity findings were fixed.
- Final focused validation passed and logged to
  `/tmp/test-mapsplice-roadmap-3-1-1-final-preserve-golden.out`.
- Final gates passed and logged to
  `/tmp/all-mapsplice-roadmap-3-1-1-final-preserve.out`,
  `/tmp/markdownlint-mapsplice-roadmap-3-1-1-final-preserve.out`, and
  `/tmp/nixie-mapsplice-roadmap-3-1-1-final-preserve.out`.

## Context and orientation

`mapsplice` edits constrained roadmap-shaped Markdown by parsing it into a
roadmap model, applying one operation, renumbering affected items, rewriting
dependency references, and rendering Markdown. The accepted grammar is
normative in `docs/users-guide.md` section "The roadmap shape `mapsplice`
expects" and summarized in `docs/mapsplice-design.md` section 4.

Read these source-of-truth documents before implementation:

- `AGENTS.md`: quality gates, Rust rules, testing rules, Markdown rules, and
  commit requirements.
- `docs/roadmap.md`: roadmap task 3.1.1 and its dependencies on 1.1.3 and
  2.1.3.
- `docs/mapsplice-design.md`: sections 4 through 8 for the grammar, fidelity
  guarantees F1-F5, contract guarantees C1-C6, dependency-reference model, and
  fixture requirements.
- `docs/users-guide.md`: command overview, operation details, output modes,
  and validation/failure cases.
- `docs/developers-guide.md`: sections 2, 3, 6, and 7 for architecture
  boundaries, public APIs, verification layers, and local gates.
- `docs/documentation-style-guide.md`: spelling, Markdown rules, and
  formatting.
- `docs/scripting-standards.md`: shell discipline for validation snippets.
- `docs/execplans/initial-tool.md`: initial grammar, parser, operation,
  renumbering, and renderer decisions.

The implementation surfaces relevant to this task are:

- `tests/roadmap_golden.rs`, the golden test binary currently registering
  reference-rewrite cases.
- `tests/golden/mod.rs`, the shared fixture harness and private case metadata.
- `tests/fixtures/reference_rewrite/`, the existing adversarial reference
  fixtures from roadmap task 1.1.3.
- `src/lib.rs::run_from_args` and `src/lib.rs::run_request`, the public
  workflow entry points used by integration tests.
- `src/fs.rs::rewrite_utf8`, the in-place write path that must preserve
  fail-closed semantics.
- `src/error.rs::MapspliceError`, the typed diagnostic surface for failure
  fixtures.
- `src/roadmap/ops/mod.rs::RoadmapOperation`, the domain operation surface.
- `src/roadmap/parse/mod.rs::parse_root`, which uses
  `markdown::to_mdast(markdown, &ParseOptions::gfm())`.
- `src/roadmap/render.rs`, `src/roadmap/render_table.rs`, and
  `src/roadmap/render_text.rs`, the deterministic renderer paths that must
  preserve bodies, nested lists, tables, code blocks, and addendum sub-tasks.

## Research evidence

Memtrace and Firecrawl were attempted first, but both were cancelled by the
host session as recorded above. The following branch-local and locked-source
evidence is sufficient for an implementer to avoid unverified mechanisms.

- `cargo tree -i markdown`, `cargo tree -i rstest`, `cargo tree -i proptest`,
  and `cargo tree -i insta` show the locked versions used by this work:
  `markdown v1.0.0`, `rstest v0.26.1`, `proptest v1.11.0`, and `insta v1.48.0`.
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/lib.rs`
  line 160 defines
  `pub fn to_mdast(value: &str, options: &ParseOptions) -> Result<mdast::Node, message::Message>`.
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/configuration.rs`
  lines 1275-1280 define `ParseOptions::gfm()` as `Constructs::gfm()` plus
  defaults, and the adjacent documentation says GFM adds tables and tasklists.
  Therefore fixtures may rely on the existing parser accepting GFM tables and
  task lists. They must not rely on the crate for exact Markdown rendering.
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rstest-0.26.1/src/lib.rs`
  documents fixture injection around lines 252-276 and re-exports
  `rstest_macros::fixture` at line 571. New tests should follow the existing
  `#[fixture]` and `#[rstest]` style.
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/sugar.rs`
  line 624 defines `prop_compose!`. Roadmap task 3.1.1 should not add the no-op
  property, but the fixture corpus should be shaped so task 3.1.2 can build
  strategies or corpus enumeration without moving files.
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/insta-1.48.0/src/macros.rs`
  line 463 defines `assert_snapshot!`. This task deliberately avoids new
  snapshots because exact Markdown fixture files are the design requirement.
- `tests/golden/mod.rs::expected_output` currently removes one final newline
  before comparison, and `src/roadmap/render.rs::render_roadmap` currently
  returns `blocks.join("\n\n")`. Work item 1 must fix this before any fixture
  claims byte identity.

## Plan of work

### Work item 1: Establish raw-byte golden comparisons and the renderer newline contract

Documentation to read: `docs/mapsplice-design.md` sections 5, 6, and 8;
`docs/developers-guide.md` sections 2, 3, and 6; `docs/users-guide.md` sections
"Output modes" and "Validation rules and failure cases";
`docs/documentation-style-guide.md` sections "Markdown rules" and "Formatting";
and `AGENTS.md` sections "Rust Specific Guidance", "Testing", "Error Handling",
and "Markdown Guidance".

Skills to load: `leta`, `rust-router`, `rust-unit-testing`, `rust-errors`,
`domain-cli-and-daemons`, `sem`, and `en-gb-oxendict-style`.

Make this the first implementation commit. Before adding any new golden
metadata or assertions, split `tests/golden/mod.rs` because it is already 358
lines. Create these required files and move code without changing behaviour:

- `tests/golden/case.rs` owns `GoldenCase`, `GoldenCommand`,
  `GoldenExpectation`, `SuccessOutput`, `FailureOutput`, `ExpectedError`, and
  `FixturePath`.
- `tests/golden/workspace.rs` owns `GoldenWorkspace`, `create_workspace`, and
  fixture-reading helpers.
- `tests/golden/runner.rs` owns `assert_golden_case`, command argument
  construction, and success/failure dispatch.
- `tests/golden/assertions.rs` owns stdout, target, written-path, and typed
  error assertion helpers.
- `tests/golden/mod.rs` keeps only the module-level documentation, module
  declarations, `metadata_tests`, and crate-visible re-exports used by
  `tests/roadmap_golden.rs`.

After the split, run the existing golden test before adding behaviour. The
split is acceptable only if every touched Rust file remains below 400 lines and
`tests/golden/mod.rs` is below 220 lines. Then add a focused red renderer test
in `src/roadmap/render_tests.rs` proving that `render_roadmap` returns a
non-empty roadmap ending in exactly one `\n`, not zero and not two. Change
`src/roadmap/render.rs::render_roadmap` minimally so non-empty rendered
roadmaps end in exactly one final newline. Update the existing nested sub-task
render expectation if it currently assumes no final newline.

Update `docs/mapsplice-design.md` section 5 so F3 names the one documented
normalization: non-empty rendered roadmaps end in exactly one final newline,
and rerendering does not add another. Update `docs/users-guide.md` section
"Output modes" with the same user-visible contract.

Update the split golden harness so expected successful output is compared as
raw fixture bytes. Remove final-newline stripping from the fixture-output
helper. Add private metadata for an identity expectation that compares stdout
directly to the original `target.md` bytes; this will be used by work item 4.

Tests to add or update: a golden-harness split smoke run with no intended
behaviour change, a renderer unit test for the final-newline contract, the
existing nested sub-task render test if needed, and golden harness metadata
self-tests for the raw fixture-byte and original-target expectation. No
property, snapshot, or new behavioural feature file is required in this work
item.

Validation for this work item:

```bash
set -o pipefail
cargo test --workspace --all-targets --all-features --test roadmap_golden \
  2>&1 | tee /tmp/test-mapsplice-roadmap-3-1-1-raw-bytes.out
cargo test --workspace --all-targets --all-features roadmap::render \
  2>&1 | tee /tmp/test-mapsplice-roadmap-3-1-1-render-newline.out
wc -l tests/golden/mod.rs tests/golden/case.rs tests/golden/workspace.rs \
  tests/golden/runner.rs tests/golden/assertions.rs \
  tests/golden/metadata_tests.rs \
  2>&1 | tee /tmp/wc-mapsplice-roadmap-3-1-1-golden-split.out
mdtablefix docs/execplans/roadmap-3-1-1.md docs/mapsplice-design.md docs/users-guide.md \
  2>&1 | tee /tmp/mdtablefix-mapsplice-roadmap-3-1-1-raw-bytes.out
markdownlint-cli2 --fix docs/execplans/roadmap-3-1-1.md docs/mapsplice-design.md docs/users-guide.md \
  2>&1 | tee /tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-raw-bytes.out
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-3-1-1-raw-bytes.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-raw-bytes.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-raw-bytes.out
sem diff --format json 2>&1 | tee /tmp/sem-mapsplice-roadmap-3-1-1-raw-bytes.out
```

Commit only after these commands pass.

### Work item 2: Add successful operation golden fixtures

Documentation to read: `docs/roadmap.md` section 3.1.1;
`docs/mapsplice-design.md` sections 4, 6, and 8; `docs/users-guide.md` sections
"Command overview", "`append`", "`insert`", "`delete`", "`replace`", and
"Output modes"; `docs/developers-guide.md` sections 2, 3, and 6; and
`AGENTS.md` sections "Rust Specific Guidance" and "Testing".

Skills to load: `leta`, `rust-router`, `rust-unit-testing`,
`domain-cli-and-daemons`, `sem`, and `en-gb-oxendict-style`.

Add one successful stdout-mode golden case per operation shape under
`tests/fixtures/golden/`, and register each case in `tests/roadmap_golden.rs`
with explicit `GoldenCase` metadata:

- `append_phase/` proves phase-level append, full renumbering, and stdout
  output.
- `insert_phase_before/` proves inserting a phase before an anchor.
- `insert_step_after/` proves `insert --after` at step level.
- `insert_task_before/` proves inserting a task before an anchor.
- `insert_sub_task_after/` proves inserting an addendum sub-task after an
  addendum anchor.
- `delete_task/` proves deleting one task and renumbering later tasks.
- `replace_step/` proves replacing one step with sibling steps.
- `replace_sub_task/` proves replacing an addendum sub-task while preserving
  the parent task.

Each directory must contain the files its command needs. Fragment commands
contain `target.md`, `fragment.md`, and `expected.md`; delete contains
`target.md` and `expected.md`. The metadata must state the exact command
arguments rather than inferring them from directory names.

Tests to add or update: named behavioural golden tests in
`tests/roadmap_golden.rs`. No property or snapshot tests are added here. Add a
unit test only if a fixture exposes an isolated production defect.

Validation for this work item:

```bash
set -o pipefail
cargo test --workspace --all-targets --all-features --test roadmap_golden \
  2>&1 | tee /tmp/test-mapsplice-roadmap-3-1-1-operations.out
mdtablefix docs/execplans/roadmap-3-1-1.md \
  tests/fixtures/golden/append_phase/target.md \
  tests/fixtures/golden/append_phase/fragment.md \
  tests/fixtures/golden/append_phase/expected.md \
  tests/fixtures/golden/insert_phase_before/target.md \
  tests/fixtures/golden/insert_phase_before/fragment.md \
  tests/fixtures/golden/insert_phase_before/expected.md \
  tests/fixtures/golden/insert_step_after/target.md \
  tests/fixtures/golden/insert_step_after/fragment.md \
  tests/fixtures/golden/insert_step_after/expected.md \
  tests/fixtures/golden/insert_task_before/target.md \
  tests/fixtures/golden/insert_task_before/fragment.md \
  tests/fixtures/golden/insert_task_before/expected.md \
  tests/fixtures/golden/insert_sub_task_after/target.md \
  tests/fixtures/golden/insert_sub_task_after/fragment.md \
  tests/fixtures/golden/insert_sub_task_after/expected.md \
  tests/fixtures/golden/delete_task/target.md \
  tests/fixtures/golden/delete_task/expected.md \
  tests/fixtures/golden/replace_step/target.md \
  tests/fixtures/golden/replace_step/fragment.md \
  tests/fixtures/golden/replace_step/expected.md \
  tests/fixtures/golden/replace_sub_task/target.md \
  tests/fixtures/golden/replace_sub_task/fragment.md \
  tests/fixtures/golden/replace_sub_task/expected.md \
  2>&1 | tee /tmp/mdtablefix-mapsplice-roadmap-3-1-1-operations.out
markdownlint-cli2 --fix docs/execplans/roadmap-3-1-1.md \
  tests/fixtures/golden/append_phase/target.md \
  tests/fixtures/golden/append_phase/fragment.md \
  tests/fixtures/golden/append_phase/expected.md \
  tests/fixtures/golden/insert_phase_before/target.md \
  tests/fixtures/golden/insert_phase_before/fragment.md \
  tests/fixtures/golden/insert_phase_before/expected.md \
  tests/fixtures/golden/insert_step_after/target.md \
  tests/fixtures/golden/insert_step_after/fragment.md \
  tests/fixtures/golden/insert_step_after/expected.md \
  tests/fixtures/golden/insert_task_before/target.md \
  tests/fixtures/golden/insert_task_before/fragment.md \
  tests/fixtures/golden/insert_task_before/expected.md \
  tests/fixtures/golden/insert_sub_task_after/target.md \
  tests/fixtures/golden/insert_sub_task_after/fragment.md \
  tests/fixtures/golden/insert_sub_task_after/expected.md \
  tests/fixtures/golden/delete_task/target.md \
  tests/fixtures/golden/delete_task/expected.md \
  tests/fixtures/golden/replace_step/target.md \
  tests/fixtures/golden/replace_step/fragment.md \
  tests/fixtures/golden/replace_step/expected.md \
  tests/fixtures/golden/replace_sub_task/target.md \
  tests/fixtures/golden/replace_sub_task/fragment.md \
  tests/fixtures/golden/replace_sub_task/expected.md \
  2>&1 | tee /tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-operations.out
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-3-1-1-operations.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-operations.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-operations.out
sem diff --format json 2>&1 | tee /tmp/sem-mapsplice-roadmap-3-1-1-operations.out
```

Commit only after these commands pass.

### Work item 3: Add grammar-surface preservation fixtures

Documentation to read: `docs/roadmap.md` section 3.1.1;
`docs/mapsplice-design.md` sections 4, 5, and 8; `docs/users-guide.md` section
"The roadmap shape `mapsplice` expects"; `docs/documentation-style-guide.md`
sections "Markdown rules" and "Formatting"; and `AGENTS.md` section "Markdown
Guidance".

Skills to load: `leta`, `rust-router`, `rust-unit-testing`, `sem`, and
`en-gb-oxendict-style`.

Add successful golden cases that each isolate one grammar-surface preservation
requirement while using a real insert operation:

- `preamble_preserved/` proves optional preamble content before the first
  phase survives an unrelated task insertion unchanged.
- `phase_step_task_surface/` proves phases, steps, tasks, and task checklist
  markers render in the accepted grammar.
- `multi_line_task_body/` proves continuation paragraphs in an untouched task
  body survive exactly.
- `nested_bullets/` proves ordinary nested bullets remain task body Markdown,
  not addendum sub-tasks.
- `tables_preserved/` proves a GitHub Flavoured Markdown table inside an
  untouched task body renders deterministically.
- `code_blocks_preserved/` proves fenced code blocks, language tags, and code
  indentation survive exactly.
- `addendum_body_surface/` proves an addendum sub-task with its own body
  remains nested under its parent task.

Every case in this work item uses `insert --after`, so every directory must
contain `target.md`, `fragment.md`, and `expected.md`. Do not combine all
grammar surfaces into one fixture; each named case should fail with a narrow
diff.

Tests to add or update: named behavioural golden tests in
`tests/roadmap_golden.rs`. No property or snapshot tests are added here.

Validation for this work item:

```bash
set -o pipefail
cargo test --workspace --all-targets --all-features --test roadmap_golden \
  2>&1 | tee /tmp/test-mapsplice-roadmap-3-1-1-grammar.out
mdtablefix docs/execplans/roadmap-3-1-1.md \
  tests/fixtures/golden/preamble_preserved/target.md \
  tests/fixtures/golden/preamble_preserved/fragment.md \
  tests/fixtures/golden/preamble_preserved/expected.md \
  tests/fixtures/golden/phase_step_task_surface/target.md \
  tests/fixtures/golden/phase_step_task_surface/fragment.md \
  tests/fixtures/golden/phase_step_task_surface/expected.md \
  tests/fixtures/golden/multi_line_task_body/target.md \
  tests/fixtures/golden/multi_line_task_body/fragment.md \
  tests/fixtures/golden/multi_line_task_body/expected.md \
  tests/fixtures/golden/nested_bullets/target.md \
  tests/fixtures/golden/nested_bullets/fragment.md \
  tests/fixtures/golden/nested_bullets/expected.md \
  tests/fixtures/golden/tables_preserved/target.md \
  tests/fixtures/golden/tables_preserved/fragment.md \
  tests/fixtures/golden/tables_preserved/expected.md \
  tests/fixtures/golden/code_blocks_preserved/target.md \
  tests/fixtures/golden/code_blocks_preserved/fragment.md \
  tests/fixtures/golden/code_blocks_preserved/expected.md \
  tests/fixtures/golden/addendum_body_surface/target.md \
  tests/fixtures/golden/addendum_body_surface/fragment.md \
  tests/fixtures/golden/addendum_body_surface/expected.md \
  2>&1 | tee /tmp/mdtablefix-mapsplice-roadmap-3-1-1-grammar.out
markdownlint-cli2 --fix docs/execplans/roadmap-3-1-1.md \
  tests/fixtures/golden/preamble_preserved/target.md \
  tests/fixtures/golden/preamble_preserved/fragment.md \
  tests/fixtures/golden/preamble_preserved/expected.md \
  tests/fixtures/golden/phase_step_task_surface/target.md \
  tests/fixtures/golden/phase_step_task_surface/fragment.md \
  tests/fixtures/golden/phase_step_task_surface/expected.md \
  tests/fixtures/golden/multi_line_task_body/target.md \
  tests/fixtures/golden/multi_line_task_body/fragment.md \
  tests/fixtures/golden/multi_line_task_body/expected.md \
  tests/fixtures/golden/nested_bullets/target.md \
  tests/fixtures/golden/nested_bullets/fragment.md \
  tests/fixtures/golden/nested_bullets/expected.md \
  tests/fixtures/golden/tables_preserved/target.md \
  tests/fixtures/golden/tables_preserved/fragment.md \
  tests/fixtures/golden/tables_preserved/expected.md \
  tests/fixtures/golden/code_blocks_preserved/target.md \
  tests/fixtures/golden/code_blocks_preserved/fragment.md \
  tests/fixtures/golden/code_blocks_preserved/expected.md \
  tests/fixtures/golden/addendum_body_surface/target.md \
  tests/fixtures/golden/addendum_body_surface/fragment.md \
  tests/fixtures/golden/addendum_body_surface/expected.md \
  2>&1 | tee /tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-grammar.out
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-3-1-1-grammar.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-grammar.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-grammar.out
sem diff --format json 2>&1 | tee /tmp/sem-mapsplice-roadmap-3-1-1-grammar.out
```

Commit only after these commands pass.

### Work item 4: Add fidelity and dependency-contract fixtures

Documentation to read: `docs/mapsplice-design.md` sections 5, 6, 7, and 8;
`docs/roadmap.md` sections 1.1.3, 2.1.3, and 3.1.1; `docs/users-guide.md`
sections "Worked example" and "Validation rules and failure cases"; and
`docs/developers-guide.md` section 6.

Skills to load: `leta`, `rust-router`, `rust-unit-testing`, `rust-verification`,
`proptest`, `rust-errors`, `sem`, and `en-gb-oxendict-style`.

Keep the existing `tests/fixtures/reference_rewrite/` cases and register any
missing coverage through the golden harness instead of moving them. Add these
new golden cases under `tests/fixtures/golden/`, except for the two C3 cases
that are explicitly assigned to `tests/fixtures/reference_rewrite/` below:

- `f1_minimal_untouched_content/` proves unrelated text, formatting, tables,
  and code blocks remain unchanged while an operation changes a separate item.
- `f2_minimal_renumber_diff/` proves the only successful-output changes are
  the addressed edit, deterministic renumbering, and dependency-reference
  rewrites.
- `f3_c5_identity_replace/` replaces a task with byte-identical task Markdown
  and uses the original-target expectation from work item 1 to prove
  byte-identical stdout.
- `c2_contiguous_renumber/` proves phase, step, task, and addendum numbers are
  contiguous after an insertion.
- Keep `c3_requires_list_rewrite` at the existing concrete fixture location
  `tests/fixtures/reference_rewrite/multi_id_requires.input.md` and
  `tests/fixtures/reference_rewrite/multi_id_requires.expected.md`. Register
  that existing case through the golden harness as the C3 multi-id `Requires`
  proof; do not create `tests/fixtures/golden/c3_requires_list_rewrite/` in
  this work item.
- Keep `c3_substring_non_match` at the existing concrete fixture location
  `tests/fixtures/reference_rewrite/substring_non_match.input.md` and
  `tests/fixtures/reference_rewrite/substring_non_match.expected.md`. Register
  that existing case through the golden harness as the C3 greedy
  token-consumption proof; do not create
  `tests/fixtures/golden/c3_substring_non_match/` in this work item.
- `c4_addendum_renumber/` proves `8.2.3.1` tracks its parent task when the
  parent renumbers.
- `c4_addendum_render_fidelity/` proves addendum nesting and indentation are
  preserved on render.

For failure contracts, add `c3_dangling_requires_failure/` with `target.md`
only and typed `ExpectedError::DanglingDependency`. The assertion must prove
the target remains unchanged. Do not provide `expected.md` for this failure
case unless the harness explicitly uses it.

Tests to add or update: named behavioural golden tests and harness metadata for
the identity expectation and dangling dependency failure. Add unit tests only
for isolated helper logic introduced in the harness. Do not add roadmap task
3.1.2's generated no-op property here, but keep the fixture shape
property-friendly.

Validation for this work item:

```bash
set -o pipefail
cargo test --workspace --all-targets --all-features --test roadmap_golden \
  2>&1 | tee /tmp/test-mapsplice-roadmap-3-1-1-contracts.out
mdtablefix docs/execplans/roadmap-3-1-1.md \
  tests/fixtures/golden/f1_minimal_untouched_content/target.md \
  tests/fixtures/golden/f1_minimal_untouched_content/fragment.md \
  tests/fixtures/golden/f1_minimal_untouched_content/expected.md \
  tests/fixtures/golden/f2_minimal_renumber_diff/target.md \
  tests/fixtures/golden/f2_minimal_renumber_diff/fragment.md \
  tests/fixtures/golden/f2_minimal_renumber_diff/expected.md \
  tests/fixtures/golden/f3_c5_identity_replace/target.md \
  tests/fixtures/golden/f3_c5_identity_replace/fragment.md \
  tests/fixtures/golden/c2_contiguous_renumber/target.md \
  tests/fixtures/golden/c2_contiguous_renumber/fragment.md \
  tests/fixtures/golden/c2_contiguous_renumber/expected.md \
  tests/fixtures/golden/c4_addendum_renumber/target.md \
  tests/fixtures/golden/c4_addendum_renumber/fragment.md \
  tests/fixtures/golden/c4_addendum_renumber/expected.md \
  tests/fixtures/golden/c4_addendum_render_fidelity/target.md \
  tests/fixtures/golden/c4_addendum_render_fidelity/fragment.md \
  tests/fixtures/golden/c4_addendum_render_fidelity/expected.md \
  tests/fixtures/golden/c3_dangling_requires_failure/target.md \
  2>&1 | tee /tmp/mdtablefix-mapsplice-roadmap-3-1-1-contracts.out
markdownlint-cli2 --fix docs/execplans/roadmap-3-1-1.md \
  tests/fixtures/golden/f1_minimal_untouched_content/target.md \
  tests/fixtures/golden/f1_minimal_untouched_content/fragment.md \
  tests/fixtures/golden/f1_minimal_untouched_content/expected.md \
  tests/fixtures/golden/f2_minimal_renumber_diff/target.md \
  tests/fixtures/golden/f2_minimal_renumber_diff/fragment.md \
  tests/fixtures/golden/f2_minimal_renumber_diff/expected.md \
  tests/fixtures/golden/f3_c5_identity_replace/target.md \
  tests/fixtures/golden/f3_c5_identity_replace/fragment.md \
  tests/fixtures/golden/c2_contiguous_renumber/target.md \
  tests/fixtures/golden/c2_contiguous_renumber/fragment.md \
  tests/fixtures/golden/c2_contiguous_renumber/expected.md \
  tests/fixtures/golden/c4_addendum_renumber/target.md \
  tests/fixtures/golden/c4_addendum_renumber/fragment.md \
  tests/fixtures/golden/c4_addendum_renumber/expected.md \
  tests/fixtures/golden/c4_addendum_render_fidelity/target.md \
  tests/fixtures/golden/c4_addendum_render_fidelity/fragment.md \
  tests/fixtures/golden/c4_addendum_render_fidelity/expected.md \
  tests/fixtures/golden/c3_dangling_requires_failure/target.md \
  2>&1 | tee /tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-contracts.out
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-3-1-1-contracts.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-contracts.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-contracts.out
sem diff --format json 2>&1 | tee /tmp/sem-mapsplice-roadmap-3-1-1-contracts.out
```

Commit only after these commands pass.

### Work item 5: Add output-mode and fail-closed golden fixtures

Documentation to read: `docs/mapsplice-design.md` sections 5 and 6;
`docs/users-guide.md` sections "Output modes" and "Validation rules and failure
cases"; `docs/developers-guide.md` sections 2, 3, and 6; and `AGENTS.md`
sections "Error Handling" and "Testing".

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
  anchor returns `MapspliceError::AnchorNotFound` through the golden harness
  and leaves the target unchanged. The golden harness must not claim to prove
  process stdout for this failure, because `run_from_args` returns
  `Err(MapspliceError)` and exposes no stdout value on failures.

Add a compiled-binary BDD scenario for the C6/F5 CLI contract that the golden
harness cannot observe. In `tests/features/mapsplice.feature`, add:

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
`["--in-place", "delete", target.as_str(), "99"]`; reuse the existing
`stdout is empty`, `stderr mentions that anchor 99 was not found`, and
`the target file remains unchanged` steps.

If `ExpectedError` currently names only `DanglingDependency`, extend it with
the typed variants actually returned by `MapspliceError` for these cases. Do
not assert on brittle full display strings when a semantic enum variant is
available.

Tests to add or update: named behavioural golden tests, `ExpectedError`
metadata tests, the compiled-binary BDD scenario named
`missing_anchor_in_place`, and any focused unit tests needed for new harness
failure assertions. No property or snapshot tests are added here.

Validation for this work item:

```bash
set -o pipefail
cargo test --workspace --all-targets --all-features --test roadmap_golden \
  2>&1 | tee /tmp/test-mapsplice-roadmap-3-1-1-output-failure.out
cargo test --workspace --all-targets --all-features --test behaviour_cli \
  missing_anchor_in_place \
  2>&1 | tee /tmp/test-mapsplice-roadmap-3-1-1-missing-anchor-cli.out
mdtablefix docs/execplans/roadmap-3-1-1.md \
  tests/fixtures/golden/c6_stdout_target_unchanged/target.md \
  tests/fixtures/golden/c6_stdout_target_unchanged/fragment.md \
  tests/fixtures/golden/c6_stdout_target_unchanged/expected.md \
  tests/fixtures/golden/c6_in_place_success/target.md \
  tests/fixtures/golden/c6_in_place_success/fragment.md \
  tests/fixtures/golden/c6_in_place_success/expected.md \
  tests/fixtures/golden/f5_malformed_grammar_failure/target.md \
  tests/fixtures/golden/f5_level_mismatch_failure/target.md \
  tests/fixtures/golden/f5_level_mismatch_failure/fragment.md \
  tests/fixtures/golden/f5_missing_anchor_in_place_failure/target.md \
  2>&1 | tee /tmp/mdtablefix-mapsplice-roadmap-3-1-1-output-failure.out
markdownlint-cli2 --fix docs/execplans/roadmap-3-1-1.md \
  tests/fixtures/golden/c6_stdout_target_unchanged/target.md \
  tests/fixtures/golden/c6_stdout_target_unchanged/fragment.md \
  tests/fixtures/golden/c6_stdout_target_unchanged/expected.md \
  tests/fixtures/golden/c6_in_place_success/target.md \
  tests/fixtures/golden/c6_in_place_success/fragment.md \
  tests/fixtures/golden/c6_in_place_success/expected.md \
  tests/fixtures/golden/f5_malformed_grammar_failure/target.md \
  tests/fixtures/golden/f5_level_mismatch_failure/target.md \
  tests/fixtures/golden/f5_level_mismatch_failure/fragment.md \
  tests/fixtures/golden/f5_missing_anchor_in_place_failure/target.md \
  2>&1 | tee /tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-output-failure.out
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-3-1-1-output-failure.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-output-failure.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-output-failure.out
sem diff --format json 2>&1 | tee /tmp/sem-mapsplice-roadmap-3-1-1-output-failure.out
```

Commit only after these commands pass.

### Work item 6: Mark roadmap 3.1.1 complete after gates

Documentation to read: `docs/roadmap.md` section 3.1.1;
`docs/mapsplice-design.md` section 8; `docs/developers-guide.md` section 7; and
`AGENTS.md` sections "Change Quality & Committing" and "Markdown Guidance".

Skills to load: `leta`, `sem`, `en-gb-oxendict-style`, and `changelog` only if
release notes are explicitly requested later.

After work items 1 through 5 are committed and gated, update `docs/roadmap.md`
to mark `3.1.1` complete. Update this ExecPlan's `Progress`, `Decision Log`, and
`Outcomes & Retrospective` with fixture counts and final gate evidence. Do not
mark 3.1.2 or 3.1.3 complete.

Tests to add or update: no new Rust tests. This is a documentation-only
completion commit.

Validation for this work item:

```bash
set -o pipefail
mdtablefix docs/execplans/roadmap-3-1-1.md docs/roadmap.md \
  2>&1 | tee /tmp/mdtablefix-mapsplice-roadmap-3-1-1-complete.out
markdownlint-cli2 --fix docs/execplans/roadmap-3-1-1.md docs/roadmap.md \
  2>&1 | tee /tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-complete.out
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-3-1-1-complete.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-complete.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-complete.out
sem diff --format json 2>&1 | tee /tmp/sem-mapsplice-roadmap-3-1-1-complete.out
```

Commit only after these commands pass.

## Concrete steps

1. Start each implementation session by checking the worktree:

   ```bash
   cd /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1
   git branch --show-current
   git status --short
   sem diff --from origin/main --to HEAD --format json
   ```

   Expected branch output is `roadmap-3-1-1`. Stop if the branch differs.

2. For each work item, load the skills listed in that item and re-read the
   named documentation sections.

3. Add the red test or fixture first, run the focused test command, and record
   the expected failure in this ExecPlan's `Progress`.

4. Make the smallest implementation or fixture change needed for the focused
   test to pass.

5. Run the work item's path-specific formatter commands and full gates exactly
   as listed.

6. Review entity-level changes with `sem diff --format json`, update this
   ExecPlan's living sections, and commit the atomic work item.

7. Repeat for the next work item. Do not skip gates between commits.

## Validation and acceptance

The final acceptance criteria for roadmap task 3.1.1 are:

- `tests/roadmap_golden.rs` contains named golden tests for every operation,
  every required grammar surface, every fidelity guarantee F1-F5 that is
  example-expressible, every contract guarantee C1-C6 that is
  example-expressible, and every required adversarial fixture class in
  `docs/mapsplice-design.md` section 8.
- Successful cases are committed as explicit input-and-expected Markdown files
  under `tests/fixtures/golden/<case-name>/` or, for existing reference
  rewrites, remain covered under `tests/fixtures/reference_rewrite/`.
- The C3 `Requires` list and substring non-match proofs remain at their
  existing `tests/fixtures/reference_rewrite/multi_id_requires.*.md` and
  `tests/fixtures/reference_rewrite/substring_non_match.*.md` locations; no
  `tests/fixtures/golden/c3_requires_list_rewrite/` or
  `tests/fixtures/golden/c3_substring_non_match/` directory is part of this
  plan.
- Failure cases assert typed `MapspliceError` variants, no roadmap body where
  appropriate, and unchanged target bytes.
- The missing-anchor `--in-place` failure has compiled-binary BDD coverage
  proving the command fails, stdout is empty, stderr reports the missing
  anchor, and the target file remains unchanged.
- Expected output is compared as raw fixture bytes; the harness no longer
  strips a final newline before comparison.
- The renderer emits canonical non-empty Markdown ending in exactly one final
  newline.
- `docs/roadmap.md` marks only 3.1.1 complete after the corpus and gates are
  complete.
- The final committed state passes:

  ```bash
  set -o pipefail
  make all 2>&1 | tee /tmp/all-mapsplice-roadmap-3-1-1-final.out
  make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-final.out
  make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-1-final.out
  ```

## Idempotence and recovery

All work items are additive except the renderer newline and golden harness
raw-byte changes in work item 1. If a fixture fails unexpectedly, inspect the
exact `target.md`, `fragment.md`, and `expected.md` for that case first, then
the command metadata in `tests/roadmap_golden.rs`. Do not update expected
fixtures until the actual output has been checked against
`docs/mapsplice-design.md` and `docs/users-guide.md`.

If a formatter changes files outside the current work item, do not commit that
churn. Park or discard it with the named stash format in `Tolerances`, then
rerun `git status --short` before continuing.

If a production defect appears, keep the red fixture, fix the smallest affected
production surface, and update `Decision Log` with the defect and fix. Stop and
revise this plan before changing a public API, adding a dependency, or touching
more than six non-fixture Rust source files for one defect.

## Artifacts and notes

Current planning evidence:

```plaintext
$ git branch --show-current
roadmap-3-1-1

$ sem diff --from origin/main --to HEAD --format json
{"summary":{"fileCount":0,"added":0,"modified":0,"deleted":0,"moved":0,"renamed":0,"reordered":0,"orphan":0,"total":0},"changes":[]}

$ mcp__memtrace.list_indexed_repositories
user cancelled MCP tool call

$ leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1
Workspace already added: /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1

$ leta files
AGENTS.md (15.1KB, 306 lines)
src/lib.rs (4.8KB, 161 lines)
tests/golden/mod.rs (10.4KB, 358 lines)
tests/behaviour_cli.rs (3.1KB, 112 lines)

$ mcp__firecrawl.firecrawl_scrape https://docs.rs/markdown/1.0.0/markdown/fn.to_mdast.html
user cancelled MCP tool call
```

## Interfaces and dependencies

No public API changes are planned. The work should extend private test metadata
in `tests/golden/mod.rs` and public behaviour only through the renderer's
documented final-newline contract.

Existing dependency behaviour pinned for this task:

- `markdown v1.0.0`: use only for parsing through the existing
  `markdown::to_mdast` and `ParseOptions::gfm()` path. Do not use it for exact
  Markdown rendering.
- `rstest v0.26.1`: use existing fixture and parameterized-test style.
- `proptest v1.11.0`: do not add 3.1.2's property here; keep fixture layout
  ready for later property enumeration.
- `insta v1.48.0`: do not add new snapshots for this corpus; exact Markdown
  files are the required artefacts.

Revision note: This first planning-round revision replaces the stale
pre-existing draft with a self-contained DRAFT ExecPlan for roadmap task 3.1.1,
records current tool availability, pins dependency behaviour to locked local
source, and decomposes implementation into six atomic, gate-passable work
items. No implementation work has started.

Revision note: The planning round 2 revision resolves the design-review blocker
in work item 4 by choosing the existing
`tests/fixtures/reference_rewrite/multi_id_requires.*.md` and
`tests/fixtures/reference_rewrite/substring_non_match.*.md` locations for the
two C3 proofs. The validation commands now omit the contradictory
`tests/fixtures/golden/c3_requires_list_rewrite/*` and
`tests/fixtures/golden/c3_substring_non_match/*` paths, so every direct
Markdown formatter path in work item 4 is a fixture file that the work item
will definitely create or edit.

Revision note: The planning round 3 revision resolves two design-review
blockers. Work item 1 now requires splitting `tests/golden/mod.rs` into smaller
harness modules before adding raw-byte, identity, or typed-error metadata, and
validates the split with an explicit line-count checkpoint. Work item 5 now
narrows the golden-harness missing-anchor claim to typed
`MapspliceError::AnchorNotFound` plus unchanged target bytes and adds a
compiled-binary BDD scenario to prove `--in-place` missing-anchor failure emits
no roadmap body on stdout.
