# Make Failed In-Place Rewrites Leave No Temporary Files

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

Roadmap task 4.4.2, "Make failed in-place rewrites leave no temporary files",
closes the audit finding that `mapsplice --in-place` can leave an orphan
temporary sibling beside the target file when the temporary write or final
rename fails. After this plan is implemented, a failed in-place rewrite still
reports the original filesystem error, still leaves the target file untouched,
and removes the generated `.<target>.mapsplice.tmp.<pid>.<nanos>.<counter>`
sibling file when cleanup is possible.

The observable success is local and concrete. Focused unit tests in `src/fs.rs`
inject a temporary-file write failure and a final rename failure. Each test
proves that `rewrite_utf8` returns the original `MapspliceError::Io` action and
that no `.mapsplice.tmp` sibling remains in the target directory. The
repository gates `make all`, `make markdownlint`, and `make nixie` pass.

## Constraints

- Work only inside
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-2`.
- Do not edit the root/control worktree.
- Treat `origin/main` as canonical integration context and
  `docs/roadmap.md` as the roadmap source of truth.
- This is the first planning round. Do not begin implementation until this
  ExecPlan is approved by the df12-build roadmap workflow.
- Implement `docs/roadmap.md` task 4.4.2 exactly: failed write or rename paths
  must remove sibling temporary files and preserve the original error.
- Preserve `docs/mapsplice-design.md` section 5, especially F5: malformed or
  failed operations must fail closed and never emit partial output.
- Preserve `docs/mapsplice-design.md` section 6, especially C6: standard output
  remains the default and `--in-place` rewrites the target atomically and only
  after the operation succeeds.
- Preserve `docs/developers-guide.md` section 2: `src/fs.rs` remains the
  capability-oriented filesystem adapter, and filesystem failures surface as
  `MapspliceError::Io`.
- Preserve `docs/users-guide.md` section "Output modes": validation failures in
  `--in-place` mode leave the target unchanged and emit no roadmap body.
- Follow `AGENTS.md` Rust guidance: modules have `//!` comments, public APIs
  have Rustdoc, Clippy warnings are denied, code files stay below 400 lines,
  and expected filesystem errors return `Result` rather than panicking.
- Follow `AGENTS.md` testing guidance: use `rstest` fixtures for shared setup,
  prefer clear failing diagnostics, avoid direct environment mutation, and keep
  tests deterministic.
- Do not add external dependencies. Use only crates already locked in
  `Cargo.lock`, especially `cap-std = 4.0.2`, `camino = 1.2.2`,
  `rstest = 0.26.1`, `serial_test = 3.2.0`, and `tempfile = 3.27.0`.
- Do not change CLI argument shape, roadmap grammar, public library signatures,
  or the temporary filename format unless this plan is revised and re-approved.
- Format only Markdown files changed by this task. Do not run repository-global
  Markdown formatters such as `make fmt` or `mdformat-all`.
- Run tests, lint, and format gates sequentially with `tee` logs under `/tmp`.
  Do not run test, lint, or format gates in parallel.
- Use the shared Cargo cache. Do not create an isolated Cargo cache.
- Do not mark this plan blocked only because Memtrace, Leta, Firecrawl, Sem, or
  another advisory tool is unavailable. Record the failed command and continue
  with bounded local docs, source, and tests.

## Tolerances

- Stop and escalate if implementation requires any public API signature change,
  CLI contract change, changed roadmap grammar, new dependency, or unsafe code.
- The planned implementation may edit only `src/fs.rs`,
  `docs/roadmap.md`, and `docs/execplans/roadmap-4-4-2.md`.
- If another file appears necessary, update this ExecPlan first and escalate for
  design review before editing that file.
- Keep `src/fs.rs` below 400 lines. It was 182 lines during planning, so the
  expected helper and tests have enough headroom.
- Stop and escalate if preserving the original write or rename error conflicts
  with removing the temporary file.
- Stop and escalate if deterministic failure injection cannot be kept private to
  `src/fs.rs` unit tests.
- Stop and escalate if the same focused test still fails after three
  implementation attempts.
- Stop and escalate if `make all` fails for an unrelated pre-existing issue
  that cannot be isolated with a focused command and a log.
- Keep each work item independently committable and gate-passable. Do not commit
  red tests; capture red evidence in the plan, then commit the passing
  implementation and tests.

## Risks

- Risk: cleanup could mask the original filesystem error with a secondary
  cleanup failure. Severity: high. Likelihood: medium. Mitigation: cleanup must
  be best-effort and ignored after logging; the returned `MapspliceError::Io`
  must be constructed from the original write or rename `io::Error`.

