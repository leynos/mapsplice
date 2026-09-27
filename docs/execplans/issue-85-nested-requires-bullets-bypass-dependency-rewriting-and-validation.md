# ExecPlan: Nested `Requires` bullets bypass dependency rewriting and validation (issue #85)

## Purpose

Issue #85: a `Requires` clause written as a nested task-body bullet was
silently ignored during renumbering and dangling-dependency validation.
Insertion redirected a consumer to an unrelated new task; deleting a
prerequisite produced a self-dependency; in-place deletion wrote that invalid
result to disk.

All four tasks of the coding plan are complete and committed, and every
deterministic gate is green. This plan tracks the CodeRabbit review and the PR.

## Progress so far

| Task | Scope                                                | State |
| ---- | ---------------------------------------------------- | ----- |
| 1    | Recognizer fix across all four clause positions      | Done  |
| 2    | Document the supported clause grammar and exclusions | Done  |
| 3    | CLI regressions, golden fixtures, property suite     | Done  |
| 4    | Verus proofs, ledger, Makefile and CI wiring         | Done  |

_Table 1: The coding plan's four tasks and their state._

Commits, oldest first. This list is deliberately exhaustive: the round-tally
error below was hidden for a while by a list that was missing entries, and a
reconciliation against a list that is itself incomplete cannot find anything.

The four coding-plan tasks:

