# Share task and sub-task checklist parsing

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

This is planning round 2. Do not begin implementation until the plan is
approved by the df12-build roadmap workflow.

## Purpose / big picture

Roadmap task 5.1.1, "Share task and sub-task checklist parsing", reduces drift
inside the roadmap parser. The remaining implementation gap is checklist-head
parsing: task items and sub-task items still separately check the GFM checklist
marker, locate the first paragraph, and slice the remaining child body.
Numbered-prefix parsing is already shared on `origin/main`; the current source
routes both `parse_task_paragraph` and `parse_sub_task_paragraph` through
`parse_numbered_paragraph`, which calls `split_numbered_prefix`.

After this change, task and sub-task item parsing use one private
checklist-head helper, while keeping their own typed numbers, parent
validation, body parsing, and exact user-facing diagnostics. Observable
behaviour must not change: parse diagnostics, golden fixtures, CLI output,
public library APIs, and rendered Markdown remain compatible with the current
implementation.

## Constraints

- Work only inside
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-1`.
- Do not edit the root/control worktree.
- Treat `origin/main` as canonical and `docs/roadmap.md` as the roadmap source
  of truth.
- Implement only roadmap task 5.1.1 from `docs/roadmap.md`:
  "Extract common checklist-head and numbered-prefix parsing helpers for task
  and sub-task items while preserving current diagnostics."
- Record in this plan that the numbered-prefix half is already satisfied by
  `src/roadmap/parse/mod.rs:315-354`: the typed wrappers delegate to
  `parse_numbered_paragraph`, and that helper calls `split_numbered_prefix`.
- Preserve `docs/mapsplice-design.md` section 2, "Non-negotiable
  constraints": parsing remains mdast-based through the locked `markdown` crate
  and edits run through the roadmap model rather than raw-string surgery.
- Preserve `docs/mapsplice-design.md` section 4, "The roadmap grammar
  (normative reference)": tasks are third-level numbered checklist items and
  addendum sub-tasks are fourth-level nested numbered checklist items.
- Preserve `docs/mapsplice-design.md` section 5, "Fidelity guarantees",
  especially F1 content preservation, F3 round-trip stability, F4 gate-clean
  output, and F5 fail-closed behaviour.
- Preserve `docs/mapsplice-design.md` section 6, "Functional and contract
  guarantees", especially C2 contiguous renumbering and C4 first-class addendum
  sub-tasks.
- Preserve `docs/developers-guide.md` section 2, "Architecture boundaries":
  `src/roadmap` owns domain parsing, mutation, renumbering, and rendering.
- Preserve `docs/developers-guide.md` section 3, "Public library APIs": do not
  add, remove, or rename public parser APIs or public error variants.
- Follow `docs/developers-guide.md` section 6, "Verification layers": use
  `rstest` unit tests for parser behaviour and keep property tests for input
  gaps rather than small structural refactors.
- Preserve `docs/users-guide.md`, "The roadmap shape `mapsplice` expects" and
  "Validation rules and failure cases".
- Follow `docs/documentation-style-guide.md`: prose must use en-GB Oxford
  spelling, sentence-case headings, fenced code languages, and 80-column
  wrapping.
- Do not add a new external dependency. Use the locked crates already in
  `Cargo.lock`.
- Keep every Rust source file under 400 lines. `src/roadmap/parse/mod.rs` is
  already exactly 400 lines, so Work Item 2 must create
  `src/roadmap/parse/checklist.rs` instead of adding helper bodies to `mod.rs`.
- Every new Rust module must begin with a module-level `//!` comment.
- Use Red-Green-Refactor. Because this is a behaviour-preserving refactor, the
  red stage is a mutation-style proof: temporarily break the expected
  diagnostic or helper route, confirm the focused test fails for the intended
  reason, and revert the temporary mutation before committing.
- Format only changed Markdown files. Do not run repository-global Markdown
  formatters such as `make fmt` or `mdformat-all`.
- Keep this ExecPlan current after every work item. Every work item therefore
  includes scoped Markdown formatting plus `make markdownlint` and `make nixie`.
- Run tests, lint, and formatting gates sequentially with `tee` logs under
  `/tmp`. Do not run test, lint, or format gates in parallel.
- Use the shared Cargo cache. Do not create an isolated Cargo cache.

## Tolerances

- Stop and escalate if implementation requires a public API signature change,
  a new public error variant, a new crate, or any change to accepted roadmap
  grammar.
