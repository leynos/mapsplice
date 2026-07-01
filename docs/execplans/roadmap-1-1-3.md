# Pin corruption cases with regression fixtures

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

## Purpose / big picture

Roadmap task 1.1.3 makes the corruption cases from the design document
impossible to lose in future refactors. After this task, `mapsplice` has
checked-in golden fixtures for the adversarial reference-rewrite cases:
section-reference preservation, semantic-version and quantity preservation,
substring non-match, and multi-id `Requires` lists. A successful change is
observable by running the focused golden-fixture tests and seeing exact
input-to-expected output comparisons pass, then running the repository gates.

This plan covers only roadmap task 1.1.3, "Pin the corruption cases with
regression fixtures." It must not change the dependency-reference semantics
unless a new fixture exposes a still-valid bug. If a production fix is needed,
it must be the smallest fix required by the failing fixture and must stay inside
the reference-rewrite seam already established by tasks 1.1.1 and 1.1.2.

## Context and orientation

The repository is a Rust command-line tool. The relevant files are:

- `src/roadmap/ops/dependency_text.rs`, where text-node anchor candidates are
  classified and rewritten.
- `src/roadmap/ops/rewrite.rs`, where the renumber plan is applied to roadmap
  model Markdown nodes.
- `tests/roadmap_ops.rs`, which already contains focused operation tests for
  dependency-reference preservation and rewriting.
- `tests/roadmap_properties.rs`, which already contains generated preservation
  properties.
- `tests/features/mapsplice.feature`, `tests/behaviour_cli.rs`, and
  `tests/steps/cli_steps.rs`, which exercise the compiled binary through
  `rstest-bdd`.
- The new fixture corpus should live under
  `tests/fixtures/reference_rewrite/`, with exact `.input.md` and
  `.expected.md` pairs, plus a small test harness in `tests/roadmap_golden.rs`
  or an equivalent existing test file if the implementer can keep the change
  clearer and smaller.

The documented source of truth is:

- `AGENTS.md`, especially "Code Style and Structure", "Change Quality &
  Committing", "Rust Specific Guidance", "Testing", "Error Handling", and
  "Markdown Guidance".
- `docs/mapsplice-design.md` sections 3, 5, 6, 7, 8, and 9. Section 7 defines
  the dependency-reference model. Section 8 requires golden fixtures for the
  adversarial corruption cases. Section 9 records the historical unscoped
  rewrite divergence.
- `docs/developers-guide.md` sections 2, 3, 6, and 7. Section 6 says
  dependency-reference rewrite coverage is layered around
  `classify_dependency_reference` in
  `src/roadmap/ops/dependency_text.rs`.
- `docs/users-guide.md` sections "The roadmap shape `mapsplice` expects",
  "Worked example", "Output modes", and "Validation rules and failure cases".
- `docs/documentation-style-guide.md` sections "Spelling", "Markdown rules",
  and "Formatting".
- `docs/rstest-bdd-users-guide.md` sections "Gherkin feature files", "Step
  definitions", and "Fixtures and implicit injection".
- `docs/rust-testing-with-rstest-fixtures.md` sections 1 and 2 for fixture and
  parameterized-test style.
- `docs/roadmap.md` task 1.1.3 and the phase 1 / step 1.1 hypothesis.

Important terminology:

- A golden fixture is a checked-in input document and expected output document
  compared byte-for-byte by a test.
- A dependency context is a `Requires` clause. Per the design, only references
  in this context may be rewritten.
- An incidental number is a number-shaped token that is not a dependency
  reference, such as `§3.2`, `1.4.0`, or a prose quantity.

## Research and verified mechanisms

Memtrace is required as the primary main-branch code-search tool, but the MCP
call failed in this planning session. The attempted call was
`mcp__memtrace.list_indexed_repositories({})`; the host returned:

```plaintext
user cancelled MCP tool call
```

This is a tooling failure, not a product blocker. At the start of
implementation, retry Memtrace. If repo_id `mapsplice` appears, use
`list_communities` or `find_central_symbols` for orientation, then
`find_symbol`, `get_symbol_context`, `get_impact`, and `get_timeline` for
`rewrite_text_value`, `classify_dependency_reference`, and
`rewrite_dependencies` before any production edit. Treat Memtrace's committed
view as canonical main-branch context. Verify branch-local facts directly in
the worktree.

Leta is required for branch-local symbol navigation. This planning session ran:

```bash
leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-3
leta files
```

The workspace add succeeded. `leta files` failed with:

```plaintext
Error: Failed to start daemon
```

During the round 2 revision, retrying the combined command
`leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-3
&& leta files | head -n 200` failed earlier with:

```plaintext
Error: IO error: Read-only file system (os error 30)
```

This is a tooling failure, not a product blocker. At implementation start,
retry `leta files`, `leta show`, `leta refs`, and `leta calls` for the
load-bearing symbols above. If the daemon still fails, record the exact failure
in this plan and use bounded file inspection.

Semantic history was available. `sem log rewrite_text_value --limit 8` showed
`rewrite_text_value` first appearing in commit `59ed7fb` and later modified in
commit `7685640`. `sem log classify_dependency_reference --limit 8` showed
`classify_dependency_reference` added in commit `7685640`. Branch-local
`git log --oneline --max-count=8` showed `04dba76 Scope dependency reference
rewrites` at `HEAD` / `origin/main`. For historical context only,
`git show 59ed7fb:src/roadmap/ops/dependency_text.rs` showed the old
`rewrite_text_value` returning `Result<(String, u64)>` and raising
`MapspliceError::DanglingDependency` when an anchor-shaped token in a
`Requires` clause resolved to no mapping. Do not reintroduce that behaviour.

