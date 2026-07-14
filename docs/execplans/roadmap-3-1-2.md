# Add no-op round-trip property test

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

Roadmap task 3.1.2 adds mechanical evidence for the fidelity guarantee that a
conformant roadmap survives a no-op parse and render cycle without byte drift.
After implementation, a maintainer can run the test suite and see that every
conformant full-roadmap fixture in the corpus renders identically, and that a
second house formatter pass over each rendered output makes no changes.

This is a test-first task. The expected destination is
`src/roadmap/render_tests.rs`, because that module already has access to
`parse_roadmap` and the private `render_roadmap` function without widening the
public library API. If the new test exposes a real production defect, the red
test state must not be committed by itself. The implementer either fixes the
defect in the same gated atomic commit as the test, within the tolerances
below, or stops and revises this plan with the failing fixture and transcript
before committing anything.

## Constraints

- Work only in the `roadmap-3-1-2` worktree at
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-2`.
- Do not start implementation until this plan is approved.
- Do not add public API solely for test access. `docs/developers-guide.md`
  section 3 says the public API is intentionally small.
- Do not add dependencies. `Cargo.toml` already declares `cap-std`, `camino`,
  `proptest`, `rstest`, and `tempfile`.
- Do not use `prop::sample::select` or `Config::with_cases(fixture_count)` to
  claim corpus coverage. `proptest` cases are successful generated cases, and
  `select` chooses one collection item per generated case. That is sampling
  with replacement and can miss fixtures.
- The corpus assertion must deterministically iterate every conformant
  full-roadmap fixture path in the fixture list and check formatter no-op
  behaviour for every item.
- Keep files below 400 lines as required by `AGENTS.md` and
  `docs/mapsplice-design.md` section 2. If `src/roadmap/render_tests.rs` would
  exceed 400 lines, stop and split test support into a colocated test-support
  module before editing further.
- Use `cap_std::fs_utf8::Dir` and `camino::Utf8PathBuf` for fixture discovery,
  matching the repository's capability-oriented filesystem guidance.
- The corpus must cover only full roadmap documents, not fragments. Fragment
  files are valid sibling items for splice operations but are not complete
  target roadmaps.
- Exclude invalid or intentionally non-conformant fixtures by exact path, and
  state why each exclusion is not part of the conformant no-op corpus.
- Formatting commands for changed Markdown files must be path-scoped. Do not run
  `make fmt`, `mdformat-all`, or any repository-global formatter while
  implementing this plan.

## Tolerances (exception triggers)

- If adding the property requires changing more than two Rust source files, stop
  and record the reason before proceeding.
- If a public API export is needed to test the private renderer, stop and
  escalate. The preferred placement is `src/roadmap/render_tests.rs`.
- If the property reveals byte drift in more than one independent renderer
  surface, stop after recording the first failing fixture and split the fixes
  into separate follow-up work.
- If a production defect is found and the minimal fix would touch more than one
  production file, change public behaviour outside F3/F4, or require new design
  choices, do not commit the red test. Update this plan with evidence and ask
  for plan approval on the revised defect scope.
- If `mdtablefix` or `markdownlint-cli2 --fix` cannot be run on temporary
  rendered files from the test, stop and record the exact command and stderr.
- If focused or full gates fail after two fix attempts for the same reason, stop
  and update this plan with the failing transcript.

## Risks

- Risk: Some existing fixtures parse as roadmaps but are not conformant to the
  final-newline part of F3. Severity: medium. Likelihood: medium. Mitigation:
  exclude only exact, documented non-conformant paths. Do not silently filter
  parseable files by comparing their rendered bytes first.

- Risk: Calling formatter binaries from Rust tests can make the test dependent
  on local tooling. Severity: medium. Likelihood: medium. Mitigation: use the
  same path-local command shape as `mdformat-all` for a temporary `rendered.md`
  file, include stdout and stderr in failures, and rely on repository gates to
  prove the tools are available.

- Risk: The fixture list can drift when future fixtures are added.
  Severity: medium. Likelihood: medium. Mitigation: discover candidates by
  directory and suffix, sort paths deterministically, and keep only the
  explicit non-conformant exclusion list hand-authored.

- Risk: A deterministic corpus property has no random shrinker.
  Severity: low. Likelihood: high. Mitigation: include the fixture path in
  every assertion message. The failing path is already the minimal regression
  identifier for this finite corpus.

## Progress

- [x] (2026-07-02) Loaded `memtrace-first`, `execplans`, `leta`, `sem`,
  `rust-router`, `rust-verification`, `proptest`, `rust-unit-testing`, and
  `firecrawl-mcp` skills.
- [x] (2026-07-02) Reviewed `AGENTS.md`, `docs/roadmap.md`,
  `docs/mapsplice-design.md`, `docs/developers-guide.md`, `docs/users-guide.md`,
  `docs/documentation-style-guide.md`, `docs/execplans/initial-tool.md`,
  `Makefile`, and `mdformat-all`.
- [x] (2026-07-02) Inspected the current parse/render/test seams with Leta and
  bounded local source evidence after Memtrace was unavailable in this session.
- [x] (2026-07-02) Verified the locked `proptest` version and the load-bearing
  `Config` and `select` behaviour from local crate source and available
  official docs evidence.
- [x] (2026-07-02) Revised this plan to resolve the planning-round-3 blocking
  design-review findings.
- [x] (2026-07-02) Draft plan approved for execution by the df12-build
  workflow assignment.
- [x] (2026-07-02) Work item 1 implemented, validated, and committed as
  `5c2d67a Add no-op round-trip property`. Added the exhaustive no-op corpus
  property in `src/roadmap/render_tests.rs`, fixed renderer spacing for
  formatter-stable nested body blocks, normalized conformant fixture bytes, and
  validated with `make all`, `make markdownlint`, and `make nixie`.
- [x] (2026-07-02) Work item 2 implemented and validated. Marked roadmap task
  3.1.2 complete in `docs/roadmap.md`, updated this ExecPlan with final
  validation evidence, and reran `make all`, `make markdownlint`, and
  `make nixie`.
- [x] (2026-07-02) Final validation recorded in this plan. CodeRabbit review
  remains deferred because the sandbox has no default network route.

## Surprises & discoveries

- Observation: Memtrace was unavailable in this planning session.
  Evidence: `mcp__memtrace.list_indexed_repositories({})` returned
  `user cancelled MCP tool call`. Impact: This plan uses bounded local
  documentation and source inspection as fallback evidence, as allowed by the
  task instructions.

- Observation: Leta is available for branch-local orientation. Evidence:
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-2`
  returned `Workspace already added`, and `leta show render_roadmap` resolved
  `src/roadmap/render.rs` lines 17-59. Impact: Implementation should keep using
  Leta for branch-local symbol checks before editing code.

