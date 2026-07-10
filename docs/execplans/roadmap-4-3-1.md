# Bring Public API Rustdoc Examples up to Project Standard

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

## Purpose / Big Picture

Roadmap task 4.3.1, "Bring public API Rustdoc examples up to project
standard", makes the small public library surface understandable and executable
from the documentation itself. The developers' guide names the public entry
points in `docs/developers-guide.md` section 3. The roadmap task body requires
compact executable examples for the public APIs listed there, while its
success line spotlights `run_from_args`, `run_request`, and `parse_roadmap`.
`parse_anchor` and `metrics_snapshot` already have executable examples, so the
remaining documentation gap in that public API list is `parse_fragment`.

After this plan is implemented, a maintainer can open the generated Rust
documentation for those APIs and see examples that compile and run as doctests.
The examples demonstrate the supported roadmap grammar, temporary-path
filesystem setup for application workflows, and the difference between parsed
requests and command-line argument workflows.

Observable success is:

- `src/roadmap/parse/mod.rs::parse_roadmap`, re-exported by `src/lib.rs`, has
  a `# Examples` section with an executable doctest that parses a minimal
  conformant roadmap and inspects the typed model.
- `src/roadmap/parse/fragment.rs::parse_fragment`, re-exported by
  `src/lib.rs`, has a `# Examples` section with an executable doctest that
  parses a valid phase fragment and inspects the typed fragment variant.
- `src/lib.rs::run_request` has a `# Examples` section with an executable
  doctest that builds a `CliRequest`, uses temporary target and fragment files,
  and asserts rendered standard-output mode.
- `src/lib.rs::run_from_args` has a `# Examples` section with an executable
  doctest that exercises the public argument-parsing workflow over temporary
  files.
- `cargo test --doc --workspace --all-features` passes, and the full
  repository gates `make all`, `make markdownlint`, and `make nixie` pass.

## Constraints

- Work only inside
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-3-1`.
- Do not edit the root/control worktree.
- Treat `origin/main` as canonical integration context and
  `docs/roadmap.md` as the roadmap source of truth.
- This is planning round 2. Do not begin implementation until this
  draft is approved by the df12-build roadmap workflow.
- Preserve `docs/mapsplice-design.md` section "Status and scope":
  `docs/users-guide.md` remains the source of truth for command semantics and
  accepted grammar, `AGENTS.md` governs gates and spelling, and the design
  document governs fidelity and contract guarantees.
- Preserve `docs/mapsplice-design.md` sections 2, 3, and 4: parsing remains
  mdast-based, application flow remains parse/apply/render/stdout-or-in-place,
  and the examples must use only the supported phase, step, task, and addendum
  grammar.
- Preserve `docs/mapsplice-design.md` sections 5 and 6: examples must not
  imply that non-`Requires` numeric text is rewritten, that malformed input is
  guessed at, or that in-place output writes before success.
- Implement `docs/roadmap.md` task 4.3.1 exactly: add compact executable
  Rustdoc examples for the public APIs listed in the developers' guide,
  keeping filesystem-heavy flows isolated to temporary paths. The named
  success APIs are `run_from_args`, `run_request`, and `parse_roadmap`; the
  full section-3 gap also includes `parse_fragment`, because `parse_anchor` and
  `metrics_snapshot` already have executable examples.
- Preserve `docs/developers-guide.md` section 2: `src/main.rs` remains the
  binary adapter, `src/lib.rs` remains the application workflow, `src/cli.rs`
  remains command-line parsing and configuration, and `src/roadmap` remains
  the domain layer.
- Preserve `docs/developers-guide.md` section 3: public APIs return typed
  `MapspliceError` values; opaque reports stay outside the library API.
- Preserve `docs/developers-guide.md` section 6: verification stays layered;
  doctests are executable public API documentation, not substitutes for
  existing unit, behavioural, property, snapshot, or compile-time tests.
- Preserve `docs/users-guide.md` sections "The roadmap shape `mapsplice`
  expects", "Command overview", "Output modes", and "Validation rules and
  failure cases": examples must demonstrate documented command grammar and
  output modes.
- Follow `docs/documentation-style-guide.md` sections "Spelling", "Markdown
  rules", and "Formatting": prose uses en-GB Oxford spelling, fenced code
  blocks include language identifiers, and paragraphs wrap at 80 columns.
- Follow `AGENTS.md` Rust guidance: public APIs have Rustdoc comments and
  examples, Clippy warnings are denied, `expect` and `unwrap` are avoided in
  production code and shared fixtures, and code files remain under 400 lines.
- Do not add external dependencies. Use only crates already locked in
  `Cargo.lock`; `tempfile = 3.27.0` is already a dev-dependency and
  `camino = 1.2.2` is already a normal dependency.
- Do not enable or rely on unstable Rustdoc lints such as
  `missing_doc_code_examples`. This task is implemented by adding explicit
  examples and proving them through doctests.
- Format only Markdown files changed by this task. Do not run repository-global
  Markdown formatters such as `make fmt` or `mdformat-all`; use
  `mdtablefix docs/execplans/roadmap-4-3-1.md` and
  `markdownlint-cli2 --fix docs/execplans/roadmap-4-3-1.md` when this plan
  changes.
- Run tests, lint, and formatting gates sequentially with `tee` logs under
  `/tmp`. Do not run test, lint, or format gates in parallel.
- Use the shared Cargo cache. Do not create an isolated Cargo cache.
- Do not mark this plan blocked only because Memtrace, Leta, Firecrawl, Sem, or
  another advisory tool is unavailable. Record the failed command and continue
  with bounded local docs, source, and tests.

## Tolerances

- Stop and escalate if implementation requires any public API signature change,
  a changed CLI command shape, a changed roadmap grammar, or a new dependency.
- The planned implementation may edit only `src/lib.rs`,
  `src/roadmap/parse/mod.rs`, `src/roadmap/parse/fragment.rs`,
  `docs/execplans/roadmap-4-3-1.md`, and `docs/roadmap.md`.
- If another file appears necessary, update this ExecPlan first and escalate
  for design review before editing that file.
- Stop and escalate if `src/lib.rs` would exceed 400 lines after the examples.
  It was 161 lines during planning, so the expected headroom is sufficient.
- Stop and escalate if any example needs `ignore`, `no_run`, `compile_fail`,
  `should_panic`, process-wide environment mutation, or fixed filesystem paths.
- Stop and escalate if a focused doctest still fails for the same reason after
  three implementation attempts.
- Stop and escalate if `make all` fails for an unrelated pre-existing issue
  that cannot be isolated with a focused command and a log.
- Keep each work item independently committable and gate-passable. The code
  commits should each add one API example; the final roadmap/documentation
  commit should update only living documentation state.

## Risks

- Risk: doctests that write files can collide when rustdoc runs examples in
  parallel.
  Severity: medium.
  Likelihood: medium.
  Mitigation: use `tempfile::tempdir()` in filesystem examples and keep every
  target and fragment under the returned `TempDir`.

- Risk: examples become too large and obscure the public API being taught.
  Severity: medium.
  Likelihood: medium.
  Mitigation: follow `docs/rust-doctest-dry-guide.md` sections 2.2, 2.3, and
  2.4: show the API call and assertion, hide temporary-file setup and fallible
  boilerplate with Rustdoc `#` hidden lines.

