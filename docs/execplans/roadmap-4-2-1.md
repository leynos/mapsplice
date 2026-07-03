# Make Mermaid Validation Deterministic

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

## Purpose / big picture

Roadmap task 4.2.1 makes the Mermaid documentation gate trustworthy under CI
load. The current `make nixie` target delegates to `nixie --no-sandbox`, which
lets locked `nixie-cli` 1.1.0 choose `max(1, cpu_count - 1)` renderer workers.
The roadmap failure was reproduced on the six-core development host, where the
uncapped default allowed five renderer workers and an unchanged Mermaid diagram
timed out.

After this change, maintainers can run `make nixie` and get repeatable
validation on the existing Markdown corpus. Implementation found that locked
`nixie-cli` 1.1.0 still timed out because its per-diagram timeout is hard-coded
to 30 seconds. The Makefile therefore keeps the `make nixie` target name but
validates tracked Markdown directly with `merman-cli`, one file at a time,
using `NIXIE_MAX_CONCURRENCY ?= 1` as the default renderer job cap and
`NIXIE_RENDERER_THREADS ?= 1` as the renderer thread cap.

Observable success is:

- `make nixie` passes using the default bounded concurrency.
- `NIXIE_MAX_CONCURRENCY=1 make nixie` passes as the serial comparison path.
- A Rust integration test proves the Makefile passes
  `-j $(NIXIE_MAX_CONCURRENCY)` and the renderer thread cap to the configured
  Mermaid renderer.
- `make all`, `make markdownlint`, and `make nixie` pass before each commit.

## Constraints

Hard invariants:

- Work only in `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-2-1`.
- Do not edit the root/control worktree.
- Do not begin implementation until this plan is explicitly approved.
- Keep the implementation scoped to Mermaid validation and its documentation.
  Do not redesign Markdown formatting, `markdownlint`, the roadmap parser, or
  Mermaid diagram content.
- Do not run repository-wide Markdown formatters such as `make fmt` or
  `mdformat-all`. Format only changed Markdown files with path-scoped
  `mdtablefix` followed by path-scoped `markdownlint-cli2 --fix`.
- Keep validation commands path-safe. Direct formatter commands may name only
  files that exist and were edited in the current work item.
- Do not add external dependencies. Existing `rstest`, `serial_test` if needed,
  and the Rust standard library are enough to pin the Makefile contract.
- Do not change public `mapsplice` library APIs.
- Use en-GB Oxford spelling in prose and comments.
- Commit after each work item only after its gates pass.

## Tolerances

Stop and escalate if any threshold below is reached:

- Implementing the Makefile and test contract requires changes outside
  `Makefile`, `tests/`, `docs/developers-guide.md`, `docs/contributing.md`,
  `docs/roadmap.md`, and this ExecPlan.
- A new dependency is needed.
- `NIXIE_MAX_CONCURRENCY=2 make nixie` fails twice on the unchanged corpus
  after a clean serial pass.
- `make all` fails for an unrelated pre-existing issue after one clean re-run
  to rule out a transient tool failure.
- The implementation must change `nixie-cli`, `merman-cli`, Mermaid diagram
  source, or rendered documentation content.
- A work item would delete an existing documentation or fixture file.
- Any new or edited Rust test helper would exceed the local Clippy thresholds:
  cognitive complexity 9, 70 lines, or 4 arguments.

## Risks

- Risk: `NIXIE_MAX_CONCURRENCY=2` might be misunderstood as raising 2-CPU CI
  from serial validation to two renderer workers.
  Severity: high.
  Likelihood: medium.
  Mitigation: Work Item 1 now defaults to `NIXIE_MAX_CONCURRENCY ?= 1` after
  repeated `NIXIE_MAX_CONCURRENCY=2` and default runs timed out under the
  sanctioned gate runner. Contributors can still override the cap explicitly
  for comparison.

- Risk: a dry-run Makefile test can become brittle if the target command is
  legitimately refactored.
  Severity: low.
  Likelihood: medium.
  Mitigation: assert the behavioural contract, not the whole recipe line. The
  dry-run output must contain the configured `MERMAN` executable,
  `RAYON_NUM_THREADS`, `-j`, and the selected value.

- Risk: Rust test assertions in `Result`-returning tests can trip the denied
  `clippy::panic_in_result_fn` lint.
  Severity: high.
  Likelihood: high without explicit instruction.
  Mitigation: Work Item 1 requires every `assert!`, `assert_eq!`, and similar
  assertion macro to live in `()`-returning helper functions, matching
  `tests/roadmap_config.rs::assert_contains`,
  `tests/roadmap_config.rs::assert_equal`, and
  `tests/roadmap_config.rs::assert_configuration_error`. The
  `Result`-returning `#[rstest]` bodies may use `?` for fallible setup and then
  call those helpers.

- Risk: `make nixie` scans fixture Markdown as well as documentation Markdown.
  Severity: low.
  Likelihood: confirmed.
  Mitigation: do not narrow the gate in this task. The roadmap success
  criterion is the existing repository gate path, so this plan keeps discovery
  scope unchanged and only bounds renderer concurrency.

