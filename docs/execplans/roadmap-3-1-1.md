# Assemble grammar-surface and per-contract golden fixtures

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: DRAFT

## Purpose / big picture

Roadmap task 3.1.1 is complete when `mapsplice` has a reviewed golden-fixture
corpus that proves the accepted roadmap grammar, every supported structural
operation, and each fidelity or contract guarantee that can be represented as a
deterministic example. A maintainer should be able to inspect
`tests/fixtures/golden/`, understand which guarantee each case proves, run the
focused golden test binary, and receive exact Markdown comparisons or exact
fail-closed assertions.

This plan does not implement roadmap task 3.1.2's generated no-op property or
roadmap task 3.1.3's rendered-output Markdown stability sweep. It does require
task 3.1.1 to include one exact identity fixture for F3/C5: replacing an item
with byte-identical content must leave the complete roadmap byte-identical.
Task 3.1.2 can later broaden that pinned example into a property over every
conformant fixture.

## Constraints

- Work only in
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1`.
- Do not edit the root/control worktree at `/home/leynos/Projects/mapsplice`.
- Treat `origin/main` as canonical and the integration branch as `main`.
- Use Memtrace as the primary canonical-main code search and graph tool when it
  is available. First call `list_indexed_repositories`, confirm repo id
  `mapsplice`, then use `find_code`, `find_symbol`, `list_communities`,
  `find_central_symbols`, `get_symbol_context`, `get_impact`, `get_timeline`,
  and `get_source_window` as the change surface warrants.
- If Memtrace is cancelled or unavailable, record the exact failure and
  continue with bounded branch-local evidence. Memtrace unavailability is not a
  blocker for this plan.
- Use `leta` for branch-local symbol navigation, references, and call graphs
  when it is available. If `leta` cannot start, record the exact failure and
  use precise local file inspection for the affected task.
- Use `sem` for codebase history navigation and entity-level diff review
  instead of raw `git log` or `git blame`.
- Use `docs/mapsplice-design.md`, `docs/developers-guide.md`,
  `docs/users-guide.md`, `docs/contributing.md`,
  `docs/documentation-style-guide.md`, `docs/scripting-standards.md`,
  `docs/execplans/initial-tool.md`, `AGENTS.md`, and `docs/roadmap.md` as the
  source-of-truth documents.
- Follow en-GB Oxford spelling in prose, comments, test names where natural,
  and commit messages.
- Do not add a new external dependency for roadmap task 3.1.1.
- Do not redesign the roadmap grammar, operation semantics, dependency
  reference model, renderer, command-line interface, or public library API
  unless a focused golden fixture exposes a real defect.
- Keep Rust source files below 400 lines. Split test metadata into focused
  modules before any source file approaches that limit.
- Fixture files are test inputs, not generated artefacts. Commit them as
  ordinary source-controlled Markdown.
- Format only Markdown files changed by the current work item with
  path-specific formatter commands. Do not run repository-global Markdown
  formatting such as `make fmt` or `mdformat-all` for this task.
- Every test, lint, format check, and gate command must be logged with `tee` to
  a branch-specific `/tmp` file.
- Commit after each work item that changes files, and gate each commit.

## Tolerances

- If `git branch --show-current` is not `roadmap-3-1-1`, stop before editing.
- If implementation requires a public API signature change, stop and revise
  this plan before editing the API.
- If implementation requires a new crate, stop and revise this plan with locked
  source evidence and official documentation evidence for that dependency.
- If a fixture exposes a real production defect, keep the defect fix in the
  same atomic work item as the red fixture that proves it. Stop and revise this
  plan before changing more than one production module for that defect.
- If any work item would touch more than six non-fixture Rust source files,
  split the work item before committing.
- If a focused test or repository gate still fails after two focused fix
  attempts, record the failing command, log path, and observed error in
  `Decision Log`, then stop for review.
- If formatter churn touches files outside the current work item, park or
  discard it with a named stash following
  `df12-stash v1 task=3.1.1 kind=<discard|park|keep> reason="<short>"`.
- If Memtrace, Firecrawl, `leta`, or another advisory tool is unavailable, do
  not mark this plan blocked. Record the exact failed command or tool result in
  `Surprises & Discoveries` and continue with bounded local evidence.

## Risks

- Risk: Adding all required golden fixtures in one commit could create an
  unreviewable test diff.
  Severity: medium.
  Likelihood: high.
  Mitigation: split the corpus into independently committable operation,
  grammar-surface, contract/adversarial, output-mode, and final-roadmap work
  items.

- Risk: A table-driven harness can hide which exact fixture failed.
  Severity: medium.
  Likelihood: medium.
  Mitigation: give every case a stable descriptive test name and make assertion
  failures include the case name, command, expected output, and actual output.

- Risk: The `markdown` crate parses GitHub Flavoured Markdown (GFM) but does
  not provide the exact Markdown writer needed by the fidelity contract.
  Severity: high.
  Likelihood: verified.
  Mitigation: use `markdown` only through the existing parser path and exercise
  the existing `mapsplice` renderer through `run_from_args`, comparing exact
  committed Markdown files.

- Risk: The later 3.1.2 property test may need a different fixture discovery
  shape.
  Severity: medium.
  Likelihood: medium.
  Mitigation: store each successful golden case under a directory with
  `target.md`, optional `fragment.md`, and `expected.md`, and avoid encoding
  the expected output only in Rust source.

- Risk: Some design guarantees are fail-closed rather than successful-output
  cases.
  Severity: medium.
  Likelihood: high.
  Mitigation: include failure fixtures where the design requires an
  adversarial class, and assert typed errors, empty stdout, and unchanged target
  files instead of inventing successful output.

## Progress

- [x] (2026-07-02T00:00:00Z) Confirmed the assigned worktree and branch:
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1` on
  `roadmap-3-1-1`.