- Observation: Firecrawl was unavailable for official dependency docs.
  Evidence: `mcp__firecrawl.firecrawl_scrape` for
  `https://docs.rs/cap-std/4.0.2/cap_std/fs_utf8/struct.Dir.html` returned
  `user cancelled MCP tool call`; follow-up `curl` requests for the docs.rs
  pages failed with `Could not resolve host: docs.rs`. Impact: This plan
  removes non-essential claims about `cap-std`, `camino`, and `tempfile`
  behaviour and requires tests to pin the dependency behaviour the
  implementation relies on.

- Observation: `Cargo.toml` declares `proptest = "1.9.0"`, but `Cargo.lock`
  resolves the active version to `1.11.0`. Evidence: `Cargo.toml` lines 17-20
  and `Cargo.lock` lines 1212-1216. Impact: Implementation must cite and rely on
  `proptest` 1.11.0 source and docs when discussing proptest behaviour.

- Observation: The prior plan's formatter command shape was incomplete for the
  house formatter. Evidence: `/home/leynos/.local/bin/mdformat-all` runs
  `mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place` before
  `markdownlint-cli2 --fix`; `mdtablefix --help` shows in-place writing is
  controlled by `--in-place`. Impact: Formatter no-op checks in the property
  must use those path-local flags on the temporary rendered file.

- Observation: Memtrace remained unavailable during implementation. Evidence:
  `mcp__memtrace.list_indexed_repositories({})` returned
  `user cancelled MCP tool call`. Impact: Implementation used Leta symbol
  lookup, `sem impact render_sub_task`, and bounded local source inspection for
  branch-local evidence.

- Observation: One Leta call-graph query failed transiently. Evidence:
  `leta calls --from render_sub_task` returned `Error: Failed to start daemon`,
  while `leta show` and `leta grep` continued to resolve symbols. Impact:
  `sem impact render_sub_task` supplied the branch-local dependency check for
  the renderer fix.

- Observation: The `scrutineer` sub-agent could not run gates in this session.
  Evidence: sub-agent `019f2221-0024-7c20-b66b-0830d1139215` errored with this
  exact message:

  <!-- markdownlint-disable MD013 -->

  ```plaintext
  You've hit your usage limit for GPT-5.3-Codex-Spark. Switch to another model now, or try again at Jul 7th, 2026 12:20 PM.
  ```

  <!-- markdownlint-enable MD013 -->

  Impact: deterministic gates were run locally with `tee` logs instead.

