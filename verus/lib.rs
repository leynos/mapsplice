//! Root entry point for production-used Verus kernels.
//!
//! Each verified kernel expands a shared `macro_rules!` body that is defined in
//! a `.macro.rs` file and included by both this proof and the production
//! module, so the text Verus proves and the text cargo compiles are one
//! artefact rather than two implementations that can drift.
//!
//! The shared body lives in the domain module that uses it, and the proof
//! reaches into `src/` for it. The dependency therefore runs from verification
//! infrastructure to the domain kernel: production carries no proof path, no
//! manifest environment lookup, and no knowledge of this directory. Reaching
//! across the tree is what keeps the domain clean.
//!
//! The convention matters twice over. A plain-Rust module pulled in with
//! `#[path]` is treated by Verus as opaque — it cannot be called from a proof at
//! all — so a proof that reaches production code must bring the body in as
//! text. And the splice cannot be a bare `include!` inside the function, because
//! Whitaker's `bumpy_road_function` lint compares an included file's line
//! numbers against the enclosing function's range, aborts the compiler, and so
//! fails `make lint` for any crate that does it. A macro definition carries the
//! expansion context that lint skips, which is why the shared text is a macro
//! rather than a body fragment.
//!
//! Do not add a standalone reimplementation here. A kernel that is not the
//! function the product calls proves nothing about the product.

use vstd::{
    prelude::*,
    seq::{lemma_seq_subrange_composition, lemma_seq_subrange_index},
    slice::spec_slice_index,
    string::StringSliceAdditionalSpecFns,
    utf8::{
        encode_utf8,
        is_char_boundary,
        is_char_boundary_iff_is_leading_byte,
        is_char_boundary_start_end_of_seq,
        valid_utf8,
        valid_utf8_split,
    },
};

// The shared kernel text is domain-owned; the proof reaches into `src/` for
// it. The macro definitions sit at the top level, outside `verus!`, and their
// `verified` arms emit `verus!` blocks of their own.
include!("../src/roadmap/verified_function.macro.rs");
include!("../src/roadmap/source_span_kernel.macro.rs");