- The planned implementation may touch only these files unless a focused test
  proves a necessary adjacent change: `docs/execplans/roadmap-5-1-1.md`,
  `docs/roadmap.md`, `src/roadmap/parse/mod.rs`,
  `src/roadmap/parse/checklist.rs`, `src/roadmap/parse/sub_task_fragment.rs`,
  `src/roadmap/parse/sub_task_body.rs`, `tests/roadmap_parse.rs`, and
  `tests/roadmap_sub_tasks.rs`.
- Stop and escalate if implementation needs changes outside `src/roadmap/parse`
  or parser tests, excluding the living ExecPlan and roadmap status update.
- Stop and escalate if exact diagnostic text changes for existing malformed
  task or sub-task inputs.
- Stop and escalate if a shared helper would force task fragments and
  sub-task fragments to share fragment-root diagnostics. Roadmap task 5.1.2
  owns fragment-root validation; this task must not absorb it.
- Stop and escalate if the net production-code diff exceeds 160 lines, or if
  any Rust source file exceeds 400 lines after the mandatory module split.
- Stop and escalate if the same focused parser test still fails after three
  implementation attempts.
- Do not treat advisory-tool unavailability as a blocker. Record the failed
  command or MCP call and continue with bounded local source, tests, and
  documentation evidence.

## Risks

- Risk: the helper could accidentally homogenize diagnostics that are currently
  level-specific. Severity: medium. Likelihood: medium. Mitigation: Work Item 1
  pins exact task and sub-task diagnostics before production refactoring.
- Risk: moving checklist logic to a new module could accidentally broaden the
  parse module's private surface. Severity: low. Likelihood: medium.
  Mitigation: keep the new module private under `src/roadmap/parse/`, expose
  only `pub(super)` helper functions or private types needed by `mod.rs`, and
  keep public library APIs unchanged.
- Risk: a broad parser abstraction could drift into roadmap task 5.1.2 by also
  consolidating fragment-root parsing. Severity: medium. Likelihood: medium.
  Mitigation: keep this task to checklist item heads. Leave
  `parse_task_fragment_root` and `parse_sub_task_fragment_root` skeleton
  duplication for 5.1.2.
- Risk: property testing is tempting but not the smallest useful verification
  layer for this task. Severity: low. Likelihood: medium. Mitigation: use
  `rstest` table tests for the finite diagnostic matrix. Keep `proptest` in
  reserve only if a generated input-space gap appears during implementation.
- Risk: advisory tools may remain unavailable in the implementation session.
  Severity: low. Likelihood: high. Mitigation: retry Memtrace and Leta before
  editing, record exact failures if they persist, and proceed with bounded
  local source inspection and tests.

## Progress

- [x] (2026-07-03T05:54:50Z) Confirmed the current branch is
  `roadmap-5-1-1`, so this plan is `docs/execplans/roadmap-5-1-1.md`.
- [x] (2026-07-03T05:54:50Z) Loaded the first-round planning skills recorded
  in the previous draft: `execplans`, `leta`, `rust-router`,
  `rust-unit-testing`, `rust-types-and-apis`, `rust-errors`,
  `rust-verification`, `proptest`, `sem`, `firecrawl-mcp`, and `commit-message`.
- [x] (2026-07-03T05:54:50Z) First-round Memtrace discovery failed with
  `mcp__memtrace.list_indexed_repositories -> user cancelled MCP tool call`.
- [x] (2026-07-03T05:54:50Z) First-round Leta setup failed with
  `Error: IO error: Read-only file system (os error 30)` and then
  `Error: Failed to start daemon` when retried with XDG paths under `/tmp`.
- [x] (2026-07-03T05:54:50Z) First-round Firecrawl docs.rs scrape attempts
  returned `user cancelled MCP tool call`; local locked source was used for
  crate evidence.
- [x] (2026-07-03T05:54:50Z) Wrote and gated the first-round ExecPlan.
- [x] (2026-07-03T09:41:00Z) Loaded the round-2 required skills:
  `execplans`, `leta`, `rust-router`, `rust-types-and-apis`,
  `rust-unit-testing`, and `en-gb-oxendict-style`.