Firecrawl was required for external documentation research, but the attempted
call:

```plaintext
mcp__firecrawl.firecrawl_scrape(
  https://docs.rs/rstest/0.26.1/rstest/attr.rstest.html
)
```

returned:

```plaintext
user cancelled MCP tool call
```

This is a tooling failure, not a product blocker. I used local locked crate
source and reachable docs.rs pages as fallback evidence. Implementation should
retry Firecrawl for the docs.rs pages before editing if the tool is available.

The plan does not add dependencies. It leans only on these locked crates and
verified APIs:

- `markdown` is locked to `1.0.0` in `Cargo.lock`. The docs.rs page
  <https://docs.rs/markdown/1.0.0/markdown/fn.to_mdast.html> and local source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/lib.rs`
  lines 140-164 define `to_mdast(value, options) -> Result<Node, Message>` and
  describe it as turning Markdown into a syntax tree. Local source
  `markdown-1.0.0/src/mdast.rs` lines 827-842 defines `Text { value:
  String, position: Option<Position> }`. Therefore fixture tests should
  exercise the public CLI or existing parse/render boundary, not raw-string
  replacement.
- `rstest` is locked to `0.26.1`. Local source
  `rstest-0.26.1/src/lib.rs` lines 20-27 says `#[rstest]` supports fixtures,
  input tables and values, and `#[fixture]` marks fixture functions; lines
  102-110 describe fixture injection by function arguments; lines 162-190 show
  `#[case]` parameterized tests generate one test per case. The focused
  commands in this plan require exact Cargo test names, so the golden harness
  must use `#[rstest]` fixture injection on standalone test functions named
  exactly after the fixture cases, with a shared private helper for the common
  logic. Do not put the golden fixture cases behind one parameterized
  `#[case]` function unless the implementer first records the generated names
  from `cargo test --test roadmap_golden -- --list` and updates every focused
  command to those exact generated names.
- `proptest` is locked to `1.11.0` in `Cargo.lock`, while `Cargo.toml` uses the
  caret requirement `proptest = "1.9.0"`. Local source
  `proptest-1.11.0/src/prelude.rs` lines 23-30 re-exports `Strategy`,
  `ProptestConfig`, `prop_assert`, `prop_assert_eq`, and `proptest`.
  Therefore generated preservation tests may continue using the existing
  `proptest::prelude::*` style, but this task's primary artefact must be
  named golden fixtures.
- `rstest-bdd` and `rstest-bdd-macros` are locked to `0.5.0`. The docs.rs page
  <https://docs.rs/rstest-bdd/0.5.0/rstest_bdd/> describes `rstest-bdd` as
  exposing helper utilities and the global step registry. Local source
  `rstest-bdd-macros-0.5.0/src/lib.rs` lines 56-140 defines the `#[given]`,
  `#[when]`, `#[then]`, and `#[scenario]` attribute macros and scenario selector
  semantics. Therefore any BDD coverage should follow the existing
  `tests/features/mapsplice.feature` and `tests/steps/cli_steps.rs` pattern.

## Constraints