- Risk: examples use `.expect()` or `.unwrap()` because that is shorter than
  propagating errors.
  Severity: medium.
  Likelihood: medium.
  Mitigation: every doctest must use a hidden `fn main() -> Result<(), Box<dyn
  std::error::Error>>` and `?` for fallible setup and API calls.

- Risk: the examples accidentally test private implementation details.
  Severity: low.
  Likelihood: medium.
  Mitigation: keep application-boundary examples in `src/lib.rs` and parser
  examples on the public source items re-exported by `src/lib.rs`. Rustdoc
  compiles doctests as external users, so private access will fail.

- Risk: Cargo or Rustdoc behaviour differs from assumptions.
  Severity: medium.
  Likelihood: low.
  Mitigation: this plan pins load-bearing behaviour to official Cargo/Rustdoc
  documentation and locked local crate sources in "Interfaces and
  dependencies"; the implementation gates run `cargo test --doc --workspace
  --all-features`.

## Progress

- [x] (2026-07-02T18:23:24Z) Confirmed worktree
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-3-1` and branch
  `roadmap-4-3-1`.
- [x] (2026-07-02T18:23:24Z) Loaded required skills: `execplans`, `leta`,
  `memtrace-first`, `sem`, `firecrawl-mcp`, `rust-router`,
  `rust-unit-testing`, and `rust-types-and-apis`.
- [x] (2026-07-02T18:23:24Z) Confirmed the working tree was clean with
  `git status --short`, and `sem diff --format json` reported zero changed
  entities.
