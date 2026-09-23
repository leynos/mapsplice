# ExecPlan: Nested `Requires` bullets bypass dependency rewriting and validation (issue #85)

## Purpose

Issue #85: a `Requires` clause written as a nested task-body bullet was
silently ignored during renumbering and dangling-dependency validation.
Insertion redirected a consumer to an unrelated new task; deleting a
prerequisite produced a self-dependency; in-place deletion wrote that invalid
result to disk.

Tasks 1–3 of the coding plan are complete and committed. This plan tracks Task
4 (Verus proofs and CI wiring) and the PR.

## Progress so far

| Task | Scope                                                | State       |
| ---- | ---------------------------------------------------- | ----------- |
| 1    | Recognizer fix across all four clause positions      | Done        |
| 2    | Document the supported clause grammar and exclusions | Done        |
| 3    | CLI regressions, golden fixtures, property suite     | Done        |
| 4    | Verus proofs, ledger, Makefile and CI wiring         | In progress |

Commits:

- `05b49d3` — Address CodeRabbit review findings on the nested-Requires work.
- `7b9032b` — Extract nested-Requires regressions to honour the 400-line file
  rule (#85).
- `b7fdcda` — Scope dependency property assertions to each item's own block.

## Task 4 design decisions (evidence-based)

The reference implementation is `leynos/mdtablefix` (ADR 0011 and
`docs/verification.md`). Its scaffolding — `tools/verus/VERSION`,
`tools/verus/SHA256SUMS`, `verus/smoke.rs`, the workflow shape — is already
present in this repository and byte-matches the reference. Its *proof* content
is not transferable, because ADR 0011's "include the production module with
`#[path]`" convention does not actually verify anything. This was established
by direct experiment against the pinned Verus binary, not by reading:

| Probe                                                              | Result                                                                                                                                         |
| ------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Plain-Rust module `#[path]`-included, function called from a proof | `error: cannot use function ... which is ignored because it is either declared outside the verus! macro or it is marked as external`           |
| Same, with `--no-external-by-default`                              | Still unverifiable; a false assertion about the body is not caught                                                                             |
| Module carrying its own `verus!` block, `#[path]`-included         | Verified as a *separate crate item*; the module's own body is proved, but this forces `verus!` syntax into a production file                   |
| `include!` at item level inside `verus!`                           | Function is again "ignored" — the macro does not see it                                                                                        |
| **`include!` inside a function body inside `verus!`**              | **The included text is verified.** A false postcondition over the included body is caught, with the diagnostic pointing into the included file |

So Verus verifies only text that is literally inside the `verus!` macro. A body
spliced by `include!` *is* verified, and that same file is also plain Rust that
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

## Remaining work

1. Extract the resolution decision into `src/roadmap/ops/remap_kernel.rs` with
   its body in `verus/kernels/`, called from `RenumberPlan::resolve_reference`.
2. Add `verus/lib.rs` as the proof entry point, proving:
   - identity preservation (a target-text reference never resolves through the
     cross-source fallback);
   - local-mapping precedence;
   - deleted-target rejection (a reference to a retired anchor does not resolve
     to the replacement that inherited its number);
   - source-span preservation for the untouched-text path.
3. Add `docs/verification.md` ledger with a claim row per theorem.
4. Add the ledger checker and wire `make verus` / `make verus-selftest`.
5. Add `tests/verus_harness.rs` (this also discharges the outstanding
   CodeRabbit finding about `verus/smoke.rs` referencing targets that did not
   exist).
6. Add `.github/workflows/verus.yml`.
7. Push and open the draft PR.

## Constraints that must hold

- No `assume`, admitted lemma, or `external_body` that merely restates the
  invariant under proof.
- Frame the risk honestly: the *clause recognizer* is the large surface here,
  and it is not proved. `docs/verification.md` must state that boundary rather
  than implying the recognizer is verified.
- Byte offsets and Unicode scalar indices are distinct types in any spec that
  touches them.