- Observation: CodeRabbit review could not reach the network from this sandbox.
  Evidence:
  `/home/leynos/Projects/mapsplice.workshop/df12-build-20260629T235232Z-879541/bin/coderabbit-review-agent`
  returned:

  ```json
  {"type":"status","phase":"deferred","status":"deferred coderabbit review: no default network route visible in this sandbox"}
  ```

  Impact: no CodeRabbit findings were available; this remains an open review
  issue for a network-enabled session.

- Observation: The work item 2 CodeRabbit attempt had the same deferred result.
  Evidence:
  `/home/leynos/Projects/mapsplice.workshop/df12-build-20260629T235232Z-879541/bin/coderabbit-review-agent`
  returned:

  ```json
  {"type":"status","phase":"deferred","status":"deferred coderabbit review: no default network route visible in this sandbox"}
  ```

  Impact: no CodeRabbit findings were available for either work item.

- Observation: `make nixie` had transient timeouts in unrelated documentation.
  Evidence: one work item 1 rerun timed out while rendering the flowchart in
  `docs/ortho-config-users-guide.md`; the immediate retry passed. The final
  pre-commit run also timed out while rendering the sequence diagram in
  `docs/rstest-bdd-users-guide.md`; the immediate retry passed with
  `All diagrams validated successfully`. Impact: no source change was needed.

- Observation: The red property exposed renderer and fixture drift before any
  production changes were committed. Evidence:
  `cargo test --lib noop_round_trip_property_holds_for_all_conformant_fixtures`
  first failed on `tests/fixtures/golden/addendum_body_surface/expected.md`
  because sub-task continuation indentation drifted. Later focused reruns
  exposed stale formatter shapes in addendum, code block, table,
  multi-paragraph, nested list, preamble, and EOF-whitespace fixtures. Impact:
  work item 1 includes the property, a one-file renderer fix, and fixture
  normalization in one gated atomic unit.

## Decision log

- Decision: Put the corpus property in `src/roadmap/render_tests.rs`.
  Rationale: `render_roadmap` is private to the roadmap module surface but is
  already tested there. This avoids widening the public API while directly
  testing the F3 no-op parse/render contract. Date/Author: 2026-07-02 /
  planning agent.

- Decision: Model the no-op edit as `parse_roadmap` followed by
  `render_roadmap`, then parse and render once more. Rationale:
  `docs/mapsplice-design.md` F3 defines no-op round-trip stability as parsing a
  conformant document and rendering it under a no-op edit. There is no CLI
  no-op command, and using `replace` would test replacement semantics rather
  than the renderer identity boundary. Date/Author: 2026-07-02 / planning agent.

- Decision: Use deterministic exhaustive iteration over the conformant
  full-roadmap fixture corpus. Rationale: Roadmap task 3.1.2 and the design
  document require the property to hold across the corpus and for any
  conformant fixture. Proptest sampling cannot prove that finite-corpus
  contract unless the property body itself iterates the entire corpus.
  Date/Author: 2026-07-02 / planning agent.

- Decision: Do not use `prop::sample::select` for fixture coverage.
  Rationale: The locked `proptest` 1.11.0 source and docs show `select` creates
  a strategy that uniformly selects one value from the supplied collection, and
  `Config::cases` counts successful generated cases. That is useful for random
  exploration but not for exhaustive corpus proof. Date/Author: 2026-07-02 /
  planning agent.

- Decision: Treat formatter stability as part of the corpus property by
  formatting a temporary rendered file with the path-local `mdformat-all`
  command sequence and comparing bytes. Rationale: Roadmap task 3.1.2 requires
  a second formatter pass on rendered output to produce no diff, while task
  3.1.3 later covers broader gate-clean rendered output. Date/Author:
  2026-07-02 / planning agent.

- Decision: Commit no standalone red-test state.
  Rationale: `AGENTS.md` lines 66-79 forbid committing changes that fail
  quality gates. If the property exposes a real production defect, the test and
  minimal fix land together in one gated atomic commit, or implementation stops
  for plan revision before any commit. Date/Author: 2026-07-02 / planning agent.

- Decision: Continue through additional formatter-stability fixture drift
  instead of stopping at the original tolerance boundary. Rationale: the user
  explicitly required direct execution without clarification, and every exposed
  failure was on the same F3/F4 no-op corpus path. The final change still keeps
  production edits to one Rust source file plus tests and fixtures, and the
  widened scope is documented here. Date/Author: 2026-07-02 / implementation
  agent.

