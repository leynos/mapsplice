# Mapsplice Rust baseline execution ledger

This record tracks the single delivery branch and its measured state. A passing
local gate applies only to the commit and live inputs named here; it is not
final acceptance or evidence of a hosted review.

## Git boundary and ownership

- Repository: `leynos/mapsplice`; delivery branch:
  `rust-baseline-hardening-mapsplice`.
- Local starting head: `ee8d82b0529472af04318c2a5ddb4cb3dcced29e`.
  Live `origin/main`: `6ce281a25975df3cac85d20c5332b76d7475a720`.
- Exclusive original replay boundary:
  `8d8535664655477a0e2f7bef9196cab77abfac19`. It is an ancestor of HEAD, the
  merge-base with current main, and the range contains 13 linear commits and no
  merge commits. `origin/main` has four later commits. Do not rebase until
  active workers, archive provenance, recovery refs, and gate evidence have
  been reconciled.
- The four later main commits include API step-level auth #108,
  Dependabot automerge pin #111, mutation reusable pin #113, and coverage
  action pin #109. They touch CI, Dependabot automerge, and mutation workflows.
  Both sides edit `.github/workflows/ci.yml` and
  `.github/workflows/mutation-testing.yml`; preserve both sides' action pins
  and job routing in those conflict resolutions. More precisely, the main CI
  delta adds step-level `GITHUB_TOKEN` to Mermaid installation and bumps
  `generate-coverage` to `abf0dcf2686de1eaf79b6dc9a16662b631bed149`; the
  archive removes PR-lane CodeScene upload and uses older
  `d4d248bbbecdcf7b4f5bc79ffd4d6caee370bd79`. The mutation workflow must
  preserve main's `abf0dcf` reusable pin and branch build-tool setup. Validate
  the resulting action contracts rather than choosing a side by SHA.
  `git check-attr` reports `merge: unspecified` for the sampled paths; global
  Weave is registered but not selected. The host conflict style is `zdiff3`.
- The full 13-commit branch range changes 78 paths, with 3,814 insertions and
  1,407 deletions against the replay boundary. The final Lody archive commit
  `ee8d82b` itself changes 44 paths, with 2,341 insertions and 384 deletions
  against its parent. It mixes build defaults, CV-005, spelling, test quality,
  source, and workflow contracts. It was archived before its combined gates
  ran; the baseline results below now measure this commit.
  `sem diff --format json` reports 212 entity-level changes across 46
  recognized files (117 added, 62 modified, 13 deleted, 1 renamed, and 19
  orphan entities); its read-only output is
  `/tmp/mapsplice-rust-baseline-archive-sem-20261001.json`. The two
  `git diff --check` findings occur in intentional CR-only fixture data and
  need byte-preservation review.
- The six worker worktrees at `e695429` contain overlapping staged or unstaged
  work. They are independently owned and must not be reset, cleaned, or blindly
  cherry-picked. Compare their changes with the archive and seek worker
  hand-off before accepting a batch.
- Live upstream checks report shared-actions #522 merged as
  `6dea5677a84fec60ca51b07202570e3af12ffdb4`, agent-helper-scripts
  #170 merged as `cedfe6f3f60af4908788ab21efdf2ca11d44ef2c`, and Concordat
  #223 merged as `817081292ccfe6beb1937dfa3502211fc34a322d`.
  The approved frozen policy snapshot for this run is
  `902d034d9da8e7ca33a0d4032770519dd1609de2`.
- That snapshot contains `rust-build-defaults` 0.1.1,
  `main-owned-codescene-coverage` 0.3.0, `markdown-formatting-baseline` 0.1.0,
  `spelling-config-baseline` 0.1.0, and `whitaker-provisioning` 0.1.0. Its
  `spelling-config-baseline` rule verifies the pinned binding gate but does not
  inspect `--scope all`, although the canonical README requires that argument.
  The isolated spelling batch adds that argument; its eight-file patch is
  awaiting combined-head integration and validation.

