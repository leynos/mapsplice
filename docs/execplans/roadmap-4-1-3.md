# Single-Source Parse-Domain Task-Number Validation

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: DRAFT

## Purpose / Big Picture

Roadmap task 4.1.3, "Single-source parse-domain task-number validation",
removes duplicated parser validation for the invariant "a task number must
belong to its containing step". After this change, target-roadmap parsing and
step-fragment parsing both call the same parse-domain helper for that invariant,
while task-fragment sibling validation remains a separate rule with its own
diagnostic.

The observable outcome is a small parser refactor whose behaviour does not
change. Focused parser tests pin the exact target and step-fragment diagnostic:
`task \`TASK\` does not belong to step \`STEP\``. They also pin that a top-level
task fragment with mixed step numbers still reports the sibling-fragment
message. Every commit that updates this plan formats
`docs/execplans/roadmap-4-1-3.md` directly and runs the Markdown gates; every
code commit also runs `make all`, which includes the current Cargo typecheck
target on`origin/main`.

## Constraints

- Work only inside
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-1-3`.
- Do not edit the root/control worktree.
- Treat `origin/main` as the integration branch and `docs/roadmap.md` as the
  roadmap source of truth.
- This is planning round 2. Do not begin implementation until this draft is
  approved by the df12-build roadmap workflow.
- Preserve the constrained roadmap grammar in `docs/users-guide.md` "The
  roadmap shape `mapsplice` expects" and "Validation rules and failure cases".
- Preserve `docs/mapsplice-design.md` section 2, which requires mdast-based
  parsing and roadmap-model edits rather than raw-string surgery.
- Preserve `docs/mapsplice-design.md` section 5 (F5): malformed input is
  rejected with typed, user-facing errors before output is produced.
- Preserve `docs/developers-guide.md` section 2: `src/roadmap` owns domain
  parsing, mutation, renumbering, and rendering; CLI command enums stay outside
  the parse-domain helper.
- Preserve `docs/developers-guide.md` section 3: public APIs continue to return
  typed `MapspliceError` values. Do not add, remove, or rename public error
  variants for this task.
- Preserve `docs/execplans/initial-tool.md` decisions to use a constrained
  roadmap grammar, mdast-driven parsing into a roadmap intermediate
  representation, and split parser submodules `parse::{document,fragment}`.
- Do not add external dependencies. Use the locked crates already in
  `Cargo.lock`.
- Keep every Rust source file under 400 lines and keep module-level `//!`
  comments in any new Rust module.
- Follow en-GB Oxford spelling in prose, comments, and commit messages.
- Use Red-Green-Refactor where behaviour changes. Because this task is a
  behaviour-preserving refactor, use mutation-style red proof for tests that
  pin existing behaviour: temporarily break the helper or expected diagnostic,
  run the focused test, record the expected failure, and revert the temporary
  mutation before committing.
- Format only changed Markdown files. Do not run repository-global Markdown
  formatters such as `make fmt` or `mdformat-all`.
- Run tests, lint, and formatting gates sequentially with `tee` logs under
  `/tmp`. Do not run test, lint, or format gates in parallel.
- Use the shared Cargo cache. Do not create an isolated Cargo cache.

## Tolerances

- Stop and escalate if implementation requires a public API signature change, a
  new external dependency, a new error variant, or any change to accepted
  roadmap grammar.
- The planned implementation may touch only these files:
  `docs/execplans/roadmap-4-1-3.md`, `docs/roadmap.md`,
  `src/roadmap/parse/document.rs`, `src/roadmap/parse/fragment.rs`,
  `src/roadmap/parse/mod.rs`, and `tests/roadmap_parse.rs`.
- Stop and escalate if any seventh source, test, or documentation file is
  needed, or if the net implementation exceeds 120 changed lines excluding the
  living ExecPlan evidence.
- Stop and escalate if sharing the helper would force
  `validate_task_siblings` to reuse the target/step-fragment diagnostic. That
  sibling rule is intentionally distinct and must keep
  `task fragments must contain tasks from one step`.
- Stop and escalate if a focused parser test still fails for the same reason
  after three implementation attempts.