- Risk: removing the temporary file before dropping the file handle could fail
  on platforms that do not allow deleting open files. Severity: medium.
  Likelihood: medium. Mitigation: explicitly drop the temporary writer before
  cleanup on the write error path, and keep the existing `drop(temp)` before
  rename on the rename path.

- Risk: tests that depend on real filesystem permissions can be flaky or
  platform-specific. Severity: medium. Likelihood: medium. Mitigation: use
  private dependency injection inside `src/fs.rs` tests to inject write and
  rename errors after a real sibling temporary file has been created. Do not
  rely on chmod, disk-full conditions, or process-wide state.

- Risk: an injected test seam could make production filesystem code harder to
  follow. Severity: medium. Likelihood: low. Mitigation: keep the seam private,
  small, and named around the actual rewrite operation. Do not expose it from
  the crate or alter public API.

- Risk: ignored cleanup errors hide useful debugging information.
  Severity: low. Likelihood: medium. Mitigation: log ignored cleanup failures
  with `tracing::debug!`, including the temporary filename and original target
  path, without changing the returned error.

## Progress

- [x] (2026-07-03T03:31:29Z) Confirmed worktree
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-2` and branch
  `roadmap-4-4-2`.
- [x] (2026-07-03T03:31:29Z) Loaded required skills: `execplans`, `leta`,
  `rust-router`, `rust-errors`, `rust-unit-testing`, `rust-verification`,
  `proptest`, `sem`, and `firecrawl-mcp`.
- [x] (2026-07-03T03:31:29Z) Attempted Memtrace first. The MCP call
  `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`; bounded local evidence was used as fallback.
- [x] (2026-07-03T03:31:29Z) Attempted Leta workspace setup with
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-2`
  followed by `leta files`; it failed with
  `Error: IO error: Read-only file system (os error 30)`. Direct bounded source
  inspection and exact text search were used as fallback.
- [x] (2026-07-03T03:31:29Z) Used Sem for branch-local history and impact:
  `sem blame src/fs.rs` showed the filesystem helpers came from the initial
  tool commit, and `sem impact rewrite_utf8` showed the only direct dependent is
  `src/lib.rs::run_request`.
- [x] (2026-07-03T03:31:29Z) Attempted Firecrawl official-doc retrieval. The MCP
  call `mcp__firecrawl.firecrawl_scrape` for
  `https://docs.rs/cap-std/4.0.2/cap_std/fs_utf8/struct.Dir.html` returned
  `user cancelled MCP tool call`; locked local crate source was used as the
  load-bearing fallback evidence.
- [x] (2026-07-03T03:31:29Z) Reviewed governing docs:
  `AGENTS.md`, `docs/roadmap.md`, `docs/mapsplice-design.md`,
  `docs/developers-guide.md`, `docs/users-guide.md`,
  `docs/documentation-style-guide.md`, `docs/contributing.md`, and
  `docs/issues/audit-4.2.2.md`.
- [x] (2026-07-03T03:31:29Z) Verified branch-local source and tests around
  `src/fs.rs::rewrite_utf8`, `src/lib.rs::run_request`, `tests/roadmap_ops.rs`,
  and `tests/support/workspace.rs`.
- [x] (2026-07-03T03:31:29Z) Verified locked `cap-std = 4.0.2` source supports
  relative `Dir::open_with`, `Dir::remove_file`, `Dir::rename`, `Dir::entries`,
  and `DirEntry::file_name`. Verified locked `tempfile = 3.27.0` source
  supports automatic and explicit temporary-directory cleanup.
- [x] (2026-07-03T03:31:29Z) Drafted this first-round ExecPlan.
- [x] (2026-07-03T03:31:29Z) Formatted this ExecPlan with
  `mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place` and
  `markdownlint-cli2 --fix --no-globs --`.
- [x] (2026-07-03T03:31:29Z) Validated the planning-only change with
  `make all`, `make markdownlint`, and `make nixie`; all three passed with logs
  under `/tmp`.
- [x] (2026-07-03T04:12:00Z) Work item 1 red stage: added the injected
  write-failure test and confirmed it failed before cleanup because the
  temporary sibling remained. Evidence:
  `/tmp/cargo-test-write-cleanup-red-mapsplice-roadmap-4-4-2.out`.