- Work only inside
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-3`.
- Do not edit the root/control worktree.
- Do not begin implementation until this draft ExecPlan is approved by the
  roadmap workflow.
- Preserve the public API and CLI syntax unless a fixture exposes a bug that
  cannot be fixed otherwise. If that happens, stop and escalate before changing
  the API.
- Do not add a dependency. The locked test dependencies are sufficient.
- Keep the dependency-context limit from `docs/mapsplice-design.md` section 10:
  only `Requires` is a rewrite context today.
- Keep unresolved valid dependency references unchanged. Diagnostics for
  dangling dependencies belong to roadmap task 4.1.2.
- Add golden fixtures; do not replace all existing focused unit, property, or
  BDD coverage with fixture tests.
- Follow Red-Green-Refactor in each work item. Because task 1.1.2 is already
  present on `origin/main`, a new correct fixture may pass on current code as
  soon as it is wired. In that case, prove the harness can fail by first
  running the new fixture test with an intentionally wrong expected output,
  then correct the expected output before committing.
- Use en-GB Oxford spelling in comments, docs, and commits.
- Format only changed Markdown files. Do not run `make fmt`, `mdformat-all`,
  or any repo-global formatter during this task.
- Do not run formatters, lints, or tests in parallel. Capture long command
  output with `tee` under `/tmp`.
- Every gate command piped through `tee` must run under `set -o pipefail` in
  the same shell block.

## Tolerances

- Scope: if the implementation needs changes outside
  `tests/fixtures/reference_rewrite/`, `tests/roadmap_golden.rs`,
  `tests/roadmap_ops.rs`, `tests/roadmap_properties.rs`,
  `tests/features/mapsplice.feature`, `tests/behaviour_cli.rs`,
  `tests/steps/cli_steps.rs`, `src/roadmap/ops/dependency_text.rs`,
  `src/roadmap/ops/rewrite.rs`, `docs/roadmap.md`, and this ExecPlan, stop and
  record the reason before escalating.
- Production size: if production Rust changes exceed 60 net lines, stop and
  escalate. This task should mostly add fixtures and tests.
- Interface: if a public API, CLI argument, fixture naming convention, or
  roadmap grammar rule must change, stop and escalate.
- Dependencies: if a new dependency appears necessary, stop and escalate.
- Red stage: if a new fixture cannot be made to fail by a wrong expected file
  or by a still-valid production bug after two attempts, record the discovery
  and proceed only if the current implementation already satisfies the fixture.
- Gates: if `make all`, `make markdownlint`, or `make nixie` fails twice for
  reasons attributable to this task, stop and escalate with the logs.
- File size: if a new or touched Rust source file would exceed the 400-line
  limit in `AGENTS.md`, split helpers by feature before committing.

## Risks

- Risk: existing task 1.1.2 coverage already proves much of the behaviour, so
  new correct fixtures may be green immediately. Severity: medium. Likelihood:
  high. Mitigation: use wrong-expected-output red checks to prove the fixture
  harness, then commit only the correct expected output.
- Risk: fixture files can become broad snapshots that obscure the exact
  contract under test. Severity: medium. Likelihood: medium. Mitigation: keep
  one small input/expected pair per adversarial class, and name the case after
  the design table row it proves.
- Risk: a golden fixture might accidentally rely on current renderer
  normalization rather than the reference-rewrite contract. Severity: medium.
  Likelihood: medium. Mitigation: keep each fixture minimal and use the same
  operation shape, usually `delete phase 1`, so the only intended differences
  are renumbering plus dependency-reference rewriting.
- Risk: adding BDD and property coverage in the same commit can make failures
  hard to attribute. Severity: medium. Likelihood: low. Mitigation: keep the
  work items independently committable and gate each one.

## Progress

- [x] (2026-07-01) Drafted first planning-round ExecPlan for task 1.1.3.
- [x] (2026-07-01) Revised the plan after design review round 2 to make
  focused golden-test commands exact-name safe and to keep Gherkin feature
  files out of Markdown formatter and linter commands.
- [x] (2026-07-01) Work item 1: Introduce the exact golden-fixture harness.
- [x] (2026-07-01) Work item 2: Add incidental-number preservation fixtures.
- [x] (2026-07-01) Work item 3: Add substring and multi-id `Requires`
  fixtures.
- [x] (2026-07-01) Work item 4: Add behavioural and property backstops for the
  corpus.
- [x] (2026-07-01) Work item 5: Mark roadmap task 1.1.3 complete after gates.

## Surprises & discoveries

- Observation: Memtrace was unavailable in planning.
  Evidence: `mcp__memtrace.list_indexed_repositories({})` returned
  `user cancelled MCP tool call`.
  Impact: the plan requires a retry, but uses bounded local evidence as the
  fallback.
- Observation: Leta workspace registration succeeded, but the daemon failed
  when listing files.
  Evidence: `leta files` returned `Error: Failed to start daemon`.
  Impact: the plan requires a retry, but allows precise file inspection if the
  daemon remains unavailable.
- Observation: the round 2 Leta retry failed before listing files because the
  tool attempted to write in a read-only filesystem location.
  Evidence: `leta workspace add
  /home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-3 && leta files | head
  -n 200` returned `Error: IO error: Read-only file system (os error 30)`.
  Impact: this remains an advisory-tool failure and not a product blocker.
- Observation: task 1.1.2 is already at `HEAD` / `origin/main`.
  Evidence: `git log --oneline --max-count=8` showed `04dba76 Scope dependency
  reference rewrites` at `HEAD`.
  Impact: new correct fixtures may pass immediately on current code; the red
  proof should first use a deliberately wrong expected output.
- Observation: design review round 2 found that exact Cargo filters are unsafe
  with one parameterized `#[rstest]` golden test because generated case names
  do not equal plain fixture names such as `section_reference`.
  Evidence: the review noted `cargo test --test roadmap_golden -- <case>
  --exact` could run zero tests if the harness used named `#[case]` values.
  Impact: this plan now requires standalone exact test functions for the
  golden cases.
- Observation: design review round 2 confirmed that
  `tests/features/mapsplice.feature` is not Markdown.
  Evidence: direct verification against the feature file failed Markdown rules
  MD041 and MD013.
  Impact: this plan validates Gherkin through the BDD test path and repository
  gates, not direct Markdown formatter or linter commands.
- Observation: implementation retry of Memtrace failed before repository
  discovery.
  Evidence: `mcp__memtrace.list_indexed_repositories({})` returned
  `user cancelled MCP tool call`.
  Impact: work item 1 used bounded branch-local inspection and `sem` history
  evidence rather than Memtrace graph context.
- Observation: implementation retry of Leta failed before branch-local file
  listing.
  Evidence: `leta workspace add
  /home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-3 && leta files | head
  -n 240` returned `Error: IO error: Read-only file system (os error 30)`.
  Impact: work item 1 used precise file inspection after recording the
  advisory-tool failure.
- Observation: the checked-in Markdown expected fixture carries the normal
  storage newline, while `mapsplice` renders standard-output roadmaps without
  a final newline.
  Evidence: the red check for `section_reference` showed the rendered output
  ended at `Requires §2.1, 1.1.1.` while the file-backed expected string had a
  trailing line feed.
  Impact: the golden harness removes one storage final line feed from expected
  fixtures before comparing the rendered stdout bytes.
- Observation: CodeRabbit did not complete for work item 1.
  Evidence: `coderabbit review --agent` reported
  `connecting_to_review_service`, then the second attempt exited with status
  124 after a 90-second timeout and produced no file-level findings.
  Impact: AI review for this work item is deferred as an open tooling issue;
  deterministic gates are green.
- Observation: CodeRabbit did not complete for work item 2.
  Evidence: `coderabbit review --agent` again stopped at
  `connecting_to_review_service` and timed out after 120 seconds without a
  review payload.
  Impact: AI review for this work item is deferred as the same open tooling
  issue; deterministic gates are green.
- Observation: CodeRabbit did not complete for work item 3.
  Evidence: `coderabbit review --agent` stopped at
  `connecting_to_review_service` and timed out after 120 seconds without a
  review payload.
  Impact: AI review for this work item is deferred as the same open tooling
  issue; deterministic gates are green.
