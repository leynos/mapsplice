# Share fragment-root validation and step accumulation

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

This is planning round 3. Do not begin implementation until the plan is
approved by the df12-build roadmap workflow.

## Purpose / big picture

Roadmap task 5.1.2, "Share fragment-root validation and step accumulation",
reduces parser drift after 5.1.1 shared task and sub-task checklist parsing.
The current parser still has two duplicated fragment-root seams:

- `src/roadmap/parse/fragment.rs::parse_task_fragment_root` and
  `parse_sub_task_fragment_root` both validate "the fragment root must contain
  exactly one top-level list", parse that list, reject empty output, validate
  sibling membership, and wrap the result.
- `src/roadmap/parse/fragment.rs::parse_step_fragment_root` maintains a
  private step lifecycle that mirrors the document parser's lifecycle in
  `src/roadmap/parse/document.rs::DocumentParser`.

After this change, task, sub-task, and step fragment parsing must exercise
shared validation machinery without changing accepted roadmap grammar, public
APIs, user-facing diagnostics, or rendered output. A user observes success by
running the same fragment-level parser tests and repository gates before and
after the refactor and seeing no behavioural drift.

## Constraints

- Work only inside
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-2`.
- Do not edit the root/control worktree.
- Treat `origin/main` as canonical and `docs/roadmap.md` as the roadmap source
  of truth. At planning time, `HEAD` and `origin/main` were both
  `4d6b24b29b8465878fc5ee256fce65d9058f1d7c`.
- Implement only roadmap task 5.1.2 from `docs/roadmap.md`:
  "Remove the duplicated single-list fragment-root skeleton and reconcile step
  fragment parsing with the document parser's step lifecycle."
- Work Item 3 deliberately overlaps `docs/issues/audit-4.2.2.md` finding 14 by
  removing the redundant document-only `validate_sub_task_numbers` helper and
  call. This is necessary to avoid `dead_code` under
  `RUST_FLAGS := -D warnings` after `DocumentParser::append_task_list`
  delegates to the shared step accumulator.
- Preserve `docs/mapsplice-design.md` section 2, "Non-negotiable constraints":
  parsing remains mdast-based through the locked `markdown` crate and edits run
  through the roadmap model rather than raw-string surgery.
- Preserve `docs/mapsplice-design.md` section 4, "The roadmap grammar
  (normative reference)": phases are level-2 headings, steps are level-3
  headings, tasks are third-level numbered checklist items, and addendum
  sub-tasks are fourth-level numbered checklist items.
- Preserve `docs/mapsplice-design.md` section 5, "Fidelity guarantees",
  especially F1 content preservation, F3 round-trip stability, F4 gate-clean
  output, and F5 fail-closed behaviour.
- Preserve `docs/mapsplice-design.md` section 6, "Functional and contract
  guarantees", especially C1 strict level matching, C2 contiguous renumbering,
  C4 first-class addendum sub-tasks, and C5 idempotence.
- Preserve `docs/mapsplice-design.md` section 8, "Fixture and test
  requirements": use `rstest` unit coverage for parser behaviour and preserve
  golden and behavioural coverage for end-to-end guarantees.
- Preserve `docs/developers-guide.md` section 2, "Architecture boundaries":
  `src/roadmap` owns domain parsing, mutation, renumbering, and rendering.
- Preserve `docs/developers-guide.md` section 3, "Public library APIs": do not
  add, remove, or rename public parser APIs or public error variants.
- Follow `docs/developers-guide.md` section 6, "Verification layers": use
  `rstest` unit tests for parser behaviour; keep `rstest-bdd`, `proptest`,
  `trybuild`, and `insta` as wider gates unless a work item explicitly changes
  their surfaces.
- Follow `docs/developers-guide.md` section 7, "Local tooling": run repository
  gates through Makefile targets, and use scoped Markdown maintenance for
  changed Markdown files.
- Preserve `docs/users-guide.md`, "The roadmap shape `mapsplice` expects" and
  "Fragments supplied to `insert` and `replace` must contain one or more
  sibling items at the same level as the target anchor."
- Follow `docs/documentation-style-guide.md`: prose must use en-GB Oxford
  spelling, sentence-case headings, fenced code languages, and 80-column
  wrapping.
- Do not add a new external dependency. Use the locked crates already in
  `Cargo.lock`.
- Keep every Rust source file under 400 lines. Current line counts are:
  `src/roadmap/parse/document.rs` 237, `src/roadmap/parse/fragment.rs` 305, and
  `src/roadmap/parse/mod.rs` 370.
- Every new Rust module must begin with a module-level `//!` comment.
- Use Red-Green-Refactor. Because this is a behaviour-preserving refactor, the
  red stage is a mutation-style proof for reachable public behaviour:
  temporarily break the expected shared path or diagnostic, confirm the focused
  test fails for the intended reason, and revert the temporary mutation before
  committing. Do not require public passing tests for private defensive
  branches that are unreachable through `parse_fragment_text`.
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
  proves a necessary adjacent change: `docs/execplans/roadmap-5-1-2.md`,
  `docs/roadmap.md`, `src/roadmap/parse/document.rs`,
  `src/roadmap/parse/fragment.rs`, `src/roadmap/parse/mod.rs`, a new
  `src/roadmap/parse/step_accumulator.rs`, and `tests/roadmap_parse.rs`.
- Stop and escalate if implementation needs changes outside `src/roadmap/parse`
  or parser tests, excluding the living ExecPlan and roadmap status update.
- Stop and escalate if exact diagnostic text changes for existing malformed
  fragment or document inputs.
- Stop and escalate if task, sub-task, and step fragment parsing cannot share
  validation machinery without broadening accepted syntax.
- Stop and escalate if the net production-code diff exceeds 220 lines or if
  any Rust source file exceeds 400 lines.
- Stop and escalate if the same focused parser test still fails after three
  implementation attempts.
- Do not treat advisory-tool unavailability as a blocker. Record the failed
  command or MCP call and continue with bounded local source, tests, and
  documentation evidence.

## Risks

- Risk: a generic single-list helper could accidentally homogenize task and
  sub-task fragment diagnostics. Severity: medium. Likelihood: medium.
  Mitigation: Work Item 1 pins exact reachable root and sibling diagnostics
  before production refactoring. Work Item 2 preserves the unreachable
  empty-vector guards verbatim inside the helper as defensive branches, but
  does not demand public tests for branches `parse_fragment_text` cannot reach.
- Risk: sharing the step lifecycle could move document-only phase validation
  into fragment parsing, or fragment-only "must contain only step sections"
  validation into document parsing. Severity: high. Likelihood: medium.
  Mitigation: introduce a step accumulator that owns only active-step state,
  task-list routing, and body-versus-trailing accumulation; keep phase
  validation in `DocumentParser` and fragment-level root validation in
  `fragment.rs`.
- Risk: a broad parser abstraction could absorb roadmap task 5.1.3 lookup and
  rendering helper work. Severity: medium. Likelihood: low. Mitigation: do not
  touch `src/roadmap/ops`, render helpers, `fragment_level`, or dependency
  rewrite recording in this task.
- Risk: step-fragment tests may only prove success paths. Severity: medium.
  Likelihood: medium. Mitigation: add focused unit tests for wrong-step
  diagnostics, trailing-content ordering, non-step heading rejection, empty
  fragments, and sibling phase validation.