verus! {

// Relative to this file. The body is domain-owned, so the proof is what crosses
// the tree boundary; see the module documentation above.
include!("../src/roadmap/ops/select_resolution.macro.rs");

// ---------------------------------------------------------------------------
// Kernel: dependency-reference resolution
// ---------------------------------------------------------------------------
//
// Production caller: `RenumberPlan::resolve_reference` in
// `src/roadmap/model.rs`, which delegates its decision to
// `select_resolution` in `src/roadmap/ops/remap_kernel.rs`.
//
// The kernel answers one question: given the source-local mapping and the
// unique cross-source mapping, which of them does a dependency reference
// resolve through? Its inputs are already-resolved anchors, so it performs no
// lookup; the proof obligations therefore concern which answer is chosen, not
// whether a lookup succeeds.

/// Specification of the resolution decision.
///
/// Read as: the local mapping wins when it exists; otherwise the cross-source
/// value is used only for fragment text; otherwise there is no resolution.
pub open spec fn select_resolution_spec<T>(
    local: Option<T>,
    cross_unique: Option<T>,
    source_is_fragment: bool,
) -> Option<T> {
    if local.is_some() {
        local
    } else if source_is_fragment {
        cross_unique
    } else {
        Option::None
    }
}

/// Choose which mapping a dependency reference resolves through.
///
/// The body is expanded from the same macro the production kernel expands, so
/// this proof is about the executable function the product calls and not about
/// a model of it.
pub fn select_resolution<T: Copy>(
    local: Option<T>,
    cross_unique: Option<T>,
    source_is_fragment: bool,
) -> (result: Option<T>)
    ensures
        result == select_resolution_spec(local, cross_unique, source_is_fragment),
{
    select_resolution_body!(local, cross_unique, source_is_fragment)
}

// ---------------------------------------------------------------------------
// Obligation 1: identity preservation
// ---------------------------------------------------------------------------
//
// A reference written in the *target* must never resolve through the
// cross-source fallback. This is the rule whose violation let an inserted task
// that inherited a prerequisite's number absorb a surviving consumer's clause.

/// A target-text reference with no source-local mapping resolves to nothing.
///
/// This is the identity-preservation obligation in its kernel form. The
/// cross-source value is present and deliberately ignored: if the decision
/// consulted it, this proof would fail.
proof fn target_text_never_uses_the_cross_source_fallback<T>(
    local: Option<T>,
    cross_unique: Option<T>,
)
    requires
        !local.is_some(),
    ensures
        select_resolution_spec(local, cross_unique, false) == Option::<T>::None,
{
}

/// A target-text reference with a source-local mapping resolves to that mapping.
///
/// The companion obligation: withholding the fallback must not discard the
/// legitimate local resolution.
proof fn target_text_keeps_its_source_local_mapping<T>(
    local: Option<T>,
    cross_unique: Option<T>,
)
    requires
        local.is_some(),
    ensures
        select_resolution_spec(local, cross_unique, false) == local,
{
}

// ---------------------------------------------------------------------------
// Obligation 2: deleted-target rejection
// ---------------------------------------------------------------------------
//
// Deleting a prerequisite retires its identity. A surviving consumer's clause
// names the retired anchor, so it must fail to resolve rather than silently
// resolving to the replacement item that inherited the old number.
//
// This obligation has no theorem of its own, and the omission is deliberate.
// The earlier `retired_anchor_never_resolves` assumed both options absent and
// concluded the result was `None`. With neither option holding a `T`, there is
// no value to fabricate: `None` is forced by the type rather than by the
// kernel, so no defect in the body can make the conclusion false. A six-defect
// battery over `select_resolution_spec` confirmed it — the theorem survived
// every one, while each of its three neighbours was rejected by at least one.
// The defects and the obligations that reject them are tabulated in
// `docs/verification.md`, whose Table 2 is the evidence for that claim.
//
// Deleted-target rejection is still guaranteed; it is just not this file's
// guarantee. In kernel form it is checked by `target_text_never_uses_the_cross_source_fallback`
// (a target reference with no local mapping resolves to nothing, whatever the
// cross-source value holds), and in production it is the `unresolved` collection
// in `src/roadmap/ops/rewrite.rs` that turns a `None` into
// `MapspliceError::DanglingDependency`. That end-to-end behaviour is pinned by
// the CLI regressions and the property suite, which assert the rejection
// directly.

// ---------------------------------------------------------------------------
// A source-span obligation is deliberately absent
// ---------------------------------------------------------------------------
//
// An earlier draft carried `resolution_is_a_function_of_its_inputs`, which took
// two triples with pairwise-equal inputs and concluded equality of results. It
// was removed because it could not fail. It is congruence over a pure `spec fn`
// and is therefore discharged from the signature alone: no defect in the kernel
// can make it false. The defect battery recorded in `docs/verification.md`
// demonstrates that rather than assuming it, and the vacuity of both removed
// theorems is why this file proves three obligations and not five.
//
// The property it was meant to capture is real — resolution must not vary with
// hidden state such as map iteration order — but it is a property of the
// *caller*. `RenumberPlan::resolve_reference` is what reads the maps; the
// kernel receives two already-resolved anchors and has no state to vary with.
// Would the kernel's reference transparency be worth proving, the missing piece
// is a functional-correctness theorem connecting its output to the resolved
// anchors, not a determinism statement about a total function of its arguments.
//
// Source-span preservation itself is a property of the renderer, which writes
// output by copying preserved spans. It is not a property of this kernel, and
// no theorem here would be evidence for it. It is covered by the golden
// fixtures and the in-place byte-identity assertions in `tests/` instead, and
// `docs/verification.md` records the boundary.
//
// The boundary is recorded as a set of tested formulations rather than as an
// impossibility, and the distinction is deliberate. `original_position_source`
// in `src/roadmap/source_preservation.rs` is the production function that would
// carry the obligation, and it calls `str::get(..offset)`, rejected by Verus as
// unsupported, over a `RangeTo`, rejected against the pinned release in turn. A
// byte-slice restatement does verify, but it would be a second implementation
// rather than the function production compiles, which #85 rules out. A
// restatement that production actually called would change that assessment.
//
// The splice convention above was probed as the route past this and does not
// reach it. Text shared with `src/` must parse as ordinary Rust, while the scan
// needs an `invariant` clause to verify, so the two blocks are not independently
// composable. Both spellings of annotation-parameterized sharing were tried —
// tokens passed as a macro parameter, and the annotation supplied by a second
// macro wrapping a shared body — and both fail identically, because the parser
// rejects `invariant` inside a macro expansion before expansion happens.
// `docs/verification.md` carries the table of formulations and outcomes, so the
// next reader can extend the list rather than re-derive it.

/// Fragment text does consult the cross-source fallback.
///
/// Recorded because the fallback is not dead code: without this obligation a
/// change that removed the fragment branch entirely would satisfy every other
/// theorem in this file.
proof fn fragment_text_uses_the_cross_source_fallback<T>(
    local: Option<T>,
    cross_unique: Option<T>,
)
    requires
        !local.is_some(),
        cross_unique.is_some(),
    ensures
        select_resolution_spec(local, cross_unique, true) == cross_unique,
{
}

} // verus!

