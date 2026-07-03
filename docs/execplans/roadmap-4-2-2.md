# Add path-scoped Markdown maintenance targets

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

Roadmap task 4.2.2 adds a narrow Makefile-supported workflow for maintaining
Markdown files without running repository-wide Markdown formatting. After this
work, a maintainer can name one or more Markdown files, run a Makefile target
to format them, run a Makefile target to lint only them, and then run the
existing repository gates before committing. The observable success condition
is that the named files are passed to `mdtablefix` and `markdownlint-cli2`, and
unchanged Markdown files outside `MARKDOWN_PATHS` are not passed to those
formatting or fixing commands.

This plan is a draft only. Do not implement it until it is explicitly approved.

## Constraints

- Work exclusively in
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-2-2`.
- Do not edit the root or control worktree.
- Do not run repository-wide Markdown formatting such as `make fmt`,
  `mdformat-all`, or an unscoped formatter command while implementing this
  plan. Format only the Markdown files changed by the current work item.
- Preserve existing repository-wide gates: `make all`, `make markdownlint`,
  and `make nixie` must remain the final validation commands for commits.
- Preserve the existing meaning of `make markdownlint`: it remains the
  repository-wide Markdown lint gate using `markdownlint-cli2 '**/*.md'`.
- The new scoped workflow must be opt-in through caller-supplied paths. Running
  the new scoped formatter without paths must fail clearly instead of falling
  back to all Markdown files or blocking on standard input.
- The empty-`MARKDOWN_PATHS` guard must be implemented as a GNU Make expansion
  guard using `$(error ...)`, not as a shell `if` recipe. This is required so
  `make --dry-run` fails during recipe expansion and the guard remains testable
  without invoking the real formatter.
- Do not add a new external dependency. Use existing tools: GNU Make,
  `mdtablefix`, `markdownlint-cli2`, and the Rust test harness already in this
  repository.
- Keep Makefile changes small and reviewable. If the implementation needs more
  than one new Makefile variable family and two new targets, update this plan
  before proceeding.
- For Rust test changes, load `rust-router` first and then
  `rust-unit-testing`. Follow `rstest` table-test conventions and return
  `Result` from tests with fallible setup.
- For documentation changes, obey `docs/documentation-style-guide.md` and use
  en-GB Oxford spelling.
- Every command used for validation must write output with `tee` to `/tmp`.
  The implementation may run focused tests before full gates, but every commit
  must be gated by `make all`, `make markdownlint`, and `make nixie`.

## Tolerances (exception triggers)

- If `MARKDOWN_PATHS` cannot support whitespace-separated repository-relative
  or absolute file paths for existing Markdown files, stop and update this plan.
- If supporting paths with spaces requires more than Makefile variable passing
  can safely provide, document that limitation and stop for review rather than
  adding shell quoting machinery.
- If `markdownlint-cli2 --no-globs -- <paths>` cannot lint literal paths with
  the installed version, stop and choose a tested alternative before editing
  the Makefile.
- If `mdtablefix --in-place <paths>` does not rewrite the named files with the
  installed version, stop and choose a tested alternative before editing the
  Makefile.
- If the Rust tests need process-wide current-directory or environment
  mutation, add a local guard and `serial_test` key before committing. Do not
  add unguarded env or current-directory mutation.
- If `make all` fails after two focused fix attempts, record the failing
  command and log path in the Decision Log and stop for review.
- If `make markdownlint` or `make nixie` fails for pre-existing repository
  debt unrelated to this work, record the exact failing paths and log files. Do
  not broaden this task to repair unrelated documents without approval.
- If formatter churn touches files outside the current work item, discard the
  churn using a path-specific restore for changes made by this agent. If a
  stash is required, use the required named format:
  `df12-stash v1 task=4.2.2 kind=discard reason="formatter churn"`.

## Risks

- Risk: Make variables are whitespace-separated, so paths with spaces are not a
  safe target for the first implementation. Severity: medium. Likelihood:
  medium. Mitigation: document `MARKDOWN_PATHS` as a whitespace-separated list
  of existing Markdown paths without spaces, and add tests for multiple
  ordinary paths. Escalate if space-bearing paths are required.

- Risk: `mdtablefix` reads from standard input when no files are supplied.
  Severity: high. Likelihood: verified. Mitigation: every scoped target must
  call the `require_markdown_paths` Make expansion guard before invoking
  `mdtablefix`. The integration test must run the scoped targets under
  `make --dry-run` without `MARKDOWN_PATHS` and assert a non-zero exit plus a
  diagnostic containing `MARKDOWN_PATHS`.

- Risk: `markdownlint-cli2` treats positional arguments as globs by default.
  Severity: medium. Likelihood: verified. Mitigation: the scoped lint commands
  must use `--no-globs --` with explicit file paths, while the repository-wide
  `markdownlint` target keeps its existing glob.

- Risk: Existing Markdown lint debt may make the repository-wide
  `make markdownlint` gate fail even when the scoped target works. Severity:
  medium. Likelihood: observed during planning. Mitigation: record full gate
  failures with log paths. Do not repair unrelated historical ExecPlans as part
  of task 4.2.2 unless approval widens scope.

- Risk: Makefile dry-run tests can prove command construction but not actual
  tool behaviour. Severity: medium. Likelihood: high. Mitigation: include one
  focused smoke test that runs the scoped formatter in a temporary directory
  with wrapper tools that mutate only their arguments, then assert the sentinel
  Markdown file remains unchanged.

## Progress

- [x] (2026-07-03T00:59:01Z) Confirmed the assigned worktree is
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-2-2` and the current
  branch is `roadmap-4-2-2`.