- [x] (2026-07-03T09:41:00Z) Retried Leta workspace setup with
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-1`;
  it failed with `Error: IO error: Read-only file system (os error 30)`.
- [x] (2026-07-03T09:41:00Z) Retried Memtrace discovery with
  `mcp__memtrace.list_indexed_repositories`; the MCP host returned
  `user cancelled MCP tool call`.
- [x] (2026-07-03T09:41:00Z) Retried Firecrawl verification for the official
  docs.rs `markdown::to_mdast` page; the MCP host returned
  `user cancelled MCP tool call`.
- [x] (2026-07-03T09:41:00Z) Used `sem entities src/roadmap/parse/mod.rs` to
  verify `parse_task_paragraph`, `parse_sub_task_paragraph`,
  `parse_numbered_paragraph`, and `split_numbered_prefix` are current entities
  in one file.
- [x] (2026-07-03T09:41:00Z) Verified `src/roadmap/parse/mod.rs` has exactly
  400 lines, making the `checklist.rs` split mandatory.
- [x] (2026-07-03T09:41:00Z) Revised the plan to resolve the round-2 blocking
  design-review points.
- [x] (2026-07-03T06:30:27Z) Work Item 1 pinned reachable task and sub-task
  checklist-head diagnostics in `tests/roadmap_parse.rs`. Focused red proof
  used a temporary expected-string mutation and failed in
  `/tmp/test-mapsplice-roadmap-5-1-1-item-1-red.out`; the corrected focused
  test passed in `/tmp/test-mapsplice-roadmap-5-1-1-item-1.out`. Deterministic
  confirmation gates passed in `/tmp/all-mapsplice-roadmap-5-1-1.out`,
  `/tmp/markdownlint-mapsplice-roadmap-5-1-1.out`, and
  `/tmp/nixie-mapsplice-roadmap-5-1-1.out`.
- [x] (2026-07-03T06:41:00Z) Work Item 2 added
  `src/roadmap/parse/checklist.rs` and routed `parse_task_item` and
  `parse_sub_task_item_unchecked` through `parse_checklist_item_head`.
  Helper-level unit coverage pins the checked-item-without-paragraph branch.
  Focused parser, sub-task, golden, helper, line-count, and semantic-diff
  evidence was written to `/tmp/test-mapsplice-roadmap-5-1-1-item-2.out`,
  `/tmp/test-sub-tasks-mapsplice-roadmap-5-1-1-item-2.out`,
  `/tmp/test-golden-mapsplice-roadmap-5-1-1-item-2.out`,
  `/tmp/test-helper-mapsplice-roadmap-5-1-1-item-2.out`,
  `/tmp/wc-mapsplice-roadmap-5-1-1-item-2.out`, and
  `/tmp/sem-diff-mapsplice-roadmap-5-1-1-item-2.out`. `make all` passed in
  `/tmp/all-mapsplice-roadmap-5-1-1.out`.
- [x] (2026-07-03T06:47:35Z) Work Item 3 marked
  `docs/roadmap.md` task 5.1.1 complete after Work Items 1 and 2 passed their
  deterministic gates. Final `make all`, `make markdownlint`, and `make nixie`
  passed in `/tmp/all-mapsplice-roadmap-5-1-1.out`,
  `/tmp/markdownlint-mapsplice-roadmap-5-1-1.out`, and
  `/tmp/nixie-mapsplice-roadmap-5-1-1.out`. CodeRabbit was attempted once and
  deferred in `/tmp/coderabbit-mapsplice-roadmap-5-1-1.out`.

## Surprises & discoveries

- Observation: numbered-prefix parsing is already shared on the current
  branch and on `origin/main` according to the design-review evidence. Evidence:
  `src/roadmap/parse/mod.rs:315-334` has thin typed wrappers,
  `src/roadmap/parse/mod.rs:336-352` implements `parse_numbered_paragraph`, and
  `src/roadmap/parse/mod.rs:354-368` implements the shared
  `split_numbered_prefix` path used by numbered paragraphs and list-shape
  detection. Impact: there is no separate numbered-prefix extraction work item.
  The only residual 5.1.1 production change is checklist-head sharing.
- Observation: `src/roadmap/parse/mod.rs` is already at the 400-line file cap.
  Evidence: `wc -l src/roadmap/parse/mod.rs` returned `400`. Impact: Work Item
  2 must create `src/roadmap/parse/checklist.rs` and keep `mod.rs` at or below
  400 lines.
- Observation: Memtrace was unavailable in both planning rounds. Evidence:
  `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`. Impact: this plan records bounded local and
  `sem` evidence instead. This is not a product blocker under the task
  instructions.
- Observation: Leta could not initialize in this sandbox. Evidence:
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-1`
  returned `Error: IO error: Read-only file system (os error 30)`. Impact: this
  plan uses `sem` plus precise local source inspection. The implementer should
  retry Leta before editing and record the result here.
