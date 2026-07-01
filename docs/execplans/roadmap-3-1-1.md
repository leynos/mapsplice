# Assemble grammar-surface and per-contract golden fixtures

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: DRAFT

## Purpose / big picture

Roadmap task 3.1.1 is complete when `mapsplice` has a self-contained golden
fixture corpus that proves the supported roadmap grammar surface, every
operation, and each fidelity or contract guarantee that can be exercised by a
single deterministic fixture. A future maintainer should be able to inspect
`tests/fixtures/golden/`, see which guarantee each case covers, run the focused
golden test binary, and receive exact Markdown comparisons or exact
fail-closed assertions.

This plan deliberately does not implement the generated no-op property from
roadmap task 3.1.2, nor the rendered-output Markdown stability sweep from
roadmap task 3.1.3. It does, however, require one exact representative
identity fixture for F3/C5 in task 3.1.1: replacing an item with byte-identical
content must leave the complete roadmap byte-identical. Task 3.1.2 broadens
that pinned example into the property "all conformant fixtures round-trip under
no-op rendering".

Planning evidence on 2026-07-01 shows the assigned worktree at
`/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1`, branch
`roadmap-3-1-1`, with no semantic delta from `origin/main` before this
ExecPlan file was introduced.

## Constraints

- Work only in
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1`.
- Do not edit the root/control worktree at `/home/leynos/Projects/mapsplice`.
- Use absolute paths for edit-tool targets that are not scoped by the assigned
  worktree.
- Treat `origin/main` as canonical and check branch skew before editing.
- Use Memtrace as the primary canonical-main code search and graph tool. First
  call `list_indexed_repositories` and use repo id `mapsplice` only when the
  call confirms it. If the MCP host cancels or rejects the call, record the
  exact failure and continue with bounded branch-local evidence.
- Use `leta` for branch-local symbol navigation, references, and call graphs
  when it is available. If the daemon or workspace setup fails, record the
  exact failure and use precise local inspection for this task.
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
  reference model, renderer, CLI configuration, or public library API unless a
  focused golden fixture exposes a real defect.
- Keep fixture and test source files below 400 lines. Split large case lists
  into helper modules before they approach that limit.
- Fixture files are test inputs, not generated artefacts. Commit them as
  ordinary source-controlled Markdown.
- Format only Markdown files changed by the current work item with
  path-specific formatter commands. Do not run repository-global Markdown
  formatting such as `make fmt` or `mdformat-all`.
- Every test, lint, format check, and gate command must be logged with `tee` to
  a branch-specific `/tmp` file.
- Commit after each work item that changes files, and gate each commit.

## Tolerances (exception triggers)

- If `git branch --show-current` is not `roadmap-3-1-1`, stop before editing.
- If `sem diff --from origin/main --to HEAD --format json` reports
  source-code or roadmap-status changes before implementation starts, inspect
  them and update this plan before editing further.
- If Memtrace, Firecrawl, `leta`, or another advisory tool is unavailable, do
  not mark this plan blocked. Record the exact failed command or tool result
  in `Surprises & Discoveries` and continue with bounded local evidence.
- If a fixture exposes a real production defect, keep the defect fix in the
  same atomic work item as the red fixture that proves it. Stop and revise this
  plan before changing more than one production module for that defect.
- If any work item needs a public API signature change, stop and revise the
  plan before editing the API.
- If any work item needs a new crate, stop and revise the plan with locked
  source and official documentation evidence for that dependency.
- If a work item would touch more than six non-fixture source files, split the
  work item before committing.
- If a Rust or Markdown gate still fails after two focused fix attempts, record
  the failing command, the log path, and the observed error in `Decision Log`
  and stop for review.
- If formatter churn touches files outside the current work item, park or
  discard it with a named stash following
  `df12-stash v1 task=3.1.1 kind=<discard|park|keep> reason="<short>"`.

## Risks

- Risk: Adding every required golden fixture in one commit could create an
  unreviewable test diff.
  Severity: medium.
  Likelihood: high.
  Mitigation: split the corpus into independently committable work items:
  harness, operation fixtures, grammar-surface fixtures, contract fixtures, and
  final roadmap status.

- Risk: A table-driven harness can hide which exact fixture failed.
  Severity: medium.
  Likelihood: medium.
  Mitigation: give every case a stable, descriptive name and make assertion
  failures include the case name, command, expected output, and actual output.

- Risk: The `markdown` crate parses GitHub Flavoured Markdown (GFM) but does
  not provide the exact Markdown writer needed by the fidelity contract.
  Severity: high.
  Likelihood: verified.
  Mitigation: do not depend on a `markdown` writer API. Exercise the existing
  `mapsplice` renderer through the public CLI workflow and compare exact
  Markdown files.

- Risk: The later 3.1.2 property test could need a different fixture discovery
  shape.
  Severity: medium.
  Likelihood: medium.
  Mitigation: store each golden case under a directory with explicit
  `target.md`, optional `fragment.md`, and either `expected.md` for success or
  Rust metadata for typed failures.

- Risk: Some design guarantees are fail-closed rather than successful-output
  cases.
  Severity: medium.
  Likelihood: high.
  Mitigation: include failure fixtures where the design requires an
  adversarial class, and assert typed errors, empty stdout, and no in-place
  write instead of inventing successful output.

## Progress

- [x] (2026-07-01T21:19:04Z) Confirmed the assigned worktree and branch:
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1` on
  `roadmap-3-1-1`.