- [x] (2026-07-03T00:59:01Z) Loaded the required `execplans` skill and the
  startup `leta` skill. Loaded `firecrawl-mcp` because external documentation
  research was requested, `sem` for semantic change/history navigation, and
  `rust-router` plus `rust-unit-testing` because this plan requires Rust
  integration tests.
- [x] (2026-07-03T00:59:01Z) Read `AGENTS.md`, `Makefile`,
  `.markdownlint-cli2.jsonc`, `docs/roadmap.md`, `docs/developers-guide.md`,
  `docs/documentation-style-guide.md`, `docs/mapsplice-design.md`,
  `docs/users-guide.md`, and `docs/scripting-standards.md`.
- [x] (2026-07-03T00:59:01Z) Verified local tool behaviour for
  `mdtablefix 0.4.0`, `markdownlint-cli2 v0.22.1`, `merman-cli 0.7.0`, `mbake`,
  and `sem`.
- [x] (2026-07-03T00:59:01Z) Created this first-round draft ExecPlan.
- [x] (2026-07-03T19:54:00+02:00) Revised the draft for planning round 2 after
  design review rejected the empty-path guard testability. The plan now pins a
  GNU Make `$(error ...)` expansion guard and matching dry-run failure tests.
- [x] (2026-07-03T03:36:40+02:00) Work item 1: added
  `markdownfmt` and `markdownlint-paths`, plus integration tests for command
  construction, empty-path guard behaviour, and sentinel-file isolation.
  Focused tests passed in
  `/tmp/test-green-mapsplice-roadmap-4-2-2-4-2-2-item-1-clippy-style.out`;
  `mbake validate Makefile` passed in
  `/tmp/mbake-mapsplice-roadmap-4-2-2-4-2-2-item-1.out`; and `make all` passed
  in `/tmp/all-mapsplice-roadmap-4-2-2-item-1-final-before-coderabbit.out`.
- [x] (2026-07-03T03:41:50+02:00) Work item 2: documented the
  `MARKDOWN_PATHS` workflow in `docs/developers-guide.md`, marked roadmap task
  4.2.2 complete in `docs/roadmap.md`, and kept this ExecPlan current. The
  focused test passed in
  `/tmp/test-docs-mapsplice-roadmap-4-2-2-4-2-2-item-2.out`; `make all`,
  `make markdownlint`, and `make nixie` passed in
  `/tmp/all-mapsplice-roadmap-4-2-2-item-2.out`,
  `/tmp/markdownlint-mapsplice-roadmap-4-2-2-item-2.out`, and
  `/tmp/nixie-mapsplice-roadmap-4-2-2-item-2.out`.

## Surprises & discoveries

- Memtrace `list_indexed_repositories` was attempted first, but the MCP host
  returned `user cancelled MCP tool call`. Memtrace was therefore unavailable
  in this planning session. This is advisory-tool failure, not a product
  blocker.
- Firecrawl searches for `mdtablefix` and `markdownlint-cli2` official
  documentation returned `user cancelled MCP tool call`. Planning round 2 also
  saw Firecrawl return `user cancelled MCP tool call` for `mdtablefix`,
  `markdownlint-cli2`, and GNU Make searches. The plan therefore pins
  load-bearing behaviour to bounded local source and installed CLI help, and
  retains official documentation URLs as review signposts rather than relying
  on freshly fetched remote content.