- Observation: Firecrawl could not fetch official docs.rs pages in this
  planning session. Evidence: `mcp__firecrawl.firecrawl_scrape` for
  `https://docs.rs/markdown/1.0.0/markdown/fn.to_mdast.html` returned
  `user cancelled MCP tool call`. Impact: load-bearing crate behaviour is
  pinned to locked local Cargo registry source, and the plan does not rely on a
  network-only claim.
- Observation: a GFM task-list marker without following paragraph text is not
  surfaced by the locked `markdown` crate as `ListItem.checked = Some(_)`.
  Evidence: temporary mdast inspection during Work Item 1 showed `- [ ]` with
  only nested body content was parsed as unchecked text `[ ]`, so public parser
  entry points report the checklist-marker diagnostic before any
  first-child-not-paragraph branch can be reached. Impact: Work Item 1 pins
  public diagnostics and Work Item 2 must add helper-level unit coverage for
  the non-paragraph branch using constructed mdast values.
- Observation: CodeRabbit review was deferred for Work Item 1. Evidence:
  `/tmp/coderabbit-mapsplice-roadmap-5-1-1.out` contains this exact line:

  ```json
  {"type":"status","phase":"deferred","status":"deferred coderabbit review: no default network route visible in this sandbox"}
  ```

  Impact: there are no actionable CodeRabbit findings for Work Item 1; the
  deferred review remains an open issue for the supervisor.
- Observation: the mandatory checklist module kept both parser source files
  under the repository file-size cap. Evidence:
  `/tmp/wc-mapsplice-roadmap-5-1-1-item-2.out` reports
  `370 src/roadmap/parse/mod.rs` and `102 src/roadmap/parse/checklist.rs`.
  Impact: the helper split satisfies the 400-line Rust source constraint.
- Observation: CodeRabbit review was deferred again for Work Item 2. Evidence:
  `/tmp/coderabbit-mapsplice-roadmap-5-1-1.out` contains the same no-default
  network route status as Work Item 1. Impact: there are no actionable
  CodeRabbit findings for Work Item 2; the deferred review remains an open
  issue for the supervisor.
- Observation: CodeRabbit review was deferred again for Work Item 3. Evidence:
  `/tmp/coderabbit-mapsplice-roadmap-5-1-1.out` contains the same no-default
  network route status and `__EXIT_STATUS__:124`. Impact: every CodeRabbit
  attempt for this task was deferred by sandbox networking before review
  findings could be produced.

## Decision log

- Decision: treat numbered-prefix sharing as current baseline, not a future
  deliverable. Rationale: the existing source already performs the four
  behaviours the previous Work Item 3 described: require first text, call
  `split_numbered_prefix`, verify level, and replace the first cloned text node
  with the remainder. Date/Author: 2026-07-03, Codex.
- Decision: make `src/roadmap/parse/checklist.rs` mandatory. Rationale:
  `src/roadmap/parse/mod.rs` is already 400 lines, and `AGENTS.md` caps Rust
  source files at 400 lines. A novice following the plan must not be directed
  to add helper bodies to an already capped file. Date/Author: 2026-07-03,
  Codex.
- Decision: include scoped Markdown formatting, `make markdownlint`, and
  `make nixie` in every work-item validation block. Rationale: this ExecPlan is
  a living document and is updated after every work item, so each commit
  changes Markdown. Date/Author: 2026-07-03, Codex.
- Decision: implement this as a behaviour-preserving parser refactor with
  focused diagnostic tests first. Rationale: `docs/roadmap.md` task 5.1.1 names
  consolidation and unchanged diagnostics as the success criteria. The design
  document already defines the grammar, so the implementation must not expand
  it. Date/Author: 2026-07-03, Codex.
- Decision: use a small private checklist helper instead of a generic parser
  framework. Rationale: `rust-types-and-apis` favours concrete, narrow APIs
  until abstraction pressure is real. The only remaining shared behaviour is
  checklist item head extraction. Date/Author: 2026-07-03, Codex.
- Decision: keep fragment-root consolidation out of this task. Rationale:
  `docs/roadmap.md` task 5.1.2 explicitly owns sharing fragment-root validation
  and step accumulation. Date/Author: 2026-07-03, Codex.
- Decision: use locked `markdown` 1.0.0 mdast fields directly, not a new
  Markdown parser or raw-string scanner. Rationale: `docs/mapsplice-design.md`
  section 2 requires mdast-based parsing, `Cargo.lock` resolves `markdown` to
  1.0.0, and local source shows the needed fields and GFM task-list handling.
  Date/Author: 2026-07-03, Codex.
