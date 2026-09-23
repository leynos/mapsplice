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

Commits:

- `05b49d3` — Address CodeRabbit review findings on the nested-Requires work.
- `7b9032b` — Extract nested-Requires regressions to honour the 400-line file
  rule (#85).
- `b7fdcda` — Scope dependency property assertions to each item's own block.
- `55e3484` — Extract dependency resolution into a production-used Verus kernel.
- `9a9b7c8` — Wire the Verus harness into the build and continuous integration.
- `9202dbf` — Share the verified kernel body as a macro, not an include splice.
- `fcbe9c6` — Answer the third CodeRabbit review of the issue #85 work.
- `92dc9bb` — Answer the fourth CodeRabbit review of the issue #85 work.

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

_Table 2: Verus splice probes and what each establishes._

So Verus verifies only text that is literally inside the `verus!` macro. A body
spliced by `include!` _is_ verified, and that same file is also plain Rust that
cargo compiles. That is the construction used here: one body file, spliced into
the production `const fn` by cargo and into the verified `fn` by Verus.

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
   with its body shared via `verus/kernels/select_resolution.macro.rs` and
   called from `RenumberPlan::resolve_reference`.
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

## Remaining work

1. Push and open the draft PR. **Done** — the branch is pushed and
   [PR #86](https://github.com/leynos/mapsplice/pull/86) is open as a draft.
2. Run `coderabbit review --agent` and clear all concerns. **In progress** —
   four rounds have run (7, 7, 7, 4 findings) and every finding is actioned or
   dismissed with recorded evidence. Re-run once the current round's fixes land.
3. Follow the CI result for the PR. The first run failed `make lint`; see the
   lesson below. The fix is in and the next run is the check on it.

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
- **A fixture can be a test of nothing.** The C2 golden case originally deleted
  the _last_ task, which shifts no later number and so rewrote no clause: it
  was a test that a trailing delete disturbs nothing, which is true but not
  what the case name claimed. It now deletes a middle task, so renumbering
  actually happens while the incidental prose is held under test.
- **Not every clause position of a theorem is falsifiable.** Two obligations
  were removed after a defect battery showed no defect could falsify them. The
  ledger records the battery; the proofs state three obligations, not five.
- **An unreachable bug is still a bug, and reachability must be checked rather
  than assumed.** Two oracle defects survived four review rounds because every
  input the generator produces happens to mask them: an identity helper that
  divided by the wrong constant twice, and a summary match that accepted a
  prefix. Neither could fail on today's inputs. Both were fixed, because the
  cost of the fix is a few lines and the cost of the failure is a test suite
  that passes more easily while asserting less. The distinction between "live"
  and "reachable" was established by enumerating the generated inputs, not by
  reading the code and judging it unlikely.
- **A review finding's own framing can be wrong.** CodeRabbit described the
  identity-helper change as preserving existing behaviour. It does not: the two
  readings differ for any step beyond the first few. The finding was right that
  something should change and wrong about why, so the reasoning was checked
  against the code before the edit, and the record says which part was taken.

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