- [x] (2026-07-03T04:16:00Z) Work item 1 green stage: implemented
  write-failure cleanup in `src/fs.rs`, preserving the original
  `"failed to write temporary file for"` error. The focused test passed with
  evidence in `/tmp/cargo-test-write-cleanup-mapsplice-roadmap-4-4-2.out`.
- [x] (2026-07-03T04:27:00Z) Work item 1 deterministic gates passed after
  lint-driven helper refinements. Evidence:
  `/tmp/workitem1-after-testdir-all-mapsplice-roadmap-4-4-2.out`,
  `/tmp/workitem1-after-testdir-markdownlint-mapsplice-roadmap-4-4-2.out`, and
  `/tmp/workitem1-after-testdir-nixie-mapsplice-roadmap-4-4-2.out`.
- [x] (2026-07-03T04:29:00Z) Work item 1 CodeRabbit review was attempted by
  `scrutineer` and deferred because the sandbox has no default network route.
  Evidence: `/tmp/workitem1-coderabbit-mapsplice-roadmap-4-4-2.out`.
- [x] (2026-07-03T04:43:00Z) Work item 2 red stage: added the injected
  rename-failure test and confirmed it failed before cleanup because the
  temporary sibling remained. Evidence:
  `/tmp/cargo-test-rename-cleanup-red-mapsplice-roadmap-4-4-2.out`.
- [x] (2026-07-03T04:45:00Z) Work item 2 green stage: implemented
  rename-failure cleanup in `src/fs.rs`, preserving the original
  `"failed to replace"` error. The focused rename test passed with evidence in
  `/tmp/cargo-test-rename-cleanup-mapsplice-roadmap-4-4-2.out`.
- [x] (2026-07-03T04:52:00Z) Work item 2 deterministic gates passed after
  reducing `src/fs.rs` back under the 400-line limit and removing a helper-level
  `expect`. Evidence: `/tmp/workitem2-pass-all-mapsplice-roadmap-4-4-2.out`,
  `/tmp/workitem2-pass-markdownlint-mapsplice-roadmap-4-4-2.out`, and
  `/tmp/workitem2-pass-nixie-mapsplice-roadmap-4-4-2.out`.
- [x] (2026-07-03T04:54:00Z) Work item 2 CodeRabbit review was attempted by
  `scrutineer` and deferred because the sandbox has no default network route.
  Evidence: `/tmp/workitem2-coderabbit-mapsplice-roadmap-4-4-2.out`.
- [x] (2026-07-03T05:02:00Z) Work item 3 marked roadmap task 4.4.2 complete in
  `docs/roadmap.md` and updated this ExecPlan for final validation.
- [x] (2026-07-03T05:06:00Z) Work item 3 deterministic gates passed:
  `make all`, `make markdownlint`, and `make nixie`. Evidence:
  `/tmp/workitem3-all-mapsplice-roadmap-4-4-2.out`,
  `/tmp/workitem3-markdownlint-mapsplice-roadmap-4-4-2.out`, and
  `/tmp/workitem3-nixie-mapsplice-roadmap-4-4-2.out`.
- [x] (2026-07-03T05:08:00Z) Work item 3 CodeRabbit review was attempted by
  `scrutineer` and deferred because the sandbox has no default network route.
  Evidence: `/tmp/workitem3-coderabbit-mapsplice-roadmap-4-4-2.out`.

## Surprises & discoveries

- Observation: Memtrace and Firecrawl MCP calls were both cancelled by the host
  session rather than returning repository or documentation data. Evidence: the
  exact MCP responses were `user cancelled MCP tool call`. Impact: this plan
  records the failures and relies on bounded local docs, source, Sem output,
  and locked crate source as fallback evidence.

- Observation: Leta could not add the worktree because its workspace storage hit
  a read-only filesystem error. Evidence:
  `leta workspace add ... && leta files` returned
  `Error: IO error: Read-only file system (os error 30)`. Impact: branch-local
  verification used bounded file inspection and exact text search. This is not
  a product blocker.

- Observation: `rewrite_utf8` has a narrow blast radius.
  Evidence: `sem impact rewrite_utf8` reported only one direct dependent,
  `src/lib.rs::run_request`. Impact: the implementation should stay contained in
  `src/fs.rs` with no public API or CLI changes.

- Observation: the planned private strategy helper initially tripped Clippy's
  argument-count lint. Impact: the two injected operations are now grouped in a
  private `RewriteStrategy` value, keeping the test seam private while
  satisfying the lint.

