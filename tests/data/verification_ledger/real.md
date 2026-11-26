# Verification ledger

`mapsplice` uses Verus to prove narrow kernels that the renumbering,
dependency-rewrite and source-preservation paths call in production. This
ledger records every claimed proof result, its executable function, input
domain, external contracts, and result class. A kernel may not be represented
by a separate `proofs/` implementation unless a refinement proof connects it to
the production function.

## Claim ledger

| Claim                                                                                           | Executable function  | Input domain                                                                                       | Unverified external contracts            | Result class      |
| ----------------------------------------------------------------------------------------------- | -------------------- | -------------------------------------------------------------------------------------------------- | ---------------------------------------- | ----------------- |
| Identity preservation: a target-text reference never resolves through the cross-source fallback | `select_resolution`  | `Option<T>` and `Option<T>` for any `T: Copy`, plus a `bool` source flag; no allocation, no lookup | None: the function has no external calls | Local correctness |
| Local-mapping precedence: a target-text reference with its own mapping keeps it                 | `select_resolution`  | As above                                                                                           | None                                     | Local correctness |
| Non-vacuity: fragment text does consult the cross-source fallback                               | `select_resolution`  | As above, with the local option absent and the cross-source option present                         | None                                     | Local correctness |
| Line start: the scan returns exactly the offset after the last CR or LF before the offset       | `line_start_offset`  | Any `&str` and a byte offset at or below its byte length                                           | `str::as_bytes`, `str::len`              | Local correctness |
| Line-start rejection: an out-of-range or non-boundary offset yields `None`                      | `checked_line_start` | Any `&str` and any `usize`                                                                         | `str::is_char_boundary`                  | Local correctness |
| Span validation: the first failing check is the one reported, in the documented order           | `checked_line_span`  | Any `&str` and any pair of `usize` offsets                                                         | `str::is_char_boundary`                  | Local correctness |
| Span bounds and boundaries: a successful span is ordered, in range and boundary-aligned         | `checked_line_span`  | As above                                                                                           | `str::is_char_boundary`                  | Local correctness |
| Extraction: the copied text is exactly the source subrange of the line-widened span             | `extract_line_span`  | Any `&str` and a validated span                                                                    | `str::split_at`, `str::to_owned`         | Local correctness |
| Task-item spans: each item resolves to the next checked line start or the list end              | `task_item_spans`    | Any slice of optional item starts, an optional list end and a source                               | As above                                 | Local correctness |

_Table 1: The verification claim ledger._

Each row is discharged by a theorem in `verus/lib.rs`, which runs under
`make verus`. The titles above name the obligation; the theorem's comment in
that file states its premise. `make verus-selftest` runs `verus/smoke.rs`,
whose assertion is deliberately unprovable, and fails unless Verus rejects it —
so a run in which the verifier never started cannot pass as a successful one.
`scripts/check-verification-ledger.sh`, wired into `make lint`, fails when a
claim names a function that no longer exists in `src/`.

The proofs run under Verus `0.2026.09.06.8dea4a2`, the release pinned in
`tools/verus/VERSION`, and `make verus` reports twenty-one verified items with
no errors: the selector's existing obligations plus the source-span kernel's
functions, specifications and supporting lemmas. The selector's four verified
under the previous release, `0.2025.04.19.1b16620`, so that bump changed no
result; the source-span rows are new.

Deleted-target rejection is a further obligation that has no row here, because
it has no theorem that can fail; the policy at the end of this document records
why and where it is covered instead.

## What is proved, and what is not

The selector kernel `select_resolution` decides **which of two already-computed
answers a dependency reference resolves through**. That is the whole of its
responsibility, and it is where issue #85's identity-preservation rule lives:
before the fix, a consumer could be silently re-pointed at an unrelated item
that had inherited its prerequisite's number.

The source-span kernel decides **where a preserved span starts and what bytes
it contains**. `original_position_source`, `task_item_sources` and the former
`line_start` adapter in `src/roadmap/source_preservation.rs` now call only its
total entry points, so the executable the product runs is the one under proof.

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