- [x] (2026-07-01T21:19:04Z) Loaded `execplans`, `leta`,
  `memtrace-first`, `sem`, `firecrawl-mcp`, `rust-router`,
  `rust-unit-testing`, `rust-verification`, and `domain-cli-and-daemons` for
  this planning round.
- [x] (2026-07-01T21:19:04Z) Read source-of-truth documents:
  `AGENTS.md`, `docs/roadmap.md`, `docs/mapsplice-design.md`,
  `docs/developers-guide.md`, `docs/users-guide.md`,
  `docs/contributing.md`, `docs/documentation-style-guide.md`,
  `docs/scripting-standards.md`, and `docs/execplans/initial-tool.md`.
- [x] (2026-07-01T21:19:04Z) Verified `sem diff --from origin/main --to HEAD
  --format json` reported no semantic delta before this plan was added.
- [x] (2026-07-01T21:19:04Z) Verified branch-local fixture and renderer
  orientation with exact file inspection after `leta` failed to start.
- [x] (2026-07-01T21:19:04Z) Revised this plan for design-review round 2:
  C6/F5 now includes stdout, in-place success, and in-place no-write failure;
  F3/C5 now has an exact identity fixture in 3.1.1; missing adversarial
  addendum and dangling cases are added explicitly; formatter recipes now use
  `tee`.
- [x] (2026-07-01T21:42:36Z) Revised this plan for design-review round 3:
  every path-specific Markdown formatter recipe now discovers tracked,
  staged, and untracked changed Markdown files while excluding deleted paths,
  so newly added `tests/fixtures/golden/**/*.md` files cannot bypass
  `mdtablefix` or `markdownlint-cli2 --fix`.
- [x] (2026-07-01T21:41:00Z) Implementation preflight reconfirmed branch
  `roadmap-3-1-1`; `sem diff --from origin/main --to HEAD --format json`
  reported no semantic delta, with only this untracked ExecPlan present.
- [x] (2026-07-01T21:41:00Z) Work item 1 complete: refactored
  `tests/roadmap_golden.rs` around typed case metadata while preserving the
  existing reference-rewrite golden cases. Focused golden tests, `make all`,
  `make markdownlint`, and `make nixie` are green in the harness logs.
- [x] Work item 1: Normalize the golden harness around explicit case metadata.
- [ ] Work item 2: Add successful operation golden fixtures.
- [ ] Work item 3: Add grammar-surface preservation fixtures.
- [ ] Work item 4: Add contract and adversarial fixtures.
- [ ] Work item 5: Mark roadmap 3.1.1 complete after gates.

## Surprises & discoveries

- Memtrace `list_indexed_repositories` returned `user cancelled MCP tool call`
  during planning. Canonical-main graph context was unavailable, so this plan
  uses bounded branch-local evidence through `sem` and exact source
  inspection. This is not a blocker.
- `leta workspace add
  /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1` reported that the
  workspace was already added, but `leta files` failed with
  `Error: Failed to start daemon`. Branch-local symbol navigation was
  unavailable, so this plan uses precise file inspection. This is not a
  blocker.
- Firecrawl `firecrawl_scrape` for
  `https://docs.rs/markdown/1.0.0/markdown/fn.to_mdast.html` returned
  `user cancelled MCP tool call`. External crate claims in this plan are
  pinned to locked local Cargo registry source and the rustdoc comments shipped
  with those crates. This is not a blocker.
- In this round, `leta workspace add
  /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1 && leta files`
  returned `Error: IO error: Read-only file system (os error 30)`. Branch-local
  symbol navigation remained unavailable, so this plan continues to rely on
  precise file inspection for the planning-only edit. This is not a blocker.
- `cargo tree -i` shows locked versions that differ from the manifest minimums
  under caret requirements: `markdown v1.0.0`, `rstest v0.26.1`,
  `proptest v1.11.0`, and `insta v1.48.0`.
- The current `tests/fixtures/reference_rewrite/` corpus contains only
  section-reference, section-reference-outside-`Requires`, version/quantity,
  substring non-match, and multi-id `Requires` fixtures. It does not contain
  addendum renumber, addendum render-fidelity, or dangling-`Requires`
  fixtures; work item 4 must add those explicitly.