- Observation: repository lints apply capability-filesystem and panic-boundary
  rules to helpers inside `#[cfg(test)]` modules when the helper itself is not a
  `#[test]` function. Impact: work item 1 test helpers use `cap_std::fs_utf8`
  for target seeding and return `io::Result` where fallible setup or directory
  inspection is needed.

- Observation: CodeRabbit could not run in this sandbox. Evidence: the review
  agent returned phase `deferred` with status
  `deferred coderabbit review: no default network route visible in this sandbox`.
  Impact: no actionable AI review findings exist for work item 1, and the
  deferred review remains an open external-tooling issue. The same deferred
  status occurred again for work items 2 and 3.

- Observation: adding the rename-failure test pushed `src/fs.rs` to 412 lines,
  exceeding the repository's 400-line cap. Impact: helper duplication was
  reduced inside `src/fs.rs`, bringing the file to 399 lines without adding a
  new module or changing public APIs.

## Decision log

- Decision: implement cleanup in `src/fs.rs::rewrite_utf8` rather than adding a
  new dependency or replacing the rewrite path with an external atomic-write
  crate. Rationale: the current code already creates a unique sibling temp file
  and uses `cap_std::fs_utf8::Dir::rename`; the defect is missing cleanup on
  two error branches. A new dependency would violate the narrow remediation
  scope. Date/Author: 2026-07-03T03:31:29Z / planning agent.

- Decision: use a private helper with injected open and rename closures for
  tests. Rationale: real OS write and rename failures are difficult to force
  deterministically after a temp file exists. A private helper lets tests
  create a real temp sibling, inject the failing operation, and assert cleanup
  without exposing test seams through the public API. Date/Author:
  2026-07-03T03:31:29Z / planning agent.

- Decision: cleanup failures are ignored after debug logging.
  Rationale: roadmap task 4.4.2 explicitly requires preserving the original
  failure. Returning the cleanup error would hide the write or rename failure
  that caused the operation to fail. Date/Author: 2026-07-03T03:31:29Z /
  planning agent.

- Decision: no property, snapshot, or end-to-end test is required for this
  narrow filesystem adapter bug. Rationale: the required property is a concrete
  cleanup invariant on two I/O failure branches. Focused unit tests with
  injected failures prove it more deterministically than generated inputs or
  full CLI subprocess tests. Date/Author: 2026-07-03T03:31:29Z / planning agent.

- Decision: keep the injected open and replace operations grouped in a private
  `RewriteStrategy` struct rather than adding a public test seam or lint
  suppression. Rationale: this preserves the public filesystem API, keeps the
  helper under the argument-count limit, and allows work item 2 to reuse the
  same private strategy for rename-failure injection. Date/Author:
  2026-07-03T04:16:00Z / implementation agent.

- Decision: clean up the temporary sibling in the rename error branch with the
  same best-effort helper used for write failures. Rationale: this preserves
  the original `"failed to replace"` error while satisfying the roadmap
  invariant that failed in-place rewrites do not leave generated sibling temp
  files behind. Date/Author: 2026-07-03T04:45:00Z / implementation agent.

## Outcomes & retrospective

Roadmap task 4.4.2 is complete. Failed temporary-file writes now drop the
temporary writer, best-effort remove the generated sibling temp file, and
return the original `MapspliceError::Io` action. Failed final renames now use
the same best-effort cleanup before returning the original
`"failed to replace"` error. Focused unit tests and repository gates prove both
failure branches. `docs/roadmap.md` now marks 4.4.2 complete. CodeRabbit review
for all three work items is deferred because this sandbox has no default
network route.

## Context and orientation

`mapsplice` reads a roadmap, parses it into a typed model, applies exactly one
structural operation, renders the model, then either returns the rendered text
for standard output mode or rewrites the target file in-place. The in-place
call path is `src/lib.rs::run_request`, which calls `src/fs.rs::rewrite_utf8`
only after target reading, fragment loading, operation application, and
rendering succeed.

The relevant filesystem code is in `src/fs.rs`. `rewrite_utf8` opens the target
file's parent directory as a `cap_std::fs_utf8::Dir`, builds a unique temporary
sibling filename with `temp_file_name`, creates the sibling using
`Dir::open_with` and `OpenOptions::create_new(true)`, writes the rendered
bytes, drops the file handle, and renames the temporary sibling over the target
with `Dir::rename`.

The current failure gap is narrow. If `write_all` fails after the temporary
file has been created, or if `rename` fails after the temporary file has been
written, the function returns `MapspliceError::Io` without removing the
temporary sibling. The target file is still protected from partial replacement,
but the working directory is left dirty.