**The mdast adapter is not verified.** The kernel takes byte offsets; the
Markdown parser supplies [`Position`] values. Mapping a parser's byte column to
a scalar column happens in `src/roadmap/source_location.rs`, and the parser's
own offset consistency is an external contract. Neither is proved here.

Extracting the recognizer into a provable kernel is not attempted here, and
would be a substantial piece of work: it consumes mdast nodes and byte strings
through several helpers whose contracts would each need stating. Recording the
gap is preferable to implying a coverage that does not exist.

## How the kernels connect to production

`verus/lib.rs` proves the macro-defined bodies in
`src/roadmap/ops/select_resolution.macro.rs` and the shared source-span macro
files beside it. `src/roadmap/ops/remap_kernel.rs` and
`src/roadmap/source_span_kernel.rs` include the same files and expand the
`exec` mode, and `verus/lib.rs` expands the `verified` mode. The verified text
and the compiled text are therefore one artefact per kernel, not two
implementations that can drift.

The shared bodies live beside the domain modules that expand them, and the
proofs are what cross the tree to reach them. The dependency therefore runs
from verification infrastructure to the domain kernel: production carries no
proof path, no `CARGO_MANIFEST_DIR` lookup, and no knowledge of `verus/`. A
kernel placed under the proof tree would invert that direction — the domain
would name verification infrastructure to compile itself — which is why the
files sit in `src/roadmap/` and `verus/lib.rs` writes `../src/`.

Sharing the text is deliberate rather than incidental. Verus treats a
plain-Rust module included with `#[path]` as opaque — it cannot be called from
a proof at all — so a proof that reaches production code must bring the body in
as text. `verus/lib.rs` documents the convention.

That text is a `macro_rules!` definition rather than a bare body fragment, and
the indirection is load-bearing twice over. Whitaker's `bumpy_road_function`
lint cannot see through `include!`: spliced tokens report
`span.from_expansion() == false` while still carrying the _included_ file's
line numbers, so the lint compares them against the enclosing function's range
in a different coordinate system and aborts the compiler with an internal
error. That abort is not a warning to be silenced — it fails `make lint`
outright, for the whole crate, on any `include!` inside a function body. And a
loop that needs an `invariant` cannot share only its body: the ordinary parser
rejects the annotation before Verus ever sees it. Emitting the whole function
from one macro gives the tokens the expansion context the lint skips, and lets
the `verified` arm add the contract and loop annotations to the same executable
slots the `exec` arm emits. `docs/adrs/adr-0002-shared-verified-kernels.md`
records the decision and its measurements.

The trade is recorded in `src/roadmap/ops/select_resolution.macro.rs` and
`src/roadmap/verified_function.macro.rs`: the kernel bodies are opaque to
`bumpy_road_function` in both crates. Nothing is lost for the selector — one
`if` inside one match arm — and the source-span kernel keeps each function
small, but a future kernel with genuinely nested conditionals would go
unflagged by that lint.

`RenumberPlan::resolve_reference` delegates the whole decision, including the
fragment check. It deliberately does **not** pre-filter the cross-source value
for target text: doing so would enforce the rule in the caller instead and
leave the kernel's fragment guard unreachable in production, making the proofs
statements about dead code. Likewise, `original_position_source` and
`task_item_sources` delegate validation and copying rather than pre-widening
offsets themselves, so the kernel's rejection branches are reachable from the
product.

## Policy

- No `assume`, admitted lemma, or `external_body` may assert the property under
  proof. Neither kernel needs one: the selector calls nothing, and the
  source-span kernel reaches only vstd-specified `str` operations.
- Byte offsets and Unicode scalar indices are distinct concerns. `ByteSpan`
  measures UTF-8 byte offsets; markdown 1.0.0 reports byte columns, while
  `src/roadmap/source_location.rs` derives scalar columns for its own
  reporting. The kernel never treats `str::len()` as a display or scalar width,
  and no claim here conflates the two.
- Filesystem and I/O assumptions are outside every proof boundary. Both kernels
  are pure, so the proofs say nothing about reading a roadmap, writing output,
  or the in-place write path. Those are covered by the golden and CLI tests.