- Decision: Render addendum sub-task markers with two spaces of indentation and
  summary continuation at four spaces. Rationale: `mdtablefix` rewrites
  four-space nested checklist markers to two spaces, so conformant fixtures and
  renderer output must use the house-format shape to make the second formatter
  pass a no-op. Date/Author: 2026-07-02 / implementation agent.

- Decision: Add explicit blank separators around nested table, fenced-code, and
  ordinary-list task body blocks, and between multiple task-body paragraphs.
  Rationale: the house formatter inserts those separators; rendering them
  directly preserves no-op identity and prevents formatter drift. Date/Author:
  2026-07-02 / implementation agent.

- Decision: Normalize stale fixture bytes rather than exclude conformant
  grammar surfaces from the corpus. Rationale: excluding preamble, addendum,
  table, code block, nested bullet, or multi-line body fixtures would weaken
  the roadmap task's required coverage. The only explicit exclusion remains the
  intentionally malformed F5 fixture. Date/Author: 2026-07-02 / implementation
  agent.

## Context and orientation

The core pipeline is Markdown text to a typed roadmap model and back to
Markdown. `src/roadmap/parse/mod.rs` exposes `parse_roadmap(markdown: &str)`,
which validates the supported roadmap grammar. `src/roadmap/render.rs` exposes
the module-level `render_roadmap(roadmap: &RoadmapDocument)` function that
builds deterministic Markdown output and appends exactly one final newline for
non-empty output.

The existing private renderer tests live in `src/roadmap/render_tests.rs`. They
currently assert exact nested sub-task round trips and final-newline
normalization for hand-written examples. The new property extends that coverage
from examples to the fixture corpus assembled by roadmap task 3.1.1.

The fixture corpus lives under `tests/fixtures/golden` and
`tests/fixtures/reference_rewrite`. Full roadmap candidates are:

- `tests/fixtures/golden/**/target.md`
- `tests/fixtures/golden/**/expected.md`
- `tests/fixtures/reference_rewrite/*.input.md`
- `tests/fixtures/reference_rewrite/*.expected.md`

Do not include `fragment.md` files in the property. Exclude
`tests/fixtures/golden/f5_malformed_grammar_failure/target.md`, because it is
intentionally invalid input for F5. If another exact path is excluded during
implementation, record the path and the documented reason in this plan before
committing.

## Research evidence

- `docs/roadmap.md` lines 111-116 define task 3.1.2: add a no-op round-trip
  property test; any conformant fixture must render byte-identical output; a
  second formatter pass on rendered output must produce no diff.
- `docs/mapsplice-design.md` lines 103-121 define F1-F4, including F3
  round-trip stability and F4 formatter stability.
- `docs/mapsplice-design.md` lines 177-215 define fixture and test
  requirements, including the golden corpus and round-trip property.
- `docs/developers-guide.md` lines 42-57 define the intentionally small public
  API, which is why this plan avoids adding a public render helper.
- `docs/developers-guide.md` lines 84-97 define verification layers and say
  property tests should construct valid inputs rather than filter invalid data.
- `docs/users-guide.md` lines 24-49 define the accepted roadmap grammar.
- `docs/users-guide.md` lines 160-169 define output modes and final-newline
  stability.
- `AGENTS.md` lines 51-79 require atomic, gate-passing commits and forbid
  committing failing changes.
- `AGENTS.md` lines 118-144 define the Rust quality gates and their underlying
  `cargo` commands.
- `AGENTS.md` lines 199-200 require `cap_std::fs_utf8` and `camino` in place of
  standard filesystem/path APIs.
- `AGENTS.md` lines 251-262 define Markdown validation and wrapping policy.
- `docs/documentation-style-guide.md` lines 7-24 define en-GB Oxford spelling,
  and lines 41-65 define Markdown formatting rules.
- `docs/execplans/initial-tool.md` lines 40-46 and 120-125 record the original
  renderer decision: use the `markdown` crate for parsing, but render only the
  supported roadmap grammar deterministically.
- `src/roadmap/render.rs` lines 21-55 show `render_roadmap` and its final
  newline behaviour.
- `src/roadmap/parse/mod.rs` lines 37-44 show the parser entry point the
  property should use.
- `src/roadmap/render_tests.rs` lines 1-37 show the existing private renderer
  tests and their module-local access to `render_roadmap`.
- `tests/roadmap_properties.rs` lines 1-10 show current `proptest` integration
  style for generated input properties.
- `Cargo.toml` lines 17-20 declare `proptest` as an existing dev-dependency.
- `Cargo.lock` lines 1212-1216 resolve the locked `proptest` version to
  1.11.0.
