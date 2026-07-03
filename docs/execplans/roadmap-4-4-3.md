# Document configuration discovery truthfully

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

Roadmap task 4.4.3, "Document configuration discovery truthfully", is complete
when `docs/users-guide.md` and `docs/developers-guide.md` describe the
configuration paths and precedence that `mapsplice` actually implements. A
reader should not need to inspect `src/cli.rs`, `src/cli_config.rs`, or locked
`ortho_config` source to know which values can come from a local file, an XDG
file, a home dotfile, an environment variable, or a command-line flag.

The observable result is precise. Tests in `tests/roadmap_config.rs` pin the
global `in_place` and insert `after` precedence claims that the guides will
state. The users' guide names the supported settings, TOML keys, file
locations, environment variables, defaults, and precedence. The developers'
guide explains that global `in_place` is loaded by mapsplice's custom global
loader, while insert `after` is loaded through
`ortho_config::load_and_merge_subcommand_for`. The roadmap item is marked done
only after the tests and documentation pass `make all`, `make markdownlint`, and
`make nixie`.

This is planning round 3. Do not begin implementation until this ExecPlan is
approved. Round 3 resolves the design-review finding that the previous insert
test matrix asserted a user-facing `~/.mapsplice.toml` discovery claim without
one positive test where the home dotfile is the only populated configuration
source.

## Constraints

- Work only inside
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-3`.
- Do not edit the root/control worktree.
- Treat `origin/main` as the integration branch and canonical baseline.
- Treat `docs/roadmap.md` as the roadmap source of truth.
- Implement only roadmap task 4.4.3 from `docs/roadmap.md` section 4.4:
  reconcile the guides with implemented global and subcommand default-loading
  paths, including local versus XDG precedence and supported environment
  variables.
- Follow `docs/mapsplice-design.md` "Status and scope" and section 2,
  "Non-negotiable constraints": CLI plumbing is covered where it bears on
  guarantees, `docs/users-guide.md` is the source of truth for command
  semantics, and the CLI is composed through `ortho-config`.
- Follow `docs/documentation-style-guide.md` sections "Spelling",
  "Markdown rules", "Formatting", "User's guide", and "Developer's guide".
- Follow `docs/developers-guide.md` section 4, "Configuration behaviour", and
  section 6, "Verification layers".
- Follow `AGENTS.md` "Change Quality & Committing", "Rust Specific Guidance",
  "Testing", and "Markdown Guidance".
- Format only Markdown files changed by each work item. For direct Markdown
  formatter commands, every path listed below exists at the start of the work
  item.
- Run gates sequentially and capture each gate with `tee` under `/tmp`.
- Use the shared Cargo cache. Do not create an isolated Cargo cache.
- Do not add dependencies or change CLI syntax, public APIs, roadmap grammar,
  configuration semantics, or `ortho_config` feature flags.
- Do not mark this plan blocked solely because Memtrace, Leta, Firecrawl,
  `sem`, CodeRabbit, or another advisory tool is unavailable. Record the exact
  failed command and continue with bounded local evidence.

## Tolerances

- Stop and escalate if satisfying the task requires a public API signature
  change, CLI argument change, configuration semantic change, new dependency,
  unsafe production code, or changed roadmap grammar.
- Planned code changes are limited to `tests/support/config.rs` and
  `tests/roadmap_config.rs`. Planned documentation changes are limited to
  `docs/users-guide.md`, `docs/developers-guide.md`, `docs/roadmap.md`, and
  this ExecPlan.
- If a production source file must change, update this plan first and escalate
  before editing that file.
- Keep each work item independently committable and gate-passable. Do not
  commit a red test; record red evidence in this plan, then commit the passing
  test or documentation update.
- Stop and escalate if the same focused test still fails after three
  implementation attempts.
- Stop and escalate if the documentation cannot state a single unambiguous
  precedence order for either global `in_place` or insert `after`.
- Stop and escalate if `make all` fails for an unrelated pre-existing issue
  that cannot be isolated with a focused command and a log.

## Risks

- Risk: tests that need an environment variable plus the current working
  directory, or two environment variables, will deadlock if they compose the
  current `EnvVarGuard::set` and `CwdGuard::enter` helpers. Severity: high.
  Likelihood: certain. Mitigation: work item 1 replaces the config test support
  surface with one process-state guard that acquires `ENV_LOCK` exactly once
  and then mutates any required environment variables and the current directory
  under that one guard.

- Risk: the global and subcommand loaders have different file discovery
  behaviour, and a generic "mapsplice configuration" section could falsely
  imply they are identical. Severity: high. Likelihood: high. Mitigation:
  document global `in_place` and insert `after` separately, and pin each
  precedence claim with focused tests before updating prose.

- Risk: `ortho_config` generic README wording may not exactly match
  mapsplice's use of `load_and_merge_subcommand_for`. Severity: medium.
  Likelihood: medium. Mitigation: rely on the locked `ortho_config = 0.8.0`
  source and rustdoc comments for load-bearing behaviour.

- Risk: documentation formatting could churn unrelated files. Severity:
  medium. Likelihood: low. Mitigation: run `mdtablefix` and
  `markdownlint-cli2 --fix` only on explicitly listed changed files, then run
  repository gates.

## Progress

- [x] (2026-07-03T00:00:00Z) Confirmed branch `roadmap-4-4-3`.
- [x] (2026-07-03T00:00:00Z) Drafted and committed the round-1 ExecPlan.
- [x] (2026-07-03T00:00:00Z) Loaded required skills for this revision:
  `execplans`, `leta`, `memtrace-first`, `firecrawl-mcp`, `rust-router`, and
  `rust-unit-testing`.
- [x] (2026-07-03T00:00:00Z) Retried Memtrace first. The MCP call
  `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`; bounded local docs, source, tests, `leta`,
  `sem`, and locked crate source were used as fallback evidence.
- [x] (2026-07-03T00:00:00Z) Retried Leta.
      `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-3`
      reported the
      workspace was already added. `leta grep` found `EnvVarGuard`, `CwdGuard`,
      `InsertConfig`, `load_global_config`, and `load_merged_config`;
      `leta refs` for `EnvVarGuard` and `CwdGuard` failed with
      `Error: Failed to start daemon`, so direct references were verified with
      bounded branch-local file inspection.
- [x] (2026-07-03T00:00:00Z) Retried Leta during round 3. The command
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-3`
  failed with `Error: IO error: Read-only file system (os error 30)`.
- [x] (2026-07-03T00:00:00Z) Used `sem entities` for `tests/support/config.rs`,
  `src/cli_config.rs`, and `src/cli.rs` orientation.