- Existing in-place coverage appears in `tests/features/mapsplice.feature` and
  `tests/roadmap_config.rs`, but roadmap task 3.1.1 still needs the golden or
  contract harness to tie stdout and in-place modes directly to the fixture
  corpus acceptance criteria.
- Implementation-session Memtrace `list_indexed_repositories` again returned
  `user cancelled MCP tool call`; canonical-main graph context remained
  unavailable. This is not a blocker under the plan's tool-failure tolerance.
- Implementation-session `leta workspace add
  /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1 && leta files`
  returned `Error: IO error: Read-only file system (os error 30)`, so
  branch-local verification used precise file inspection and `sem` evidence.
- This toolchain's `cargo fmt` does not accept `--workspace`; `cargo fmt
  --all` formatted the Rust workspace without changing unrelated Markdown.
- Scrutineer delegation for work item 1 failed before running gates with:
  `You've hit your usage limit for GPT-5.3-Codex-Spark. Switch to another
  model now, or try again at Jul 7th, 2026 12:20 PM.` Deterministic gates were
  run locally with the same `/tmp` log paths instead.
- The work item 1 CodeRabbit attempt produced only:
  `{"type":"review_context","reviewType":"all","currentBranch":"roadmap-3-1-1","baseBranch":"main","workingDirectory":"/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1"}`
  and
  `{"type":"status","phase":"connecting","status":"connecting_to_review_service"}`
  in `/tmp/coderabbit-mapsplice-roadmap-3-1-1-harness.out`, then remained
  active without further output until it was interrupted with exit code 130.
  This is recorded as a deferred review issue for supervisor follow-up.
- Staging work item 1 failed with
  `fatal: Unable to create
  '/home/leynos/Projects/mapsplice/.git/worktrees/roadmap-3-1-1/index.lock':
  Read-only file system`. The assigned worktree files are writable, but this
  sandbox cannot write the shared control-checkout Git metadata needed to
  stage or commit. Work item 2 must not start until the harness commit is
  created or the git metadata permission issue is resolved.

## Decision log

- Decision: Use directory-per-case golden fixtures under
  `tests/fixtures/golden/<case-name>/`.
  Rationale: A directory can contain `target.md`, optional `fragment.md`, and
  `expected.md` without overloading file suffixes. Failure cases can omit
  `expected.md` and use typed Rust metadata for the expected error.
  Date/Author: 2026-07-01T21:19:04Z / Codex.

- Decision: Keep the golden harness in Rust rather than adding a manifest
  parser.
  Rationale: The repository already has `rstest`, cap-std, camino, and
  `run_from_args` test helpers. A Rust case table avoids a new dependency and
  lets Clippy and typecheck catch command-shape drift.
  Date/Author: 2026-07-01T21:19:04Z / Codex.

- Decision: Use exact file comparisons, not `insta`, for this corpus.
  Rationale: The design requires committed input-and-expected Markdown pairs.
  Plain file fixtures make review and later property iteration clearer than
  generated snapshot names for these Markdown artefacts.
  Date/Author: 2026-07-01T21:19:04Z / Codex.

- Decision: Cover F3/C5 in 3.1.1 with an exact identity operation fixture,
  then leave the generated no-op property to 3.1.2.
  Rationale: There is no public no-op CLI command today, but replacing an item
  with byte-identical content exercises the same parse, operation, renumber,
  rewrite, and render pipeline and must produce byte-identical output. That
  makes 3.1.1's exact comparison concrete without claiming the broader
  generated property early.
  Date/Author: 2026-07-01T21:19:04Z / Codex.

- Decision: Add C6/F5 coverage to the golden or contract harness even though
  older behavioural tests already mention in-place mode.
  Rationale: Roadmap task 3.1.1's acceptance is fixture-corpus based. The
  corpus must therefore prove stdout leaves the target unchanged, in-place
  success writes the expected body and emits no stdout, and in-place failure
  emits no stdout and leaves the target byte-identical.
  Date/Author: 2026-07-01T21:19:04Z / Codex.

## Outcomes & retrospective

Work item 1 replaced the original delete-only golden helper with explicit
private case metadata for command shape, target fixture, optional fragment,
success or typed-failure expectations, and stdout or in-place output modes.
The reusable harness now lives under `tests/golden/` so no Rust source file
exceeds the 400-line project limit. The existing five reference-rewrite
fixtures still pass through the new runner, and harness self-tests pin the
supported command and output-mode metadata surface for later fixture additions.
Validation logs:
`/tmp/test-mapsplice-roadmap-3-1-1-harness.out`,
`/tmp/all-mapsplice-roadmap-3-1-1-harness.out`,
`/tmp/markdownlint-mapsplice-roadmap-3-1-1-harness.out`, and
`/tmp/nixie-mapsplice-roadmap-3-1-1-harness.out`. The item is not committed
because the sandbox cannot write
`/home/leynos/Projects/mapsplice/.git/worktrees/roadmap-3-1-1/index.lock`.

## Context and orientation

`mapsplice` edits constrained roadmap-shaped Markdown by parsing it into a
roadmap model, applying one operation, renumbering affected items, rewriting
dependency references, and rendering Markdown. The design's normative grammar
is summarized in `docs/mapsplice-design.md` section 4 and
`docs/users-guide.md` section "The roadmap shape `mapsplice` expects".

The implementation surfaces relevant to this task are:

- `tests/roadmap_golden.rs`, the existing golden test binary for
  reference-rewrite fixtures.
- `tests/fixtures/reference_rewrite/`, the existing delete-focused
  reference fixtures from roadmap task 1.1.3.
- `tests/support/workspace.rs`, `tests/support/roadmap_workspace.rs`, and
  related helpers, which show the existing capability-scoped test workspace
  style.
- `src/lib.rs::run_from_args` and `src/lib.rs::run_request`, the public CLI
  workflow entry points used by integration tests.
- `src/fs.rs::rewrite_utf8`, which writes through a temporary sibling file and
  renames it into place.
- `src/error.rs::MapspliceError`, the typed diagnostic surface for
  fail-closed fixtures.
- `src/roadmap/ops/mod.rs::RoadmapOperation` and `apply_command`, the domain
  operation surface.
- `src/roadmap/parse/mod.rs::parse_root`, which uses
  `markdown::to_mdast(markdown, &ParseOptions::gfm())`.
- `src/roadmap/render.rs` and `src/roadmap/render_table.rs`, the deterministic
  renderer paths that must preserve task bodies, nested lists, tables, code
  blocks, and addendum sub-tasks.

The relevant source-of-truth requirements are:

- `docs/roadmap.md` section 3.1.1: add one input-and-expected fixture per
  operation and per guarantee, covering preamble, phases, steps, tasks,
  multi-line bodies, nested bullets, tables, and code blocks.
- `docs/mapsplice-design.md` section 4: the accepted roadmap grammar and
  addendum sub-task level.
- `docs/mapsplice-design.md` section 5: fidelity guarantees F1 through F5.
- `docs/mapsplice-design.md` section 6: contract guarantees C1 through C6.
- `docs/mapsplice-design.md` section 7: the dependency-reference model.
- `docs/mapsplice-design.md` section 8: golden corpus, required adversarial
  classes, test shapes, round-trip property, and regression discipline.
- `docs/developers-guide.md` section 6: verification layers and existing test
  frameworks.
- `docs/users-guide.md` sections "Command overview", "Output modes", and
  "Validation rules and failure cases": user-visible CLI and fail-closed
  semantics.
- `AGENTS.md` sections "Change Quality & Committing", "Rust Specific
  Guidance", "Testing", and "Markdown Guidance".
- `docs/documentation-style-guide.md` sections "Spelling", "Markdown rules",
  "Formatting", and "Roadmap task writing guidelines".

## Research evidence

Memtrace was requested first, but `list_indexed_repositories` returned
`user cancelled MCP tool call`. The fallback evidence below is bounded and
branch-local.

The locked `markdown` crate version is 1.0.0. Its local source at
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/lib.rs`
defines `pub fn to_mdast(value: &str, options: &ParseOptions) ->
Result<mdast::Node, message::Message>` at line 160. Its local source at
`.../markdown-1.0.0/src/configuration.rs` defines `ParseOptions::gfm()` at
line 1275 and documents that GFM adds autolink literals, footnotes,
strikethrough, tables, and tasklists. Therefore fixtures may rely on the
existing parser accepting GFM tables, task lists, and fenced code, but must not
rely on the crate for exact Markdown rendering.

