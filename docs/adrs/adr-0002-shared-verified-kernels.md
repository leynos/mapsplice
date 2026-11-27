# Architectural decision record (ADR) 002: Share verified kernels as whole functions

## Status

Accepted on 26 November 2026. Production kernels are written once as a
`macro_rules!` invocation that emits a whole function; the `exec` arm omits
proof annotations and the `verified` arm adds them and wraps the result in
`verus!`. The shipped executable and the Verus proof therefore expand one
domain-owned text.

## Date

2026-11-26.

## Context and problem statement

At `0e1ae939`, `docs/verification.md` recorded that the source-span scan was
verifiable over bytes, but the working restatement was not adopted because the
product called a different implementation. `original_position_source` and
`line_start` in `src/roadmap/source_preservation.rs` used `str::get(..offset)`
and a byte scan, while the only shared proved body was the two-line resolution
selector. That left the source-preservation behaviour outside the proof.

The obvious splice — share the loop body and pass the `invariant` as macro
tokens — does not compile. The ordinary Rust parser rejects `invariant` before
Verus can see it, and both spellings of annotation-parameterized sharing failed
on the pinned verifier: tokens passed as a macro parameter, and an annotation
supplied by a second macro around a shared body, each produced "expected `{`,
found `invariant`". Sharing only a text fragment cannot express a loop
invariant at all.

Mdtablefix solved the same problem by making the whole function the unit of
sharing: `verified_loop_function!` emits a complete `verus!` function with its
loop header, rather than inserting annotations into an already-parsed loop. The
ordinary build consumes the same executable tokens without the proof
annotations. This ADR records adopting that shape, the evidence that it works
on the current pins, and the boundaries that remain.

## Decision drivers

- The executable the verifier proves must be the executable the product ships,
  not a parallel model connected by prose.
- The proof must be able to fail: a mutation to the shared body, with the
  specification unchanged, must be rejected and attributed to the function
  under proof.
- Production must not depend on verification infrastructure. The domain owns
  the kernel text; the proof reaches into `src/` for it.
- The sharing mechanism must survive the ordinary compiler, Clippy, rustdoc and
  Whitaker, not only Verus.
- The decision must be reproducible from the pins committed in the repository.

## Requirements

### Functional requirements

- A backward scan must recognize CR and LF separately and return the start
  after the last relevant terminator, or zero when none precedes the offset.
- Span validation must reject out-of-range offsets, non-boundary offsets and
  reversed spans in a defined order, and extraction must return the exact
  source bytes of the widened span.
- Task-item extraction must use the next item's checked line start or fall back
  to the list end.
- The parser and render paths must call the same implementation the proof
  verifies.

### Technical requirements

- The ordinary build must compile the shared text as plain Rust.
- Verus must parse the shared text with loop invariants, `requires`, `ensures`
  and `decreases`.
- Ghost code must not change an executable branch, argument, conversion,
  binding, return value or effect.
- A pinned, reproducible toolchain must accept the arrangement before
  production changes land.

## Options considered

### Option A: Pass the invariant as macro tokens into a shared loop header

Rejected by the parser on both the old and current pins. The annotation tokens
arrive before loop parsing, and the result is not ordinary Rust.

### Option B: Wrap a shared body in a second annotation macro

Rejected for the same reason. The parser sees `invariant` inside an expansion
before the loop header is recognized.

### Option C: Use a ghost-erasure build of a Verus-native source tree

Not selected. It would require a second build pipeline and a toolchain-wide
mode switch for one kernel, and it was not measured to pass on the pinned
release. It remains a fallback if a future kernel defeats the whole-function
splice.

### Option D: Emit the whole function from one macro, selected by a mode token

Selected. The `exec` arm emits the signature, preamble, loop condition, body
and epilogue; the `verified` arm emits the same tokens plus the contract and
loop annotations and wraps the function in `verus!`. Both arms consume the same
executable slots in the same order.

## Decision outcome

