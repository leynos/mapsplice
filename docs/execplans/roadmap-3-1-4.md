# Pin no-op behaviour for formatter-unstable accepted input

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

Roadmap task 3.1.4 makes the boundary between F1 byte preservation and F4
formatter cleanliness executable. `mapsplice` may accept roadmap-shaped
Markdown that the house formatter later rewrites. After this change, a no-op
`replace` over such input must have explicit, tested behaviour: gate-clean
untouched content stays byte-exact, while the specific formatter-unstable
accepted shapes named here are normalized deterministically.

Success is observable in three ways. First, exact no-op tests prove a
gate-clean loose list and a gate-clean but non-canonical fenced code block are
preserved byte-for-byte. Second, exact no-op tests prove repeated ordered
markers, over-indented nested lists, and tilde or oversized fences are
normalized to gate-clean output. Third, a property test covers both sides of
the boundary so future renderer changes cannot widen normalization merely
because canonical rendering differs from preserved source.

## Constraints

- Work only in `/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-4`.
- Do not begin implementation until this draft is approved by the roadmap
  workflow.
- The integration branch is `main`; treat `origin/main` as canonical.
- Use Memtrace first for canonical main-branch code context when available.
  In this planning session `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`. Record that as advisory-tool failure, not as
  a product blocker.