The audit finding that created roadmap task 4.4.2 is
`docs/issues/audit-4.2.2.md` finding 5. It names `src/fs.rs:32-61`, describes
the orphan temporary filename shape, and proposes using
`cap.dir.remove_file(&temp_name)` on write and rename errors while ignoring a
secondary cleanup failure.

## Plan of work

### Work item 1: Clean up temporary siblings on write failure

This work item implements the write-failure half of roadmap task 4.4.2 in one
gate-passable commit.

Read these docs before editing:

- `docs/roadmap.md` task 4.4.2.
- `docs/issues/audit-4.2.2.md` finding 5.
- `docs/mapsplice-design.md` sections 5 and 6, especially F5 and C6.
- `docs/developers-guide.md` sections 2, 6, and 7.
- `docs/users-guide.md` section "Output modes".
- `AGENTS.md` Rust Specific Guidance, Testing, and Error Handling.

Load these skills before editing:

- `leta` for branch-local navigation. If it still fails, record the exact error
  and use bounded file inspection.
- `rust-router`, then `rust-errors` for preserving the original error and
  `rust-unit-testing` for deterministic injected-failure tests.
- `rust-verification` only to confirm that property or model-checking tools are
  not the smallest useful verification layer here.
- `sem` before reviewing the final diff.

Implementation:

1. Add a private helper under `src/fs.rs::rewrite_utf8` that owns the temp-file
   lifecycle and accepts narrow test-injection closures for opening the temp
   writer and replacing the target. Keep `rewrite_utf8` as the only public
   filesystem entry point and have it call the helper with real
   `Dir::open_with` and `Dir::rename`.
2. Add a private cleanup helper, for example `discard_temporary_file`, that
   calls `cap.dir.remove_file(temp_name)` and ignores any secondary failure
   after logging it with `tracing::debug!`.
3. On the `write_all` error path, drop the temporary writer, call the cleanup
   helper, and return the original `MapspliceError::Io` with action
   `"failed to write temporary file for"`.
4. Do not change rename-failure behaviour in this work item except as needed to
   route through the new private helper. Work item 2 covers that branch.

Tests:

- Unit: in `src/fs.rs` tests, add a deterministic write-failure test. It should
  create a real temporary workspace with `tempfile::tempdir()`, write an
  initial target, open the parent directory with `open_parent_dir`, pass a
  fixed temp name such as `.target.md.mapsplice.tmp.write-failure-test`, use an
  injected opener that creates that temp sibling and then returns a `Write`
  implementation whose `write` method returns an injected
  `io::ErrorKind::Other`, then assert:
  - the returned error is `MapspliceError::Io` with action
    `"failed to write temporary file for"`;
  - the target file still contains the original bytes;
  - `Dir::entries()` contains no filename with `.mapsplice.tmp`.
- Behavioural: no new BDD scenario in this work item. The failure is injected
  below the CLI boundary and the CLI has no deterministic way to force a
  post-create write failure without widening public interfaces.
- Property: none. This is not a generated-input invariant.
- Snapshot: none. No rendered diagnostic snapshot is changed.
- End-to-end: none beyond `make all`, because the unit test is the deterministic
  reproducer for the failure branch.

Validation for this work item:

```bash
cargo test fs::tests::write_failure_removes_temporary_sibling 2>&1 | tee /tmp/cargo-test-write-cleanup-mapsplice-roadmap-4-4-2.out
make all 2>&1 | tee /tmp/make-all-write-cleanup-mapsplice-roadmap-4-4-2.out
make markdownlint 2>&1 | tee /tmp/make-markdownlint-write-cleanup-mapsplice-roadmap-4-4-2.out
make nixie 2>&1 | tee /tmp/make-nixie-write-cleanup-mapsplice-roadmap-4-4-2.out
```

Expected red evidence before implementation: the focused test fails because the
temporary sibling remains after the injected write failure. Expected green
evidence after implementation: the focused test passes and `make all`,
`make markdownlint`, and `make nixie` pass.

### Work item 2: Clean up temporary siblings on rename failure

This work item implements the rename-failure half of roadmap task 4.4.2 in one
gate-passable commit.

Read these docs before editing:

- `docs/roadmap.md` task 4.4.2.
- `docs/issues/audit-4.2.2.md` finding 5.
- `docs/mapsplice-design.md` sections 5 and 6, especially F5 and C6.
- `docs/developers-guide.md` section 2 for filesystem-adapter boundaries.
- `AGENTS.md` Error Handling and Rust Specific Guidance.