- [x] (2026-07-02T00:00:00Z) Loaded `execplans`, `leta`, `firecrawl-mcp`,
  `rust-router`, and `roadmap-doc` for this planning round.
- [x] (2026-07-02T00:00:00Z) Read source-of-truth documents:
  `AGENTS.md`, `docs/roadmap.md`, `docs/mapsplice-design.md`,
  `docs/developers-guide.md`, `docs/users-guide.md`,
  `docs/contributing.md`, `docs/documentation-style-guide.md`,
  `docs/scripting-standards.md`, and `docs/execplans/initial-tool.md`.
- [x] (2026-07-02T00:00:00Z) Verified
  `sem diff --from origin/main --to HEAD --format json` reported no semantic
  branch delta before this planning edit.
- [x] (2026-07-02T00:00:00Z) Verified the locked local source for `markdown`
  1.0.0, `rstest` 0.26.1, `proptest` 1.11.0, and `insta` 1.48.0.
- [ ] Work item 1: Add successful operation golden fixtures.
- [ ] Work item 2: Add grammar-surface preservation fixtures.
- [ ] Work item 3: Add fidelity and contract fixtures.
- [ ] Work item 4: Add output-mode and fail-closed fixtures.
- [ ] Work item 5: Mark roadmap 3.1.1 complete after gates.

## Surprises & discoveries

- Memtrace `list_indexed_repositories` returned
  `user cancelled MCP tool call` during this planning round. Canonical-main
  graph context was unavailable, so this plan uses bounded branch-local
  evidence through `sem` and precise source inspection. This is not a blocker.
- `leta workspace add
  /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1 && leta files`
  returned `Error: IO error: Read-only file system (os error 30)`.
  A follow-up `leta files` returned `Error: Failed to start daemon`.
  Branch-local symbol navigation was unavailable, so this plan uses precise
  local source inspection. This is not a blocker.
- Firecrawl `firecrawl_scrape` for
  `https://docs.rs/markdown/1.0.0/markdown/fn.to_mdast.html` returned
  `user cancelled MCP tool call`. No load-bearing claim in this plan depends on
  inaccessible official documentation; crate behaviour is pinned to the locked
  local source listed in `Research evidence`.
- `cargo tree -i` shows locked versions that differ from manifest minimums
  under caret requirements: `markdown v1.0.0`, `rstest v0.26.1`,
  `proptest v1.11.0`, and `insta v1.48.0`.
- The current harness already supports explicit case metadata in
  `tests/golden/mod.rs`: command shape, target fixture, optional fragment,
  success or typed-failure expectation, and stdout or in-place output modes.
  Roadmap task 3.1.1 should use this harness instead of reworking it first.
- The current committed fixture corpus under `tests/fixtures/reference_rewrite/`
  covers section-reference preservation, version/quantity preservation,
  section-reference-outside-`Requires`, substring non-match, and multi-id
  `Requires` lists. It does not yet provide the complete operation,
  grammar-surface, addendum, output-mode, and fail-closed corpus required by
  roadmap task 3.1.1.

## Decision log

- Decision: Use the existing `tests/golden/mod.rs` harness and add cases
  through explicit metadata rather than replacing the harness.
  Rationale: Current `origin/main` already includes a typed harness with
  command and output-mode coverage. Reworking it would add risk before the
  missing fixture corpus exists.
  Date/Author: 2026-07-02 / Codex.

- Decision: Store new successful cases under
  `tests/fixtures/golden/<case-name>/` with `target.md`, optional
  `fragment.md`, and `expected.md`.
  Rationale: This directory shape is reviewable, works with exact Markdown
  comparison, and gives roadmap task 3.1.2 a straightforward conformant corpus
  to enumerate.
  Date/Author: 2026-07-02 / Codex.