- [x] (2026-07-03T00:00:00Z) Attempted Firecrawl official-doc retrieval for
  `https://docs.rs/ortho_config/0.8.0/ortho_config/subcommand/fn.load_and_merge_subcommand_for.html`.
  The MCP call returned `user cancelled MCP tool call`; locked crate source
  and rustdoc comments from the published registry source were used as fallback
  evidence.
- [x] (2026-07-03T00:00:00Z) Reviewed governing docs:
  `AGENTS.md`, `docs/roadmap.md`, `docs/mapsplice-design.md`,
  `docs/developers-guide.md`, `docs/users-guide.md`, and
  `docs/documentation-style-guide.md`.
- [x] (2026-07-03T00:00:00Z) Verified the design-review deadlock report against
  `tests/support/config.rs`: `ENV_LOCK` is a non-reentrant static mutex at line
  37; `EnvVarGuard::set` locks it at line 74 and holds it for the guard
  lifetime; `CwdGuard::enter` locks it at line 110 and holds it for the guard
  lifetime.
- [x] (2026-07-03T00:00:00Z) Revised this ExecPlan around a single combined
  process-state guard and removed the previous unimplementable guard
  composition from the test work items.
- [x] (2026-07-03T00:00:00Z) Revised work item 1 to require
  `ProcessStateGuard::remove_env`, so tests can pin absence of inherited
  environment sources.
- [x] (2026-07-03T00:00:00Z) Revised work item 3 to add
  `insert_after_home_dotfile_default`, a positive home-dotfile-only discovery
  test for insert `after`.
- [x] (2026-07-03T06:46:31Z) Automated workflow supplied the approved
  execution request for this ExecPlan, so implementation began.
- [x] (2026-07-03T06:46:31Z) Execution resumed in the assigned worktree. The
  Memtrace MCP call `mcp__memtrace.list_indexed_repositories` again returned
  `user cancelled MCP tool call`; implementation continued with bounded
  branch-local evidence as authorised by this plan.
- [x] (2026-07-03T06:46:31Z) Retried Leta for this execution round. The command
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-3`
  failed with `Error: IO error: Read-only file system (os error 30)`; bounded
  branch-local source inspection was used for work item 1.
- [x] (2026-07-03T06:46:31Z) Work item 1 implementation replaced
  `EnvVarGuard` and `CwdGuard` with `ProcessStateGuard`, migrated existing
  config tests, added `Workspace::write_home_config`, and added
  `process_state_guard_allows_multiple_env_vars_and_cwd`.
- [x] (2026-07-03T06:46:31Z) Work item 1 focused validation passed:
  `cargo test --test roadmap_config process_state_guard -- --nocapture`
  captured at `/tmp/workitem1-roadmap-config-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T06:49:30Z) Work item 1 deterministic gates passed after
  fixing Clippy feedback from the first `make all` attempt: `make all` at
  `/tmp/workitem1-all-mapsplice-roadmap-4-4-3.out`, `make markdownlint` at
  `/tmp/workitem1-markdownlint-mapsplice-roadmap-4-4-3.out`, and `make nixie` at
  `/tmp/workitem1-nixie-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T06:49:30Z) Work item 1 CodeRabbit review was attempted via
  `/home/leynos/Projects/mapsplice.workshop/df12-build-20260629T235232Z-879541/bin/coderabbit-review-agent`.
  The command returned a deferred status with
  `deferred coderabbit review: no default network route visible in this sandbox`;
  the output is captured at
  `/tmp/workitem1-coderabbit-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T06:49:30Z) Reviewed the work item 1 semantic diff with
  `sem diff --file-exts .rs .md`; the diff is limited to
  `tests/support/config.rs`, `tests/roadmap_config.rs`, and this ExecPlan.
- [x] (2026-07-03T06:52:12Z) Work item 2 added global `in_place` precedence
  tests for local-over-XDG and environment-false-over-local-true cases.
- [x] (2026-07-03T06:52:12Z) Work item 2 red evidence was captured by
  temporarily inverting the `in_place_env_false_overrides_local_config_true`
  stdout assertion. The focused command failed at the intended assertion and is
  logged at `/tmp/workitem2-red-roadmap-config-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T06:52:12Z) Work item 2 focused validation passed:
  `cargo test --test roadmap_config in_place -- --nocapture` captured at
  `/tmp/workitem2-roadmap-config-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T06:53:20Z) Work item 2 deterministic gates passed:
  `make all` at `/tmp/workitem2-all-mapsplice-roadmap-4-4-3.out`,
  `make markdownlint` at
  `/tmp/workitem2-markdownlint-mapsplice-roadmap-4-4-3.out`, and `make nixie` at
  `/tmp/workitem2-nixie-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T06:53:20Z) Work item 2 CodeRabbit review was attempted via
  the review-agent wrapper. It returned
  `deferred coderabbit review: no default network route visible in this sandbox`;
  the output is captured at
  `/tmp/workitem2-coderabbit-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T06:53:20Z) Reviewed the work item 2 semantic diff with
  `sem diff --file-exts .rs .md`; the diff is limited to two
  `tests/roadmap_config.rs` tests and this ExecPlan.
- [x] (2026-07-03T06:56:36Z) Work item 3 added insert `after` tests for
  home-dotfile-only discovery, local-over-XDG, XDG-over-home,
  environment-false-over-local-true, and CLI-over-environment-false cases.
- [x] (2026-07-03T06:56:36Z) Work item 3 red evidence was captured by
  temporarily replacing the `insert_after_home_dotfile_default` expected
  inserted task number with `9.9.9`. The focused command failed at the intended
  assertion and is logged at
  `/tmp/workitem3-red-roadmap-config-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T06:56:36Z) Work item 3 focused validation passed:
  `cargo test --test roadmap_config insert_after -- --nocapture` captured at
  `/tmp/workitem3-roadmap-config-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T06:57:57Z) Work item 3 deterministic gates passed:
  `make all` at `/tmp/workitem3-all-mapsplice-roadmap-4-4-3.out`,
  `make markdownlint` at
  `/tmp/workitem3-markdownlint-mapsplice-roadmap-4-4-3.out`, and `make nixie` at
  `/tmp/workitem3-nixie-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T06:57:57Z) Work item 3 CodeRabbit review was attempted via
  the review-agent wrapper. It returned
  `deferred coderabbit review: no default network route visible in this sandbox`;
  the output is captured at
  `/tmp/workitem3-coderabbit-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T06:57:57Z) Reviewed the work item 3 semantic diff with
  `sem diff --file-exts .rs .md`; the diff is limited to insert configuration
  tests, two local test helpers, and this ExecPlan.
- [x] (2026-07-03T06:59:29Z) Work item 4 updated
  `docs/users-guide.md` to describe supported configuration keys, files,
  environment variables, defaults, and precedence separately for global
  `in_place` and insert `after`.