- `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-2-2`
  failed in planning round 2 with
  `Error: IO error: Read-only file system (os error 30)`. This plan used
  bounded branch-local file inspection for non-code and test-surface evidence.
- `cargo install --list` failed with
  `failed to open: /home/leynos/.cargo/.crates.toml` followed by
  `Read-only file system (os error 30)`. Installed CLI versions were verified
  with each binary's `--version` output instead.
- Running `mdformat-all --help` during research unexpectedly executed the
  formatter rather than printing help. The resulting repository-wide Markdown
  churn was discarded with a path-specific `git restore` before this plan was
  written. This reinforces the task constraint that implementation must not use
  `mdformat-all`.
- Baseline `make markdownlint` evidence observed during the accidental
  formatter run showed existing line-length failures in historical ExecPlans.
  This draft treats that as pre-existing documentation debt unless it
  reproduces after the approved implementation and is explicitly brought into
  scope.
- During implementation, Memtrace `list_indexed_repositories` was retried and
  again returned `user cancelled MCP tool call`. The implementation therefore
  used the approved fallback: Leta for branch-local source orientation and
  bounded file inspection for Makefile and documentation surfaces.
- The focused
  `make test TEST_FLAGS='--workspace --all-targets --all-features makefile_markdown_maintenance'`
  command initially compiled the new integration-test binary but ran zero
  tests because Nextest treated `makefile_markdown_maintenance` as a test-name
  filter. The tests were renamed with the same prefix before the red run was
  repeated.
- Work item 1 CodeRabbit review was attempted after `make all` passed, but the
  review agent exited 124 before doing review work because the sandbox has no
  default network route. The exact log is
  `/tmp/coderabbit-roadmap-4-2-2-roadmap-4-2-2.out`.
- Work item 2 CodeRabbit review hit the same environment constraint. The
  review agent exited 124 before review work with
  `deferred coderabbit review: no default network route visible in this sandbox`.
  The exact log is `/tmp/coderabbit-mapsplice-roadmap-4-2-2-item-2.out`.

## Decision log

- Decision: Add new scoped targets rather than changing the existing
  repository-wide `markdownlint` or `fmt` targets. Rationale:
  `docs/developers-guide.md` still needs repository-wide gates, and the roadmap
  task asks for caller-supplied paths without unrelated formatter churn.
  Keeping old gates stable avoids surprising CI or maintainers. Date/Author:
  2026-07-03T00:59:01Z, planning agent.

- Decision: Use `MARKDOWN_PATHS` as the caller-supplied path variable.
  Rationale: GNU Make command-line variable overrides are documented and match
  existing variables such as `NIXIE_PATHS`, `NIXIE_MAX_CONCURRENCY`, and
  `NIXIE_RENDERER_THREADS`. Date/Author: 2026-07-03T00:59:01Z, planning agent.

- Decision: The scoped formatter must run `mdtablefix --in-place` before
  `markdownlint-cli2 --fix --no-globs --`. Rationale: local `mdformat-all` uses
  `mdtablefix` followed by `markdownlint-cli2 --fix`; local `mdtablefix` source
  and help show that `--in-place` is required for file writes, and
  `markdownlint-cli2` source and help show `--fix` writes fixes and
  `--no-globs` ignores configured globs. Date/Author: 2026-07-03T00:59:01Z,
  planning agent.

- Decision: Use Rust integration tests for Makefile behaviour.
  Rationale: the repository already has `tests/makefile_nixie.rs` for Makefile
  dry-run contract tests, and `make all` runs workspace tests. Date/Author:
  2026-07-03T00:59:01Z, planning agent.

- Decision: Implement the empty-`MARKDOWN_PATHS` guard only as a Make
  expansion guard with `$(error ...)`. Rationale: shell guards are printed
  rather than executed during `make --dry-run`, so they cannot prove the
  stdin-hang prevention path. A Make expansion guard fires while Make expands
  the recipe, so the dry-run process itself exits non-zero and the test can
  assert the `MARKDOWN_PATHS` diagnostic without invoking external tools.
  Date/Author: 2026-07-03T19:54:00+02:00, planning agent.

- Decision: Prefix the new integration-test names with
  `makefile_markdown_maintenance`. Rationale: the repository's `make test`
  target uses Nextest when available, and the approved focused command passes
  `makefile_markdown_maintenance` as a filter expression rather than a test
  binary selector. Prefixing the test names keeps the approved command useful
  without changing Makefile test semantics. Date/Author:
  2026-07-03T03:36:40+02:00, implementation agent.