- Decision: Use exact file comparisons, not `insta`, for the task 3.1.1
  corpus.
  Rationale: The design requires committed input-and-expected Markdown pairs.
  `insta` remains appropriate for existing stable CLI help snapshots, but these
  fixtures should be plain Markdown artefacts.
  Date/Author: 2026-07-02 / Codex.

- Decision: Cover F3/C5 in 3.1.1 with an exact identity replacement fixture,
  then leave generated no-op coverage to 3.1.2.
  Rationale: There is no public no-op CLI command. Replacing an item with
  byte-identical content exercises the same parse, operation, renumber,
  dependency-rewrite, and render pipeline and must produce byte-identical
  output.
  Date/Author: 2026-07-02 / Codex.

## Outcomes & retrospective

No implementation has started in this planning round. This section must be
updated after each work item with fixture counts, focused test evidence,
repository gate logs, and any lessons that affect roadmap tasks 3.1.2 or
3.1.3.

## Context and orientation

`mapsplice` edits constrained roadmap-shaped Markdown by parsing it into a
roadmap model, applying one operation, renumbering affected items, rewriting
dependency references, and rendering Markdown. The accepted grammar is
normative in `docs/users-guide.md`, section "The roadmap shape `mapsplice`
expects", and summarized in `docs/mapsplice-design.md` section 4.

The implementation surfaces relevant to this task are:

- `tests/roadmap_golden.rs`, the golden test binary currently registering the
  reference-rewrite cases.
- `tests/golden/mod.rs`, the shared fixture harness and private case metadata.
- `tests/fixtures/reference_rewrite/`, the existing golden Markdown pairs from
  roadmap task 1.1.3.
- `src/lib.rs::run_from_args` and `src/lib.rs::run_request`, the public
  workflow entry points used by integration tests.
- `src/fs.rs::rewrite_utf8`, the in-place write path that must preserve
  fail-closed semantics.
- `src/error.rs::MapspliceError`, the typed diagnostic surface for failure
  fixtures.
- `src/roadmap/ops/mod.rs::RoadmapOperation`, the domain operation surface.
- `src/roadmap/parse/mod.rs::parse_root`, which uses
  `markdown::to_mdast(markdown, &ParseOptions::gfm())`.
- `src/roadmap/render.rs` and `src/roadmap/render_table.rs`, the deterministic
  renderer paths that must preserve bodies, nested lists, tables, code blocks,
  and addendum sub-tasks.

The relevant source-of-truth requirements are:

- `docs/roadmap.md` section 3.1.1: add one input-and-expected fixture per
  operation and per guarantee, covering preamble, phases, steps, tasks,
  multi-line bodies, nested bullets, tables, and code blocks.
- `docs/mapsplice-design.md` section 4: accepted roadmap grammar and addendum
  sub-task level.
- `docs/mapsplice-design.md` section 5: fidelity guarantees F1 through F5.
- `docs/mapsplice-design.md` section 6: contract guarantees C1 through C6.
- `docs/mapsplice-design.md` section 7: dependency-reference model.
- `docs/mapsplice-design.md` section 8: golden corpus, required adversarial
  classes, test shapes, round-trip property, and regression discipline.
- `docs/developers-guide.md` sections 2, 3, and 6: architecture boundaries,
  public APIs, and verification layers.
- `docs/users-guide.md` sections "Command overview", "Output modes", and
  "Validation rules and failure cases": user-visible command and fail-closed
  semantics.
- `docs/execplans/initial-tool.md` sections "Scope and grammar assumptions",
  "Constraints", and "Proposed implementation": initial parser, operation,
  renumbering, and renderer decisions.
- `AGENTS.md` sections "Change Quality & Committing", "Rust Specific
  Guidance", "Testing", and "Markdown Guidance".
- `docs/documentation-style-guide.md` sections "Spelling", "Markdown rules",
  "Formatting", and "Roadmap task writing guidelines".

## Research evidence

Memtrace was requested first, but `list_indexed_repositories` returned
`user cancelled MCP tool call`. Firecrawl was also requested for official docs,
but `firecrawl_scrape` returned `user cancelled MCP tool call`. The evidence
below is therefore limited to branch-local repository files and locked crate
source. No work item depends on an unverified external API.