- [x] (2026-07-03T07:00:28Z) Work item 4 deterministic gates passed:
  `make all` at `/tmp/workitem4-all-mapsplice-roadmap-4-4-3.out`,
  `make markdownlint` at
  `/tmp/workitem4-markdownlint-mapsplice-roadmap-4-4-3.out`, and `make nixie` at
  `/tmp/workitem4-nixie-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T07:00:28Z) Work item 4 CodeRabbit review was attempted via
  the review-agent wrapper. It returned
  `deferred coderabbit review: no default network route visible in this sandbox`;
  the output is captured at
  `/tmp/workitem4-coderabbit-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T07:00:28Z) Reviewed the work item 4 semantic diff with
  `sem diff --file-exts .md`; the diff is limited to `docs/users-guide.md` and
  this ExecPlan.
- [x] (2026-07-03T07:01:48Z) Work item 5 updated
  `docs/developers-guide.md` to name the owning loaders for global `in_place`
  and insert `after`, corrected the shared guard path to
  `tests/support/config.rs`, and required future configuration settings to pin
  source and precedence claims with tests before guide updates.
- [x] (2026-07-03T07:01:48Z) Work item 5 marked `docs/roadmap.md` task 4.4.3
  complete.
- [x] (2026-07-03T07:03:16Z) Work item 5 deterministic gates passed:
  `make all` at `/tmp/workitem5-all-mapsplice-roadmap-4-4-3.out`,
  `make markdownlint` at
  `/tmp/workitem5-markdownlint-mapsplice-roadmap-4-4-3.out`, and `make nixie` at
  `/tmp/workitem5-nixie-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T07:03:16Z) Work item 5 CodeRabbit review was attempted via
  the review-agent wrapper. It returned
  `deferred coderabbit review: no default network route visible in this sandbox`;
  the output is captured at
  `/tmp/workitem5-coderabbit-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T07:03:16Z) Reviewed the work item 5 semantic diff with
  `sem diff --file-exts .md`; the diff is limited to `docs/developers-guide.md`,
  `docs/roadmap.md`, and this ExecPlan.
- [x] (2026-07-03T07:11:53Z) Fix round 1 began for the blocking dual-review
  finding against `docs/users-guide.md` insert `after` precedence prose.
- [x] (2026-07-03T07:11:53Z) Retried Memtrace for fix round 1. The MCP call
  `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`; the change continued with bounded
  branch-local evidence under this plan's tooling-fallback tolerance.
- [x] (2026-07-03T07:11:53Z) Retried Leta for fix round 1. The command
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-3`
  reported that the workspace was already added, and
  `leta grep "InsertConfig|with_absent_flags_removed|ArgAction|after" "src/cli.rs"`
  found `InsertConfig`, `after`, and `with_absent_flags_removed`.
- [x] (2026-07-03T07:11:53Z) Verified branch-local `src/cli.rs` evidence with
  `leta show`: `InsertConfig.after` uses
  `#[arg(long, action = ArgAction::SetTrue)]`, and
  `InsertConfig::with_absent_flags_removed` drops Clap's implicit value to
  `None` when `--after` is absent.
- [x] (2026-07-03T07:11:53Z) Updated `docs/users-guide.md` so the insert
  `after` precedence section now states that `--after` can only force
  `after = true` and that `MAPSPLICE_CMDS_INSERT_AFTER=false` is the
  per-process way to disable a file default.
- [x] (2026-07-03T07:15:21Z) Fix round 1 targeted Markdown formatting passed
  for `docs/users-guide.md` and `docs/execplans/roadmap-4-4-3.md`: first
  `mdtablefix`, then `markdownlint-cli2 --fix --no-globs`.
- [x] (2026-07-03T07:15:21Z) Scrutineer ran the deterministic fix-round gates
  sequentially. `make all` passed at `/tmp/all-mapsplice-roadmap-4-4-3.out`,
  `make markdownlint` passed at
  `/tmp/markdownlint-mapsplice-roadmap-4-4-3.out`, and `make nixie` passed at
  `/tmp/nixie-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T07:15:21Z) Scrutineer attempted CodeRabbit through
  `/home/leynos/Projects/mapsplice.workshop/df12-build-20260629T235232Z-879541/bin/coderabbit-review-agent`.
  The wrapper returned
  `deferred coderabbit review: no default network route visible in this sandbox`;
  the output is captured at `/tmp/coderabbit-mapsplice-roadmap-4-4-3.out`.
- [x] (2026-07-03T07:15:21Z) Fix round 1 is complete: the blocking review
  finding is resolved in `docs/users-guide.md`, deterministic gates are green,
  and the remaining CodeRabbit item is an infrastructure-deferred review issue.

## Surprises & discoveries

- Observation: Memtrace and Firecrawl MCP calls were cancelled by the host
  session rather than returning repository or documentation data. Evidence:
  both tools returned `user cancelled MCP tool call`. Impact: this plan records
  the failures and uses bounded local evidence plus locked Cargo registry
  source as fallback. This is not a product blocker under the task instructions.

- Observation: Leta is partially available. Evidence: `leta grep` returned
  branch-local symbols, but `leta refs EnvVarGuard` and `leta refs CwdGuard`
  failed with `Error: Failed to start daemon`. Impact: the implementer should
  retry Leta before editing and record the result here; this revision used
  bounded file inspection only for references that Leta could not provide.

- Observation: Leta could not be reattached during round 3. Evidence:
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-3`
  failed with `Error: IO error: Read-only file system (os error 30)`. Impact:
  the plan remains implementable because the round-3 change is a bounded test
  matrix correction verified by branch-local file inspection and locked crate
  source.

