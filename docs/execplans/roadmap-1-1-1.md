# Specify the dependency-reference predicate in code

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

## Purpose / big picture

Roadmap task 1.1.1 is complete when `mapsplice` has one documented code path
that classifies a dotted numeric token as one of three things: a valid
dependency reference, an invalid dependency-context token, or incidental prose.
This matters because `docs/mapsplice-design.md` says dependency references in
`Requires` clauses may be rewritten, while section references, semantic
versions, quantities, and other incidental numbers must be preserved exactly.

The observable outcome is a small internal classifier in
`src/roadmap/ops/dependency_text.rs`. Unit tests prove that `Requires 99.1.1`
is a valid dependency-reference candidate even when no renumber-plan mapping
exists, while `Requires 1.4.0` is an invalid version-like token because anchor
components must be canonical positive integers. Behavioural and property tests
then prove the rewrite path preserves both unresolved valid references and
invalid or incidental numeric text.

This is planning round 2. Do not begin implementation until this plan is
approved by the controlling workflow.

## Constraints

- Work only in
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-1`.
- Do not edit the root/control worktree.
- Use absolute paths for every `apply_patch` hunk if this plan is implemented
  by an agent using that tool.
- Treat `origin/main` as the integration branch and canonical product
  baseline.
- Use Memtrace as the primary canonical-main search tool. First call
  `list_indexed_repositories`; proceed with repo id `mapsplice` only if that
  call confirms it. If the MCP host cancels or fails, record the exact failure
  and verify branch-local facts with `leta`, `sem`, exact text search, and
  bounded file inspection.
- Use `leta` for branch-local symbol navigation and references when the daemon
  starts. If `leta` cannot start, record the exact failure and use bounded
  local inspection for the current task.
- Use `sem` for semantic history, entity, and impact checks instead of raw
  git-blame style inspection.
- Treat `docs/mapsplice-design.md`, `docs/developers-guide.md`,
  `docs/users-guide.md`, `docs/contributing.md`,
  `docs/documentation-style-guide.md`, `docs/scripting-standards.md`,
  `AGENTS.md`, and `docs/roadmap.md` as source-of-truth documents.
- Follow en-GB Oxford spelling in prose, comments, and commit messages.
- Do not add a new external dependency for task 1.1.1.
- Do not change the accepted roadmap grammar. This task specifies the
  predicate for the existing `Requires` dependency context only.
- Do not make a standalone committed red-test work item. For each code work
  item, add the red test first, run and record the expected failing command in
  this plan, then implement the green code and commit the test, code, and
  updated plan together after all gates pass.
- Every work item that records living ExecPlan progress changes
  `docs/execplans/roadmap-1-1-1.md`; therefore that work item must run
  path-safe Markdown formatting for this file and then `make markdownlint` and
  `make nixie` before committing.
- Commit after each logical change and gate each commit.
- Format only changed Markdown files with path-safe commands. Do not run
  repository-global Markdown formatting such as `make fmt` or `mdformat-all`.
- Every test, lint, format check, and gate command must be logged with `tee` to
  a branch-specific `/tmp` file.

## Tolerances (exception triggers)

- If Memtrace or `leta` are unavailable, record the failure and continue with
  bounded local evidence. Do not mark the plan blocked for advisory-tool
  failure alone.
- If implementing the predicate requires changes outside
  `src/roadmap/ops/dependency_text.rs`, `tests/roadmap_ops.rs`,
  `tests/roadmap_properties.rs`, and documentation, stop and update this plan
  before editing more files.
- If a public API signature in `src/lib.rs`, `src/roadmap/mod.rs`, or
  `src/roadmap/anchor.rs` must change, stop for review. The predicate should
  stay inside the roadmap rewrite module unless a later approved task widens
  it.
- If preserving unresolved valid dependency references conflicts with a
  still-valid product requirement, document the conflict in the Decision Log
  and stop. The current source-of-truth roadmap and design both say unresolved
  references are left unchanged for this task.
- If a work item touches Rust code, load `rust-router` first, then the
  smallest follow-on skill named in that work item.
- If a work item touches more than four production functions or more than six
  files, split the work item before implementation.
- If `make all` fails after two focused fix attempts, record the failing
  command and log path in the Decision Log and stop for review.
- If formatter churn touches files outside the work item, park or discard it
  with a named stash following the df12 stash format before proceeding.

## Risks

- Risk: The current implementation reports `DanglingDependency` for a valid
  unresolved `Requires 99.1.1`, while roadmap task 1.1.1 says that value should
  remain unchanged when no renumber-plan mapping exists.
  Severity: high.
  Likelihood: high.
  Mitigation: work item 2 changes only the resolution fallback after work item
  1 has pinned valid, invalid, and incidental classification paths.

- Risk: The current private helper name `is_dependency_anchor` hides that it
  checks only context, not anchor validity.
  Severity: medium.
  Likelihood: high.
  Mitigation: work item 1 introduces one classification function with an
  explicit result type and tests each branch directly.

- Risk: A scanner change could accidentally rewrite section references or
  semantic versions in later roadmap tasks.
  Severity: high.
  Likelihood: medium.
  Mitigation: work items 1 and 3 include adversarial unit and property tests
  for section sigils, version-like zero components, prose numbers,
  punctuation, and greedy token consumption.

- Risk: Official external documentation tools can be unavailable in the
  sandbox.
  Severity: medium.
  Likelihood: observed.
  Mitigation: this planning pass records Firecrawl cancellation and `curl`
  DNS failure, verifies locked crate behaviour from local registry source, and
  requires compile, unit, and property tests for every relied-on behaviour.

## Progress

- [x] (2026-07-01T11:00Z) Confirmed the assigned worktree is
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-1` and the current
  branch is `roadmap-1-1-1`.
- [x] (2026-07-01T11:00Z) Loaded `execplans`, `leta`,
  `en-gb-oxendict-style`, `memtrace-first`, `rust-router`,
  `rust-types-and-apis`, `rust-unit-testing`, `rust-verification`, `proptest`,
  `rust-errors`, `firecrawl-mcp`, and `sem` for this planning pass.