### Frozen rule artefacts

The selected `rule.yaml` SHA-256 values are fixed for this run:

| Package                         | Version | Manifest SHA-256                                                   |
| ------------------------------- | ------- | ------------------------------------------------------------------ |
| `rust-build-defaults`           | 0.1.1   | `1fa3d07c7d666314ec23b8ba0381fab5bc082d7faf5ddcd7a030bf71cb55c551` |
| `main-owned-codescene-coverage` | 0.3.0   | `b46d5566d8ef87a250e337acbfef41c2967c3f8709133bee71ff24da8b47c9aa` |
| `markdown-formatting-baseline`  | 0.1.0   | `2ef3cbabbb9bb1fcff2bc12c7a41a812bfd25b968e2987501e10732461ac2626` |
| `spelling-config-baseline`      | 0.1.0   | `905b08a399dc8ce651aa49c996a4bb62b48b77ebaf2506aa7ffab6fc90844cb3` |
| `whitaker-provisioning`         | 0.1.0   | `009abf41480130a1b9c438eb24567a999b29cf600d430a08ce710cbd7a3df412` |

The spelling package README SHA-256 is
`99e5d6e0b26ed09e6830ee61cf8d6557917131bdc3ac52347a76d5fa6cd265a5`; its Rego
policy SHA-256 is
`f90ca770936f9cb022b16bb03fa1649fc51fdfaca75754cb316bbc819d18cfdd`.

### Administrative state

- `codescene` environment does not exist (API GET total_count=0), and secret
  metadata lookup returned 403. Authenticated GitHub user `leynos` has
  collaborator `admin`/`role_name` `admin`, but one authorized
  `gh api --method PUT repos/leynos/mapsplice/environments/codescene --input -`
  with this body returned HTTP 403 `Resource not accessible by integration`
  (exit 1):

  ```json
  {"deployment_branch_policy":{"protected_branches":false,"custom_branch_policies":true}}
  ```

  No retry or alternate write route was used. The environment remains absent.
  Its owner must create it, add `main` through the deployment branch policy
  endpoint, and provide successful GET read-back of the environment and
  policies. No secret value was inspected or moved; secret metadata remains
  unresolved after a separate 403.
- No remote delivery branch or PR has been established by the supervisor's
  live check. Publication is authorized by the task, but the failing required
  gates and unresolved provenance prevent a reviewable draft PR now. History
  rewrite awaits worker reconciliation, recovery refs, and a gated candidate.