- Observation: Memtrace and Leta remained unavailable during execution.
  Evidence: `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`, and
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-3`
  returned `Error: IO error: Read-only file system (os error 30)`. Impact: work
  item 1 used precise branch-local inspection of `tests/support/config.rs`,
  `tests/roadmap_config.rs`, `src/cli_config.rs`, and `src/cli.rs`, which is
  within this plan's tooling-fallback tolerance.

- Observation: Memtrace remained unavailable during fix round 1. Evidence:
  `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`. Impact: this documentation-only fix used
  bounded branch-local `leta` evidence against `src/cli.rs` and direct
  inspection of the affected users' guide section.

- Observation: Leta was usable for the fix-round branch-local symbol check.
  Evidence:
  `leta grep "InsertConfig|with_absent_flags_removed|ArgAction|after" "src/cli.rs"`
  found the relevant insert configuration symbols, and `leta show` confirmed
  the `ArgAction::SetTrue` field and absent-flag normalisation behaviour.
  Impact: no ad hoc source search was needed for the reviewed CLI claim.

- Observation: the `scrutineer` recipe is installed but could not run in this
  execution environment. Evidence: `goose run --recipe scrutineer --no-session`
  failed with `error: No provider configured. Run 'goose configure' first.`
  Impact: deterministic gates and CodeRabbit were run directly, sequentially,
  with `/tmp` logs, preserving the evidence discipline required by this plan.

- Observation: CodeRabbit review is unavailable in this sandbox. Evidence: the
  review-agent wrapper returned
  `deferred coderabbit review: no default network route visible in this sandbox`.
  Impact: the deterministic gates remain authoritative for work item 1, and
  the deferred review is an open issue for the supervising workflow.

- Observation: the existing config test helpers cannot be composed safely.
  Evidence: `tests/support/config.rs` has one `static ENV_LOCK: Mutex<()>` at
  line 37. `EnvVarGuard::set` and `CwdGuard::enter` each acquire that same lock
  and store the `MutexGuard` in the returned guard. Impact: tests for
  local-over-XDG, env-over-local-with-cwd, or home-plus-XDG precedence must use
  one combined guard that takes the lock once.

- Observation: global and insert configuration are not loaded through one
  identical mechanism. Evidence: `src/cli_config.rs` lines 13-24 merge global
  file, environment, and CLI values manually. `src/cli.rs` lines 262-277 merge
  insert defaults through `load_merged_config`, which calls
  `ortho_config::load_and_merge_subcommand_for` at lines 317-322. Impact: the
  guides must not imply every optional default uses the same file discovery
  path.

- Observation: an XDG-over-home test alone does not prove that
  `~/.mapsplice.toml` is discovered. Evidence: with home `after = false` and XDG
  `after = true`, the final inserted task would still become `1.1.2` if the
  loader ignored home and read only XDG. Impact: work item 3 now requires
  `insert_after_home_dotfile_default`, with home `after = true` as the only
  populated source and explicit clearing of inherited insert environment values.

- Observation: global `in_place` file discovery is narrower than subcommand
  discovery. Evidence: `src/cli_config.rs` lines 40-48 search
  `$XDG_CONFIG_HOME/mapsplice/config.toml` when `XDG_CONFIG_HOME` is Unicode,
  then `./.mapsplice.toml`; lines 27-37 apply later discovered files over
  earlier ones. No home dotfile or `XDG_CONFIG_DIRS` path is searched there.
  Impact: the users' guide must describe global files separately from insert
  subcommand files.

- Observation: insert `after` uses the locked `ortho_config` subcommand path
  order on this Unix-like environment. Evidence: locked
  `ortho_config-0.8.0/src/subcommand/paths.rs` lines 93-115 add `HOME` then XDG
  candidates, lines 139-152 document home, platform config, and
  current-directory order, and lines 179-189 append local candidates last.
  `src/subcommand/mod.rs` lines 51-58 merge files in candidate order, lines
  61-79 merge environment variables after files, and lines 118-129 and 165-170
  merge CLI values last through the configured prefix.
  `src/subcommand/types.rs` lines 23-43 normalise `MAPSPLICE_` to `mapsplice`
  for file paths while preserving the raw prefix for environment names. Impact:
  the plan can state insert precedence as CLI `--after` over
  `MAPSPLICE_CMDS_INSERT_AFTER` over local `./.mapsplice.toml` over XDG
  `mapsplice/config.toml` over home `~/.mapsplice.toml`.

## Decision Log

- Decision: revise the plan to make process-state test support the first
  implementation work item. Rationale: the previous plan required tests that
  would deadlock by composing two guards that each held the same non-reentrant
  mutex. Date/Author: 2026-07-03 / planning agent.

- Decision: replace the public config-test guard surface with a single
  `ProcessStateGuard` rather than asking implementers to remember which guard
  combinations are forbidden. Rationale: one lock-owning guard can set two or
  more environment variables and enter the workspace root without re-locking
  `ENV_LOCK`, which directly resolves both design-review blocking points.
  Date/Author: 2026-07-03 / planning agent.

- Decision: implement this roadmap item as test support, tests, and
  documentation, with no production code changes. Rationale: the task says to
  document implemented discovery truthfully, not to change discovery. Existing
  code already has behaviour to document, but the guides need tests that pin
  newly stated precedence claims. Date/Author: 2026-07-03 / planning agent.

- Decision: document global and insert settings as distinct mechanisms.
  Rationale: global `in_place` is handled by mapsplice's custom loader, while
  insert `after` is handled by locked `ortho_config` subcommand helpers. A
  single generic precedence paragraph would be inaccurate. Date/Author:
  2026-07-03 / planning agent.

- Decision: do not use property tests, CrossHair, mutmut, Kani, or Verus for
  this task. Rationale: the failure mode is documentation drift around concrete
  configuration precedence cases. `rust-unit-testing` with `rstest` and
  `serial_test` is the smallest useful verification layer. Date/Author:
  2026-07-03 / planning agent.

- Decision: add a positive home-dotfile-only insert test before documenting
  `~/.mapsplice.toml` as a supported insert source. Rationale: documentation
  must be truthful, and XDG-over-home precedence does not prove that the home
  dotfile is read or that the test guard's `HOME` override reaches
  `ortho_config`. Date/Author: 2026-07-03 / planning agent.

- Decision: make `ProcessStateGuard::set_env` and
  `ProcessStateGuard::remove_env` infallible methods returning `()`, while
  keeping `acquire` and `enter_dir` fallible. Rationale: Clippy correctly
  reported that environment set/remove operations have no local error path
  under the guard, and returning `Result` would violate the repository's
  `-D warnings` lint policy. Date/Author: 2026-07-03 / implementation agent.

- Decision: resolve fix round 1 with documentation only. Rationale: the
  reviewed behaviour is already implemented by `ArgAction::SetTrue` and
  absent-flag normalisation; the blocker is that the users' guide did not state
  the one-way command-line override clearly. Date/Author: 2026-07-03 / fix
  round 1 agent.

## Outcomes & Retrospective

Implementation completed the planned test and documentation surfaces. The test
suite now includes one combined `ProcessStateGuard`, global `in_place`
precedence tests, and insert `after` discovery and precedence tests, including
the positive home-dotfile-only case. The users' guide now documents the
supported configuration keys, files, environment variables, defaults, and
precedence separately for global `in_place` and insert `after`. The developers'
guide now identifies the owning loader for each setting and points future
configuration changes at source-and-precedence tests before prose changes.

`docs/roadmap.md` task 4.4.3 is marked complete. CodeRabbit review attempts
were deferred in this sandbox because no default network route was visible; the
deterministic gates were run directly and sequentially with logs under `/tmp`.
Work item 5 gate logs are `/tmp/workitem5-all-mapsplice-roadmap-4-4-3.out`,
`/tmp/workitem5-markdownlint-mapsplice-roadmap-4-4-3.out`, and
`/tmp/workitem5-nixie-mapsplice-roadmap-4-4-3.out`. Final HEAD gate evidence
for fix round 1 is `/tmp/all-mapsplice-roadmap-4-4-3.out`,
`/tmp/markdownlint-mapsplice-roadmap-4-4-3.out`, and
`/tmp/nixie-mapsplice-roadmap-4-4-3.out`. The fix-round CodeRabbit attempt is
captured at `/tmp/coderabbit-mapsplice-roadmap-4-4-3.out` and remains deferred
because this sandbox has no default network route.

## Context and orientation

The current users' guide configuration section is short. In
`docs/users-guide.md` section "Configuration", it says optional subcommand
settings can come from configuration and gives only the
`[cmds.insert] after = true` and `MAPSPLICE_CMDS_INSERT_AFTER=true` examples.
It does not name the files searched, file precedence, global `in_place`, or the
fact that global and subcommand defaults use different loaders.

The current developers' guide configuration section is also too generic. In
`docs/developers-guide.md` section 4, "Configuration behaviour", it says global
`in_place` may come from environment, configuration files, or flags, and insert
`after` may come from environment, `[cmds.insert]`, or `--after`. It does not
tell maintainers that `src/cli_config.rs` owns global `in_place` discovery while
`src/cli.rs` calls `ortho_config::load_and_merge_subcommand_for` for insert
defaults. It also mentions `tests/support/mod.rs`, but the actual config helper
file is `tests/support/config.rs`.

The implementation facts to preserve are:

- Global `in_place` is represented by `GlobalCli.in_place` in `src/cli.rs`
  lines 29-37 and resolves to `false` when absent at lines 240-246.
- Global `in_place` file defaults are loaded by `src/cli_config.rs` lines
  27-48. Candidate files are `$XDG_CONFIG_HOME/mapsplice/config.toml`, then
  `./.mapsplice.toml`; later candidates override earlier candidates.
- Global `MAPSPLICE_IN_PLACE` is parsed at `src/cli_config.rs` lines 101-123
  and overrides files at lines 17-19.
- The command-line `--in-place` / `-i` flag can set `in_place` to `true` and
  overrides files and environment at `src/cli_config.rs` lines 20-22. There is
  no command-line flag to force `false`.
- Insert `after` is represented by private `InsertConfig.after` in
  `src/cli.rs` lines 218-225. `InsertArgs::into_request` at lines 262-277
  removes absent implicit `false` values and then calls `load_merged_config`.
- `load_merged_config` in `src/cli.rs` lines 317-322 delegates to locked
  `ortho_config::load_and_merge_subcommand_for`.
- Locked `ortho_config = 0.8.0` is recorded in `Cargo.toml` and `Cargo.lock`.
  Its default features include `serde_json` and `toml`, so this plan must rely
  on TOML config discovery only.
- Locked `ortho_config = 0.8.0` turns `MAPSPLICE_` into the file prefix
  `mapsplice` while keeping `MAPSPLICE_` for environment names. The relevant
  evidence is `src/subcommand/types.rs` lines 23-43 and `src/subcommand/mod.rs`
  lines 70-76.
- Locked `ortho_config = 0.8.0` subcommand candidates on this Unix-like
  environment are home `~/.mapsplice.toml`, XDG
  `$XDG_CONFIG_HOME/mapsplice/config.toml` when present, and local
  `./.mapsplice.toml`. The relevant evidence is `src/subcommand/paths.rs` lines
  93-115 and 139-189.
- Locked `ortho_config = 0.8.0` merges subcommand files in candidate order,
  then environment, then CLI. The relevant evidence is `src/subcommand/mod.rs`
  lines 51-58, 61-79, 118-129, and 165-170.

## Plan of work

### Work item 1: Replace config tests with one process-state guard

Docs and sections to read first:

- `AGENTS.md` "Rust Specific Guidance" and "Testing".
- `docs/developers-guide.md` section 6, "Verification layers".
- `docs/mapsplice-design.md` section 2, "Non-negotiable constraints".

Skills to load:

- `leta` for branch-local symbol navigation; if `leta refs` fails, record the
  exact failure and use bounded source inspection.
- `rust-router`, then `rust-unit-testing` for test helper and `serial_test`
  shape.

Implementation:

1. In `tests/support/config.rs`, replace the public `EnvVarGuard` and
   `CwdGuard` config-test surface with one public `ProcessStateGuard`.
2. `ProcessStateGuard` must own one `MutexGuard<'static, ()>` from `ENV_LOCK`
   for its whole lifetime. It must provide methods equivalent to:

```rust
pub struct ProcessStateGuard { /* lock, saved env, saved cwd */ }

impl ProcessStateGuard {
    pub fn acquire() -> TestResult<Self>;
    pub fn set_env(&mut self, key: &'static str, value: impl AsRef<str>) -> TestResult;
    pub fn remove_env(&mut self, key: &'static str) -> TestResult;
    pub fn enter_dir(&mut self, path: &Utf8Path) -> TestResult;
}
```

1. Store each environment key's original value once, even if a test sets the
   same key twice, so `Drop` restores the pre-test state.
2. `remove_env` must also store the original value once before removing the
   variable, so tests can prove a configuration source is absent even when the
   invoking shell exported that variable.
3. Store the original current directory the first time `enter_dir` is called.
   `Drop` restores the directory and environment without panicking.
4. Add
   `Workspace::enter_root(&self, guard: &mut ProcessStateGuard) -> TestResult`
   so cwd changes use the already-held lock instead of acquiring a second guard.
5. Add
   `Workspace::write_home_config(&self, contents: &str) -> TestResult<Utf8PathBuf>`
   that writes `home/.mapsplice.toml` under the temporary workspace and
   returns the `home` directory path. This helper must not mutate `HOME`;
   callers set `HOME` through `ProcessStateGuard`.
6. Migrate existing `tests/roadmap_config.rs` tests that use `EnvVarGuard` or
   `Workspace::enter_root()` to the new `ProcessStateGuard` surface.
7. Add a helper-focused test named
   `process_state_guard_allows_multiple_env_vars_and_cwd` in
   `tests/roadmap_config.rs`. It must set two harmless test environment
   variables and enter the workspace root through one guard, assert the values
   and cwd while the guard is alive, then assert restoration after the guard is
   dropped.

Tests:

- Add or update unit-style integration tests in `tests/roadmap_config.rs`.
- Use `#[rstest]` and `#[serial_test::serial(cli_env)]` because these tests
  mutate process-wide state.
- Red evidence: first add the helper-focused test against the existing
  separate-guard surface and record that the required simultaneous state cannot
  be expressed without deadlock. Then add `ProcessStateGuard` and rerun the
  focused command.