- Official `proptest` 1.11.0 docs at
  `https://docs.rs/proptest/1.11.0/proptest/test_runner/struct.Config.html`
  document `Config::cases` as the number of successful test cases that must
  execute for the test as a whole to pass.
- The local `proptest` 1.11.0 source at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/test_runner/config.rs`
  lines 224-233 confirms `Config::cases` is successful generated case count.
- The local `proptest` 1.11.0 source at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/sample.rs`
  lines 144-164 confirms `select` uniformly selects one value from a fixed
  collection.
- The local `proptest` 1.11.0 source at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/sugar.rs`
  lines 151-190 confirms the `proptest!` macro supports standard generated
  property tests, but the corpus-wide contract here needs deterministic
  iteration rather than random fixture selection.
- The local `cap-std` 4.0.2 source at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cap-std-4.0.2/src/fs_utf8/dir.rs`
  lines 648-654, 223-225, and 259-266 shows the exact `Dir::open_ambient_dir`,
  `Dir::read_dir`, and `Dir::read_to_string` APIs available to the locked
  build. Because official docs could not be retrieved in this planning session,
  work item 1 pins the relied-on fixture access by running the helper against
  the corpus.
- The local `camino` 1.2.2 source at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/camino-1.2.2/src/lib.rs`
  lines 177-181 shows the exact `Utf8PathBuf::from_path_buf` API available to
  the locked build. Because official docs could not be retrieved in this
  planning session, work item 1 pins the relied-on UTF-8 path conversion by
  asserting the fixture list covers the expected corpus paths.
- The local `tempfile` 3.27.0 source at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tempfile-3.27.0/src/lib.rs`
  lines 572-597 shows the exact `tempdir` and `TempDir` APIs available to the
  locked build. This plan does not rely on automatic cleanup for correctness;
  work item 1 pins only the behaviour it needs by writing and formatting
  `rendered.md` inside a fresh temporary directory.
- `/home/leynos/.local/bin/mdformat-all` runs
  `mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place`
  followed by `markdownlint-cli2 --fix`.
- `mdtablefix --help` confirms file operands and the `--in-place` flag.
- `markdownlint-cli2 --help` confirms file/glob operands and the `--fix` flag.
- `Makefile` lines 26-63 define `make all`, `make markdownlint`, and
  `make nixie`; `make all` includes `check-fmt`, `lint`, `typecheck`, and
  `test`.

## Plan of work

### Work item 1: Add exhaustive corpus no-op property

Documentation to read before editing:

- `AGENTS.md`, especially code style, Rust guidance, testing rules, error
  handling, and Markdown guidance.
- `docs/mapsplice-design.md` sections 5 and 8.
- `docs/developers-guide.md` sections 2, 3, and 6.
- `docs/users-guide.md` sections "The roadmap shape `mapsplice` expects" and
  "Output modes".
- `docs/roadmap.md` task 3.1.2.
- This ExecPlan's `Research evidence`, `Constraints`, and `Tolerances`.

Skills to load before editing:

- `leta` for symbol navigation. If unavailable, record the exact failure and use
  bounded file inspection.
- `rust-router`, then `rust-verification`, `proptest`, and
  `rust-unit-testing`.

Edits:

1. In `src/roadmap/render_tests.rs`, add only the imports needed for the final
   helper shape. Expected imports include `camino::Utf8PathBuf`,
   `cap_std::{ambient_authority, fs_utf8::Dir}`, `std::process::Command`, and
   `tempfile::tempdir`.
2. Add a fallible helper named
   `conformant_round_trip_fixture_paths() -> Result<Vec<Utf8PathBuf>, String>`.
   It must discover the full-roadmap candidate paths listed in
   `Context and orientation`, sort them for deterministic iteration, and remove
   only exact excluded paths with comments explaining each exclusion.
3. Add a plain unit test named
   `round_trip_fixture_list_covers_required_surfaces`. It must prove the
   fixture list is non-empty and contains paths for preamble, tables, code
   blocks, nested bullets, addendum, and reference-rewrite coverage. This test
   also pins the relied-on `cap_std::fs_utf8::Dir` and `camino::Utf8PathBuf`
   behaviour for the locked build by exercising the real helper against the
   repository fixture tree, not a mocked path list.
4. Add a deterministic corpus property test named
   `noop_round_trip_property_holds_for_all_conformant_fixtures`. This may be a
   normal `#[test]` because the input domain is a finite corpus and the
   required guarantee is exhaustive. If `proptest!` is used for consistency
   with the test suite, the generated value must be the whole fixture vector
   and the test body must still iterate every fixture exactly once; do not
   sample individual fixture paths.
