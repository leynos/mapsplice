# Mapsplice Rust baseline execution ledger

This record tracks the single delivery branch and its measured state. A passing
local gate applies only to the commit and live inputs named here; it is not
final acceptance or evidence of a hosted review.

## Rebase checkpoint: 2026-10-02

PR [#116](https://github.com/leynos/mapsplice/pull/116) was rebased from old
base `2a9c6224422069d1984310ac8016a60fc7b1e874` and old head
`9885885f4c479aa694bb30e4517e643f26e10b34` onto fetched live `main` at
`c6ba81beea4c12b59328e728837a6a6424ff69df`. The rebase replayed 20 linear
commits. Its new head before the pending follow-up is
`3760cbd8de1e2c0ce21b6ffe49d1d1e9c6aabfbe`; recovery refs use prefix
`refs/recovery/rust-baseline-hardening-mapsplice/20261002T000000Z`.

The `ci.yml` Setup Rust conflict retains the target's `setup-rust` pin
`ff1dd759...` and the branch's `install-mold: true`, empty `rustflags`,
`clang`, and build-tool preflight settings. The action definition at that
immutable pin supports both inputs. The `Cargo.lock` resolution keeps the
target's `thiserror` 2.0.21 and the branch's `temp-env`; the target's
Dependabot workflow remains byte-identical. Git automatically combined the
mutation reusable workflow's target `ff1dd759...` pin with the branch's setup
commands.

Two local workflow-contract pin expectation updates remain uncommitted: one for
the build-standard setup pin and one for the coverage contract's Dependabot
reusable-workflow pin. Read-only inspection confirmed that the target
Dependabot workflow blob `931290b9d32e42c6a28ae1b05a4960f3c427bad7` is
byte-identical to the previously approved `abf` version and contains no
CodeScene, token, or environment route. Gates for the new head are pending, so
no result is attributed to it. The protected `codescene` environment remains
absent and continues to block the required coverage contract and final
acceptance.

## Current delivery checkpoint

- Draft PR [#116](https://github.com/leynos/mapsplice/pull/116) targets the
  discovered default branch `main`, base
  `2a9c6224422069d1984310ac8016a60fc7b1e874`. The delivery branch is
  `rust-baseline-hardening-mapsplice`, at
  `8cefe59bf49a3f697d185f927dabf8c10146caaf`; local and remote heads matched at
  this checkpoint. The PR remains draft and blocked.
- The PR body groups the 19 committed delivery changes into an ordered
  work-batch map. The proposed twentieth commit addresses the hosted Whitaker
  cold-driver failure described below; its source repair is still pending, so
  no gates or CI result are attributed to it.
- Managed CodeRabbit queue `ae3a9049` is pending. No review verdict is
  available yet. Hosted CI run `36878449649`, job `110423802511`, failed the
  Whitaker lint step at the current head; the failure and proposed repair are
  recorded below.
- Required external blockers are unchanged: the protected `codescene`
  environment is absent, its one authorized creation attempt returned HTTP 403,
  and token provisioning/removal and the CodeScene project identity are
  unverified. The frozen Concordat revision and repository contain no approved
  `disallowed_methods` environment-access list. Neither blocker has been
  cleared by this checkpoint.

## Git boundary and ownership

- Repository: `leynos/mapsplice`; delivery branch:
  `rust-baseline-hardening-mapsplice`.
- Current delivery head: `8cefe59bf49a3f697d185f927dabf8c10146caaf`.
  Its verified base is fetched default `main` at
  `2a9c6224422069d1984310ac8016a60fc7b1e874`. Exact-head local gates have run;
  their results and the subsequent hosted failure are recorded below.
- The completed textual `zdiff3` rebase replayed 18 commits from the exclusive
  original boundary `8d8535664655477a0e2f7bef9196cab77abfac19`. The old
  delivery head was `68fb6f0cc3d08c6956c871bb9ce13190e16b87e5`; recovery refs
  use prefix
  `refs/recovery/rust-baseline-hardening-mapsplice/20261001T141649Z`. The
  ordered work-batch map below records the pre-rebase history; commit IDs from
  that map are historical and must not be used as current-head evidence.
- The five later main commits include API step-level auth #108,
  Dependabot automerge pin #111, mutation reusable pin #113, coverage action
  pin #109, and shared Whitaker installation #114. They touch CI, Dependabot
  automerge, and mutation workflows. Both sides edit `.github/workflows/ci.yml`
  and `.github/workflows/mutation-testing.yml`; preserve both sides' action
  pins and job routing in those conflict resolutions. More precisely, the main
  CI delta adds step-level `GITHUB_TOKEN` to Mermaid installation and bumps
  `generate-coverage` to `abf0dcf2686de1eaf79b6dc9a16662b631bed149`; the
  archive removes PR-lane CodeScene upload and uses older
  `d4d248bbbecdcf7b4f5bc79ffd4d6caee370bd79`. The mutation workflow preserves
  main's `abf0dcf` reusable pin and branch build-tool setup. One CI conflict
  was resolved by retaining main's step-scoped `GITHUB_TOKEN`, `abf0dcf` PR
  coverage generator and mutation reusable pins, and approved Whitaker action
  pin `6dea5677a84fec60ca51b07202570e3af12ffdb4`, while preserving the branch's
  coverage and workflow contracts. The branch's later user-approved Cranelift
  input removal is also preserved. The coverage workflow and contract
  reconciliation is committed in the delivery history: PR and publisher use the
  `abf0dcf` generator while the uploader and setup action remain at
  `d4d248bbbecdcf7b4f5bc79ffd4d6caee370bd79`. Exact-head local and contract
  evidence is recorded below; CV-005 remains blocked on protected-environment
  provisioning. `git check-attr` reports `merge: unspecified` for the sampled
  paths; global Weave was not selected. The conflict used textual `zdiff3`; the
  exact-head contract result is recorded below.
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
  The eight-file spelling batch, including that argument, is now integrated in
  `ecd4b0c`; combined-tree gate and audit validation remain pending.

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
- At the earlier pre-publication snapshot, no remote delivery branch or PR had
  been established. The current draft PR and its blocked state are recorded in
  the delivery checkpoint above. The protected-environment and policy blockers
  remain unresolved.
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
- No MSRV, stable-channel, feature, OS, or broad platform support promise is
  declared. First-party CI currently exercises Ubuntu only; target-specific GNU
  settings cover x86_64 and aarch64, while other platforms remain unmeasured.
  Cargo publication is enabled by default, but no package/publish workflow or
  route is established. The prior release contract passed; probe the verbose
  release route at the combined-head checkpoint before claiming release
  compatibility.
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
- `ee8d82b` archives the implementation as it stood before integration. That
  historical snapshot temporarily selected Cranelift by default; the accepted
  LLVM batch later removed that default while retaining the pinned linker and
  parallel frontend routing. The archive also contains Make and CI
  provisioning, a main coverage workflow, local workflow contracts, partial
  spelling and Markdown changes, source/test refactors, and fixture material.

The original pre-rebase branch series was linear and child-owned after
`8d853566`:

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

The spelling and stderr batches are integrated in the rebased history as
`ecd4b0c` and `e476656`, respectively. Their pre-rebase worker trees remain
available as provenance:

| Branch                           | Base      | Owner and scope                                                     | State                                                                                                                                                                                                                              |
| -------------------------------- | --------- | ------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `rust-baseline-stderr-remedy`    | `ee8d82b` | Artisan: `src/main.rs` stderr boundary                              | Patch replayed as pre-rebase commit `5513e0c` (now `e476656`); focused tests and sequential repository gates passed before rebase, so evidence is historical. Preserve this worktree as source provenance.                         |
| `rust-baseline-spelling-scope`   | `ee8d82b` | Spelling Journeyman: full-scope gate and exact exemptions           | Eight-file patch passes isolated spelling twice on the same shared base; generated output is stable; contracts retain one inherited environment failure; patch is integrated as `ecd4b0c`, with combined-head gates still pending. |
| Six pre-existing worker branches | `e695429` | CV-005, Markdown, test quality, Whitaker, lint policy, and spelling | Dirty worktrees overlap the archive; preserve and reconcile before integration.                                                                                                                                                    |

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

| Requirement    | Observed state                                                                                                                                                                                                                                | Required next evidence                                                                                                       |
| -------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Build standard | `41f2549` selects LLVM, retains `-Zthreads=8` and the pinned Linux linker, and records the approved Cranelift exception. The frozen build audit and full local suite pass at PR head `8cefe59`.                                               | Reconfirm on the head containing the pending Whitaker repair and in hosted CI.                                               |
| CV-005         | At `8cefe59`, the frozen audit and workflow contract retain only the absent protected `codescene` environment finding; the other four audits pass.                                                                                            | Obtain protected environment read-back, token migration owner evidence, and project identity; rerun after any changes.       |
| Markdown       | Direct tool wiring, `make markdownlint`, and the frozen Markdown audit pass at `8cefe59`.                                                                                                                                                     | Preserve these results for the integrated head; rerun if files or configuration change.                                      |
| Whitaker       | Approved action `6dea5677a84fec60ca51b07202570e3af12ffdb4` and frozen provisioning audit pass. Local lint passed at `8cefe59`, but hosted run `36878449649` failed on Dylint's cold driver bootstrap after successful installer provisioning. | Integrate the pending Cargo unstable opt-in repair, then rerun the local binding gate, audit, and hosted CI on the new head. |
| Spelling       | Builder `v0.1.3`, canonical AGENTS block, `make spelling`, and frozen spelling audit pass at `8cefe59`; the audit package does not inspect `--scope all`.                                                                                     | Preserve evidence for the integrated head; rerun if spelling inputs or configuration change.                                 |
| Source lints   | `e476656` removes the three stderr suppressions; Clippy and local Whitaker passed at `8cefe59`. The approved environment-method policy is still absent.                                                                                       | Policy owner supplies the approved method list, then measure and fix any resulting findings.                                 |
| Integration    | The 18-commit textual `zdiff3` rebase and subsequent integration commits are present at `8cefe59` on main `2a9c622`; the coverage changes are committed. The 19-commit PR map is published, with the twentieth Whitaker repair pending.       | Integrate the bounded Whitaker routing repair and validate the resulting exact head.                                         |
| PR/review      | Draft [PR #116](https://github.com/leynos/mapsplice/pull/116) is open. Managed CodeRabbit queue `ae3a9049` is pending; hosted run `36878449649` has a Whitaker cold-driver failure at `8cefe59`.                                              | Address and remeasure the Whitaker failure; resolve required review findings and external blockers before ready or merge.    |

## Findings map and evidence status

- The three `#[expect(clippy::print_stderr)]` calls in `src/main.rs` were
  removed in the pre-rebase `5513e0c` commit. The new stderr boundary uses
  fallible writers and retains CLI output semantics. Focused exact-byte and
  writer-error tests and the historical source gates passed before rebasing;
  these results are stale for the rebased head. Re-run them on the accepted
  combined tree.
- Cranelift: the historical `ee8d82b` full development test route failed the
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
  requirement. The candidate later committed as `faa8c94` passed all 293
  ordinary tests and 12 doctests (two ignored) under its default LLVM route;
  bare Cargo and Make verbose builds showed the parallel frontend and pinned
  Linux linker without a Cranelift selector. Receipts are under
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
- At the baseline head, the standalone docsrs probe passed with
  `RUSTDOCFLAGS='--cfg docsrs -D warnings'`, but the binding Make lint route
  lacked `--cfg docsrs`. The working candidate at `ce7204a` adds the flag to
  the workspace/no-deps documentation command and the doctest route. Contract
  and gate validation are pending; the standalone probe does not prove the Make
  wiring.
- At the pre-stderr LLVM head, focused build/Whitaker/component contracts passed
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
- Hosted PR run `36878449649`, job `110423802511`, failed at head `8cefe59` in
  the Whitaker lint step after cold-cache installation succeeded. The approved
  shared action pin `6dea5677a84fec60ca51b07202570e3af12ffdb4` installed
  Whitaker installer `0.2.9`; its check resolved the rolling suite to
  `a77c4d252fad81c1fe571851cf505b212a0fa636` for `nightly-2026-05-28`. The
  installer reported `suite-source=prebuilt`, fetched the published suite and
  Dylint assets, and completed successfully with its no-source-fallback
  contract intact. The later failure came from Dylint `6.0.1` building its
  missing per-toolchain driver in a temporary directory: Cargo `1.98.0-nightly`
  there reported that the `codegen-backend` feature is required to use the
  inherited LLVM profile override. That directory did not discover the
  repository's unstable Cargo opt-in. This is Dylint's cold runtime bootstrap,
  not an installer fallback or a missing published Whitaker suite asset. The
  local warm-cache `make lint` pass at this head did not exercise that cold
  driver path. The hosted log is
  `/tmp/mapsplice-pr116-review-20261001/initial/job-110423802511.log`.
- A bounded Cargo routing experiment at the same pinned Whitaker nightly
  reproduced the nested-project failure with
  `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm`,
  `CARGO_PROFILE_TEST_CODEGEN_BACKEND=llvm`, and `RUSTFLAGS=-D warnings`:
  control without the Cargo opt-in exited 101; treatment with
  `CARGO_UNSTABLE_CODEGEN_BACKEND=true` exited 0, and verbose output selected
  LLVM. This proves the narrow Cargo opt-in for a temporary project, not a
  Dylint build or a green Whitaker rerun. The approved consumer repair is to
  retain the explicit LLVM profile overrides and add that Whitaker-only opt-in;
  source implementation and binding-gate revalidation are pending. Logs are
  under `/tmp/mapsplice-whitaker-codegen-optin-20261001/`.
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
- The stderr remedy was replayed from the preserved
  `rust-baseline-stderr-remedy` worktree and committed as `5513e0c`. Focused
  tests passed 2/2. Sequential run3 gates passed `make check-fmt`, `make lint`
  (including Whitaker), `make test` (295/295 ordinary tests; 12 doctests passed
  and two were ignored), `make typecheck`, `make spelling`,
  `make markdownlint`, and `make nixie`. Spelling produced no generated
  `typos.toml` drift. Whitaker used rolling suite revision
  `e768ba5833df5db16e44c361e19ee724997baab9`. The workflow-contract target was
  not rerun for this source-only batch; its inherited result remains 122 pass,
  one failure for the missing protected `codescene` environment. Run3 logs are
  under `/tmp/mapsplice-stderr-batch-20261001/run3/`. At that pre-rebase
  checkpoint, the spelling replay had not yet passed gates on the combined
  tree; the integrated spelling gate passed at PR head `8cefe59` in run 2.

### Rebased PR-head local gates

At the rebased delivery head `8cefe59`, sequential run 2 recorded
`make check-fmt`, `make lint`, `make test`, `make typecheck`, `make spelling`,
`make markdownlint`, and `make nixie` as passing. Nextest passed 295/295 tests
with none skipped; docsrs doctests passed 12 with two ignored. Four frozen
audits for build defaults, Markdown, spelling, and Whitaker provisioning were
compliant. The fifth audit, CV-005, and `make test-workflow-contracts` still
failed only on the absent protected `codescene` environment (129 contract cases
passed, one failed). Logs and receipts are under
`/tmp/mapsplice-rebased-checkpoint-20261001/run2/`. These local results do not
supersede the subsequent cold-driver failure in hosted CI recorded above.

### Historical exact-head baseline gates

All commands below ran sequentially at the pre-rebase head `ee8d82b` from the
repository root. Each output file is under
`/tmp/mapsplice-baseline-gates-20261001/run1/`. The gate owner retained command
failures through the logging pipeline. These results are retained for
provenance only; none proves the rebased delivery head.

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
test verdict. The corrected sequential run at historical head `ee8d82b` used:

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
results prove the explicit LLVM route at that historical head only. The
separate accepted build-default commit `faa8c94` made LLVM the development
default under the recorded Cranelift exception.

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

## Current next actions

1. Complete the approved Whitaker-only Cargo opt-in repair as the next commit
   on the existing PR branch. Keep LLVM profile routing and do not add
   development `RUSTFLAGS` to the Whitaker toolchain.
2. Validate the Make-evaluated Whitaker route with the minimal outside-cwd
   Cargo probe and run the required serial local gates and frozen audits on the
   resulting head. Then request a fresh hosted run, whose runner can exercise
   the cold Dylint driver bootstrap without clearing a local cache or building
   installer components from source. Keep the current hosted failure open until
   that run passes.
3. Obtain the protected `codescene` environment, its main-only policy, and the
   required token/project provisioning evidence from an authorized owner.
   Obtain the approved `disallowed_methods` policy source and revision from the
   baseline owner. Keep both blockers visible until independently verified.
4. Resolve the pending CodeRabbit queue and other current-head review findings.
   Keep PR #116 draft and unmerged until the complete one-PR acceptance
   criteria and required external prerequisites are satisfied.

## Gate and publication ledger

| Head                                                 | Batch                                         | Owner                                                             | Gate evidence                                                                                                                                                                                                                                                                                                                                                                                        | CI/review                                          | Next action                                                                                                      |
| ---------------------------------------------------- | --------------------------------------------- | ----------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `ee8d82b`                                            | Existing branch/archive baseline              | Scrutineer                                                        | Historical check-fmt, lint, typecheck, default-scope spelling, Markdown lint, and Nixie passed; contracts 112/113; default test 81 passed, 2 failed, 210 not run.                                                                                                                                                                                                                                    | No delivery PR                                     | Historical Cranelift failure and protected-environment blocker recorded above.                                   |
| `ee8d82b`                                            | Explicit LLVM and frozen-policy measurement   | Scrutineer                                                        | Historical LLVM run: 293 ordinary tests, 12 doctests passed and 2 ignored; docsrs passed; four of five audits compliant; full-scope spelling found 92 raw occurrences.                                                                                                                                                                                                                               | No delivery PR                                     | Reconfirm after integrated changes.                                                                              |
| Isolated `rust-baseline-spelling-scope` at `ee8d82b` | Full-scope spelling repair                    | Spelling Journeyman; Scrutineer gates                             | `make spelling` twice passed with stable overlay and recorded hashes; targeted negative samples failed as intended; contracts 112 passed/1 inherited CV-005 failure. Patch is integrated as `ecd4b0c`; combined-tree gates and audit remain pending.                                                                                                                                                 | No separate PR                                     | Re-run spelling and workflow contracts on the combined tree.                                                     |
| Pre-rebase `faa8c94`                                 | Approved Cranelift exception and LLVM default | Integration Journeyman; contract Artisan, docs Scribe, Scrutineer | Focused contracts 81 passed; 293 ordinary tests and 12 doctests passed/2 ignored; check-fmt, lint, typecheck, Markdown lint, spelling, and Nixie passed; verbose routes prove frontend/linker and no Cranelift; frozen build audit compliant. Workflow contracts 122 pass/1 inherited CV-005 failure. Results are historical after rebase.                                                           | No delivery PR                                     | Reconfirm on final integrated head; preserve admin and policy blockers.                                          |
| Pre-rebase `5513e0c`                                 | Fallible stderr output and lint cleanup       | Integration Journeyman; source Artisan; Scrutineer                | Focused stderr tests 2/2; sequential run3 check-fmt, lint (including Whitaker), full test 295/295 plus 12 doctests/2 ignored, typecheck, spelling, Markdown lint, and Nixie passed. No generated spelling drift. Workflow contracts were not rerun; inherited 122 pass/1 protected-environment failure remains. Results are historical after rebase.                                                 | No delivery PR                                     | Re-run all gates on the combined tree.                                                                           |
| Pre-rebase `ce7204a`                                 | Binding docsrs Rustdoc route                  | Integration Journeyman; contract Artisan; docs Scribe             | Focused route contracts 33 passed; check-fmt, lint, test (295/295 plus 12 docsrs doctests), typecheck, spelling, Markdown lint, and Nixie passed. Full workflow contracts retained only the known CV-005 environment failure. These results are historical after rebase.                                                                                                                             | No delivery PR                                     | Re-run route contracts and sequential gates on the combined tree.                                                |
| `43b70e8`                                            | Rebase checkpoint (superseded)                | Integration Journeyman; rebase owner                              | Rebased from `68fb6f0` onto `2a9c622` by textual `zdiff3`; 18 commits replayed. At that intermediate checkpoint, post-rebase gates had not run and the coverage patch was uncommitted. This row is historical; see `8cefe59` below.                                                                                                                                                                  | Superseded                                         | No action; current delivery state is recorded at `8cefe59`.                                                      |
| `8cefe59`                                            | Current PR head and local baseline checkpoint | Integration Journeyman; Scrutineer                                | Sequential run 2: check-fmt, lint, test (295/295, none skipped; docsrs doctests 12 passed/2 ignored), typecheck, spelling, Markdown lint, and Nixie passed. Four frozen audits compliant; workflow contracts 129 passed/1 failed on missing protected environment. Hosted run `36878449649`, job `110423802511`, failed Dylint's cold driver bootstrap after installer and prebuilt suite succeeded. | Draft PR #116; CodeRabbit queue `ae3a9049` pending | Integrate the Whitaker-only Cargo opt-in repair, then rerun binding and hosted gates; resolve external blockers. |