- Decision: use existing `rstest` 0.26.1 for the diagnostic matrix and do not
  add property tests unless implementation discovers an input-space gap.
  Rationale: `docs/developers-guide.md` section 6 routes parser behaviour to
  `rstest` unit tests; this task is a finite refactor with exact diagnostics.
  Date/Author: 2026-07-03, Codex.
- Decision: pin non-paragraph checklist-head diagnostics at the helper level in
  Work Item 2 instead of forcing an unreachable public-parser fixture in Work
  Item 1. Rationale: the locked GFM parser does not produce a checked list item
  without paragraph content from Markdown text, so a public test would pin a
  different checklist-marker diagnostic rather than the intended branch.
  Date/Author: 2026-07-03, Codex.
- Decision: keep the checklist helper private to `src/roadmap/parse` and pass a
  small `ChecklistKind` value for task-specific diagnostics. Rationale: this
  single-sources checklist head extraction without changing public parser APIs
  or merging numbered-prefix, parent-validation, or body-parsing concerns.
  Date/Author: 2026-07-03, Codex.

## Outcomes & retrospective

Work Item 1 is complete. It added `rstest` coverage for task and sub-task
checklist-head diagnostics, wrong task/sub-task prefix levels, wrong sub-task
parents, and out-of-order sub-tasks. The focused green command reported
`10 passed; 0 failed` in `/tmp/test-mapsplice-roadmap-5-1-1-item-1.out`.
`make all`, `make markdownlint`, and `make nixie` passed before commit. The
CodeRabbit pass was attempted once and deferred because the sandbox has no
default network route; no review findings were produced.

Work Item 2 is complete. The new private `checklist` module returns checked
state, first paragraph, and the remaining child-body slice for both task and
sub-task item parsers. Numbered-prefix parsing still flows through
`parse_numbered_paragraph` and `split_numbered_prefix`, and fragment-root
validation remains untouched for task 5.1.2. `make all` passed before the
ExecPlan update, and CodeRabbit was attempted once but deferred because the
sandbox has no default network route.

Work Item 3 is complete. `docs/roadmap.md` now marks task 5.1.1 complete. Final
`make all`, `make markdownlint`, and `make nixie` passed at HEAD. All three
CodeRabbit attempts for Work Items 1, 2, and 3 were deferred with the same
no-default-network-route status, so the only remaining gap is external advisory
review in a network-enabled environment.

## Context and orientation

The parser lives under `src/roadmap/parse/`.

- `src/roadmap/parse/mod.rs` exposes `parse_roadmap`, shared heading and
  numbered-prefix helpers, `parse_task_list`, `parse_task_item`,
  `parse_sub_task_item_unchecked`, `parse_numbered_paragraph`,
  `split_numbered_prefix`, `looks_like_task_list`, and
  `looks_like_sub_task_list`.
- `src/roadmap/parse/checklist.rs` does not exist yet. Work Item 2 creates it
  as the mandatory home for shared checklist-head parsing.
- `src/roadmap/parse/fragment.rs` chooses fragment level and currently keeps
  separate single-list roots for task and sub-task fragments. This file should
  not be touched for 5.1.1 unless imports must be adjusted after the checklist
  split.
- `src/roadmap/parse/sub_task_fragment.rs` parses top-level sub-task fragment
  lists with fragment-specific messages.
- `src/roadmap/parse/sub_task_body.rs` rejects nested roadmap-shaped lists
  inside sub-task bodies.
- `src/roadmap/parse/task_children.rs` preserves ordered task children while
  separating body blocks from first-class sub-tasks.
- `tests/roadmap_parse.rs` already pins top-level anchor and fragment parsing
  diagnostics.
- `tests/roadmap_sub_tasks.rs` already pins structural sub-task behaviour and
  malformed nested-roadmap cases.

The current duplicated checklist-head skeleton is:

1. Reject ordered lists at the list level.
2. Reject list items without a GFM checklist marker by checking
   `ListItem.checked`.
3. Require the first child to be a `Paragraph`.
4. Parse the numbered paragraph through the already shared
   `parse_numbered_paragraph` path.
5. Parse the rest of the list item children as either task body plus sub-tasks,
   or as sub-task body.

The helper shape should express steps 2, 3, and the child-body slice without
deciding parent task membership, sibling ordering, numbered-prefix parsing, or
body parsing.

## Interfaces and dependencies

Create a private module and wire it from `src/roadmap/parse/mod.rs`:

```rust
mod checklist;
```

The module should define a private data carrier with this shape:

```rust
pub(super) struct ChecklistItemHead<'item> {
    pub(super) checked: Option<bool>,
    pub(super) paragraph: &'item Paragraph,
    pub(super) child_body: &'item [Node],
}
```

Use a small private enum or equivalent helper argument to preserve exact
messages:

```rust
pub(super) enum ChecklistKind {
    Task,
    SubTask,
}
```

The exact names may change, but the resulting API must remain private to
`src/roadmap/parse`. The helper must not escape through `src/lib.rs` or
`src/roadmap/mod.rs`.

The plan relies on these locked dependency facts:

- `Cargo.lock:943-946` resolves `markdown` to version 1.0.0.
- `Cargo.lock:1363-1366` resolves `rstest` to version 0.26.1.
- `Cargo.lock:1439-1442` resolves `rstest_macros` to version 0.26.1.
- Locked source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/mdast.rs`
  defines `Paragraph.children`, `List.children`, `List.ordered`,
  `ListItem.children`, `ListItem.checked`, and `Text.value`.
- Locked source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/configuration.rs`
  defines `ParseOptions::gfm()` and `Constructs::gfm()` with
  `gfm_task_list_item: true`.
- Locked source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/to_mdast.rs`
  sets `ListItem.checked = Some(checked)` when a GFM task-list item value is
  encountered.
- Locked source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/lib.rs`
  exposes `to_mdast(value, &ParseOptions) -> Result<mdast::Node, _>`.
- Locked `rstest` 0.26.1 README and source docs show `#[rstest]` and
  `#[case]` for table-driven tests.

## Plan of work

### Work Item 1: Pin checklist-head diagnostics

Docs and skills to read before editing: `AGENTS.md`; `docs/roadmap.md` section
5.1.1; `docs/mapsplice-design.md` sections 2, 4, 5, 6, and 8;
`docs/developers-guide.md` sections 2, 3, and 6; `docs/users-guide.md`, "The
roadmap shape `mapsplice` expects" and "Validation rules and failure cases";
`docs/documentation-style-guide.md`; `execplans`; `leta`; `rust-router`;
`rust-unit-testing`; `rust-errors`; `sem`; `en-gb-oxendict-style`.

Add focused `rstest` cases in `tests/roadmap_parse.rs` that assert existing
diagnostics for malformed task and sub-task checklist heads. Cover at least:

- ordered top-level task list:
  `roadmap task lists must be unordered checklist items`;
- unordered non-checklist task list item:
  `roadmap task lists must be unordered checklist items`;
- task list item whose first child is not a paragraph:
  `task list items must start with a paragraph`;
- task paragraph whose first child is not plain text:
  `task paragraphs must start with plain text`;
- task item with a fourth-level number in a task context:
  `expected a task prefix in ...`;
- ordered nested sub-task list:
  `roadmap sub-task lists must be unordered checklist items`;
- unordered non-checklist nested sub-task item:
  `roadmap sub-task lists must be unordered checklist items`;
- sub-task list item whose first child is not a paragraph:
  `sub-task list items must start with a paragraph`;
- sub-task paragraph whose first child is not plain text:
  `sub-task paragraphs must start with plain text`;
- sub-task item with a third-level number in a sub-task context:
  `expected a sub-task prefix in ...`;
- wrong-parent sub-task:
  `sub-task \`1.1.2.1\` does not belong to task \`1.1.1\``;
- out-of-order sub-task:
  `sub-task \`1.1.1.3\` is not in document order`.

Use parser APIs such as `parse_roadmap_text` and `parse_fragment_text`. The red
proof is to temporarily change one expected string and run the focused test,
confirming the test fails for an exact diagnostic mismatch. Revert the
temporary mutation before committing. Update this ExecPlan's `Progress`,
`Decision Log`, and `Outcomes & Retrospective` with the red and green log paths
before the work-item commit.

Validation for this item:

```bash
cargo test --test roadmap_parse checklist -- --nocapture 2>&1 | tee /tmp/test-mapsplice-roadmap-5-1-1-item-1.out
MARKDOWN_PATHS='docs/execplans/roadmap-5-1-1.md' make markdownfmt 2>&1 | tee /tmp/markdownfmt-mapsplice-roadmap-5-1-1-item-1.out
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-5-1-1-item-1.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-1-item-1.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-1-item-1.out
```

Commit after all gates pass.

### Work Item 2: Add a mandatory checklist module and route both parsers

Docs and skills to read before editing: all Work Item 1 docs and skills, plus
`rust-types-and-apis` for private helper shape.