- Observation: CodeRabbit did not complete for work item 4.
  Evidence: `coderabbit review --agent` stopped at
  `connecting_to_review_service` and timed out after 30 seconds without a
  review payload.
  Impact: AI review for this work item is deferred as the same open tooling
  issue; deterministic gates are green.
- Observation: post-update `make nixie` retries timed out on untouched guide
  diagrams after work item 4.
  Evidence: `/tmp/nixie-mapsplice-roadmap-1-1-3-work-item-4-rerun.out`
  timed out on `docs/ortho-config-users-guide.md` diagram 1,
  `/tmp/nixie-mapsplice-roadmap-1-1-3-work-item-4-serial.out` timed out on
  `docs/documentation-style-guide.md` diagram 1 even with
  `NIXIE='nixie --max-concurrency 1'`, and
  `/tmp/nixie-mapsplice-roadmap-1-1-3-work-item-4-mmdc.out` failed because
  the alternate `mmdc` renderer could not launch its browser process.
  Impact: the work item changed no guide diagrams; the touched ExecPlan file
  passed `nixie --no-sandbox docs/execplans/roadmap-1-1-3.md`, so the
  remaining full-repository Nixie failure is tracked as an unrelated renderer
  environment issue.
- Observation: CodeRabbit did not complete for work item 5.
  Evidence: `/tmp/coderabbit-mapsplice-roadmap-1-1-3-final.out` stopped at
  `connecting_to_review_service` and timed out after 120 seconds with exit
  status 124.
  Impact: final AI review is deferred as the same open tooling issue; final
  deterministic gates are green.

## Decision log

- Decision: implement task 1.1.3 as a checked-in fixture corpus plus a narrow
  exact-comparison harness, not as another production rewrite by default.
  Rationale: `docs/mapsplice-design.md` section 8 explicitly requires golden
  fixtures, while task 1.1.2 has already landed the scope behaviour.
  Date/Author: 2026-07-01 / planning agent.
- Decision: place fixtures under `tests/fixtures/reference_rewrite/`.
  Rationale: the directory groups the adversarial C3/F1 reference cases by
  feature and avoids mixing large Markdown literals into ordinary test code.
  Date/Author: 2026-07-01 / planning agent.
- Decision: keep `proptest` as a backstop, not the primary deliverable.
  Rationale: roadmap task 1.1.3 asks for regression fixtures; generated tests
  cannot replace named corruption examples.
  Date/Author: 2026-07-01 / planning agent.
- Decision: use standalone golden test function names instead of a single
  parameterized `#[rstest]` case table for the fixture corpus.
  Rationale: Cargo's `--exact` filter must match the generated test name. The
  design review found that `#[case]` names such as `section_reference` are not
  guaranteed to be the full exact test names, so standalone functions named
  `section_reference`, `version_quantity`, `substring_non_match`, and
  `multi_id_requires` are the safer, reviewable command contract.
  Date/Author: 2026-07-01 / planning agent.
- Decision: do not run Markdown formatters or Markdown linters directly on
  `tests/features/mapsplice.feature`.
  Rationale: it is a Gherkin file and currently fails Markdown-only rules; its
  syntax and behaviour are validated through `cargo test --test behaviour_cli`
  and the repository gates.
  Date/Author: 2026-07-01 / planning agent.
- Decision: keep expected fixtures as Markdown files with the house final
  newline and remove exactly one trailing line feed in the comparison helper.
  Rationale: this keeps fixture files gate-clean under Markdown tooling without
  changing the existing CLI renderer's no-final-newline output convention.
  Date/Author: 2026-07-01 / implementing agent.

## Plan of work

### Work item 1: Introduce the exact golden-fixture harness

Read before editing: `AGENTS.md` "Rust Specific Guidance" and "Testing";
`docs/mapsplice-design.md` sections 5, 6, 7, and 8;
`docs/developers-guide.md` sections 2 and 6; `docs/users-guide.md` "Worked
example"; `docs/rust-testing-with-rstest-fixtures.md` sections 1 and 2.

Load skills: `leta`, `rust-router`, `rust-unit-testing`, and
`en-gb-oxendict-style`. Use the `sem` skill for entity-level history if a
fixture exposes a regression that needs historical comparison. Retry Memtrace
and Leta as described in "Research and verified mechanisms".

Add a small fixture harness that reads a named input fixture, writes it to a
temporary workspace target, runs the relevant `mapsplice` operation through
`run_from_args`, and compares `stdout` byte-for-byte to the matching expected
fixture. Prefer a new `tests/roadmap_golden.rs` file if this keeps the fixture
runner cohesive. Use `#[rstest]` fixture injection on each standalone test
function, not a single `#[case]` table, so focused Cargo commands can use
`--exact` safely. The first standalone test function must be named exactly
`section_reference`; it may delegate to a private helper such as
`assert_golden_delete_case`.

Add the first fixture pair:

- `tests/fixtures/reference_rewrite/section_reference.input.md`
- `tests/fixtures/reference_rewrite/section_reference.expected.md`

The input should contain two phases. The second phase's task body should
include a `Requires` clause with both `§2.1` and a valid moved dependency such
as `2.1.1`, for example "Requires §2.1, 2.1.1." Deleting phase 1 must preserve
`§2.1` and rewrite only `2.1.1` to `1.1.1`.

Red step: first run the focused golden test with
`section_reference.expected.md` intentionally containing a wrong expected line,
such as `Requires §2.1, 2.1.1.`. The focused command must fail by showing the
expected/actual diff. Then correct the expected fixture to preserve `§2.1` and
rewrite the dependency to `1.1.1`.