- Focused command:

```bash
set -o pipefail
cargo test --test roadmap_config process_state_guard -- --nocapture 2>&1 | tee /tmp/workitem1-roadmap-config-mapsplice-roadmap-4-4-3.out
```

Validation before commit:

```bash
set -o pipefail
mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place docs/execplans/roadmap-4-4-3.md 2>&1 | tee /tmp/workitem1-mdtablefix-mapsplice-roadmap-4-4-3.out
set -o pipefail
markdownlint-cli2 --fix --no-globs -- docs/execplans/roadmap-4-4-3.md 2>&1 | tee /tmp/workitem1-markdownfix-mapsplice-roadmap-4-4-3.out
set -o pipefail
make all 2>&1 | tee /tmp/workitem1-all-mapsplice-roadmap-4-4-3.out
set -o pipefail
make markdownlint 2>&1 | tee /tmp/workitem1-markdownlint-mapsplice-roadmap-4-4-3.out
set -o pipefail
make nixie 2>&1 | tee /tmp/workitem1-nixie-mapsplice-roadmap-4-4-3.out
```

Commit:

```bash
git add tests/support/config.rs tests/roadmap_config.rs docs/execplans/roadmap-4-4-3.md
git commit -m "Reshape config test state guard"
```

### Work item 2: Pin global `in_place` discovery and precedence

Docs and sections to read first:

- `docs/roadmap.md` section 4.4, task 4.4.3.
- `docs/developers-guide.md` section 4, "Configuration behaviour".
- `docs/mapsplice-design.md` "Status and scope" and section 2.
- `AGENTS.md` "Testing".

Skills to load:

- `leta` for `load_global_config`, `global_config_file_default`,
  `global_config_candidates`, and `global_env_default`; if unavailable, record
  the exact failure and use bounded source inspection.
- `rust-router`, then `rust-unit-testing`.

Implementation:

1. Re-read `src/cli_config.rs::load_global_config`,
   `global_config_file_default`, `global_config_candidates`, and
   `global_env_default`.
2. Add `in_place_local_config_overrides_xdg_config` to
   `tests/roadmap_config.rs`. The test must write
   `$XDG_CONFIG_HOME/mapsplice/config.toml` with `in_place = false`, write local
   `./.mapsplice.toml` with `in_place = true`, set `XDG_CONFIG_HOME`, and
   enter the workspace root using one `ProcessStateGuard`. It must prove local
   wins by observing an in-place delete: no stdout and the target file changed.
3. Add `in_place_env_false_overrides_local_config_true`. The test must write
   local `./.mapsplice.toml` with `in_place = true`, set
   `MAPSPLICE_IN_PLACE=false`, and enter the workspace root using one
   `ProcessStateGuard`. It must prove the environment wins by observing normal
   stdout output and an unchanged target file.

Tests:

- Add unit-style integration tests in `tests/roadmap_config.rs`.
- Red evidence: temporarily invert one expected assertion and record the
  focused failure before restoring the intended assertion. Do not commit the
  red mutation.
- Focused command:

```bash
set -o pipefail
cargo test --test roadmap_config in_place -- --nocapture 2>&1 | tee /tmp/workitem2-roadmap-config-mapsplice-roadmap-4-4-3.out
```

Validation before commit:

```bash
set -o pipefail
mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place docs/execplans/roadmap-4-4-3.md 2>&1 | tee /tmp/workitem2-mdtablefix-mapsplice-roadmap-4-4-3.out
set -o pipefail
markdownlint-cli2 --fix --no-globs -- docs/execplans/roadmap-4-4-3.md 2>&1 | tee /tmp/workitem2-markdownfix-mapsplice-roadmap-4-4-3.out
set -o pipefail
make all 2>&1 | tee /tmp/workitem2-all-mapsplice-roadmap-4-4-3.out
set -o pipefail
make markdownlint 2>&1 | tee /tmp/workitem2-markdownlint-mapsplice-roadmap-4-4-3.out
set -o pipefail
make nixie 2>&1 | tee /tmp/workitem2-nixie-mapsplice-roadmap-4-4-3.out
```

Commit:

```bash
git add tests/roadmap_config.rs docs/execplans/roadmap-4-4-3.md
git commit -m "Pin global configuration precedence"
```

### Work item 3: Pin insert `after` discovery and precedence

Docs and sections to read first:

- `docs/roadmap.md` section 4.4, task 4.4.3.
- `docs/developers-guide.md` section 4 and section 6.
- Locked `ortho_config = 0.8.0` source:
  `src/subcommand/mod.rs`, `src/subcommand/paths.rs`, and
  `src/subcommand/types.rs`.

Skills to load:

- `leta` for `InsertConfig`, `InsertArgs::into_request`, and
  `load_merged_config`; if unavailable, record the exact failure and use
  bounded source inspection.
- `rust-router`, then `rust-unit-testing`.

Implementation:

1. Re-read `src/cli.rs::InsertConfig`,
   `InsertConfig::with_absent_flags_removed`, `InsertArgs::into_request`, and
   `load_merged_config`.
2. Add `insert_after_home_dotfile_default`. The test must write home
   `[cmds.insert] after = true` through `Workspace::write_home_config`, set
   `HOME` to the returned home directory, set `XDG_CONFIG_HOME` to a temporary
   workspace directory that does not contain `mapsplice/config.toml`, remove
   `MAPSPLICE_CMDS_INSERT_AFTER`, and enter the workspace root through one
   `ProcessStateGuard` without writing local `./.mapsplice.toml`. It must not
   pass `--after`. It must prove the home dotfile alone is discovered by
   asserting the inserted task becomes `1.1.2`.
3. Add `insert_after_local_config_overrides_xdg_config`. The test must write
   XDG `[cmds.insert] after = false`, local `[cmds.insert] after = true`, set
   `XDG_CONFIG_HOME`, and enter the workspace root through one
   `ProcessStateGuard`. It must prove local wins by asserting the inserted task
   becomes `1.1.2`.
4. Add `insert_after_xdg_config_overrides_home_dotfile`. The test must write
   home `[cmds.insert] after = false` through `Workspace::write_home_config`,
   write XDG `[cmds.insert] after = true`, set both `HOME` and
   `XDG_CONFIG_HOME`, and enter the workspace root through one
   `ProcessStateGuard`. It must prove XDG wins by asserting the inserted task
   becomes `1.1.2`.
5. Add `insert_after_env_false_overrides_local_config_true`. The test must
   write local `[cmds.insert] after = true`, set
   `MAPSPLICE_CMDS_INSERT_AFTER=false`, and enter the workspace root through one
   `ProcessStateGuard`. It must prove the environment wins by asserting the
   inserted task becomes `1.1.1`.