- [x] (2026-07-01T11:00Z) Read the local source-of-truth documents and the
  existing dependency rewrite code.
- [x] (2026-07-01T11:00Z) Verified with `leta grep`, `leta show`, and bounded
  file inspection that `rewrite_text_value` depends on
  `next_anchor_candidate`, `is_dependency_anchor`, `parse_anchor`,
  `RenumberPlan::resolve`, and `RenumberPlan::resolve_unique`.
- [x] (2026-07-01T11:00Z) Verified locked crate API details from local Cargo
  registry source. Official docs content could not be scraped because
  Firecrawl returned `user cancelled MCP tool call` and `curl` returned
  `Could not resolve host: docs.rs`.
- [x] (2026-07-01T11:00Z) Revised this DRAFT ExecPlan for round 2 design
  review.
- [x] (2026-07-01T11:24Z) Work item 1: Added red classifier tests,
  introduced `DependencyReferenceClassification`, and routed
  `rewrite_text_value` through the classifier while preserving the existing
  unresolved-reference error path for work item 2.
- [x] (2026-07-01T11:50Z) Work item 1 gates: `make all`,
  `make markdownlint`, and `make nixie` passed after focused lint fixes.
  CodeRabbit was attempted once and deferred because the CLI stayed at
  `connecting_to_review_service` until interrupted after an extended wait.
- [x] (2026-07-01T11:52Z) Work item 2: Updated behavioural coverage so
  `Requires 99.1.1` must be preserved, then changed valid unmapped references
  to copy their original candidate text without incrementing the rewrite count.
- [x] (2026-07-01T12:00Z) Work item 2 gates: `make markdownlint`,
  `make nixie`, and `make all` passed. CodeRabbit was attempted once with a
  bounded timeout and again produced only `connecting_to_review_service`.
- [x] (2026-07-01T12:08Z) Work item 3: Added generated invalid-token and
  incidental-number preservation properties. The properties passed on the
  already-fixed implementation, then a temporary scanner mutation failed
  `generated_invalid_dependency_tokens_are_preserved` with a shrunk `2.1.0`
  case before the mutation was reverted.
- [x] (2026-07-01T12:14Z) Work item 3 gates: `make markdownlint`,
  `make nixie`, and `make all` passed. CodeRabbit was attempted once with a
  bounded timeout and again produced only `connecting_to_review_service`.
- [x] (2026-07-01T12:23Z) Work item 4: Marked roadmap task 1.1.1 complete,
  added the developers' guide verification-layer note for
  `classify_dependency_reference`, and updated this plan's outcomes.
- [x] (2026-07-01T12:23Z) Work item 4 gates: `make markdownlint`,
  `make nixie`, and `make all` passed. CodeRabbit was attempted once with a
  bounded timeout and again produced only `connecting_to_review_service`.
- [x] (2026-07-01T12:23Z) Final validation and return prepared after all work
  items were implemented and deterministic gates were green for the final tree.

## Surprises & discoveries

- Memtrace `list_indexed_repositories` was attempted first, but the MCP host
  returned `user cancelled MCP tool call`. Memtrace was therefore unavailable
  in this planning session. This is advisory-tool failure, not a product
  blocker.
- Memtrace was re-attempted during implementation with
  `list_indexed_repositories`, `list_communities`, and scoped `find_code`.
  Each call returned `user cancelled MCP tool call`, so implementation used
  bounded branch-local inspection.
- `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-1`
  first succeeded, but later `leta files src` and `leta refs rewrite_text_value`
  returned `Error: Failed to start daemon`. `leta grep` and `leta show` still
  produced bounded symbol results, so branch-local verification used those
  commands plus exact file inspection.
- `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-1`
  returned `Error: IO error: Read-only file system (os error 30)` in this
  implementation session, and subsequent `leta grep` and `leta show` returned
  `Error: Failed to start daemon`.
- `goose run --recipe /home/leynos/.config/goose/recipes/scrutineer.yaml`
  could not be combined with either `--text` or `--instructions` in this Goose
  CLI, so the requested `scrutineer` delegation could not be parameterized.
  The implementation session ran the same sequential gates directly and
  captured tee logs under `/tmp`.
- `coderabbit review --agent` was attempted for work item 1 after
  deterministic gates passed. It printed only review context and
  `connecting_to_review_service`, then remained silent until interrupted.
- `make nixie` timed out twice on the pre-existing
  `docs/rstest-bdd-users-guide.md` sequence diagram during work item 2, then
  passed on the next retry without content changes.
- `coderabbit review --agent` was attempted for work item 2 after
  deterministic gates passed. It printed only review context and
  `connecting_to_review_service`, then exited with status 124 under a
  five-minute timeout.
- Work item 3's initial property run was non-red because work item 2 had
  already fixed the invalid-token and incidental-token preservation behaviour.
  A deliberate local mutation to make `consume_anchor` consume only one dotted
  segment failed `generated_invalid_dependency_tokens_are_preserved` with
  minimal input `phase = 2, step = 1, task = 1, sub_task = 1, extra = 1`.
- `coderabbit review --agent` was attempted for work item 3 after
  deterministic gates passed. It printed only review context and
  `connecting_to_review_service`, then exited with status 124 under a
  five-minute timeout.
- `coderabbit review --agent` was attempted for work item 4 after
  deterministic gates passed. It printed only review context and
  `connecting_to_review_service`, then exited with status 124 under a
  five-minute timeout.
- Firecrawl `firecrawl_scrape` against the docs.rs pages for `markdown`,
  `rstest`, and `proptest` returned `user cancelled MCP tool call`.
- Direct `curl -L --max-time 30` to those docs.rs pages returned
  `curl: (6) Could not resolve host: docs.rs`, so this plan does not rely on
  official-doc content. It pins behaviour to local registry source and tests.
- `cargo tree -i proptest` reports the locked resolved crate as
  `proptest v1.11.0`, even though `Cargo.toml` declares the caret requirement
  `proptest = "1.9.0"`.

## Decision log