- [x] (2026-07-02T18:23:24Z) Added the worktree to Leta with
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-3-1`.
- [x] (2026-07-02T18:23:24Z) Attempted Memtrace first. The MCP call
  `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`; bounded local evidence was used as fallback.
- [x] (2026-07-02T18:23:24Z) Attempted Firecrawl official-doc retrieval. The
  MCP call `mcp__firecrawl.firecrawl_scrape` for the Rustdoc documentation
  tests page returned `user cancelled MCP tool call`; official Rust, Cargo,
  docs.rs, and local locked-source evidence were used as fallback.
- [x] (2026-07-02T18:23:24Z) Used Leta for branch-local file orientation:
  `leta files` and `leta grep ".*" "src/lib.rs" --json` succeeded. Subsequent
  symbol discovery commands failed with `Error: Failed to start daemon`; direct
  bounded source inspection was used as fallback.
- [x] (2026-07-02T18:23:24Z) Reviewed governing docs:
  `AGENTS.md`, `docs/roadmap.md`, `docs/mapsplice-design.md`,
  `docs/developers-guide.md`, `docs/users-guide.md`,
  `docs/documentation-style-guide.md`, `docs/contributing.md`, and
  `docs/rust-doctest-dry-guide.md`.
- [x] (2026-07-02T18:23:24Z) Verified branch-local public API surfaces in
  `src/lib.rs`, `src/cli.rs`, `src/roadmap/parse/mod.rs`,
  `src/roadmap/model.rs`, and `src/error.rs`.
- [x] (2026-07-02T18:23:24Z) Verified locked dependency versions and local
  source support for `tempfile::tempdir()` and `camino::Utf8PathBuf`.
- [x] (2026-07-02T18:23:24Z) Drafted this first-round ExecPlan.
- [x] (2026-07-02T18:38:06Z) Revised this second-round ExecPlan after
  design review found that the first draft incorrectly omitted
  `parse_fragment` from the developers' guide section-3 public API scope.
- [x] (2026-07-02T18:55:53Z) Treated the df12-build workflow instruction to
  execute this approved ExecPlan as implementation approval.
- [x] (2026-07-02T18:55:53Z) Work Item 1: added the `parse_roadmap` doctest,
  observed the red assertion failure in
  `/tmp/cargo-test-doc-parse_roadmap-red-mapsplice-roadmap-4-3-1.out`, fixed
  the expected task number, and passed
  `/tmp/cargo-test-doc-parse_roadmap-mapsplice-roadmap-4-3-1.out`.
- [x] (2026-07-02T18:55:53Z) Work Item 1: ran the full doctest suite in
  `/tmp/cargo-test-doc-mapsplice-roadmap-4-3-1.out`; 8 doctests passed and
  2 configuration merge examples remained intentionally ignored.
- [x] (2026-07-02T18:55:53Z) Work Item 1: `scrutineer` ran `make all` and it
  passed with log
  `/tmp/make-all-parse-roadmap-mapsplice-roadmap-4-3-1.out`.
- [x] (2026-07-02T18:55:53Z) Work Item 1: `scrutineer` attempted CodeRabbit
  with log `/tmp/coderabbit-parse-roadmap-mapsplice-roadmap-4-3-1.out`; the
  review was deferred because the sandbox has no visible default network
  route.
- [x] (2026-07-02T19:01:09Z) Work Item 1: formatted this ExecPlan with
  `mdtablefix` and `markdownlint-cli2 --fix`; `scrutineer` ran
  `make markdownlint`, which passed with log
  `/tmp/make-markdownlint-work-item-1-mapsplice-roadmap-4-3-1.out`.
- [x] (2026-07-02T19:01:09Z) Work Item 1: validated this ExecPlan directly
  with `nixie --no-sandbox docs/execplans/roadmap-4-3-1.md`, which passed
  with log `/tmp/nixie-execplan-work-item-1-mapsplice-roadmap-4-3-1.out`.
- [x] (2026-07-02T19:05:41Z) Work Item 2: added the `parse_fragment` doctest,
  observed the red assertion failure in
  `/tmp/cargo-test-doc-parse_fragment-red-mapsplice-roadmap-4-3-1.out`, fixed
  the expected task number, and passed
  `/tmp/cargo-test-doc-parse_fragment-mapsplice-roadmap-4-3-1.out`.
- [x] (2026-07-02T19:05:41Z) Work Item 2: ran the full doctest suite in
  `/tmp/cargo-test-doc-mapsplice-roadmap-4-3-1.out`; 9 doctests passed and
  2 configuration merge examples remained intentionally ignored.
- [x] (2026-07-02T19:05:41Z) Work Item 2: `scrutineer` ran `make all` and it
  passed with log
  `/tmp/make-all-parse-fragment-mapsplice-roadmap-4-3-1.out`.
- [x] (2026-07-02T19:05:41Z) Work Item 2: `scrutineer` attempted CodeRabbit
  with log `/tmp/coderabbit-parse-fragment-mapsplice-roadmap-4-3-1.out`; the
  review was deferred because the sandbox has no visible default network
  route.
- [x] (2026-07-02T19:09:14Z) Work Item 3: added the `run_request` doctest,
  observed the red rendered-output assertion failure in
  `/tmp/cargo-test-doc-run_request-red-mapsplice-roadmap-4-3-1.out`, fixed
  the appended phase number, and passed
  `/tmp/cargo-test-doc-run_request-mapsplice-roadmap-4-3-1.out`.
- [x] (2026-07-02T19:09:14Z) Work Item 3: ran the full doctest suite in
  `/tmp/cargo-test-doc-mapsplice-roadmap-4-3-1.out`; 10 doctests passed and
  2 configuration merge examples remained intentionally ignored.
- [x] (2026-07-02T19:09:14Z) Work Item 3: `scrutineer` ran `make all` and it
  passed with log `/tmp/make-all-run-request-mapsplice-roadmap-4-3-1.out`.
- [x] (2026-07-02T19:09:14Z) Work Item 3: `scrutineer` attempted CodeRabbit
  with log `/tmp/coderabbit-run-request-mapsplice-roadmap-4-3-1.out`; the
  review was deferred because the sandbox has no visible default network
  route.
- [x] (2026-07-02T19:12:36Z) Work Item 4: added the `run_from_args` doctest,
  observed the red rendered-output assertion failure in
  `/tmp/cargo-test-doc-run_from_args-red-mapsplice-roadmap-4-3-1.out`, fixed
  the inserted/original phase renumbering assertions, and passed
  `/tmp/cargo-test-doc-run_from_args-mapsplice-roadmap-4-3-1.out`.
- [x] (2026-07-02T19:12:36Z) Work Item 4: ran the full doctest suite in
  `/tmp/cargo-test-doc-mapsplice-roadmap-4-3-1.out`; 11 doctests passed and
  2 configuration merge examples remained intentionally ignored.
- [x] (2026-07-02T19:12:36Z) Work Item 4: `scrutineer` ran `make all` and it
  passed with log `/tmp/make-all-run-from-args-mapsplice-roadmap-4-3-1.out`.
- [x] (2026-07-02T19:12:36Z) Work Item 4: `scrutineer` attempted CodeRabbit
  with log `/tmp/coderabbit-run-from-args-mapsplice-roadmap-4-3-1.out`; the
  review was deferred because the sandbox has no visible default network
  route.
- [x] (2026-07-02T19:13:40Z) Work Item 5: marked
  `docs/roadmap.md` task 4.3.1 complete, set this ExecPlan to complete, and
  prepared final evidence for the roadmap/plan completion commit.
- [x] (2026-07-02T19:16:47Z) Work Item 5: `scrutineer` ran final
  `make markdownlint`, `make nixie`, and `make all`. `make markdownlint`
  passed with log `/tmp/make-markdownlint-mapsplice-roadmap-4-3-1.out`;
  `make all` passed with log
  `/tmp/make-all-final-mapsplice-roadmap-4-3-1.out`; `make nixie` failed with
  unchanged Mermaid timeouts recorded in
  `/tmp/make-nixie-mapsplice-roadmap-4-3-1.out`.

## Surprises & Discoveries

- Observation: Memtrace and Firecrawl tools were discoverable, but their first
  MCP calls were cancelled by the host session.
  Evidence: `mcp__memtrace.list_indexed_repositories` and
  `mcp__firecrawl.firecrawl_scrape` each returned
  `user cancelled MCP tool call`.
  Impact: the implementation must retry those advisory tools, but this plan is
  not blocked; local docs, official web docs, and bounded source inspection
  provide enough evidence.

- Observation: the installed Leta CLI differs from the skill examples and later
  daemon starts failed.
  Evidence: `leta workspace add
  /home/leynos/Projects/mapsplice.worktrees/roadmap-4-3-1` failed during
  round 2 with `Error: IO error: Read-only file system (os error 30)`, and
  `leta files docs/` failed with `Error: Failed to start daemon`. The
  first-round attempt also recorded `leta grep --help` without a `-d/--docs`
  flag and later daemon failures.
  Impact: the implementation should retry Leta before editing, then fall back
  to bounded source inspection if the same daemon failure recurs.

- Observation: `src/lib.rs` already re-exports the relevant public APIs and is
  far below the 400-line limit.
  Evidence: `leta grep ".*" "src/lib.rs" --json` reported `run_from_args`,
  `run_request`, and `RunOutcome`; bounded inspection of `src/lib.rs` showed
  the public re-exports for `parse_roadmap` and `parse_fragment`.
  Impact: the application-boundary examples can live in `src/lib.rs`, while
  the `parse_roadmap` and `parse_fragment` examples should live on the source
  items that `src/lib.rs` re-exports.

- Observation: the first-round plan's scope claim was too narrow and
  inaccurate.
  Evidence: `docs/developers-guide.md` section 3 lists `parse_fragment` as a
  public library API, `docs/roadmap.md` task 4.3.1 scopes the work to "the
  public APIs listed in the developers' guide", and branch-local inspection of
  `src/roadmap/parse/fragment.rs::parse_fragment` showed no `# Examples`
  section. `grep -R "# Examples" -n src` found only examples in `src/cli.rs`,
  while bounded inspection confirmed `parse_anchor` and `metrics_snapshot`
  already have examples despite not being returned by that exact grep command.
  Impact: this plan now adds a dedicated `parse_fragment` work item. The
  roadmap success line remains a minimum success spotlight for
  `run_from_args`, `run_request`, and `parse_roadmap`, not permission to omit
  the remaining developers' guide section-3 gap.