- Risk: a future implementer may try to force public tests for the task and
  sub-task empty-list diagnostics. Severity: medium. Likelihood: medium.
  Mitigation: this plan documents why those branches are unreachable through
  `parse_fragment_text` and limits them to preserved private defensive
  diagnostics unless an implementer adds in-module private scaffolding.
- Risk: advisory tools may remain unavailable in the implementation session.
  Severity: low. Likelihood: high. Mitigation: retry Memtrace and Leta before
  editing, record exact failures if they persist, and proceed with bounded
  local source inspection and tests.

## Progress

- [x] (2026-07-03T08:10:38Z) Confirmed the current branch is
  `roadmap-5-1-2`, so this plan is `docs/execplans/roadmap-5-1-2.md`.
- [x] (2026-07-03T08:10:38Z) Loaded required skills for planning:
  `execplans`, `leta`, `rust-router`, `sem`, and `firecrawl-mcp`.
- [x] (2026-07-03T08:10:38Z) Loaded `AGENTS.md`,
  `docs/mapsplice-design.md`, `docs/developers-guide.md`, `docs/users-guide.md`,
  `docs/contributing.md`, `docs/documentation-style-guide.md`,
  `docs/issues/audit-4.2.2.md`, and `docs/execplans/roadmap-5-1-1.md`.
- [x] (2026-07-03T08:10:38Z) Memtrace discovery failed with
  `mcp__memtrace.list_indexed_repositories -> user cancelled MCP tool call`.
- [x] (2026-07-03T08:10:38Z) Leta workspace setup failed with
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-2`
  returning `Error: IO error: Read-only file system (os error 30)`.
- [x] (2026-07-03T08:10:38Z) Firecrawl official-docs verification failed with
  `mcp__firecrawl.firecrawl_scrape` for
  `https://docs.rs/markdown/1.0.0/markdown/index.html` returning
  `user cancelled MCP tool call`.
- [x] (2026-07-03T08:10:38Z) Used `sem entities src/roadmap/parse --format json`
  and `sem blame src/roadmap/parse/fragment.rs` for entity-level parser
  navigation and history.
- [x] (2026-07-03T08:10:38Z) Verified locked crate source for
  `markdown 1.0.0` and `rstest 0.26.1` from the local Cargo registry.
- [x] (2026-07-03T08:10:38Z) Wrote the first-round ExecPlan.
- [x] (2026-07-03T10:58:00Z) Re-reviewed the two round-2 design-review
  blockers against branch-local source and Makefile evidence.
- [x] (2026-07-03T10:58:00Z) Memtrace discovery failed again with
  `mcp__memtrace.list_indexed_repositories -> user cancelled MCP tool call`.
- [x] (2026-07-03T10:58:00Z) Leta workspace setup succeeded, and `leta show`
  verified `DocumentParser::append_task_list`, `append_step_fragment_tasks`,
  `parse_step_fragment_root`, `parse_task_item`, `parse_sub_task_list`, and
  `validate_sub_task_number`.
- [x] (2026-07-03T10:58:00Z) Leta reference search for
  `validate_sub_task_numbers` failed with `Error: Failed to start daemon`;
  exact local text search then confirmed the helper is only defined and called
  in `src/roadmap/parse/document.rs`.
- [x] (2026-07-03T10:58:00Z) Firecrawl official-docs verification failed again
  with this tool result:

  ```plaintext
  mcp__firecrawl.firecrawl_scrape https://docs.rs/markdown/1.0.0/markdown/index.html
  user cancelled MCP tool call
  ```

- [x] (2026-07-03T10:58:00Z) Revised the ExecPlan to pin the real
  trailing-content diagnostic and to remove the redundant sub-task number
  cross-check as part of Work Item 3.
- [x] (2026-07-03T12:35:00Z) Re-reviewed the round-3 blocking point against
  branch-local parser code. `leta show` verified `parse_fragment`,
  `parse_task_fragment_root`, `parse_sub_task_fragment_root`, `parse_task_list`,
  `parse_sub_task_fragment_list`, and `looks_like_numbered_list`.
- [x] (2026-07-03T12:35:00Z) Memtrace discovery failed again with
  `mcp__memtrace.list_indexed_repositories -> user cancelled MCP tool call`.
- [x] (2026-07-03T12:35:00Z) Leta workspace setup failed again with
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-2`
  returning `Error: IO error: Read-only file system (os error 30)`, but
  branch-local `leta grep` and `leta show` succeeded for the parser symbols
  named above. `leta files` and one `leta refs parse_fragment` call failed with
  `Error: Failed to start daemon`.
- [x] (2026-07-03T12:35:00Z) Firecrawl official-docs verification failed again
  with `mcp__firecrawl.firecrawl_scrape` for
  `https://docs.rs/markdown/1.0.0/markdown/index.html` returning
  `user cancelled MCP tool call`; locked local crate source remains the
  verified dependency evidence.
- [x] (2026-07-03T12:35:00Z) Revised Work Items 1 and 2, acceptance criteria,
  and the revision note so the plan no longer requires public passing tests for
  `task fragment list is empty` or `sub-task fragment list is empty`.
- [x] (2026-07-03T08:52:56Z) Started Work Item 1 in
  `tests/roadmap_parse.rs` and this ExecPlan.
- [x] (2026-07-03T08:52:56Z) Retried Memtrace discovery; it failed with
  `mcp__memtrace.list_indexed_repositories -> user cancelled MCP tool call`.
- [x] (2026-07-03T08:52:56Z) Retried Leta workspace setup; it failed with
  `Error: IO error: Read-only file system (os error 30)`. Branch-local
  `leta show` still succeeded for `parse_fragment`, `parse_step_fragment_root`,
  `validate_step_siblings`, and `validate_sub_task_siblings`.
- [x] (2026-07-03T08:56:03Z) Added Work Item 1 parser diagnostic tests for
  extra task and sub-task fragment root nodes, sub-task sibling parents,
  non-step headings, trailing-content ordering, and cross-phase step siblings.
- [x] (2026-07-03T08:56:03Z) Captured the Work Item 1 mutation-style red proof:
  changing the trailing-content expected diagnostic to omit backticks made
  `parse_step_fragment_keeps_lifecycle_diagnostics::case_2_task_after_trailing_content`
  fail in `/tmp/test-red-mapsplice-roadmap-5-1-2-item-1.out`.
- [x] (2026-07-03T08:56:03Z) Reverted the temporary mutation and confirmed
  `cargo test --test roadmap_parse fragment` passed with 14 tests in
  `/tmp/test-mapsplice-roadmap-5-1-2-item-1.out`.
- [x] (2026-07-03T08:56:03Z) Formatted the changed ExecPlan with
  `MARKDOWN_PATHS='docs/execplans/roadmap-5-1-2.md' make markdownfmt`; log:
  `/tmp/markdownfmt-mapsplice-roadmap-5-1-2-item-1.out`.
- [x] (2026-07-03T08:56:03Z) `scrutineer` reported green deterministic gates
  for Work Item 1: `make all`, `make markdownlint`, and `make nixie` passed
  with logs under `/tmp/*-mapsplice-roadmap-5-1-2-item-1.out`.
- [x] (2026-07-03T08:56:03Z) `scrutineer` attempted CodeRabbit for Work Item 1;
  it deferred without review findings because
  `deferred coderabbit review: no default network route visible in this sandbox`.
  Log: `/tmp/coderabbit-mapsplice-roadmap-5-1-2-item-1.out`.