The locked `rstest` crate version is 0.26.1. Its local rustdoc source at
`.../rstest-0.26.1/src/lib.rs` documents fixture injection around lines
252-276 and re-exports `rstest_macros::fixture` at line 571. Existing tests
already use `#[fixture]`, `#[rstest]`, and named cases, so the corpus harness
should extend that style.

The locked `proptest` crate version is 1.11.0. Its local source at
`.../proptest-1.11.0/src/collection.rs` defines `collection::vec` at line 205,
and `.../proptest-1.11.0/src/sugar.rs` defines `prop_compose!` at line 624.
Roadmap task 3.1.1 should not add a new property test, but the fixture
directory layout must support task 3.1.2 using proptest or deterministic
iteration over the same valid fixture corpus.

The locked `insta` crate version is 1.48.0. Its local source at
`.../insta-1.48.0/src/macros.rs` defines `assert_snapshot!` at line 463, and
the runtime assertion path starts at `.../insta-1.48.0/src/runtime.rs` line
846. This task deliberately does not use it because committed
input-and-expected Markdown files are the required golden artefacts.

## Plan of work

### Work item 1: Normalize the golden harness around explicit case metadata

Read these documents before editing: `docs/mapsplice-design.md` sections 4, 5,
6, 7, and 8; `docs/developers-guide.md` sections 2, 3, and 6;
`docs/users-guide.md` sections "The roadmap shape `mapsplice` expects",
"Command overview", and "Validation rules and failure cases"; `AGENTS.md`
"Rust Specific Guidance" and "Testing"; and
`docs/documentation-style-guide.md` "Spelling" and "Markdown rules".