- Stop and escalate if `make all` fails for an unrelated pre-existing issue
  that cannot be isolated with a focused command and a log.
- Stop and escalate if advisory-tool unavailability is the only blocker. Record
  the failed tool call and continue with bounded local source and tests.

## Risks

- Risk: a helper placed in `src/roadmap/parse/mod.rs` could make the parse
  module more crowded.
  Severity: low.
  Likelihood: medium.
  Mitigation: add only one small private `pub(super)` helper near the other
  shared parser helpers. Do not create a new module unless the 400-line file
  cap or readability forces it.

- Risk: a shared helper could accidentally replace the task-fragment sibling
  diagnostic.
  Severity: medium.
  Likelihood: medium.
  Mitigation: add a focused test for the sibling diagnostic before refactoring,
  and keep `validate_task_siblings` in `src/roadmap/parse/fragment.rs`.

- Risk: exact error-text tests can be brittle.
  Severity: low.
  Likelihood: medium.
  Mitigation: roadmap task 4.1.3 explicitly requires unchanged error text, so
  exact assertions are the desired contract here.

- Risk: Memtrace or Leta evidence may be unavailable in a sub-agent session.
  Severity: low.
  Likelihood: high.
  Mitigation: this planning round records the exact failures and pins the plan
  to bounded local source windows, `sem` entity output, and focused tests.

## Progress

- [x] (2026-07-02T00:00:00Z) Confirmed the current branch is
  `roadmap-4-1-3`, so this plan is
  `docs/execplans/roadmap-4-1-3.md`.
- [x] (2026-07-02T00:00:00Z) Loaded the `execplans`, `leta`,
  `memtrace-first`, `rust-router`, `rust-unit-testing`, `rust-errors`, and
  `firecrawl-mcp` skills for this planning round.
- [x] (2026-07-02T00:00:00Z) Attempted Memtrace primary discovery with
  `mcp__memtrace.list_indexed_repositories`; the MCP host returned
  `user cancelled MCP tool call`. Continued with bounded branch-local evidence.
- [x] (2026-07-02T00:00:00Z) Attempted Leta workspace setup with
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-1-3`;
  it failed with `Error: IO error: Read-only file system (os error 30)`.
  Branch-local `leta grep` still succeeded for parser symbols, so use Leta as
  the primary verification route during implementation and fall back only if
  the implementation session's Leta command itself fails.
- [x] (2026-07-02T00:00:00Z) Attempted Firecrawl official-doc verification
  with
  `mcp__firecrawl.firecrawl_scrape https://docs.rs/markdown/1.0.0/markdown/fn.to_mdast.html`;
  repeated Firecrawl MCP calls returned `user cancelled MCP tool call`.
  Continued with docs.rs via `curl` and local Cargo registry source for the
  locked crates.
- [x] (2026-07-02T00:00:00Z) Used `sem entities src/roadmap/parse` to list the
  parse-domain entities and `sem blame src/roadmap/parse/document.rs` to
  confirm the target parser helper dates to the initial tool commit.
- [x] (2026-07-02T00:00:00Z) Verified the duplicated invariant locally:
  `src/roadmap/parse/document.rs::validate_task_numbers` and
  `src/roadmap/parse/fragment.rs::validate_task_numbers` have the same
  task-belongs-to-step logic and error text.
- [x] (2026-07-02T00:00:00Z) Verified the distinct sibling invariant locally:
  `src/roadmap/parse/fragment.rs::validate_task_siblings` reports
  `task fragments must contain tasks from one step` and must not be folded into
  the shared helper.
- [ ] Work item 1: add focused parser diagnostic coverage.
- [ ] Work item 2: extract the shared task-belongs-to-step helper.
- [ ] Work item 3: reconcile the roadmap and close the plan.

## Surprises & Discoveries

- Observation: advisory tools were partially unavailable in this sub-agent
  session.
  Evidence: Memtrace and Firecrawl MCP calls returned
  `user cancelled MCP tool call`; Leta failed first with a read-only workspace
  setup error, but `leta grep` still returned branch-local parser symbols.
  Impact: the implementation is still feasible because the affected surface is
  small, known, and pinned by Leta symbol output, local source windows, `sem`
  entity history, and tests.