The locked `markdown` crate version is 1.0.0. Its local source at
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/lib.rs`
defines `pub fn to_mdast(value: &str, options: &ParseOptions) ->
Result<mdast::Node, message::Message>` at line 160. Its local source at
`.../markdown-1.0.0/src/configuration.rs` defines `ParseOptions::gfm()` at
line 1275 and documents that GFM extends CommonMark with autolink literals,
footnotes, strikethrough, tables, and tasklists. Therefore fixtures may rely on
the existing parser accepting GFM tables and task lists, but must not rely on
the crate for exact Markdown rendering.

The locked `rstest` crate version is 0.26.1. Its local source at
`.../rstest-0.26.1/src/lib.rs` documents fixture injection around lines 252-276
and re-exports `rstest_macros::fixture` at line 571. Existing tests already
use `#[fixture]`, `#[rstest]`, and named cases, so new golden cases should
extend that style.

The locked `proptest` crate version is 1.11.0. Its local source at
`.../proptest-1.11.0/src/collection.rs` defines `collection::vec` at line 205,
and `.../proptest-1.11.0/src/sugar.rs` defines `prop_compose!` at line 624.
Roadmap task 3.1.1 should not add a new property test, but the fixture layout
must remain easy for task 3.1.2 to enumerate.

The locked `insta` crate version is 1.48.0. Its local source at
`.../insta-1.48.0/src/macros.rs` defines `assert_snapshot!` at line 463, and
the runtime assertion path starts at `.../insta-1.48.0/src/runtime.rs` line
846. This task deliberately avoids new `insta` snapshots because committed
input-and-expected Markdown pairs are the required golden artefacts.

## Plan of work

### Work item 1: Add successful operation golden fixtures

Read these documents before editing: `docs/roadmap.md` section 3.1.1;
`docs/mapsplice-design.md` sections 4, 6, and 8; `docs/users-guide.md`
sections "Command overview", "`append`", "`insert`", "`delete`", "`replace`",
and "Output modes"; `docs/developers-guide.md` sections 2, 3, and 6; and
`AGENTS.md` sections "Rust Specific Guidance" and "Testing".

Load these skills for this work item: `leta`, `rust-router`,
`rust-unit-testing`, `domain-cli-and-daemons`, `sem`, and
`en-gb-oxendict-style`.

Add one successful stdout-mode golden case per operation shape under
`tests/fixtures/golden/` and register each case in `tests/roadmap_golden.rs`
using the existing `GoldenCase` metadata:

- `append_phase/` proves phase-level append, full renumbering, and stdout
  output.
- `insert_phase_before/` proves inserting a phase before an anchor.
- `insert_step_after/` proves `insert --after` at step level.
- `insert_task_before/` proves inserting a task before an anchor.
- `insert_sub_task_after/` proves inserting an addendum sub-task after an
  addendum anchor.
- `delete_task/` proves deleting one task and renumbering later tasks.
- `replace_step/` proves replacing one step with one or more sibling steps.
- `replace_sub_task/` proves replacing an addendum sub-task while preserving
  the parent task.

Each case directory must contain only paths that exist for that case:
`target.md`, `expected.md`, and `fragment.md` for operations that take a
fragment. The Rust metadata must state the exact command arguments; it must not
infer commands from directory names.

Tests to add or update:

- Unit tests: none unless a fixture exposes an isolated production defect.
- Behavioural tests: add named golden cases in `tests/roadmap_golden.rs`.
- Property tests: none in task 3.1.1.
- Snapshot tests: none; exact fixture comparison replaces new snapshots.
- End-to-end tests: every case must call `run_from_args` through the golden
  harness.

Validation for this work item:

```bash
cargo test --workspace --all-targets --all-features --test roadmap_golden \
  | tee /tmp/test-mapsplice-roadmap-3-1-1-operations.out
item=operations
changed_md=$(
  {
    git diff --name-only --diff-filter=ACMRT -- '*.md'
    git diff --cached --name-only --diff-filter=ACMRT -- '*.md'
    git ls-files --others --exclude-standard -- '*.md'
  } | sort -u
)
if test -n "$changed_md"; then
  printf '%s\n' "$changed_md" | xargs mdtablefix 2>&1 \
    | tee "/tmp/mdtablefix-mapsplice-roadmap-3-1-1-${item}.out"
  printf '%s\n' "$changed_md" | xargs markdownlint-cli2 --fix 2>&1 \
    | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
else
  : | tee "/tmp/mdtablefix-mapsplice-roadmap-3-1-1-${item}.out"
  : | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
fi
make all | tee /tmp/all-mapsplice-roadmap-3-1-1-operations.out
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-operations.out
make nixie | tee /tmp/nixie-mapsplice-roadmap-3-1-1-operations.out
sem diff --format json | tee /tmp/sem-mapsplice-roadmap-3-1-1-operations.out
```

Commit only after these commands pass.

### Work item 2: Add grammar-surface preservation fixtures