- [x] (2026-07-03T08:59:49Z) Started Work Item 2 in
  `src/roadmap/parse/fragment.rs` and this ExecPlan.
- [x] (2026-07-03T08:59:49Z) Verified Work Item 2 target symbols with
      `leta show`
  for `parse_task_fragment_root`, `parse_sub_task_fragment_root`,
  `validate_task_siblings`, and `validate_sub_task_siblings`.
- [x] (2026-07-03T08:59:49Z) Captured the Work Item 2 mutation-style red proof:
  changing the task fragment single-list diagnostic made
  `parse_single_list_fragments_reject_extra_root_nodes::case_1_task_fragment_with_trailing_paragraph`
  fail in `/tmp/test-red-mapsplice-roadmap-5-1-2-item-2.out`.
- [x] (2026-07-03T08:59:49Z) Added the private
  `SingleListFragmentMessages` and `parse_single_list_fragment` helper, then
  rewired task and sub-task fragment roots through it without changing public
  parser APIs or accepted grammar.
- [x] (2026-07-03T08:59:49Z) Confirmed
  `cargo test --test roadmap_parse fragment` passed with 14 tests in
  `/tmp/test-mapsplice-roadmap-5-1-2-item-2.out`.
- [x] (2026-07-03T08:59:49Z) Ran
  `sem diff --format json` for the Work Item 2 implementation; log:
  `/tmp/sem-diff-mapsplice-roadmap-5-1-2-item-2.out`.
- [x] (2026-07-03T08:59:49Z) Initial Work Item 2 `make all` failed Clippy on
  `shadow-reuse`, `too_many_arguments`, and `needless_pass_by_value`; log:
  `/tmp/all-mapsplice-roadmap-5-1-2-item-2.out`.
- [x] (2026-07-03T09:04:43Z) Replaced the six-argument helper signature with a
  private `SingleListFragmentParser<T>` struct containing the messages, list
  parser, validator, and wrapper function pointers. This preserves the same
  fragment-root behaviour while satisfying the repository Clippy policy.
- [x] (2026-07-03T09:04:43Z) Re-ran
  `cargo test --test roadmap_parse fragment`; the focused fragment tests passed
  again in `/tmp/test-mapsplice-roadmap-5-1-2-item-2.out`.
- [x] (2026-07-03T09:04:43Z) Verified
  `src/roadmap/parse/fragment.rs` is 343 lines after the Clippy repair, under
  the 400-line cap.
- [x] (2026-07-03T09:04:43Z) `scrutineer` reported green deterministic gates
  for Work Item 2 after the Clippy repair: `make all`, `make markdownlint`, and
  `make nixie` passed with logs under
  `/tmp/*-mapsplice-roadmap-5-1-2-item-2-rerun.out`.
- [x] (2026-07-03T09:04:43Z) `scrutineer` attempted CodeRabbit for Work Item 2;
  it deferred without review findings because
  `deferred coderabbit review: no default network route visible in this sandbox`.
  Log: `/tmp/coderabbit-mapsplice-roadmap-5-1-2-item-2.out`.
- [x] (2026-07-03T09:10:06Z) Started Work Item 3 in
  `src/roadmap/parse/document.rs`, `src/roadmap/parse/fragment.rs`,
  `src/roadmap/parse/mod.rs`, new `src/roadmap/parse/step_accumulator.rs`, and
  this ExecPlan.
- [x] (2026-07-03T09:10:06Z) Retried Memtrace discovery before changing
  load-bearing parser symbols; it failed again with
  `mcp__memtrace.list_indexed_repositories -> user cancelled MCP tool call`.
- [x] (2026-07-03T09:10:06Z) Verified Work Item 3 target symbols with
  `leta show` for `DocumentParser.begin_phase`, `DocumentParser.begin_step`,
  `DocumentParser.append_task_list`, `DocumentParser.flush_step`,
  `DocumentParser.push_non_structural_node`, `DocumentParser.finish`,
  `parse_step_fragment_root`, `append_step_fragment_tasks`,
  `push_step_fragment_body`, `validate_sub_task_numbers`, and
  `validate_tasks_belong_to_step`.
- [x] (2026-07-03T09:10:06Z) Captured the Work Item 3 mutation-style red proof:
  changing the trailing-content diagnostic to omit backticks made
  `parse_step_fragment_keeps_lifecycle_diagnostics::case_2_task_after_trailing_content`
  fail in `/tmp/test-red-mapsplice-roadmap-5-1-2-item-3.out`.
- [x] (2026-07-03T09:10:06Z) Added the private `StepAccumulator` module,
  delegated document and step-fragment active-step state to it, removed
  `append_step_fragment_tasks`, `push_step_fragment_body`, and
  `validate_sub_task_numbers`, and kept phase/preamble handling in
  `DocumentParser`.
- [x] (2026-07-03T09:10:06Z) Confirmed
  `cargo test --test roadmap_parse fragment` passed with 14 tests in
  `/tmp/test-mapsplice-roadmap-5-1-2-item-3.out`.
- [x] (2026-07-03T09:10:06Z) Confirmed `cargo test --test roadmap_golden`
  passed with 64 tests in `/tmp/test-golden-mapsplice-roadmap-5-1-2-item-3.out`.
- [x] (2026-07-03T09:10:06Z) Confirmed
  `cargo test --test roadmap_parse parse_sub_task_checklist_validation_diagnostics`
  passed with the wrong-parent and out-of-order sub-task diagnostics in
  `/tmp/test-sub-task-validation-mapsplice-roadmap-5-1-2-item-3.out`.
- [x] (2026-07-03T09:10:06Z) Ran
  `sem diff --format json` for the Work Item 3 implementation; log:
  `/tmp/sem-diff-mapsplice-roadmap-5-1-2-item-3.out`.
- [x] (2026-07-03T09:10:06Z) Verified line counts after the accumulator split:
  `document.rs` 187, `fragment.rs` 283, `mod.rs` 371, and `step_accumulator.rs`
  93.
- [x] (2026-07-03T09:12:03Z) Initial Work Item 3 `make all` failed Clippy on a
  shadowed `phase` binding in `DocumentParser::begin_step`; log:
  `/tmp/all-mapsplice-roadmap-5-1-2-item-3.out`.
- [x] (2026-07-03T09:12:03Z) Renamed the mutable phase binding to
  `current_phase` and re-ran focused parser checks:
  `cargo test --test roadmap_parse fragment` and
  `cargo test --test roadmap_parse parse_sub_task_checklist_validation_diagnostics`
  both passed in the Work Item 3 logs.
- [x] (2026-07-03T09:15:57Z) `scrutineer` first hit a gate-runner
  infrastructure failure because its shell attempted `/usr/bin/time`, which is
  absent in this environment. A retry that explicitly avoided external timing
  wrappers passed `make all`, `make markdownlint`, and `make nixie` with logs
  under `/tmp/*-mapsplice-roadmap-5-1-2-item-3-rerun.out`.
- [x] (2026-07-03T09:15:57Z) `scrutineer` attempted CodeRabbit for Work Item 3;
  it deferred without review findings because
  `deferred coderabbit review: no default network route visible in this sandbox`.
  Log: `/tmp/coderabbit-mapsplice-roadmap-5-1-2-item-3.out`.
- [x] (2026-07-03T09:18:04Z) Started Work Item 4 in `docs/roadmap.md` and this
  ExecPlan.