- Decision: implement a classification result, not a bare Boolean predicate.
  Rationale: roadmap task 1.1.1 requires invalid/version-like text and valid
  unresolved dependency references to remain separate classification paths. A
  result enum such as `DependencyReferenceClassification` lets tests assert
  `Reference(anchor)`, `InvalidDependencyToken`, and `NotDependencyReference`
  separately while still providing a single predicate entry point.
  Date/Author: 2026-07-01, planning agent.

- Decision: keep the predicate internal to
  `src/roadmap/ops/dependency_text.rs` for this task.
  Rationale: the developers' guide keeps public APIs small, and the current
  affected surface is the internal rewrite scanner. Later tasks can promote
  the classifier if another module needs it.
  Date/Author: 2026-07-01, planning agent.

- Decision: do not add a regex or parser dependency.
  Rationale: the existing scanner already consumes ASCII digit and dot tokens
  with byte-boundary helpers, `parse_anchor` already validates canonical
  positive anchor components, and AGENTS.md requires dependency minimality and
  caret hygiene.
  Date/Author: 2026-07-01, planning agent.

- Decision: unresolved valid dependency references remain unchanged in task
  1.1.1.
  Rationale: `docs/mapsplice-design.md` section 7 says an unresolved
  dependency reference is left unchanged, and `docs/roadmap.md` task 1.1.1
  explicitly uses `Requires 99.1.1` as the valid unresolved example.
  Date/Author: 2026-07-01, planning agent.

- Decision: use the new classifier from `rewrite_text_value` in work item 1
  while keeping unresolved valid references as `DanglingDependency` until work
  item 2.
  Rationale: the focused tests passed with the classifier unused, but Clippy
  would reject the resulting dead code. Routing through the classifier removes
  dead-code warnings without changing the unresolved-reference behaviour that
  work item 2 specifies with a red behavioural test.
  Date/Author: 2026-07-01T11:24Z, implementation agent.

- Decision: preserve valid dependency references when neither source-specific
  nor unique renumber-plan mappings exist.
  Rationale: this matches `docs/mapsplice-design.md` section 7 and
  `docs/roadmap.md` task 1.1.1. The existing `DanglingDependency` type remains
  available for a later task that reintroduces unresolved-reference reporting
  with the documented predicate semantics.
  Date/Author: 2026-07-01T11:52Z, implementation agent.

- Decision: narrow the private `rewrite_text_value` return type and its only
  caller in `src/roadmap/ops/rewrite.rs`.
  Rationale: preserving valid unmapped references removed the last error path
  from `rewrite_text_value`, and `make all` failed Clippy's
  `unnecessary_wraps` lint. This is a private helper signature, not a public
  API change, and it keeps lint policy intact without suppressions.
  Date/Author: 2026-07-01T11:52Z, implementation agent.

- Decision: pin external-library behaviour to local registry source plus tests
  for this plan revision.
  Rationale: Firecrawl was cancelled by the MCP host, and direct `curl` could
  not resolve `docs.rs`. Local source is available for the locked resolved
  crates, and the plan requires compile, unit, behavioural, and property tests
  to prove every relied-on behaviour in the branch.
  Date/Author: 2026-07-01, planning agent.

## Context and orientation

`mapsplice` parses a constrained roadmap-shaped Markdown document, applies one
structural edit, renumbers roadmap items, and rewrites dependency references
that point to moved items. The dependency rewrite pass is implemented in
`src/roadmap/ops/rewrite.rs` and `src/roadmap/ops/dependency_text.rs`.

The current source scan in `src/roadmap/ops/dependency_text.rs` works as
follows:

- `rewrite_text_value` scans text node values for anchor-shaped candidates.
- `next_anchor_candidate` consumes digit and dot runs greedily.
- `is_dependency_anchor` checks whether the candidate appears after `Requires`
  in the same clause.
- `parse_anchor` in `src/roadmap/anchor.rs` accepts one to four canonical
  positive integer components and rejects zero, leading zero, empty, and
  over-long anchors.
- `RenumberPlan::resolve` and `resolve_unique` in `src/roadmap/model.rs` supply
  mapped anchors after a structural edit.

The load-bearing documentation is:

- `docs/mapsplice-design.md` section 6 C3 requires dependency references to be
  rewritten and non-dependency numbers never to be rewritten.
- `docs/mapsplice-design.md` section 7 defines anchor tokens, dependency
  contexts, incidental numbers, section sigils, and unresolved resolution
  behaviour.
- `docs/mapsplice-design.md` section 8 requires `rstest` unit coverage,
  generated dependency-rewrite properties, and regression discipline.
- `docs/mapsplice-design.md` section 9 D1 records the current unscoped
  reference-rewrite hazard.
- `docs/roadmap.md` task 1.1.1 defines the dependency-reference predicate
  deliverable and success criteria.
- `docs/developers-guide.md` section 6 defines the verification layers and
  property-test discipline.
- `docs/users-guide.md`, "The roadmap shape `mapsplice` expects", defines the
  roadmap levels and command anchors.
- `AGENTS.md`, "Rust Specific Guidance", "Testing", "Dependency Management",
  and "Error Handling", defines Rust implementation and testing requirements.
- `AGENTS.md`, "Documentation Maintenance", and
  `docs/documentation-style-guide.md` define Markdown and prose style.
- `docs/contributing.md` and `Makefile` define the local gates.

No standalone architectural decision record is specific to task 1.1.1. The
accepted initial tool plan remains relevant background:
`docs/execplans/initial-tool.md` section 4 established renumber-plan-based
dependency rewrites and token preservation around rewritten prose.

## External library evidence

This task must not lean on unverified library behaviour. Official docs could
not be scraped in this planning session: Firecrawl returned `user cancelled MCP
tool call` for the three docs.rs pages, and direct `curl -L --max-time 30`
returned `curl: (6) Could not resolve host: docs.rs`. Therefore this plan pins
load-bearing library behaviour to local registry source for the locked
resolved crates and requires tests to prove each branch.