- Observation: the target parser and step-fragment parser duplicate the exact
  same helper body, but task-fragment sibling validation deliberately has a
  different message.
  Evidence: `src/roadmap/parse/document.rs` lines 223-235 and
  `src/roadmap/parse/fragment.rs` lines 239-251 share the same loop and
  formatted error text; `src/roadmap/parse/fragment.rs` lines 270-283 reports
  the sibling-only text.
  Impact: the shared helper should be used only by
  `DocumentParser::append_task_list` and `append_step_fragment_tasks`.

## Decision Log

- Decision: implement one private parse-domain helper in
  `src/roadmap/parse/mod.rs` with a signature equivalent to
  `validate_tasks_belong_to_step(step_number, tasks)`.
  Rationale: both callers already import `StepNumber`, `TaskEntry`, `Result`,
  and `MapspliceError` through the parse module boundary, and no public API is
  needed.
  Date/Author: 2026-07-02, planning agent.

- Decision: do not merge `validate_task_siblings` into the new helper.
  Rationale: a top-level task fragment containing mixed step numbers is a
  sibling-fragment error, not a containing-step error, and roadmap task 4.1.3
  requires unchanged diagnostics.
  Date/Author: 2026-07-02, planning agent.

- Decision: use focused unit tests, not new behavioural, property, snapshot, or
  end-to-end tests, for the shared helper extraction.
  Rationale: this task changes an internal parser helper while preserving CLI
  behaviour. `docs/developers-guide.md` section 6 says `rstest` unit tests
  cover parser behaviour, while BDD and golden tests cover user workflows and
  render fidelity.
  Date/Author: 2026-07-02, planning agent.

## Outcomes & Retrospective

No implementation has begun. This section must be updated after each approved
work item with the committed change, gate evidence, and any lessons learned.

Round 2 addressed the design-review blocking points by making the Markdown
formatting and gate commands explicit for every work item that updates this
plan, adding `make all` before the work item 1 commit, and replacing `rg`
acceptance checks with Leta-first verification plus an exact-text fallback.

## Context and Orientation

`mapsplice` parses roadmap-shaped Markdown into a domain model, applies one
structural operation, renumbers affected items, rewrites dependency references,
and renders the supported grammar. `src/roadmap/parse/mod.rs` contains shared
parser helpers, `src/roadmap/parse/document.rs` parses complete target
roadmaps, and `src/roadmap/parse/fragment.rs` parses fragment files at one
structural level.

Two parser paths validate the same invariant today:

- Target roadmap path: `DocumentParser::append_task_list` parses a task list,
  calls its local `validate_task_numbers`, and appends the tasks to the current
  step.
- Step-fragment path: `append_step_fragment_tasks` parses a task list, calls
  its local `validate_task_numbers`, and appends the tasks to the current step
  fragment.

Both helpers reject a task number whose `TaskNumber::step_number()` does not
match the containing `StepNumber`. Both format the same diagnostic:

```plaintext
task `<task>` does not belong to step `<step>`
```

`validate_task_siblings` in `src/roadmap/parse/fragment.rs` is a separate
fragment-level rule. It checks that a top-level task fragment contains sibling
tasks from one step and reports:

```plaintext
task fragments must contain tasks from one step
```

Do not change that rule or its message.

External API research is pinned as follows:

- `markdown` is locked to 1.0.0 in `Cargo.lock`; `Cargo.toml` declares
  `markdown = "1.0.0"`. Local source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/lib.rs`
  defines `pub fn to_mdast(value: &str, options: &ParseOptions) ->
  Result<mdast::Node, message::Message>`, and
  `configuration.rs` defines `ParseOptions::gfm()` as GitHub Flavoured
  Markdown with task-list support. The docs.rs page
  `https://docs.rs/markdown/1.0.0/markdown/fn.to_mdast.html` identifies
  `to_mdast` as turning Markdown into a syntax tree.