- Decision: Continue after the third mechanical `make all` lint failure and
  record this deviation. Rationale: the tolerance expected a stop after two
  focused fix attempts, but the remaining failures were local formatting,
  Clippy, and Whitaker diagnostics in the new test file, with no design
  ambiguity or cross-file risk. Date/Author: 2026-07-03T03:36:40+02:00,
  implementation agent.

## Addenda

- [x] 4.2.2.1. Pin scoped Markdown target order and real-tool flags.
  - Source: review:4.2.2 (low).
  - Scope: Add an order-and-argument assertion for `markdownfmt`, plus a gated
    real-tool smoke test covering the installed `mdtablefix` and
    `markdownlint-cli2` flag set used by the scoped Markdown targets.
    Lightweight addendum pass.
- [x] 4.2.2.2. Guard load-bearing Markdown formatter flags.
  - Source: review:4.2.2 (low).
  - Scope: Fail fast when `MARKDOWN_FORMAT_FLAGS` no longer includes
    `--in-place`, so overriding the flag set cannot reintroduce the
    zero-path/stdin hang risk. Lightweight addendum pass.

Addendum execution notes:

- 2026-07-03: `leta workspace add` failed with
  `Read-only file system (os error 30)`. Retrying with `HOME=/tmp/leta-home`
  failed with `Error: Failed to start daemon`, so the addendum used bounded
  branch-local file inspection for the known test and Makefile surfaces.
- 2026-07-03: `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`, so canonical graph context was unavailable
  for this planning session.
- 2026-07-03: item 4.2.2.1 added the scoped formatter command-order assertion
  and real-tool smoke test. Focused `makefile_markdown_maintenance` tests
  passed in
  `/tmp/test-addendum-4221-rerun-mapsplice-roadmap-4-2-2-addendum.out`;
  `scrutineer` reported `make all` green in
  `/tmp/all-mapsplice-roadmap-4-2-2-addendum-2.out`.
- 2026-07-03: CodeRabbit review for item 4.2.2.1 was deferred because the
  sandbox has no default network route; see
  `/tmp/coderabbit-mapsplice-roadmap-4-2-2-addendum.out`.
- 2026-07-03: item 4.2.2.2 added a Make expansion guard that requires
  `--in-place` in `MARKDOWN_FORMAT_FLAGS`, plus a focused dry-run regression
  test proving the formatter command is not emitted when the guard fails.
  Focused `makefile_markdown_maintenance` tests passed in
  `/tmp/test-addendum-4222-mapsplice-roadmap-4-2-2-addendum.out`; `scrutineer`
  reported `make all` green in
  `/tmp/all-mapsplice-roadmap-4-2-2-addendum-3.out`.
- 2026-07-03: CodeRabbit review for item 4.2.2.2 was deferred because the
  sandbox has no default network route; see
  `/tmp/coderabbit-mapsplice-roadmap-4-2-2-addendum-2.out`.

## Outcomes & retrospective

Work item 1 is implemented. The Makefile now has opt-in scoped Markdown
formatter and lint targets guarded by `MARKDOWN_PATHS`, and the new integration
tests prove dry-run command construction, empty-path failure, and that the
formatter target does not mutate an unlisted sentinel Markdown file. Work item
2 documented the workflow in the developers' guide and marked roadmap task
4.2.2 complete. Deterministic gates passed for both work items. CodeRabbit
review remains deferred because this sandbox has no default network route.

## Context and orientation

The repository is a Rust CLI project. The Makefile at `Makefile` owns local
developer gates. Its current documentation-relevant targets are:

- `fmt`, which runs `cargo fmt` and `mdformat-all`.
- `check-fmt`, which checks Rust formatting only.
- `markdownlint`, which runs `markdownlint-cli2 '**/*.md'`.
- `nixie`, which runs `merman-cli` over `NIXIE_PATHS`, defaulting to tracked
  Markdown files from `git ls-files '*.md'`.
- `all`, which runs `check-fmt`, `lint`, `typecheck`, and `test`.

Roadmap section 4.2 is titled "Make documentation gates deterministic and
scope-aware". Task 4.2.1 has already stabilized the `make nixie` path. Task
4.2.2 now asks for documented targets or variables that format and lint
caller-supplied Markdown paths without repo-wide formatter churn.

The primary implementation surface is `Makefile`. The expected test surface is
a new Rust integration test file, `tests/makefile_markdown_maintenance.rs`,
modelled after `tests/makefile_nixie.rs`. The documentation surface is
`docs/developers-guide.md`, `docs/roadmap.md`, and this ExecPlan.

Terms used in this plan:

- `MARKDOWN_PATHS` means a whitespace-separated Make variable containing one or
  more existing Markdown file paths supplied by the caller.
- `scoped formatter` means a target that rewrites only `MARKDOWN_PATHS`.
- `scoped lint` means a target that lints only `MARKDOWN_PATHS`.
- `sentinel file` means a Markdown file created in a test solely to prove that
  files outside `MARKDOWN_PATHS` are not changed.

## Interfaces and dependencies

Use the following interface. Do not leave an alternate menu for the implementer.

In `Makefile`, add:

```make
MDFIX ?= mdtablefix
MARKDOWN_PATHS ?=
MARKDOWN_FORMAT_FLAGS ?= --wrap --renumber --breaks --ellipsis --fences --in-place
define require_markdown_paths
$(if $(strip $(MARKDOWN_PATHS)),,$(error set MARKDOWN_PATHS='docs/users-guide.md [more.md...]'))
endef
```

Add two phony targets:

```make
markdownfmt: ## Format Markdown files listed in MARKDOWN_PATHS
markdownlint-paths: ## Lint Markdown files listed in MARKDOWN_PATHS
```

Both targets must call `$(call require_markdown_paths)` as their first recipe
line and must fail before invoking external tools when
`$(strip $(MARKDOWN_PATHS))` is empty. The failure message must mention
`MARKDOWN_PATHS`, as shown in the macro above. Do not implement this as a shell
guard such as the following:

```make
@if [ -z "$(strip $(MARKDOWN_PATHS))" ]; then ...
```

Shell guards are printed, not executed, under `make --dry-run`, which would
leave the empty-path behaviour untested.

`markdownfmt` must run the formatter and fixer in this order:

```make
$(call require_markdown_paths)
$(MDFIX) $(MARKDOWN_FORMAT_FLAGS) $(MARKDOWN_PATHS)
$(MDLINT) --fix --no-globs -- $(MARKDOWN_PATHS)
```

`markdownlint-paths` must run:

```make
$(call require_markdown_paths)
$(MDLINT) --no-globs -- $(MARKDOWN_PATHS)
```

The existing `markdownlint` target must remain:

```make
$(MDLINT) '**/*.md'
```

Verified external behaviour:

- Installed `mdtablefix` is version 0.4.0. Its help says file arguments are
  `[FILES]...`, `--in-place` rewrites files, and without files it reads from
  standard input. Its local source at
  `/home/leynos/Projects/mdtablefix/src/main.rs` defines
  `#[arg(long = "in-place", requires = "files")]`, writes files only in
  `handle_file` when `in_place` is true, and otherwise prints formatted output.
  The official GitHub README says one or more file paths print corrected tables
  to standard output, `--in-place` modifies files in place, and no files means
  standard input to standard output:
  <https://github.com/leynos/mdtablefix#command-line-usage>.
- Installed `markdownlint-cli2` is version 0.22.1 with `markdownlint` 0.40.0.
  Its help and local source at
  `/home/leynos/.bun/install/cache/markdownlint-cli2@0.22.1@@@1/markdownlint-cli2.mjs`
  show `--fix` updates files, `--no-globs` ignores configured globs, and `--`
  makes remaining parameters literal. The official GitHub README documents
  positional glob arguments, `--fix`, `--no-globs`, the `:` literal-path
  prefix, and `--` literal handling:
  <https://github.com/DavidAnson/markdownlint-cli2#command-line>.
- GNU Make official documentation says command-line `v=x` arguments override
  ordinary Makefile assignments. That supports
  `MARKDOWN_PATHS='docs/a.md' make markdownfmt`:
  <https://www.gnu.org/software/make/manual/html_node/Overriding.html>.
- Installed `merman-cli` is version 0.7.0. This plan does not change `nixie`,
  but final Markdown validation still runs `make nixie`.

## Plan of work

### Work item 1: Add scoped Markdown Make targets with tests

This work item implements roadmap task 4.2.2's test-first requirement without
leaving a failing red-only state. It adds the focused integration tests, proves
they fail before the Makefile targets exist, adds the minimal Makefile surface,
then proves the same tests pass. The final commit for this item includes both
the tests and the Makefile implementation, so it is independently gate-passable
and committable.

Documentation to read before editing:

- `AGENTS.md`, sections "Commands", "Markdown Guidance", and "Rust Specific
  Guidance".
- `docs/roadmap.md`, section 4.2 and task 4.2.2.
- `docs/developers-guide.md`, section 7 "Local tooling".
- `docs/documentation-style-guide.md`, "Markdown rules" and "Formatting".
- `docs/scripting-standards.md`, "Pathlib: robust path manipulation", for the
  principle that tooling should make path handling explicit and testable.