- `markdown = "1.0.0"` is locked as `markdown v1.0.0`.
  Local registry source at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/mdast.rs`
  lines 168-221 defines `markdown::mdast::Node` and the `Text(Text)` variant.
  Lines 835-841 define `Text { pub value: String, pub position:
  Option<Position> }`. This supports continuing to rewrite only
  `Node::Text` values in `rewrite_node`; `make all` and the behavioural tests
  pin the crate integration.
- `rstest = "0.26.1"` is locked as `rstest v0.26.1`.
  Local registry source at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rstest-0.26.1/src/lib.rs`
  lines 704-763 documents `#[rstest]` with `#[case]` table tests producing
  independent test cases. Lines 799-818 document named cases such as
  `#[case::zero_base_case(...)]`. Line 1563 re-exports the `rstest` macro.
  Work item 1 pins this with compiling named classifier cases.
- `Cargo.toml` declares `proptest = "1.9.0"`, but the caret requirement
  resolves in `Cargo.lock` and `cargo tree -i proptest` to
  `proptest v1.11.0`. Local registry source at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/sugar.rs`
  lines 12-21 documents `proptest!` functions with inputs generated from
  strategies. Lines 77-94 document `#![proptest_config(...)]`. Lines 748-806
  define `prop_assert!` and `prop_assert_eq!` to return
  `TestCaseError::fail` rather than panic. Work item 3 pins this with property
  tests using generated inputs and `prop_assert!` or `prop_assert_eq!`.

## Plan of work

### Work item 1: Add red classifier tests, then introduce the predicate

Docs to read before editing: `docs/mapsplice-design.md` sections 6, 7, 8, and
9; `docs/roadmap.md` task 1.1.1; `docs/developers-guide.md` section 6;
`docs/users-guide.md`, "The roadmap shape `mapsplice` expects"; `AGENTS.md`
Rust Specific Guidance and Testing; and `docs/execplans/initial-tool.md`
section 4.

Skills to load: `leta`, `memtrace-first`, `sem`, `rust-router`,
`rust-types-and-apis`, `rust-unit-testing`, and `rust-errors`.

Implementation steps:

1. Re-run Memtrace and `leta` availability checks. If either still fails,
   append the exact command and failure to Surprises & Discoveries and continue
   with bounded local inspection.
2. Use `sem diff --from origin/main --to HEAD --format json`,
   `leta grep "rewrite_text_value|next_anchor_candidate|is_dependency_anchor"`,
   and `leta show rewrite_text_value -n 5` to confirm the current symbol
   surface before editing.
3. Red stage: add only the `#[cfg(test)]` classifier tests in
   `src/roadmap/ops/dependency_text.rs`. Use `rstest` named cases and refer to
   the intended private function and enum names. Do not add the classifier,
   enum, or production control-flow change yet.
4. Run the focused command below and record the red failure in this plan before
   any production-code edit. The expected red failure is a compile failure for
   the missing classifier symbol or enum variant, or a test assertion failure
   proving that the classifier branches do not yet exist. A zero-test pass is
   not acceptable.
5. Green stage: add an internal enum in
   `src/roadmap/ops/dependency_text.rs`, for example
   `DependencyReferenceClassification`, with branches for a valid
   `RoadmapAnchor`, an invalid dependency-context token, and a non-dependency
   token.
6. Replace the current context-only helper with a single classification
   function that takes the text value plus the candidate byte span returned by
   `next_anchor_candidate`.
7. The classifier must check dependency context, immediate section sigil before
   the candidate, candidate text extraction, `parse_anchor`, and final
   classification. The exact internal ordering may be adjusted only when all
   listed branches remain independently testable.
8. Keep `next_anchor_candidate` greedy and byte-boundary based. Do not add a
   regex dependency.
9. Update this plan with the red and green evidence before committing this
   work item.

Tests required for this work item:

- Unit tests: cover `Requires 1.2.3`, `Requires: 1.2.3`, comma-separated
  `Requires 1.2.3, 2.3.4`, invalid `Requires 1.4.0`, valid unresolved-shaped
  `Requires 99.1.1`, section sigil `Requires §1.2`, non-context `See 1.2`,
  sentence-terminated `Requires 1.2. Then 2.3`, alphanumeric prefix/suffix
  boundaries, and greedy `1.2.17.1`.
- Behavioural tests: not required in this work item because the rewrite path
  fallback is not yet changed.
- Property tests: not required in this work item; work item 3 widens generated
  input coverage after the production path uses the classifier.
- Snapshot tests: not required because no stable rendered artefact is
  introduced.
- End-to-end tests: not required because this is an internal classifier.

Validation for this work item:

```bash
cargo test --workspace --all-targets --all-features dependency_reference \
  2>&1 | tee /tmp/test-dependency-reference-red-wi1-mapsplice-roadmap-1-1-1.out
cargo test --workspace --all-targets --all-features dependency_reference \
  2>&1 | tee /tmp/test-dependency-reference-green-wi1-mapsplice-roadmap-1-1-1.out
mdtablefix docs/execplans/roadmap-1-1-1.md \
  2>&1 | tee /tmp/mdtablefix-wi1-mapsplice-roadmap-1-1-1.out
markdownlint-cli2 --fix docs/execplans/roadmap-1-1-1.md \
  2>&1 | tee /tmp/markdownlint-fix-wi1-mapsplice-roadmap-1-1-1.out
make markdownlint 2>&1 | tee /tmp/make-markdownlint-wi1-mapsplice-roadmap-1-1-1.out
make nixie 2>&1 | tee /tmp/make-nixie-wi1-mapsplice-roadmap-1-1-1.out
make all 2>&1 | tee /tmp/make-all-wi1-mapsplice-roadmap-1-1-1.out
```

Expected result: the first focused command fails after the tests are added and
before the classifier exists, the second focused command passes after the
classifier is implemented, the Markdown commands touch only this plan, and all
repository gates exit with status 0. Commit this work item only after the
green state and gates.

### Work item 2: Route rewriting through the classifier and preserve valid unresolved references