- Use Leta for branch-local symbol navigation when available. The planning
  command
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-4`
  reported that the workspace was already added, but
  `leta show render_list -n 5` and `leta show render_code_block -n 5` failed
  with `Error: EOF while parsing a value at line 1 column 0`;
  `leta grep "replace|round|formatter|noop|no_op|golden" "tests|src" -k function,method`
  failed with `Error: Connection closed unexpectedly`. Record those failures
  and fall back to bounded file inspection.
- Use `sem` for semantic change inspection before commits. The planning run
  `sem diff --from origin/main --to HEAD` showed this ExecPlan as the only
  branch change relative to `origin/main`.
- Do not normalize untouched Markdown solely because
  `render_block(node, indent) != original`. That predicate is broader than the
  design permits and would rewrite gate-clean content.
- Keep normalization limited to source text that is itself known to be
  formatter-unstable: repeated or non-contiguous ordered list markers,
  over-indented nested list markers, and tilde or oversized fences.
- Do not normalize a gate-clean loose list. `render_list` currently joins list
  items with a single newline and ignores `list.spread`, so raw canonical
  rendering would tighten loose lists and violate F1.
- Do not normalize a gate-clean non-canonical fenced code block, such as a
  one-space-indented triple-backtick block that the house formatter leaves
  unchanged.
- Keep intentionally unstable Markdown snippets in Rust tests, not Markdown
  fixture files. The standing formatter rule requires touched Markdown paths to
  be fixed, which would erase unstable fixture evidence.
- Keep every Rust source file at or below 400 lines. At planning time
  `src/roadmap/render.rs` has 375 lines and `tests/roadmap_golden/contracts.rs`
  has 393 lines, so implementation must use a new renderer helper module and a
  new test module rather than growing those files substantially.
- Do not change public library or CLI signatures.
- Do not add new external dependencies.
- Documentation and comments must use en-GB Oxford spelling and grammar, with
  API names preserved exactly.

## Tolerances (exception triggers)

- If implementation requires changing more than one production renderer module
  plus one new helper module, stop and update this plan before continuing.
- If any public API in `src/lib.rs`, `src/cli.rs`, or `src/roadmap` must
  change, stop and escalate.
- If a proposed detector needs to invoke `mdtablefix` or
  `markdownlint-cli2` at runtime, stop and choose a source-text predicate
  instead.
- If formatter-stability logic would need to normalize tables, stop and split
  that into a later roadmap task. Tables are not part of task 3.1.4.
- If a code file would exceed 400 lines, extract or split before committing.
- If the same changed-code gate fails twice, stop and record the failure in
  `Decision Log`.
- If a direct formatter or linter command would include a path that may not
  exist at that point, omit the direct command and rely on `make all`,
  `make markdownlint`, or `make nixie`.

## Risks

- Risk: normalization broadens and breaks F1 for gate-clean input.
  Severity: high. Likelihood: medium. Mitigation: first commit exact
  preservation tests and a corpus audit before changing renderer behaviour.
- Risk: the existing golden corpus already contains gate-clean-but-
  non-canonical top-level list or code content. Severity: high. Likelihood:
  low. Mitigation: add an audit step that enumerates top-level List and Code
  surfaces in `tests/fixtures/golden` and the 3.1.2 no-op corpus before any
  renderer change.
- Risk: lexical detectors accidentally become a Markdown parser. Severity:
  medium. Likelihood: medium. Mitigation: keep detectors narrow and tied only
  to the three reviewed unstable shapes.
- Risk: official docs.rs retrieval is unavailable through Firecrawl in this
  session. Severity: low. Likelihood: high. Mitigation: pin load-bearing
  library claims to `Cargo.lock` plus locked local crate source, and do not use
  undocumented or newly introduced APIs.

## Progress

- [x] (2026-07-02 22:13+02:00) Confirmed branch `roadmap-3-1-4` and the plan
  path `docs/execplans/roadmap-3-1-4.md`.
- [x] (2026-07-02 22:15+02:00) Loaded `execplans`, `leta`, `sem`,
  `firecrawl-mcp`, `rust-router`, `rust-unit-testing`, `rust-verification`,
  `proptest`, and `en-gb-oxendict-style`.
- [x] (2026-07-02 22:52+02:00) Recorded Memtrace, Leta, and Firecrawl
  failures as advisory-tool failures.
- [x] (2026-07-02 23:00+02:00) Verified the reviewer counterexample:
  `render_list` ignores `list.spread`, and the lint config has no tight-list
  rule.
- [x] (2026-07-02 23:01+02:00) Verified formatter behaviour in `/tmp`:
  `mdtablefix --renumber` changes repeated ordered markers,
  `markdownlint-cli2 --fix` changes four-space nested list indentation, and
  `mdtablefix --fences` changes tilde or oversized fences. A loose list inside
  a headed Markdown document and a one-space-indented triple-backtick fence are
  left unchanged.
- [x] (2026-07-02 23:05+02:00) Revised this ExecPlan for design-review round
  2.
- [x] (2026-07-02 23:31+02:00) Approval received for implementation from
  the roadmap workflow prompt.
- [x] (2026-07-02 23:31+02:00) Work item 1 implemented. Added
  `tests/roadmap_golden/formatter_boundary.rs`, wired it from
  `tests/roadmap_golden.rs`, added the gate-clean property analogue in
  `tests/roadmap_properties.rs`, and introduced
  `src/roadmap/render_preservation.rs` because the initial loose-list test
  exposed an existing preserved-block separator defect.
- [x] (2026-07-02 23:35+02:00) Work item 1 committed after deterministic
  gates passed.
- [x] (2026-07-02 23:39+02:00) Work item 2 implemented. Extended the
  formatter-boundary golden tests with repeated ordered markers, over-indented
  nested lists, tilde fences, and oversized backtick fences; added the
  generated formatter-unstable property; and extended
  `src/roadmap/render_preservation.rs` plus
  `tests/support/formatter_boundary.rs` to canonicalize only those narrow
  formatter-unstable preserved shapes.
- [x] (2026-07-02 23:39+02:00) Work item 2 committed after deterministic gates
  passed.
- [x] (2026-07-02 23:53+02:00) Work item 3 implemented. Marked
  `docs/roadmap.md` task 3.1.4 complete and prepared final documentation gate
  evidence.
- [x] (2026-07-02 23:53+02:00) Work item 3 committed after final gates passed.

## Surprises & discoveries

- Observation: the rejected `rendered != original` mechanism would rewrite
  gate-clean loose lists. Evidence: `src/roadmap/render.rs` renders lists with
  `rendered.join("\n")`, while `.markdownlint-cli2.jsonc` configures MD004,
  MD010, MD013, and MD029 but no tight-list rule. Impact: the implementation
  must inspect formatter-unstable source shapes, not renderer divergence.
- Observation: a one-space-indented triple-backtick fence is gate-clean but not
  canonical according to `render_code_block`. Evidence: a `/tmp` probe running
  `mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place`
  followed by `markdownlint-cli2 --fix` produced no diff. Impact: this exact
  case is an F1 preservation regression test.
- Observation: no tilde fences or four-or-more-backtick fences currently appear
  in `tests/fixtures/golden`. Evidence:
  `grep -RInE '^( {0,3})~~~' tests/fixtures/golden` and
  `grep -RInE '^( {0,3})````+' tests/fixtures/golden` returned no matches.
  Impact: new unstable evidence must be inline Rust test data.
- Observation: the first gate-clean loose-list no-op test failed before
  formatter-boundary normalization work because preserved list source included
  the block separator newline, and `render_roadmap` also joins blocks with a
  blank line. Evidence: `cargo test --test roadmap_golden formatter_boundary`
  wrote an extra blank line before the following step heading. Impact: Work
  item 1 had to introduce the private preservation helper earlier than planned
  so gate-clean preservation could be pinned and kept green.
- Observation: Memtrace remained unavailable during implementation. Evidence:
  `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call` again before Work item 2, while
  `leta refs render_preserved_or_canonical -n 2` returned
  `Error: Failed to start daemon`. Impact: branch-local verification continued
  with bounded file inspection and focused tests, as allowed by the plan.

## Decision log

- Decision: normalize only when the original source block matches a known
  formatter-unstable lexical shape. Rationale: `docs/mapsplice-design.md`
  section 5 allows normalization only for formatter-unstable syntax and
  requires untouched gate-clean content to remain byte-exact. Date/Author:
  2026-07-02, planning agent.
- Decision: add preservation coverage before normalization coverage.
  Rationale: this guards the F1 side of the F1/F4 boundary and catches the
  exact loose-list regression identified in review. Date/Author: 2026-07-02,
  planning agent.
- Decision: implement the policy as a private renderer helper rather than by
  calling external formatter tools. Rationale: rendering must be deterministic,
  self-contained, and free of runtime formatter dependencies. Date/Author:
  2026-07-02, planning agent.
- Decision: keep adversarial Markdown in Rust tests instead of `.md` golden
  fixture files. Rationale: touched Markdown files must be formatted by
  `mdtablefix` and `markdownlint-cli2 --fix`; unstable raw evidence would be
  destroyed. Date/Author: 2026-07-02, planning agent.
- Decision: start `src/roadmap/render_preservation.rs` in Work item 1 rather
  than waiting for Work item 2. Rationale: the preservation tests exposed an
  existing F1 defect in preserved block separator handling, and keeping the
  Work item 1 commit gate-green required the helper to trim renderer-owned
  separator newlines while leaving inner loose-list spacing intact.
  Date/Author: 2026-07-02, implementation agent.
- Decision: defer CodeRabbit review as an open issue for Work item 1.
  Rationale: the sanctioned review agent returned
  `deferred coderabbit review: no default network route visible in this sandbox`,
  so there were no actionable findings to address and the failure is
  environmental rather than a deterministic gate failure. Date/Author:
  2026-07-02, implementation agent.
- Decision: use lexical detectors rather than renderer divergence for Work
  item 2 normalization. Rationale: the detector checks only ordered marker
  continuity, child-list indentation deeper than two spaces, and fence openers
  beginning with tildes or more than three backticks, so loose lists and
  one-space-indented triple-backtick fences remain preservation cases.
  Date/Author: 2026-07-02, implementation agent.
- Decision: defer CodeRabbit review as an open issue for Work item 2.
  Rationale: the sanctioned review agent again returned
  `deferred coderabbit review: no default network route visible in this sandbox`
  with exit 124, so there were no actionable review findings to address.
  Date/Author: 2026-07-02, implementation agent.

## Outcomes & retrospective

Work item 1 added preservation-boundary coverage in
`tests/roadmap_golden/formatter_boundary.rs` and `tests/roadmap_properties.rs`,
wired the golden module from `tests/roadmap_golden.rs`, and added the private
renderer helper `src/roadmap/render_preservation.rs` with a call from
`src/roadmap/render.rs`. Focused tests passed with logs:
`/tmp/cargo-test-roadmap-golden-formatter-boundary-roadmap-3-1-4.out` and
`/tmp/cargo-test-roadmap-properties-gate-clean-roadmap-3-1-4.out`. The green
deterministic gate log is `/tmp/all-mapsplice-roadmap-3-1-4.out`. CodeRabbit
was attempted once for Work item 1 and deferred with log
`/tmp/coderabbit-mapsplice-roadmap-3-1-4.out` because the sandbox has no
default network route. After the ExecPlan update, the commit gates were re-run:
`/tmp/make-all-mapsplice-roadmap-3-1-4.out`,
`/tmp/markdownlint-mapsplice-roadmap-3-1-4.out`, and
`/tmp/nixie-mapsplice-roadmap-3-1-4.out` were all green. No tolerance threshold
was reached.

Work item 2 extended the same test surfaces to pin normalization of repeated
ordered markers, over-indented nested lists, tilde fences, and oversized
backtick fences. It extended `src/roadmap/render_preservation.rs` so preserved
`List` and `Code` nodes canonicalize only when their original source matches
those formatter-unstable lexical shapes, with property support split into
`tests/support/formatter_boundary.rs` to keep source files under the 400-line
limit. Focused logs are
`/tmp/cargo-test-roadmap-golden-formatter-unstable-roadmap-3-1-4.out` and
`/tmp/cargo-test-roadmap-properties-formatter-unstable-roadmap-3-1-4.out`. The
green deterministic gate log is
`/tmp/make-all-mapsplice-roadmap-3-1-4-wi2.out`. After the ExecPlan update,
`/tmp/all-mapsplice-roadmap-3-1-4.out` and
`/tmp/markdownlint-mapsplice-roadmap-3-1-4.out` were green; `make nixie` timed
out once on the unrelated `docs/rstest-bdd-users-guide.md` Mermaid diagram and
then passed on rerun with `/tmp/nixie-mapsplice-roadmap-3-1-4-rerun2.out`.
CodeRabbit was attempted once for Work item 2 and deferred with log
`/tmp/coderabbit-mapsplice-roadmap-3-1-4.out` because the sandbox has no
default network route.

Work item 3 marked `docs/roadmap.md` task 3.1.4 complete and formatted the
touched Markdown paths only: `docs/roadmap.md` and
`docs/execplans/roadmap-3-1-4.md`. `make all` passed with
`/tmp/all-mapsplice--roadmap-3-1-4-.out`, `make markdownlint` passed with
`/tmp/markdownlint-mapsplice-roadmap-3-1-4.out`, and `make nixie` passed on
rerun with `/tmp/nixie-mapsplice-roadmap-3-1-4-wi3-rerun1.out` after the same
unrelated `docs/rstest-bdd-users-guide.md` Mermaid timeout. CodeRabbit was
attempted once for Work item 3 and deferred with log
`/tmp/coderabbit-mapsplice-roadmap-3-1-4.out` because the sandbox has no
default network route.

## Context and orientation

`mapsplice` parses roadmap-shaped Markdown with the locked `markdown` crate,
keeps non-structural Markdown as mdast nodes plus preserved source snippets,
applies one roadmap operation, renumbers structural items, rewrites dependency
references, and renders Markdown to standard output or back to the target file.

The relevant files are:

- `src/roadmap/model.rs`: `MarkdownNodes` stores mdast nodes and
  `original_blocks`. `push_preserved` records an exact source slice for an
  unchanged non-structural node.
- `src/roadmap/render.rs`: `render_markdown_nodes` currently returns the
  preserved original block whenever one exists; otherwise it calls
  `render_block`.
- `src/roadmap/render.rs`: `render_list` canonicalizes list markers and joins
  list items with a single newline.
- `src/roadmap/render_text.rs`: `render_code_block` emits backtick fences using
  `safe_code_fence`, which is canonical but not proof that a different original
  fence is formatter-unstable.
- `tests/golden/format_gate.rs`: rendered golden output is written to a
  temporary file, processed with
  `mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place`, then
  processed with `markdownlint-cli2 --fix`; any formatter diff fails the test.
- `tests/roadmap_properties.rs`: property tests already use
  `proptest::prelude::*` and `run_from_args` against temporary workspaces.
- `.markdownlint-cli2.jsonc`: MD004, MD010, MD013, and MD029 are configured;
  no rule requires loose lists to become tight.

The governing documents are:

- `docs/roadmap.md`, task 3.1.4: add adversarial accepted-input cases that
  record whether formatter-unstable input is preserved or normalized.
- `docs/mapsplice-design.md`, section 5: F1-F5 fidelity guarantees, especially
  the F4-first boundary for formatter-unstable syntax and the F1 requirement
  that untouched gate-clean content remains byte-exact.
- `docs/mapsplice-design.md`, section 8: exact golden fixture, property, and
  regression requirements.
- `docs/developers-guide.md`, sections 2 and 6: architecture boundaries and
  verification layers.
- `docs/users-guide.md`, "The roadmap shape `mapsplice` expects" and "Output
  modes": accepted grammar and observable CLI behaviour.
- `AGENTS.md`: Rust style, testing rules, quality gates, file-size limit,
  commit discipline, and Markdown formatting rules.
- `docs/documentation-style-guide.md`: Markdown and en-GB Oxford prose
  conventions.

## Verified libraries, tools, and APIs

Firecrawl calls to the official docs.rs pages for `markdown`, `rstest`, and
`proptest` all returned `user cancelled MCP tool call`. Do not rely on any
unverified upstream behaviour beyond the locked local source listed here:

- `markdown` is locked at 1.0.0. `Cargo.lock` records `markdown` 1.0.0.
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/lib.rs`
  exports `pub mod mdast` and defines
  `pub fn to_mdast(value: &str, options: &ParseOptions)`. The same crate's
  `configuration.rs` defines `ParseOptions`.