Read these documents before editing: `docs/roadmap.md` section 3.1.1;
`docs/mapsplice-design.md` sections 4, 5, and 8; `docs/users-guide.md`
section "The roadmap shape `mapsplice` expects";
`docs/documentation-style-guide.md` sections "Markdown rules" and
"Formatting"; and `AGENTS.md` section "Markdown Guidance".

Load these skills for this work item: `leta`, `rust-router`,
`rust-unit-testing`, `sem`, and `en-gb-oxendict-style`.

Add successful golden cases that each isolate one grammar-surface preservation
requirement while still using a real operation:

- `preamble_preserved/` proves optional preamble content before the first phase
  survives an operation unchanged.
- `phase_step_task_surface/` proves phases, steps, tasks, and task checklist
  markers render in the accepted grammar.
- `multi_line_task_body/` proves continuation paragraphs in a task body survive
  exactly.
- `nested_bullets/` proves ordinary nested bullets remain task body Markdown,
  not addendum sub-tasks.
- `tables_preserved/` proves a GFM table inside a task body renders
  deterministically.
- `code_blocks_preserved/` proves fenced code blocks, language tags, and code
  indentation survive exactly.
- `addendum_body_surface/` proves an addendum sub-task with its own body
  remains nested under its parent task.

Prefer the smallest operation that demonstrates preservation, usually deleting
or inserting an unrelated later task. Do not combine all grammar surfaces into
one fixture; each named case should fail with a narrow diff if a renderer path
regresses.

Tests to add or update:

- Unit tests: none unless a fixture exposes an isolated parser or renderer
  defect.
- Behavioural tests: add named golden cases in `tests/roadmap_golden.rs`.
- Property tests: none in task 3.1.1.
- Snapshot tests: none.
- End-to-end tests: every case must call `run_from_args`.

Validation for this work item:

```bash
cargo test --workspace --all-targets --all-features --test roadmap_golden \
  | tee /tmp/test-mapsplice-roadmap-3-1-1-grammar.out
item=grammar
changed_md=$(
  {
    git diff --name-only --diff-filter=ACMRT -- '*.md'
    git diff --cached --name-only --diff-filter=ACMRT -- '*.md'
    git ls-files --others --exclude-standard -- '*.md'
  } | sort -u
)
if test -n "$changed_md"; then
  printf '%s\n' "$changed_md" | xargs mdtablefix 2>&1 \
    | tee "/tmp/mdtablefix-mapsplice-roadmap-3-1-1-${item}.out"
  printf '%s\n' "$changed_md" | xargs markdownlint-cli2 --fix 2>&1 \
    | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
else
  : | tee "/tmp/mdtablefix-mapsplice-roadmap-3-1-1-${item}.out"
  : | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
fi
make all | tee /tmp/all-mapsplice-roadmap-3-1-1-grammar.out
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-grammar.out
make nixie | tee /tmp/nixie-mapsplice-roadmap-3-1-1-grammar.out
sem diff --format json | tee /tmp/sem-mapsplice-roadmap-3-1-1-grammar.out
```

Commit only after these commands pass.

### Work item 3: Add fidelity and contract fixtures

Read these documents before editing: `docs/mapsplice-design.md` section 5 for
F1 through F5, section 6 for C1 through C5, section 7 for the
dependency-reference model, and section 8 for required adversarial fixtures;
`docs/developers-guide.md` section 6; and `AGENTS.md` sections "Testing" and
"Error Handling".

Load these skills for this work item: `leta`, `rust-router`,
`rust-unit-testing`, `rust-verification`, `domain-cli-and-daemons`, `sem`, and
`en-gb-oxendict-style`. `rust-verification` is signposted because this item
sets the boundary between exact examples and the later generated property; it
does not require adding Kani, Verus, Miri, or proptest in task 3.1.1.

Add or register named cases so the golden corpus explicitly covers:

- `f1_content_preservation/`, proving unrelated content is byte-identical
  except for documented edit consequences.
- `f2_minimal_diff/`, proving the only changed numbers are addressed items,
  renumbered later items, and dependency references to those items.
- `f3_c5_identity_replace/`, proving the 3.1.1 F3/C5 acceptance criterion:
  replacing an addressed item with byte-identical content emits a complete
  roadmap byte-identical to `target.md`.
- `c2_renumber_contiguous/`, proving phases, steps, tasks, and addenda are
  contiguous after an edit.
- `c3_requires_rewrite/`, proving mapped `Requires` dependencies are rewritten
  and incidental numbers are preserved.
- `c4_addendum_contract/`, proving addendum sub-tasks renumber with their
  parent and preserve indentation.
- `adversarial_addendum_renumber/`, proving `8.2.3.1` tracks its parent task.
- `adversarial_addendum_render_fidelity/`, proving addendum nesting and
  indentation are preserved.