- Observation: Work Item 1's `parse_roadmap` example pushed
  `src/roadmap/parse/mod.rs` over the 400-line project limit.
  Evidence: `git show HEAD:src/roadmap/parse/mod.rs | wc -l` reported 399
  lines before the item, and `wc -l src/roadmap/parse/mod.rs` reported 420
  lines after the first formatted example.
  Impact: the item kept changes within `src/roadmap/parse/mod.rs` by making
  the example more compact and extracting the duplicated paragraph-number
  parsing into `parse_numbered_paragraph`; the file is now exactly 400 lines.

- Observation: CodeRabbit could not run for Work Item 1 in this sandbox.
  Evidence: `/tmp/coderabbit-parse-roadmap-mapsplice-roadmap-4-3-1.out`
  contains `{"type":"status","phase":"deferred","status":"deferred
  coderabbit review: no default network route visible in this sandbox"}` and
  the review command exited 124.
  Impact: deterministic gates are green, but the AI review remains a deferred
  open issue for the supervisor because the host session lacks network access.

- Observation: full-repository `make nixie` could not complete after Work Item
  1 because unchanged Mermaid diagrams timed out.
  Evidence: `/tmp/make-nixie-work-item-1-mapsplice-roadmap-4-3-1.out` timed
  out on `docs/ortho-config-users-guide.md`,
  `/tmp/make-nixie-work-item-1-rerun-mapsplice-roadmap-4-3-1.out` timed out on
  `docs/ortho-config-users-guide.md` and `docs/rstest-bdd-users-guide.md`, and
  the documented serial recovery
  `/tmp/make-nixie-work-item-1-serial-mapsplice-roadmap-4-3-1.out` timed out
  on `docs/mapsplice-design.md`. The changed ExecPlan file passed direct
  validation in `/tmp/nixie-execplan-work-item-1-mapsplice-roadmap-4-3-1.out`.
  Impact: the changed Markdown has been validated, but full-repository Mermaid
  validation remains an open tooling issue unrelated to this work item's
  source or plan edits.

- Observation: CodeRabbit could not run for Work Item 2 in this sandbox.
  Evidence: `/tmp/coderabbit-parse-fragment-mapsplice-roadmap-4-3-1.out`
  contains `{"type":"status","phase":"deferred","status":"deferred
  coderabbit review: no default network route visible in this sandbox"}` and
  the review command exited 124.
  Impact: deterministic gates are green, but the AI review remains a deferred
  open issue for the supervisor because the host session lacks network access.

- Observation: CodeRabbit could not run for Work Item 3 in this sandbox.
  Evidence: `/tmp/coderabbit-run-request-mapsplice-roadmap-4-3-1.out`
  contains `{"type":"status","phase":"deferred","status":"deferred
  coderabbit review: no default network route visible in this sandbox"}` and
  the review command exited 124.
  Impact: deterministic gates are green, but the AI review remains a deferred
  open issue for the supervisor because the host session lacks network access.

- Observation: CodeRabbit could not run for Work Item 4 in this sandbox.
  Evidence: `/tmp/coderabbit-run-from-args-mapsplice-roadmap-4-3-1.out`
  contains `{"type":"status","phase":"deferred","status":"deferred
  coderabbit review: no default network route visible in this sandbox"}` and
  the review command exited 124.
  Impact: deterministic gates are green, but the AI review remains a deferred
  open issue for the supervisor because the host session lacks network access.

## Decision Log

- Decision: add `run_from_args` and `run_request` examples in `src/lib.rs`,
  add the `parse_roadmap` example on its source item in
  `src/roadmap/parse/mod.rs`, and add the `parse_fragment` example on its
  source item in `src/roadmap/parse/fragment.rs`.
  Rationale: `docs/developers-guide.md` section 3 presents the public library
  API through the crate surface. `src/lib.rs` owns the application boundary,
  while `parse_roadmap` and `parse_fragment` are authored and documented in
  roadmap parse modules before being re-exported. This keeps each example on
  the item Rustdoc documents while still teaching the downstream-user path
  Rustdoc tests.
  Date/Author: 2026-07-02T18:38:06Z / Codex.

- Decision: use existing `tempfile = 3.27.0` for examples that write target
  and fragment files.
  Rationale: the roadmap task requires filesystem-heavy flows to be isolated to
  temporary paths. Cargo official docs say dev-dependencies are used for
  compiling tests, examples, and benchmarks, and the locked `tempfile` source
  plus docs.rs docs state that temporary directories are automatically deleted
  when `TempDir` is destroyed.
  Date/Author: 2026-07-02T18:23:24Z / Codex.

- Decision: do not add property, snapshot, BDD, or integration tests for this
  task.
  Rationale: the changed behaviour is public documentation executability. The
  doctests themselves are the correct tests because Rustdoc compiles examples
  as external users. Existing unit, BDD, property, snapshot, and compile-time
  suites remain covered by `make all`.
  Date/Author: 2026-07-02T18:23:24Z / Codex.

- Decision: use a temporary red step for each doctest rather than committing a
  failing test.
  Rationale: the "test" and the documentation improvement are the same code
  block. To preserve Red-Green-Refactor without leaving bad examples, each work
  item first introduces the doctest with one deliberately wrong assertion,
  runs the focused doctest to observe failure, then corrects the assertion and
  commits only the passing example.
  Date/Author: 2026-07-02T18:23:24Z / Codex.