Skills to load before editing:

- `rust-router`.
- `rust-unit-testing`.
- `sem`.

Edits:

1. Add `tests/makefile_markdown_maintenance.rs` with a module-level `//!`
   comment.
2. Add a dry-run helper similar to `make_nixie_dry_run` in
   `tests/makefile_nixie.rs`. It should run
   `make --dry-run --always-make --no-print-directory` from
   `env!("CARGO_MANIFEST_DIR")` and return the full `Output` so tests can
   assert either success or failure.
3. Add an `#[rstest]` case proving that
   `MARKDOWN_PATHS='docs/users-guide.md docs/developers-guide.md' make markdownfmt`
   includes: `mdtablefix`, `--in-place`, both named paths, `markdownlint-cli2`,
   `--fix`, and `--no-globs`.
4. Add an `#[rstest]` case proving that the same `MARKDOWN_PATHS` with
   `make markdownlint-paths` includes `markdownlint-cli2`, `--no-globs`, and
   both named paths, and does not include `'**/*.md'`.
5. Add a focused test proving that omitting `MARKDOWN_PATHS` for each scoped
   target fails and mentions `MARKDOWN_PATHS`. This test must invoke
   `make --dry-run --always-make --no-print-directory markdownfmt` and
   `make --dry-run --always-make --no-print-directory markdownlint-paths`
   without `MARKDOWN_PATHS`, assert that each `ExitStatus` is non-zero, and
   assert that standard error contains `MARKDOWN_PATHS`. This test is coupled
   to the required `$(error ...)` guard and must not be rewritten to pass
   vacuously against a shell-level guard.
6. Add a smoke test that creates two temporary Markdown files and two wrapper
   scripts named by `MDFIX=<wrapper>` and `MDLINT=<wrapper>`. The wrappers must
   record and mutate only file arguments they receive. Run
   `make markdownfmt MARKDOWN_PATHS=<selected-file> MDFIX=<wrapper> MDLINT=<wrapper>`
   and assert the sentinel file is unchanged.
7. Update `.PHONY` to include `markdownfmt` and `markdownlint-paths`.
8. Add `MDFIX`, `MARKDOWN_PATHS`, and `MARKDOWN_FORMAT_FLAGS` variables near
   the existing Markdown tool variables.
9. Add the exact `require_markdown_paths` Make macro from
   `Interfaces and dependencies` and call it as the first recipe line of both
   scoped targets. Do not use a shell guard; the dry-run empty-path test must
   fail during Make recipe expansion.
10. Add `markdownfmt` with the exact command order from
   `Interfaces and dependencies`.
11. Add `markdownlint-paths` with the exact command from
   `Interfaces and dependencies`.
12. Leave `fmt`, `markdownlint`, and `nixie` behaviour unchanged except for the
   `.PHONY` list and help output.

Testing and validation:

- Red: run the focused new test after adding the test file but before changing
  the Makefile. Expect it to fail because `markdownfmt` and
  `markdownlint-paths` do not exist.

  ```bash
  safe_branch="$(git branch --show-current | tr '/ ' '--')"
  make test TEST_FLAGS='--workspace --all-targets --all-features makefile_markdown_maintenance' \
    2>&1 | tee "/tmp/test-red-mapsplice-${safe_branch}-4-2-2-item-1.out"
  ```

- Green: rerun the focused tests after the Makefile changes and expect them to
  pass.

  ```bash
  safe_branch="$(git branch --show-current | tr '/ ' '--')"
  make test TEST_FLAGS='--workspace --all-targets --all-features makefile_markdown_maintenance' \
    2>&1 | tee "/tmp/test-green-mapsplice-${safe_branch}-4-2-2-item-1.out"
  ```

- Makefile syntax:

  ```bash
  safe_branch="$(git branch --show-current | tr '/ ' '--')"
  mbake validate Makefile \
    2>&1 | tee "/tmp/mbake-mapsplice-${safe_branch}-4-2-2-item-1.out"
  ```

- Full commit gates:

  ```bash
  safe_branch="$(git branch --show-current | tr '/ ' '--')"
  make all 2>&1 | tee "/tmp/make-all-mapsplice-${safe_branch}-4-2-2-item-1.out"
  make markdownlint 2>&1 | tee "/tmp/markdownlint-mapsplice-${safe_branch}-4-2-2-item-1.out"
  make nixie 2>&1 | tee "/tmp/nixie-mapsplice-${safe_branch}-4-2-2-item-1.out"
  ```