// ---------------------------------------------------------------------------
// Kernel: source-span scanning and copying
// ---------------------------------------------------------------------------
//
// Production callers: `original_position_source`, `task_item_sources` and the
// line-start adapter in `src/roadmap/source_preservation.rs`, through the total
// entry points in `src/roadmap/source_span_kernel.rs`.
//
// The kernel answers three questions in byte offsets: where the line
// containing an offset starts, whether a requested span names a valid
// substring, and how to copy that substring. CR and LF are recognized
// separately, so a CR-only boundary is preserved as faithfully as an LF or
// CRLF one.

verus! {

broadcast use {vstd::slice::group_slice_axioms, vstd::utf8::group_utf8_lib};

/// The two bytes Markdown treats as line terminators in this scan.
pub open spec fn is_line_terminator(byte: u8) -> bool {
    byte == b'\n' || byte == b'\r'
}

/// The start of the line containing `offset`, as a byte offset.
///
/// This is the recursive specification the executable backward scan is proved
/// equal to. It walks back from `offset` to the first preceding CR or LF and
/// returns the position after it, or zero when no terminator precedes it.
pub open spec fn spec_line_start(bytes: Seq<u8>, offset: int) -> int
    decreases offset,
{
    if offset <= 0 {
        0
    } else if is_line_terminator(bytes[offset - 1]) {
        offset
    } else {
        spec_line_start(bytes, offset - 1)
    }
}

/// True when `offset` can name a byte position in `bytes`.
pub open spec fn spec_offset_within(bytes: Seq<u8>, offset: int) -> bool {
    0 <= offset <= bytes.len() && is_char_boundary(bytes, offset)
}

/// The first failing span check, or `None` when the span is acceptable.
///
/// The order matches the executable validation: start range, start boundary,
/// end range, end boundary, ordering, and finally the widened line start's
/// boundary. The last check is redundant for any `&str`, whose bytes are valid
/// UTF-8 and whose line starts after a CR or LF are therefore boundaries, but
/// stating it keeps the successful span's boundary facts independent of a
/// deep UTF-8 lemma.
pub open spec fn spec_span_error(bytes: Seq<u8>, start: int, end: int) -> Option<SpanError> {
    if start > bytes.len() {
        Some(SpanError::StartOutOfBounds)
    } else if !is_char_boundary(bytes, start) {
        Some(SpanError::StartNotCharBoundary)
    } else if end > bytes.len() {
        Some(SpanError::EndOutOfBounds)
    } else if !is_char_boundary(bytes, end) {
        Some(SpanError::EndNotCharBoundary)
    } else if start > end {
        Some(SpanError::Reversed)
    } else if !is_char_boundary(bytes, spec_line_start(bytes, start)) {
        Some(SpanError::StartNotCharBoundary)
    } else {
        None
    }
}

/// Compare an actually produced optional span with its specification.
pub open spec fn spec_span_matches(actual: Option<ByteSpan>, expected: Option<(int, int)>) -> bool {
    match actual {
        Some(span) => expected == Some((span.start as int, span.end as int)),
        None => expected.is_none(),
    }
}

/// The successful result of `checked_line_start`.
pub open spec fn spec_checked_line_start(bytes: Seq<u8>, offset: int) -> Option<int> {
    if spec_offset_within(bytes, offset) {
        Some(spec_line_start(bytes, offset))
    } else {
        None
    }
}

/// The successful result of `checked_line_span`.
pub open spec fn spec_checked_line_span(
    bytes: Seq<u8>,
    start: int,
    end: int,
) -> Option<(int, int)> {
    if spec_span_error(bytes, start, end).is_none() {
        Some((spec_line_start(bytes, start), end))
    } else {
        None
    }
}

/// The end offset a task item's extraction uses.
pub open spec fn spec_task_item_end(
    starts: Seq<Option<usize>>,
    list_end: Option<usize>,
    bytes: Seq<u8>,
    index: int,
) -> Option<int> {
    let from_next = if index + 1 < starts.len() {
        match starts[index + 1] {
            Some(next) => spec_checked_line_start(bytes, next as int),
            None => None,
        }
    } else {
        None
    };
    match from_next {
        Some(value) => Some(value),
        None => match list_end {
            Some(value) => Some(value as int),
            None => None,
        },
    }
}

/// The span each task item extracts, or `None` when one cannot be formed.
pub open spec fn spec_task_item_span(
    starts: Seq<Option<usize>>,
    list_end: Option<usize>,
    bytes: Seq<u8>,
    index: int,
) -> Option<(int, int)> {
    if 0 <= index < starts.len() {
        match starts[index] {
            None => None,
            Some(start) => match spec_task_item_end(starts, list_end, bytes, index) {
                None => None,
                Some(end) => spec_checked_line_span(bytes, start as int, end),
            },
        }
    } else {
        None
    }
}

/// True when every produced span matches its specification.
pub open spec fn spec_all_spans_match(
    spans: Seq<Option<ByteSpan>>,
    starts: Seq<Option<usize>>,
    list_end: Option<usize>,
    bytes: Seq<u8>,
) -> bool {
    forall|position: int| 0 <= position < spans.len() ==> spec_span_matches(
        spans[position],
        spec_task_item_span(starts, list_end, bytes, position),
    )
}

/// Carry the per-item property across an append.
proof fn lemma_spans_match_push(
    spans: Seq<Option<ByteSpan>>,
    starts: Seq<Option<usize>>,
    list_end: Option<usize>,
    bytes: Seq<u8>,
    appended: Option<ByteSpan>,
)
    requires
        spec_all_spans_match(spans, starts, list_end, bytes),
        spec_span_matches(
            appended,
            spec_task_item_span(starts, list_end, bytes, spans.len() as int),
        ),
    ensures
        spec_all_spans_match(spans.push(appended), starts, list_end, bytes),
{
    assert forall|position: int|
        0 <= position < spans.push(appended).len() implies spec_span_matches(
        #[trigger] spans.push(appended)[position],
        spec_task_item_span(starts, list_end, bytes, position),
    ) by {
        if position < spans.len() {
            assert(spans.push(appended)[position] == spans[position]);
        }
    }
}

/// Relate the scan's stopping point to the recursive specification.
proof fn lemma_spec_line_start(bytes: Seq<u8>, offset: int, index: int)
    requires
        0 <= index <= offset <= bytes.len(),
        forall|i: int| index <= i < offset ==> !is_line_terminator(bytes[i]),
        index == 0 || is_line_terminator(bytes[index - 1]),
    ensures
        spec_line_start(bytes, offset) == index,
    decreases offset - index,
{
    if offset == index {
        if index > 0 {
            assert(is_line_terminator(bytes[offset - 1]));
        }
    } else {
        assert(!is_line_terminator(bytes[offset - 1]));
        lemma_spec_line_start(bytes, offset - 1, index);
    }
}

/// The line start is a byte position at or before the supplied offset.
proof fn lemma_line_start_bounds(bytes: Seq<u8>, offset: int)
    requires
        0 <= offset <= bytes.len(),
    ensures
        0 <= spec_line_start(bytes, offset) <= offset,
    decreases offset,
{
    if offset > 0 && !is_line_terminator(bytes[offset - 1]) {
        lemma_line_start_bounds(bytes, offset - 1);
    }
}

/// A character boundary inside a prefix is a boundary of that prefix.
///
/// This is what lets the copy split at the span start on the prefix returned
/// by the first split, using the boundary the validator established for the
/// whole source. The two boundaries agree because, away from the shared end,
/// both are characterized by the same leading byte.
proof fn lemma_boundary_prefix(bytes: Seq<u8>, prefix_len: int, index: int)
    requires
        valid_utf8(bytes),
        is_char_boundary(bytes, prefix_len),
        0 <= index <= prefix_len <= bytes.len(),
    ensures
        is_char_boundary(bytes.subrange(0, prefix_len), index) == is_char_boundary(bytes, index),
{
    let prefix = bytes.subrange(0, prefix_len);
    valid_utf8_split(bytes, prefix_len);
    if index == 0 {
        assert(is_char_boundary(prefix, 0));
        assert(is_char_boundary(bytes, 0));
    } else if index == prefix_len {
        is_char_boundary_start_end_of_seq(prefix);
    } else {
        lemma_seq_subrange_index(bytes, 0, prefix_len, index);
        is_char_boundary_iff_is_leading_byte(bytes, index);
        is_char_boundary_iff_is_leading_byte(prefix, index);
    }
}

} // verus!

source_span_kernel!(verified);

fn main() {}