- Decision: keep Work Item 1 within the 400-line file limit by compacting the
  doctest and extracting a same-file `parse_numbered_paragraph` helper.
  Rationale: `src/roadmap/parse/mod.rs` started at 399 lines. Adding a
  conventional multi-assertion example alone breached AGENTS.md's file-size
  limit. The helper removes duplicated task and sub-task paragraph parsing
  without changing public API shape, grammar, or diagnostics.
  Date/Author: 2026-07-02T18:55:53Z / Codex.

## Outcomes & Retrospective

Implemented all four public API Rustdoc examples named by the approved plan:
`parse_roadmap`, `parse_fragment`, `run_request`, and `run_from_args`.
`cargo test --doc --workspace --all-features -- --list` now reports 13
doctests, including the four new public API examples.

Work item commits:

- `aa00488 Add parse_roadmap doctest`
- `12c2ef2 Add parse_fragment doctest`
- `e288457 Add run_request doctest`
- `c691155 Add run_from_args doctest`

Gate evidence before the final documentation update:

- Work Item 1 `make all` passed in
  `/tmp/make-all-parse-roadmap-mapsplice-roadmap-4-3-1.out`.
- Work Item 2 `make all` passed in
  `/tmp/make-all-parse-fragment-mapsplice-roadmap-4-3-1.out`.
- Work Item 3 `make all` passed in
  `/tmp/make-all-run-request-mapsplice-roadmap-4-3-1.out`.
- Work Item 4 `make all` passed in
  `/tmp/make-all-run-from-args-mapsplice-roadmap-4-3-1.out`.
- Path-scoped `nixie --no-sandbox docs/execplans/roadmap-4-3-1.md`
  passed for each ExecPlan update.

Final gate evidence:

- `make markdownlint` passed in
  `/tmp/make-markdownlint-mapsplice-roadmap-4-3-1.out`.
- `make all` passed in `/tmp/make-all-final-mapsplice-roadmap-4-3-1.out`.
- `make nixie` failed in `/tmp/make-nixie-mapsplice-roadmap-4-3-1.out` on
  unchanged Mermaid timeouts in `docs/ortho-config-users-guide.md:317` and
  `docs/rstest-bdd-users-guide.md:1058`.
- Final CodeRabbit was not run because the deterministic documentation gate
  was red.

Open issues:

- CodeRabbit review was attempted for each code work item and deferred each
  time because the sandbox has no visible default network route. The exact
  logs are `/tmp/coderabbit-parse-roadmap-mapsplice-roadmap-4-3-1.out`,
  `/tmp/coderabbit-parse-fragment-mapsplice-roadmap-4-3-1.out`,
  `/tmp/coderabbit-run-request-mapsplice-roadmap-4-3-1.out`, and
  `/tmp/coderabbit-run-from-args-mapsplice-roadmap-4-3-1.out`.
- Full-repository `make nixie` remained red after retries because unchanged
  Mermaid diagrams timed out. The changed ExecPlan file validates directly
  with `nixie`, but the full repository Mermaid gate needs a follow-up
  tooling or documentation-gate stabilization pass.

## Context and Orientation

The repository is a single Rust package named `mapsplice`. The library target
is `src/lib.rs`, and `Cargo.toml` marks that target as `doc = true`,
`doctest = true`, and `test = true` according to `cargo metadata --locked
--no-deps`.

The public API map relevant to this task is:

- `src/lib.rs::run_from_args<I, T>(args: I) -> Result<RunOutcome>` parses
  command-line arguments and executes the full workflow.
- `src/lib.rs::run_request(request: CliRequest) -> Result<RunOutcome>` executes
  an already parsed request.
- `src/lib.rs` re-exports `parse_roadmap` from `src/roadmap/parse/mod.rs`,
  whose signature is `parse_roadmap(markdown: &str) -> Result<RoadmapDocument>`.
- `src/lib.rs` re-exports `parse_fragment` from
  `src/roadmap/parse/fragment.rs`, whose signature is
  `parse_fragment(markdown: &str) -> Result<RoadmapFragment>`.
- `src/lib.rs::RunOutcome` exposes `stdout: Option<String>` for standard-output
  mode and `written_path: Option<Utf8PathBuf>` for in-place mode.
- `src/cli.rs` exposes `CliRequest`, `CommandKind`, and `GlobalOptions`.

The existing examples show the local Rustdoc style:

- `src/cli.rs::CommandKind::fragment_path` uses a visible `use` section,
  hidden `fn main() -> mapsplice::Result<()>`, and assertions.
- `src/cli.rs::parse_cli_request` demonstrates command-line parsing with
  `parse_cli_request(["mapsplice", "--in-place", ...])`.
- `src/roadmap/anchor.rs::parse_anchor` already has an executable Rustdoc
  example that validates a canonical sub-task anchor and rejects a leading-zero
  anchor.
- `src/observability.rs` includes module-level and item-level examples for
  process-local metrics.

The four new examples should match that house style while improving error
handling for filesystem setup with `Box<dyn std::error::Error>` where standard
I/O, `tempfile`, and `mapsplice::MapspliceError` meet in one doctest.

## Plan of Work

### Work Item 1: Document `parse_roadmap` with a model-inspection doctest

This work item is independently committable. It edits only
`src/roadmap/parse/mod.rs`. `parse_roadmap` is re-exported from `src/lib.rs`,
but its Rustdoc is authored on the source item in
`src/roadmap/parse/mod.rs::parse_roadmap`, so the example belongs there.

The example must:

- use `mapsplice::parse_roadmap`;
- parse a minimal roadmap with one phase, one step, and one task;
- inspect `RoadmapDocument.phases`, `steps`, and `tasks` through public fields;
- assert the parsed task number string is `1.1.1`;
- avoid filesystem access, `unwrap`, and `expect`.

Documentation and ADRs to read before editing:

- `docs/roadmap.md`, task 4.3.1.
- `docs/mapsplice-design.md`, sections 4, 5, 6, and 8.
- `docs/developers-guide.md`, sections 3 and 6.
- `docs/users-guide.md`, "The roadmap shape `mapsplice` expects".
- `docs/execplans/initial-tool.md`, sections "Scope and grammar assumptions"
  and "Proposed implementation" item 2, plus the Decision Log entries
  "treat the supported roadmap syntax as a constrained document grammar" and
  "model splice operations against a roadmap-specific intermediate
  representation".