- [x] (2026-07-03T09:18:04Z) Marked roadmap task 5.1.2 complete in
  `docs/roadmap.md`.
- [x] (2026-07-03T09:18:04Z) Closed this ExecPlan with final outcomes,
  remaining open issue, and documentation/skill signposts for follow-on agents.
- [x] (2026-07-03T09:21:32Z) `scrutineer` passed the Work Item 4 deterministic
  gates: `make all`, `make markdownlint`, and `make nixie`. Logs:
  `/tmp/all-mapsplice-roadmap-5-1-2-item-4.out`,
  `/tmp/markdownlint-mapsplice-roadmap-5-1-2-item-4.out`, and
  `/tmp/nixie-mapsplice-roadmap-5-1-2-item-4.out`.
- [x] (2026-07-03T09:22:18Z) `scrutineer` attempted CodeRabbit for Work Item 4;
  it deferred without review findings because
  `deferred coderabbit review: no default network route visible in this sandbox`.
  Log: `/tmp/coderabbit-mapsplice-roadmap-5-1-2-item-4.out`.
- [x] (2026-07-03T09:24:11Z) Re-ran the Work Item 4 deterministic gates after
  recording the CodeRabbit deferral and final outcome notes. `make all`,
  `make markdownlint`, and `make nixie` all passed in
  `/tmp/*-mapsplice-roadmap-5-1-2-item-4-final.out`.

## Surprises & discoveries

- Observation: Memtrace was available in tool metadata but the host cancelled
  the discovery call. Evidence:
  `mcp__memtrace.list_indexed_repositories -> user cancelled MCP tool call`.
  Impact: this plan records bounded local, `sem`, and locked-source evidence
  instead. This is not a product blocker under the task instructions.
- Observation: Leta could not initialize in this sandbox. Evidence:
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-2`
  returned `Error: IO error: Read-only file system (os error 30)`. Impact: this
  plan uses precise branch-local source inspection and `sem` entity navigation
  instead.
- Observation: Firecrawl could not retrieve official docs.rs documentation in
  this session. Evidence: `mcp__firecrawl.firecrawl_scrape` returned
  `user cancelled MCP tool call`. Impact: the locked registry source is used
  for load-bearing crate behaviour, and the implementation session should retry
  Firecrawl if the MCP host allows it.
- Observation: 5.1.1 has already introduced `src/roadmap/parse/checklist.rs`,
  and `src/roadmap/parse/mod.rs` is now 370 lines, not the former 400-line
  pressure point. Evidence: local line counts and the 5.1.1 ExecPlan. Impact:
  5.1.2 can add a small `step_accumulator.rs` module without pushing `mod.rs`
  over the file-size limit.
- Observation: Leta availability changed between planning rounds. Evidence:
  round 1 recorded `leta workspace add` failing with read-only filesystem
  error; round 2 `leta workspace add` succeeded and `leta show` worked, while
  `leta refs validate_sub_task_numbers` failed with
  `Error: Failed to start daemon`. Impact: implementation agents should still
  try Leta first, record exact failures, and use exact local source inspection
  only for failed Leta operations.
- Observation: `sem blame src/roadmap/parse/fragment.rs` shows
  `parse_step_fragment_root` and `parse_task_fragment_root` originate in the
  initial tool, while `parse_sub_task_fragment_root` was added later with
  operation golden fixtures. Impact: the duplicated fragment roots are real
  drift risk, not merely local stylistic similarity.
- Observation: the current document parser already validates sub-task parent
  membership while parsing each sub-task. Evidence: the document path flows
  through `parse_task_item`, `parse_sub_task_list`, `parse_sub_task_item`, and
  `validate_sub_task_number`, which rejects this diagnostic:

  ```plaintext
  sub-task `{number}` does not belong to task `{parent}`
  ```

  `DocumentParser::append_task_list` then calls the redundant
  `validate_sub_task_numbers` helper, while `append_step_fragment_tasks` does
  not. Impact: Work Item 3 removes the helper and its call rather than widening
  the accumulator API solely to preserve a duplicate cross-check.
- Observation: the current trailing-content diagnostic includes backticks around
  the displayed step number. Evidence: `src/roadmap/parse/document.rs:157` and
  `src/roadmap/parse/fragment.rs:228` both use
  ``"task list for step `{}` cannot appear after trailing step content"``.
  Impact: Work Items 1 and 3 now pin the byte-for-byte diagnostic with the
  backticks.
- Observation: the task and sub-task empty-vector guards in
  `parse_task_fragment_root` and `parse_sub_task_fragment_root` are unreachable
  through the public `parse_fragment_text` surface. Evidence:
  `src/roadmap/parse/fragment.rs::parse_fragment` dispatches to those roots
  only when the first node passes `looks_like_task_list` or
  `looks_like_sub_task_list`; `looks_like_numbered_list` returns `false` for an
  empty list and otherwise requires a first list item, first paragraph, first
  text node, and correctly levelled numbered prefix. Once dispatch has reached
  the root parser, `parse_task_list` and `parse_sub_task_fragment_list` iterate
  over the non-empty `list.children` vector and either return at least one
  parsed entry or return an earlier item-level diagnostic. Impact: Work Item 1
  no longer mandates public passing tests for `task fragment list is empty` or
  `sub-task fragment list is empty`, and Work Item 2 preserves those messages
  only as defensive helper branches.
- Observation: CodeRabbit could not run in this sandbox. Evidence:
  `/home/leynos/Projects/mapsplice.workshop/df12-build-20260629T235232Z-879541/bin/coderabbit-review-agent`
  returned
  `deferred coderabbit review: no default network route visible in this sandbox`.
  Impact: Work Item 1 has no CodeRabbit findings to address, and the deferred
  review remains an open issue for a network-enabled supervisor or retry.

## Decision log

- Decision: implement a generic single-list fragment-root helper rather than
  separate task and sub-task helpers. Rationale: the audit identifies a
  near-verbatim skeleton, and the per-level differences can be represented as
  closures plus per-level messages without sharing diagnostics accidentally.
  Date/Author: 2026-07-03, Codex.
- Decision: implement a dedicated step accumulator module rather than driving
  step fragments through `DocumentParser` directly. Rationale: `DocumentParser`
  also owns phase and preamble handling; using it directly would require a
  synthetic phase wrapper or phase-stripping post-processing, which is more
  invasive and risks changing fragment diagnostics. A `StepAccumulator`
  precisely shares the lifecycle that both paths need: begin step, flush
  previous step, append validated task lists, and route body versus trailing
  nodes. Date/Author: 2026-07-03, Codex.
- Decision: do not add or change dependencies. Rationale: `markdown 1.0.0`
  already exposes the mdast `Root`, `Heading`, `List`, and `ListItem` shapes
  the parser uses, and `rstest 0.26.1` already supports the table tests needed
  to pin diagnostics. Date/Author: 2026-07-03, Codex.
- Decision: use mutation-style red proofs for this refactor. Rationale: the
  intended behaviour already exists; the missing value is single-sourcing, so
  committed tests should pass before and after while temporary mutations prove
  the tests fail for the intended reason. Date/Author: 2026-07-03, Codex.
- Decision: Work Item 3 removes `validate_sub_task_numbers` and its call from
  `DocumentParser::append_task_list`. Rationale: the parse-time checked
  document path already enforces the same invariant through
  `parse_task_item -> parse_sub_task_list -> parse_sub_task_item -> validate_sub_task_number`,
  and the step-fragment path has never called the helper. Keeping the helper
  after delegation would either create `dead_code` under
  `RUST_FLAGS := -D warnings` or force the shared accumulator to expose parsed
  tasks only for a duplicate document-path cross-check. This is a justified,
  scoped overlap with audit finding 14. Date/Author: 2026-07-03, Codex.
- Decision: do not add in-module private scaffolding solely to test the
  unreachable empty-vector guards. Rationale: the plan can satisfy roadmap
  5.1.2 by preserving those branches verbatim inside the shared helper while
  testing the reachable public diagnostics. Adding `#[cfg(test)]` constructors
  for synthetic empty `Root { children: [List { children: [] }] }` values would
  increase parser-private test surface only to exercise defensive code that the
  public parser cannot dispatch to. If implementation later changes dispatch so
  those branches become reachable, the implementer must add focused coverage in
  the same work item before relying on them. Date/Author: 2026-07-03, Codex.