Keep the existing reference-rewrite fixtures covered: section-reference
preservation, section-reference-outside-`Requires`, version/quantity
preservation, substring non-match, and multi-id `Requires` lists. Do not claim
those existing files cover addendum renumber or addendum render fidelity; those
cases must be added explicitly here.

Tests to add or update:

- Unit tests: add only when a fixture exposes an isolated pure-function defect.
- Behavioural tests: add named golden cases for all listed contract fixtures.
- Property tests: none in task 3.1.1; ensure conformant success cases can be
  enumerated by a later property test without parsing command metadata.
- Snapshot tests: none.
- End-to-end tests: every success case must call `run_from_args`.

Validation for this work item:

```bash
cargo test --workspace --all-targets --all-features --test roadmap_golden \
  | tee /tmp/test-mapsplice-roadmap-3-1-1-contracts.out
item=contracts
changed_md=$(
  {
    git diff --name-only --diff-filter=ACMRT -- '*.md'
    git diff --cached --name-only --diff-filter=ACMRT -- '*.md'
    git ls-files --others --exclude-standard -- '*.md'
  } | sort -u
)
if test -n "$changed_md"; then
  printf '%s\n' "$changed_md" | xargs mdtablefix 2>&1 \
    | tee "/tmp/mdtablefix-mapsplice-roadmap-3-1-1-${item}.out"
  printf '%s\n' "$changed_md" | xargs markdownlint-cli2 --fix 2>&1 \
    | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
else
  : | tee "/tmp/mdtablefix-mapsplice-roadmap-3-1-1-${item}.out"
  : | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
fi
make all | tee /tmp/all-mapsplice-roadmap-3-1-1-contracts.out
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-contracts.out
make nixie | tee /tmp/nixie-mapsplice-roadmap-3-1-1-contracts.out
sem diff --format json | tee /tmp/sem-mapsplice-roadmap-3-1-1-contracts.out
```

Commit only after these commands pass.

### Work item 4: Add output-mode and fail-closed fixtures

Read these documents before editing: `docs/mapsplice-design.md` sections 5,
6, 7, and 8; `docs/users-guide.md` sections "Output modes" and "Validation
rules and failure cases"; `docs/developers-guide.md` sections 3, 4, and 6; and
`AGENTS.md` sections "Testing" and "Error Handling".

Load these skills for this work item: `leta`, `rust-router`,
`rust-unit-testing`, `rust-errors`, `domain-cli-and-daemons`, `sem`, and
`en-gb-oxendict-style`.

Add or register named cases that prove output and fail-closed behaviour:

- `c6_stdout_mode/` proves stdout mode emits the roadmap body and leaves the
  target file byte-identical to `target.md`.
- `c6_in_place_success/` proves `--in-place` emits no roadmap body on stdout,
  writes the target byte-identically to `expected.md`, and returns a
  `written_path`.
- `f5_in_place_failure_no_write/` proves an in-place failure emits no stdout,
  returns the expected typed `MapspliceError`, and leaves the target
  byte-identical to `target.md`.
- `dangling_requires_fails_closed/` proves a valid unresolved dependency
  reference reports `MapspliceError::DanglingDependency`, emits no stdout, and
  leaves the target unchanged.
- `adversarial_dangling_requires/` proves the required adversarial class from
  design section 8 explicitly, rather than relying on unrelated behavioural
  tests.

If the existing harness lacks a helper for asserting failure stdout is empty,
add the smallest private helper in `tests/golden/mod.rs` and cover it through
the new failure cases. Do not introduce a public API for fixture metadata.

Tests to add or update:

- Unit tests: add only when a fixture exposes an isolated error-shape defect.
- Behavioural tests: add named golden cases for all listed output and failure
  fixtures.
- Property tests: none in task 3.1.1.
- Snapshot tests: none.
- End-to-end tests: success cases and failure cases must call `run_from_args`;
  failure cases assert `MapspliceError` shape, empty stdout when applicable,
  and unchanged target file.

Validation for this work item:

```bash
cargo test --workspace --all-targets --all-features --test roadmap_golden \
  | tee /tmp/test-mapsplice-roadmap-3-1-1-output-fail.out
item=output-fail
changed_md=$(
  {
    git diff --name-only --diff-filter=ACMRT -- '*.md'
    git diff --cached --name-only --diff-filter=ACMRT -- '*.md'
    git ls-files --others --exclude-standard -- '*.md'
  } | sort -u
)
if test -n "$changed_md"; then
  printf '%s\n' "$changed_md" | xargs mdtablefix 2>&1 \
    | tee "/tmp/mdtablefix-mapsplice-roadmap-3-1-1-${item}.out"
  printf '%s\n' "$changed_md" | xargs markdownlint-cli2 --fix 2>&1 \
    | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
else
  : | tee "/tmp/mdtablefix-mapsplice-roadmap-3-1-1-${item}.out"
  : | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
fi
make all | tee /tmp/all-mapsplice-roadmap-3-1-1-output-fail.out
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-output-fail.out
make nixie | tee /tmp/nixie-mapsplice-roadmap-3-1-1-output-fail.out
sem diff --format json | tee /tmp/sem-mapsplice-roadmap-3-1-1-output-fail.out
```