Before running the exact focused command, confirm the standalone test function
exists and is discoverable:

```bash
set -o pipefail
cargo test --test roadmap_golden -- --list \
  | tee /tmp/list-mapsplice-roadmap-1-1-3-work-item-1.out
```

The list output must contain `section_reference: test`. If it does not, fix the
test function name before using `--exact`.

Focused validation:

```bash
set -o pipefail
cargo test --test roadmap_golden -- section_reference --exact \
  | tee /tmp/test-mapsplice-roadmap-1-1-3-work-item-1.out
```

Formatting and gate before commit:

```bash
mdtablefix docs/execplans/roadmap-1-1-3.md \
  tests/fixtures/reference_rewrite/section_reference.input.md \
  tests/fixtures/reference_rewrite/section_reference.expected.md
markdownlint-cli2 --fix docs/execplans/roadmap-1-1-3.md \
  tests/fixtures/reference_rewrite/section_reference.input.md \
  tests/fixtures/reference_rewrite/section_reference.expected.md
set -o pipefail
make all | tee /tmp/make-all-mapsplice-roadmap-1-1-3-work-item-1.out
set -o pipefail
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-1-1-3-work-item-1.out
set -o pipefail
make nixie | tee /tmp/nixie-mapsplice-roadmap-1-1-3-work-item-1.out
```

Commit once the worktree contains only this work item and gates pass.

### Work item 2: Add incidental-number preservation fixtures

Read before editing: `docs/mapsplice-design.md` sections 5, 6, 7, and 8,
especially the fixture-class rows for section-reference and version /
quantity preservation; `docs/developers-guide.md` section 6; and
`AGENTS.md` "Testing".

Load skills: `rust-router`, `rust-unit-testing`, and `rust-verification`.
`proptest` does not need new generated cases unless the named fixtures expose
an input family that is still under-specified.

Add exact fixture pairs for:

- `version_quantity.input.md`
- `version_quantity.expected.md`

The input should include `Released 1.4.0`, a prose quantity such as `Count 27`,
and a mapped `Requires 2.1.1` in the same task text. Deleting phase 1 must
preserve `1.4.0` and `27` while rewriting the dependency to `1.1.1`.
Add this as a standalone `#[rstest]` test function named exactly
`version_quantity`, delegating to the same private helper as work item 1.

If work item 1 did not use `§2.1` outside a `Requires` clause, add a second
small pair:

- `section_reference_outside_requires.input.md`
- `section_reference_outside_requires.expected.md`

That fixture should prove a design-document section reference in ordinary
prose survives when a nearby dependency reference is rewritten.
If this optional pair is added, add it as a standalone `#[rstest]` test
function named exactly `section_reference_outside_requires`.

Red step: add each case with an intentionally wrong expected output first, run
the focused test and confirm the exact diff fails, then correct the expected
file.

Before using `--exact`, run:

```bash
set -o pipefail
cargo test --test roadmap_golden -- --list \
  | tee /tmp/list-mapsplice-roadmap-1-1-3-work-item-2.out
```

The list output must contain `version_quantity: test`; if the optional section
fixture is added, it must also contain
`section_reference_outside_requires: test`.

Focused validation:

```bash
set -o pipefail
cargo test --test roadmap_golden -- version_quantity --exact \
  | tee /tmp/test-mapsplice-roadmap-1-1-3-work-item-2.out
```

If the optional `section_reference_outside_requires` pair is added, run its
focused test too:

```bash
set -o pipefail
cargo test --test roadmap_golden -- section_reference_outside_requires --exact \
  | tee -a /tmp/test-mapsplice-roadmap-1-1-3-work-item-2.out
```

Formatting and gate before commit:

```bash
mdtablefix docs/execplans/roadmap-1-1-3.md \
  tests/fixtures/reference_rewrite/version_quantity.input.md \
  tests/fixtures/reference_rewrite/version_quantity.expected.md
markdownlint-cli2 --fix docs/execplans/roadmap-1-1-3.md \
  tests/fixtures/reference_rewrite/version_quantity.input.md \
  tests/fixtures/reference_rewrite/version_quantity.expected.md
set -o pipefail
make all | tee /tmp/make-all-mapsplice-roadmap-1-1-3-work-item-2.out
set -o pipefail
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-1-1-3-work-item-2.out
set -o pipefail
make nixie | tee /tmp/nixie-mapsplice-roadmap-1-1-3-work-item-2.out
```

If the optional section-reference pair is created, include its two concrete
paths in both direct formatter commands after the files exist.

### Work item 3: Add substring and multi-id `Requires` fixtures

Read before editing: `docs/mapsplice-design.md` section 7's greedy
anchor-token rule and section 8's fixture table; `docs/developers-guide.md`
section 6; `src/roadmap/ops/dependency_text.rs` if production behaviour still
needs verification.

Load skills: `rust-router`, `rust-unit-testing`, and `rust-errors` only if a
fixture exposes an error-boundary bug.

Add exact fixture pairs for:

- `substring_non_match.input.md`
- `substring_non_match.expected.md`
- `multi_id_requires.input.md`
- `multi_id_requires.expected.md`

The substring fixture must include a four-level token such as `1.2.17.1` or
`2.1.1.1` and prove that the renderer never partially rewrites a prefix of the
token. The multi-id fixture must include at least two moved ids in one
`Requires` clause and prove each is rewritten exactly once while punctuation
and surrounding prose are preserved.
Add them as standalone `#[rstest]` test functions named exactly
`substring_non_match` and `multi_id_requires`.