- Decision: represent the single-list helper's per-fragment behaviour with a
  private `SingleListFragmentParser<T>` struct instead of the draft
  six-argument helper signature. Rationale: the draft shape preserved behaviour
  but violated the repository's `too_many_arguments` and
  `needless_pass_by_value` Clippy policy under `make all`. Grouping the
  messages and function pointers keeps the helper private, explicit, and
  diagnostic-preserving without adding lint suppressions. Date/Author:
  2026-07-03, Codex.
- Decision: make `StepAccumulator::begin_step` return `()` rather than
  `Result<()>`. Rationale: beginning a step only flushes an already-active step
  into a caller-supplied vector and constructs a `StepSection`; all fallible
  phase and heading validation remains in `DocumentParser` or
  `parse_step_heading`. Returning a `Result` would add an unneeded fallible
  surface solely to mirror the draft signature. Date/Author: 2026-07-03, Codex.

## Outcomes & retrospective

Roadmap task 5.1.2 is implemented. Work Item 1 pinned the reachable
fragment-root and step lifecycle diagnostics in `tests/roadmap_parse.rs`. Work
Item 2 shared task and sub-task single-list fragment-root validation in
`src/roadmap/parse/fragment.rs`. Work Item 3 added
`src/roadmap/parse/step_accumulator.rs` and routed both document parsing and
step-fragment parsing through the same active-step lifecycle for step creation,
task-list ordering, and body-versus-trailing content. Work Item 4 marked
`docs/roadmap.md` task 5.1.2 complete and closed this ExecPlan.

The public parser APIs stayed unchanged, no dependency was added, and accepted
roadmap grammar and diagnostics stayed stable. The redundant
`validate_sub_task_numbers` document-only helper was removed because the
document parser still enforces wrong-parent and out-of-order sub-task
diagnostics through the parse-time `validate_sub_task_number` path.

Final deterministic evidence before the Work Item 4 commit:

- Work Item 1: `make all`, `make markdownlint`, and `make nixie` passed in
  `/tmp/*-mapsplice-roadmap-5-1-2-item-1-rerun.out`.
- Work Item 2: `make all`, `make markdownlint`, and `make nixie` passed in
  `/tmp/*-mapsplice-roadmap-5-1-2-item-2-final.out`.
- Work Item 3: `make all`, `make markdownlint`, and `make nixie` passed in
  `/tmp/*-mapsplice-roadmap-5-1-2-item-3-final.out`.
- Work Item 4: `make all`, `make markdownlint`, and `make nixie` passed in
  `/tmp/*-mapsplice-roadmap-5-1-2-item-4-final.out`.

CodeRabbit was attempted once for each implementation work item and deferred
each time with
`deferred coderabbit review: no default network route visible in this sandbox`.
No actionable CodeRabbit findings were available to address.

Documentation and skill signposts used during implementation:

- Documentation: `AGENTS.md`, `docs/roadmap.md`,
  `docs/mapsplice-design.md`, `docs/developers-guide.md`, `docs/users-guide.md`,
  `docs/documentation-style-guide.md`, `docs/issues/audit-4.2.2.md`, and
  `docs/execplans/roadmap-5-1-1.md`.
- Skills and tools: `execplans`, `leta`, `rust-router`,
  `rust-unit-testing`, `rust-types-and-apis`, `rust-errors`, `sem`,
  `commit-message`, Memtrace MCP attempts, and `scrutineer`.

## Context and orientation

`mapsplice` parses Markdown into mdast with `markdown::to_mdast`, converts the
mdast root into a typed roadmap model, applies one operation, renumbers,
rewrites dependency references, and renders Markdown. The parser surface for
this task is private to `src/roadmap/parse`.

The key files are:

- `src/roadmap/parse/mod.rs`: public parse entry points, heading and numbered
  paragraph helpers, task parsing, sub-task parsing, and shared list-shape
  probes.
- `src/roadmap/parse/document.rs`: document-level parser with a
  `DocumentParser` that owns phase state and active-step state.
- `src/roadmap/parse/fragment.rs`: fragment-level parser that chooses fragment
  level and currently owns task, sub-task, and step fragment-root parsing.
- `src/roadmap/parse/sub_task_fragment.rs`: top-level sub-task list parser for
  sub-task fragments.
- `tests/roadmap_parse.rs`: focused `rstest` coverage for anchors, fragments,
  and document parser diagnostics.
- `docs/roadmap.md`: the living roadmap; task 5.1.2 must be marked complete
  only after implementation and gates pass.

Important terms:

- A fragment is the Markdown file supplied to `append`, `insert`, or
  `replace`. It must contain sibling items at one structural level.
- Fragment-root validation is the first validation over the mdast root of the
  fragment, before individual roadmap items are parsed.
- Step accumulation is the lifecycle that starts a step from a heading,
  flushes the previous active step, appends task lists to the current step, and
  stores non-structural nodes in either step body or step trailing content.

## Verified dependency behaviour

The plan relies on the locked `markdown` crate and `rstest` test macro already
present in the workspace. No new external library is required.

`Cargo.lock` pins `markdown` to `1.0.0` and `rstest` to `0.26.1`. The local
registry source verifies the load-bearing API:

- `markdown 1.0.0` exposes
  `to_mdast(value: &str, options: &ParseOptions) -> Result<mdast::Node, message::Message>`
  in
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/lib.rs:160`.
- `markdown::mdast::Node` has `Root`, `List`, and `Heading` variants in
  `markdown-1.0.0/src/mdast.rs:168`.
- `markdown::mdast::Root` owns `pub children: Vec<Node>` in
  `markdown-1.0.0/src/mdast.rs:591`.
- `markdown::mdast::Heading` owns `pub children: Vec<Node>` and
  `pub depth: u8` in `markdown-1.0.0/src/mdast.rs:625`.
- `markdown::mdast::List` owns `pub children: Vec<Node>` and
  `pub ordered: bool` in `markdown-1.0.0/src/mdast.rs:677`.
- `markdown::mdast::ListItem` owns `pub children: Vec<Node>` and
  `pub checked: Option<bool>` in `markdown-1.0.0/src/mdast.rs:704`.
- `ParseOptions::gfm()` enables GFM constructs, including tasklists, in
  `markdown-1.0.0/src/configuration.rs:1275`.
- `rstest 0.26.1` documents `#[rstest]` and `#[case]` table tests in
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rstest-0.26.1/src/lib.rs`.

Firecrawl official-documentation verification was attempted in both planning
rounds and failed with `user cancelled MCP tool call`. The implementation
session should retry Firecrawl for
`https://docs.rs/markdown/1.0.0/markdown/index.html` if available, but the
mechanism above is pinned to the locked source and does not depend on
undocumented behaviour.

