//! Root entry point for production-used Verus kernels.
//!
//! Each verified kernel expands a shared `macro_rules!` body that is defined in
//! a `.macro.rs` file and included by both this proof and the production
//! module, so the text Verus proves and the text cargo compiles are one
//! artefact rather than two implementations that can drift.
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

use vstd::prelude::*;

verus! {

include!("kernels/select_resolution.macro.rs");

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

/// A reference to a retired anchor does not resolve, in either source.
///
/// "Retired" is modelled as: the anchor has no mapping in the local table, and
/// it is not uniquely mapped cross-source either. Under those conditions the
/// reference must not resolve, whatever its source, which is what makes the
/// operation fail with a dangling-dependency error instead of re-pointing the
/// consumer at a different item.
proof fn retired_anchor_never_resolves<T>(
    local: Option<T>,
    cross_unique: Option<T>,
    source_is_fragment: bool,
)
    requires
        !local.is_some(),
        !cross_unique.is_some(),
    ensures
        select_resolution_spec(local, cross_unique, source_is_fragment) == Option::<T>::None,
{
}

// ---------------------------------------------------------------------------
// Obligation 3: source-span preservation
// ---------------------------------------------------------------------------
//
// Resolution is a decision over two already-computed answers. It reads neither
// the document nor the plan, so it cannot alter the source spans the renderer
// preserves; the obligation is that the decision leaves other text alone,
// which holds because the function has no other output channel.

/// Resolution depends on nothing but its three inputs.
///
/// Two calls with equal inputs return equal results, so resolution cannot
/// vary with hidden state such as map iteration order. The equal-inputs
/// premise is what makes this a statement about determinism rather than a
/// tautology.
proof fn resolution_is_a_function_of_its_inputs<T>(
    local_a: Option<T>,
    cross_a: Option<T>,
    fragment_a: bool,
    local_b: Option<T>,
    cross_b: Option<T>,
    fragment_b: bool,
)
    requires
        local_a == local_b,
        cross_a == cross_b,
        fragment_a == fragment_b,
    ensures
        select_resolution_spec(local_a, cross_a, fragment_a)
            == select_resolution_spec(local_b, cross_b, fragment_b),
{
}

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

fn main() {}