Commit only after these commands pass.

### Work item 5: Mark roadmap 3.1.1 complete after gates

Read these documents before editing: `docs/roadmap.md` section 3.1.1,
`docs/documentation-style-guide.md` section "Roadmap task writing guidelines",
and `AGENTS.md` section "Markdown Guidance".

Load these skills for this work item: `leta`, `sem`, `execplans`, and
`en-gb-oxendict-style`.

After work items 1 through 4 are committed and gated, update only
`docs/roadmap.md` to mark task 3.1.1 complete. Then update this ExecPlan's
`Progress`, `Decision Log`, and `Outcomes & Retrospective` sections with final
fixture counts, gate log paths, and follow-up notes for tasks 3.1.2 and 3.1.3.
Do not mark 3.1.2 or 3.1.3 complete.

Tests to add or update:

- Unit tests: none.
- Behavioural tests: none beyond rerunning the full golden corpus.
- Property tests: none.
- Snapshot tests: none.
- End-to-end tests: rerun the golden harness and repository gates.

Validation for this work item:

```bash
cargo test --workspace --all-targets --all-features --test roadmap_golden \
  | tee /tmp/test-mapsplice-roadmap-3-1-1-final.out
item=final
changed_md=$(
  {
    git diff --name-only --diff-filter=ACMRT -- '*.md'
    git diff --cached --name-only --diff-filter=ACMRT -- '*.md'
    git ls-files --others --exclude-standard -- '*.md'
  } | sort -u
)
if test -n "$changed_md"; then
  printf '%s\n' "$changed_md" | xargs mdtablefix 2>&1 \
    | tee "/tmp/mdtablefix-mapsplice-roadmap-3-1-1-${item}.out"
  printf '%s\n' "$changed_md" | xargs markdownlint-cli2 --fix 2>&1 \
    | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
else
  : | tee "/tmp/mdtablefix-mapsplice-roadmap-3-1-1-${item}.out"
  : | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
fi
make all | tee /tmp/all-mapsplice-roadmap-3-1-1-final.out
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-final.out
make nixie | tee /tmp/nixie-mapsplice-roadmap-3-1-1-final.out
sem diff --format json | tee /tmp/sem-mapsplice-roadmap-3-1-1-final.out
```

Commit only after these commands pass.

## Concrete steps

Begin every implementation session from the assigned worktree:

```bash
cd /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1
git branch --show-current | tee /tmp/branch-mapsplice-roadmap-3-1-1-preflight.out
sem diff --from origin/main --to HEAD --format json \
  | tee /tmp/sem-mapsplice-roadmap-3-1-1-preflight.out
```

Expected preflight before implementation starts:

```plaintext
roadmap-3-1-1
{"summary":{"fileCount":1,...
```

After this ExecPlan is committed, the only expected semantic branch delta
before implementation is `docs/execplans/roadmap-3-1-1.md`. If additional
files are present, inspect them and update this plan before editing.

For each work item:

1. Update this ExecPlan's `Progress` entry before starting the item.
2. Add the red fixture or narrow harness assertion first and run the focused
   `cargo test --workspace --all-targets --all-features --test roadmap_golden`
   command with `tee`.
3. If a new fixture is meant to expose a production defect, record the expected
   failure in `Surprises & Discoveries` before changing production code.
4. Make the smallest source or fixture changes needed for the focused command
   to pass.
5. Format changed Markdown files only using the path-safe changed-file recipe
   from the relevant work item.
6. Run `make all`, `make markdownlint`, and `make nixie` with `tee`.
7. Use `sem diff --format json` with `tee` to review entity-level changes
   before commit.
8. Commit the work item with an imperative subject and explanatory body.

## Validation and acceptance

The required repository validation commands for the completed task are:

```bash
make all | tee /tmp/all-mapsplice-roadmap-3-1-1.out
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1.out
make nixie | tee /tmp/nixie-mapsplice-roadmap-3-1-1.out
```

`make all` is required because current `origin/main` includes the `typecheck`
target in the aggregate gate. `make markdownlint` and `make nixie` are required
because the task changes Markdown fixtures, roadmap status, and this ExecPlan.

The focused acceptance command is:

