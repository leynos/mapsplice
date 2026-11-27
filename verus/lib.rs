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
include!("../src/roadmap/source_span_types.macro.rs");
include!("../src/roadmap/source_span_kernel.macro.rs");
include!("../src/roadmap/task_item_span_kernel.macro.rs");

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
// The source-span obligation is now present
// ---------------------------------------------------------------------------
//
// An earlier revision recorded that no source-span theorem existed. Production
// called `original_position_source`, which used `str::get(..offset)`, and the
// formulations tried against the then-pinned verifier did not reach it. The
// whole-function splice in `verified_function.macro.rs` removed that blocker:
// the executable scan, validation and copy are expanded by both the product
// and this proof, so they are inside the proof boundary.
//
// The source-span specifications, lemmas and verified expansion live in
// `source_span_proofs.rs`, included below. `docs/verification.md` retains the
// historical formulation table, records the current-pin outcomes, and names
// the vstd contracts the proof trusts.
//
// The earlier `resolution_is_a_function_of_its_inputs` note stays relevant to
// the selector: it was removed because it could not fail. It was congruence
// over a pure spec fn and is discharged from the signature alone, so no kernel
// defect could falsify it. The property it meant to capture is real —
// resolution must not vary with hidden state such as map iteration order — but
// that is a property of `RenumberPlan::resolve_reference`, which reads the
// maps; the kernel receives two already-resolved anchors. The source-span
// obligations are a different shape: they constrain executable text and are
// exercised by `make verus-mutation`.

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

// The source-span specifications, lemmas and verified expansion live beside
// this entry point to keep each proof file below the 400-line limit.
include!("source_span_proofs.rs");

source_span_kernel!(verified);
task_item_span_kernel!(verified);

fn main() {}