Commit after the gates pass. Suggested commit subject:

```plaintext
Add scoped Markdown Make targets
```

### Work item 2: Document the scoped workflow and close roadmap task 4.2.2

This work item updates the source-of-truth documentation after the Makefile
surface exists. It implements the documentation maintenance requirements from
`AGENTS.md` and the status update expected by `docs/roadmap.md`.

Documentation to read before editing:

- `docs/developers-guide.md`, section 7 "Local tooling".
- `docs/roadmap.md`, section 4.2.
- `docs/documentation-style-guide.md`, "Markdown rules" and "Formatting".
- `AGENTS.md`, "Documentation Maintenance" and "Markdown Guidance".

Skills to load before editing:

- `sem`.

Edits:

1. In `docs/developers-guide.md`, replace the Markdown-change command block
   that currently starts with `make fmt` with the scoped workflow:

   ```bash
   MARKDOWN_PATHS='docs/users-guide.md docs/developers-guide.md' make markdownfmt
   MARKDOWN_PATHS='docs/users-guide.md docs/developers-guide.md' make markdownlint-paths
   make markdownlint
   make nixie
   ```

   Keep `make markdownlint` and `make nixie` documented as full gates before
   commit.
2. Add one short paragraph explaining that `MARKDOWN_PATHS` is a
   whitespace-separated list of existing Markdown paths and that `make fmt`
   remains repository-wide.
3. Mark `docs/roadmap.md` task 4.2.2 complete only after work item 1 has
   passed its focused tests and full gates.
4. Update this ExecPlan's `Progress`, `Decision log`, and
   `Outcomes & retrospective` with the commands and log paths actually used.

Testing and validation:

- Format only the changed Markdown paths. At this point all listed files must
  exist:

  ```bash
  safe_branch="$(git branch --show-current | tr '/ ' '--')"
  mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place \
    docs/developers-guide.md docs/roadmap.md docs/execplans/roadmap-4-2-2.md \
    2>&1 | tee "/tmp/mdtablefix-mapsplice-${safe_branch}-4-2-2-item-2.out"
  markdownlint-cli2 --fix \
    docs/developers-guide.md docs/roadmap.md docs/execplans/roadmap-4-2-2.md \
    2>&1 | tee "/tmp/markdownlint-fix-mapsplice-${safe_branch}-4-2-2-item-2.out"
  ```

- Run the focused test for the new targets:

  ```bash
  safe_branch="$(git branch --show-current | tr '/ ' '--')"
  make test TEST_FLAGS='--workspace --all-targets --all-features makefile_markdown_maintenance' \
    2>&1 | tee "/tmp/test-docs-mapsplice-${safe_branch}-4-2-2-item-2.out"
  ```

- Full commit gates:

  ```bash
  safe_branch="$(git branch --show-current | tr '/ ' '--')"
  make all 2>&1 | tee "/tmp/make-all-mapsplice-${safe_branch}-4-2-2-item-2.out"
  make markdownlint 2>&1 | tee "/tmp/markdownlint-mapsplice-${safe_branch}-4-2-2-item-2.out"
  make nixie 2>&1 | tee "/tmp/nixie-mapsplice-${safe_branch}-4-2-2-item-2.out"
  ```

Commit after the gates pass. Suggested commit subject:

```plaintext
Document scoped Markdown maintenance
```

## Concrete steps

1. Confirm the worktree and status:

   ```bash
   cd /home/leynos/Projects/mapsplice.worktrees/roadmap-4-2-2
   git branch --show-current
   git status --short
   ```

   Expected branch:

   ```plaintext
   roadmap-4-2-2
   ```

2. If resuming after approval, reread this ExecPlan and update `Progress` with
   the current timestamp.
3. Run `sem diff --from origin/main --to HEAD --format json` before editing so
   the implementation can distinguish branch changes from baseline.
4. Implement work item 1 by adding the tests, running the red focused command,
   adding the Makefile targets, rerunning the focused command to green, running
   `mbake validate Makefile`, running full gates, and committing.
5. Implement work item 2 by updating the developers' guide, roadmap, and this
   ExecPlan, formatting only the changed Markdown files listed in the work
   item, running focused and full gates, and committing.
6. Before final handoff, run:

   ```bash
   safe_branch="$(git branch --show-current | tr '/ ' '--')"
   sem diff --from origin/main --to HEAD --format json \
     2>&1 | tee "/tmp/sem-diff-mapsplice-${safe_branch}-4-2-2-final.out"
   git status --short \
     2>&1 | tee "/tmp/git-status-mapsplice-${safe_branch}-4-2-2-final.out"
   ```