Load these skills for this work item: `leta`, `rust-router`,
`rust-unit-testing`, `domain-cli-and-daemons`, `sem`, and
`en-gb-oxendict-style`. Use Memtrace first if it is available in the
implementation session; otherwise record the exact failure and use `leta show`
/ `leta refs` plus bounded file inspection.

Change `tests/roadmap_golden.rs` so it can run a named case with:

- a command shape: append, insert before, insert after, delete, replace, or the
  same command with `--in-place`;
- a `target.md` fixture path;
- an optional `fragment.md` fixture path;
- an `expected.md` fixture path for successful cases;
- an expected typed-error class for fail-closed cases;
- an output-mode assertion: stdout body, stdout with target unchanged,
  in-place success, or in-place failure with target unchanged.

Keep the existing `tests/fixtures/reference_rewrite/` cases passing during the
refactor. Do not move those fixtures in this work item unless the harness keeps
backwards-compatible case definitions for them. Keep setup fallible and
capability-scoped through `cap_std::fs_utf8::Dir` and `camino::Utf8PathBuf`.
Do not use `.unwrap()` in helpers; return `TestResult` and use `?`.

Tests to add or update:

- Unit tests: none, because this is an integration-test harness refactor.
- Behavioural tests: update `tests/roadmap_golden.rs` so the existing five
  reference-rewrite cases still pass through the new case runner.
- Property tests: none in 3.1.1; preserve a fixture layout that 3.1.2 can
  iterate.
- Snapshot tests: none; exact fixture file comparison replaces snapshot macros.
- End-to-end tests: the golden harness must call `run_from_args` so every case
  exercises CLI parsing, file loading, roadmap operations, and rendering.

Validation for this work item:

```bash
cargo test --workspace --all-targets --all-features --test roadmap_golden | tee /tmp/test-mapsplice-roadmap-3-1-1-harness.out
item=harness
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
else
  : | tee "/tmp/mdtablefix-mapsplice-roadmap-3-1-1-${item}.out"
fi
if test -n "$changed_md"; then
  printf '%s\n' "$changed_md" | xargs markdownlint-cli2 --fix 2>&1 \
    | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
else
  : | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
fi
make all | tee /tmp/all-mapsplice-roadmap-3-1-1-harness.out
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-harness.out
make nixie | tee /tmp/nixie-mapsplice-roadmap-3-1-1-harness.out
sem diff --format json | tee /tmp/sem-mapsplice-roadmap-3-1-1-harness.out
```

Commit only after these commands pass.

### Work item 2: Add successful operation golden fixtures

Read these documents before editing: `docs/mapsplice-design.md` section 6,
especially C1, C2, C3, C4, and C6; `docs/users-guide.md` sections "Command
overview", "`append`", "`insert`", "`delete`", "`replace`", and "Output
modes"; `docs/roadmap.md` section 3.1.1; and `AGENTS.md` "Change Quality &
Committing".

Load these skills for this work item: `leta`, `rust-router`,
`rust-unit-testing`, `domain-cli-and-daemons`, `sem`, and
`en-gb-oxendict-style`.

Add one successful stdout-mode golden case per supported operation and mode in
`tests/fixtures/golden/`:

- `append_phase/` proves phase-level append, full renumbering, and stdout mode.
- `insert_phase_before/` proves inserting a phase before an anchor.
- `insert_step_after/` proves `insert --after` at step level.
- `insert_task_before/` proves inserting a task before an anchor.
- `insert_sub_task_after/` proves inserting an addendum sub-task after an
  addendum anchor.
- `delete_task/` proves deleting one task and renumbering later tasks.
- `replace_step/` proves replacing one step with one or more sibling steps.
- `replace_sub_task/` proves replacing an addendum sub-task while preserving
  the parent task.

Each case directory must contain only files that exist for that case:
`target.md`, `expected.md`, and `fragment.md` for operations that take a
fragment. The Rust case metadata must state the exact command arguments, not
infer an operation from the directory name. The expected Markdown must be the
complete rendered output from stdout mode with no trailing-byte ambiguity; if
the harness trims one final newline to match `RunOutcome::stdout`, document
that helper once in code.

Tests to add or update:

- Unit tests: none.
- Behavioural tests: add named golden cases in `tests/roadmap_golden.rs` for
  every operation fixture listed above.