Load these skills before editing:

- `leta` for `rewrite_utf8` references and branch-local verification, with the
  same bounded-inspection fallback if unavailable.
- `rust-router`, then `rust-errors` and `rust-unit-testing`.
- `sem` before reviewing the final diff.

Implementation:

1. Extend the helper introduced in work item 1 so the final rename error path
   calls the same cleanup helper before returning.
2. Preserve the original `MapspliceError::Io` action `"failed to replace"` and
   original rename `io::Error`.
3. Keep `drop(temp)` before `Dir::rename`, and keep cleanup after the failed
   rename. This avoids removing an open file handle and matches the locked
   `cap-std` path-relative API.

Tests:

- Unit: in `src/fs.rs` tests, add a deterministic rename-failure test. It should
  create a temporary workspace, write an initial target, use the real temp-file
  opener, inject a rename closure that returns an
  `io::ErrorKind::PermissionDenied`, then assert:
  - the returned error is `MapspliceError::Io` with action
    `"failed to replace"`;
  - the target file still contains the original bytes;
  - the rendered replacement bytes are not present in the target;
  - `Dir::entries()` contains no filename with `.mapsplice.tmp`.
- Behavioural: no new BDD scenario, for the same deterministic-injection reason
  as work item 1.
- Property: none.
- Snapshot: none.
- End-to-end: none beyond `make all`.

Validation for this work item:

```bash
cargo test fs::tests::rename_failure_removes_temporary_sibling 2>&1 | tee /tmp/cargo-test-rename-cleanup-mapsplice-roadmap-4-4-2.out
make all 2>&1 | tee /tmp/make-all-rename-cleanup-mapsplice-roadmap-4-4-2.out
make markdownlint 2>&1 | tee /tmp/make-markdownlint-rename-cleanup-mapsplice-roadmap-4-4-2.out
make nixie 2>&1 | tee /tmp/make-nixie-rename-cleanup-mapsplice-roadmap-4-4-2.out
```

Expected red evidence before implementation: the focused test fails because the
temporary sibling remains after the injected rename failure. Expected green
evidence after implementation: the focused test passes and `make all`,
`make markdownlint`, and `make nixie` pass.

### Work item 3: Record roadmap completion and final validation

This work item updates living documentation after the code behaviour is proven.
It is a documentation-only, gate-passable commit.

Read these docs before editing:

- `docs/roadmap.md` task 4.4.2.
- `docs/documentation-style-guide.md` sections "Spelling", "Markdown rules",
  and "Formatting".
- `docs/contributing.md` section "Development gates".
- `AGENTS.md` Documentation Maintenance and Markdown Guidance.

Load these skills before editing:

- `execplans` to keep this plan's living sections current.
- `sem` to review the entity-level diff before commit.

Implementation:

1. Update `docs/roadmap.md` to mark task 4.4.2 complete only after work items 1
   and 2 pass their gates.
2. Update this ExecPlan's `Progress`, `Surprises & Discoveries`,
   `Decision Log`, and `Outcomes & Retrospective` with the implementation and
   validation evidence.
3. Format only changed Markdown paths. If both files are edited, use the exact
   path-safe command shown below. If only this ExecPlan changed, omit
   `docs/roadmap.md` from `MARKDOWN_PATHS`.

Tests:

- Unit: no new unit tests in this documentation-only work item.
- Behavioural: no new BDD scenario.
- Property: none.
- Snapshot: none.
- End-to-end: repository gates only.

Validation for this work item:

```bash
MARKDOWN_PATHS='docs/roadmap.md docs/execplans/roadmap-4-4-2.md' make markdownfmt 2>&1 | tee /tmp/markdownfmt-docs-mapsplice-roadmap-4-4-2.out
make all 2>&1 | tee /tmp/make-all-docs-mapsplice-roadmap-4-4-2.out
make markdownlint 2>&1 | tee /tmp/make-markdownlint-docs-mapsplice-roadmap-4-4-2.out
make nixie 2>&1 | tee /tmp/make-nixie-docs-mapsplice-roadmap-4-4-2.out
```

Expected evidence: `make all`, `make markdownlint`, and `make nixie` pass, and
the Markdown formatter changes only `docs/roadmap.md` and
`docs/execplans/roadmap-4-4-2.md`.

## Concrete steps

All commands run from `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-2`.

1. Confirm the branch and working tree:

   ```bash
   git branch --show-current
   git status --short
   ```

   Expected output includes `roadmap-4-4-2`, and `git status --short` is empty
   or contains only files already explained in this plan.

