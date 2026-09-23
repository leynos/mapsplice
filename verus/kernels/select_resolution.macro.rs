// The kernel body, shared verbatim between the proof and the product.
//
// This is a macro definition rather than a bare body because Whitaker's
// `bumpy_road_function` lint cannot see through `include!`. Tokens spliced by
// `include!` report `span.from_expansion() == false` while still carrying the
// *included* file's line numbers, so the lint compares them against the
// enclosing function's range in a different coordinate system and aborts the
// compiler with an internal error. Macro-expanded tokens carry the expansion
// context that lint already skips, so wrapping the splice in a macro is what
// lets the two sides share one text at all.
//
// `verus/lib.rs` expands this under a proof; `src/roadmap/ops/remap_kernel.rs`
// expands it in production. Keep those two call sites in step.
//
// One consequence is worth stating plainly: because the expansion is what the
// lint sees, the kernel body is opaque to `bumpy_road_function` in both
// crates. That lint has nothing to report here — the body holds one `if`
// inside one match arm — but a future kernel with genuinely nested conditionals
// would go unflagged by it.
macro_rules! select_resolution_body {
    ($local:expr, $cross_unique:expr, $source_is_fragment:expr) => {
        match $local {
            Some(mapped) => Some(mapped),
            None => {
                if $source_is_fragment { $cross_unique } else { None }
            }
        }
    };
}