- Property tests: none in this task.
- Snapshot tests: none.
- End-to-end tests: every case goes through `run_from_args`.

Validation for this work item:

```bash
cargo test --workspace --all-targets --all-features --test roadmap_golden | tee /tmp/test-mapsplice-roadmap-3-1-1-operations.out
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
else
  : | tee "/tmp/mdtablefix-mapsplice-roadmap-3-1-1-${item}.out"
fi
if test -n "$changed_md"; then
  printf '%s\n' "$changed_md" | xargs markdownlint-cli2 --fix 2>&1 \
    | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
else
  : | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
fi
make all | tee /tmp/all-mapsplice-roadmap-3-1-1-operations.out
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-operations.out
make nixie | tee /tmp/nixie-mapsplice-roadmap-3-1-1-operations.out
sem diff --format json | tee /tmp/sem-mapsplice-roadmap-3-1-1-operations.out
```

Commit only after these commands pass.

### Work item 3: Add grammar-surface preservation fixtures

Read these documents before editing: `docs/mapsplice-design.md` sections 4, 5,
and 8; `docs/users-guide.md` "The roadmap shape `mapsplice` expects";
`docs/documentation-style-guide.md` "Markdown rules" and "Formatting"; and
`AGENTS.md` "Markdown Guidance".

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
  deterministically and remains gate-clean.
- `code_blocks_preserved/` proves fenced code blocks, language tags, and code
  indentation survive exactly.
- `addendum_body_surface/` proves an addendum sub-task with its own body
  remains nested under its parent task.

Prefer the smallest operation that demonstrates preservation, usually deleting
or inserting an unrelated later task. Do not combine all grammar surfaces into
one fixture; each named case should fail with a narrow diff if a renderer path
regresses.

Tests to add or update:

- Unit tests: none unless a focused fixture reveals an isolated parser or
  renderer defect; if so, add a unit regression beside the production fix.
- Behavioural tests: add named golden cases in `tests/roadmap_golden.rs` for
  every grammar fixture listed above.
- Property tests: none in this task.
- Snapshot tests: none.
- End-to-end tests: every case goes through `run_from_args`.

Validation for this work item:

```bash
cargo test --workspace --all-targets --all-features --test roadmap_golden | tee /tmp/test-mapsplice-roadmap-3-1-1-grammar.out
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
else
  : | tee "/tmp/mdtablefix-mapsplice-roadmap-3-1-1-${item}.out"
fi
if test -n "$changed_md"; then
  printf '%s\n' "$changed_md" | xargs markdownlint-cli2 --fix 2>&1 \
    | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
else
  : | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
fi
make all | tee /tmp/all-mapsplice-roadmap-3-1-1-grammar.out
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-grammar.out
make nixie | tee /tmp/nixie-mapsplice-roadmap-3-1-1-grammar.out
sem diff --format json | tee /tmp/sem-mapsplice-roadmap-3-1-1-grammar.out
```

Commit only after these commands pass.

### Work item 4: Add contract and adversarial fixtures

Read these documents before editing: `docs/mapsplice-design.md` section 5 for
F1 through F5, section 6 for C1 through C6, section 7 for the
dependency-reference model, and section 8 for the required adversarial fixture
classes; `docs/users-guide.md` "Output modes" and "Validation rules and
failure cases"; `docs/developers-guide.md` section 6; and `AGENTS.md`
"Testing" and "Error Handling".

Load these skills for this work item: `leta`, `rust-router`,
`rust-unit-testing`, `rust-verification`, `domain-cli-and-daemons`, `sem`, and
`en-gb-oxendict-style`. `rust-verification` is signposted because this item
decides the boundary between exact example fixtures and the later generated
property; it does not require adding a Kani, Verus, Miri, or proptest harness
in 3.1.1.

Add or migrate named cases so the golden corpus explicitly covers these
contracts:

- `f1_content_preservation/` proves unrelated content is byte-identical except
  for the documented edit consequences.
- `f2_minimal_diff/` proves the only changed numbers are addressed items,
  renumbered later items, and dependency references to those items.
- `f3_c5_identity_replace/` proves the 3.1.1 F3/C5 acceptance criterion:
  replacing an addressed item with byte-identical content emits a complete
  roadmap byte-identical to `target.md`. This is the exact golden comparison
  for F3/C5 in 3.1.1; task 3.1.2 expands it to a generated no-op property over
  every conformant fixture.
- `f4_gate_clean_ready/` adds a conformant fixture containing a table and code
  block that should remain stable under the house Markdown gates in task
  3.1.3.
- `c2_renumber_contiguous/` proves phases, steps, tasks, and addenda are
  contiguous after an edit.
- `c3_requires_rewrite/` proves mapped `Requires` dependencies are rewritten
  and incidental numbers are preserved.