- `rstest` is locked to 0.26.1 in `Cargo.lock`; `Cargo.toml` declares
  `rstest = "0.26.1"`. Local source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rstest-0.26.1/src/lib.rs`
  documents `#[rstest]` with named `#[case]` arguments and says it generates
  one test for every case. The docs.rs page
  `https://docs.rs/rstest/0.26.1/rstest/attr.rstest.html` describes the
  attribute as supporting fixtures, parameterized cases, and value lists.

No work item should rely on unverified external behaviour beyond those APIs.

## Plan of Work

### Work item 1: Add focused parser diagnostic coverage

Documentation to read before this work item:
`docs/roadmap.md` task 4.1.3, `docs/mapsplice-design.md` sections 4, 5, and 8,
`docs/developers-guide.md` sections 2 and 6, `docs/users-guide.md` "Validation
rules and failure cases", `AGENTS.md` "Testing", and
`docs/documentation-style-guide.md` "Spelling".

Skills to load before editing: `rust-router`, then `rust-unit-testing`. Keep
`rust-errors` in reserve only if the implementation would change error shape;
the expected path should not need it.

Add tests to `tests/roadmap_parse.rs`:

1. A target-roadmap case where a task list under `### 1.1. Step one` contains
   `- [ ] 1.2.1. Wrong step.`. Assert `parse_roadmap_text` returns
   `MapspliceError::InvalidRoadmap` and the message equals
   `task \`1.2.1\` does not belong to step \`1.1\``.
2. A step-fragment case where `### 9.1. Step` contains
   `- [ ] 9.2.1. Wrong step.`. Assert `parse_fragment_text` returns
   `MapspliceError::InvalidRoadmap` with
   `task \`9.2.1\` does not belong to step \`9.1\``.
3. A task-fragment sibling case with one list containing
   `- [ ] 9.1.1. First.` and `- [ ] 9.2.1. Second.`. Assert the message equals
   `task fragments must contain tasks from one step`.

Use `#[rstest]` named cases only if the resulting assertion helper stays
clear. A small local helper such as `invalid_roadmap_message(error)` is
acceptable if it only extracts the message and returns `&str`; do not hide
setup or assertions in a broad helper.

Red proof: before committing, temporarily change one expected diagnostic or
temporarily change one parser message locally, run the focused command below,
observe the expected failure, then revert the temporary mutation. Do not commit
the red state.

Focused validation:

```bash
make test TEST_FLAGS='--workspace --all-targets --all-features parse' 2>&1 | tee /tmp/test-mapsplice-roadmap-4-1-3-item-1.out
```

Because this item also updates `docs/execplans/roadmap-4-1-3.md` with red and
green evidence before committing, format that existing plan file directly and
run the Markdown gates before the commit:

```bash
mdtablefix --in-place docs/execplans/roadmap-4-1-3.md
markdownlint-cli2 --fix docs/execplans/roadmap-4-1-3.md
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-4-1-3-item-1.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-4-1-3-item-1.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-4-1-3-item-1.out
```

Commit after the focused validation, `make all`, and the Markdown gates pass.

### Work item 2: Extract the shared task-belongs-to-step helper

Documentation to read before this work item:
`docs/mapsplice-design.md` sections 2, 3, 4, and 5; `docs/developers-guide.md`
sections 2, 3, and 6; `docs/execplans/initial-tool.md` "Decision Log";
`AGENTS.md` "Rust Specific Guidance"; and `docs/roadmap.md` task 4.1.3.

Skills to load before editing: `rust-router`, `rust-unit-testing`, and
`rust-errors`. `rust-errors` is needed here only to preserve the existing typed
`MapspliceError::InvalidRoadmap` boundary and avoid string-parsing callers.

In `src/roadmap/parse/mod.rs`, add a private shared helper near
`parse_task_list` or near the other validation helpers:

```rust
pub(super) fn validate_tasks_belong_to_step(
    step_number: StepNumber,
    tasks: &[TaskEntry],
) -> Result<()> {
    for task in tasks {
        if task.number.step_number() != step_number {
            return Err(MapspliceError::InvalidRoadmap {
                message: format!(
                    "task `{}` does not belong to step `{}`",
                    task.number, step_number
                ),
            });
        }
    }
    Ok(())
}
```