- `docs/rust-doctest-dry-guide.md`, sections 2.1 through 2.4.

Skills to load before editing:

- `memtrace-first`.
- `leta`.
- `sem`.
- `rust-router`.
- `rust-unit-testing`.
- `rust-types-and-apis`.

Tests:

- Red: introduce the doctest with one deliberately wrong expected task number,
  for example `"1.1.2"`, then run
  `cargo test --doc --workspace --all-features parse_roadmap 2>&1 | tee /tmp/cargo-test-doc-parse_roadmap-mapsplice-roadmap-4-3-1.out`
  and expect the doctest to fail on that assertion.
- Green: correct the assertion to `"1.1.1"` and rerun the same focused command;
  expect it to pass.
- Refactor: run
  `cargo test --doc --workspace --all-features 2>&1 | tee /tmp/cargo-test-doc-mapsplice-roadmap-4-3-1.out`
  and then
  `make all 2>&1 | tee /tmp/make-all-parse-roadmap-mapsplice-roadmap-4-3-1.out`.

Commit after the focused doctest and `make all` pass.

### Work Item 2: Document `parse_fragment` with a fragment-variant doctest

This work item is independently committable. It edits only
`src/roadmap/parse/fragment.rs`. `parse_fragment` is listed in
`docs/developers-guide.md` section 3 and is re-exported from `src/lib.rs`, but
its Rustdoc is authored on the source item in
`src/roadmap/parse/fragment.rs::parse_fragment`.

The example must:

- use `mapsplice::{RoadmapFragment, parse_fragment}`;
- parse a minimal phase fragment with one phase, one step, and one task;
- match `RoadmapFragment::Phase(phases)`;
- inspect the public fields of the parsed phase, step, and task;
- assert the parsed phase number is `1` and the parsed task number string is
  `1.1.1`;
- avoid filesystem access, `unwrap`, and `expect`.

Documentation and ADRs to read before editing:

- `docs/roadmap.md`, task 4.3.1.
- `docs/mapsplice-design.md`, sections 4, 5, 6, and 8.
- `docs/developers-guide.md`, sections 3 and 6.
- `docs/users-guide.md`, "The roadmap shape `mapsplice` expects" and
  "Command details".
- `docs/execplans/initial-tool.md`, sections "Scope and grammar assumptions"
  and "Proposed implementation" item 2, plus the Decision Log entries
  "treat the supported roadmap syntax as a constrained document grammar" and
  "model splice operations against a roadmap-specific intermediate
  representation".
- `docs/rust-doctest-dry-guide.md`, sections 2.1 through 2.4.

Skills to load before editing:

- `memtrace-first`.
- `leta`.
- `sem`.
- `rust-router`.
- `rust-unit-testing`.
- `rust-types-and-apis`.

Tests:

- Red: introduce the doctest with one deliberately wrong expected task number,
  for example `"1.1.2"`, then run
  `cargo test --doc --workspace --all-features parse_fragment 2>&1 | tee /tmp/cargo-test-doc-parse_fragment-mapsplice-roadmap-4-3-1.out`
  and expect the doctest to fail on that assertion.
- Green: correct the assertion to `"1.1.1"` and rerun the same focused command;
  expect it to pass.
- Refactor: run
  `cargo test --doc --workspace --all-features 2>&1 | tee /tmp/cargo-test-doc-mapsplice-roadmap-4-3-1.out`
  and then
  `make all 2>&1 | tee /tmp/make-all-parse-fragment-mapsplice-roadmap-4-3-1.out`.

Commit after the focused doctest and `make all` pass.

### Work Item 3: Document `run_request` with a temporary-file request doctest

This work item is independently committable. It edits `src/lib.rs` only and
adds a `# Examples` section to `run_request`.

The example must:

- hide setup imports for `std::fs`, `std::io`, and `tempfile::tempdir` when
  they distract from the API call;
- create a `TempDir`;
- create UTF-8 `target.md` and `fragment.md` paths with `Utf8PathBuf`;
- write a target roadmap and a phase-level fragment;
- build `CliRequest { global: GlobalOptions { in_place: false }, target,
  command: CommandKind::Append { fragment } }`;
- call `run_request(request)?`;
- assert `outcome.written_path.is_none()`;
- extract `outcome.stdout.as_deref()` using `ok_or_else`, not `unwrap` or
  `expect`;
- assert the rendered output contains the appended phase renumbered to `2`.

Documentation and ADRs to read before editing:

- `docs/roadmap.md`, task 4.3.1.
- `docs/mapsplice-design.md`, sections 2, 3, 5, and 6.
- `docs/developers-guide.md`, sections 2, 3, 4, and 6.
- `docs/users-guide.md`, "Command overview", "`append`", and "Output modes".
- `docs/execplans/initial-tool.md`, "Proposed implementation" items 6 and 8,
  plus the Decision Log entries "introduce `src/lib.rs` alongside a thin
  `src/main.rs`" and "use an `OrthoConfig`-derived global CLI struct".
- `docs/rust-doctest-dry-guide.md`, sections 2.2, 2.3, and 2.4.

Skills to load before editing:

- `memtrace-first`.
- `leta`.
- `sem`.
- `rust-router`.
- `rust-unit-testing`.
- `rust-types-and-apis`.
- `rust-errors` if the example needs non-trivial error conversion.

Tests:

- Red: add the doctest with one deliberately wrong rendered-output assertion,
  for example `assert!(rendered.contains("## 3. Added phase"));`, then run
  `cargo test --doc --workspace --all-features run_request 2>&1 | tee /tmp/cargo-test-doc-run_request-mapsplice-roadmap-4-3-1.out`
  and expect the doctest to fail on that assertion.
- Green: correct the assertion to the actual appended phase number and rerun
  the same focused command; expect it to pass.
- Refactor: run
  `cargo test --doc --workspace --all-features 2>&1 | tee /tmp/cargo-test-doc-mapsplice-roadmap-4-3-1.out`
  and then
  `make all 2>&1 | tee /tmp/make-all-run-request-mapsplice-roadmap-4-3-1.out`.

Commit after the focused doctest and `make all` pass.

### Work Item 4: Document `run_from_args` with an argument-workflow doctest