- `rstest` is locked at 0.26.1. The locked local
  `rstest-0.26.1/src/lib.rs` exports `fixture` and documents `#[rstest]`,
  `#[case]`, and named case descriptions.
- `proptest` is locked at 1.11.0. The locked local
  `proptest-1.11.0/src/sugar.rs` defines `proptest!`, `prop_assert!`, and
  `prop_assert_eq!`.
- `mdtablefix --help` verifies the house format arguments used in
  `tests/golden/format_gate.rs`: `--wrap`, `--renumber`, `--breaks`,
  `--ellipsis`, `--fences`, and `--in-place`.
- `markdownlint-cli2 --help` reports version 0.22.1 with markdownlint 0.40.0
  and documents `--fix` as updating files to resolve fixable issues.

## Plan of work

### Work item 1: Audit and pin gate-clean preservation

Read first:

- `docs/roadmap.md`, task 3.1.4.
- `docs/mapsplice-design.md`, sections 5 and 8.
- `docs/developers-guide.md`, sections 2 and 6.
- `docs/users-guide.md`, "The roadmap shape `mapsplice` expects" and "Output
  modes".
- `AGENTS.md`, "Rust Specific Guidance" and "Testing".

Load skills:

- `leta` for branch-local symbol navigation, or record the exact failure and
  use bounded file inspection.