- `db94b11` — Rewrite dependency clauses in nested task-body bullets.
- `5ba0ab2` — Document the supported Requires clause grammar.
- `962d5eb` — Add CLI and golden regressions for nested Requires clauses (#85).
- `0dd99de` — Add property suite and split oversized test modules (#85).
- `7b9032b` — Extract nested-Requires regressions to honour the 400-line file
  rule (#85).
- `b7fdcda` — Scope dependency property assertions to each item's own block.
- `55e3484` — Extract dependency resolution into a production-used Verus kernel.
- `9a9b7c8` — Wire the Verus harness into the build and continuous integration.
- `9202dbf` — Share the verified kernel body as a macro, not an include splice.
- `4daffb0` — Record the ICE resolution and the macro trade in the ExecPlan.

The eight CodeRabbit rounds and their response commits:

| Round | Findings | Answered by                                | Evidence artefact                                      |
| ----- | -------- | ------------------------------------------ | ------------------------------------------------------ |
| 1     | 8        | `05b49d3`                                  | `/tmp/coderabbit-mapsplice-issue-85.out`               |
| 2     | 6        | `67ab4b9`                                  | `.../tasks/bh6xlkhs1.output`                           |
| 3     | 7        | `16ef675`                                  | `.../tasks/bfs030ibl.output`                           |
| 4     | 7        | `fcbe9c6`                                  | `/tmp/coderabbit-issue-85-...out`                      |
| 5     | 4        | `92dc9bb`                                  | `/tmp/coderabbit-mapsplice-issue-85-...out.raw`        |
| 6     | 6        | `08b62f9`                                  | `/tmp/coderabbit-mapsplice-issue85-round6-1bd792a.out` |
| 7     | 4        | `067accc`                                  | `/tmp/coderabbit-mapsplice-issue85-round7-bcd0506.out` |
| 8     | 3        | `ef8cda3`, `bcc6e08`, `fe72e93`, `0ae84d1` | `/tmp/walkthrough-86-0523.md`                          |

_Table 2: the review rounds, re-derived from the artefacts by matching each
round's findings against the files its response commit touched._

Round 7 is the first round that ran as a GitHub App review on the PR rather
than as a local `coderabbit review --agent` pass. It was triggered by marking
the PR ready for review, not queued through `comenq`;
`gh api …/pulls/86/reviews` had shown zero CodeRabbit submissions before it. It
returned one inline finding plus three failed pre-merge checks for a count of
four. The tally is therefore 8, 6, 7, 7, 4, 6, 4 and 3.

Round 8 returned one inline finding and two pre-merge rows for a count of
three, and its review decision was still `CHANGES_REQUESTED`. All three were
live and all three were actioned: the inline finding at `ef8cda3` and the
line-ending row at `bcc6e08`, both recorded in item 7 of Remaining work, and
the domain-architecture row at `fe72e93` and `0ae84d1`, recorded in item 8.
Round 8 re-raised the domain-architecture row that round 7 had raised, but with
a **revised remedy**. Round 7 required the kernel be "self-contained in the
domain module", connected by "a separate proof adapter"; round 8 asks instead
to "define **or include**" it "from a domain-owned source", keeping the adapter
only as one of two options ("a proof-side adapter or refinement proof"). That
relaxation is what unblocked the round-7 objection — the adapter was never
dropped, it stopped being mandatory — and the remedy was implemented. This is
worth recording as a case where the first dismissal was right about the remedy
and wrong about the underlying complaint, so a re-raise was not a duplicate.

Supporting commits that are not themselves a round response:

- `de24f4c` — Use `is_multiple_of` where the crate denies integer remainder.
- `87edd94` — Record the fourth review round and its lessons in the ExecPlan.
- `0c084a8` — Record the green CI run in the ExecPlan.
- `1477434` — Correct the review-round count in the ExecPlan.
- `e277c1f` — Fix the spelling and formatting the gates caught in the ExecPlan.
- `67becfe` — Give the two masked oracle defects their own survival counts.
- `72ff94f` — Correct the evidence-glob lesson to what was measured.
- `0c222e4` — Re-wrap the corrected lesson paragraphs to the 80-column limit.
- `e083639` — Answer the sixth CodeRabbit review of the issue #85 work
  (amended to `08b62f9`, which is the commit the round is answered by; the
  original said "fourth", continuing an off-by-one the round table corrects).
- `97da0e4` — Fix the formatting and lint the gates caught in the round-6
  ExecPlan edit.
- `a720876` — Record the two formatter lessons from the round-6 gate fix.
- `bcd0506` — Mark the PR ready for review, and record the widened local gate
  scope.
- `067accc` — Answer the seventh CodeRabbit review of the issue #85 work.
- `e540078` — Record the round-7 response in the ExecPlan.
- `07da3c3` — Replace the ephemeral queue identifiers with a durable artefact.
- `df9b380` — Record the round-7 re-gate as plan item 6, and correct two stale
  heads.
- `06bf3a4` — Record the three round-7 reconciliation lessons.
- `e456dc7` — Record the posted round-7 reply by its durable comment
  identifier.
- `56f3572` — Fix the MD049 emphasis style the Markdown gate caught in the
  round-8 ExecPlan text.

## Task 4 outcome: the splice is a macro, and why

The construction described below was correct about Verus and wrong about
Whitaker. A bare `include!` inside a function body _is_ verified — that part
held — but it also aborts Whitaker's `bumpy_road_function` lint with an
internal compiler error, which fails `make lint` for the whole crate.

The mechanism, established in a twenty-line reproduction crate rather than by
reading:

- Tokens spliced by `include!` report `span.from_expansion() == false` while
  still carrying the _included_ file's line numbers. `bumpy_road_function`
  skips expansion spans precisely because such line numbers can point outside
  the enclosing function, but that guard cannot see through `include!`. It
  therefore compares the body file's lines against the function's range in a
  different coordinate system and panics in `push_segment`
  (`crates/bumpy_road_function/src/driver/segment_builder.rs`).
- The trigger is _any_ `include!` inside a function body. An `include!` at item
  level is fine; a nested `include!` inside an included item is not. The lint
  has no `excluded_paths` support (unlike `no_std_fs_operations`), and every
  `allow`/`expect` form is rejected earlier by the attribute lints this
  repository already denies, so attribute suppression cannot reach it either.
- Expansion context is the discriminator the lint dispatches on. Sharing the
  body as a `macro_rules!` definition gives the tokens the expansion context
  the lint already skips, while the proof and the product still expand one text.

The shared text therefore moved from `verus/kernels/select_resolution.body.rs`
(a body fragment) to `verus/kernels/select_resolution.macro.rs` (a macro
definition), and both `verus/lib.rs` and `src/roadmap/ops/remap_kernel.rs`
expand `select_resolution_body!` and include the macro file at item level.

Confirmed after the change: `make verus` reports `4 verified, 0 errors`;
`make lint` exits 0 with zero ICEs and the Whitaker toolchain banner present
(so the suite genuinely loaded); and all six deterministic gates pass. The fix
was verified both cold (a run that recompiled the crate) and warm.

The trade this makes is recorded in the macro file and in
`docs/verification.md`: the kernel body is now opaque to `bumpy_road_function`
in both crates. Nothing is lost for a body holding one `if` inside one match
arm, but a future kernel with genuinely nested conditionals would go unflagged
by that lint.

## Task 4 design decisions (evidence-based)

The reference implementation is `leynos/mdtablefix` (ADR 0011 and
`docs/verification.md`). Its scaffolding — `tools/verus/VERSION`,
`tools/verus/SHA256SUMS`, `verus/smoke.rs`, the workflow shape — is already
present in this repository and byte-matches the reference. Its _proof_ content
is not transferable, because ADR 0011's "include the production module with
`#[path]`" convention does not actually verify anything. This was established
by direct experiment against the pinned Verus binary, not by reading:

| Probe                                                              | Result                                                                                                                                         |
| ------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Plain-Rust module `#[path]`-included, function called from a proof | `error: cannot use function ... which is ignored because it is either declared outside the verus! macro or it is marked as external`           |
| Same, with `--no-external-by-default`                              | Still unverifiable; a false assertion about the body is not caught                                                                             |
| Module carrying its own `verus!` block, `#[path]`-included         | Verified as a _separate crate item_; the module's own body is proved, but this forces `verus!` syntax into a production file                   |
| `include!` at item level inside `verus!`                           | Function is again "ignored" — the macro does not see it                                                                                        |
| **`include!` inside a function body inside `verus!`**              | **The included text is verified.** A false postcondition over the included body is caught, with the diagnostic pointing into the included file |

_Table 3: Verus splice probes and what each establishes._

So Verus verifies only text that is literally inside the `verus!` macro. A body
spliced by `include!` _is_ verified, and that same file is also plain Rust that
cargo compiles. The shipped construction refines this: the shared text is a
`macro_rules!` definition, included at item level and expanded in both the
production `const fn` and the verified `fn` (see "Task 4 outcome" above). The
probe result is untouched — text inside the macro is verified — while the
body-fragment splice is history rather than the current design.

Constraints confirmed by experiment:

- `vstd` and `builtin_macros` are **not** on crates.io, so production files
  cannot carry `verus!` syntax without breaking the ordinary build.
- Generic kernels with `Option<T>` verify, provided `T: Copy`.
- A non-private `spec fn` must be `pub open spec fn` (or `pub closed`).
- `#[must_use]` plus clippy's `missing_const_for_fn` (deny) means the
  production kernel must be `const fn`; Verus accepts the `const` form.

## Evidence recorded

- Pinned Verus binary: `.verus/0.2025.04.19.1b16620/verus/verus` (gitignored).
- Self-test marker for this binary:
  `verification results:: 0 verified, 1 errors`, exit status 1.
- `rust-prover-tools` is reachable at
  `https://github.com/leynos/rust-prover-tools`; the reference pins commit
  `2ba17da6b7e24160c4041e8d41ba1ba6cca87b35`.

## Completed work

1. Extracted the resolution decision into `src/roadmap/ops/remap_kernel.rs`,
   with its body shared via `src/roadmap/ops/select_resolution.macro.rs` and
   called from `RenumberPlan::resolve_reference`. The body is domain-owned and
   the proof reaches into `src/` for it, so the dependency runs from
   verification infrastructure to the domain kernel; see item 7.
2. Added `verus/lib.rs` as the proof entry point: identity preservation,
   local-mapping precedence, and non-vacuity — three theorems, all discharged,
   measured as `4 verified, 0 errors` (the three `proof fn`s plus the verified
   kernel `fn`).

   Two further obligations were drafted and then removed, and neither removal
   was discretionary. "Source-span preservation" concluded equality of results
   from pairwise-equal arguments, which Verus discharges from the signature
   alone, so no kernel defect could falsify it; a CodeRabbit review found that
   independently. "Deleted-target rejection" looked falsifiable but was not:
   its theorem assumed both options absent and concluded `None`, and with
   neither option holding a `T` there is no value to fabricate, so the
   conclusion is forced by the type rather than by the kernel. A six-defect
   battery established it — the theorem survived all six specification defects
   while each of its three neighbours rejected at least one. Table 2 of
   `docs/verification.md` records the defect that each surviving theorem
   rejects; deleted-target rejection is covered instead by the identity
   obligation in kernel form and by the `unresolved` collection in production.

   One body defect was injected as well, into the shared macro body alone with
   the specification left correct. Verus rejected it at the `select_resolution`
   `ensures` clause (4 verified, 1 error), which is the evidence that the proof
   is about the text the product compiles rather than about the specification.
3. Added the `docs/verification.md` ledger with a claim row per theorem, and
   `scripts/check-verification-ledger.sh` wired into `make lint`.
4. Wired `make verus` and `make verus-selftest`; the selftest runs a
   deliberately false proof and fails unless Verus rejects it.
5. Added `tests/verus_harness.rs` (this also discharged the outstanding
   CodeRabbit finding about `verus/smoke.rs` referencing targets that did not
   exist).
6. Added `.github/workflows/verus.yml`.
7. Moved the shared kernel body from `verus/kernels/` to
   `src/roadmap/ops/select_resolution.macro.rs`, so the dependency runs from
   verification infrastructure to the domain kernel rather than the reverse.
   This is what item 1 refers to. Round 8 of review asked for exactly that
   direction, and the reason it is worth the churn is that the earlier layout
   had production naming a proof path and doing a `CARGO_MANIFEST_DIR` lookup
   to reach it: the domain could not compile without knowing where the proof
   tree was. Now `remap_kernel.rs` includes a file beside itself and
   `verus/lib.rs` writes `../src/`, so the knowledge lives on the proof side
   where it belongs. Committed as `fe72e93`, with `make verus` at
   `4 verified, 0 errors` and the 39 harness tests green.

## Remaining work

1. Push and open the draft PR. **Done** — the branch is pushed and
   [PR #86](https://github.com/leynos/mapsplice/pull/86) is open as a draft.
2. Run `coderabbit review --agent` and clear all concerns. **Done for eight
   rounds** — 8, 6, 7, 7, 4, 6, 4 and 3 findings, every one actioned or
   dismissed with recorded evidence, counts re-derived from the artefacts
   (Table 2). Round 6 returned six findings that are four distinct issues,
   because two pairs are the same finding stated twice. Two were accepted and
   fixed (the ExecPlan's table captions were out of document order; the
   `verus.yml` concurrency comment described a `push` trigger the workflow does
   not have). Two were dismissed against evidence already on file: the ExecPlan
   rename re-raises round 1's finding with a new justification, and the
   ledger-fixture finding would undo what round 3's major finding asked for.

   Round 7 ran on the PR itself and returned `CHANGES_REQUESTED` for one inline
   finding and three pre-merge warnings. One inline finding was accepted (the
   ExecPlan still described the superseded `include!` splice as current); the
   developer-documentation warning was accepted after checking it against the
   guide's own conventions; the line-ending warning was partly accepted, its
   reasoning refuted but a real narrow gap closed with two CRLF tests; and the
   domain-architecture warning was dismissed, because its premise is wrong and
   its proposed remedy is the standalone reimplementation the issue rules out.

   The round-7 response is committed at `067accc` (the two tests, the guide
   paragraph and section 7.1), with the ExecPlan records at `e540078`. The
   inline finding's paragraph was rewritten at its own anchor, and the
   dispositions are triaged with measured evidence in
   `/tmp/cr-triage-round7.md`.

   Two facts about round 7 that a successor needs. First, it was a GitHub App
   review, not a local pass, so it is the first round with a thread to answer:
   comment `4113958501`, review `5328698796`. Second, CodeRabbit then **paused
   automatic reviews** on the branch (`auto_pause_after_reviewed_commits`),
   which is not the same as rate limiting — the paused walkthrough offers
   `@coderabbitai review` as the documented way back. Reconciliation was
   therefore enqueued deliberately rather than left to the automation: the
   focused reply first and the review request second, so the reviewer reads the
   dispositions before re-reviewing. The reply is posted as comment
   `5852783897` (2026-09-27T04:57:41Z, identity `buzzybee-df12`),
   byte-identical to the draft; the review request was enqueued behind it.
   Queue identifiers are ephemeral and are not recorded here —
   `comenq hist -n 20` shows the entries, the reply opening "Reconcile, please".

   A third party reviewed as well and needs no disposition. `sourcery-ai`
   declined the pull request for being over its 150,000-character diff limit (a
   size refusal, not findings), and `chatgpt-codex-connector` reported its code
   review as completed with no suggestions. Neither raised a thread.

   Continuous integration is green on the current head `07da3c3`: `build-test`
   (run `36294273124`) and `verify` (run `36294273062`) both succeeded,
   re-triggered by the push rather than started by hand. The merge state moved
   from `BLOCKED` to `CLEAN` once those landed, leaving the stale
   `CHANGES_REQUESTED` as the only remaining gate. The dispositions are
   recorded in `/tmp/cr-triage-round7.md`.
3. Follow the CI result for the PR. **Done** — the first run failed `make lint`
   (see the lesson below), and a later one failed `make spelling` on a commit
   hash written into the ExecPlan. Both are fixed. At the tip that was then
   current, `5e38bcb`, run `35871439602` reported `build-test success` and
   `35871439535` reported `verify success`, with the `Spelling` step reaching
   `refreshed: typos.toml` and no error. Head has moved since; see item 6.
4. The code gates for the branch as a whole. **Done locally at `08b62f9`.**
   The full set was run there — `check-fmt`, `lint`, `typecheck`, `test`,
   `markdownlint`, `nixie` and `spelling` — and five passed, with `check-fmt`
   and `markdownlint` failing on the ExecPlan text this branch had just added.
   Both were fixed, and all four Markdown gates were re-run green at `a720876`.
   `lint`, `typecheck` and `test` were green at `08b62f9` and nothing after
   that touched a Rust file or the `Makefile`, so they were not re-run; the
   committed `verus.yml` edit _is_ covered by them, because
   `tests/verus_harness/workflow.rs` reads that file through `include_str!` and
   again from disk. Logs are under `/tmp/<gate>-mapsplice-issue85-08b62f9.out`
   and `...-97da0e4.out`.
5. Local Markdown gates. **Done twice** — at `5e38bcb` (203 files left
   unchanged; `Linting: 60 file(s)`, `Summary: 0 error(s)`) and again at
   `a720876` after the round-6 text was added and formatted. Both runs were
   sequential, with HEAD unmoved and `typos.toml` reported in sync (`current:`)
   rather than drifting. This tracked 42 of the 61 paths the branch changes
   relative to `main`, which is why item 4 was widened rather than inherited
   from here.
6. Re-gate after the round-7 response. **Done at `067accc` and then at
   `07da3c3`.** The full nine-gate set ran first: eight passed and `check-fmt`
   failed, because `mdtablefix --wrap` wanted the new developer-guide prose and
   three ExecPlan paragraphs rewrapped. That is the finding the round-7 triage
   records as one the gates should have caught before the reviewer saw it.
   Reflowed with the gate's own `MDTABLEFIX_RULES` via `make markdownfmt`, and
   the four Markdown gates were re-run green — at `e540078` after the reflow,
   and again at `07da3c3` after the amend that replaced the ephemeral queue
   identifiers. `lint`, `typecheck`, `test`, `verus` and `verus-selftest` were
   _not_ re-run for either, because `git diff --name-only 067accc..07da3c3`
   lists exactly three paths — two Markdown documents and the regenerated
   `typos.toml` — so the compiled artefact is byte-identical and five gates
   cannot change verdict. `typos.toml` is regenerated by the `spelling`
   prerequisite of `make markdownlint` and committed rather than reverted,
   which is why every subsequent spelling run reports `current:` instead of
   `refreshed:`.
7. Round 8 of review, and the line-ending property coverage it required.
   Round 8 returned two rows that were live, and the second was the largest
   piece of work since the fix itself.

   **The inline finding** (`4114203677`) asked the CRLF rejection test to use
   `--in-place`. It was right and the gap was real: the test asserted
   byte-identity on the path that never writes, so it would have passed with a
   rejection path that normalized the target on its way to a write. Fixed at
   `ef8cda3` with a negative control — an in-place rejection leaves the file
   byte-identical, an in-place success rewrites it.

   **The linked-issues row** said `tests/roadmap_dependency_properties.rs`
   still generated LF-only targets, and it was correct. Closed at `bcc6e08` by
   giving the generator a `Shape::CRLF` and a `retarget` pass, threading a
   `crlf` parameter through `build_case`, and adding an independent `in_place`
   parameter so half the cases run on the path that actually writes. Line
   endings and mode are their own parameters rather than bits of a random byte,
   so every run generates both; a combination reachable only when a random byte
   happens to set one bit can go unexercised while the suite reports success.

   Two facts make this more than a coverage tidy-up. First, the old suite could
   not have caught the inline finding's defect: with every case forced to
   preview, the injected write-on-rejection defect passes. Only the mode
   dimension makes the byte-identity assertion bite, which was measured by
   injecting that defect both ways rather than argued. Second, the module doc
   saying "line endings are deliberately LF-only … the parser normalizes CRLF
   to LF on render" was doubly stale — the modes now vary, and the mechanism it
   named does not exist. There is no parse-time normalization anywhere in the
   product; the renderer joins canonical lines with `\n` while preserved spans
   are emitted as raw byte slices, which is why an accepted edit can return a
   document holding both conventions.

   The assertions were split so the two obligations cannot be conflated.
   Content checks (a summary renders as `{anchor}. {stem}`, a clause survives
   outside its fences) normalize `\r\n`; the preservation check compares raw
   bytes and is the only one that may. The assertion layer moved to
   `tests/support/dependency_case.rs` for that separation and to keep every
   file under the 400-line limit.

   **The domain-architecture row's third part** asked for a gate preventing
   infrastructure path or environment access in `src/roadmap`. The first two
   parts landed at `fe72e93`; this one had no implementation, so the rule it
   states was still only prose. `scripts/check-domain-purity.sh` now enforces
   it, wired into `make lint` beside the verification-ledger check. It matches
   `use` declarations line-anchored with the root spelled out, so
   `use crate::fs;` is caught alongside `use std::fs;` — the offence is
   reaching the filesystem, not which root names it — and matches module paths,
   process calls, and the build-time environment and file macros wherever they
   appear. `src/roadmap` is free of all of it today, so the gate passes and now
   fails on the exact historical defect:

   ```text
   include!(concat!(env!("CARGO_MANIFEST_DIR"), "/verus/..."));
   ```

   The gate was verified by injection rather than by argument: seventeen
   separate defects — the crate-rooted and `std::` import forms for `fs`,
   `path`, `process` and `io`, `env!`, `option_env!`, `include_str!`,
   `include_bytes!`, a qualified `std::fs::` call, a bare `current_dir()`,
   `std::env::args()`, `std::env::var()`, and the historical
   `concat!(env!(...))` — each drove it to exit 1, and an empty domain tree and
   a missing domain directory were both refused rather than passing vacuously.
   Test files are exempt by name, because `render_tests.rs` drives the built
   binary; the exemption is by file rather than by region so no brace count has
   to be trusted, and the contract tests (`tests/domain_purity.rs`, nine cases)
   assert it from both sides.

   Commit `0ae84d1` also clears what the first gate run on `bcc6e08` reported:
   two `clippy::shadow_reuse` bindings and a denied integer division in the
   property-test support, and the Markdown rewrap `mdtablefix --check` asked
   for. All four gates that failed on `bcc6e08` pass on `0ae84d1`, and the
   Whitaker stage ran for real: its toolchain banner is present and the
   `No libraries were found` warning appears zero times, which is the check
   that distinguishes a genuine Dylint pass from an empty `DYLINT_LIBRARY_PATH`
   exiting 0 while doing nothing.

   **A caution learned here.** The first run's report noted that Clippy's
   failure aborted the `lint` recipe before Whitaker, so the Dylint suite was
   _unexecuted_ rather than passing. That distinction is the reason the fix was
   followed by a second full run rather than by a claim of a clean gate set.

8. The domain-architecture row's third sub-item, and the gate run that followed
   it. **Done at `0ae84d1`.** Full gate run on `bcc6e08` (run
   `/tmp/lint-mapsplice-issue85-bcc6e08.out` and siblings): `typecheck`, `test`
   (365 nextest cases, 12 doctests), `nixie`, `verus` (`4 verified, 0 errors`)
   and `verus-selftest` passed; `check-fmt`, `lint`, `markdownlint` and
   `spelling` failed. The failures were deterministic and are all fixed: four
   Clippy errors in the property-test support, three `-ise` spellings, and
   `mdtablefix --check` on three Markdown files. The run also established that
   the Whitaker stage had never executed, because Clippy aborted the recipe
   before it — recorded as unexecuted rather than passed. Gate run two on
   `0ae84d1` covers the same nine targets plus the new gate; see
   `/tmp/<gate>-mapsplice-issue85-gate2.out`.

   **A process error in that second run, recorded because it changed a rule.**
   The run was launched before `0ae84d1` was committed, so its first two gates
   executed against a tree that still held the fix and the new gate as
   uncommitted edits — content-identical to the commit, but reached by a
   different path, since `mdtablefix` selects files by
   `git ls-files --cached --others --exclude-standard` and an untracked script
   is not in any gate's path list. The three Markdown gates had no such
   wrinkle. Then the ExecPlan commit `1ce921e` landed at 08:42:26, while
   `check-fmt` had finished at 08:41:48: the `check-fmt` pass therefore did not
   see the ExecPlan text that commit added. That text is Markdown this branch
   authors, and `mdtablefix --check` is exactly what rejected the previous such
   edit, so the pass could not be inherited. `check-fmt`, `spelling` and
   `markdownlint` were re-run on the restored, fully committed tree; the first
   two pass and the third is the golden-fixture-inclusive run. The rule is now
   applied without exception: **no tracked edit may land while a gate run owns
   the tree, and a gate result is only evidence for the tree state whose hash
   is in its verdict line.** A run started on uncommitted work is a label in a
   log, not a candidate.

## Lessons

- **A local gate run can be vacuous about a later step in the same target.** An
  early `make lint` failure (a denied `%`) aborted the target two steps before
  Whitaker, so the run's silence about the Dylint suite proved nothing. A gate
  is only green when it reaches its last step; the log was checked for the
  Dylint banner rather than for a zero exit alone.
- **A new prerequisite for `make lint` is a new CI prerequisite.** Wiring
  `check-verification-ledger` into `lint` made `rg` a hard dependency of the
  lint job. It is present locally, so the gate passed here and failed on the
  runner. The install step now provides it.
- **The formatter has to be run with the gate's own flags, or it is a different
  tool.** `make check-fmt` runs `mdtablefix --check` over its selected files
  _with the flags held in `MDTABLEFIX_RULES`_. Fixing a table-alignment
  complaint with a bare `mdtablefix --in-place <file>`, without those flags,
  produced a file that looked repaired and failed the very next `check-fmt` —
  the alignment was fixed but the paragraph wrapping the default rules leave
  alone was not. The second attempt passed the rule set and succeeded. The
  flags are declared once in the Makefile, so they are visible rather than
  hidden; the mistake was to reach for the bare command instead of reading how
  the gate calls it. Run the gate's command, not the tool.
- **Two failures in one file can have one cause, and fixing the second can
  reintroduce work in the first.** Seven `markdownlint` errors and a
  `check-fmt` complaint all landed in this file at once. Converting the three
  `*emphasis*` spans to `_emphasis_` shortened those lines, which re-flowed a
  paragraph `--wrap` had already settled, so `check-fmt` failed again after the
  MD049 fix. Format and lint are not independent when the formatter re-packs
  lines to a column limit: run the formatter _last_, after every content edit,
  and re-run it if any edit lands afterwards.
- **A fixture can be a test of nothing.** The C2 golden case originally deleted
  the _last_ task, which shifts no later number and so rewrote no clause: it
  was a test that a trailing delete disturbs nothing, which is true but not
  what the case name claimed. It now deletes a middle task, so renumbering
  actually happens while the incidental prose is held under test.
- **Not every clause position of a theorem is falsifiable.** Two obligations
  were removed after a defect battery showed no defect could falsify them. The
  ledger records the battery; the proofs state three obligations, not five.
- **An unreachable bug is still a bug, and reachability must be checked rather
  than assumed.** Two oracle defects survived every review round that ran after
  they were written — five rounds for the identity helper that divided by the
  wrong constant twice, four for the summary match that accepted a prefix —
  because every input the generator produces happens to mask them. Neither
  could fail on today's inputs. Both were fixed, because the cost of the fix is
  a few lines and the cost of the failure is a test suite that passes more
  easily while asserting less. The distinction between "live" and "reachable"
  was established by enumerating the generated inputs, not by reading the code
  and judging it unlikely.
- **A review finding's own framing can be wrong.** CodeRabbit described the
  identity-helper change as preserving existing behaviour. It does not: the two
  readings differ for any step beyond the first few. The finding was right that
  something should change and wrong about why, so the reasoning was checked
  against the code before the edit, and the record says which part was taken.
- **A count in a plan is a claim, and this one was wrong twice over.** The round
  tally stood at "four rounds (7, 7, 7, 4 findings)" and was carried from the
  plan into the pull-request body without being re-derived. Both halves were
  false. There were five rounds, and the per-round counts are 8, 6, 7, 7 and 4,
  for 32 findings in total. The two errors nearly cancelled: the wrong
  breakdown summed to 25, which is what the correct count was wrongly recorded
  as, so a re-read of the total confirmed a number that was itself groundless.
  (A sixth round has since run, so the live tally is 8, 6, 7, 7, 4, 6 for 38;
  Table 2 is the current record. This paragraph is left as written because it
  is the record of what the correction established at the time, and the lesson
  it draws would be weakened by editing the numbers that make the point. The
  paragraph has already gone stale once for exactly the reason it describes,
  which is the cleanest illustration of it available.)
- **A missing round announces itself as a count that does not close.** Five
  commits name CodeRabbit; only four review artefacts could be found. That
  one-line reconciliation was available from the start and would have caught
  the omission immediately, where re-reading the tally did not. Prefer a check
  that compares two independently-derived numbers over a re-read of the number
  in question.
- **The same flaw has a mirror, and it is worse, because it inflates.** Table 2
  briefly carried a ninth row — `| 9 | 5 | 0ae84d1 | ... (pending) |` — written
  while recording the domain-purity gate, before any ninth review existed. The
  PR has only ever carried two CodeRabbit review submissions — `5328698796` on
  `bcd0506` and `5328960095` on `e456dc7` — and no ninth-review artefact
  appears under `/tmp` by content or by name. The row was not invented
  wholesale: the work it pointed at was real and the commit was real, but that
  work answers round 8's third finding, which item 8 of Remaining work already
  records. Three signals were visible at the time and none was read — the
  caption still said "the eight CodeRabbit rounds", the running tally in the
  prose still listed eight numbers, and the row's own evidence cell said
  "(pending)", which for a table captioned _re-derived from the artefacts_ is
  an admission that nothing had been derived. A tally that does not close is
  the safe failure; a tally that closes on a row with no artefact behind it
  passes every re-read, because the arithmetic works. Anchor each row to an
  artefact that exists before the row is written, not to work that needs
  recording somewhere. The row also exposed a second defect next to it: round
  8's "Answered by" cell named only `ef8cda3`, though its three findings landed
  across four commits — `ef8cda3`, `bcc6e08`, `fe72e93` and `0ae84d1`. A cell
  that names one commit for a round that needed four reads as a settled fact
  and sends a successor to the wrong diff.

  The count that refuted the row was itself nearly misread the same way.
  `gh api …/pulls/86/reviews` returns every reviewer's submission, and the
  unfiltered list has _three_ entries — the two CodeRabbit reviews and one
  `sourcery-ai` comment — so taking its length gave a number that was right
  about the API and wrong about the claim it was used to settle. Count by
  author, not by page length; an aggregating endpoint answers a question about
  the page, not about the reviewer.
- **Widening a table cell re-pads the whole table, so probe it on a copy.** The
  round-8 cell above had to name four commits instead of one. Editing it in
  place would have left every other row's padding stale, and
  `mdtablefix --check` would have failed the gate — the same class of coupling
  between content and formatting that the emphasis fix already demonstrated.
  The edit was rehearsed on a `/tmp` copy, the formatter was run there, and the
  resulting table was ported back already formatted; the probe then passed
  `--check` unchanged. This is the general form of "run the gate's command, not
  the tool": when an edit changes a line's length inside a formatted construct,
  produce the formatted result first and move that, rather than moving the
  content and hoping the formatter agrees.
- **The evidence glob was narrower than the evidence.** Round artefacts were
  enumerated with `ls /tmp/*coderabbit*issue-85*`, which found four of the
  five. The fifth was captured only inside a subagent task-output file under
  `/tmp/claude-1000/.../tasks/`, which any `/tmp` glob rooted at the top level
  cannot see. The four that did match were then indexed against the wrong
  response commits — the file named `.out.round3` actually holds the round
  answered by `67ab4b9`, which the plan's own commit list did not mention.
  Enumerate by content, not by a filename pattern that only holds when the
  convention was followed. Measured, rather than assumed: the narrower pattern
  `/tmp/coderabbit-*.out` reaches only two of the five; the one actually used
  reaches four; a content sweep for the branch name **and** a review marker
  (`Review completed` or `"severity"`) reaches all five, as six file hits,
  because round 2 was captured twice. Two cautions, both measured against this
  machine rather than reasoned about: the branch name alone is worthless as a
  key — it matches 49 files here, including other sessions' gate logs and every
  `tee` output on the branch — and the doubled hit means hits must be
  deduplicated by comparing finding lists before they are counted as rounds.
  The authoritative re-derivation matched each artefact's findings against the
  files each response commit touched, rather than trusting timestamps or
  filenames.
- **Two unrelated things shared the label "round 3", and correcting the first
  error caused a second.** The note file `cr-triage-round3.md` and the artefact
  file `.out.round3` are different rounds. The first correction asserted that
  the note documented `67ab4b9`'s round on the strength of the filename match
  alone; the file-touch check showed the note's F1–F7 table corresponds to
  `fcbe9c6` and the note is round 4, not round 2. A correction written from the
  same faulty signal as the error it fixes is not a correction.
- **A scope decision that is right for the diff can still understate the
  branch.** Gating the seven commits after `92dc9bb` as docs-only is correct —
  they touch no code — but "these gates pass" then means something narrower
  than it sounds, because the branch also changes 42 non-Markdown paths whose
  last local gate run was earlier. The green result was read next to a CI run
  covering the full set at the same commit, which is what made the narrow scope
  sound. State which gates ran and which were replaced by other evidence, as
  the Remaining work section now does, rather than letting a clean local result
  imply coverage it did not have.
- **Writing a commit hash into prose can fail the spelling gate.** `5ba0ab2`
  is a valid revision, but Typos splits the hex string into letter-runs and
  reads the middle two characters of that run as a typo — a misspelling of a
  short English word. The failure surfaced twice: locally in `make spelling`,
  and in CI at 13:53 (run `35869927771`, the `Spelling` step), both naming the
  same line. Two things are worth keeping. The exemption went into
  `typos.local.toml` — the tracked input — rather than `typos.toml`, which is
  generated and says so; the generated file is committed alongside it because
  the builder reports the drift otherwise. And the pattern was scoped to the
  backticked 7–40 hex form rather than to that one revision, with a negative
  control run in a scratch repository to show the wider scope is still safe: a
  common sending-verb misspelling and the US spelling of `colour` both kept
  failing while the SHA was ignored. Of the 33 backticked hex tokens in the
  tracked docs, all are commit or Verus hashes and none is an English word, so
  the form is the right thing to exclude. The lesson's own first draft wrote
  the misspelling out as an example and was itself rejected by the gate — which
  is the gate behaving correctly, and a small demonstration that quoting an
  error verbatim is not free.
- **A grep over a freshly-written file can compare it with itself.** Checking
  that the pull-request body matched the local draft, the first attempt wrote
  the live body to a file inside a pipeline and then diffed that same file
  against the draft. It printed `IDENTICAL` while the byte counts in the very
  same output differed by 26. The bytes disagreed with the verdict, and the
  bytes were right: the extraction had added a trailing newline. When two
  measurements of the same thing disagree, that is the finding — re-measure by
  an independent route (here, hashing both sides) rather than trusting the one
  that agrees with what was expected.
- **A re-review can re-raise a settled finding, and a _checkable_ new reason
  deserves a check rather than a repeat of the old argument.** Round 6 asked
  again for the ExecPlan to be renamed to `docs/execplans/roadmap-1-2-1.md`,
  which round 1 had already raised and which the round-1 triage had already
  refused. The difference was the justification: round 6 said the rename was
  needed "so the renumber planner can classify and carry the plan correctly".
  That is a factual claim about the repository and it is false in two
  independent ways, both one command from being shown so —
  `git grep -n "execplans" -- src/` finds no such subcommand, and
  `git grep -n "execplans/issue-85" -- .` finds no reference to the file. There
  is no planner, no classifier, and no link. Re-arguing the original refusal
  would have been the wrong move twice over: it would have treated a new claim
  as though it were the old one, and it would have left the claim standing
  unrefuted. When a re-raise arrives with a reason that can be tested, test it.
  When the reason turns out to be false, that is the strongest possible form of
  the dismissal — and the weakest possible reason to change the code.
- **Two of round 6's six findings were the same finding stated twice, so the
  round's own count is not its issue count.** The ExecPlan rename came back
  once as "rename to the conventional roadmap name for item 1.2.1" and once as
  "rename the file to `docs/execplans/roadmap-1-2-1.md`"; the ledger-fixture
  finding came back twice differing only in a subordinate clause. Six findings,
  four issues. Deduplicate by the change requested, not by the number the
  report prints — the count in a summary line is a measure of how many times
  the scanner spoke, not of how many defects exist.
- **A comment can be the defect, and the honest fix can be smaller than the
  one requested.** Round 6 noticed that `verus.yml`'s concurrency comment
  mentions "a push to `main`" and asked for a push trigger to be added so the
  comment would be true. Measured across the repository,
  `grep -rn "push:" .github/workflows/` returns nothing: no workflow here
  triggers on push, including `ci.yml`. Adding one to `verus.yml` on its own
  would have made proofs run on every merge to `main` while lint and the test
  suite did not — trading a comment's inaccuracy for a CI layout that is harder
  to reason about and still inaccurate about every other workflow. The comment
  was the defect; the triggers were correct as they stood. Where a finding
  identifies two things that disagree and only one of them is wrong, the work
  is to determine _which_, not to move the one that is easier to move.
- **Three of round 6's findings pointed at two files spelled with the same
  table numbers, and only two of the three could be right.** The ExecPlan had
  its captions out of document order (`Table 3` above `Table 2`), while line
  171's "Table 2 of `docs/verification.md`" is a cross-_document_ reference
  into the ledger's own `Table 2` and was correct. A search-and-replace over
  "Table 2"/"Table 3" would have swapped all of them and introduced exactly the
  defect the round was reporting. The review had scoped its own suggestion to
  the local captions and the local cross-reference, and that scoping was
  correct — worth noting because a plausible-looking mechanical fix here was
  the one that would have broken something.
- **A pre-merge warning can be identified by a check while its stated reason is
  false, and the useful work is separating the two.** Round 7's Linked Issues
  check reported that the line-ending variation issue #85 asks for was missing.
  Two of its three supporting claims were wrong: CRLF targets _are_ generated
  elsewhere in the suite (`tests/support/task_source_properties.rs:60,114`),
  and the issue does not require a byte-identical CRLF round trip, because F3
  promises identity modulo the documented normalization. But the third claim —
  that CRLF _with a nested-bullet clause_ was untested — was true, and it is
  the intersection this issue is about. Refuting the reasoning and then
  stopping would have left a real gap standing on the strength of a good
  argument. The gap was closed with two tests instead. A finding's
  justification and its claim are separate objects; each has to be checked on
  its own.
- **A negative control is what makes a new test's fixture trustworthy.** The
  CRLF fixture is written with `concat!` and `"\r\n"` escapes, and an escape
  lost in editing degrades it silently to `"\n"` — a CRLF test built on an LF
  fixture passes while testing nothing. Before the tests were accepted, the
  fixture was checked to hold nine CRs with every LF preceded by a CR, and an
  expected anchor was flipped to confirm the assertion actually fails. The
  second check is the one that catches a test that asserts nothing; the first
  catches a fixture that is not what its name says. Both are cheap, and both
  were needed here because the whole point of the test is a byte that is
  invisible when it is right.
- **An architecture finding can be right about a mechanism and wrong about its
  consequence.** Round 7 reported that `src/roadmap/ops/remap_kernel.rs` uses
  `env!("CARGO_MANIFEST_DIR")` and `include!` to reach into `verus/`, which is
  accurate and is the only such usage in `src/`. The consequence it drew — a
  domain-to-infrastructure dependency — does not follow: both operands are
  compile-time, and nothing about resolution consults a repository path while
  the program runs. The proposed remedy was worse than the finding: a "separate
  proof adapter" is a standalone reimplementation, which is what `verus/lib.rs`
  forbids and what issue #85 excludes as "not sufficient". When a finding
  proposes removing a mechanism, the question is not whether the mechanism is
  ugly but whether the thing it exists for still gets done. Here the mechanism
  exists precisely because the issue requires the proof to reach the production
  kernel.
- **A finding can arrive as a GitHub App review rather than a local run, and the
  two have different failure modes.** Rounds 1-6 were local `review --agent`
  passes; Round 7 was the App's own review, auto-triggered by marking the PR
  ready. It posted a `CHANGES_REQUESTED` review, three pre-merge checks, and a
  walkthrough that auto-paused itself after an influx of commits. The suite of
  surfaces to read is therefore larger than the findings text: review
  submissions carry the `commit_id` they assessed, pre-merge checks carry their
  own resolutions, and the walkthrough is edited in place rather than appended.
  Reading only the inline comments would have missed three of the four findings.
- **"Reviews paused" is not "ratelimited", and the difference decides the
  remedy.** The round-7 walkthrough reports reviews paused because of an influx
  of commits (`auto_review.auto_pause_after_reviewed_commits`), which is a
  different condition from the rate limit that would call for sleeping and
  retrying. Neither condition, however, reconciles anything on its own: with
  automatic reviews off, the stale `CHANGES_REQUESTED` would have blocked merge
  indefinitely. The documented way back is `@coderabbitai review`, so the reply
  and the review request were both enqueued deliberately rather than left to
  the automation. Read the walkthrough's own wording before choosing a remedy —
  two conditions that both look like "the reviewer has stopped" need different
  responses.
- **A comment queued before the final push describes a head that no longer
  exists.** The reconciliation reply was queued three times: once naming
  `e540078` and disclaiming a review request, then again naming `07da3c3` after
  a fixup, then finally on a frozen `df9b380`. Each earlier draft became false
  in a different way — a superseded head, a claim about which gates had been
  re-run, a statement that no new review was requested when one was. In
  between, the PR body had to be rewritten twice and the ExecPlan's recorded
  queue identifiers were invalidated by re-enqueueing. The sequence that works
  is: finish every edit, push, run the gates _and_ watch CI on that exact
  commit, and only then queue the comment. Queueing is the last step of a
  round, not a step that can overlap one.
- **A sentence in a document that is true of a named commit must be checked
  against that commit, not against the working tree.** The reply's
  inline-finding claim cited `execplan:139` and named `## Task 4 outcome` as
  living at line 74. Both were correct for `bcd0506`, the commit the reviewer
  anchored to — the line really did hold the stale paragraph — but by the time
  the reply was written the file had grown and been reformatted, so the section
  sat at line
  1. Anchors are per-commit facts; the reply now cites the section by name and
  binds the line number to its commit explicitly.

## Constraints that must hold

- No `assume`, admitted lemma, or `external_body` that merely restates the
  invariant under proof.
- Frame the risk honestly: the _clause recognizer_ is the large surface here,
  and it is not proved. `docs/verification.md` states that boundary rather than
  implying the recognizer is verified.
- The macro wrapper means the kernel body is opaque to `bumpy_road_function`.
  This is a real, if narrow, loss of lint coverage, and it is recorded in both
  the macro file and the ledger rather than left implicit.
- Byte offsets and Unicode scalar indices are distinct types in any spec that
  touches them.