- `c4_addendum_contract/` proves addendum sub-tasks renumber with their parent
  and preserve indentation.
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
  leaves the target unchanged in both stdout and in-place modes where practical.

Keep the existing reference-rewrite classes from
`tests/fixtures/reference_rewrite/` covered: section-reference preservation,
section-reference-outside-`Requires`, version / quantity preservation,
substring non-match, and multi-id `Requires` lists. Do not claim those files
cover addendum renumber, addendum render fidelity, or dangling `Requires`;
those three adversarial classes are absent in the current worktree and must be
added explicitly under `tests/fixtures/golden/` in this work item. The new
explicit adversarial fixtures are:

- `adversarial_addendum_renumber/`, proving `8.2.3.1` tracks its parent task.
- `adversarial_addendum_render_fidelity/`, proving nesting and indentation are
  preserved.
- `adversarial_dangling_requires/`, proving unresolved valid anchors fail
  closed.

The preferred mechanism is to keep old fixtures in place and add golden-case
metadata pointing at them until a later cleanup can move the files without
mixing fixture migration with new coverage. If a fixture is duplicated under
`tests/fixtures/golden/`, delete the duplicate source only in the same commit
and keep every command path valid after deletion.

Tests to add or update:

- Unit tests: add only when a fixture exposes an isolated pure-function defect.
- Behavioural tests: add named golden cases for all listed contract fixtures.
- Property tests: none in 3.1.1; ensure the conformant success cases can be
  enumerated by a later property test without parsing command metadata.
- Snapshot tests: none.
- End-to-end tests: success cases call `run_from_args`; fail-closed cases
  assert `MapspliceError` shape, empty stdout, and unchanged target file when
  applicable.

Validation for this work item:

```bash
cargo test --workspace --all-targets --all-features --test roadmap_golden | tee /tmp/test-mapsplice-roadmap-3-1-1-contracts.out
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
else
  : | tee "/tmp/mdtablefix-mapsplice-roadmap-3-1-1-${item}.out"
fi
if test -n "$changed_md"; then
  printf '%s\n' "$changed_md" | xargs markdownlint-cli2 --fix 2>&1 \
    | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
else
  : | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
fi
make all | tee /tmp/all-mapsplice-roadmap-3-1-1-contracts.out
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1-contracts.out
make nixie | tee /tmp/nixie-mapsplice-roadmap-3-1-1-contracts.out
sem diff --format json | tee /tmp/sem-mapsplice-roadmap-3-1-1-contracts.out
```

Commit only after these commands pass.

### Work item 5: Mark roadmap 3.1.1 complete after gates

Read these documents before editing: `docs/roadmap.md` section 3.1.1,
`docs/documentation-style-guide.md` "Roadmap task writing guidelines", and
`AGENTS.md` "Markdown Guidance".

Load these skills for this work item: `leta`, `sem`, `execplans`, and
`en-gb-oxendict-style`.

After work items 1 through 4 are committed and gated, update only
`docs/roadmap.md` to mark task 3.1.1 complete. Then update this ExecPlan's
`Progress`, `Decision Log`, and `Outcomes & Retrospective` sections with the
final fixture counts, gate log paths, and any follow-up notes for tasks 3.1.2
and 3.1.3. Do not mark 3.1.2 or 3.1.3 complete.

Tests to add or update:

- Unit tests: none.
- Behavioural tests: none beyond rerunning the full corpus.
- Property tests: none.
- Snapshot tests: none.
- End-to-end tests: rerun the golden harness and repository gates.

Validation for this work item:

```bash
cargo test --workspace --all-targets --all-features --test roadmap_golden | tee /tmp/test-mapsplice-roadmap-3-1-1-final.out
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
else
  : | tee "/tmp/mdtablefix-mapsplice-roadmap-3-1-1-${item}.out"
fi
if test -n "$changed_md"; then
  printf '%s\n' "$changed_md" | xargs markdownlint-cli2 --fix 2>&1 \
    | tee "/tmp/markdownlint-fix-mapsplice-roadmap-3-1-1-${item}.out"
else
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
sem diff --from origin/main --to HEAD --format json | tee /tmp/sem-mapsplice-roadmap-3-1-1-preflight.out
```

Expected preflight before implementation starts:

```plaintext
roadmap-3-1-1
{"summary":{"fileCount":1,...
```

After this plan is committed, the only expected semantic branch delta before
implementation is `docs/execplans/roadmap-3-1-1.md`. If additional files are
present, inspect them and update this plan before editing.

For each work item:

1. Update this ExecPlan's `Progress` entry before starting the item.
2. Add the red fixture or harness change first and run the focused
   `cargo test --workspace --all-targets --all-features --test roadmap_golden`
   command with `tee`. If a new fixture is meant to expose a production defect,
   record the expected failure before changing production code.