`src/roadmap/verified_function.macro.rs` defines `verified_function!` with
`exec` and `verified` arms. `src/roadmap/source_span_kernel.macro.rs` and
`src/roadmap/task_item_span_kernel.macro.rs` write each kernel body once at an
invocation site, and `source_span_kernel.rs` expands the `exec` mode while
`verus/lib.rs` expands the `verified` mode. The mode is an explicit token, not a
`cfg`: both spellings remain visible in one file, and a reviewer can compare
the executable slots directly.

The ghost-erasure audit rule follows from the shape. Only the signature's
executable tokens, `before`, the loop condition, `loop_body`, `loop_step` and
`after` are emitted by both arms. `proof_first`, `proof_before`, `proof_loop`
and `proof_after` are emitted only by the verified arm. A change to an
executable slot changes both expansions; a change confined to a proof slot
changes neither executable expansion.

### Measured toolchain gate

The arrangement was measured before production changed, on x86-64 Linux with
`make`, `uvx`, `rustup`, `curl`, `unzip` and `sha256sum`. The feasibility
measurements below were taken on `nightly-2026-11-25`, the Cargo pin at the
time; the branch was later rebased onto `nightly-2026-11-26`, and the proof and
kernel were re-verified on that pin before publication.

```sh
make verus-install
make verus
make verus-selftest
```

`make verus-install` installed Verus `0.2026.09.06.8dea4a2` and Z3 4.16.0 under
`.verus/`. `make verus` reported **4 verified, 0 errors** on the pre-existing
selector proofs. `make verus-selftest` ran `verus/smoke.rs`, whose
`assert(false)` was rejected, printed `Verus proofs failed (exit 1).`, and
therefore proved the verifier started.

The probes then established, on the same pins:

- a macro-defined whole function with a backward byte scan, a loop invariant
  and `decreases` verifies;
- `ByteSpan` and the five-variant `SpanError` are usable in executable and
  specification positions;
- a private callee with `requires` discharged by a total caller verifies;
- `str::split_at` (twice) and `str::to_owned` carry vstd contracts sufficient
  to prove the copied bytes equal the source subrange;
- `Vec::with_capacity` and `Vec::push` are specified, but `String::from_utf8`
  has no vstd contract, so the `Vec<u8>` fallback would need an adapter
  obligation for the conversion;
- Clippy's `indexing_slicing` fires on the guarded backward index, and a single
  targeted `#[expect(clippy::indexing_slicing, reason = "...")]` satisfies the
  lint in both arms.

The historical formulations were re-run beside the old table. `str::get` is
still unsupported: `core::str::impl&%0::get` is not supported. A byte-slice
`RangeTo`/`Range` `get` now verifies — **2 verified, 0 errors** — where the old
pin reported `core::ops::range::RangeTo` as unsupported. The upgrade therefore
widened the supported surface, but `str::get` remains the reason the product
had to change to reach the proof.

The full kernel then verified **24 verified, 0 errors** on `verus/lib.rs`: the
selector's four obligations plus the scan, validation, copy and task-item
obligations, their supporting lemmas, and the span completeness theorem.

### Negative control

`scripts/check-source-span-mutation.sh` copies the tracked tree, mutates only
one executable slot in the copied kernel, and leaves the specification alone.
It records two controls because they exercise different obligations:

- removing the CR test from the loop condition is rejected by the scan's
  progress invariant, with `invariant not satisfied at end of loop body`
  attributed to `line_start_offset`;
- changing the returned index is rejected by the function postcondition, with
  `postcondition not satisfied` attributed to `line_start_offset`.

A loop-based scan cannot produce a postcondition failure from the first
mutation: any invariant strong enough to prove the postcondition also describes
the skipped region, so the defect is caught at the end of the loop body first.
The second mutation is therefore the postcondition control, and both failures
are attributed to the same function. The runtime mapping follows the same
split: the CR mutation is observed by the CR-only unit and append regressions,
and the return mutation by the extraction regression. Each selected regression
must pass on the unmutated copy first and fail with its own test name after its
mutation.