```bash
cargo test --workspace --all-targets --all-features --test roadmap_golden \
  | tee /tmp/test-mapsplice-roadmap-3-1-1.out
```

Acceptance criteria:

- `tests/fixtures/golden/` contains named input-and-expected cases for every
  supported operation: append, insert before, insert after, delete, and
  replace.
- The corpus covers the grammar surface named in roadmap task 3.1.1: preamble,
  phases, steps, tasks, multi-line bodies, nested bullets, tables, and code
  blocks.
- The corpus covers design guarantees F1 through F5 and C1 through C6 where
  those guarantees have observable output or failure behaviour.
- F3/C5 are not merely "ready" for 3.1.2: `f3_c5_identity_replace/` must be an
  exact 3.1.1 golden comparison whose expected output is byte-identical to the
  target.
- C6 and F5 include stdout target-unchanged, in-place success, and in-place
  failure/no-write assertions.
- Existing reference-rewrite adversarial fixtures remain covered, and the
  missing addendum renumber, addendum render-fidelity, and dangling-`Requires`
  adversarial classes are added explicitly.
- Each golden comparison is exact and includes the fixture name in failure
  messages.
- No new dependency is added.
- `docs/roadmap.md` marks only task 3.1.1 complete after all gates pass.

## Idempotence and recovery

The fixture-creation steps are additive and can be rerun safely. If a fixture
directory is partially created, either finish all required files in that
directory before running tests or delete the incomplete directory before
committing. Do not leave empty fixture directories.

Formatter commands operate only on currently changed Markdown files that still
exist in the worktree. They combine unstaged tracked paths, staged paths, and
untracked paths:

```bash
changed_md=$(
  {
    git diff --name-only --diff-filter=ACMRT -- '*.md'
    git diff --cached --name-only --diff-filter=ACMRT -- '*.md'
    git ls-files --others --exclude-standard -- '*.md'
  } | sort -u
)
```

The `ACMRT` filters exclude deleted tracked paths, and
`git ls-files --others --exclude-standard -- '*.md'` adds new fixture Markdown
before it is staged. If formatter output touches unrelated Markdown files,
inspect the diff and use a named stash with `kind=discard` for unrelated churn
before continuing.

If a golden fixture reveals a renderer, parser, operation, or error-shape
defect, keep the failing fixture, add the smallest production fix, and rerun
the focused golden test before broad gates. If the fix exceeds the tolerances
above, stop and revise this plan rather than widening the change silently.

## Artifacts and notes

Planning commands and observed evidence:

```plaintext
$ pwd && git branch --show-current
/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1
roadmap-3-1-1

$ sem diff --from origin/main --to HEAD --format json
{"summary":{"fileCount":0,"added":0,"modified":0,"deleted":0,"moved":0,"renamed":0,"reordered":0,"orphan":0,"total":0},"changes":[]}

$ cargo tree -i markdown
markdown v1.0.0
└── mapsplice v0.1.0 (/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1)

$ cargo tree -i rstest
rstest v0.26.1
[dev-dependencies]
└── mapsplice v0.1.0 (/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1)
```

Tooling failures recorded for this planning round:

```plaintext
mcp__memtrace.list_indexed_repositories -> user cancelled MCP tool call
leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1
  && leta files -> Error: IO error: Read-only file system (os error 30)
leta files -> Error: Failed to start daemon
mcp__firecrawl.firecrawl_scrape -> user cancelled MCP tool call
```

## Interfaces and dependencies

Do not add new dependencies.

Use the existing public test-facing entry point:

```rust
pub fn run_from_args<I, T>(args: I) -> Result<RunOutcome>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone;
```

Use the existing operation vocabulary exposed by the test harness:

```rust
pub(crate) enum GoldenCommand {
    Append,
    InsertBefore { anchor: &'static str },
    InsertAfter { anchor: &'static str },
    Delete { anchor: &'static str },
    Replace { anchor: &'static str },
}
```

Use the existing fixture directory contract:

```plaintext
tests/fixtures/golden/<case-name>/target.md
tests/fixtures/golden/<case-name>/fragment.md
tests/fixtures/golden/<case-name>/expected.md
```

`fragment.md` exists only for append, insert, and replace cases. Failure cases
may use an expected error enum in Rust metadata rather than an output file when
there is no successful rendered body.

## Revision note

2026-07-02 first planning-round revision: replaced stale implementation-round
state with a clean DRAFT plan based on current `origin/main`, where the golden
harness already exists and the branch has no semantic delta before this
ExecPlan edit. The revised plan keeps all work items implementable, cites the
governing documents and skills per item, records Memtrace/Leta/Firecrawl
tooling failures as non-blocking evidence, and uses path-safe validation
commands.