Docs to read before editing: `docs/mapsplice-design.md` section 7 resolution
rules; `docs/roadmap.md` task 1.1.1; `docs/developers-guide.md` section 6;
`AGENTS.md` Error Handling; and `docs/execplans/initial-tool.md` section 4.

Skills to load: `leta`, `memtrace-first`, `sem`, `rust-router`,
`rust-errors`, `rust-types-and-apis`, and `rust-unit-testing`.

Implementation steps:

1. Red stage: update or add the behavioural tests in `tests/roadmap_ops.rs`
   before changing `rewrite_text_value`. The red test must show that
   `Requires 99.1.1` currently errors or otherwise fails the new expectation
   that unresolved valid dependency references are preserved unchanged.
2. Run the focused red command below and record the expected failure in this
   plan before production-code edits.
3. Change `rewrite_text_value` so it calls the classifier for every candidate
   span returned by `next_anchor_candidate`.
4. For `NotDependencyReference`, copy the candidate unchanged.
5. For `InvalidDependencyToken`, copy the candidate unchanged. This is the
   branch that preserves version-like `Requires 1.4.0`.
6. For `Reference(anchor)`, resolve with `RenumberPlan::resolve(source, anchor)`
   and then `RenumberPlan::resolve_unique(anchor)`.
7. If no mapping exists for a valid reference, copy the candidate unchanged and
   do not increment the rewrite counter. Do not emit `DanglingDependency` in
   task 1.1.1.
8. If a mapping exists, push the mapped anchor string and increment the rewrite
   counter exactly once.
9. Rename or update the existing
   `tests/roadmap_ops.rs::dangling_dependency_is_rejected` test so
   `Requires 99.1.1` remains unchanged and the operation succeeds. Leave
   `MapspliceError::DanglingDependency` in place unless dead-code lints require
   a narrower cleanup; full unresolved-reference reporting belongs to roadmap
   task 4.1.2.
10. Update this plan with red and green evidence before committing this work
    item.

Tests required for this work item:

- Unit tests: keep work item 1 classifier tests passing.
- Behavioural tests: update or add `tests/roadmap_ops.rs` coverage proving
  `Requires 99.1.1` is preserved without error, `Requires 1.4.0` is preserved
  through the invalid-token path, and a mapped `Requires 2.1.1` still rewrites.
- Property tests: not required in this item; work item 3 adds generated
  preservation cases.
- Snapshot tests: not required because no snapshot-managed output changes.
- End-to-end tests: covered through `run_from_args` integration-style tests in
  `tests/roadmap_ops.rs`; no separate binary fixture is needed.

Validation for this work item:

```bash
cargo test --workspace --all-targets --all-features --test roadmap_ops \
  2>&1 | tee /tmp/test-roadmap-ops-red-wi2-mapsplice-roadmap-1-1-1.out
cargo test --workspace --all-targets --all-features --test roadmap_ops \
  2>&1 | tee /tmp/test-roadmap-ops-green-wi2-mapsplice-roadmap-1-1-1.out
cargo test --workspace --all-targets --all-features dependency_reference \
  2>&1 | tee /tmp/test-dependency-reference-wi2-mapsplice-roadmap-1-1-1.out
mdtablefix docs/execplans/roadmap-1-1-1.md \
  2>&1 | tee /tmp/mdtablefix-wi2-mapsplice-roadmap-1-1-1.out
markdownlint-cli2 --fix docs/execplans/roadmap-1-1-1.md \
  2>&1 | tee /tmp/markdownlint-fix-wi2-mapsplice-roadmap-1-1-1.out
make markdownlint 2>&1 | tee /tmp/make-markdownlint-wi2-mapsplice-roadmap-1-1-1.out
make nixie 2>&1 | tee /tmp/make-nixie-wi2-mapsplice-roadmap-1-1-1.out
make all 2>&1 | tee /tmp/make-all-wi2-mapsplice-roadmap-1-1-1.out
```

Expected result: the red `roadmap_ops` command fails before the production
change for the unresolved-reference expectation, the green command passes
after implementation, the classifier-focused command still passes, the
Markdown commands touch only this plan, and all repository gates exit with
status 0.

### Work item 3: Add generated preservation coverage for adversarial tokens

Docs to read before editing: `docs/mapsplice-design.md` sections 5, 7, and 8;
`docs/developers-guide.md` section 6; `docs/roadmap.md` task 1.1.1; and
`AGENTS.md` Testing.

Skills to load: `leta`, `memtrace-first`, `sem`, `rust-router`,
`rust-verification`, `proptest`, and `rust-unit-testing`.

Implementation steps:

1. Red stage: add property coverage in `tests/roadmap_properties.rs` before
   changing any production code in this work item. These properties should fail
   if the classifier partially rewrites invalid or incidental numeric tokens.
   If work item 2 already makes the properties pass, record that the red stage
   is non-red because the preceding committed behaviour already satisfies the
   property; then validate the property by temporarily mutating the local
   production branch, observing failure without committing it, and reverting
   only that deliberate mutation.
2. Generate version-like or invalid dependency-context tokens by construction
   instead of filtering broad strings.
3. Include at least zero-component cases such as `{phase}.0.{task}`,
   four-level values with a zero component, and over-long dotted values that
   must be preserved rather than partially rewritten.
4. Add a generated incidental-number property proving that the same numeric
   token outside a `Requires` clause is preserved while a known mapped
   `Requires` token in the same document still rewrites.
5. Use `prop_assert!` and `prop_assert_eq!`; do not use `assert!` inside the
   property body.
6. Keep property input ranges small enough for the normal `make all` path.
7. Update this plan with red, mutation-check, or green evidence before
   committing this work item.

Tests required for this work item:

- Unit tests: no new unit tests beyond work item 1.
- Behavioural tests: no new behavioural scenarios beyond work item 2.
- Property tests: add generated invalid-token and incidental-token preservation
  properties in `tests/roadmap_properties.rs`.
- Snapshot tests: not required.
- End-to-end tests: not required beyond the `run_from_args` calls already used
  by the property tests.

Validation for this work item:

```bash
cargo test --workspace --all-targets --all-features --test roadmap_properties \
  2>&1 | tee /tmp/test-roadmap-properties-red-wi3-mapsplice-roadmap-1-1-1.out
cargo test --workspace --all-targets --all-features --test roadmap_properties \
  2>&1 | tee /tmp/test-roadmap-properties-green-wi3-mapsplice-roadmap-1-1-1.out
mdtablefix docs/execplans/roadmap-1-1-1.md \
  2>&1 | tee /tmp/mdtablefix-wi3-mapsplice-roadmap-1-1-1.out
markdownlint-cli2 --fix docs/execplans/roadmap-1-1-1.md \
  2>&1 | tee /tmp/markdownlint-fix-wi3-mapsplice-roadmap-1-1-1.out
make markdownlint 2>&1 | tee /tmp/make-markdownlint-wi3-mapsplice-roadmap-1-1-1.out
make nixie 2>&1 | tee /tmp/make-nixie-wi3-mapsplice-roadmap-1-1-1.out
make all 2>&1 | tee /tmp/make-all-wi3-mapsplice-roadmap-1-1-1.out
```

Expected result: the generated preservation properties either fail before the
fix or are validated by an uncommitted deliberate mutation, then pass in the
committed implementation. The Markdown commands touch only this plan, and all
repository gates exit with status 0.

### Work item 4: Reconcile documentation and roadmap status

Docs to read before editing: `docs/roadmap.md` task 1.1.1,
`docs/mapsplice-design.md` section 7, `docs/developers-guide.md` section 6,
`docs/documentation-style-guide.md`, and `AGENTS.md` Documentation
Maintenance.

Skills to load: `en-gb-oxendict-style` and `execplans`. If any Rustdoc example
is added or changed, also load `rust-router` and `rust-unit-testing`.

Implementation steps:

1. Update `docs/roadmap.md` to mark task 1.1.1 complete only after work items
   1 through 3 are green.
2. If implementation introduced a named classifier that maintainers need to
   understand, add a short note to `docs/developers-guide.md` section 6 naming
   the predicate and its test layer. Do not duplicate the normative model from
   `docs/mapsplice-design.md`.
3. Update this ExecPlan with final evidence, surprises, decisions, and the
   Outcomes & Retrospective section.
4. Format only Markdown files changed in this work item. The command below
   always formats the required plan and roadmap paths, both of which exist and
   are edited in this work item. It includes `docs/developers-guide.md` only
   when `git diff --name-only` shows that this work item has edited that
   existing file.

```bash
markdown_files="docs/execplans/roadmap-1-1-1.md docs/roadmap.md"
if git diff --name-only -- docs/developers-guide.md \
  | grep -qx docs/developers-guide.md; then
  markdown_files="$markdown_files docs/developers-guide.md"
fi
mdtablefix $markdown_files \
  2>&1 | tee /tmp/mdtablefix-with-dev-guide-wi4-mapsplice-roadmap-1-1-1.out
markdownlint-cli2 --fix $markdown_files \
  2>&1 | tee /tmp/markdownlint-fix-with-dev-guide-wi4-mapsplice-roadmap-1-1-1.out
```

Tests required for this work item:

- Unit tests: no new unit tests.
- Behavioural tests: no new behavioural tests.
- Property tests: no new property tests.
- Snapshot tests: not required.
- End-to-end tests: not required.
- Documentation gates: `make markdownlint` and `make nixie` are required, and
  `make all` remains required for the complete task.

Validation for this work item:

```bash
make markdownlint 2>&1 | tee /tmp/make-markdownlint-wi4-mapsplice-roadmap-1-1-1.out
make nixie 2>&1 | tee /tmp/make-nixie-wi4-mapsplice-roadmap-1-1-1.out
make all 2>&1 | tee /tmp/make-all-wi4-mapsplice-roadmap-1-1-1.out
```

Expected result: all three commands exit with status 0.

## Concrete steps

Run all commands from
`/home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-1`.

Before any implementation edit:

```bash
git branch --show-current
git status --short
sem diff --from origin/main --to HEAD --format json
leta grep "rewrite_text_value|next_anchor_candidate|is_dependency_anchor" \
  -k function,method,struct,enum --head 200
leta show rewrite_text_value -n 5
```

Expected branch output:

```plaintext
roadmap-1-1-1
```

Expected `sem diff --from origin/main --to HEAD --format json` before
implementation may include only documentation changes from this plan or prior
approved branch work. If semantic source changes appear before the predicate
implementation, inspect them and update this plan before editing.

After each work item, run its focused validation first, then its path-safe
Markdown formatting if this plan was updated, then:

```bash
make markdownlint 2>&1 | tee /tmp/make-markdownlint-mapsplice-roadmap-1-1-1.out
make nixie 2>&1 | tee /tmp/make-nixie-mapsplice-roadmap-1-1-1.out
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-1-1-1.out
```

Before final return, run:

```bash
git status --short
make all 2>&1 | tee /tmp/final-make-all-mapsplice-roadmap-1-1-1.out
make markdownlint 2>&1 | tee /tmp/final-markdownlint-mapsplice-roadmap-1-1-1.out
make nixie 2>&1 | tee /tmp/final-nixie-mapsplice-roadmap-1-1-1.out
```

## Validation and acceptance

The implementation is accepted when all of the following are true:

- A single internal classifier in `src/roadmap/ops/dependency_text.rs`
  determines whether an anchor-shaped candidate is a valid dependency
  reference, an invalid dependency-context token, or incidental text.
- `Requires 1.4.0` is preserved because `0` is not a positive anchor
  component.
- `Requires 99.1.1` is recognized as a valid dependency-reference candidate and
  preserved unchanged when no renumber-plan mapping exists.
- A mapped dependency reference, such as `Requires 2.1.1` after deleting phase
  1, still rewrites to the mapped anchor.
- Section references such as `§3.2`, prose numbers, and non-`Requires` clauses
  are preserved.
- Unit tests cover every classifier branch.
- Behavioural tests cover unresolved valid references, invalid version-like
  tokens, and mapped dependency references.
- Property tests cover generated invalid and incidental token preservation.
- `make all`, `make markdownlint`, and `make nixie` exit with status 0.