- Existing property tests remain in place. Verus proofs complement them and do
  not replace them.
- A proof's obligations are only meaningful if they can fail. Each remaining
  selector theorem was checked by injecting a defect and confirming the proof
  is rejected, and the source-span kernel is checked by a scripted control. A
  theorem that survives every injected defect is removed rather than listed,
  which is why the ledger has nine rows and not more.

  Two kinds of defect were injected into the selector, and the distinction
  matters. A **specification defect** changes `select_resolution_spec`, the
  definition the `ensures` clause compares against; it tests whether an
  obligation can distinguish one decision rule from another. A **body defect**
  changes only the shared macro body in
  `src/roadmap/ops/select_resolution.macro.rs`, leaving the specification
  untouched; it tests whether verification reaches the text the product
  actually compiles, which is the whole point of the splice convention above.

  | Injected specification defect                    | Theorem that rejects it                            |
  | ------------------------------------------------ | -------------------------------------------------- |
  | Target text falls back to the cross-source value | `target_text_never_uses_the_cross_source_fallback` |
  | Fragment guard inverted                          | `target_text_never_uses_the_cross_source_fallback` |
  | Local mapping loses precedence                   | `target_text_keeps_its_source_local_mapping`       |
  | Local branch returns the cross-source value      | `target_text_keeps_its_source_local_mapping`       |
  | Fragment text ignores the cross-source fallback  | `fragment_text_uses_the_cross_source_fallback`     |
  | Both branches resolve to nothing                 | `fragment_text_uses_the_cross_source_fallback`     |

  _Table 2: Injected specification defects and the obligations that reject
  them._

  One selector body defect was injected: the macro dropping its fragment guard
  while the specification stayed correct. Verus rejected it at the
  `select_resolution` `ensures` clause — 4 verified, 1 error — which is the
  evidence that the proof is about the shared body and not merely about the
  specification. A body defect this file could not catch would mean the splice
  had silently stopped reaching production.

- The source-span kernel carries a scripted negative control because its body
  is larger than a two-line selector. `make verus-mutation` copies the tracked
  tree, mutates one executable slot in the copied kernel and leaves the
  specification intact. Two mutations are used:

  | Mutation                                 | Rejecting obligation                          |
  | ---------------------------------------- | --------------------------------------------- |
  | CR test removed from the loop condition  | `invariant not satisfied at end of loop body` |
  | Returned index moved past the line start | `postcondition not satisfied`                 |

  _Table 3: Injected source-span body defects and their rejecting obligations._

  Both are attributed to `line_start_offset` by source line. A loop-based scan
  cannot produce a postcondition failure from the first mutation: any invariant
  strong enough to prove the postcondition also describes the skipped region,
  so the defect is caught at the end of the loop body first. The second
  mutation is therefore the postcondition control. The control also runs the
  CR-only unit and append regressions against the mutated copy and requires
  both to fail, so the shipped product is shown to be sensitive to the same
  defect the proof rejects.

- The obligations are not all the same shape, and the ledger should not imply
  they are. Three selector theorems state properties of
  `select_resolution_spec` and are discharged over the specification alone; the
  executable function is tied to that specification by its own `ensures`
  clause, which is what the body defect above exercises. The source-span rows
  instead tie each executable function to a specification through its own
  postcondition, so the mutation control is what demonstrates the tie rather
  than a separate specification-only theorem.

- No theorem here inspects allocation failure, so the ledger does not claim
  it. `str::to_owned` and `Vec` may abort through `handle_alloc_error`; the
  proofs establish the mathematical byte equality of the copied text, not that
  the allocator cannot fail. A fallback route that converted a `Vec<u8>` with
  `String::from_utf8` would add a second unverified contract, because vstd
  provides no specification for `String::from_utf8`. The selected route avoids
  that conversion.

- The applicable vstd contracts for the source-span rows are
  `str::as_bytes`, `str::len`, `str::is_char_boundary`, `str::split_at` and
  `str::to_owned`, each an `assume_specification` item in vstd. They are
  trusted library contracts, not results proved here; the ledger names them so
  the trust boundary is visible.