If a fixture fails on current code for a real product reason, inspect
`rewrite_text_value`, `classify_dependency_reference`, and
`rewrite_dependencies` through Memtrace/Leta if available, then make the
smallest production fix in the same work item. If the failure is only an
expected-output mistake, correct the fixture and do not edit production code.

Before using `--exact`, run:

```bash
set -o pipefail
cargo test --test roadmap_golden -- --list \
  | tee /tmp/list-mapsplice-roadmap-1-1-3-work-item-3.out
```

The list output must contain both `substring_non_match: test` and
`multi_id_requires: test`.

Focused validation:

```bash
set -o pipefail
cargo test --test roadmap_golden -- substring_non_match --exact \
  | tee /tmp/test-mapsplice-roadmap-1-1-3-work-item-3.out
set -o pipefail
cargo test --test roadmap_golden -- multi_id_requires --exact \
  | tee -a /tmp/test-mapsplice-roadmap-1-1-3-work-item-3.out
```

Formatting and gate before commit:

```bash
mdtablefix docs/execplans/roadmap-1-1-3.md \
  tests/fixtures/reference_rewrite/substring_non_match.input.md \
  tests/fixtures/reference_rewrite/substring_non_match.expected.md \
  tests/fixtures/reference_rewrite/multi_id_requires.input.md \
  tests/fixtures/reference_rewrite/multi_id_requires.expected.md
markdownlint-cli2 --fix docs/execplans/roadmap-1-1-3.md \
  tests/fixtures/reference_rewrite/substring_non_match.input.md \
  tests/fixtures/reference_rewrite/substring_non_match.expected.md \
  tests/fixtures/reference_rewrite/multi_id_requires.input.md \
  tests/fixtures/reference_rewrite/multi_id_requires.expected.md
set -o pipefail
make all | tee /tmp/make-all-mapsplice-roadmap-1-1-3-work-item-3.out
set -o pipefail
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-1-1-3-work-item-3.out
set -o pipefail
make nixie | tee /tmp/nixie-mapsplice-roadmap-1-1-3-work-item-3.out
```

### Work item 4: Add behavioural and property backstops for the fixture corpus

Read before editing: `docs/rstest-bdd-users-guide.md` "Gherkin feature files"
and "Step definitions"; `docs/developers-guide.md` section 6; `AGENTS.md`
"Testing"; and `docs/mapsplice-design.md` sections 7 and 8.

Load skills: `rust-router`, `rust-unit-testing`, `rust-verification`, and
`proptest`. Keep this work item as a test backstop; do not broaden production
scope.

Add one BDD scenario to `tests/features/mapsplice.feature` that describes the
user-visible fixture result, for example:

```gherkin
Scenario: Delete preserves adversarial reference text while rewriting Requires dependencies
  Given the target roadmap with adversarial reference text
  When I delete phase 1
  Then the command succeeds
  And stdout preserves adversarial references and rewrites Requires dependencies
```

Add matching step definitions in `tests/steps/cli_steps.rs` using the same
small text as the golden fixtures or by reading the fixture input if that keeps
the code clearer without adding brittle path assumptions.
Bind the scenario in `tests/behaviour_cli.rs` with a standalone scenario test
function named `delete_preserves_adversarial_reference_text`, matching the
focused filter below.

If the existing property tests do not already cover all generated invalid and
incidental token families, add one small property in `tests/roadmap_properties.rs`
that uses generated section/version/prose tokens beside a mapped `Requires`
reference. Do not use rejection-heavy strategies; construct only valid token
shapes.

Focused validation:

```bash
set -o pipefail
cargo test --test behaviour_cli -- delete_preserves_adversarial_reference_text \
  | tee /tmp/test-mapsplice-roadmap-1-1-3-work-item-4.out
set -o pipefail
cargo test --test roadmap_properties -- scoped_reference_generated_incidental_tokens_are_preserved \
  | tee -a /tmp/test-mapsplice-roadmap-1-1-3-work-item-4.out
```

If the property keeps its existing name and no new property is needed, run the
existing focused property command above and record that no property-code change
was required.

Formatting and gate before commit:

```bash
mdtablefix docs/execplans/roadmap-1-1-3.md
markdownlint-cli2 --fix docs/execplans/roadmap-1-1-3.md
set -o pipefail
make all | tee /tmp/make-all-mapsplice-roadmap-1-1-3-work-item-4.out
set -o pipefail
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-1-1-3-work-item-4.out
set -o pipefail
make nixie | tee /tmp/nixie-mapsplice-roadmap-1-1-3-work-item-4.out
```

### Work item 5: Mark roadmap task 1.1.3 complete after gates

Read before editing: `docs/roadmap.md` task 1.1.3; `AGENTS.md` "Markdown
Guidance"; `docs/documentation-style-guide.md` "Spelling", "Markdown rules",
and "Formatting"; this ExecPlan's `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective`.

Load skills: `en-gb-oxendict-style` and `execplans`. Update this ExecPlan to
record the implemented fixture paths, commands run, and gate outcomes. Then
mark `docs/roadmap.md` task 1.1.3 as complete only after all fixture tests and
repository gates pass.

Formatting and final validation:

```bash
mdtablefix docs/execplans/roadmap-1-1-3.md docs/roadmap.md
markdownlint-cli2 --fix docs/execplans/roadmap-1-1-3.md docs/roadmap.md
set -o pipefail
make all | tee /tmp/make-all-mapsplice-roadmap-1-1-3-final.out
set -o pipefail
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-1-1-3-final.out
set -o pipefail
make nixie | tee /tmp/nixie-mapsplice-roadmap-1-1-3-final.out
```

Commit the roadmap and ExecPlan update separately from fixture/test commits if
possible.

## Concrete steps

All commands run from
`/home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-3`.

