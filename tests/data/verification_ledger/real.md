# Verification ledger

`mapsplice` uses Verus to prove narrow kernels that the renumbering and
dependency-rewrite path calls in production. This ledger records every claimed
proof result, its executable function, input domain, external contracts, and
result class. A kernel may not be represented by a separate `proofs/`
implementation unless a refinement proof connects it to the production function.

## Claim ledger

| Claim                                                                                            | Executable function | Input domain                                                                                       | Unverified external contracts            | Result class      |
| ------------------------------------------------------------------------------------------------ | ------------------- | -------------------------------------------------------------------------------------------------- | ---------------------------------------- | ----------------- |
| Identity preservation: a target-text reference never resolves through the cross-source fallback  | `select_resolution` | `Option<T>` and `Option<T>` for any `T: Copy`, plus a `bool` source flag; no allocation, no lookup | None: the function has no external calls | Local correctness |
| Local-mapping precedence: a target-text reference with its own mapping keeps it                  | `select_resolution` | As above                                                                                           | None                                     | Local correctness |
| Deleted-target rejection: a reference with neither mapping resolves to nothing, in either source | `select_resolution` | As above, with both options absent                                                                 | None                                     | Local correctness |
| Source-span preservation: resolution depends on nothing but its three inputs                     | `select_resolution` | Two triples of equal inputs                                                                        | None                                     | Local correctness |
| Non-vacuity: fragment text does consult the cross-source fallback                                | `select_resolution` | As above, with the local option absent and the cross-source option present                         | None                                     | Local correctness |

_Table 1: The verification claim ledger._

Each row is discharged by a theorem in `verus/lib.rs`, which runs under
`make verus`. The titles above name the obligation; the theorem's comment in
that file states its premise. `make verus-selftest` runs `verus/smoke.rs`,
whose assertion is deliberately unprovable, and fails unless Verus rejects it —
so a run in which the verifier never started cannot pass as a successful one.
`scripts/check-verification-ledger.sh`, wired into `make lint`, fails when a
claim names a function that no longer exists in `src/`.

## What is proved, and what is not

The kernel `select_resolution` decides **which of two already-computed answers
a dependency reference resolves through**. That is the whole of its
responsibility, and it is where issue #85's identity-preservation rule lives:
before the fix, a consumer could be silently re-pointed at an unrelated item
that had inherited its prerequisite's number.

**The clause recognizer is not verified.** This is the honest boundary and the
larger surface. Deciding whether a given stretch of Markdown _is_ a `Requires`
clause — the nested-bullet, continuation, and inline positions, the
anchor-token grammar, the section-sigil and clause-terminator exclusions — is
performed by `src/roadmap/ops/dependency_text.rs` and the mdast walk in
`src/roadmap/ops/rewrite.rs`. Neither is inside any proof boundary. Nothing in
this ledger should be read as evidence that the recognizer is correct. The
defect tracked as <https://github.com/leynos/mapsplice/issues/85> was in the
recognizer, and the recognizer is covered by the CLI regressions, golden
fixtures, and property suite in `tests/` instead. Naming it as a link rather
than as a bare number keeps it from being read as a heading wherever the
paragraph happens to wrap.

Extracting the recognizer into a provable kernel is not attempted here, and
would be a substantial piece of work: it consumes mdast nodes and byte strings
through several helpers whose contracts would each need stating. Recording the
gap is preferable to implying a coverage that does not exist.

## How the kernel connects to production

`verus/lib.rs` proves the macro-defined body in
`verus/kernels/select_resolution.macro.rs`. `src/roadmap/ops/remap_kernel.rs`
includes that same file and expands `select_resolution_body!` inside its
`const fn`, and `RenumberPlan::resolve_reference` in `src/roadmap/model.rs`
calls it. The verified text and the compiled text are therefore one artefact,
not two implementations that can drift.

Sharing the text is deliberate rather than incidental. Verus treats a
plain-Rust module included with `#[path]` as opaque — it cannot be called from
a proof at all — so a proof that reaches production code must bring the body in
as text. `verus/lib.rs` documents the convention.

That text is a `macro_rules!` definition rather than a bare body fragment, and
the indirection is load-bearing. Whitaker's `bumpy_road_function` lint cannot
see through `include!`: spliced tokens report `span.from_expansion() == false`
while still carrying the _included_ file's line numbers, so the lint compares
them against the enclosing function's range in a different coordinate system
and aborts the compiler with an internal error. That abort is not a warning to
be silenced — it fails `make lint` outright, for the whole crate, on any
`include!` inside a function body. Expanding the shared text as a macro gives
it the expansion context the lint already skips, which is what lets the two
sides share one text at all.

The trade is recorded in `verus/kernels/select_resolution.macro.rs`: the kernel
body is opaque to `bumpy_road_function` in both crates. Nothing is lost here —
the body holds one `if` inside one match arm — but a future kernel with
genuinely nested conditionals would go unflagged by that lint.

`RenumberPlan::resolve_reference` delegates the whole decision, including the
fragment check. It deliberately does **not** pre-filter the cross-source value
for target text: doing so would enforce the rule in the caller instead and
leave the kernel's fragment guard unreachable in production, making the proofs
statements about dead code.

## Policy

- No `assume`, admitted lemma, or `external_body` may assert the property under
  proof. The kernel above needs none: it calls nothing and inspects no
  infrastructure.
- Byte offsets and Unicode scalar indices are distinct concerns. No claim in
  this ledger inspects text, so none conflates them; a future text-handling
  kernel must keep them as distinct types and must not treat `String::len()` as
  a display or scalar width.
- Filesystem and I/O assumptions are outside every proof boundary. Resolution
  is pure, so the proofs say nothing about reading a roadmap, writing output,
  or the in-place write path. Those are covered by the golden and CLI tests.
- Existing property tests remain in place. Verus proofs complement them and do
  not replace them.
- A proof's obligations are only meaningful if they can fail. Each theorem in
  `verus/lib.rs` was checked by injecting a kernel defect and confirming the
  proof is rejected.
- Every claim row must name a function that exists in `src/`. The checker
  matches a line-initial declaration, so a name surviving only in a doc comment
  does not satisfy a claim. The check is necessary rather than sufficient: it
  cannot tell whether the named function is the one the proofs are about, which
  is why the splice convention above — one body file, textually shared — is the
  actual guarantee.