- `rust-router`, routed to `rust-unit-testing`.
- `rust-verification`, routed to `proptest`.
- `en-gb-oxendict-style`.

Implementation:

1. Add a new test module `tests/roadmap_golden/formatter_boundary.rs` and wire
   it from `tests/roadmap_golden.rs`.
2. Add a small, passing corpus audit test that enumerates
   `tests/fixtures/golden/**/*.md` and reports any existing top-level loose
   list, non-canonical code fence, repeated ordered markers, over-indented
   nested list, or tilde/oversized fence. The test must print the offending
   path and line if it fails. This proves the existing corpus is not silently
   relying on gate-clean-but-non-canonical top-level List or Code preservation
   before the renderer changes.
3. Add exact no-op `replace` tests, using inline Rust strings and the existing
   `GoldenWorkspace` style, for these gate-clean inputs:
   - a top-level loose list in an untouched phase or step body, with blank
     lines between items, expected byte-exact after no-op replace;
   - a one-space-indented triple-backtick fenced code block, expected
     byte-exact after no-op replace.
4. Add a property analogue in `tests/roadmap_properties.rs` that generates a
   small gate-clean loose list and a one-space-indented triple-backtick code
   block around a stable no-op replace and asserts the untouched body is
   preserved exactly. Construct valid inputs directly; do not use
   `prop_assume!` for structural filtering.