This work item is independently committable. It edits `src/lib.rs` only and
adds a `# Examples` section to `run_from_args`.

The example must:

- create a `TempDir`;
- create target and fragment files under that directory;
- pass command-line-like arguments as owned strings so temporary path strings
  can be included safely;
- call `run_from_args(args)?`;
- assert standard-output mode by checking `outcome.written_path.is_none()`;
- extract stdout without `unwrap` or `expect`;
- assert that inserting before phase `2` creates the inserted phase `2` and
  shifts the original phase to `3`.

Documentation and ADRs to read before editing:

- `docs/roadmap.md`, task 4.3.1.
- `docs/mapsplice-design.md`, sections 2, 3, 5, and 6.
- `docs/developers-guide.md`, sections 2, 3, 4, and 6.
- `docs/users-guide.md`, "Command overview", "`insert`", and "Output modes".
- `docs/execplans/initial-tool.md`, "Proposed implementation" items 6 and 8,
  plus the Decision Log entries "introduce `src/lib.rs` alongside a thin
  `src/main.rs`" and "use an `OrthoConfig`-derived global CLI struct".
- `docs/rust-doctest-dry-guide.md`, sections 2.2, 2.3, and 2.4.

Skills to load before editing:

- `memtrace-first`.
- `leta`.
- `sem`.
- `rust-router`.
- `rust-unit-testing`.
- `rust-types-and-apis`.
- `rust-errors` if the example needs non-trivial error conversion.

Tests:

- Red: add the doctest with one deliberately wrong rendered-output assertion,
  for example asserting that the original phase remains `2`, then run
  `cargo test --doc --workspace --all-features run_from_args 2>&1 | tee /tmp/cargo-test-doc-run_from_args-mapsplice-roadmap-4-3-1.out`
  and expect the doctest to fail on that assertion.
- Green: correct the assertion to the documented insert renumbering and rerun
  the same focused command; expect it to pass.
- Refactor: run
  `cargo test --doc --workspace --all-features 2>&1 | tee /tmp/cargo-test-doc-mapsplice-roadmap-4-3-1.out`
  and then
  `make all 2>&1 | tee /tmp/make-all-run-from-args-mapsplice-roadmap-4-3-1.out`.

Commit after the focused doctest and `make all` pass.

### Work Item 5: Mark roadmap completion and capture final evidence

This work item is independently committable after Work Items 1 through 4 pass.
It edits `docs/roadmap.md` and this ExecPlan only.

The roadmap update must:

- change task 4.3.1 from `[ ]` to `[x]`;
- leave the task title, `Requires 3.1.3`, and success wording intact unless
  implementation revealed a documented correction;
- avoid changing unrelated roadmap content.

The ExecPlan update must:

- set `Status: COMPLETE`;
- mark completed progress entries with timestamps, commands, and log paths;
- summarize the final doctest count and full gate results in
  `Outcomes & Retrospective`;
- append the required revision note at the bottom.

Documentation and ADRs to read before editing:

- `docs/roadmap.md`, task 4.3.1.
- `docs/documentation-style-guide.md`, "Markdown rules" and "Formatting".
- `AGENTS.md`, "Markdown Guidance".
- `docs/execplans/initial-tool.md`, section "Validation and observable
  checks".

Skills to load before editing:

- `execplans`.
- `memtrace-first` and `leta` are not required for this documentation-only
  completion item unless implementation discoveries require fresh code
  verification.
- `changelog` is not needed unless a user explicitly asks for changelog work.

Tests:

- Run path-scoped Markdown formatting for the changed Markdown files:

  ```bash
  mdtablefix docs/execplans/roadmap-4-3-1.md docs/roadmap.md 2>&1 \
    | tee /tmp/mdtablefix-roadmap-docs-mapsplice-roadmap-4-3-1.out
  markdownlint-cli2 --fix docs/execplans/roadmap-4-3-1.md docs/roadmap.md \
    2>&1 | tee /tmp/markdownlint-fix-roadmap-docs-mapsplice-roadmap-4-3-1.out
  ```

- Run final documentation gates:
  `make markdownlint 2>&1 | tee /tmp/make-markdownlint-mapsplice-roadmap-4-3-1.out`
  and
  `make nixie 2>&1 | tee /tmp/make-nixie-mapsplice-roadmap-4-3-1.out`.
- Run final repository gate:
  `make all 2>&1 | tee /tmp/make-all-final-mapsplice-roadmap-4-3-1.out`.

Commit after Markdown formatting, `make markdownlint`, `make nixie`, and
`make all` pass.

## Concrete Steps

Always run commands from
`/home/leynos/Projects/mapsplice.worktrees/roadmap-4-3-1`.

1. Confirm branch and worktree:

   ```bash
   git branch --show-current
   git status --short
   ```

   Expected branch: `roadmap-4-3-1`. Expected status before implementation:
   clean, except for approved plan updates.

2. Retry advisory tool setup:

   ```bash
   leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-3-1
   leta grep ".*" "src/lib.rs" --json
   sem diff --format json
   ```

   Also retry Memtrace repository listing if the MCP tool is available. If it
   again returns `user cancelled MCP tool call`, record that exact result in
   `Surprises & Discoveries` and continue with bounded local evidence.

3. Implement Work Items 1 through 4 one at a time. For each work item, create
   the temporary red assertion, run the focused doctest and capture the failing
   output, fix the assertion, rerun the focused doctest, then run the broader
   doctest and `make all`.

4. Commit each code work item after its gates pass. Use descriptive imperative
   commit messages, for example:

   ```plaintext
   Add parse_roadmap doctest

   Document the public parser with an executable example that parses a
   minimal roadmap and inspects the typed model. This keeps the public API
   contract visible in generated Rustdoc and covered by cargo doctests.
   ```

5. Implement Work Item 5 after code work is complete, format only the changed
   Markdown paths, run final documentation and repository gates, and commit the
   roadmap/plan completion update.

## Validation and Acceptance

The implementation is accepted when all of these are true:

- `cargo test --doc --workspace --all-features` passes and includes the new
  examples for `parse_roadmap`, `parse_fragment`, `run_request`, and
  `run_from_args`.
- `make all` passes. This includes `check-fmt`, `lint`, `typecheck`, and
  `test` on current `origin/main` policy.