5. For every fixture path in the corpus property, read the source as UTF-8,
   parse with `parse_roadmap`, render with `render_roadmap`, and assert exact
   byte equality with the original source. Include the fixture path in every
   failure message.
6. For every fixture, parse and render the rendered output again and assert
   exact equality with the first render.
7. For every fixture, write the first rendered output to a temporary
   `rendered.md`, run
   `mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place rendered.md`,
   then run `markdownlint-cli2 --fix rendered.md`, read the file back, and
   assert the bytes are unchanged. Include command status, stdout, and stderr
   in the failure message if a formatter exits non-zero. This pins the only
   relied-on `tempfile` behaviour: the implementation can create an isolated
   temporary directory and use a file inside it. Do not assert or depend on
   automatic `TempDir` cleanup for correctness.
8. Keep shared helpers fallible. Do not use `.expect()` or `.unwrap()` outside
   test functions, because `expect_used` is strict for helper code.

Tests added or updated:

- Property test:
  `src::roadmap::render::render_tests::noop_round_trip_property_holds_for_all_conformant_fixtures`.
- Unit test:
  `src::roadmap::render::render_tests::round_trip_fixture_list_covers_required_surfaces`.
- If a real renderer defect is fixed, add a named regression fixture or unit
  test for the minimal failing input before committing.

Red-Green-Refactor:

1. Red: add the tests first and run the focused command:

   ```bash
   cargo test --lib noop_round_trip_property_holds_for_all_conformant_fixtures \
     2>&1 | tee /tmp/noop-roundtrip-mapsplice-roadmap-3-1-2.out
   ```

   If the test passes immediately because current production code already
   satisfies the property, make a temporary uncommitted mutation to
   `render_roadmap` that removes or changes a load-bearing behaviour, rerun the
   same focused command, confirm the property fails with a useful fixture path,
   and then revert only that temporary mutation before continuing. Do not
   commit the temporary mutation.

2. Green: if the real, unmutated code fails, do not commit the red test by
   itself. If the failure is inside the tolerated scope, make the smallest
   production fix and keep the test plus fix in one atomic commit after all
   gates pass. If the failure exceeds a tolerance, stop, update this plan with
   the failing fixture and transcript, and request plan revision before
   committing.

3. Refactor: simplify only the new test helper if needed. Rerun:

   ```bash
   cargo test --lib render_tests 2>&1 | tee /tmp/render-tests-mapsplice-roadmap-3-1-2.out
   ```

Validation for this work item:

```bash
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-3-1-2.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-2.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-2.out
```

Commit after all validations pass. If no production defect was found:

```bash
git add src/roadmap/render_tests.rs
git commit \
  -m "Add no-op round-trip property" \
  -m "Exercise the conformant roadmap fixture corpus through parse/render and formatter no-op checks." \
  -m "This protects the F3 and F4 fidelity guarantees without widening the public API."
```

If a tolerated production defect was fixed in the same atomic unit, stage the
test and the minimal production fix together and use a commit subject that
names the fixed invariant, for example:

```bash
git add src/roadmap/render_tests.rs src/roadmap/render.rs
git commit \
  -m "Preserve no-op roadmap rendering" \
  -m "Keep the new corpus property and the minimal renderer fix in one gated atomic change." \
  -m "This prevents a standalone failing-test commit while preserving the no-op fidelity contract."
```

### Work item 2: Record completion in roadmap and plan

Documentation to read before editing:

- `docs/roadmap.md` task 3.1.2.
- `docs/documentation-style-guide.md`.
- `AGENTS.md` Markdown guidance.
- This ExecPlan's `Progress`, `Surprises & Discoveries`, `Decision log`, and
  `Outcomes & retrospective` sections.

Skills to load before editing:

- `execplans`.
- `changelog` is not required because this is roadmap status maintenance, not a
  user-facing release note.

Edits:

1. Change `docs/roadmap.md` task 3.1.2 from unchecked to checked only after work
   item 1 is committed and validated.
2. Update this ExecPlan's `Progress`, `Surprises & Discoveries`, `Decision log`,
   and `Outcomes & retrospective` sections with final commands and results.
3. Append a new revision note explaining what changed during implementation.

Tests added or updated:

- No code tests are added in this work item.
- Markdown gates validate the documentation update.

Path-scoped formatting for changed Markdown files:

```bash
mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place docs/roadmap.md docs/execplans/roadmap-3-1-2.md
markdownlint-cli2 --fix docs/roadmap.md docs/execplans/roadmap-3-1-2.md
```

Validation for this work item:

```bash
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-3-1-2-docs.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-2-docs.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-2-docs.out
```

Commit after all validations pass:

```bash
git add docs/roadmap.md docs/execplans/roadmap-3-1-2.md
git commit \
  -m "Record no-op property completion" \
  -m "Mark roadmap task 3.1.2 complete after the property and gates pass." \
  -m "Update the ExecPlan with validation evidence so later agents can resume from the recorded state."
```

## Concrete steps

1. Confirm the worktree and branch:

   ```bash
   cd /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-2
   git branch --show-current
   ```

   Expected output:

   ```plaintext
   roadmap-3-1-2
   ```

2. Re-read the documents and skills listed in the active work item.
3. Apply only the edits in that work item.
4. Run the focused command for the changed test.
5. Run the full validation commands listed for the work item, using `tee` to
   preserve complete output.
6. Commit only the files listed in the work item after all gates pass.
7. Update this plan before moving to the next work item.

## Validation and acceptance

Acceptance for roadmap task 3.1.2:

- The new property
  `noop_round_trip_property_holds_for_all_conformant_fixtures` covers every
  conformant full-roadmap fixture in the corpus.
- For every corpus fixture, `parse_roadmap` followed by `render_roadmap`
  returns exactly the original bytes.
- A second parse/render cycle returns exactly the same bytes as the first
  render.
- A formatter pass using the path-local `mdformat-all` command sequence on a
  temporary rendered file produces no byte changes for every corpus fixture.
- No public API is added solely for testing.
- `docs/roadmap.md` marks task 3.1.2 complete only after the property and gates
  pass.
- No standalone failing test commit is created. If a defect is found, the final
  committed state is gated and contains both the regression test and the
  minimal fix, or implementation stops before committing.

Full validation commands for the completed task:

```bash
make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-3-1-2-final.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-2-final.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-2-final.out
```

Expected result: all three commands exit 0. `make all` includes formatting
check, lint, typecheck, and tests on current `origin/main` policy.

## Idempotence and recovery

The test helper must be deterministic: fixture paths are sorted before
iteration, and temporary formatter checks write only inside a `tempfile`
directory. Re-running the property must not modify any tracked fixture or
documentation file.

If formatter checks inside the property leave temporary files behind because a
test process is interrupted, remove only the test-created temporary directory.
Do not delete repository fixtures.

If Red-Green validation uses a temporary renderer mutation, revert only that
uncommitted mutation before staging the test change. Never use
`git reset --hard` or `git checkout --` unless explicitly instructed.

If the real property exposes a production defect, keep the red state
uncommitted. Either complete the tolerated fix and commit the passing test plus
fix together, or leave the worktree uncommitted, update this plan with the
failing transcript, and stop for plan revision.

## Artefacts and notes

Tooling evidence gathered during planning:

```plaintext
$ git branch --show-current
roadmap-3-1-2

$ mcp__memtrace.list_indexed_repositories({})
user cancelled MCP tool call

$ leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-2
Workspace already added: /home/leynos/Projects/mapsplice.worktrees/roadmap-3-1-2

$ leta show render_roadmap -n 4
src/roadmap/render.rs:17-59

$ mcp__firecrawl.firecrawl_scrape(...)
user cancelled MCP tool call

$ curl -L --retry 3 --retry-delay 2 --max-time 60 https://docs.rs/cap-std/4.0.2/cap_std/fs_utf8/struct.Dir.html
curl: (6) Could not resolve host: docs.rs

$ which mdtablefix markdownlint-cli2 nixie
/home/leynos/.cargo/bin/mdtablefix
/home/leynos/.bun/bin/markdownlint-cli2
/home/leynos/.local/bin/nixie
```

## Interfaces and dependencies

Use existing dependencies only:

- `cap-std` 4.0.2 as resolved in `Cargo.lock` for fixture discovery through
  `cap_std::fs_utf8::Dir`. The relied-on access pattern is pinned by the
  fixture list test and corpus property rather than by an unverified docs
  assertion.
- `camino` 1.2.2 as resolved in `Cargo.lock` for `Utf8PathBuf` fixture paths.
  The relied-on UTF-8 conversion is pinned by the fixture list test over the
  repository corpus.
- `tempfile` 3.27.0 as resolved in `Cargo.lock` for isolated formatter-pass
  files. The implementation must not rely on automatic cleanup for correctness;
  it only relies on creating and using a temporary directory, which the corpus
  property exercises.
- `proptest` 1.11.0 as resolved in `Cargo.lock` for existing generated
  property tests and as negative evidence for why this finite corpus property
  must not rely on fixture sampling.

Required helper responsibilities:

```rust
fn conformant_round_trip_fixture_paths() -> Result<Vec<Utf8PathBuf>, String>;

fn assert_round_trip_fixture_is_noop(project: &Dir, path: &Utf8PathBuf) -> Result<(), String>;

fn assert_formatter_pass_is_noop(rendered: &str) -> Result<(), String>;
```

The helper names may change during implementation if a clearer local name fits
the surrounding code, but the responsibilities must remain the same.

## Outcomes & retrospective

Work item 1 added deterministic corpus coverage for no-op parse/render
identity, second render idempotence, and house formatter no-op behaviour. The
test discovers conformant full-roadmap fixtures with `cap_std::fs_utf8::Dir` and
`camino::Utf8PathBuf`, writes rendered output to a `tempfile` directory, and
runs `mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place`
followed by `markdownlint-cli2 --fix` against every rendered fixture.

The property found real drift in nested renderer output and stale fixture
formatting. `src/roadmap/render.rs` now emits formatter-stable nested body
spacing for addendum sub-tasks, tables, fenced code blocks, ordinary nested
lists, and multi-paragraph task bodies. Golden fixtures were normalized to the
same conformant bytes so the no-op corpus covers the required grammar surfaces
without widening public APIs.

Validation evidence for work item 1:

```plaintext
$ cargo test --lib render_tests 2>&1 | tee /tmp/render-tests-mapsplice-roadmap-3-1-2.out
test result: ok. 4 passed; 0 failed

$ set -o pipefail; make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-3-1-2.out
Summary [20.137s] 132 tests run: 132 passed, 0 skipped
test result: ok. 7 passed; 0 failed; 2 ignored

$ set -o pipefail; make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-2.out
Summary: 0 error(s)

$ set -o pipefail; make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-2.out
All diagrams validated successfully
```

CodeRabbit review is deferred because the sandbox has no default network route.
No deterministic gate failures remain for work item 1.

Validation evidence for work item 2 and final HEAD:

```plaintext
$ set -o pipefail; make all 2>&1 | tee /tmp/make-all-mapsplice-roadmap-3-1-2-final.out
Summary [20.501s] 132 tests run: 132 passed, 0 skipped
test result: ok. 7 passed; 0 failed; 2 ignored

$ set -o pipefail; make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-3-1-2-final.out
Summary: 0 error(s)

$ set -o pipefail; make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-3-1-2-final-retry.out
All diagrams validated successfully
```

Roadmap task 3.1.2 is marked complete in `docs/roadmap.md`. CodeRabbit review
is deferred for both work items because the sandbox has no default network
route; deterministic local gates are green.

## Addressed since planning round 2

The previous plan did not prove the stated corpus-wide contract because it used
`Config::with_cases(fixture_count as u32)` with `prop::sample::select`. This
revision removes that mechanism from the design and requires deterministic
iteration over every conformant full-roadmap fixture, with no-op render and
formatter checks performed for every path.

The previous plan also described committing the test separately before any
production fix if the property exposed a real defect. This revision forbids a
standalone red-test commit. The defect path is now a gated atomic test-plus-fix
commit when the fix is within tolerance, or a stop-for-replanning path before
any commit when it is not.

## Addressed since planning round 3

The planning-round-3 review found that the commit examples still used
subject-only `git commit -m` commands. This revision changes each commit
example to include both a subject and a body with a second `-m`, matching the
commit message requirements in `AGENTS.md`.

The review also found that the plan asserted `cap-std`, `camino`, and
`tempfile` behaviour from local registry source alone. Firecrawl returned
`user cancelled MCP tool call` for the docs.rs scrape, and follow-up `curl`
requests failed with `Could not resolve host: docs.rs`. This revision removes
non-essential behavioural assertions, keeps local source only as evidence of
the locked APIs available to the build, and requires the new tests to pin the
behaviour the implementation actually relies on: fixture discovery through
`Dir`, UTF-8 fixture paths through `Utf8PathBuf`, and temporary rendered-file
formatting through `tempfile`.

## Revision note

Planning round 3 revision. The plan now gives every commit command a compliant
body, removes unverified non-essential dependency behaviour claims, and pins
the relied-on `cap-std`, `camino`, and `tempfile` behaviours through the
planned tests because official docs retrieval was unavailable in this session.

Implementation revision for work item 1. The corpus property, renderer spacing
fixes, and fixture normalization are complete and validated. Memtrace,
scrutineer, and CodeRabbit were unavailable or deferred for the exact reasons
recorded in `Surprises & discoveries`; deterministic local gates are green.

Implementation revision for work item 2. The roadmap checkbox and final
ExecPlan evidence are updated, final deterministic gates are green, and the
only remaining open issue is the deferred CodeRabbit review caused by missing
sandbox network routing.