Red/green/commit:

- These tests should pass before production-code changes. If they fail, fix
  only the test harness or update this plan because the branch already violates
  F1.
- Run:

  ```sh
  cargo test --test roadmap_golden formatter_boundary
  cargo test --test roadmap_properties gate_clean
  make all
  ```

- Commit after the gates pass.

### Work item 2: Normalize only verified formatter-unstable shapes

Read first:

- `docs/mapsplice-design.md`, section 5, especially "normalization must be
  deterministic, limited to formatter-unstable syntax".
- `src/roadmap/render.rs`, `render_markdown_nodes`, `render_block`,
  `render_list`, and `render_code_block` call sites.
- `tests/golden/format_gate.rs`, especially `MDTABLEFIX_ARGS`.
- `.markdownlint-cli2.jsonc`.

Load skills:

- `leta` for `render_markdown_nodes`, `render_block`, and references when
  available.
- `rust-router`, routed to `rust-unit-testing`.
- `rust-verification`, routed to `proptest`.

Implementation:

1. Extend `tests/roadmap_golden/formatter_boundary.rs` with exact red tests for
   no-op `replace` over accepted but formatter-unstable input:
   - repeated ordered markers such as `1. first` followed by `1. second`
     normalize to contiguous numbering;
   - a nested list indented with four spaces beneath a list item normalizes to
     the renderer's gate-clean indentation;
   - a tilde fence and an oversized backtick fence normalize to the renderer's
     backtick fence spelling.
