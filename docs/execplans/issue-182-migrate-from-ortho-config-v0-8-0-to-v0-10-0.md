# Migrate from ortho_config v0.8.0 to v0.10.0

This ExecPlan is a living document. The sections `Constraints`, `Tolerances`,
`Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: IN PROGRESS

## Purpose / big picture

`mapsplice` must be demonstrably migrated from `ortho_config` v0.8.0 to the
published v0.10.0 release. The dependency version was already raised by a
Dependabot change, so the remaining work is to prove that the repository
actually behaves as a v0.10.0 consumer, to isolate configuration tests from
process-global state, and to record the authoritative upstream documentation
with byte-exact provenance.

A reader can observe success in four ways:

1. `make check-fmt`, `make lint`, `make test`, and the remaining repository
   gates pass with no suppressed warnings, ignores, or `--ignore-rust-version`
   flags.
2. Configuration tests resolve every precedence tier through injected sources,
   so they neither mutate the process environment nor depend on the host's
   configuration files.
3. The imported upstream guides are byte-identical to the files at tag
   `v0.10.0`, and their repository/tag/commit/checksum provenance is recorded.
4. Every change named by the v0.10.0 migration guide is marked applicable or
   inapplicable with a concrete reason, and every applicable item has evidence.

## Context and orientation

The repository is a Rust 2024 workspace with one package, `mapsplice`, whose
library entry points are `src/lib.rs` and whose binary entry point is
`src/main.rs`. Configuration is loaded in two places:

- `src/cli_config.rs` hand-rolls discovery and parsing for the global
  `in_place` setting. It reads `$XDG_CONFIG_HOME/mapsplice/config.toml`, then
  `./.mapsplice.toml`, then `MAPSPLICE_IN_PLACE`, and finally lets `--in-place`
  / `-i` force `true`.
- `src/cli.rs` derives `OrthoConfig` for `GlobalCli` and for the insert
  subcommand's `InsertConfig`, and calls
  `ortho_config::load_and_merge_subcommand_for` to merge the insert `after`
  setting.

The relevant upstream artefacts are imported into `docs/`:

- `docs/ortho-config-users-guide.md`
- `docs/ortho-config-v0-9-0-migration-guide.md`
- `docs/ortho-config-v0-10-0-migration-guide.md`

A provenance record beside them names the upstream repository, tag, tag object,
commit, original paths, destination paths, SHA-256 checksums, and the mapping
for relative links that cannot resolve identically in this repository.

The baseline already declared `ortho_config = "0.10.0"` before this plan
started, because Dependabot change `2c4ac33`
(`Bump ortho_config from 0.8.0 to 0.10.0 (#199)`) raised the requirement. The
lockfile contains exactly one `ortho_config` 0.10.0, one `ortho_config_macros`
0.10.0, and one `ortho_l10n` 0.10.0, with no Git dependencies. The migration
therefore cannot be discharged by another version edit; it must be discharged
by behavioural, testing, and documentation evidence.

The toolchain is dated nightly `nightly-2026-11-13`, reported as
`rustc 1.102.0-nightly (8eb778d04 2026-11-12)`. `ortho_config` v0.10.0 has a
floor of Rust 1.89.0, which this toolchain satisfies. The manifest declares no
`rust-version` key, so no MSRV change is required; the dated nightly pin is
also already sufficient.

Configuration test fixtures live in `tests/support/config.rs` and are used only
by `tests/roadmap_config.rs` and `tests/roadmap_config/in_place.rs`. Those
fixtures currently mutate process environment variables and the working
directory through `temp_env` and `ProcessState`. Binary-level contract tests in
`tests/cli_process_contract.rs` already use isolated child processes with
controlled environments; they must stay that way.

## Constraints

- Do not downgrade `ortho_config`; the baseline is already v0.10.0.
- Keep the `serde_json` feature enabled. `ortho_config` v0.10.0 does not build
  with `default-features = false` (leynos/ortho-config#627).
- Do not depend on roadmap items. Per-field localized defaults and linting
  command verbs are not shipped in v0.10.0.
- Preserve configuration precedence: defaults < configuration files <
  environment < command-line flags.
- Preserve discovery order on Unix: `HOME` dotfiles, XDG bases, then the
  explicit local base.
- Preserve exit-status and stream contracts: Clap display on the requested
  stream, failures on standard error with a failing status, broken standard
  output treated as success where already specified.
- Do not introduce new process-environment mutation in tests. Do not invent
  injection APIs that v0.10.0 does not provide.
- Do not suppress warnings, add lint ignores, or pass `--ignore-rust-version`.
- Imported upstream text stays byte-for-byte intact. Local migration notes live
  outside the imported copies.
- Do not merge the pull request or publish a release.
- Commit after each coherent change and run the applicable gates before each
  commit.

## Tolerances (exception triggers)

- If any required gate fails for a reason not attributable to this change, stop
  and report the failing gate and its log path rather than editing unrelated
  code.
- If the injected-source API cannot express an existing discovery behaviour
  without changing user-visible semantics, stop and record a proposed deviation
  in the decision log before implementing it.
- If a checksum of an imported guide differs from the value recorded at import
  time, treat the import as corrupt and re-import from the pinned commit.
- If compiler admission is unhealthy (supervisor unreachable, `manager` or
  `stopped_populated` fault, or an unrecovered live-work fault), report the full
  `build-limits-status` output and continue only with non-Rust work.
- Scope budget: production code changes are expected to stay below roughly 200
  net lines, test-fixture changes below roughly 300 net lines, and
  documentation changes below roughly 200 net lines excluding imported upstream
  copies and this ExecPlan. Exceeding a budget by more than 50% requires a
  decision-log entry explaining why.

## Risks

- **Hidden process-environment dependence.** A test that still calls
  `run_from_args` could read the host's real environment or configuration
  files. Mitigation: every touched configuration test builds explicit
  `ConfigSources`; the sources helper names `HOME`, `XDG_CONFIG_HOME`, and
  `XDG_CONFIG_DIRS` so `/etc/xdg` is never reached.
- **Behaviour drift in global discovery.** Replacing `env::var` with an injected
  lookup could change non-Unicode handling or diagnostic text. Mitigation: keep
  the existing error messages and add regression tests for invalid values.
- **Divergent link resolution in imported guides.** Upstream links name sibling
  files that do not exist under the same names here. Mitigation: leave the text
  untouched and record the mapping in the provenance document.
- **Idempotence of source injection.** `MapEnv` implements both `EnvSource` and
  `ScanEnvSource`; cloning one `Arc` into both roles must not change
  precedence. Mitigation: precedence tests cover each tier.
- **Gate queueing under shared compiler admission.** Rust gates may queue behind
  other agents. Mitigation: submit the gate once and wait; never start a second
  build or bypass admission.

## Progress

- [x] (2026-11-14) Confirm branch
  `issue-182-migrate-from-ortho-config-v0-8-0-to-v0-10-0` and clean worktree.
- [x] (2026-11-14) Verify tag `v0.10.0` resolves to tag object
  `996e5a1c9f22ace4f55bf5dcc196d2c0524262bf` and commit
  `38812c563e3953a490e3efc202609f37e21a0517`; confirm the cached checkout is
  clean at that commit.
- [x] (2026-11-14) Read the v0.10.0 migration guide, the v0.9.0 migration guide,
  the relevant user-guide sections, and the v0.10.0 changelog.
- [x] (2026-11-14) Capture baseline: `cargo check --all-targets --all-features`
  exits 0; `cargo test --test roadmap_config --all-features` reports 15 passed.
- [x] (2026-11-14) Import the three upstream guides byte-for-byte and write the
  provenance record. `cmp` reports no difference for all three files, and
  `docs/ortho-config-doc-provenance.md` records the tag, commit, checksums, and
  relative-link mapping.
- [x] (2026-11-14) Add `ConfigSources` and the source-aware entry points without
  changing process-backed behaviour. `ConfigSources::process` remains the
  default for `parse_cli_request` and `run_from_args`.
- [x] (2026-11-14) Migrate the configuration tests off `ProcessState` and
  process environment mutation. The focused target now reports 17 passed, and
  `temp-env` is no longer a dependency.
- [x] (2026-11-14) Update repository documentation for the new test seam and the
  migration, including a new `--flag=false` binary-level regression test.
- [x] (2026-11-14) Run every applicable gate and record evidence under
  `/tmp/gate-*.out`: `make check-fmt`, `make lint`, `make test`,
  `make typecheck`, `make markdownlint`, `make nixie`, `make spelling`,
  `make interrogate`, and `make test-workflow-contracts` all pass.
- [x] (2026-11-14) Commit the documentation, implementation, and developer
  guide as three focused commits, push over SSH as `leynos`, refresh the
  tracking ref, and open draft pull request
  <https://github.com/leynos/mapsplice/pull/230>.
- [x] (2026-11-14) Rebase onto `origin/main` at `1a9644d`, which disables the
  Cranelift development selection and makes LLVM the estate standard. The
  rebase reported no conflicts, all four branch commits replayed unchanged under
  `range-diff`, and the Rust and documentation gates re-ran green.
- [x] (2026-11-14) Complete a CodeRabbit `review --agent` pass and clear its
  concerns. The retry reviewed all 16 changed files and raised one minor
  finding about the source-aware doctest using the crate root; both
  source-aware doctests now use a `tempfile` base and an absolute target inside
  it.

## Surprises & discoveries

- The dependency version edit had already landed. The useful migration work is
  behavioural and documentary, not a manifest bump.
- `mapsplice` writes its boolean flags by hand as `Option<bool>` with
  `ArgAction::SetTrue`. The v0.10.0 generated-boolean `--flag=false` change
  therefore does not alter this CLI; `--in-place=false` and `--after=false`
  remain invalid clap input.
- `mapsplice` uses no Fluent catalogues, no `AgentContext`, no `DocMetadata`,
  no `cargo-orthohelp`, no YAML feature, and no `ConfigDiscovery::load_first`
  call. Most of the v0.9.0 and v0.10.0 guides are consequently inapplicable.
- The old imported user guide was 55,990 bytes. The v0.10.0 guide is 80,278
  bytes, so replacing it is a substantial documentation refresh rather than a
  cosmetic update.
- `tests/support/unit.rs` duplicates a smaller set of workspace helpers with
  its own lock and working-directory guard. Those helpers are used by tests
  that exercise operations, not configuration discovery, so they are outside
  this migration's touched surface.
- Adding the source-aware parsing pushed `src/cli.rs` to 452 lines, past the
  repository's 400-line module limit enforced by Whitaker. The parsed request
  types (`CliRequest`, `CommandKind`, `GlobalOptions`) were extracted to
  `src/cli_request.rs` and re-exported through `cli`, which restores the limit
  without changing the public surface.
- `ortho-config` names the discovery and merge capabilities separately, and the
  Unix discovery order probes `HOME`, `XDG_CONFIG_HOME`, and `XDG_CONFIG_DIRS`.
  A test that names only `HOME` still reaches the host's `/etc/xdg` through the
  unset XDG variables, so `TestSources` names all three explicitly.
- `cargo fmt --workspace` is rejected by the pinned toolchain; the Makefile
  already selects `cargo fmt --all`, so only the raw command needed adjusting.
- CodeRabbit found that the `run_from_args_with_sources` doctest used the crate
  root as its base and a relative `roadmap.md` target. It passed only because
  no such file existed; a crate-root roadmap or `.mapsplice.toml` could have
  changed the result or been rewritten. Both source-aware doctests now use a
  `tempfile::tempdir()` base and an absolute target inside it.

## Decision log

- **Decision:** Treat the pre-existing v0.10.0 manifest entry as the starting
  point, not as completion. **Rationale:** the task requires implementation and
  validation of the migration, and a version number alone does not demonstrate
  v0.10.0 conformance.
- **Decision:** Add a small repository-owned `ConfigSources` value object
  rather than exposing raw `SharedEnvSource` and `SharedScanEnvSource` through
  every call. **Rationale:** the global loader needs a named discovery lookup
  and a local file base, while the subcommand loader needs a scanning source
  and an ortho `SubcommandFileContext`. One object keeps the two capabilities
  aligned and makes the process-backed default explicit.
- **Decision:** Keep `parse_cli_request` and `run_from_args` process-backed and
  add `_with_sources` variants. **Rationale:** the public API and binary
  behaviour must not change for ordinary callers; tests opt into isolation.
- **Decision:** Delete `ProcessState`, `CwdRestore`, and their self-tests rather
  than keep an unused fixture. **Rationale:** once configuration tests use
  injected sources, the helper has no consumer, and leaving it would advertise
  a testing pattern the repository no longer uses.
- **Decision:** Record upstream relative-link mappings in the provenance
  document instead of rewriting imported links. **Rationale:** the task
  requires byte-for-byte imported text; link rewriting would break that
  guarantee.
- **Decision:** Extract the parsed request types into `src/cli_request.rs`.
  **Rationale:** the source-aware entry points pushed `src/cli.rs` past the
  400-line module limit. The extraction is a mechanical move that keeps the
  existing re-exports, so no caller changes.
- **Decision:** Do not adopt opt-in profiles, `stack_scopes`,
  `cli_default_as_absent`, docs IR 2.0, Fluent identifier reshaping, or
  `cargo-orthohelp` changes. **Rationale:** `mapsplice` does not use the
  profile, generated-documentation, or localization surfaces those changes
  affect, and the task forbids adopting capabilities the repository context
  does not ask for.
- **Decision:** Pin the handwritten boolean flag behaviour with a binary-level
  test instead of changing the flags to the v0.10.0 generated form.
  **Rationale:** `--in-place` and `--after` are `Option<bool>` with
  `ArgAction::SetTrue`; adopting `--flag=false` would be a new CLI capability,
  not a migration requirement, and the test records the actual contract.
- **Decision:** Treat `origin/main`'s LLVM build-standard change as not
  pertinent to this migration and make no branch documentation changes for it.
  **Rationale:** the change concerns `.cargo/config.toml`, `AGENTS.md`, and the
  build-backend sections of the developer guide. This branch touches the
  configuration-source seam and its documentation; it makes no Cranelift or
  LLVM claim. The rebase itself is recorded because it moved the branch base.

## Conformance basis

This plan has no upstream technical-design revision or ADR set. Its governing
artefacts are:

- `docs/ortho-config-v0-10-0-migration-guide.md`, imported from
  `leynos/ortho-config` tag `v0.10.0`, commit
  `38812c563e3953a490e3efc202609f37e21a0517`.
- `docs/ortho-config-v0-9-0-migration-guide.md`, imported from the same commit.
- `docs/ortho-config-users-guide.md`, imported from the same commit.
- The v0.10.0 `CHANGELOG.md` at the same commit.
- `AGENTS.md`, `docs/developers-guide.md`, and `docs/users-guide.md` for
  repository conventions.

Trace links used by this plan:

- `EP-M1` (provenance import) → provenance document checksums and `cmp` against
  the pinned checkout.
- `EP-M2` (source seam) → `tests/roadmap_config.rs` precedence and unhappy-path
  tests.
- `EP-M3` (test isolation) → absence of `temp_env`, `ProcessState`, and
  `serial_test` in the touched configuration tests.
- `EP-M4` (documentation) → `docs/developers-guide.md` section 5 matches the
  implemented seam.
- `EP-M5` (validation) → gate logs under `/tmp` and the final test inventory.

## Applicability matrix

Every item in the v0.10.0 migration guide, plus the v0.9.0 items that matter
for a repository starting at v0.8.0.

| Guide item                                         | Applicability                 | Action and evidence                                                                                                                                        |
| -------------------------------------------------- | ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| v0.10.0 §1 dependency versions                     | Already applied               | `Cargo.toml` and `Cargo.lock` pin 0.10.0; verify no duplicates or Git sources.                                                                             |
| v0.10.0 §2 explicit CLI values win                 | Not applicable                | No OrthoConfig field restates a clap default; `after` and `in_place` are `Option<bool>` with no `default_value`. Existing CLI-over-environment test stays. |
| v0.10.0 §3 Fluent identifiers and IR 2.0           | Not applicable                | Repository generates and consumes no Fluent catalogues or documentation IR.                                                                                |
| v0.10.0 §4 `AgentContext` / `DocMetadata` literals | Not applicable                | Neither type is constructed or referenced.                                                                                                                 |
| v0.10.0 §5 boolean flag spellings                  | Not applicable, with evidence | Flags are handwritten `ArgAction::SetTrue`; add a regression assertion that `--in-place=false` is rejected.                                                |
| v0.10.0 §6 named profiles                          | Not adopted                   | No repository requirement; opting in would reserve new keys and flags.                                                                                     |
| v0.10.0 §6 stacked scopes                          | Not adopted                   | Existing first-wins discovery is the documented behaviour.                                                                                                 |
| v0.10.0 §6 injected environment sources            | Adopted for tests             | New `ConfigSources` seam and `_with_sources` entry points; configuration tests use `MapEnv` plus an explicit base.                                         |
| v0.10.0 §6 `cli_default_as_absent`                 | Not applicable                | No OrthoConfig field has a clap `default_value` to infer.                                                                                                  |
| v0.10.0 §6 behaviour metadata and policy           | Not applicable                | No `cargo-orthohelp` or agent-context consumption.                                                                                                         |
| v0.10.0 §6 Cargo external subcommand helper        | Not applicable                | `mapsplice` is not a Cargo external subcommand.                                                                                                            |
| v0.10.0 §6 standalone `ortho_l10n`                 | Not adopted directly          | It arrives transitively; the repository performs no Fluent lookup, so a direct dependency would add no value.                                              |
| v0.9.0 §1 dependency versions                      | Already applied               | Same manifest evidence as v0.10.0 §1.                                                                                                                      |
| v0.9.0 §2 `ConfigDiscovery::load_first` errors     | Not applicable                | No direct call; the subcommand loader owns discovery.                                                                                                      |
| v0.9.0 §3 YAML 1.2 and feature selection           | Not applicable                | The `yaml` feature is not enabled and only TOML is read.                                                                                                   |
| v0.9.0 §4 clearer inheritance errors               | Not applicable                | `extends` is not used and no error-text snapshot exists.                                                                                                   |
| v0.9.0 §5 hermetic discovery tests                 | Adopted                       | `MapEnv` replaces `temp_env`; `ConfigSources` names the local base explicitly.                                                                             |
| v0.9.0 §6 `discovery(...)` attribute               | Not adopted                   | Current default candidate names already match the shipped contract; changing them would alter discovery.                                                   |
| v0.9.0 §7 subcommand documentation derive          | Not applicable                | No documentation artefact is generated.                                                                                                                    |
| v0.9.0 §8 combined localization path               | Not applicable                | The CLI does not localize help or errors.                                                                                                                  |
| v0.9.0 §9 dependency aliases and re-exports        | Partly applicable             | The code already uses the `ortho_config::toml` re-export; no alias exists.                                                                                 |
| v0.9.0 §10 tracing and metrics                     | Not adopted                   | Existing tracing coverage is repository-owned and unaffected.                                                                                              |
| v0.9.0 §11 agent context                           | Not applicable                | No agent-context consumer.                                                                                                                                 |
| v0.9.0 §12 proposed `MissingRequiredValues`        | Respected                     | Code matches only repository-owned errors.                                                                                                                 |

## Plan of work

### Milestone EP-M1: authoritative documentation and provenance

Import `docs/users-guide.md`, `docs/v0-10-0-migration-guide.md`, and
`docs/v0-9-0-migration-guide.md` from commit
`38812c563e3953a490e3efc202609f37e21a0517` into the three destination files.
Verify each destination with `cmp` and compute SHA-256 checksums. Write a
provenance document that records the repository, tag, tag object, commit,
original path, destination path, checksum, and the relative-link mapping.

End state: the repository carries byte-exact upstream references and a reviewer
can reproduce the import.

### Milestone EP-M2: source-aware configuration seam

Introduce `ConfigSources` in `src/cli_config.rs`. It carries an explicit local
base, a lookup-only `SharedEnvSource` for discovery variables, and a
`SharedScanEnvSource` for the environment merge layer. `ConfigSources::process`
reproduces today's behaviour exactly.

Thread the sources through:

- `load_global_config(config, sources)`, which uses the discovery source for
  `XDG_CONFIG_HOME` and `MAPSPLICE_IN_PLACE`;
- `parse_cli_request_with_sources`, which passes an ortho
  `SubcommandFileContext` and merge source to
  `load_and_merge_subcommand_for_with_sources_at`;
- `run_from_args_with_sources`, which preserves failure metrics.

Keep `parse_cli_request` and `run_from_args` as process-backed wrappers.

End state: production behaviour is unchanged, and a caller can supply every
discovery and merge input explicitly.

### Milestone EP-M3: isolated configuration tests

Replace `ProcessState` in `tests/support/config.rs` with a `TestSources`
fixture that owns a `MapEnv` and the workspace base. Every test in
`tests/roadmap_config.rs` and `tests/roadmap_config/in_place.rs` constructs the
sources it needs, including `HOME`, `XDG_CONFIG_HOME`, and `XDG_CONFIG_DIRS`,
then calls `run_from_args_with_sources`.

Remove `temp-env` from the dev dependencies when no consumer remains. Remove
the two tests that only exercised the deleted helper. Add regression coverage
for:

- the command-line tier overriding an environment `false` for `in_place`;
- a malformed global configuration file;
- a boolean `--flag=false` spelling being rejected by clap.

End state: the touched configuration tests read no process environment and do
not take a global test lock.

### Milestone EP-M4: repository documentation

Update `docs/developers-guide.md` section 5 to describe the two configuration
owners in terms of `ConfigSources` and injected sources. Keep the existing
description of which loader owns which setting. Add local migration notes
outside the imported guides, either in the provenance document or in a short
`docs/migrations` record.

End state: a maintainer can add a configuration setting and a hermetic test
without rediscovering the seam.

### Milestone EP-M5: validation and delivery

Run the canonical gates sequentially, capture logs under `/tmp`, compare the
test inventory with the baseline, inspect the final dependency graph, commit,
push over SSH as `leynos`, and open a draft pull request. Do not merge.

## Concrete steps

1. Copy the three upstream files with `cp` from the pinned checkout, verify with
   `cmp -s`, and record `sha256sum` output.
2. Edit `src/cli_config.rs` to add `ConfigSources` and source-aware global
   loading.
3. Edit `src/cli.rs` to thread sources through parsing and subcommand merging.
4. Edit `src/lib.rs` to export the new seam and re-export the new entry points.
5. Rewrite `tests/support/config.rs` and the two configuration test files.
6. Update `docs/developers-guide.md` and the provenance document.
7. Run `make check-fmt`, `make lint`, `make test`, then the remaining gates.
8. Commit each milestone separately, then push and open the draft PR.

## Validation and acceptance

Acceptance is behavioural, not merely structural:

- `cargo test --test roadmap_config --all-features` passes and no test in that
  target references `temp_env`, `ProcessState`, or `serial_test`.
- A test proves every precedence tier: defaults, configuration files,
  environment, and command line, for both `in_place` and `after`.
- A test proves malformed configuration still produces
  `MapspliceError::Configuration`.
- `--in-place=false` and `--after=false` remain rejected, while the bare flags
  work.
- `make check-fmt`, `make lint`, `make test`, `make typecheck`,
  `make markdownlint`, `make nixie`, `make spelling`, `make interrogate`, and
  `make test-workflow-contracts` pass, or any unavailable gate is reported with
  its exact failure.
- `cargo tree -i ortho_config` shows one 0.10.0 source and no Git dependency.
- The imported guides compare byte-identical to the pinned checkout.

Expected focused output:

```console
$ cargo test --test roadmap_config --all-features
...
test result: ok. 17 passed; 0 failed
```

The baseline was 15 passed. Two of those were `ProcessState` self-tests that
were deleted with the fixture; four precedence and regression tests were added.

## Verification plan

Invariants:

- **I1 precedence.** For every setting and every adjacent pair of tiers, the
  higher tier wins. Verified by parameterized configuration tests that vary one
  tier at a time.
- **I2 isolation.** A test using `ConfigSources` reads only the supplied
  `MapEnv` and base. Verified by constructing closed environments that name
  `HOME`, `XDG_CONFIG_HOME`, and `XDG_CONFIG_DIRS`, and by the absence of
  `temp_env` in the touched tests.
- **I3 diagnostic preservation.** Malformed files still surface
  `MapspliceError::Configuration`. Verified by an invalid-value test at both
  the global and subcommand layers.
- **I4 byte-exact import.** Each imported file equals the upstream file at the
  pinned commit. Verified with `cmp` and SHA-256.
- **I5 dependency uniqueness.** One `ortho_config` version, no Git source.
  Verified with `cargo tree`.

Lemmas:

- **L1.** `ConfigSources::process()` supplies the same discovery variables and
  local base as the previous process-backed code. Verified by the unchanged
  focused configuration suite and by reading the process-backend tests that
  still pass.
- **L2.** The ortho `SubcommandFileContext` plus `SharedScanEnvSource` pair
  reaches both the file-discovery and environment-merge layers. Verified by a
  test in which a file supplies `after = true` and another in which
  `MAPSPLICE_CMDS_INSERT_AFTER` supplies it.

Axioms:

- `ortho_config` v0.10.0's documented `MapEnv`, `EnvSource`, `ScanEnvSource`,
  and `_with_sources_at` interfaces behave as documented. The repository does
  not verify their internals, but it does exercise the real interface through
  integration tests.
- Clap's `ArgAction::SetTrue` rejects `=false` for handwritten flags. Verified
  by a binary-level test rather than assumed.

Non-vacuity: each precedence test sets a distinguishable observable outcome
(insert before versus after; rewrite versus stdout). The malformed-file tests
assert the error variant, which is unreachable from a success path. The
`--flag=false` test asserts a non-zero status and a clap diagnostic, so it
fails if the parser ever starts accepting the spelling.

## Idempotence

Copying the imported guides is idempotent because the source commit is
immutable and the destination paths are fixed. Re-running the tests creates
fresh temporary workspaces and does not depend on prior runs. `make fmt` and
`make spelling` may regenerate tracked files; any regenerated file is inspected
and committed only when the change belongs to this work.

## Artefacts

- `docs/ortho-config-users-guide.md`
- `docs/ortho-config-v0-10-0-migration-guide.md`
- `docs/ortho-config-v0-9-0-migration-guide.md`
- `docs/ortho-config-doc-provenance.md`
- `src/cli_config.rs`, `src/cli.rs`, `src/lib.rs`
- `tests/support/config.rs`, `tests/roadmap_config.rs`,
  `tests/roadmap_config/in_place.rs`
- `docs/developers-guide.md`
- Gate logs under `/tmp/gate-*`.

## Interfaces

```rust
pub struct ConfigSources { /* private fields */ }

impl ConfigSources {
    pub fn process() -> Self;
    pub fn new(
        base: impl Into<Utf8PathBuf>,
        discovery: SharedEnvSource,
        merge: SharedScanEnvSource,
    ) -> Self;
}

pub fn parse_cli_request_with_sources<I, T>(
    args: I,
    sources: &ConfigSources,
) -> Result<CliRequest>;

pub fn run_from_args_with_sources<I, T>(
    args: I,
    sources: &ConfigSources,
) -> Result<RunOutcome>;
```

## Outcomes & retrospective

The migration delivered a source-aware configuration seam, a byte-exact import
of the authoritative v0.9.0 and v0.10.0 documentation, and a configuration test
suite that no longer mutates process state. The production entry points
(`parse_cli_request`, `run_from_args`) keep their process-backed behaviour and
signatures; the new `_with_sources` variants are additive.

Evidence recorded during delivery:

- `cargo check --all-targets --all-features` exited 0 before and after.
- `cargo test --test roadmap_config --all-features` moved from 15 passed to 17
  passed: two `ProcessState` self-tests removed, four precedence and regression
  tests added.
- `make test` reports 710 nextest tests passed with none skipped, plus 20
  doctests (17 passed, 3 ignored).
- `make check-fmt`, `make lint`, `make typecheck`, `make markdownlint`,
  `make nixie`, `make spelling`, `make interrogate` (100%, 444/444), and
  `make test-workflow-contracts` (314 passed) all exited 0.
- `cargo tree -i ortho_config` shows a single `ortho_config v0.10.0`;
  `ortho_l10n v0.10.0` arrives only through the facade and macros. `Cargo.lock`
  contains no Git source and exactly one `ortho_config` entry.

Lessons:

- The manifest bump was already present, so the real work was proving
  behavioural conformance and replacing the test isolation mechanism. Treating
  the dependency edit as completion would have skipped every observable
  requirement.
- The repository's module-length lint participates in `make lint` through
  Whitaker, not in `cargo check`. Splitting `cli.rs` was required even though
  the code compiled cleanly.
- Naming only one of the XDG variables in a test environment is not isolation:
  the Unix discovery order falls through to the host's `/etc/xdg` for any
  variable left unset.

All planned work is complete. Draft pull request
<https://github.com/leynos/mapsplice/pull/230> carries the rebased branch, the
CodeRabbit `review --agent` pass completed with its one finding fixed, and the
deterministic gates pass. No release is published and nothing is merged from
this plan; the draft remains open for human review.

## Revision note

Initial revision written on 2026-11-14 after reading the upstream v0.9.0 and
v0.10.0 migration guides and surveying the repository. No implementation edits
had been made when this revision was written.

Revised on 2026-11-14 after implementation and validation: Progress items,
Surprises, Decision log, expected focused count, and Outcomes were updated to
match the delivered tree and gate evidence. The delivery item remains open
until the draft pull request exists.

Revised again on 2026-11-14 after rebasing onto `origin/main` at `1a9644d`. The
rebase was conflict-free, every branch commit replayed unchanged, and the
Progress and Decision log now record the new base and the assessment that the
LLVM build-standard change is not pertinent to this migration.

Revised once more on 2026-11-14 after the CodeRabbit review: the isolated
doctest fix and the cleared review finding are recorded in Progress and
Surprises, and the Outcomes no longer describe review as outstanding.