## Consequences

- The kernel is domain-owned. `source_span_kernel.rs` names no proof path; the
  proof includes `../src/` from `verus/lib.rs`.
- The product exports only total entry points. `line_start_offset` and
  `copy_span` carry caller-established preconditions and stay private;
  `checked_line_start`, `checked_line_span`, `extract_span`,
  `extract_line_span` and `task_item_spans` validate their inputs first.
- The `verified` arm declares `ByteSpan` and `SpanError` as `pub` because a
  `pub open spec fn` may not expose a `pub(crate)` datatype. The production arm
  keeps `pub(crate)`, and the module re-exports nothing further.
- The kernel text is opaque to Whitaker's `bumpy_road_function` because it is a
  macro expansion. `select_resolution.macro.rs` already records the same trade.
  The compensating controls are small kernels, normal Clippy and rustdoc gates,
  and review of the one shared body. A future kernel with genuinely nested
  conditionals would go unflagged by that lint.
- `include!` remains unusable inside a function body: spliced tokens report
  `span.from_expansion() == false` while carrying the included file's line
  numbers, which aborts Whitaker's lint. The whole-function macro is what gives
  the tokens the expansion context the lint skips.
- Allocation failure is outside the theorem. `str::to_owned` and `Vec` may
  abort through `handle_alloc_error`; the proof establishes the mathematical
  byte equality, not that the allocator cannot fail. The same boundary applies
  to any future fallback that converts a `Vec<u8>` with `String::from_utf8`,
  which has no vstd contract and would add an adapter obligation.
- Mdtablefix's trusted `external_body` matchers in
  <https://github.com/leynos/mdtablefix/issues/567> are debt to eliminate, not
  an architectural template. No kernel here uses `assume`, an admitted lemma or
  `external_body`.

## Repository and shared-tool ownership

The repository owns the kernel text, the production calls, the proof
obligations, the verification ledger, the mutation control and the Make
routing. The shared tool owns the Verus download, the archive checksum, the
installation layout, the Z3 installation and the verifier invocation. The Make
targets install no prover; the runner still ensures the Rust toolchain Verus
builds with through `rustup`.

## Pinned inputs

- Verus release: `0.2026.09.06.8dea4a2` (`tools/verus/VERSION`), archive digest
  `13d01e134c0620c3b29770874707d16c33b3d227c843a489c8ceb744d43c0a16`
  (`tools/verus/SHA256SUMS`).
- Verus runner: `592558f7183b554fc749696246984c5d7300d029`
  (`tools/rust-prover-tools/REF`).
- Cargo toolchain: `nightly-2026-11-25` during the feasibility experiment;
  `nightly-2026-11-26` at the assessed head after rebasing onto `main`
  (`rust-toolchain.toml`). The Verus binary builds with its own bundled
  toolchain, reported as `1.98.0-x86_64-unknown-linux-gnu`.
- CI installer action: `a0519a1476ab769ca7b8badfa794f3751213f1ea`
  (`install-prover-tools` in `.github/workflows/verus.yml`).

## Known risks and limitations

- A whole-function macro hides its body from macro-aware lints. Small kernels
  and review are the compensating controls, not a claim that the lint ran.
- The specification is written against `Seq<u8>` and vstd's `str` contracts.
  Those contracts are trusted `assume_specification` items; the ledger names
  them rather than implying they are proved here.
- The mutation control depends on exact one-occurrence substitutions. A
  refactor that changes the kernel line must update the script; the script
  exits 2 when the text no longer matches, and the harness has a zero-match
  case so a silent no-op cannot pass as a rejection.
- The macro contract is held by `tests/ui/verified_function_exec.rs`, which
  includes the real macro file and invokes the `exec` arm with every proof slot
  present, and `tests/ui/verified_function_malformed.rs`, which omits a slot
  and must fail. The fixtures compile the shipped macro text without exporting
  it as public API.