Red-Green-Refactor evidence must be recorded in Progress or Artefacts for each
code work item:

- Red: the focused test command fails for the expected missing classification
  or preservation behaviour before production edits. Work item 1 must add the
  classifier tests before adding the classifier so the red command cannot be a
  zero-test pass.
- Green: the focused command passes after the smallest implementation change.
- Refactor: `make all`, `make markdownlint`, and `make nixie` pass after
  cleanup and living-plan updates.

## Idempotence and recovery

All work items are safe to retry from a clean worktree. If a focused test
fails, inspect the corresponding `/tmp` log and fix only the failing work-item
surface. If a formatter changes unrelated Markdown, do not commit the churn.
Revert or park it with a named stash:

```bash
git stash push -m 'df12-stash v1 task=1.1.1 kind=discard reason="formatter churn outside task"'
```

If implementation discovers that a public API must change, stop at the current
green commit, update this plan's Decision Log with the options, and wait for
review.

## Artefacts and notes

Planning evidence from 2026-07-01:

```plaintext
Memtrace: list_indexed_repositories -> user cancelled MCP tool call
Leta: workspace add succeeded; later leta files/refs -> Error: Failed to start daemon
Firecrawl: docs.rs markdown/rstest/proptest scrapes -> user cancelled MCP tool call
curl: docs.rs markdown/rstest/proptest pages -> curl: (6) Could not resolve host: docs.rs
leta grep dependency symbols -> rewrite_text_value, is_dependency_anchor,
  next_anchor_candidate, parse_anchor, roadmap_ops tests, and property tests listed
leta show rewrite_text_value -> current path raises DanglingDependency for
  valid unresolved dependency references
cargo tree -i markdown -> markdown v1.0.0
cargo tree -i rstest -> rstest v0.26.1
cargo tree -i proptest -> proptest v1.11.0
```

Round 2 design-review fixes:

```plaintext
1. Work items 1-3 now explicitly update this living plan and include
   mdtablefix, markdownlint-cli2 --fix, make markdownlint, and make nixie.
2. Work item 1 now adds the classifier tests first, runs and records red
   failure before production code, then adds the classifier and reruns green.
3. External docs are recorded as unavailable through Firecrawl and curl; all
   load-bearing markdown/rstest/proptest behaviour is pinned to local locked
   source plus compile, unit, behavioural, and property tests.
```

Work item 1 evidence from 2026-07-01:

```plaintext
cargo test --workspace --all-targets --all-features dependency_reference
  red log: /tmp/test-dependency-reference-red-wi1-mapsplice-roadmap-1-1-1.out
  result: failed before production edit with unresolved imports for
  DependencyReferenceClassification and classify_dependency_reference.
cargo test --workspace --all-targets --all-features dependency_reference
  green log: /tmp/test-dependency-reference-green-wi1-mapsplice-roadmap-1-1-1.out
  result: passed 11 classifier-focused unit cases and one filtered
  dependency-rewrite property.
make all
  log: /tmp/make-all-wi1-mapsplice-roadmap-1-1-1.out
  result: passed check-fmt, cargo doc, clippy, whitaker, cargo check,
  nextest, and doctests after focused lint fixes.
make markdownlint
  log: /tmp/make-markdownlint-wi1-mapsplice-roadmap-1-1-1.out
  result: passed for 18 Markdown files.
make nixie
  log: /tmp/make-nixie-wi1-mapsplice-roadmap-1-1-1.out
  result: passed Mermaid validation.
coderabbit review --agent
  log: /tmp/coderabbit-wi1-mapsplice-roadmap-1-1-1.out
  result: deferred; output stopped after connecting_to_review_service.
```

Work item 2 evidence from 2026-07-01:

```plaintext
cargo test --workspace --all-targets --all-features --test roadmap_ops
  red log: /tmp/test-roadmap-ops-red-wi2-mapsplice-roadmap-1-1-1.out
  result: failed before production edit because Requires 99.1.1 still raised
  DanglingDependency.
cargo test --workspace --all-targets --all-features --test roadmap_ops
  green log: /tmp/test-roadmap-ops-green-wi2-mapsplice-roadmap-1-1-1.out
  result: passed 12 roadmap operation tests, including
  unresolved_dependency_reference_is_preserved.
cargo test --workspace --all-targets --all-features dependency_reference
  log: /tmp/test-dependency-reference-wi2-mapsplice-roadmap-1-1-1.out
  result: passed the classifier-focused tests and the filtered mapped
  dependency rewrite property.
make markdownlint
  log: /tmp/make-markdownlint-wi2-mapsplice-roadmap-1-1-1.out
  result: passed for 18 Markdown files.
make nixie
  log: /tmp/make-nixie-wi2-mapsplice-roadmap-1-1-1.out
  result: passed after two transient timeouts on an unchanged sequence diagram.
make all
  log: /tmp/make-all-wi2-mapsplice-roadmap-1-1-1.out
  result: passed check-fmt, cargo doc, clippy, whitaker, cargo check,
  nextest, and doctests.
coderabbit review --agent
  log: /tmp/coderabbit-wi2-mapsplice-roadmap-1-1-1.out
  result: deferred; timeout after connecting_to_review_service.
```

Work item 3 evidence from 2026-07-01:

```plaintext
cargo test --workspace --all-targets --all-features --test roadmap_properties
  red log: /tmp/test-roadmap-properties-red-wi3-mapsplice-roadmap-1-1-1.out
  result: non-red; the new generated properties already passed because work
  item 2 had fixed preservation.
cargo test --test roadmap_properties --workspace --all-features
  mutation log: /tmp/test-roadmap-properties-mutation-wi3-mapsplice-roadmap-1-1-1.out
  result: failed generated_invalid_dependency_tokens_are_preserved after a
  temporary one-dot scanner mutation; minimal failing token shape was 2.1.0.
cargo test --test roadmap_properties --workspace --all-features
  green log: /tmp/test-roadmap-properties-green-wi3-mapsplice-roadmap-1-1-1.out
  result: passed 5 property tests after reverting the mutation and deleting the
  generated proptest regression artefact.
make markdownlint
  log: /tmp/make-markdownlint-wi3-retry-mapsplice-roadmap-1-1-1.out
  result: passed for 18 Markdown files.
make nixie
  log: /tmp/make-nixie-wi3-retry-mapsplice-roadmap-1-1-1.out
  result: passed Mermaid validation.
make all
  log: /tmp/make-all-wi3-retry-mapsplice-roadmap-1-1-1.out
  result: passed check-fmt, cargo doc, clippy, whitaker, cargo check,
  nextest, and doctests.
coderabbit review --agent
  log: /tmp/coderabbit-wi3-mapsplice-roadmap-1-1-1.out
  result: deferred; timeout after connecting_to_review_service.
```

Work item 4 evidence from 2026-07-01:

```plaintext
mdtablefix docs/execplans/roadmap-1-1-1.md docs/roadmap.md docs/developers-guide.md
  log: /tmp/mdtablefix-with-dev-guide-wi4-mapsplice-roadmap-1-1-1.out
  result: path-scoped formatting for changed Markdown files.
markdownlint-cli2 --fix docs/execplans/roadmap-1-1-1.md docs/roadmap.md docs/developers-guide.md
  log: /tmp/markdownlint-fix-with-dev-guide-wi4-mapsplice-roadmap-1-1-1.out
  result: passed for 3 changed Markdown files.
make markdownlint
  log: /tmp/make-markdownlint-wi4-mapsplice-roadmap-1-1-1.out
  result: passed for 18 Markdown files.
make nixie
  log: /tmp/make-nixie-wi4-mapsplice-roadmap-1-1-1.out
  result: passed Mermaid validation.
make all
  log: /tmp/make-all-wi4-mapsplice-roadmap-1-1-1.out
  result: passed check-fmt, cargo doc, clippy, whitaker, cargo check,
  nextest, and doctests.
coderabbit review --agent
  log: /tmp/coderabbit-wi4-mapsplice-roadmap-1-1-1.out
  result: deferred; timeout after connecting_to_review_service.
```

## Interfaces and dependencies

The end-state interface should stay internal to
`src/roadmap/ops/dependency_text.rs`. A concrete shape the implementer may use:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DependencyReferenceClassification {
    Reference(RoadmapAnchor),
    InvalidDependencyToken,
    NotDependencyReference,
}
```

The classifier should be a private function in the same module, for example:

```rust
fn classify_dependency_reference(
    value: &str,
    start: usize,
    end: usize,
) -> DependencyReferenceClassification
```

The exact name may vary to match local style, but the single-function contract
must not vary: it owns the dependency context, section-sigil exclusion, token
text extraction, and `parse_anchor` validity check. `rewrite_text_value` then
uses only this result to decide whether to copy, resolve, or rewrite a
candidate.

No new crates are required. Continue to use the locked dependencies already in
`Cargo.toml`: `markdown 1.0.0`, `rstest 0.26.1`, and the resolved
`proptest 1.11.0` from the `proptest = "1.9.0"` caret requirement.

## Outcomes & retrospective

Implemented roadmap task 1.1.1 as an internal dependency-reference
classification path in `src/roadmap/ops/dependency_text.rs`. The classifier
separates valid dependency references, invalid dependency-context tokens, and
incidental prose before the rewrite path resolves mapped references.

The task leaves unresolved valid dependency references unchanged, matching the
design-document model for this milestone and leaving diagnostic reporting for
roadmap task 4.1.2. The private text-rewrite helper no longer returns a
`Result` because preservation removed its error path; its only caller was
updated with no public API change.

Validation is covered by classifier unit tests, behavioural coverage for
unresolved references and mapped rewrites, generated property tests for invalid
and incidental token preservation, and the repository gates. CodeRabbit review
remains an open external-tooling issue because all four attempts reached only
`connecting_to_review_service` and produced no findings.

## Revision note

Round 2 revised this DRAFT plan after design review. The work items now include
living ExecPlan Markdown formatting and Markdown gates wherever progress
evidence changes this plan; work item 1 is explicitly red-green executable by
adding failing classifier tests before production code; and external-library
evidence no longer relies on docs.rs reachability, instead pinning behaviour to
local locked registry source plus planned tests. No implementation has begun.

Work item 1 changed the plan status to IN PROGRESS, recorded current Memtrace
and `leta` implementation-session failures, and captured the classifier
red/green evidence. Remaining work starts from the classifier-backed rewrite
path and changes only the unresolved-reference fallback in work item 2.

Work item 1 gate updates record that deterministic gates passed and that
CodeRabbit review was attempted once but deferred because the CLI stayed in the
connecting phase without returning review findings. The remaining work may
continue with this open review issue because local gates are green and the
failure is external tooling, not product behaviour.

Work item 2 changed the unresolved-reference fallback after the classifier was
already in the production path. Valid references without a renumber-plan
mapping now remain unchanged, while invalid dependency-context tokens and
incidental prose continue to be copied unchanged.

Work item 2 also records a narrow deviation before editing
`src/roadmap/ops/rewrite.rs`: the private text-rewrite helper no longer has an
error path, so its return type and only caller are adjusted to satisfy the
repository's lint policy without suppressing `unnecessary_wraps`.

Work item 2 gate updates record that deterministic gates passed, that Nixie had
transient renderer timeouts on an unchanged document before passing, and that
CodeRabbit review was attempted with a bounded timeout but still did not return
review findings.

Work item 3 added generated preservation coverage. Its red stage is recorded as
non-red because work item 2 had already fixed the behaviour, and the deliberate
scanner mutation proved the new invalid-token property fails when the scanner
partially rewrites a version-like token.

Work item 3 gate updates record that deterministic gates passed and that
CodeRabbit review was attempted with a bounded timeout but still did not return
review findings.

Work item 4 reconciled the roadmap and developers' guide with the implemented
predicate, marked this plan complete, and recorded final deterministic-gate
evidence. CodeRabbit review was attempted with a bounded timeout but still did
not return review findings.