Create `src/roadmap/parse/checklist.rs` with a module-level `//!` comment. Add
a private helper that accepts `&ListItem` and a level-specific diagnostic kind,
then returns the shared head data:

- checked state from `ListItem.checked`;
- first `Paragraph`;
- body children after the first paragraph.

Route both `parse_task_item` and `parse_sub_task_item_unchecked` through this
helper. Keep `parse_task_paragraph`, `parse_sub_task_paragraph`,
`parse_numbered_paragraph`, and `split_numbered_prefix` as the existing shared
numbered-prefix path; do not create another numbered-prefix abstraction. Do not
change `parse_task_list`, `parse_sub_task_fragment_list`, `parse_sub_task_body`,
`validate_sub_task_number`, or fragment-root parsing except where imports or
helper calls require it.

The helper must preserve exact task and sub-task diagnostics from Work Item 1.
The green proof is that all focused parser tests from Work Item 1 pass without
snapshot, golden, or CLI-output changes. Use `sem diff --format json` before
committing to confirm the entity-level change is a narrow parser refactor.

Update this ExecPlan after the production change and before committing. Record
the final line counts for `src/roadmap/parse/mod.rs` and
`src/roadmap/parse/checklist.rs`; both must be at or below 400 lines.

Validation for this item:

```bash
cargo test --test roadmap_parse checklist -- --nocapture 2>&1 | tee /tmp/test-mapsplice-roadmap-5-1-1-item-2.out
cargo test --test roadmap_sub_tasks -- --nocapture 2>&1 | tee /tmp/test-sub-tasks-mapsplice-roadmap-5-1-1-item-2.out
cargo test --test roadmap_golden -- --nocapture 2>&1 | tee /tmp/test-golden-mapsplice-roadmap-5-1-1-item-2.out
wc -l src/roadmap/parse/mod.rs src/roadmap/parse/checklist.rs 2>&1 | tee /tmp/wc-mapsplice-roadmap-5-1-1-item-2.out
sem diff --format json 2>&1 | tee /tmp/sem-diff-mapsplice-roadmap-5-1-1-item-2.out
MARKDOWN_PATHS='docs/execplans/roadmap-5-1-1.md' make markdownfmt 2>&1 | tee /tmp/markdownfmt-mapsplice-roadmap-5-1-1-item-2.out
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-5-1-1-item-2.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-1-item-2.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-1-item-2.out
```

Commit after all gates pass.

### Work Item 3: Close roadmap task 5.1.1 and finalize documentation

Docs and skills to read before editing: `docs/roadmap.md` section 5.1.1;
`docs/documentation-style-guide.md`; `AGENTS.md`; `execplans`;
`en-gb-oxendict-style`; `sem`; `commit-message`.

Mark `docs/roadmap.md` task 5.1.1 complete only after Work Items 1 and 2 have
passed. Update this ExecPlan's `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` with the actual validation logs,
advisory-tool status, line-count evidence, and any deviations. Set
`Status: COMPLETE` only after all final gates pass.

Format only the changed Markdown paths. If only this ExecPlan and the roadmap
changed, run the exact command below. If another Markdown file was changed in
this work item, add only that existing path to `MARKDOWN_PATHS`.

```bash
MARKDOWN_PATHS='docs/execplans/roadmap-5-1-1.md docs/roadmap.md' make markdownfmt 2>&1 | tee /tmp/markdownfmt-mapsplice-roadmap-5-1-1-final.out
```

Then run final gates:

```bash
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-5-1-1-final.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-1-final.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-1-final.out
```

Commit after all gates pass.

## Concrete steps

1. Start each implementation session in the assigned worktree:

   ```bash
   cd /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-1
   git branch --show-current
   git status --short
   ```

   Expected branch output:

   ```plaintext
   roadmap-5-1-1
   ```

2. Retry advisory tools before editing:

   ```bash
   leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-1
   sem entities src/roadmap/parse/mod.rs
   sem diff --format json
   ```

   Also retry Memtrace MCP with `list_indexed_repositories`. If Memtrace still
   returns `user cancelled MCP tool call`, record it in this plan and continue.

3. Perform Work Items 1-3 in order. Do not combine work-item commits.

4. Use file-based commit messages:

   ```bash
   COMMIT_MSG_DIR="$(mktemp -d)"
   ${EDITOR:-vi} "$COMMIT_MSG_DIR/COMMIT_MSG.md"
   git commit -F "$COMMIT_MSG_DIR/COMMIT_MSG.md"
   rm -rf "$COMMIT_MSG_DIR"
   ```

5. Keep `docs/execplans/roadmap-5-1-1.md` current after every work item.