In `src/roadmap/parse/document.rs`, import the helper from `super`, call it
from `DocumentParser::append_task_list`, and delete the local
`validate_task_numbers`.

In `src/roadmap/parse/fragment.rs`, import the helper from `super`, call it
from `append_step_fragment_tasks`, and delete the local
`validate_task_numbers`. Leave `validate_task_siblings`,
`validate_step_siblings`, and `validate_sub_task_siblings` in place.

Refactor validation:

```bash
make test TEST_FLAGS='--workspace --all-targets --all-features parse' 2>&1 | tee /tmp/test-mapsplice-roadmap-4-1-3-item-2.out
mdtablefix --in-place docs/execplans/roadmap-4-1-3.md
markdownlint-cli2 --fix docs/execplans/roadmap-4-1-3.md
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-4-1-3-item-2.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-4-1-3-item-2.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-4-1-3-item-2.out
```

Acceptance for this item:

- `leta grep "^validate_task_numbers$" "src/roadmap/parse" -k function`
  prints no matches. If Leta fails in the implementation session, use exact
  text fallback
  `grep -R -n "fn validate_task_numbers" src/roadmap/parse` and expect no
  output.
- `leta grep "^validate_tasks_belong_to_step$" "src/roadmap/parse" -k function`
  shows the helper definition in `src/roadmap/parse/mod.rs`. If Leta fails,
  use exact text fallback
  `grep -R -n "fn validate_tasks_belong_to_step" src/roadmap/parse`.
- `leta refs validate_tasks_belong_to_step` shows calls from
  `src/roadmap/parse/document.rs` and `src/roadmap/parse/fragment.rs`. If Leta
  fails, use exact text fallback
  `grep -R -n "validate_tasks_belong_to_step" src/roadmap/parse`.
- The focused parser tests from work item 1 still pass.
- `make all` passes.
- `make markdownlint` and `make nixie` pass because this work item updates the
  ExecPlan evidence before committing.

Commit this work item after validation passes.

### Work item 3: Reconcile the roadmap and close the plan

Documentation to read before this work item:
`docs/roadmap.md` task 4.1.3, `docs/developers-guide.md` section 7,
`docs/contributing.md` "Development gates", `docs/documentation-style-guide.md`
"Markdown rules", and `AGENTS.md` "Markdown Guidance".

Skills to load before editing: `execplans`. No Rust router skill is needed if
this item only edits Markdown after the Rust implementation is complete.

Update `docs/roadmap.md` to mark `4.1.3` complete only after work item 2 is
green and committed. Update this ExecPlan's `Progress`, `Decision Log`, and
`Outcomes & Retrospective` with the focused-test and gate evidence. Set
`Status: COMPLETE` only when the task is actually complete and no required
work remains.

Format only the Markdown files changed by this work item. If both files are
edited, run:

```bash
mdtablefix --in-place docs/roadmap.md docs/execplans/roadmap-4-1-3.md
markdownlint-cli2 --fix docs/roadmap.md docs/execplans/roadmap-4-1-3.md
```

If only this ExecPlan is edited, run:

```bash
mdtablefix --in-place docs/execplans/roadmap-4-1-3.md
markdownlint-cli2 --fix docs/execplans/roadmap-4-1-3.md
```

Final validation:

```bash
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-4-1-3-final.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-4-1-3-final.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-4-1-3-final.out
```

Commit this item after the formatter and final gates pass.

## Concrete Steps

1. Confirm a clean baseline:

   ```bash
   git status --short
   git branch --show-current
   ```

   Expected branch: `roadmap-4-1-3`. If `git status --short` is not empty,
   inspect ownership before editing.

2. Implement work item 1 and record red-proof and green-test evidence in this
   plan before committing. Run the path-scoped plan formatter commands,
   `make all`, `make markdownlint`, and `make nixie` before the commit.

3. Implement work item 2, run focused parser tests, record evidence in this
   plan, run the Leta-first acceptance checks, then run the path-scoped plan
   formatter commands, `make all`, `make markdownlint`, and `make nixie`
   before committing.