## Plan of work

### Work Item 1: Pin fragment parser diagnostics and lifecycle behaviour

Documentation to read before editing:

- `docs/roadmap.md` section 5.1.2.
- `docs/mapsplice-design.md` sections 2, 4, 5, 6, and 8.
- `docs/developers-guide.md` sections 2, 3, 6, and 7.
- `docs/users-guide.md`, "The roadmap shape `mapsplice` expects".
- `docs/issues/audit-4.2.2.md` findings 12 and 13.
- `AGENTS.md`, "Rust Specific Guidance" and "Testing".

Skills to load:

- `leta` for branch-local symbol navigation; if unavailable, record the exact
  failure and use precise file inspection.
- `rust-router`, routed to `rust-unit-testing`.
- `sem` for entity-level change review.

Add focused passing tests in `tests/roadmap_parse.rs` before production-code
changes. Use `rstest` parameterized cases where the existing tests can be
extended without duplication. Pin these reachable public behaviours:

- task fragments reject multiple top-level nodes with
  `task fragments must contain only a single task list`;
- sub-task fragments reject multiple top-level nodes with
  `sub-task fragments must contain only a single sub-task list`;
- sub-task fragments with sibling sub-tasks from different parent tasks reject
  with `sub-task fragments must contain sub-tasks from one task`;
- step fragments reject non-step headings with
  `step fragments must contain only step sections`;
- step fragments reject task lists after trailing step content with the exact
  current diagnostic, including backticks around the displayed `StepNumber`.
  For step `1.2`, the byte-for-byte diagnostic is:

  ```plaintext
  task list for step `1.2` cannot appear after trailing step content
  ```

- step fragments with steps from different phases reject with
  `step fragments must contain steps from one phase`.

Do not add `tests/roadmap_parse.rs` cases that expect
`task fragment list is empty` or `sub-task fragment list is empty`. Those two
diagnostics are unreachable through `parse_fragment_text`: fragment dispatch
requires `looks_like_task_list` or `looks_like_sub_task_list`, both of which
return `false` for an empty mdast list before the private root functions are
called. The list parsers also map over list children, so any successfully
dispatched list either yields at least one entry or fails earlier with an
item-level diagnostic. Treat these messages as private defensive diagnostics
preserved by Work Item 2, not public behaviours to pin in this work item.

Use mutation-style red proof: temporarily change one expected diagnostic or
temporarily bypass one existing validation branch, run the focused test and
confirm failure, then revert the temporary mutation before committing.

Validation for this work item:

```bash
cargo test --test roadmap_parse fragment 2>&1 | tee /tmp/test-mapsplice-roadmap-5-1-2-item-1.out
MARKDOWN_PATHS='docs/execplans/roadmap-5-1-2.md' make markdownfmt 2>&1 | tee /tmp/markdownfmt-mapsplice-roadmap-5-1-2-item-1.out
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-5-1-2-item-1.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-2-item-1.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-2-item-1.out
sem diff --format json 2>&1 | tee /tmp/sem-diff-mapsplice-roadmap-5-1-2-item-1.out
```

Commit after the gates pass with a message in imperative mood, for example
`Pin fragment parser diagnostics`.

### Work Item 2: Share single-list fragment-root validation

Documentation to read before editing:

- The same documents as Work Item 1.
- `docs/execplans/roadmap-5-1-1.md` sections "Constraints", "Decision log",
  and "Outcomes & retrospective", because 5.1.1 deliberately left fragment-root
  parsing for this task.

Skills to load:

- `leta` for `parse_task_fragment_root`, `parse_sub_task_fragment_root`,
  `validate_task_siblings`, and `validate_sub_task_siblings`; if unavailable,
  record the exact failure and use precise file inspection.
- `rust-router`, routed to `rust-types-and-apis` for the helper signature and
  `rust-unit-testing` for test shape.
- `sem` for entity-level review before commit.

In `src/roadmap/parse/fragment.rs`, add a private helper with this shape:

```rust
struct SingleListFragmentMessages {
    single_list: &'static str,
    empty_list: &'static str,
}

fn parse_single_list_fragment<T>(
    root: Root,
    source_text: &str,
    messages: SingleListFragmentMessages,
    parse_list: impl FnOnce(&List, &str) -> Result<Vec<T>>,
    validate: impl FnOnce(&[T]) -> Result<()>,
    wrap: impl FnOnce(Vec<T>) -> RoadmapFragment,
) -> Result<RoadmapFragment>
```

The helper must:

1. Reject any root whose `children.len() != 1` with `messages.single_list`.
2. Reject a sole non-list child with `messages.single_list`.
3. Parse the list with the supplied `parse_list` closure.
4. Reject an empty parsed vector with `messages.empty_list`. This branch is a
   defensive branch preserved verbatim from the current private functions; it
   is unreachable through the public `parse_fragment_text` dispatch because
   `looks_like_task_list` and `looks_like_sub_task_list` require a non-empty
   list before calling the private fragment-root parser.
5. Run the supplied sibling validator.
6. Wrap the vector into the supplied `RoadmapFragment` variant.

Then rewrite `parse_task_fragment_root` and `parse_sub_task_fragment_root` to
call this helper. Preserve the exact current reachable diagnostics:

- `task fragments must contain only a single task list`
- `task fragments must contain tasks from one step`
- `sub-task fragments must contain only a single sub-task list`
- `sub-task fragments must contain sub-tasks from one task`

Also preserve `task fragment list is empty` and
`sub-task fragment list is empty` as the helper's private defensive
`messages.empty_list` values, but do not list them as public acceptance
behaviour unless the implementer chooses the alternative private-test route
below.

Alternative private-test route, only if the implementer decides these defensive
branches must be covered: add an in-module `#[cfg(test)]` unit test inside
`src/roadmap/parse/fragment.rs` that constructs a synthetic mdast `Root` with
one empty `List` and calls the private helper or private root function
directly. If that route is chosen, add `src/roadmap/parse/fragment.rs` to the
touched test-scaffolding surface for this work item and keep the tests private
to the module. Do not attempt to cover these branches through
`tests/roadmap_parse.rs`; the external crate cannot reach the private helper,
and the public parser will not dispatch empty lists to it.

Do not change `parse_phase_fragment`, `parse_step_fragment_root`, operation
routing, or public parser exports in this work item.

Validation for this work item:

```bash
cargo test --test roadmap_parse fragment 2>&1 | tee /tmp/test-mapsplice-roadmap-5-1-2-item-2.out
MARKDOWN_PATHS='docs/execplans/roadmap-5-1-2.md' make markdownfmt 2>&1 | tee /tmp/markdownfmt-mapsplice-roadmap-5-1-2-item-2.out
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-5-1-2-item-2.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-2-item-2.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-2-item-2.out
sem diff --format json 2>&1 | tee /tmp/sem-diff-mapsplice-roadmap-5-1-2-item-2.out
```

Commit after the gates pass with a message in imperative mood, for example
`Share single-list fragment validation`.

### Work Item 3: Share step accumulation between document and fragment parsing

Documentation to read before editing:

- `docs/mapsplice-design.md` sections 2, 4, 5, 6, and 8.
- `docs/developers-guide.md` sections 2, 3, 6, and 7.
- `docs/users-guide.md`, "The roadmap shape `mapsplice` expects".
- `docs/issues/audit-4.2.2.md` findings 13 and 14.
- `AGENTS.md`, "Rust Specific Guidance", especially small modules, clear
  errors, and the 400-line file cap.

Skills to load:

- `leta` for `DocumentParser.begin_step`,
  `DocumentParser.append_task_list`, `DocumentParser.flush_step`,
  `DocumentParser.push_non_structural_node`, `parse_step_fragment_root`,
  `append_step_fragment_tasks`, and `push_step_fragment_body`; if unavailable,
  record the exact failure and use precise file inspection.
- `rust-router`, routed to `rust-types-and-apis`, `rust-errors`, and
  `rust-unit-testing`.
- `sem` for entity-level impact review.

Create a new private module `src/roadmap/parse/step_accumulator.rs` with a
module-level `//!` comment. Add `mod step_accumulator;` in
`src/roadmap/parse/mod.rs`.

Implement a private `StepAccumulator<'source>` that owns:

- `current: Option<StepSection>`
- `source: SourceId`
- `source_text: &'source str`

The accumulator must provide these methods:

- `const fn new(source: SourceId, source_text: &'source str) -> Self`
- `const fn has_active_step(&self) -> bool`
- `fn begin_step(&mut self, number: StepNumber, title: Vec<Node>, completed:
  &mut Vec<StepSection>) -> Result<()>`
- `fn append_task_list(&mut self, list: &List) -> Result<()>`
- `fn push_non_structural_node(&mut self, node: Node) -> Result<()>`
- `fn flush_into(&mut self, completed: &mut Vec<StepSection>)`

`begin_step` must flush any active step into `completed` before constructing a
new `StepSection` with
`ItemIdentity { source: self.source, anchor: number.into() }`,
`MarkdownNodes::from_nodes(title)`, empty body and trailing nodes, and an empty
task vector.

`append_task_list` must preserve the current behaviour from both document and
step-fragment paths:

- if there is no active step, return
  `task list appeared without a current step`;
- if the active step already has trailing content, return the exact current
  diagnostic format with backticks around the displayed step number:

  ```rust
  format!(
      "task list for step `{}` cannot appear after trailing step content",
      current.number
  )
  ```

- parse with `parse_task_list(list, self.source, self.source_text)`;
- validate with `validate_tasks_belong_to_step(current.number, &tasks)`;
- do not call `validate_sub_task_numbers`; remove that helper and its
  document-path call in this same work item because
  `parse_task_item -> parse_sub_task_list -> parse_sub_task_item -> validate_sub_task_number`
  already enforces sub-task parent membership during document parsing, and the
  step-fragment path does not have this duplicate cross-check today;
- append parsed tasks to the active step.

`push_non_structural_node` must preserve the current body/trailing split: nodes
before the first task list go to `StepSection.body`; nodes after task lists go
to `StepSection.trailing`. If there is no active step, return
`step fragments must contain only step sections`. `DocumentParser` must only
call this method after `has_active_step()` is true, so document preamble and
phase body handling remain document-specific.

Refactor `src/roadmap/parse/document.rs` so `DocumentParser` keeps phase and
preamble handling, but delegates active-step state to `StepAccumulator`:

- `begin_phase` flushes any active step into the current phase before moving
  the current phase into the document.
- `begin_step` continues to validate that the step belongs to the current
  phase before delegating to `StepAccumulator::begin_step`.
- `append_task_list` delegates to `StepAccumulator::append_task_list`.
- Remove `validate_sub_task_numbers` and the current
  `DocumentParser::append_task_list` call to it. This intentionally implements
  the finding 14 cleanup inside 5.1.2 because otherwise delegation leaves an
  unreferenced private helper and fails `make all` under `-D warnings`.
- Add or preserve a document parser test proving a sub-task whose task prefix
  does not match its containing task still fails through the parse-time
  `validate_sub_task_number` path after the redundant cross-check is removed.
- `push_non_structural_node` delegates to
  `StepAccumulator::push_non_structural_node` only when an active step exists;
  otherwise it preserves the existing phase body, phase trailing, or preamble
  behaviour.
- `finish` flushes any active step, moves the final phase, and preserves the
  existing empty-document diagnostic.

Refactor `src/roadmap/parse/fragment.rs::parse_step_fragment_root` to use the
same `StepAccumulator`:

- keep fragment root dispatch and `is_step_fragment_start` unchanged;
- on a step heading, parse the heading and call `begin_step`;
- on any other heading, return
  `step fragments must contain only step sections`;
- on a task list, call `append_task_list`;
- on any other node, call `push_non_structural_node`;
- after the loop, flush into the local step vector, reject empty vectors with
  `step fragments must contain only step sections`, and keep
  `validate_step_siblings`.

After this refactor, remove the now-redundant private functions
`append_step_fragment_tasks` and `push_step_fragment_body` from `fragment.rs`.
Also remove `validate_sub_task_numbers` from `document.rs`; it is redundant
with parse-time validation and would otherwise become dead code. Do not remove
`validate_step_siblings`, `validate_task_siblings`, or
`validate_sub_task_siblings`; they remain fragment-level sibling validators.

Validation for this work item:

```bash
cargo test --test roadmap_parse fragment 2>&1 | tee /tmp/test-mapsplice-roadmap-5-1-2-item-3.out
cargo test --test roadmap_golden 2>&1 | tee /tmp/test-golden-mapsplice-roadmap-5-1-2-item-3.out
MARKDOWN_PATHS='docs/execplans/roadmap-5-1-2.md' make markdownfmt 2>&1 | tee /tmp/markdownfmt-mapsplice-roadmap-5-1-2-item-3.out
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-5-1-2-item-3.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-2-item-3.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-2-item-3.out
sem diff --format json 2>&1 | tee /tmp/sem-diff-mapsplice-roadmap-5-1-2-item-3.out
```

Commit after the gates pass with a message in imperative mood, for example
`Share parser step accumulation`.

### Work Item 4: Update roadmap status and close the plan

Documentation to read before editing:

- `docs/roadmap.md` section 5.1.2.
- `docs/documentation-style-guide.md`.
- This ExecPlan's `Progress`, `Decision log`, and `Outcomes & retrospective`
  sections.

Skills to load:

- `execplans`.
- `sem` for final entity-level diff review.
- `commit-message` if making the final commit.

After Work Items 1 through 3 are implemented, gated, and committed, update
`docs/roadmap.md` to mark task 5.1.2 complete. Update this ExecPlan's
`Progress`, `Surprises & discoveries`, `Decision log`, and
`Outcomes & retrospective` sections with final evidence.

Validation for this work item:

```bash
MARKDOWN_PATHS='docs/execplans/roadmap-5-1-2.md docs/roadmap.md' make markdownfmt 2>&1 | tee /tmp/markdownfmt-mapsplice-roadmap-5-1-2-item-4.out
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-5-1-2-item-4.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-2-item-4.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-2-item-4.out
sem diff --format json 2>&1 | tee /tmp/sem-diff-mapsplice-roadmap-5-1-2-item-4.out
```

Commit after the gates pass with a message in imperative mood, for example
`Mark step parser sharing complete`.

## Concrete steps

All commands run from `/home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-2`.