2. Extend `tests/roadmap_properties.rs` with a property analogue over a closed
   enum of the three unstable shape families. The property asserts that no-op
   output is stable under the existing house format gate and matches the
   documented canonical output for the generated shape.
3. Add `src/roadmap/render_preservation.rs` with a private helper used by
   `render_markdown_nodes`, for example:

   ```rust
   fn render_preserved_or_canonical(node: &Node, original: &str, indent: usize) -> Result<String>
   ```

   The helper must call `render_block(node, indent)` only when a source-text
   predicate says the original block is one of the formatter-unstable shapes.
   The predicate must not use `rendered != original` as proof of instability.
4. Keep the detector narrow:
   - for `Node::List`, detect ordered marker sequences at the same indentation
     that are repeated or non-contiguous, and nested list markers indented more
     deeply than the renderer's accepted two-space child indentation;
   - for `Node::Code`, detect opening fences that start with `~` or use more
     than three fence characters;
   - return false for loose-list spacing and one-space-indented
     triple-backtick fences.
5. Add focused unit tests for the helper predicates in
   `src/roadmap/render_preservation.rs` under `#[cfg(test)]`, including the
   loose-list and one-space-indented-fence false cases.

Red/green/commit:

- Run the new exact tests before production edits and confirm they fail for
  the unstable cases while the Work item 1 preservation tests pass.
- Make the minimal renderer change, then run:

  ```sh
  cargo test --test roadmap_golden formatter_boundary
  cargo test --test roadmap_properties formatter
  make all
  ```

- Commit only after all listed commands pass.

### Work item 3: Update roadmap status and run final documentation gates

Read first:

- `docs/roadmap.md`, task 3.1.4.
- `docs/documentation-style-guide.md`.
- `AGENTS.md`, "Markdown Guidance" and "Change Quality & Committing".
- This ExecPlan's `Outcomes & Retrospective`.

Load skills:

- `en-gb-oxendict-style`.
- `commit-message`.
- `sem` for semantic diff inspection before committing.

Implementation:

1. Update `docs/roadmap.md` to mark task 3.1.4 complete only after Work item 2
   passes.
2. Update this ExecPlan's `Progress`, `Decision Log`, and
   `Outcomes & Retrospective` with the commits, test names, and gate log paths.
3. Format only touched Markdown paths. At this point the touched Markdown files
   are expected to be `docs/roadmap.md` and `docs/execplans/roadmap-3-1-4.md`,
   both of which exist before the command:

   ```sh
   mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place docs/roadmap.md docs/execplans/roadmap-3-1-4.md
   markdownlint-cli2 --fix docs/roadmap.md docs/execplans/roadmap-3-1-4.md
   ```