4. Implement work item 3, run the path-safe Markdown formatter commands for
   files that exist and were edited, then run `make all`, `make markdownlint`,
   and `make nixie`.

5. After every commit, run `git status --short`. Expected output is empty.

## Validation and Acceptance

Quality criteria:

- Unit tests: focused parser diagnostics in `tests/roadmap_parse.rs` prove the
  shared invariant and the separate sibling diagnostic.
- Behavioural, snapshot, and end-to-end tests: no new tests are required
  because the CLI contract and rendered output do not change.
- Property tests: no new property test is required because this task extracts
  an internal validation helper and does not expand generated input space.
- Lint/typecheck: `make all` passes. On current `origin/main`, `make all`
  includes `check-fmt`, `lint`, `typecheck`, and `test`.
- Markdown: `make markdownlint` and `make nixie` pass after documentation
  changes.

Acceptance behaviour:

- Target roadmap parsing and step-fragment parsing both reject tasks whose
  number belongs to a different step with unchanged text:
  `task \`TASK\` does not belong to step \`STEP\``.
- Top-level task-fragment sibling validation remains separate and still
  reports `task fragments must contain tasks from one step`.
- `src/roadmap/parse/document.rs` and `src/roadmap/parse/fragment.rs` no
  longer each define their own task-belongs-to-step helper.
- No public API or user-facing command behaviour changes.

## Idempotence and Recovery

All work items are safe to retry from a clean working tree. If a mutation-style
red proof leaves a temporary edit behind, revert only that temporary edit before
continuing; do not use `git reset --hard` or overwrite unrelated user work.

If Markdown formatting changes unrelated files, park that churn in a named
stash before committing:

```bash
git stash push -m 'df12-stash v1 task=4.1.3 kind=discard reason="formatter-churn"' -- <paths>
```

If a gate fails, read the `/tmp` log named by the command before rerunning. Only
rerun after changing something relevant or confirming the prior failure was
environmental.

## Artifacts and Notes

Planning evidence:

```plaintext
Memtrace: mcp__memtrace.list_indexed_repositories -> user cancelled MCP tool call
Leta: leta workspace add <worktree> -> Error: IO error: Read-only file system (os error 30)
Leta: leta grep parser symbols -> succeeded for append_task_list,
validate_task_numbers, append_step_fragment_tasks, and validate_task_siblings
Firecrawl scrape docs.rs markdown to_mdast/rstest -> user cancelled MCP tool call
curl docs.rs markdown/rstest pages -> official rustdoc pages reachable
sem entities src/roadmap/parse -> listed both validate_task_numbers helpers
sem blame --json src/roadmap/parse/document.rs -> helper introduced by initial tool commit 59ed7fb6
sem blame --json src/roadmap/parse/fragment.rs -> fragment helper introduced by initial tool commit 59ed7fb6
```

## Interfaces and Dependencies

No public interface changes are allowed.

The final internal helper should be visible only inside `src/roadmap/parse`:

```rust
pub(super) fn validate_tasks_belong_to_step(
    step_number: StepNumber,
    tasks: &[TaskEntry],
) -> Result<()>;
```

Dependencies remain unchanged:

- `markdown` 1.0.0 continues to parse Markdown into mdast through
  `to_mdast(markdown, &ParseOptions::gfm())`.
- `rstest` 0.26.1 continues to provide focused parameterized parser tests.
- No `hypothesis`, `crosshair`, `mutmut`, `proptest`, or new verification
  dependency is required for this internal Rust parser refactor.

## Revision Note

- 2026-07-02: Created the first draft for roadmap task 4.1.3. The draft
  records advisory-tool failures, pins the local parser surface, and decomposes
  implementation into independently committable parser-test, helper-refactor,
  and roadmap-closeout work items. Implementation has not begun.
- 2026-07-02: Revised for planning round 2 after design review. The revision
  adds explicit path-scoped Markdown formatting and Markdown gates to work
  items 1 and 2, adds `make all` before the work item 1 commit, and changes
  work item 2 acceptance checks to use Leta first with exact text search only
  as a fallback. Implementation has not begun.