2. Attempt advisory tools and record exact failures without blocking:

   ```bash
   leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-2
   leta refs rewrite_utf8
   sem impact rewrite_utf8
   ```

   Expected output from Sem shows `src/lib.rs::run_request` as the direct
   dependent. If Leta fails with the read-only filesystem error seen during
   planning, record it in `Surprises & Discoveries` and continue with bounded
   source inspection.

3. Work item 1 red stage: add the write-failure unit test and run the focused
   command:

   ```bash
   cargo test fs::tests::write_failure_removes_temporary_sibling 2>&1 | tee /tmp/cargo-test-write-cleanup-red-mapsplice-roadmap-4-4-2.out
   ```

   Expected failure before production changes: the assertion that no
   `.mapsplice.tmp` sibling remains fails.

4. Work item 1 green stage: implement write-failure cleanup in `src/fs.rs`, run
   the focused test, then run the work-item gates listed in work item 1.

5. Commit work item 1 only after all gates pass:

   ```bash
   git add src/fs.rs docs/execplans/roadmap-4-4-2.md
   git commit
   ```

   Use an imperative subject such as `Clean up temp files on write failure`.

6. Work item 2 red stage: add the rename-failure unit test and run the focused
   command:

   ```bash
   cargo test fs::tests::rename_failure_removes_temporary_sibling 2>&1 | tee /tmp/cargo-test-rename-cleanup-red-mapsplice-roadmap-4-4-2.out
   ```

   Expected failure before production changes: the assertion that no
   `.mapsplice.tmp` sibling remains fails.

7. Work item 2 green stage: implement rename-failure cleanup, run the focused
   test, then run the work-item gates listed in work item 2.

8. Commit work item 2 only after all gates pass:

   ```bash
   git add src/fs.rs docs/execplans/roadmap-4-4-2.md
   git commit
   ```

   Use an imperative subject such as `Clean up temp files on rename failure`.

9. Work item 3: update `docs/roadmap.md` and this ExecPlan, run the path-safe
   Markdown formatter and final repository gates, then commit:

   ```bash
   MARKDOWN_PATHS='docs/roadmap.md docs/execplans/roadmap-4-4-2.md' make markdownfmt 2>&1 | tee /tmp/markdownfmt-final-mapsplice-roadmap-4-4-2.out
   make all 2>&1 | tee /tmp/make-all-final-mapsplice-roadmap-4-4-2.out
   make markdownlint 2>&1 | tee /tmp/make-markdownlint-final-mapsplice-roadmap-4-4-2.out
   make nixie 2>&1 | tee /tmp/make-nixie-final-mapsplice-roadmap-4-4-2.out
   git add docs/roadmap.md docs/execplans/roadmap-4-4-2.md
   git commit
   ```

   Use an imperative subject such as `Record 4.4.2 completion`.

## Validation and acceptance

Acceptance criteria:

- Injected write failure leaves no `.mapsplice.tmp` sibling and returns the
  original `"failed to write temporary file for"` I/O error.
- Injected rename failure leaves no `.mapsplice.tmp` sibling and returns the
  original `"failed to replace"` I/O error.
- The target file remains byte-identical to its original contents after both
  injected failures.
- `src/lib.rs::run_request` still calls `rewrite_utf8` only after the roadmap
  operation and render succeed.
- `docs/roadmap.md` marks 4.4.2 complete only after the behaviour is proven.
- Final gates pass:

  ```bash
  make all 2>&1 | tee /tmp/make-all-final-mapsplice-roadmap-4-4-2.out
  make markdownlint 2>&1 | tee /tmp/make-markdownlint-final-mapsplice-roadmap-4-4-2.out
  make nixie 2>&1 | tee /tmp/make-nixie-final-mapsplice-roadmap-4-4-2.out
  ```

Red-Green-Refactor evidence to record in `Progress`:

- Red: focused write-failure test fails because the temp sibling remains.
- Green: focused write-failure test passes after write cleanup.
- Red: focused rename-failure test fails because the temp sibling remains.
- Green: focused rename-failure test passes after rename cleanup.
- Refactor: any helper naming or structure cleanup keeps both focused tests and
  `make all` passing.

## Idempotence and recovery

The implementation is idempotent. Re-running the focused tests creates fresh
temporary workspaces and fixed temp names inside those workspaces, then removes
them before the workspace is dropped. No test should depend on the process
environment, current working directory mutation, fixed global paths, or disk
permission changes.