4. Run the final path-safe repository gates:

   ```sh
   make all
   make markdownlint
   make nixie
   ```

5. Inspect `sem diff --from origin/main --to HEAD` and commit the documentation
   update after gates pass.

## Concrete steps

All commands run from `/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-4`.

1. Confirm the branch:

   ```sh
   git branch --show-current
   ```

   Expected output:

   ```text
   roadmap-3-1-4
   ```

2. For each work item, write gate output to `/tmp` with a branch-qualified log,
   for example:

   ```sh
   make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-3-1-4.out
   ```

3. Never run formatters, lints, or tests in parallel. The repository uses
   shared build caches and the sanctioned gate order is sequential.

4. Use file-based commit messages through `git commit -F`, following the
   `commit-message` skill. Do not use `git commit -m`.

## Validation and acceptance

Acceptance requires the following observable behaviours:

- A no-op `replace` over a gate-clean loose list preserves the exact blank-line
  spacing between list items.
- A no-op `replace` over a gate-clean one-space-indented triple-backtick fenced
  code block preserves the exact original bytes.
- A no-op `replace` over repeated ordered markers emits contiguous ordered
  numbering and is stable under the house format gate.
- A no-op `replace` over an over-indented nested list emits gate-clean
  indentation and is stable under the house format gate.
- A no-op `replace` over tilde or oversized fences emits the renderer's
  backtick fence spelling and is stable under the house format gate.
- The generated property suite checks both preservation and normalization
  families.
- `make all`, `make markdownlint`, and `make nixie` pass at the end.

Quality method:

- Use exact `rstest` cases for byte comparisons and red/green clarity.
- Use `proptest` only over constructed-valid snippets and a closed enum of
  shape families.
- Keep full repository gates path-safe by using `make all`,
  `make markdownlint`, and `make nixie`.

## Idempotence and recovery

All planned changes are source edits and tests; they are safe to rerun. If a
red test does not fail for the expected unstable case, stop and inspect whether
the current branch already contains an implementation. If formatter commands
touch unrelated Markdown, do not commit that churn; revert only files changed
by this work item and record the incident in `Decision Log`.

Do not use a bare stash. If stashing becomes unavoidable, use a named stash in
this shape:

```text
df12-stash v1 task=3.1.4 kind=<discard|park|keep> reason="<short>"
```

## Artefacts and notes

Important planning evidence:

- `render_list` in `src/roadmap/render.rs` pushes each rendered item and
  returns `rendered.join("\n")`, so canonical rendering tightens loose lists.
- `.markdownlint-cli2.jsonc` configures MD004, MD010, MD013, and MD029 and has
  no tight-list rule.
- `mdtablefix --renumber` changes repeated ordered list markers from `1, 1` to
  `1, 2`.
- `markdownlint-cli2 --fix` changes a four-space nested list marker beneath
  `- parent` to two spaces.
- `mdtablefix --fences` changes tilde and oversized fences to three
  backticks.
- A headed loose list and a one-space-indented triple-backtick fence are left
  unchanged by the house formatter probe.

## Interfaces and dependencies

No public interface changes are planned. The only new production interface
should be private to `src/roadmap/render_preservation.rs` and called from
`src/roadmap/render.rs`.

The final internal shape should be equivalent to:

```rust
fn render_preserved_or_canonical(node: &Node, original: &str, indent: usize) -> Result<String>
```

It returns `original.to_owned()` unless `original` matches one of the verified
formatter-unstable lexical shapes. It must not compare canonical rendering to
the original as the deciding predicate.

No new dependencies are allowed. Use the existing locked `markdown`, `rstest`,
and `proptest` APIs verified above.

## Revision note

Round 2 revision: replace the rejected broad normalization plan with a
source-text detector limited to reviewed formatter-unstable shapes. Add an
initial preservation and corpus-audit work item so a gate-clean loose list and
a gate-clean non-canonical fenced code block are pinned byte-exact before any
renderer policy change. Add property coverage for both sides of the F1/F4
boundary and require final validation with `make all`, `make markdownlint`, and
`make nixie`.