- Two selector rows have been removed rather than listed, both for the same
  reason.

  The earlier "Source-span preservation: resolution depends on nothing but its
  three inputs" concluded equality of results from pairwise-equal arguments,
  which Verus discharges from the signature alone for any total function, so no
  kernel defect could falsify it. It was a property of the caller rather than
  of the selector, and it is superseded by the source-span kernel rows above,
  which do own source-preservation behaviour and can fail.

  "Deleted-target rejection" was removed when the defect battery showed its
  theorem, which assumed both options absent, survived all six specification
  defects while its neighbours each rejected at least one. With neither option
  holding a value there is nothing to fabricate, so the conclusion is forced by
  the type rather than by the kernel. The rejection itself is still guaranteed:
  `target_text_never_uses_the_cross_source_fallback` covers its kernel form,
  and the `unresolved` collection in `src/roadmap/ops/rewrite.rs` turns a
  `None` into `MapspliceError::DanglingDependency`, which the CLI regressions
  and property suite assert directly.

  Retaining either row would have made the ledger's own falsifiability rule
  false, which is the failure this entry exists to prevent.

- The formulation table is retained as history and extended with the results
  measured on the current pins. It records what was tried, so a later reader
  can extend the list rather than repeat it.

  | Tested formulation                                         | Outcome on `0.2025.04.19.1b16620`            | Outcome on `0.2026.09.06.8dea4a2`          |
  | ---------------------------------------------------------- | -------------------------------------------- | ------------------------------------------ |
  | `str::get(..offset)`, as production wrote it               | `core::str::impl&%0::get` is not supported   | `core::str::impl&%0::get` is not supported |
  | `bytes.get(..offset)` on `&[u8]`                           | `core::ops::range::RangeTo` is not supported | 2 verified, 0 errors                       |
  | Line scan over `Seq<u8>`, `while` with invariant           | 3 verified, 0 errors                         | verifies                                   |
  | The same loop with the invariant removed                   | postcondition not satisfied                  | postcondition not satisfied                |
  | Iterator-only restatement, needing no invariant            | closure pattern unsupported                  | not re-measured                            |
  | Annotation tokens passed into a shared `while` header      | expected `{`, found `invariant`              | expected `{`, found `invariant`            |
  | Annotation supplied by a second macro around a shared body | expected `{`, found `invariant`              | expected `{`, found `invariant`            |
  | Whole-function macro with mode tokens, loop invariant      | not tried                                    | 21 verified, 0 errors                      |

  _Table 4: Span-kernel formulations tested, by pinned release._

  The last two rows of the old pin are worth stating precisely, because they
  ruled out the most promising route at the time. Sharing executable text with
  `src/` requires the shared text to parse as ordinary Rust, since `src/`
  compiles it, while the scan needs an `invariant` clause to verify. Passing
  the annotation as macro tokens fails because the parser rejects `invariant`
  inside a macro expansion before expansion is attempted. The reconciliation is
  to share the whole function rather than a body fragment: the annotations then
  sit in the verified arm's expansion, which Verus parses as its own syntax,
  while the exec arm emits only ordinary Rust. That is the adopted
  architecture, and it is a finding about the tested arrangements rather than a
  proof of impossibility for any other.

  What is preserved under the adopted architecture is the property issue #85
  asks for. The span copy is covered by the golden fixtures, the in-place
  byte-identity assertions, and the kernel's own unit and property tests,
  including the case #85 makes load-bearing: a rejected in-place deletion
  leaves the target byte-identical. Those tests remain; the proof is added
  beside them rather than counted in place of them.

- Every claim row must name a function that exists in `src/`. The checker
  matches a line-initial declaration, which is why each kernel declaration is
  written on its own line inside the macro invocation. The check is textual, so
  it cannot tell whether the named function is the one the proofs are about;
  the splice convention above — one body file, textually shared — is the actual
  guarantee.

[`Position`]: https://docs.rs/markdown/1.0.0/markdown/unist/struct.Position.html