- The maintainer approved excluding Cranelift from all development defaults
  after the pinned compiler's unwind failure. Revisit the backend on 2027-04-01
  in [issue #115](https://github.com/leynos/mapsplice/issues/115). The issue is
  open and names `leynos` as owner in its body. GitHub's assignee mutation
  returned `Resource not accessible by integration`, so its assignee metadata
  remains empty; no repeat write was attempted.
- Cargo's unstable `codegen-backend` feature switch remains enabled for
  explicit LLVM profile overrides in coverage and Whitaker. The switch alone
  does not select Cranelift; the development profile no longer names it.
  Retaining the switch keeps those non-development routes isolated from
  inherited backend settings. Current-head compiler proof remains pending.
- Integration owner: this journeyman. Terra High is unavailable in the
  provided model list; review remediation replacement is a `gpt-6-sol`
  journeyman at medium reasoning, to be recorded on dispatch.

## Rust surfaces and tools

- `cargo metadata --no-deps` finds one Rust 2024 root package, with one lib,
  one bin, and 18 integration-test crates; no other tracked `Cargo.toml`
  exists. There is no workspace inheritance requirement for this tree.
- No Rust source file exceeds 400 lines. `tests/roadmap_ops.rs` is exactly
  400 and `src/roadmap/ops/dependency_text.rs` is 396; re-check after fixes.
- Observed tools on this host: nightly-2026-03-26, rustfmt 1.9.0-nightly,
  Clippy 0.1.96, mdtablefix 0.6.0, markdownlint-cli2 0.22.1, cargo-dylint 6.0.1,
  `mold` 2.41.0, and clang 21.1.8. `leta workspace add` worked; CodeGraph
  reindex was unavailable.
- The selected spelling builder is `v0.1.3` at tag object
  `2632c2d6d00a35f4e1f2f610ea0ede0c129d028f`; its executable interface was
  checked. The baseline spelling gate left `typos.toml` unchanged. The observed
  shared-base SHA-256 was
  `d67b4110813615a4eda3e8962e898466191e4af25b2e28baedcbab348696aeac`. The
  shared dictionary remains live by design, so a later run needs its own
  observed base identity.

### Configuration hashes at `ee8d82b`

These SHA-256 hashes identify the local policy inputs measured by the first
gate run. The spelling builder's live shared dictionary is a separate input;
its observed hashes for the later spelling run are recorded below.

| File                                  | SHA-256                                                            |
| ------------------------------------- | ------------------------------------------------------------------ |
| `Cargo.toml`                          | `3b0d97c8b28b076357b265672b2b60074dd65c229bbd1d5ed3d8e545c554bdd5` |
| `clippy.toml`                         | `22e06d5eca866af56282df39e7103cdb59ed6cc20ce59e52033ccfd208790ef5` |
| `.cargo/config.toml`                  | `ed9c4fca65ed48c040ca1312a88995f221b123be8ab2e2677f0b55c2a79ff945` |
| `.rustfmt.toml`                       | `98110ec77cefac6e5559ebf2e2ce73f67540d4fdce8265e82f3d455fdf6e2eb3` |
| `Makefile`                            | `af270051c62929563a6355a85ec323fba56124ff113e7973ba9a120bf38fef34` |
| `rust-toolchain.toml`                 | `96065a27e4308fb846b011c39566e9951d76cfed2dcbf8dcdcddf566bda2bc22` |
| `.github/workflows/ci.yml`            | `60bf7e9139c60d43a2c0c6993032931cc24f512534b7310c457131d025ad8fb4` |
| `.github/workflows/coverage-main.yml` | `9e72b9a996db12a6b21ef3adf511e955981075703162f3e99d5e97e949035b7e` |
| `.markdownlint-cli2.jsonc`            | `4ec56005a6b935505d98d5e437e25b3bec96f14438b35ef1bc728849d2b00179` |
| `typos.local.toml`                    | `0bba4267f7ecd044b36e9c76a48aeed8e1468801d38152097b5f2aec4ee40eaf` |
| `typos.toml`                          | `2091fd7d4dc408285458a4153e82ff841350becdd8cb2cf50cef14bde2254676` |
| `AGENTS.md`                           | `34afaad1b5762c21506427e413d7a995673726be6e153b9c5570e0abcaeeec14` |

## Prior branch work

- `aeadcb5` through `da47be7` are 11 focused test, documentation, structural,
  and source commits. `58da927` records the first Cranelift unwind hypothesis,
  which later experiments falsified; H2 has since isolated the compiler
  difference without Cargo configuration.
- `ee8d82b` archives the uncommitted implementation. It adds default Cargo
  Cranelift/mold/frontend routing, Make and CI provisioning, a main coverage
  workflow, local workflow contracts, partial spelling and Markdown changes,
  source/test refactors, and fixture material.

The original branch series is linear and child-owned after `8d853566`:

| Order | Commit    | Work batch                                        |
| ----- | --------- | ------------------------------------------------- |
| 1     | `aeadcb5` | Extract in-place configuration tests              |
| 2     | `2819efc` | Scope configuration test process state            |
| 3     | `3d570aa` | Document private CLI parsing model                |
| 4     | `7f92d7d` | Document private rendering and metrics state      |
| 5     | `438b4bb` | Split near-limit Rust modules by responsibility   |
| 6     | `d0f8ee6` | Preserve top-level CLI help without inferred text |
| 7     | `cb215e2` | Extract task-source operation test helpers        |
| 8     | `608f049` | Document roadmap parser internals                 |
| 9     | `9cf12c0` | Document roadmap and filesystem internals         |
| 10    | `5c71c18` | Document roadmap mutation internals               |
| 11    | `da47be7` | Document binary helpers and test assertions       |
| 12    | `58da927` | Record inconclusive Cranelift unwind probes       |
| 13    | `ee8d82b` | Archive mixed baseline onboarding and source work |

Active isolated work is not part of the delivery branch yet:

| Branch                           | Base      | Owner and scope                                                     | State                                                                                                                                       |
| -------------------------------- | --------- | ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `rust-baseline-stderr-remedy`    | `ee8d82b` | Artisan: `src/main.rs` stderr boundary                              | Patch reviewed statically; tests and gates pending; no commit.                                                                              |
| `rust-baseline-spelling-scope`   | `ee8d82b` | Spelling Journeyman: full-scope gate and exact exemptions           | Eight-file patch passes isolated spelling twice; generated output is stable; contracts retain one inherited environment failure; no commit. |
| Six pre-existing worker branches | `e695429` | CV-005, Markdown, test quality, Whitaker, lint policy, and spelling | Dirty worktrees overlap the archive; preserve and reconcile before integration.                                                             |

### Inherited enforcement conflict

- The current branch already has final lint tables in `Cargo.toml` and the
  thresholds in `clippy.toml`; the developer guide identifies Concordat
  `902d034d9da8e7ca33a0d4032770519dd1609de2`, now accepted as this run's frozen
  policy revision. The exact audit commands and rule-package versions are
  recorded below. The guide says the `disallowed_methods` list has not been
  selected, so the environment policy is not yet enforced. No `dylint.toml` is
  tracked; `make lint` ran Whitaker successfully, but the rolling suite
  revision was not emitted. Its required exclusions still need policy review.
  This archived enforcement violates the requested fixes-before-enforcement
  sequence; preserve it as inherited history and repair at source rather than
  adding mass lint suppressions.

## Open acceptance map

| Requirement    | Observed state                                                                                                                                        | Required next evidence                                                                                                      |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| Build standard | LLVM candidate passed 293/293 tests, 12 doctests, verbose bare Cargo/Make route checks, and the frozen audit with its documented Cranelift exception. | Commit the accepted batch, then reconfirm the final combined PR head and CI.                                                |
| CV-005         | Main publisher workflow and PR lane edits in archive.                                                                                                 | Contract/gate audit, protected `codescene` environment read-back, token migration owner evidence, current-head CI.          |
| Markdown       | Direct tool wiring is present; frozen audit passes at `ee8d82b`.                                                                                      | Prove untracked selection, format the ledger, rerun full Markdown gates on integrated head.                                 |
| Whitaker       | Approved merged action `6dea5677a84fec60ca51b07202570e3af12ffdb4`, binding local gate, and frozen audit pass.                                         | Observe rolling suite revision, CI installation and gate, final-head audit.                                                 |
| Spelling       | Builder `v0.1.3` and canonical AGENTS block are present; isolated `--scope all` batch passes twice on the same shared base.                           | Integrate reviewed patch, compare block, rerun gate and audit on combined head.                                             |
| Source lints   | Current-head Clippy and Whitaker pass, but three stderr suppressions and absent environment method policy remain.                                     | Integrate isolated stderr remedy after gates; policy owner supplies approved method list, then re-measure and fix findings. |
| Integration    | 13 commits diverged from current main; archive overlaps dirty worker trees.                                                                           | Worker hand-offs, semantic overlap audit, accepted commits, safe rebase, final exact-head gates.                            |
| PR/review      | None established.                                                                                                                                     | One draft PR after coherent gated branch; Codex, CodeScene, CodeRabbit, CI and pre-merge dispositions.                      |

## Findings map and evidence status

- Three `#[expect(clippy::print_stderr)]` calls in `src/main.rs` are outside
  the task's sanctioned exception categories. They are user-facing CLI output;
  any fix must retain output semantics with a fallible stderr writer. Gate
  measurement must confirm the current lint result.
- Cranelift: the current `ee8d82b` full development test route failed the
  `catch_unwind` and spawned-thread panic probes. A bounded H2 experiment
  reproduced the compiler difference with a bare, unchanged `catch_unwind`
  program: LLVM exited 0 and Cranelift exited 101 without Cargo configuration,
  dependencies, parallel frontend, or the pinned linker. Both compiled
  successfully under `rustc 1.96.0-nightly (80d0e4be6)`; the source SHA-256 was
  `9948b1f1fd3deb1c9f3bad63d8b7a6a29dfc6aefc6401e4650d5be945f007a85`. Logs are
  `/tmp/mapsplice-cranelift-plan-20261001/h2-llvm-run.out` and
  `/tmp/mapsplice-cranelift-plan-20261001/h2-cranelift-run.out`. The maintainer
  approved excluding Cranelift from all development routes, with a revisit in
  issue #115 on 2027-04-01. The current batch removes its default and installer
  requirement. The uncommitted candidate passed all 293 ordinary tests and 12
  doctests (two ignored) under its default LLVM route; bare Cargo and Make
  verbose builds showed the parallel frontend and pinned Linux linker without a
  Cranelift selector. Receipts are under
  `/tmp/mapsplice-llvm-batch-20261001/run2/`. Revalidate after the commit or
  any routing change.
- `git diff --check 58da927..ee8d82b` exits 2 for two intentional CR-only
  fixture files. Audit their authored bytes and test use before any change.
- The frozen-policy audits at `ee8d82b` report build defaults, Markdown,
  spelling, and Whitaker provisioning compliant. CV-005 is noncompliant: the
  publisher lacks `environment: codescene`. The spelling audit's passing result
  does not cover the canonical `--scope all` requirement. The exact commands,
  revision, receipts, and audit gap appear below.
- The current coverage workflow does not identify the CodeScene project. Its
  actual analysed project and branch must be confirmed by the service owner; no
  exemplar project identifier may be substituted.
- Scrutineer baseline at `ee8d82b`: check-fmt, lint (Clippy and Whitaker),
  typecheck, spelling, Markdown lint, and Nixie pass. Workflow contracts run
  112 passed and one failed: the protected CodeScene environment is absent.
  `make test` ran 83 of 293 tests: 81 passed, two Cranelift panic/unwind probes
  failed, and 210 did not run after abort. The gate logs are under
  `/tmp/mapsplice-baseline-gates-20261001/run1/`, named for each Make target.
  `make lint` ran rustdoc, Clippy, and Whitaker in that order. The installer
  managed nightly-2026-05-28; its rolling suite revision was not observed.
  During lint startup, an unrelated Peregrine Cargo/doc process briefly held
  the shared package-cache read lock; the interval ended after about 30
  seconds. No contention-free host claim follows from these gate results. These
  failures block any gated commit.
- The current lint tables produced no Clippy, Whitaker, or docsrs findings in
  their completed runs. The intended environment-access method list is absent
  from the selected canon and local config, so that policy dimension remains
  unmeasured. The policy owner must supply an approved list with its source
  revision and exact methods; then the team can add the policy in scratch,
  re-measure, repair source findings, and audit the final enforcement. The three
  `print_stderr` suppressions also hide source findings. Keep configuration
  drift, source findings, operational audit errors, and skipped surfaces
  separate.
- The standalone docsrs probe passed with
  `RUSTDOCFLAGS='--cfg docsrs -D warnings'`, but the binding Make lint route
  currently defaults `RUSTDOC_FLAGS` to `-D warnings` without `--cfg docsrs`.
  The final lint-enforcement batch must add that flag and verify the evaluated
  `cargo doc` invocation. A separate passing probe does not prove Make wiring.
- At the LLVM candidate, focused build/Whitaker/component contracts passed
  81/81. The full workflow contract target passed 122 and failed its one
  inherited CV-005 environment assertion. `make lint`, `make typecheck`,
  `make check-fmt`, `make markdownlint`, `make spelling`, and `make nixie`
  passed. Whitaker used suite revision
  `e768ba5833df5db16e44c361e19ee724997baab9` under `nightly-2026-05-28`;
  library and driver SHA-256 values were
  `1258389d774dacc281737e12d2018302cccd10ea918415123725a980b4ff7aac` and
  `b338910add5934f960307d978e5137321b86361a2c51e39c12aeeb87c772c9e8`. The
  default-scope spelling run left generated `typos.toml` unchanged at SHA-256
  `2091fd7d4dc408285458a4153e82ff841350becdd8cb2cf50cef14bde2254676`; the live
  shared-base content SHA-256 was
  `d67b4110813615a4eda3e8962e898466191e4af25b2e28baedcbab348696aeac`. These
  results do not clear the wider spelling scope or CV-005 blocker.
- The first frozen `rust-build-defaults` audit on this LLVM candidate returned
  BD-004 noncompliant: the developer guide stated the exception only in prose.
  The frozen rule requires a `Cranelift`-named heading in that guide and the
  pinned channel within its section. A bounded guide edit added both. The
  second audit under the same Concordat `902d034` rule returned compliant with
  zero findings; its receipt is
  `/tmp/mapsplice-llvm-batch-20261001/run3/rust-build-defaults-audit.out`. The
  rule was not changed or weakened.
- The first explicit `typos-config-builder gate --repository . --scope all`
  measurement found 92 raw occurrences. They cluster around fixed external
  Cargo's terminal-colour setting, the GitHub label event, and the
  `-fuse-ld=mold` linker flag in action inputs, paths, version strings, local
  identifiers, and tool prose. This is a count of occurrences, not distinct
  defects. The existing default-scope spelling pass did not measure the same
  Rust and workflow files. A spelling Journeyman owns exact-context repairs in
  the isolated `rust-baseline-spelling-scope` tree. The log is
  `/tmp/mapsplice-baseline-gates-20261001/run2/spelling-all-mapsplice-rust-baseline-hardening-mapsplice.out`.
- The spelling patch renames the local Clang wrapper to
  `scripts/clang-linker.sh` without changing its bytes because builder `v0.1.3`
  cannot ignore only a filename: its ignore patterns apply to content, whereas
  file exclusions skip the whole file. The wrapper's old and new SHA-256 are
  both `138be6ef7f61d2bc4d452551ebd43a67b3aa4427e03de5158bdbd13f54f88f24`. An
  earlier builder run raced final overlay edits; the gate owner then ran
  `make spelling` twice from the stable overlay. Both runs passed and left the
  generated SHA-256
  `e7b68db3a220f0e8268dff9f16a34a64d64415e3cc61703cfd95105cabcc63ea` unchanged.
  The overlay hash was
  `6793c0633bfacd97ffe3699961b6cc8a223a6bcdda4b08fd1204713e9137e778`;
  shared-base TOML was
  `d67b4110813615a4eda3e8962e898466191e4af25b2e28baedcbab348696aeac` and JSON
  was `216390a12a035d1b74015496d2e6f3d5a461a05b8b43bd95542dbe2f181205d7`. Logs
  are under `/tmp/mapsplice-baseline-spelling-20261001/`, with final runs ending
  `spelling-rust-baseline-spelling-scope-{3,4}.out`. Exact API/tool forms pass
  while the bare linker name, US spelling for colour, bare label-event term, an
  extended Cargo colour variable, and a Rust source typo fail the focused
  negative samples. No generated file was hand-edited, and this isolated pass
  does not validate the integration head. The isolated workflow contracts ran
  112 passing cases and retain the single inherited CV-005 environment failure.
- An isolated `src/main.rs` repair removes all three `print_stderr`
  suppressions and retains explicit fallible stderr output. An independent
  Scrutineer static review found no byte or exit-code defect; its exact-byte
  and writer-error unit tests remain unexecuted until a gate slot is free. This
  patch is uncommitted in `rust-baseline-stderr-remedy` and is not part of the
  measured integration head.

### Exact-head baseline gates

All commands below ran sequentially at `ee8d82b` from the repository root. Each
output file is under `/tmp/mapsplice-baseline-gates-20261001/run1/`. The gate
owner retained command failures through the logging pipeline.

| Order | Command                        | Result                          | Output file                                                               |
| ----- | ------------------------------ | ------------------------------- | ------------------------------------------------------------------------- |
| 1     | `make check-fmt`               | Pass                            | `check-fmt-mapsplice-rust-baseline-hardening-mapsplice.out`               |
| 2     | `make test-workflow-contracts` | Fail: 112 passed, one failed    | `test-workflow-contracts-mapsplice-rust-baseline-hardening-mapsplice.out` |
| 3     | `make markdownlint`            | Pass                            | `markdownlint-mapsplice-rust-baseline-hardening-mapsplice.out`            |
| 4     | `make nixie`                   | Pass                            | `nixie-mapsplice-rust-baseline-hardening-mapsplice.out`                   |
| 5     | `make spelling`                | Pass                            | `spelling-mapsplice-rust-baseline-hardening-mapsplice.out`                |
| 6     | `make lint`                    | Pass: rustdoc, Clippy, Whitaker | `lint-mapsplice-rust-baseline-hardening-mapsplice.out`                    |
| 7     | `make typecheck`               | Pass                            | `typecheck-mapsplice-rust-baseline-hardening-mapsplice.out`               |
| 8     | `make test`                    | Fail: Cranelift panic probes    | `test-mapsplice-rust-baseline-hardening-mapsplice.out`                    |

### Explicit LLVM and frozen-policy measurement

The first LLVM command included an unsupported `-vv` argument to the selected
nextest route. It stopped before compilation and is an invocation error, not a
test verdict. The corrected sequential run at `ee8d82b` used:

```sh
env CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm \
  CARGO_PROFILE_TEST_CODEGEN_BACKEND=llvm \
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang \
  make test RUST_FLAGS='-D warnings' \
  TEST_FLAGS='--workspace --all-targets --all-features' \
  BUILD_JOBS='-j 2' 'gate_rust_flags=$(RUST_FLAGS)'
```

It passed all 293 ordinary tests, with no skipped tests; the doctest leg passed
12 and ignored two. The log is
`/tmp/mapsplice-baseline-gates-20261001/run2/test-llvm-corrected-mapsplice-rust-baseline-hardening-mapsplice.out`,
with the matching `.meta` receipt. Separately,
`RUSTDOCFLAGS='--cfg docsrs -D warnings' cargo doc --workspace --no-deps`
passed; its log is in the same run directory as
`doc-rustdocflags-mapsplice-rust-baseline-hardening-mapsplice.out`. These
results prove the explicit LLVM route at this head, not approval to make it the
repository's default test route.

The approved Concordat executable is version 0.1.0 from the checkout fixed at
`902d034`. Each frozen package was run with this command shape, substituting
the package ID:

```sh
/home/leynos/.lody/repos/github---leynos---concordat/worktrees/audit-cabochon-baseline-902d/.venv/bin/concordat \
  artefact rule run RULE_ID \
  --repo /home/leynos/.lody/repos/github---leynos---mapsplice/worktrees/3c141d42-0280-4dd8-bff6-2e48ae26ac13 \
  --format json
```

| Frozen rule                     | Result                                             |
| ------------------------------- | -------------------------------------------------- |
| `rust-build-defaults`           | Compliant                                          |
| `main-owned-codescene-coverage` | Noncompliant: missing protected job environment    |
| `markdown-formatting-baseline`  | Compliant                                          |
| `spelling-config-baseline`      | Compliant, but rule does not inspect `--scope all` |
| `whitaker-provisioning`         | Compliant                                          |

Each audit's output and `.meta` receipt are under
`/tmp/mapsplice-baseline-gates-20261001/run2/`, named
`audit-<rule>-mapsplice-rust-baseline-hardening-mapsplice`. No verdict is
claimed for later source changes until the audit is rerun at their head.

## Proposed bounded sequence

1. Keep the frozen Concordat revision and completed surface inventory fixed;
   maintain the external environment and method-list blockers explicitly.
2. Reconcile each dirty worker tree against archive content without modifying
   worker state. Assign one coherent remaining batch per owner; do not absorb
   partial patches merely because the archive contains their files.
3. Preserve the measured H2 compiler evidence. Apply the approved LLVM
   default across build, check, lint, documentation, and tests; prove actual
   compiler invocations and keep issue #115 as the dated follow-up.
4. Review the isolated stderr and spelling patches. Complete independent
   spelling and Markdown preparation, but do not commit or publish a candidate
   while the required current-head gates fail. Arrange sequential Scrutineer
   validation of the LLVM batch; the protected environment remains an
   independent blocker to the complete one-PR objective.
5. Complete onboarding batches in dependency order, with sequential
   Scrutineer gates before each commit.
6. Establish recovery refs and an explicit semantic conflict plan before a
   `zdiff3` rebase onto the recorded main commit. Revalidate the resulting
   candidate, then publish exactly one draft PR and work its current-head
   review and CI to merge eligibility.

## Gate and publication ledger

| Head                                                 | Batch                                         | Owner                                                             | Gate evidence                                                                                                                                                                                                                                                                                                          | CI/review       | Next action                                                                          |
| ---------------------------------------------------- | --------------------------------------------- | ----------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------- | ------------------------------------------------------------------------------------ |
| `ee8d82b`                                            | Existing branch/archive baseline              | Scrutineer                                                        | check-fmt, lint, typecheck, default-scope spelling, Markdown lint, Nixie pass; contracts 112/113; default test 81 pass, 2 fail, 210 not run                                                                                                                                                                            | No delivery PR  | Cranelift exception and protected environment decisions pending.                     |
| `ee8d82b`                                            | Explicit LLVM and frozen-policy measurement   | Scrutineer                                                        | 293 ordinary tests pass, 12 doctests pass and 2 ignored; docsrs pass; four of five audits compliant; full-scope spelling finds 92 raw occurrences                                                                                                                                                                      | No delivery PR  | Integrate reviewed fixes only after required gate path is resolved.                  |
| Isolated `rust-baseline-spelling-scope` at `ee8d82b` | Full-scope spelling repair                    | Spelling Journeyman; Scrutineer gates                             | `make spelling` twice pass with stable overlay, shared-base and generated hashes recorded above; targeted negative samples fail; contracts 112 pass/1 inherited CV-005 failure                                                                                                                                         | No commit or PR | Integrate after required route/admin blockers and rerun combined-head gates.         |
| Isolated `rust-baseline-stderr-remedy` at `ee8d82b`  | Remove three stderr lint suppressions         | Artisan; independent static Scrutineer review                     | Static byte/exit/error review found no defect; unit and repository gates have not run                                                                                                                                                                                                                                  | No commit or PR | Run unit/full gates under approved route, then integrate and remeasure lints.        |
| `68cc5d5` + LLVM working patch                       | Approved Cranelift exception and LLVM default | Integration Journeyman; contract Artisan, docs Scribe, Scrutineer | Focused contracts 81 pass; default Nextest 293 pass and doctests 12 pass/2 ignored; check-fmt, lint, typecheck, Markdown lint, spelling, Nixie pass; bare Cargo/Make verbose routes prove frontend/linker and no Cranelift; frozen build audit compliant. Full workflow contracts 122 pass/1 inherited CV-005 failure. | No delivery PR  | Commit the bounded batch after doc checks; preserve admin and final policy blockers. |