1. Confirm branch and worktree:

   ```bash
   git branch --show-current
   pwd
   git status --short --branch
   ```

   Expected branch is `roadmap-1-1-3`; expected path is the assigned worktree.

2. Retry advisory tools and record failures or results in this plan:

   ```bash
   leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-3
   leta files
   sem log rewrite_text_value --limit 8
   sem log classify_dependency_reference --limit 8
   ```

   Use Memtrace MCP first if it is available in the session. Use Firecrawl MCP
   first for docs.rs research if it is available in the session.

3. Implement work item 1 and commit it after its focused test and gates pass.

4. Implement work item 2 and commit it after its focused test and gates pass.

5. Implement work item 3 and commit it after its focused tests and gates pass.

6. Implement work item 4 and commit it after its focused tests and gates pass.

7. Implement work item 5, run the final gates, and commit the roadmap/ExecPlan
   completion update.

## Validation and acceptance

Acceptance for this roadmap task:

- `tests/fixtures/reference_rewrite/` contains golden input/expected pairs for
  section-reference preservation, version and quantity preservation, substring
  non-match, and multi-id `Requires` lists.
- A Rust test compares each pair exactly after running the documented
  operation.
- The fixture tests fail when an expected file is deliberately wrong and pass
  with the design-correct expected file.
- At least one compiled-binary behavioural scenario covers the adversarial
  user-visible result.
- Existing or new property coverage still checks generated incidental numeric
  text beside mapped `Requires` references.
- `docs/roadmap.md` task 1.1.3 is marked complete only after all final gates
  pass.

Final validation commands:

```bash
set -o pipefail
make all | tee /tmp/make-all-mapsplice-roadmap-1-1-3-final.out
set -o pipefail
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-1-1-3-final.out
set -o pipefail
make nixie | tee /tmp/nixie-mapsplice-roadmap-1-1-3-final.out
```

`make all` includes `check-fmt`, `lint`, `typecheck`, and `test` on current
`origin/main`. `make markdownlint` and `make nixie` are mandatory because this
task edits Markdown fixtures and documentation.

Do not run `mdtablefix` or `markdownlint-cli2 --fix` directly on
`tests/features/mapsplice.feature`; it is Gherkin, and the executable
validation for that file is the BDD test plus the repository gates above.

## Idempotence and recovery

Fixture tests are idempotent: rerunning them rewrites only temporary workspace
files created by test helpers. If a red step uses an intentionally wrong
expected output, restore the expected fixture before any commit. Do not stash
with a bare message. If a stash is unavoidable, use:

```plaintext
df12-stash v1 task=1.1.3 kind=<discard|park|keep> reason="<short>"
```

If a formatter changes unrelated Markdown files, stop and inspect the diff.
Park unrelated formatter churn only with a named `kind=discard` stash, then
continue with direct formatter commands on the files touched by this task.

## Interfaces and dependencies

No new public interface or dependency is planned. The final fixture harness may
introduce private test-only helpers such as:

```rust
struct GoldenCase {
    name: &'static str,
    input: &'static str,
    expected: &'static str,
    command: GoldenCommand,
}
```

Keep such helpers private to the test module unless more than one test module
needs them. If shared support is necessary, place it under `tests/support/` and
keep it feature-scoped to golden fixtures.

## Outcomes & retrospective

Work item 1 added `tests/roadmap_golden.rs` and the
`section_reference.input.md` / `section_reference.expected.md` fixture pair
under `tests/fixtures/reference_rewrite/`. The red check used an intentionally
wrong expected dependency and failed with a stdout diff; the corrected focused
test passed with:

```bash
set -o pipefail
cargo test --test roadmap_golden -- section_reference --exact \
  | tee /tmp/test-mapsplice-roadmap-1-1-3-work-item-1.out
```

The deterministic work-item gates passed after replacing panicking assertions
and `std::fs` fixture reads with fallible result handling and
capability-scoped file access:

```bash
set -o pipefail
make all | tee /tmp/make-all-mapsplice-roadmap-1-1-3-work-item-1.out
set -o pipefail
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-1-1-3-work-item-1.out
set -o pipefail
make nixie | tee /tmp/nixie-mapsplice-roadmap-1-1-3-work-item-1.out
```

`coderabbit review --agent` did not produce file-level findings because the
review process did not progress past service connection and timed out on a
retry. This remains an open review issue for the supervisor rather than a
fixture implementation failure.

Work item 2 added `version_quantity.input.md` /
`version_quantity.expected.md` and the optional
`section_reference_outside_requires.input.md` /
`section_reference_outside_requires.expected.md` fixture pairs. Both red checks
first kept the old `Requires 2.1.1` dependency in the expected file and failed
with a stdout diff; the corrected fixtures passed with:

```bash
set -o pipefail
cargo test --test roadmap_golden -- version_quantity --exact \
  | tee /tmp/test-mapsplice-roadmap-1-1-3-work-item-2.out
set -o pipefail
cargo test --test roadmap_golden -- section_reference_outside_requires --exact \
  | tee -a /tmp/test-mapsplice-roadmap-1-1-3-work-item-2.out
```

The deterministic work-item gates passed:

```bash
set -o pipefail
make all | tee /tmp/make-all-mapsplice-roadmap-1-1-3-work-item-2.out
set -o pipefail
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-1-1-3-work-item-2.out
set -o pipefail
make nixie | tee /tmp/nixie-mapsplice-roadmap-1-1-3-work-item-2.out
```

`coderabbit review --agent` again timed out while connecting to the review
service and produced no file-level findings.