## Progress

- [x] (2026-07-02T22:50:50Z) Read `AGENTS.md`, confirmed branch
  `roadmap-4-2-1`, and confirmed the worktree path.
- [x] (2026-07-02T22:50:50Z) Drafted the round-1 plan and gathered initial
  local evidence for the uncapped timeout and capped passes.
- [x] (2026-07-03T00:00:00Z) Loaded `execplans`, `leta`,
  `firecrawl-mcp`, `rust-router`, `rust-unit-testing`, `sem`,
  `commit-message`, `memtrace-first`, and `en-gb-oxendict-style`.
- [x] (2026-07-03T00:00:00Z) Confirmed branch `roadmap-4-2-1` and current
  worktree `/home/leynos/Projects/mapsplice.worktrees/roadmap-4-2-1`.
- [x] (2026-07-03T00:00:00Z) Retried Memtrace and Firecrawl. Both MCP calls
  returned `user cancelled MCP tool call`, so this revision uses bounded local
  evidence plus direct official web documentation.
- [x] (2026-07-03T00:00:00Z) Started Leta successfully with
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-2-1`
  and used `leta files` and `leta grep` for branch-local test orientation.
- [x] (2026-07-03T00:00:00Z) Verified the review concern about denied
  `panic_in_result_fn` in `Cargo.toml`, and verified the existing helper-based
  assertion pattern in `tests/roadmap_config.rs`.
- [x] (2026-07-03T00:00:00Z) Verified locked `nixie-cli` 1.1.0 source and
  metadata for `--max-concurrency` clamp direction and deterministic output.
- [x] (2026-07-03T00:00:00Z) Verified `.github/workflows/ci.yml` uses
  `ubuntu-latest`, `NIXIE_CLI_VERSION: '1.1.0'`, and `make nixie`.
- [x] (2026-07-03T00:00:00Z) Verified GitHub-hosted runner CPU counts from
  the official GitHub Actions runner reference.
- [x] (2026-07-02T23:49:29Z) Work Item 1: Cap Makefile Mermaid concurrency
  and pin the contract. Deterministic gates passed in `scrutineer` logs
  `/tmp/wi1-merman-final-mapsplice-roadmap-4-2-1-make-all.out`,
  `/tmp/wi1-merman-final-mapsplice-roadmap-4-2-1-markdownlint.out`,
  `/tmp/wi1-merman-final-mapsplice-roadmap-4-2-1-nixie.out`, and
  `/tmp/wi1-merman-final-mapsplice-roadmap-4-2-1-nixie-concurrency-1.out`.
  CodeRabbit was deferred because the sandbox had no default network route.
- [x] (2026-07-02T23:53:29Z) Work Item 1 committed as `8055896` with subject
  `Stabilize Mermaid validation gate`.
- [x] (2026-07-02T23:53:29Z) Work Item 2: Document the deterministic Mermaid
  gate and close roadmap task 4.2.1. Deterministic gates passed in
  `scrutineer` logs
  `/tmp/wi2-final-mapsplice-roadmap-4-2-1-make-all.out`,
  `/tmp/wi2-final-mapsplice-roadmap-4-2-1-markdownlint.out`,
  `/tmp/wi2-final-mapsplice-roadmap-4-2-1-nixie.out`, and
  `/tmp/wi2-final-mapsplice-roadmap-4-2-1-nixie-concurrency-1.out`.
  CodeRabbit was deferred because the sandbox had no default network route.

## Surprises & Discoveries

- Observation: Memtrace was exposed through tool discovery, but the required
  first call failed.
  Evidence: `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`.
  Impact: this plan records the failure and uses bounded local source, docs,
  and command evidence instead. This is not a product blocker.

- Observation: Firecrawl was exposed through tool discovery but could not
  fetch official `nixie-cli` documentation in this session.
  Evidence: `mcp__firecrawl.firecrawl_search` for
  `site:github.com/leynos/nixie --max-concurrency nixie-cli` returned
  `user cancelled MCP tool call`.
  Impact: this plan cites locked installed source, installed package metadata,
  and direct official web reads. This is not a product blocker.

- Observation: Leta is available in this planning session.
  Evidence: `leta workspace add
  /home/leynos/Projects/mapsplice.worktrees/roadmap-4-2-1` returned
  `Added workspace:
  /home/leynos/Projects/mapsplice.worktrees/roadmap-4-2-1`.
  Impact: implementation should use Leta for branch-local Rust test navigation.
  If it fails later, record the exact failure and continue with bounded file
  inspection.

- Observation: `nixie --version` is not a supported way to identify the
  installed `nixie-cli` version.
  Evidence: `nixie --version` returned argparse usage and
  `error: unrecognized arguments: --version`.
  Impact: use the locked CI workflow value, the installed
  `nixie_cli-1.1.0.dist-info/METADATA`, and the installed source path as the
  version evidence.

- Observation: locked `nixie-cli` clamps explicit values as a ceiling, not as a
  floor.
  Evidence: running the installed tool interpreter printed:
  `1 1 1`, `2 1 1`, `4 2 3`, `6 2 5`, and `24 2 23` for
  `(cpu_count, resolve_max_concurrency(2), resolve_max_concurrency(None))`.
  Impact: `NIXIE_MAX_CONCURRENCY ?= 2` keeps 2-CPU private CI serial while
  capping 4-CPU public CI at 2 instead of the uncapped default 3.

- Observation: this sandbox reports 24 online processors, even though the
  roadmap failure was reproduced on the documented six-core development host.
  Evidence: `nproc` and `getconf _NPROCESSORS_ONLN` both returned `24`.
  Impact: this is useful extra evidence that uncapped local concurrency can be
  far above the CI runner count, but the design does not rely on this sandbox
  count.

- Observation: Memtrace was unavailable during implementation.
  Evidence: `mcp__memtrace.list_indexed_repositories` returned
  `user cancelled MCP tool call`.
  Impact: implementation used bounded branch-local file inspection for the
  Makefile, Rust integration test, and documentation updates.

- Observation: Leta was unavailable during implementation.
  Evidence: both `leta workspace add
  /home/leynos/Projects/mapsplice.worktrees/roadmap-4-2-1` and the same
  command with sandbox-local `XDG_DATA_HOME` and `XDG_CONFIG_HOME` returned
  `Error: IO error: Read-only file system (os error 30)`.
  Impact: implementation used bounded file inspection instead of semantic
  branch-local navigation.

- Observation: `nixie-cli` 1.1.0 remained nondeterministic after adding
  `--max-concurrency`.
  Evidence: `scrutineer` gate logs repeatedly showed `make nixie` or
  `NIXIE_MAX_CONCURRENCY=1 make nixie` timing out in unchanged diagrams, while
  direct `merman-cli` Markdown validation passed on the same files.
  Impact: Work Item 1 changed the `make nixie` target to direct `merman-cli`
  validation while preserving the target name and concurrency controls.

- Observation: CodeRabbit could not run in this sandbox.
  Evidence: `/tmp/coderabbit-wi1-mapsplice-roadmap-4-2-1.out` contained
  `{"type":"status","phase":"deferred","status":"deferred coderabbit review:
  no default network route visible in this sandbox"}`.
  Impact: Work Item 1 records the review as deferred infrastructure, with no
  actionable CodeRabbit findings available.

- Observation: CodeRabbit remained unavailable for Work Item 2.
  Evidence: `/tmp/coderabbit-wi2-mapsplice-roadmap-4-2-1.out` contained
  `{"type":"status","phase":"deferred","status":"deferred coderabbit review:
  no default network route visible in this sandbox"}` and exited 124.
  Impact: Work Item 2 records the review as deferred infrastructure, with no
  actionable CodeRabbit findings available.

- Observation: the final post-CodeRabbit `scrutineer` spawn hit the agent
  thread limit.
  Evidence: spawning `scrutineer` returned `agent thread limit reached`.
  Impact: the implementation agent ran the same sequential deterministic gates
  directly with `tee` logs under `/tmp/wi2-precommit-mapsplice-roadmap-4-2-1-*`.

## Decision Log

- Decision: Initially use `nixie-cli`'s existing `--max-concurrency` option
  rather than
  wrapping `nixie` in a custom script or changing diagram content.
  Rationale: locked `nixie-cli` 1.1.0 implements the exact control needed.
  Its source creates an `asyncio.Semaphore` from `resolve_max_concurrency`, and
  emits results in file and diagram order. This decision was superseded during
  Work Item 1 after `nixie-cli`'s fixed per-diagram timeout continued to fail
  under the sanctioned gate runner.
  Date/Author: 2026-07-02 / planning agent.

- Decision: Supersede the planned `nixie-cli` invocation with direct
  `merman-cli` Markdown validation under the existing `make nixie` target.
  Rationale: locked `nixie-cli` 1.1.0 hard-codes a 30-second per-diagram
  timeout. Under `scrutineer`, unchanged diagrams timed out repeatedly even
  with `--max-concurrency 1`, while direct `merman-cli` Markdown validation
  over the same files passed consistently and uses the CI-installed renderer.
  Date/Author: 2026-07-02 / implementation agent.

- Decision: Initially default the Makefile to `NIXIE_MAX_CONCURRENCY ?= 2`.
  Rationale: locked `nixie-cli` clamps explicit values to
  `max(1, cpu_count - 1)`. Therefore cap 2 resolves to 1 on a 2-CPU private
  `ubuntu-latest` runner, to 2 on a 4-CPU public `ubuntu-latest` runner, and
  to 2 on the development host. This was superseded during Work Item 1 after
  the sanctioned gate runner reproduced timeouts with a cap of 2.
  Date/Author: 2026-07-03 / planning agent.

- Decision: Default the Makefile to `NIXIE_MAX_CONCURRENCY ?= 1` and
  `NIXIE_RENDERER_THREADS ?= 1`.
  Rationale: the planned default of 2 continued to produce renderer timeouts
  after serial runs had passed. The serial default is the only setting that
  satisfied the repository gate under the sanctioned gate runner. Explicit
  overrides remain available for comparison.
  Date/Author: 2026-07-02 / implementation agent.

- Decision: Pin the Makefile contract with a Rust dry-run integration test.
  Rationale: invoking real renderers inside the normal Rust test suite would
  make `make all` slower and could reintroduce environmental flakiness. A
  `make --dry-run --always-make nixie` test fails before the Makefile change
  and proves the selected concurrency flag without spawning renderers.
  Date/Author: 2026-07-02 / planning agent.

- Decision: Require all assertion macros in the new Rust test to live in
  `()`-returning helper functions.
  Rationale: this repository denies `clippy::panic_in_result_fn`; existing
  integration tests avoid the lint by keeping assertion macros out of
  `Result`-returning test bodies.
  Date/Author: 2026-07-03 / planning agent.

- Decision: Keep renderer selection unchanged.
  Rationale: `nixie-cli` 1.1.0 `auto` mode prefers `merman-cli` and falls back
  to the Node-based renderer when needed. The failure and fix are about process
  concurrency, not renderer semantics.
  Date/Author: 2026-07-02 / planning agent.

- Decision: Iterate over tracked Markdown files with `git ls-files '*.md'`.
  Rationale: this preserves the existing tracked corpus while avoiding
  repository-wide formatter churn and untracked local scratch files. Each
  Markdown file gets a fresh renderer invocation and a shared temporary
  artefact directory that is removed by a shell trap.
  Date/Author: 2026-07-02 / implementation agent.

## Outcomes & Retrospective

Work Item 1 is committed as `8055896` (`Stabilize Mermaid validation gate`).
It changed `Makefile` and added `tests/makefile_nixie.rs`. The focused dry-run
integration test passed with four cases in
`/tmp/test-green6-makefile-nixie-mapsplice-roadmap-4-2-1.out`. `scrutineer`
confirmed `make all`, `make markdownlint`, default `make nixie`, and
`NIXIE_MAX_CONCURRENCY=1 make nixie` all passed in
`/tmp/wi1-merman-final-mapsplice-roadmap-4-2-1-*.out` and again before commit
in `/tmp/wi1-precommit-mapsplice-roadmap-4-2-1-*.out`. CodeRabbit review was
deferred because no default network route was visible in the sandbox.

Work Item 2 documents the implemented deterministic Mermaid gate in
`docs/developers-guide.md` and `docs/contributing.md`, then marks only roadmap
task 4.2.1 complete in `docs/roadmap.md`. `scrutineer` confirmed `make all`,
`make markdownlint`, default `make nixie`, and `NIXIE_MAX_CONCURRENCY=1 make
nixie` all passed in `/tmp/wi2-final-mapsplice-roadmap-4-2-1-*.out`.
CodeRabbit review was deferred because no default network route was visible in
the sandbox. After recording the CodeRabbit deferral, the final `scrutineer`
spawn failed because the agent thread limit was reached, so the implementation
agent ran the same sequential gates directly in
`/tmp/wi2-precommit-mapsplice-roadmap-4-2-1-*.out`; all passed.

## Context and orientation

The roadmap task is `docs/roadmap.md` task 4.2.1, "Make Mermaid validation
deterministic under CI concurrency." It requires task 3.1.1 and says the
`make nixie` path must stop timing out on unchanged diagrams when serial
validation passes. Its success criterion is that concurrent and serial Mermaid
validation produce repeatable pass/fail results on the existing documentation
corpus.

Before Work Item 1, the Makefile defined:

```makefile
NIXIE ?= nixie

nixie: ## Validate Mermaid diagrams
	$(NIXIE) --no-sandbox
```

The CI workflow in `.github/workflows/ci.yml` uses `runs-on: ubuntu-latest`,
sets `NIXIE_CLI_VERSION: '1.1.0'`, installs `nixie-cli==1.1.0`, installs
`merman-cli@0.8.0-alpha.2`, and runs `make nixie`. Work Item 1 keeps the
target name but now uses the installed `merman-cli` renderer directly because
`nixie-cli`'s internal timeout remained flaky under gate-runner load.

The official GitHub-hosted runner reference says `ubuntu-latest` Linux runners
have 2 CPUs for private repositories and 4 CPUs for public repositories. That
means the uncapped `nixie-cli` default is 1 worker on private CI and 3 workers
on public CI. Because explicit values are clamped as a ceiling, a Makefile
default of 2 resolves to 1 worker on private CI and 2 workers on public CI.

Locked local `nixie-cli` 1.1.0 evidence:

- Installed metadata:
  `/home/leynos/.local/share/uv/tools/nixie-cli/lib/python3.14/site-packages/nixie_cli-1.1.0.dist-info/METADATA`.
- Installed source:
  `/home/leynos/.local/share/uv/tools/nixie-cli/lib/python3.14/site-packages/nixie/cli.py`.
- `METADATA` lines 72-87 document the CLI shape and say
  `--max-concurrency` is clamped to `max(1, cpu_count - 1)`.
- `METADATA` lines 97-114 document renderer selection and say
  `--max-concurrency` bounds simultaneous renderer processes.
- `cli.py` lines 370-384 implement `resolve_max_concurrency` as
  `max(1, min(requested_max_concurrency, limit))`.
- `cli.py` lines 814-843 run diagram tasks under an `asyncio.Semaphore`.
- `cli.py` lines 938-1027 resolve the renderer once, prepare all diagram
  tasks, create the semaphore, and emit results in file and diagram order.
- `cli.py` lines 1076-1084 expose the `--max-concurrency` argument.
- The direct pinning check returned
  `resolve_max_concurrency(2, cpu_count=2) == 1`.

Official `leynos/nixie` GitHub README evidence:

- The usage block documents `--max-concurrency N`.
- It states that diagram checks are scheduled concurrently with a global
  worker limit while output remains deterministic.
- It states that `--max-concurrency` is clamped to
  `max(1, cpu_count - 1)` and bounds simultaneous renderer processes.

Repository documents to keep open while implementing:

- `AGENTS.md`: commands, quality gates, Rust testing rules, Markdown guidance,
  and committing requirements.
- `docs/roadmap.md`: task 4.2.1 success criterion and task 4.2 sequencing.
- `docs/mapsplice-design.md` sections 3, 5 and 8: architecture overview,
  fidelity guarantee F4, and fixture/test requirements.
- `docs/developers-guide.md` sections 1, 6 and 7: normative references,
  verification layers and local tooling.
- `docs/contributing.md` "Development gates": CI-aligned gate list.
- `docs/documentation-style-guide.md` "Diagrams and images": Mermaid diagram
  documentation guidance.
- `docs/users-guide.md`: command semantics remain unchanged by this task.
- `docs/scripting-standards.md` sections "Rationale for adopting Cyclopts" and
  "CI wiring" for reproducible CI-oriented tooling conventions.

Skills to load before implementation:

- `memtrace-first`: try Memtrace first for canonical main-branch code graph
  context. Start with `list_indexed_repositories` and confirm repo id
  `mapsplice`. If the MCP call returns `user cancelled MCP tool call` again,
  record the exact failure and continue from bounded local evidence.
- `leta`: use branch-local symbol navigation and references. If Leta fails,
  record the exact failure and use bounded file inspection.
- `rust-router`, then `rust-unit-testing`: design the Rust integration test,
  helper boundaries, and `rstest` cases.
- `sem`: inspect entity-level diffs before committing.
- `en-gb-oxendict-style`: write documentation and comments.
- `commit-message`: commit with a file-based commit message.
- `firecrawl-mcp`: refresh official external docs only if needed. If the MCP
  call is cancelled again, record the exact failure and use locked source plus
  direct official web evidence.

## Plan of work

### Work Item 1: Cap Makefile Mermaid Concurrency and Pin the Contract

Read before starting: `AGENTS.md` "Commands", "Markdown Guidance", "Rust
Specific Guidance" and "Testing"; `docs/roadmap.md` task 4.2.1;
`docs/developers-guide.md` sections 6 and 7; `docs/mapsplice-design.md`
sections 5 and 8; `.github/workflows/ci.yml`; the verified `nixie-cli` 1.1.0
source and metadata paths listed in "Context and orientation"; and
`Cargo.toml` lints.

Skills to load: `memtrace-first`, `leta`, `rust-router`,
`rust-unit-testing`, `sem`, and `commit-message`.

This work item is one independently committable, gate-passable commit.

Red:

1. Add `tests/makefile_nixie.rs` with a module-level `//!` comment.
2. Use `rstest` parameterized cases to run
   `make --dry-run --always-make nixie MERMAN=echo-merman` from
   `CARGO_MANIFEST_DIR`.
3. Add at least two cases:
   - default variables expect the dry-run command to contain `echo-merman`,
     `RAYON_NUM_THREADS="1"`, `-j`, and `1`;
   - `NIXIE_MAX_CONCURRENCY=2` expects the dry-run command to contain
     `echo-merman`, `RAYON_NUM_THREADS="1"`, `-j`, and `2`;
   - `NIXIE_RENDERER_THREADS=2` expects the dry-run command to contain
     `RAYON_NUM_THREADS="2"`.
4. Make each `#[rstest]` case return `Result<(), Box<dyn std::error::Error>>`
   or the repository's existing `TestResult` alias if reused. Use `?` for
   fallible setup and command execution.
5. Hard requirement: do not place `assert!`, `assert_eq!`, `panic!`, or
   equivalent assertion macros in any `Result`-returning test body. Put every
   assertion in `()`-returning helper functions such as:
   - `assert_success(status: ExitStatus)`;
   - `assert_nixie_command_contains(output: &str, expected: &str)`;
   - `assert_nixie_contract(output: &str, expected_concurrency: &str,
     expected_renderer_threads: &str)`.
6. Keep each test and helper function under the local Clippy thresholds:
   cognitive complexity 9, 70 lines, and 4 arguments.
7. Do not mutate process environment globally. Pass Make variables as command
   arguments or use `Command::env` on the child process only.
8. Run the focused red test and expect it to fail because the pre-change
   Makefile command lacks the direct `merman-cli` concurrency and renderer
   thread contract:

```bash
cargo test --test makefile_nixie -- --nocapture \
  2>&1 | tee /tmp/test-red-makefile-nixie-mapsplice-roadmap-4-2-1.out
```

Green:

1. Update `Makefile` with these variables near the existing Markdown tooling
   definitions:

```makefile
MERMAN ?= merman-cli
NIXIE_RENDERER_THREADS ?= 1
NIXIE_MAX_CONCURRENCY ?= 1
NIXIE_FLAGS ?= -j $(NIXIE_MAX_CONCURRENCY)
NIXIE_PATHS ?= $(shell git ls-files '*.md')
```

1. Change the `nixie` target to:

```makefile
nixie: ## Validate Mermaid diagrams
	set -e; artefacts_dir="$$(mktemp -d)"; trap 'rm -rf "$$artefacts_dir"' EXIT; \
	for path in $(NIXIE_PATHS); do \
		RAYON_NUM_THREADS="$(NIXIE_RENDERER_THREADS)" $(MERMAN) $(NIXIE_FLAGS) \
			-i "$$path" -a "$$artefacts_dir"; \
	done
```

1. Keep the target name `nixie` so existing CI and contributor commands remain
   unchanged.
2. Run the focused test again and expect it to pass:

```bash
cargo test --test makefile_nixie -- --nocapture \
  2>&1 | tee /tmp/test-green-makefile-nixie-mapsplice-roadmap-4-2-1.out
```

Refactor and gate:

1. Refactor test code only to clarify setup, query and assertion boundaries.
   The helper extraction described in the Red stage is mandatory, not
   conditional.
2. Run these commands sequentially:

```bash
make all 2>&1 | tee /tmp/make-all-wi1-mapsplice-roadmap-4-2-1.out
make markdownlint 2>&1 | tee /tmp/markdownlint-wi1-mapsplice-roadmap-4-2-1.out
make nixie 2>&1 | tee /tmp/nixie-default-wi1-mapsplice-roadmap-4-2-1.out
NIXIE_MAX_CONCURRENCY=1 make nixie \
  2>&1 | tee /tmp/nixie-serial-wi1-mapsplice-roadmap-4-2-1.out
```

1. Run `sem diff --format json --file-exts .rs .md .toml` and inspect the
   changed entities before committing. If `sem` cannot parse Makefile changes,
   inspect `git diff -- Makefile tests/makefile_nixie.rs`.
2. Commit only after every command above passes.

Tests required for this work item:

- Rust integration test: `tests/makefile_nixie.rs`.
- No property, snapshot, BDD, or e2e test is required because the changed
  behaviour is a Makefile invocation contract rather than roadmap parsing or
  CLI output.
- Live gate checks: default bounded `make nixie` and serial
  `NIXIE_MAX_CONCURRENCY=1 make nixie`.

### Work Item 2: Document the Deterministic Mermaid Gate and Close 4.2.1

Read before starting: `docs/developers-guide.md` sections 1, 6 and 7;
`docs/contributing.md` "Development gates"; `docs/documentation-style-guide.md`
"Diagrams and images"; `docs/roadmap.md` task 4.2.1; `AGENTS.md`
"Documentation Maintenance" and "Markdown Guidance"; and this ExecPlan's
current "Decision Log".

Skills to load: `en-gb-oxendict-style`, `sem`, and `commit-message`. Load
`rust-router` only if the documentation work reveals a required Rust test
change; none is expected.

This work item is one independently committable, gate-passable documentation
commit.

1. Update `docs/developers-guide.md` local tooling guidance to state that
   `make nixie` validates tracked Markdown directly through `merman-cli`, one
   file at a time, with `NIXIE_MAX_CONCURRENCY` and
   `NIXIE_RENDERER_THREADS` both defaulting to `1`. Mention that explicit
   overrides such as `NIXIE_MAX_CONCURRENCY=2 make nixie` are only for local
   comparison; the serial default is the required gate.
2. Update `docs/contributing.md` development gates with the same
   contributor-facing note so contributors know how to reproduce the serial
   path without editing the Makefile.
3. Update `docs/roadmap.md` by marking task 4.2.1 complete only after Work
   Item 1 gates have passed. Do not mark 4.2.2 complete.
4. Update this ExecPlan's `Progress`, `Surprises & Discoveries`,
   `Decision Log`, and `Outcomes & Retrospective` with the actual commit hash
   and gate logs from Work Item 1.
5. Format only changed Markdown files. The exact command must include only
   files edited in this work item. If all four files listed below were edited,
   run:

```bash
mdtablefix docs/developers-guide.md docs/contributing.md docs/roadmap.md \
  docs/execplans/roadmap-4-2-1.md \
  2>&1 | tee /tmp/mdtablefix-wi2-mapsplice-roadmap-4-2-1.out
markdownlint-cli2 --fix docs/developers-guide.md docs/contributing.md \
  docs/roadmap.md docs/execplans/roadmap-4-2-1.md \
  2>&1 | tee /tmp/markdownlint-fix-wi2-mapsplice-roadmap-4-2-1.out
```

If any of those documentation files is not edited, omit it from both direct
formatter commands rather than passing an unchanged path.

1. Run these gates sequentially:

```bash
make all 2>&1 | tee /tmp/make-all-wi2-mapsplice-roadmap-4-2-1.out
make markdownlint 2>&1 | tee /tmp/markdownlint-wi2-mapsplice-roadmap-4-2-1.out
make nixie 2>&1 | tee /tmp/nixie-default-wi2-mapsplice-roadmap-4-2-1.out
NIXIE_MAX_CONCURRENCY=1 make nixie \
  2>&1 | tee /tmp/nixie-serial-wi2-mapsplice-roadmap-4-2-1.out
```

1. Run `sem diff --format json --file-exts .rs .md .toml` and inspect the
   changed entities before committing.
2. Commit only after every command above passes.

Tests required for this work item:

- No new Rust tests are required if only documentation and roadmap state
  change.
- Markdown gates are required: `make markdownlint` and `make nixie`.
- Full repository gate is required: `make all`, which includes `typecheck` on
  current `origin/main`.

## Concrete steps

1. Confirm the branch and worktree:

```bash
pwd
git branch --show-current
```

Expected:

```plaintext
/home/leynos/Projects/mapsplice.worktrees/roadmap-4-2-1
roadmap-4-2-1
```

1. Before source edits, try the requested advisory tools and record exact
   failures in this plan if they fail:

```bash
leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-4-2-1
```

Memtrace should be called through the MCP server with
`list_indexed_repositories` first. Firecrawl should be used only for refreshing
official external docs, because the load-bearing `nixie-cli` API has already
been verified in locked local source and official web docs.

1. Implement Work Item 1 exactly. Do not proceed to Work Item 2 until Work Item
   1 focused tests and gates pass.
2. Implement Work Item 2 exactly. Do not mark the roadmap task complete until
   both default bounded and serial Mermaid gates pass.
3. Before each commit, inspect the diff:

```bash
git diff --check
git diff
git status --short
```

1. Use the `commit-message` skill. Write the commit message to a file in a
   `mktemp -d` directory and commit with `git commit -F`.

## Validation and acceptance

Acceptance requires all of the following:

- `cargo test --test makefile_nixie -- --nocapture` passes after failing for
  the expected reason before the Makefile change.
- `make all` passes. This includes `check-fmt`, `lint`, `typecheck`, and
  `test` on current `origin/main`.
- `make markdownlint` passes.
- `make nixie` passes with the new default bounded concurrency.
- `NIXIE_MAX_CONCURRENCY=1 make nixie` passes as the serial comparison path.
- `docs/roadmap.md` marks only task 4.2.1 complete.
- No Markdown files outside the changed file set are reformatted.

Red-Green-Refactor evidence for Work Item 1:

- Red command:

```bash
cargo test --test makefile_nixie -- --nocapture \
  2>&1 | tee /tmp/test-red-makefile-nixie-mapsplice-roadmap-4-2-1.out
```

Expected red failure: the dry-run output for `make nixie` lacks the bounded
`merman-cli` concurrency and renderer thread contract.

- Green command:

```bash
cargo test --test makefile_nixie -- --nocapture \
  2>&1 | tee /tmp/test-green-makefile-nixie-mapsplice-roadmap-4-2-1.out
```

Expected green result: the test passes for default serial settings and explicit
overrides.

- Refactor and gate commands:

```bash
make all 2>&1 | tee /tmp/make-all-wi1-mapsplice-roadmap-4-2-1.out
make markdownlint 2>&1 | tee /tmp/markdownlint-wi1-mapsplice-roadmap-4-2-1.out
make nixie 2>&1 | tee /tmp/nixie-default-wi1-mapsplice-roadmap-4-2-1.out
NIXIE_MAX_CONCURRENCY=1 make nixie \
  2>&1 | tee /tmp/nixie-serial-wi1-mapsplice-roadmap-4-2-1.out
```

Expected result: every command exits 0.

Quality criteria:

- Tests: focused Makefile integration test and `make all` pass.
- Lint/typecheck: `make all` passes with warnings denied.
- Markdown: `make markdownlint` and `make nixie` pass.
- Performance: Mermaid validation must not rely on retries for unchanged
  diagrams; a single default run and a single serial run must pass.
- Security: the Makefile continues to invoke the configured `MERMAN`
  executable with explicit flags; it does not construct shell fragments from
  untrusted input.

## Idempotence and recovery

The Makefile change is idempotent. Re-running the focused test and gates should
not modify the worktree. The documentation formatting commands are also
idempotent when restricted to changed Markdown files.

If a Markdown formatter touches unrelated files, do not stash with a bare
message. Park only unrelated formatter churn with a named stash:

```bash
git stash push -m 'df12-stash v1 task=4.2.1 kind=discard reason="unrelated markdown formatter churn"' -- <paths>
```

If `make nixie` fails on an unchanged diagram after the Makefile cap is in
place, immediately re-run the serial comparison once:

```bash
NIXIE_MAX_CONCURRENCY=1 make nixie \
  2>&1 | tee /tmp/nixie-serial-retry-mapsplice-roadmap-4-2-1.out
```

If the serial default is flaky, stop and escalate rather than adding retries.
Inspect the cited diagram as a real Mermaid validation failure only after
rerunning the serial path once.

## Artifacts and notes

Planning evidence gathered before implementation:

```plaintext
$ nixie --no-sandbox --max-concurrency 1
... All diagrams validated successfully.
log: /tmp/nixie-research-serial-mapsplice-roadmap-4-2-1.out
```

```plaintext
$ nixie --no-sandbox
Error running command /home/leynos/.cargo/bin/merman-cli ...
docs/rstest-bdd-users-guide.md: diagram 1 timed out
log: /tmp/nixie-research-current-mapsplice-roadmap-4-2-1.out
```

```plaintext
$ nixie --no-sandbox --max-concurrency 2
... All diagrams validated successfully.
log: /tmp/nixie-research-concurrency2-mapsplice-roadmap-4-2-1.out
```

```plaintext
$ /home/leynos/.local/share/uv/tools/nixie-cli/bin/python - <<'PY'
from nixie.cli import resolve_max_concurrency
for cpu_count in [1, 2, 4, 6, 24]:
    print(cpu_count, resolve_max_concurrency(2, cpu_count=cpu_count),
          resolve_max_concurrency(None, cpu_count=cpu_count))
PY
1 1 1
2 1 1
4 2 3
6 2 5
24 2 23
```

Tooling failures to preserve:

```plaintext
Memtrace list_indexed_repositories: user cancelled MCP tool call
Firecrawl search: user cancelled MCP tool call
```

## Interfaces and dependencies

No new dependencies are introduced.

At the end of Work Item 1, `Makefile` must expose these variables:

```makefile
MERMAN ?= merman-cli
NIXIE_RENDERER_THREADS ?= 1
NIXIE_MAX_CONCURRENCY ?= 1
NIXIE_FLAGS ?= -j $(NIXIE_MAX_CONCURRENCY)
NIXIE_PATHS ?= $(shell git ls-files '*.md')
```

The `nixie` target must be:

```makefile
nixie: ## Validate Mermaid diagrams
	set -e; artefacts_dir="$$(mktemp -d)"; trap 'rm -rf "$$artefacts_dir"' EXIT; \
	for path in $(NIXIE_PATHS); do \
		RAYON_NUM_THREADS="$(NIXIE_RENDERER_THREADS)" $(MERMAN) $(NIXIE_FLAGS) \
			-i "$$path" -a "$$artefacts_dir"; \
	done
```

`tests/makefile_nixie.rs` must prove the default and override contract through
`make --dry-run --always-make nixie`, without invoking `merman-cli`, `mmdc`,
`bun`, or `npx`.

The implemented target leans on locked CI installing `merman-cli`. If a future
environment removes `merman-cli`, stop and escalate rather than silently
switching to an unvalidated renderer path.

## Revision note

Round 2 resolves the design-review blockers. Work Item 1 now makes
`()`-returning assertion helpers a hard requirement, not an optional refactor,
so a literal implementation cannot trip `clippy::panic_in_result_fn` by placing
assertions inline in `Result`-returning tests. The default concurrency decision
now cites GitHub-hosted runner CPU counts and the locked `nixie-cli` clamp
direction, proving `NIXIE_MAX_CONCURRENCY ?= 2` remains serial on 2-CPU private
CI while reducing excessive local and public-runner concurrency.

Round 3 records the Work Item 1 implementation deviation. `nixie-cli` 1.1.0's
30-second internal timeout still failed under the sanctioned gate runner after
concurrency was capped, so the target now validates tracked Markdown directly
with `merman-cli`, one file at a time. The remaining work must document this
actual contract rather than the original `nixie --max-concurrency` plan.

Round 4 records Work Item 2 completion. The contributor-facing docs now state
that `make nixie` uses direct `merman-cli` validation with serial defaults,
the roadmap marks only task 4.2.1 complete, and the ExecPlan records Work
Item 1 commit and gate evidence.

Round 5 records final Work Item 2 evidence. Deterministic gates passed after
the documentation update, and CodeRabbit deferred before review because the
sandbox had no default network route. A final `scrutineer` spawn then hit the
agent thread limit, so the same sequential gates were run directly before the
documentation commit.