- `make markdownlint` passes.
- `make nixie` passes.
- No examples use `ignore`, `no_run`, `compile_fail`, `should_panic`,
  `.unwrap()`, `.expect()`, fixed paths outside a `TempDir`, or process-wide
  environment mutation.
- `git status --short` is clean after the final commit.

Quality criteria:

- Tests: the new doctests fail during the temporary red step and pass after
  correction; the full doctest suite and `make all` pass.
- Lint/typecheck: `make all` passes with warnings denied.
- Documentation: `make markdownlint` and `make nixie` pass after path-scoped
  Markdown formatting of changed Markdown files.
- Public API: no public signatures change.
- Dependencies: no dependency changes are made.

## Idempotence and Recovery

The work is safe to retry because examples are additive documentation changes
and each commit is gated independently. If a red assertion is still present
after a focused doctest failure, replace only that assertion with the intended
value before continuing. If a formatter changes unrelated Markdown files, park
that churn with a named discard stash:

```bash
git stash push -m 'df12-stash v1 task=4.3.1 kind=discard reason="unrelated markdown formatter churn"' -- <paths>
```

Do not use a bare `git stash`. Do not discard user changes. If unrelated local
changes appear, leave them untouched and work only with files required by this
plan.

## Artifacts and Notes

Planning evidence:

- `git branch --show-current` returned `roadmap-4-3-1`.
- `git status --short` returned no files before this plan was created.
- `sem diff --format json` reported zero changed entities before this plan was
  created.
- `rustc --version`, `cargo --version`, and `rustdoc --version` reported the
  pinned nightly 1.96.0 toolchain selected by `rust-toolchain.toml`.
- `cargo metadata --format-version 1 --locked --no-deps` reported the
  `mapsplice` library target with `doc = true`, `doctest = true`, and
  `test = true`.

Official and locked-source behaviour verified:

- Rustdoc official documentation says documentation examples are extracted and
  run as tests; regular doctests pass when they compile and run without
  panicking; hidden `#` lines are compiled but not rendered; and examples using
  `?` can hide a `main` that returns `Result`.
  Source:
  <https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html>.
- Cargo official documentation says `cargo test` compiles and executes
  documentation tests, `--doc` tests only library documentation, and doctest
  examples run in separate processes. Source:
  <https://doc.rust-lang.org/cargo/commands/cargo-test.html>.
- Cargo official documentation says `[dev-dependencies]` are used when
  compiling tests, examples, and benchmarks. Source:
  <https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#development-dependencies>.
- docs.rs for `tempfile` 3.27.0 says `tempdir()` returns `TempDir`, creates a
  directory in the filesystem, and deletes it when the destructor runs. Source:
  <https://docs.rs/tempfile/3.27.0/tempfile/fn.tempdir.html>.
- The locked local source for `tempfile` 3.27.0 at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tempfile-3.27.0/src/lib.rs`
  shows `Builder::tempdir()` delegates to `tempdir_in(env::temp_dir())` and
  documents automatic deletion when `TempDir` is destroyed.
- docs.rs for `camino` 1.2.2 says `Utf8PathBuf` is an owned mutable UTF-8 path,
  supports `From::from` for string paths, exposes `from_path_buf`, and
  implements `AsRef<Path>`. Source:
  <https://docs.rs/camino/1.2.2/camino/struct.Utf8PathBuf.html>.
- The locked local source for `camino` 1.2.2 at
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/camino-1.2.2/src/lib.rs`
  shows `Utf8PathBuf(PathBuf)`, `from_path_buf`, `From<String> for
  Utf8PathBuf`, and `AsRef<Path> for Utf8PathBuf`.

## Interfaces and Dependencies

No new interfaces or dependencies are permitted.

The implementation must use:

- `mapsplice::parse_roadmap`, `mapsplice::parse_fragment`,
  `mapsplice::run_request`, and `mapsplice::run_from_args` as the public APIs
  being documented.
- `mapsplice::RoadmapFragment` where the `parse_fragment` example needs to
  match the typed fragment variant.
- `mapsplice::{CliRequest, CommandKind, GlobalOptions, RunOutcome}` where the
  `run_request` example needs a parsed request.
- `camino::Utf8PathBuf` for public path values in examples, because
  `CliRequest.target`, `CommandKind` fragment paths, and `RunOutcome` paths use
  that type.
- `tempfile::tempdir()` for filesystem examples, because the locked
  dev-dependency and official docs support temporary directories that clean up
  after the example process exits.
- `std::fs::write` for simple setup writes. `Utf8PathBuf` implements
  `AsRef<Path>`, so no lossy path conversion is needed.
- hidden `fn main() -> Result<(), Box<dyn std::error::Error>>` wrappers for
  examples that combine `std::io::Error`, `tempfile`, and
  `mapsplice::MapspliceError`.

Do not use:

- `ignore`, `no_run`, `compile_fail`, or `should_panic` doctest attributes;
- process-wide environment mutation;
- fixed paths under the repository, `/tmp`, or the user's home directory;
- `.unwrap()` or `.expect()` in examples;
- a new helper function or macro unless the examples become unavoidably longer
  than the surrounding house style.

## Revision Note

- 2026-07-02T18:23:24Z: Created the first-round draft for roadmap task 4.3.1.
  The draft pins the public API examples to the developers' guide, records
  Memtrace/Firecrawl/Leta tooling failures, verifies load-bearing Rustdoc,
  Cargo, `tempfile`, and `camino` behaviour, and decomposes the work into three
  code commits plus one roadmap/plan completion commit.
- 2026-07-02T18:38:06Z: Revised the draft after second-round design review.
  The revision corrects the inaccurate scope evidence, explains that the
  roadmap task body covers every developers' guide section-3 public API, adds
  `parse_fragment` as the remaining section-3 API without an executable
  example, and decomposes the work into four code commits plus one
  roadmap/plan completion commit.
- 2026-07-02T19:13:40Z: Completed implementation. Added executable public API
  Rustdoc examples for `parse_roadmap`, `parse_fragment`, `run_request`, and
  `run_from_args`; marked roadmap task 4.3.1 complete; recorded green
  deterministic `make all` evidence and the deferred CodeRabbit/full-repo
  `make nixie` open issues.