Work item 4 added one compiled-binary BDD scenario,
`delete_preserves_adversarial_reference_text`, plus matching `Given` and
`Then` step definitions. The red check first added only the scenario binding
and feature text; it failed with `Step not found` for
`Given the target roadmap with adversarial reference text`. After adding the
step definitions, the focused BDD scenario and the existing generated
incidental-token property passed:

```bash
set -o pipefail
cargo test --test behaviour_cli -- delete_preserves_adversarial_reference_text \
  | tee /tmp/test-mapsplice-roadmap-1-1-3-work-item-4.out
set -o pipefail
cargo test --test roadmap_properties -- scoped_reference_generated_incidental_tokens_are_preserved \
  | tee -a /tmp/test-mapsplice-roadmap-1-1-3-work-item-4.out
```

No new property was required because
`scoped_reference_generated_incidental_tokens_are_preserved` already generates
section references, versions, prose counts, and a mapped `Requires` reference.
The deterministic work-item gates passed after applying `cargo fmt --all` to
the touched Rust test files:

```bash
set -o pipefail
make all | tee /tmp/make-all-mapsplice-roadmap-1-1-3-work-item-4.out
set -o pipefail
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-1-1-3-work-item-4.out
set -o pipefail
make nixie | tee /tmp/nixie-mapsplice-roadmap-1-1-3-work-item-4.out
```

`coderabbit review --agent` again timed out while connecting to the review
service and produced no file-level findings.

Work item 3 added `substring_non_match.input.md` /
`substring_non_match.expected.md` and `multi_id_requires.input.md` /
`multi_id_requires.expected.md`. The substring red check first rewrote the
four-level unresolved token prefix in the expected file and failed with a
stdout diff; the corrected fixture preserves `2.1.1.1` and rewrites only the
valid `2.1.1` dependency. The multi-id red check first left the second moved
dependency stale and failed with a stdout diff; the corrected fixture rewrites
both `Requires` ids exactly once. The focused tests passed with:

```bash
set -o pipefail
cargo test --test roadmap_golden -- substring_non_match --exact \
  | tee /tmp/test-mapsplice-roadmap-1-1-3-work-item-3.out
set -o pipefail
cargo test --test roadmap_golden -- multi_id_requires --exact \
  | tee -a /tmp/test-mapsplice-roadmap-1-1-3-work-item-3.out
```

The deterministic work-item gates passed:

```bash
set -o pipefail
make all | tee /tmp/make-all-mapsplice-roadmap-1-1-3-work-item-3.out
set -o pipefail
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-1-1-3-work-item-3.out
set -o pipefail
make nixie | tee /tmp/nixie-mapsplice-roadmap-1-1-3-work-item-3.out
```

`coderabbit review --agent` again timed out while connecting to the review
service and produced no file-level findings.

Work item 5 marked `docs/roadmap.md` task 1.1.3 complete after the fixture
corpus, BDD backstop, property backstop, and deterministic code gates were in
place.
Final deterministic validation passed with:

```bash
set -o pipefail
make all | tee /tmp/make-all-mapsplice-roadmap-1-1-3-final.out
set -o pipefail
make markdownlint | tee /tmp/markdownlint-mapsplice-roadmap-1-1-3-final.out
set -o pipefail
make nixie | tee /tmp/nixie-mapsplice-roadmap-1-1-3-final.out
```

The final CodeRabbit review attempt timed out while connecting to the review
service:

```bash
coderabbit review --agent | tee /tmp/coderabbit-mapsplice-roadmap-1-1-3-final.out
```

## Revision notes

- 2026-07-01: Initial draft for roadmap task 1.1.3. The plan decomposes the
  task into fixture-harness, preservation-fixture, substring/multi-id-fixture,
  behavioural/property-backstop, and roadmap-completion work items. It records
  Memtrace, Leta, and Firecrawl planning-session failures as non-blocking
  tooling issues and pins locked library behaviour to local source and docs.rs
  fallback evidence.
- 2026-07-01: Round 2 revision after design review. The golden harness now
  requires exact standalone `#[rstest]` test functions before any `--exact`
  focused commands, with `cargo test --test roadmap_golden -- --list`
  confirmation before each exact filter. Work item 4 no longer runs Markdown
  formatters or Markdown linters on `tests/features/mapsplice.feature`; that
  file is validated through the BDD test path and repository gates.
- 2026-07-01: Work item 1 implementation update. The plan now records the
  section-reference fixture harness, the red/green focused test evidence,
  deterministic gate results, the storage-newline comparison decision, and the
  deferred CodeRabbit review connectivity issue.
- 2026-07-01: Work item 2 implementation update. The plan now records the
  version/quantity fixture, the ordinary-prose section-reference fixture, their
  red/green focused tests, deterministic gate results, and repeated
  CodeRabbit connection timeout.
- 2026-07-01: Work item 3 implementation update. The plan now records the
  substring non-match fixture, multi-id `Requires` fixture, their red/green
  focused tests, deterministic gate results, and repeated CodeRabbit
  connection timeout.
- 2026-07-01: Work item 4 implementation update. The plan now records the BDD
  scenario backstop, matching step definitions, focused BDD/property test
  evidence, deterministic gate results, and repeated CodeRabbit connection
  timeout.
- 2026-07-01: Work item 4 gate follow-up. The plan now records three
  full-repository `make nixie` retries that timed out or failed on untouched
  guide diagrams after the ExecPlan update, plus a scoped successful Nixie
  validation of the touched ExecPlan file.
- 2026-07-01: Work item 5 implementation update. The plan now marks the
  roadmap task complete and records the final roadmap completion action.
- 2026-07-01: Work item 5 validation update. The plan now records green final
  deterministic gates and the deferred final CodeRabbit service-connection
  timeout.