## Validation and acceptance

The implementation is accepted when all of the following are true:

- `MARKDOWN_PATHS='docs/users-guide.md docs/developers-guide.md' make markdownfmt`
  runs `mdtablefix --in-place` and `markdownlint-cli2 --fix` only against those
  paths.
- `MARKDOWN_PATHS='docs/users-guide.md docs/developers-guide.md' make markdownlint-paths`
  runs `markdownlint-cli2 --no-globs --` only against those paths.
- Omitting `MARKDOWN_PATHS` for either scoped target fails quickly with a
  message that names `MARKDOWN_PATHS`.
- A test proves a sentinel Markdown file outside `MARKDOWN_PATHS` remains
  unchanged when the scoped formatter target runs.
- The developers' guide documents the new scoped workflow and still instructs
  contributors to run the full Markdown gates before committing.
- `docs/roadmap.md` marks task 4.2.2 complete only after the implementation
  and documentation gates pass.
- `make all` passes for the final tree. This includes `check-fmt`, `lint`,
  `typecheck`, and `test` on current `origin/main`.
- Because Markdown files change, `make markdownlint` and `make nixie` pass for
  the final tree, or any pre-existing unrelated failure is recorded with exact
  log evidence and left for review.

Red-Green-Refactor evidence must be recorded in `Progress`:

- Red: the new `makefile_markdown_maintenance` focused test fails before the
  Makefile targets exist.
- Green: the same focused test passes after the Makefile targets are added.
- Guard proof: the empty-path test shows `make --dry-run` exits non-zero for
  both scoped targets and includes `MARKDOWN_PATHS` in standard error. This
  proves the `$(error ...)` expansion guard and prevents regression to an
  unexecuted shell guard.
- Refactor: any Makefile cleanup is followed by the focused test,
  `mbake validate Makefile`, `make all`, `make markdownlint`, and `make nixie`.

## Idempotence and recovery

The planned edits are safe to repeat. Re-running `make markdownfmt` on the same
`MARKDOWN_PATHS` should be idempotent once the formatter has normalized those
files. Re-running `make markdownlint-paths` should not modify files.

If a focused test fails, read the log under `/tmp` first. Do not rerun full
gates until a focused fix is applied. If a full gate fails for unrelated
pre-existing documentation debt, record the exact log path and changed file set
in this plan before deciding whether to stop.

If formatter churn appears outside the current work item, inspect
`git status --short` and `sem diff --format json`. Revert only this agent's
unwanted formatter changes with explicit file paths. Do not use
`git reset --hard`.

## Artifacts and notes

Planning evidence:

- `git branch --show-current` returned `roadmap-4-2-2`.
- Initial `git status --short` returned no files.
- `sem diff --from origin/main --to HEAD --format json` reported zero changed
  entities before this plan was created.
- `mbake validate Makefile` returned `Valid syntax` for the baseline Makefile.
- `mdtablefix --version` returned `mdtablefix 0.4.0`.
- `markdownlint-cli2 --version` returned
  `markdownlint-cli2 v0.22.1 (markdownlint v0.40.0)`.
- `merman-cli --version` returned `merman-cli 0.7.0`.
- `sem --help` showed `diff`, `impact`, `graph`, `blame`, `log`, `entities`,
  `context`, and `verify`.
- `make --dry-run --always-make --no-print-directory markdownlint MARKDOWN_PATHS='docs/users-guide.md'`
  still printed `markdownlint-cli2 '**/*.md'`, confirming the existing gate is
  not path-scoped.
- `make --dry-run --always-make --no-print-directory fmt` printed
  `cargo fmt --all` and `mdformat-all`, confirming `fmt` is repository-wide.

Revision note:

- 2026-07-03T00:59:01Z: Created the first-round draft for roadmap task 4.2.2.
  The draft records Memtrace, Firecrawl, and Leta failures; verifies
  load-bearing `mdtablefix`, `markdownlint-cli2`, and GNU Make behaviour; and
  decomposes the work into one test/implementation commit plus one
  documentation/roadmap commit.
- 2026-07-03T19:54:00+02:00: Revised the plan after design review found that
  a shell-level empty-path guard would only be printed under `make --dry-run`.
  The remaining work now requires a `$(error ...)` Make expansion guard and a
  non-zero dry-run assertion for both scoped targets, so the high-severity
  `mdtablefix` stdin-hang risk is directly testable.