If a gate fails, read the corresponding `/tmp` log before re-running it. Apply
the smallest fix, update this ExecPlan's living sections, and re-run the failed
gate followed by the relevant wider gates. Do not use a bare `git stash`. If a
stash becomes unavoidable, name it with the required deterministic form:
`df12-stash v1 task=4.4.2 kind=<discard|park|keep> reason="<short>"`.

Rollback is straightforward before committing: revert only the files changed by
the current work item. Do not revert unrelated user changes. After a commit,
use a new corrective commit rather than rewriting history unless the workflow
explicitly directs otherwise.

## Artifacts and notes

Planning evidence:

```plaintext
$ git branch --show-current
roadmap-4-4-2

$ mcp__memtrace.list_indexed_repositories
user cancelled MCP tool call

$ leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-2
Error: IO error: Read-only file system (os error 30)

$ mcp__firecrawl.firecrawl_scrape https://docs.rs/cap-std/4.0.2/cap_std/fs_utf8/struct.Dir.html
user cancelled MCP tool call

$ sem impact rewrite_utf8
rewrite_utf8 depends on open_parent_dir and temp_file_name; direct dependent is
src/lib.rs::run_request.
```

Branch-local source evidence:

- `src/fs.rs:32-61` creates the temporary sibling, writes bytes, drops the file,
  and renames it over the target, but currently returns on write or rename
  errors without cleanup.
- `src/fs.rs:128-144` builds temp names in the form
  `.{file_name}.mapsplice.tmp.<pid>.<nanos>.<counter>`.
- `src/lib.rs:172-178` calls `rewrite_utf8` only after apply and render succeed,
  then records in-place metrics.
- `tests/roadmap_ops.rs:338-360` already covers successful in-place CLI output.

## Interfaces and dependencies

No public interface changes are planned.

Internal helper shape in `src/fs.rs` should remain private. A concrete shape
the implementer can use is:

```rust
fn rewrite_utf8_with_strategy<W, OpenTemp, ReplaceTarget>(
    cap: &FileCap,
    temp_name: &str,
    contents: &str,
    open_temp: OpenTemp,
    replace_target: ReplaceTarget,
) -> Result<()>
where
    W: Write,
    OpenTemp: FnOnce(&Dir, &str, &OpenOptions) -> io::Result<W>,
    ReplaceTarget: FnOnce(&Dir, &str, &str) -> io::Result<()>,
```

This is not a public API. It exists only to keep the production filesystem path
small and the two failure branches testable. If Clippy suggests a clearer
private shape with fewer type parameters, prefer the clearer shape while
keeping the same observable behaviour.

Locked dependency evidence:

- `cap-std = 4.0.2` is the direct dependency selected for `mapsplice` in
  `Cargo.toml` and `Cargo.lock`.
- In locked source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cap-std-4.0.2/src/fs_utf8/dir.rs`,
  `Dir::open_with` accepts paths relative to the capability directory (lines
  75-90), `Dir::remove_file` removes a relative file path (lines 311-319), and
  `Dir::rename` renames relative paths between capability directories,
  replacing the target if it exists (lines 321-335).
- The same locked source exposes `Dir::entries` and `Dir::read_dir` for
  directory iteration (lines 212-225), and `src/fs_utf8/dir_entry.rs` exposes
  `DirEntry::file_name` as a UTF-8 string (lines 88-91). These APIs support
  deterministic temp-sibling assertions.
- `tempfile = 3.27.0` is already a dev-dependency. In locked source
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tempfile-3.27.0/src/dir/mod.rs`,
  `tempdir()` returns `TempDir` (lines 64-66), `TempDir::new` creates a
  directory removed when destroyed (lines 187-220), `TempDir::close` reports
  explicit cleanup failure (lines 470-480), and `Drop` removes the directory
  best-effort (lines 498-503).

Official docs note:

- Firecrawl official-doc retrieval for the `cap-std` docs.rs page was attempted
  and cancelled by the host session. Because the locked crate source contains
  the rustdoc comments and implementations for the exact version used by this
  repository, this plan pins load-bearing API claims to that source instead of
  relying on unstated external behaviour.

## Revision note

Initial first-round ExecPlan for roadmap task 4.4.2. It records advisory tool
failures, pins the affected filesystem APIs to locked crate source, and
decomposes implementation into two focused cleanup commits plus one final
documentation commit. The draft was formatted and validated with `make all`,
`make markdownlint`, and `make nixie`.