Before editing in an implementation session, retry advisory tool setup:

```bash
git branch --show-current
leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-2
sem entities src/roadmap/parse --format json
```

If Memtrace tools are available, first call
`mcp__memtrace.list_indexed_repositories`, confirm `mapsplice` appears, then use
`repo_id = "mapsplice"` for `find_code`, `find_symbol`, `get_symbol_context`,
`get_impact`, and `get_timeline` before modifying the load-bearing parser
symbols. If the host cancels or rejects the tool again, record the exact
failure in `Progress` and continue with bounded local evidence.

For each work item:

1. Update this ExecPlan's `Progress` with the start timestamp and intended
   files.
2. Add or update the focused tests first.
3. Run the focused red proof with a temporary mutation and save the failing log
   under `/tmp`.
4. Revert the temporary mutation.
5. Make the production-code change.
6. Run the work-item validation commands sequentially.
7. Update this ExecPlan with evidence, format changed Markdown paths, and
   commit the atomic change.

Expected successful gate endings are short. For example:

```plaintext
test result: ok.
```

and:

```plaintext
Finished `test` profile
```

When a gate fails, read the cited `/tmp` log before rerunning. Rerun a gate
only after applying a fix.

## Validation and acceptance

Acceptance is behavioural and structural:

- `parse_task_fragment_root` and `parse_sub_task_fragment_root` no longer carry
  independent copies of the single-list fragment-root skeleton.
- Task and sub-task fragment diagnostics remain byte-for-byte unchanged for
  reachable malformed roots, wrong item kinds, and sibling mismatches.
  `task fragment list is empty` and `sub-task fragment list is empty` remain
  private defensive helper diagnostics, but are not required public acceptance
  behaviours because the current `parse_fragment_text` dispatch cannot reach
  them.
- Document step parsing and step fragment parsing both use the same
  step-accumulation machinery for active step state, task-list ordering, and
  body-versus-trailing routing.
- Step fragment diagnostics remain byte-for-byte unchanged for empty roots,
  non-step headings, task lists without a step, task lists after trailing
  content, tasks from another step, and steps from another phase. The
  trailing-content diagnostic must keep the backticks around the displayed step
  number, for example
  ``task list for step `1.2` cannot appear after trailing step content``.
- Public parser APIs from `docs/developers-guide.md` section 3 remain
  unchanged.
- `make all`, `make markdownlint`, and `make nixie` pass before every commit.

The final validation command set is:

```bash
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-5-1-2-final.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-5-1-2-final.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-5-1-2-final.out
```

For the Red-Green-Refactor record:

- Red: each work item must include a temporary mutation-style failure for the
  focused parser test it relies on.
- Green: the focused parser test must pass after the minimal implementation.
- Refactor: `make all`, `make markdownlint`, and `make nixie` must pass after
  cleanup and before commit.

## Idempotence and recovery

The planned edits are source-code refactors and additive tests. They are safe
to retry from a clean worktree. If a work item fails midway, inspect
`git status --short` and `sem diff --format json`, keep only the files named in
that work item, and revert only the agent's own uncommitted changes when
necessary.

Do not use a bare `git stash`. If temporary formatter or build churn must be
parked, use a named stash such as:

```bash
git stash push -m 'df12-stash v1 task=5.1.2 kind=discard reason="formatter churn"'
```

No destructive commands such as `git reset --hard` or `git checkout -- <path>`
are part of this plan.

## Artefacts and notes

Planning evidence collected in round 1:

```plaintext
git branch --show-current
roadmap-5-1-2
```

```plaintext
mcp__memtrace.list_indexed_repositories
user cancelled MCP tool call
```

```plaintext
leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-2
Error: IO error: Read-only file system (os error 30)
```

```plaintext
mcp__firecrawl.firecrawl_scrape https://docs.rs/markdown/1.0.0/markdown/index.html
user cancelled MCP tool call
```

```plaintext
sem blame src/roadmap/parse/fragment.rs
parse_step_fragment_root      Initial tool (#4)
parse_task_fragment_root      Initial tool (#4)
parse_sub_task_fragment_root  Add operation golden fixtures
```

Planning evidence collected in round 3:

```plaintext
mcp__memtrace.list_indexed_repositories
user cancelled MCP tool call
```

```plaintext
leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-5-1-2
Error: IO error: Read-only file system (os error 30)
```

```plaintext
leta files src/roadmap
Error: Failed to start daemon
```

```plaintext
mcp__firecrawl.firecrawl_scrape https://docs.rs/markdown/1.0.0/markdown/index.html
user cancelled MCP tool call
```

```plaintext
sem log parse_task_fragment_root
59ed7fb  Leynos  2026-06-29  added  Initial tool (#4)
```

## Interfaces and dependencies

The final internal parser interface must include a private single-list helper in
`src/roadmap/parse/fragment.rs` and a private step accumulator in
`src/roadmap/parse/step_accumulator.rs`. These are not public library APIs.

The public APIs named in `docs/developers-guide.md` section 3 must remain
unchanged:

- `run_from_args`
- `run_request`
- `parse_roadmap`
- `parse_fragment`
- `parse_anchor`
- `metrics_snapshot`

No dependency changes are permitted. The implementation uses existing locked
versions from `Cargo.lock`, especially `markdown 1.0.0` and `rstest 0.26.1`.

## Revision notes

- 2026-07-03, planning round 2: revised the plan after design review. The
  trailing-content diagnostic is now pinned with the real backticks around the
  displayed `StepNumber`, and Work Item 3 explicitly removes
  `validate_sub_task_numbers` and its document-path call as a scoped overlap
  with audit finding 14. This keeps the accumulator API small, preserves the
  already-enforced parse-time sub-task invariant, and prevents `dead_code`
  failures under `RUST_FLAGS := -D warnings`.
- 2026-07-03, planning round 3: revised the plan after design review. Work
  Item 1 no longer mandates public passing tests for
  `task fragment list is empty` or `sub-task fragment list is empty`, because
  branch-local parser evidence shows those branches are unreachable through
  `parse_fragment_text`. Work Item 2 now preserves those messages as defensive
  helper branches and documents the optional private-test route if a future
  implementer chooses to cover synthetic empty mdast lists directly.
- 2026-07-03, Work Item 1: added public parser regression tests for the
  reachable fragment-root and step lifecycle diagnostics, recorded the
  mutation-style red proof, and documented the sandbox-network CodeRabbit
  deferral. Work Items 2 through 4 remain unchanged.
- 2026-07-03, Work Item 2: added the private single-list fragment-root helper
  and routed task and sub-task fragment roots through it while preserving
  diagnostics and public APIs. A Clippy repair grouped the helper messages and
  callbacks into `SingleListFragmentParser<T>` instead of the draft
  six-argument helper signature. Work Items 3 and 4 remain unchanged.
- 2026-07-03, Work Item 3: added the private `StepAccumulator` module and
  shared active-step state, task-list ordering, and body-versus-trailing
  routing between document and step-fragment parsing. `begin_step` is
  infallible in the implementation because validation remains at the parser
  boundary. Work Item 4 remains unchanged.
- 2026-07-03, Work Item 4: marked roadmap task 5.1.2 complete and closed this
  ExecPlan with final deterministic evidence, CodeRabbit deferral notes, and
  documentation and skill signposts. The per-item gates passed before commit;
  CodeRabbit deferred because the sandbox had no visible default network route.