3. Make the smallest source or fixture changes needed for the focused command
   to pass.
4. Format changed Markdown files only and log both formatter commands:

   ```bash
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
   ```

5. Run `make all`, `make markdownlint`, and `make nixie` with `tee`.
6. Use `sem diff --format json` with `tee` to review entity-level changes
   before commit.
7. Commit the work item with an imperative subject and an explanatory body.

## Validation and acceptance

The required repository validation commands for the completed task are:

```bash
make all | tee /tmp/all-mapsplice-roadmap-3-1-1.out
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-3-1-1.out
make nixie | tee /tmp/nixie-mapsplice-roadmap-3-1-1.out
```

`make all` is required because current `origin/main` includes the `typecheck`
target in the aggregate gate. `make markdownlint` and `make nixie` are
required because the task changes Markdown fixtures, roadmap status, and this
ExecPlan.

The focused acceptance command is:

```bash
cargo test --workspace --all-targets --all-features --test roadmap_golden | tee /tmp/test-mapsplice-roadmap-3-1-1.out
```

Acceptance criteria:

- `tests/fixtures/golden/` contains named input-and-expected cases for every
  supported operation: append, insert before, insert after, delete, and
  replace.
- The corpus covers the grammar surface named in roadmap task 3.1.1: preamble,
  phases, steps, tasks, multi-line bodies, nested bullets, tables, and code
  blocks.
- The corpus covers design guarantees F1 through F5 and C1 through C6 where
  those guarantees have observable output.
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

The `ACMRT` filters exclude deleted tracked paths, and `git ls-files --others
--exclude-standard -- '*.md'` adds new fixture Markdown before it is staged. If
formatter output touches unrelated Markdown files, inspect the diff and use a
named stash with `kind=discard` for unrelated churn before continuing.

If a golden fixture reveals a renderer or parser defect, keep the failing
fixture, add the smallest production fix, and rerun the focused golden test
before broad gates. If the fix exceeds the tolerances above, stop and revise
this plan rather than widening the change silently.

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

$ rg --files tests/fixtures/reference_rewrite | sort
tests/fixtures/reference_rewrite/multi_id_requires.expected.md
tests/fixtures/reference_rewrite/multi_id_requires.input.md
tests/fixtures/reference_rewrite/section_reference.expected.md
tests/fixtures/reference_rewrite/section_reference.input.md
tests/fixtures/reference_rewrite/section_reference_outside_requires.expected.md
tests/fixtures/reference_rewrite/section_reference_outside_requires.input.md
tests/fixtures/reference_rewrite/substring_non_match.expected.md
tests/fixtures/reference_rewrite/substring_non_match.input.md
tests/fixtures/reference_rewrite/version_quantity.expected.md
tests/fixtures/reference_rewrite/version_quantity.input.md
```

Tooling failures recorded for this planning round:

```plaintext
mcp__memtrace.list_indexed_repositories -> user cancelled MCP tool call
leta files -> Error: Failed to start daemon
leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-1
  && leta files -> Error: IO error: Read-only file system (os error 30)
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

Use the existing operation vocabulary exposed by `RoadmapOperation`: `Append`,
`Insert { anchor, after }`, `Delete { anchor }`, and `Replace { anchor }`.

The new or revised golden harness should provide a small internal metadata type
similar to this shape:

```rust
struct GoldenCase {
    name: &'static str,
    command: GoldenCommand,
    target: &'static str,
    fragment: Option<&'static str>,
    expectation: GoldenExpectation,
}
```

This type is an implementation guide, not a public API requirement. Keep it
private to `tests/roadmap_golden.rs` or a submodule under `tests/golden/`.

The fixture directory contract should be:

```plaintext
tests/fixtures/golden/<case-name>/target.md
tests/fixtures/golden/<case-name>/fragment.md
tests/fixtures/golden/<case-name>/expected.md
```

`fragment.md` exists only for append, insert, and replace cases. Failure cases
may use an expected error enum in Rust metadata rather than an output file when
there is no successful rendered body.

## Revision note

Round 2 revision on 2026-07-01: this plan now resolves the design-review
blocking points by requiring explicit C6/F5 stdout, in-place success, and
in-place failure/no-write coverage; defining `f3_c5_identity_replace/` as the
3.1.1 exact F3/C5 comparison while leaving the generated property to 3.1.2;
correcting the existing adversarial corpus inventory; and logging path-specific
Markdown formatter commands with `tee`.

Round 3 revision on 2026-07-01: this plan now resolves the remaining
design-review blocker by replacing the tracked-only changed-Markdown discovery
recipe with a tracked, staged, and untracked discovery recipe. New
`tests/fixtures/golden/**/*.md` files are formatted before staging, deleted
paths remain excluded, and repository gates (`make all`, `make markdownlint`,
and `make nixie`) remain the final path-safe validation commands.