6. Add `insert_after_cli_flag_overrides_env_false`. The test must set
   `MAPSPLICE_CMDS_INSERT_AFTER=false`, pass `--after`, and prove the CLI wins
   by asserting the inserted task becomes `1.1.2`.

Tests:

- Add unit-style integration tests in `tests/roadmap_config.rs`.
- Use existing output assertions: after insertion is visible through the
  inserted task becoming `1.1.2`; before insertion is visible through the
  inserted task becoming `1.1.1`.
- Red evidence: temporarily invert one expected precedence case and record the
  focused test failure before restoring the intended assertion. Do not commit
  the red mutation.
- Focused command:

```bash
set -o pipefail
cargo test --test roadmap_config insert_after -- --nocapture 2>&1 | tee /tmp/workitem3-roadmap-config-mapsplice-roadmap-4-4-3.out
```

Validation before commit:

```bash
set -o pipefail
mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place docs/execplans/roadmap-4-4-3.md 2>&1 | tee /tmp/workitem3-mdtablefix-mapsplice-roadmap-4-4-3.out
set -o pipefail
markdownlint-cli2 --fix --no-globs -- docs/execplans/roadmap-4-4-3.md 2>&1 | tee /tmp/workitem3-markdownfix-mapsplice-roadmap-4-4-3.out
set -o pipefail
make all 2>&1 | tee /tmp/workitem3-all-mapsplice-roadmap-4-4-3.out
set -o pipefail
make markdownlint 2>&1 | tee /tmp/workitem3-markdownlint-mapsplice-roadmap-4-4-3.out
set -o pipefail
make nixie 2>&1 | tee /tmp/workitem3-nixie-mapsplice-roadmap-4-4-3.out
```

Commit:

```bash
git add tests/roadmap_config.rs docs/execplans/roadmap-4-4-3.md
git commit -m "Pin insert configuration precedence"
```

### Work item 4: Update the users' guide configuration reference

Docs and sections to read first:

- `docs/users-guide.md` section "Configuration".
- `docs/documentation-style-guide.md` sections "User's guide", "Markdown
  rules", and "Formatting".
- `docs/mapsplice-design.md` "Status and scope".

Skills to load:

- `en-gb-oxendict-style`.
- `leta` is not required for Markdown-only edits unless code facts need to be
  rechecked; if code facts are rechecked and Leta is unavailable, record the
  exact failure.

Implementation:

1. Replace the current short configuration section in `docs/users-guide.md`
   with a user-facing reference that separates the two supported settings:
   global `in_place` and insert `after`.
2. State the actual file syntax: top-level `in_place = true` or `false`, and
   `[cmds.insert] after = true` or `false`.
3. State global `in_place` discovery exactly: XDG
   `$XDG_CONFIG_HOME/mapsplice/config.toml` when `XDG_CONFIG_HOME` is valid
   Unicode, then local `./.mapsplice.toml`.
4. State insert `after` discovery exactly on Unix-like systems: home
   `~/.mapsplice.toml`, XDG `$XDG_CONFIG_HOME/mapsplice/config.toml` when
   present, then local `./.mapsplice.toml`.
5. State precedence without hedging. Global `in_place`: `--in-place` or `-i`
   true over `MAPSPLICE_IN_PLACE` over local file over XDG file over default
   `false`; there is no command-line flag to force `false`. Insert `after`:
   `--after` over `MAPSPLICE_CMDS_INSERT_AFTER` over local file over XDG file
   over home dotfile over default before-anchor insertion.
6. Keep maintainer rationale out of the users' guide. Link to the developers'
   guide only if the text would otherwise need internal implementation details.

Tests:

- No new Rust test belongs in this work item because work items 2 and 3 pin the
  behavioural claims before this prose changes.

Validation before commit:

```bash
set -o pipefail
mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place \
  docs/users-guide.md docs/execplans/roadmap-4-4-3.md \
  2>&1 | tee /tmp/workitem4-mdtablefix-mapsplice-roadmap-4-4-3.out
set -o pipefail
markdownlint-cli2 --fix --no-globs -- \
  docs/users-guide.md docs/execplans/roadmap-4-4-3.md \
  2>&1 | tee /tmp/workitem4-markdownfix-mapsplice-roadmap-4-4-3.out
set -o pipefail
make all 2>&1 | tee /tmp/workitem4-all-mapsplice-roadmap-4-4-3.out
set -o pipefail
make markdownlint 2>&1 | tee /tmp/workitem4-markdownlint-mapsplice-roadmap-4-4-3.out
set -o pipefail
make nixie 2>&1 | tee /tmp/workitem4-nixie-mapsplice-roadmap-4-4-3.out
```

Commit:

```bash
git add docs/users-guide.md docs/execplans/roadmap-4-4-3.md
git commit -m "Document user configuration discovery"
```

### Work item 5: Update maintainer docs and close roadmap task 4.4.3

Docs and sections to read first:

- `docs/developers-guide.md` section 4, "Configuration behaviour".
- `docs/developers-guide.md` section 7, "Local tooling".
- `docs/roadmap.md` section 4.4.
- `docs/documentation-style-guide.md` sections "Developer's guide" and
  "Formatting".

Skills to load:

- `en-gb-oxendict-style`.
- `sem` for semantic diff review before committing.
- `leta` only if code symbols must be rechecked; record unavailability if it
  fails.

Implementation:

1. Update `docs/developers-guide.md` section 4 so maintainers know where to
   look: `src/cli_config.rs` owns global `in_place` discovery and parsing;
   `src/cli.rs::InsertConfig` uses
   `ortho_config::load_and_merge_subcommand_for` for insert defaults.
2. State that future configuration settings must document which loader owns
   them and must add tests that pin their source and precedence.
3. Correct the shared guard reference to `tests/support/config.rs`.
4. Update `docs/roadmap.md` task 4.4.3 from `[ ]` to `[x]` only after the guide
   updates and final gates pass.
5. Update this ExecPlan's `Progress`, `Surprises & Discoveries`,
   `Decision Log`, and `Outcomes & Retrospective` with final evidence.

Tests:

- No new Rust test belongs in this work item because it is a maintainer
  documentation and roadmap-status update. Work items 2 and 3 already pin the
  behavioural claims.
- Review the semantic diff before final gates:

```bash
sem diff --from origin/main --to HEAD --file-exts .rs .md
```

Validation before commit:

```bash
set -o pipefail
mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place \
  docs/developers-guide.md docs/roadmap.md docs/execplans/roadmap-4-4-3.md \
  2>&1 | tee /tmp/workitem5-mdtablefix-mapsplice-roadmap-4-4-3.out
set -o pipefail
markdownlint-cli2 --fix --no-globs -- \
  docs/developers-guide.md docs/roadmap.md docs/execplans/roadmap-4-4-3.md \
  2>&1 | tee /tmp/workitem5-markdownfix-mapsplice-roadmap-4-4-3.out
set -o pipefail
make all 2>&1 | tee /tmp/workitem5-all-mapsplice-roadmap-4-4-3.out
set -o pipefail
make markdownlint 2>&1 | tee /tmp/workitem5-markdownlint-mapsplice-roadmap-4-4-3.out
set -o pipefail
make nixie 2>&1 | tee /tmp/workitem5-nixie-mapsplice-roadmap-4-4-3.out
```

Commit:

```bash
git add docs/developers-guide.md docs/roadmap.md docs/execplans/roadmap-4-4-3.md
git commit -m "Close configuration discovery docs"
```

## Concrete steps

Before implementation, the next agent should:

1. Re-read this ExecPlan completely.
2. Confirm branch and worktree:

```bash
git branch --show-current
pwd
```

Expected output includes:

```plaintext
roadmap-4-4-3
/home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-3
```

1. Retry Memtrace and Leta before editing. If they fail, append the exact
   failure to `Surprises & Discoveries` and continue with bounded local
   evidence:

```bash
leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-4-3
leta grep "load_global_config|InsertConfig|load_merged_config|ProcessStateGuard" "src/|tests/"
```

1. Implement the work items in order. Each item ends with gates and a commit.
   Do not skip ahead after a failed gate.

## Validation and acceptance

Final acceptance requires all of the following:

- `tests/support/config.rs` exposes one combined `ProcessStateGuard` for config
  tests that need multiple environment variables and/or cwd mutation under one
  `ENV_LOCK` acquisition.
- `tests/roadmap_config.rs` contains focused tests for global `in_place`
  local-over-XDG and environment-over-local precedence.
- `tests/roadmap_config.rs` contains focused tests for insert `after`
  home-dotfile-only discovery, home/XDG/local precedence,
  environment-over-local precedence, and CLI-over-environment precedence.
- `docs/users-guide.md` names the actual supported config files, keys,
  environment variables, defaults, and precedence for global `in_place` and
  insert `after`.
- `docs/developers-guide.md` names the owning loader for global and insert
  configuration and tells maintainers how to preserve documentation accuracy
  for future settings.
- `docs/roadmap.md` marks task 4.4.3 complete.
- This ExecPlan is updated as a living document.
- The final deterministic gates pass:

```bash
set -o pipefail
make all 2>&1 | tee /tmp/final-all-mapsplice-roadmap-4-4-3.out
set -o pipefail
make markdownlint 2>&1 | tee /tmp/final-markdownlint-mapsplice-roadmap-4-4-3.out
set -o pipefail
make nixie 2>&1 | tee /tmp/final-nixie-mapsplice-roadmap-4-4-3.out
```

The expected result for each final gate is exit status 0. If any command fails,
read the cited `/tmp` log and fix the specific failure before proceeding.

## Idempotence and recovery

The planned test and documentation edits are idempotent. If a work item is
interrupted, inspect `git status --short`, re-read this ExecPlan's `Progress`,
and continue from the first unchecked step. Do not use a bare `git stash`. If a
stash is unavoidable, name it with:

```plaintext
df12-stash v1 task=4.4.3 kind=<discard|park|keep> reason="<short>"
```

If scoped Markdown formatting creates unrelated churn, do not commit it. Revert
only the unrelated files after confirming they were produced by the formatter
and are not user work.

## Artifacts and notes

Important planning evidence:

```plaintext
Memtrace: mcp__memtrace.list_indexed_repositories -> user cancelled MCP tool call
Firecrawl: mcp__firecrawl.firecrawl_scrape docs.rs ortho_config 0.8.0 -> user cancelled MCP tool call
Leta: leta workspace add ... -> Workspace already added
Leta round 3: leta workspace add ... -> Error: IO error: Read-only file system (os error 30)
Leta: leta grep ... -> located config symbols
Leta: leta refs EnvVarGuard -> Error: Failed to start daemon
Leta: leta refs CwdGuard -> Error: Failed to start daemon
Sem: sem entities tests/support/config.rs, src/cli_config.rs, src/cli.rs -> listed expected entities
```

The branch-local code and tests that drove this plan are:

- `src/cli_config.rs`
- `src/cli.rs`
- `tests/roadmap_config.rs`
- `tests/support/config.rs`

The locked external library evidence used because Firecrawl/docs.rs retrieval
was unavailable is:

- `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ortho_config-0.8.0/src/subcommand/mod.rs`
- `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ortho_config-0.8.0/src/subcommand/paths.rs`
- `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ortho_config-0.8.0/src/subcommand/types.rs`
- `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ortho_config-0.8.0/Cargo.toml`

## Interfaces and dependencies

No public interface or dependency changes are planned.

The implementation must continue to use the existing dependencies and versions
locked in `Cargo.lock`:

- `ortho_config = 0.8.0` for insert subcommand configuration merging.
- `rstest = 0.26.1` for configuration test fixtures and cases.
- `serial_test = 3.2.0` for process-wide environment and current-directory
  serialization.

No work item may add, remove, or retune dependencies. If implementation appears
to need a dependency change, stop, update this plan, and escalate for design
review.

## Revision note

Round 2 revises the plan around the design-review finding that the previous
test matrix would deadlock by composing `EnvVarGuard` and `CwdGuard`, or two
`EnvVarGuard`s, while each held the same non-reentrant `ENV_LOCK`. Work item 1
now authorises and specifies a single combined `ProcessStateGuard` that acquires
`ENV_LOCK` once and supports multiple environment mutations plus cwd changes.
Work items 2 and 3 use that surface for every local-plus-XDG, env-plus-cwd, and
home-plus-XDG test.

Round 3 revises the plan around the design-review finding that
`insert_after_xdg_config_overrides_home_dotfile` did not positively demonstrate
home-dotfile discovery. Work item 1 now requires an env-removal method, and
work item 3 now requires `insert_after_home_dotfile_default`, where
`~/.mapsplice.toml` with `[cmds.insert] after = true` is the only populated
configuration source and must make the inserted task become `1.1.2`.

Execution update 1 implements work item 1. The helper-focused test now proves a
single guard can set multiple environment variables, remove an environment
variable, enter the workspace root, and restore all process state when dropped.
It also exercises `Workspace::write_home_config`, which later work items use
for home-dotfile discovery tests.

Execution update 2 completes roadmap task 4.4.3. The remaining work item
updated maintainer-facing documentation, marked the roadmap checkbox complete,
and recorded that all CodeRabbit attempts were deferred by the sandbox network
environment rather than by actionable review findings.