## Validation and acceptance

The accepted end state is:

- task and sub-task item parsers both use the same helper path for checklist
  item head extraction;
- numbered paragraph prefix extraction remains single-sourced through the
  existing `parse_numbered_paragraph` and `split_numbered_prefix` path;
- task-specific and sub-task-specific diagnostics are unchanged;
- fragment-root parsing remains scoped for roadmap task 5.1.2;
- `tests/roadmap_parse.rs` contains focused exact-diagnostic coverage for the
  checklist parser head and numbered-prefix seam;
- existing `tests/roadmap_sub_tasks.rs` coverage and golden fixtures pass
  unchanged;
- `docs/roadmap.md` marks task 5.1.1 complete only after the code gates pass;
- each work item that edits this ExecPlan runs scoped Markdown formatting,
  `make markdownlint`, and `make nixie` before commit;
- `make all`, `make markdownlint`, and `make nixie` pass at final acceptance.

Quality criteria:

- Tests: focused parser tests, sub-task integration tests, roadmap golden
  tests, and `make all` pass.
- Lint/typecheck: `make all` passes, including `check-fmt`, `lint`,
  `typecheck`, and `test`.
- Documentation: changed Markdown paths are formatted with scoped Markdown
  commands; `make markdownlint` and `make nixie` pass.
- Behaviour: no public CLI, public library API, or rendered-output change is
  accepted for this refactor.

## Idempotence and recovery

All planned edits are ordinary text changes and can be rerun safely from a
clean branch. If a focused parser test fails after a refactor, inspect the
`/tmp` log from the failed command before rerunning. Re-run the focused command
only after applying a fix.

If Markdown formatting touches files outside the intended paths, do not commit
that churn. Inspect `git diff --name-only`, restore only unrelated formatter
churn after confirming it is unrelated, and record the event here.

If Cargo waits on the shared package-cache lock, wait for the lock to clear. Do
not create an isolated Cargo cache.

## Artefacts and notes

Planning evidence gathered before this revision:

```plaintext
Memtrace: mcp__memtrace.list_indexed_repositories -> user cancelled MCP tool call
Leta: leta workspace add ... -> Error: IO error: Read-only file system (os error 30)
Firecrawl docs.rs scrape -> user cancelled MCP tool call
sem entities src/roadmap/parse/mod.rs -> parse_numbered_paragraph and split_numbered_prefix present
wc -l src/roadmap/parse/mod.rs -> 400
```

Important local source evidence:

```plaintext
src/roadmap/parse/mod.rs:155-187 parses task items.
src/roadmap/parse/mod.rs:262-295 parses sub-task items.
src/roadmap/parse/mod.rs:315-334 has thin typed numbered-paragraph wrappers.
src/roadmap/parse/mod.rs:336-352 strips numbered paragraph prefixes.
src/roadmap/parse/mod.rs:354-368 single-sources split_numbered_prefix.
src/roadmap/parse/mod.rs:378-389 uses split_numbered_prefix for list detection.
```

Round-2 design-review resolution:

```plaintext
1. Removed the no-op numbered-prefix extraction work item and documented that
   parse_numbered_paragraph plus split_numbered_prefix already satisfy that half
   of 5.1.1.
2. Made src/roadmap/parse/checklist.rs mandatory because mod.rs is already
   exactly 400 lines.
3. Added scoped Markdown formatting, make markdownlint, and make nixie to every
   work-item validation block because the living ExecPlan changes each time.
```

## Revision note

Round-2 revision. It resolves the design-review blockers by treating
numbered-prefix sharing as existing baseline, making the checklist module split
mandatory from the outset, and adding Markdown gates to every work-item commit
that updates this living ExecPlan. Remaining implementation work is reduced to
diagnostic pinning, shared checklist-head extraction, and roadmap closure.

Work Item 1 revision. It records the completed diagnostic-pinning tests, the
focused red and green evidence, the deterministic gate logs, and the deferred
CodeRabbit result. It also narrows the remaining non-paragraph branch coverage
to Work Item 2 helper-level unit tests because the public Markdown parser does
not expose that branch from ordinary GFM source text.

Work Item 2 revision. It records the shared checklist helper implementation,
the helper-level non-paragraph diagnostic tests, line-count evidence, semantic
diff evidence, deterministic gate result, and deferred CodeRabbit status. The
remaining work is roadmap closure and final documentation.

Work Item 3 revision. It records roadmap closure, final deterministic gate
evidence, final deferred CodeRabbit status, and marks this ExecPlan complete.
