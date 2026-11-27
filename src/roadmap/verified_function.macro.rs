// The whole-function splice used by the verifiable kernels.
//
// A kernel body that needs a loop invariant cannot be shared by inserting the
// annotation into an already parsed loop: the ordinary Rust compiler rejects
// `invariant` before Verus ever sees it. The unit of sharing is therefore the
// whole function. Both arms below splice the same `before`, `loop_body` and
// `after` token streams in the same order; the `verified` arm adds the
// contract and loop annotations and wraps the result in `verus!`.
//
// The ghost-erasure audit rule follows from that shape. The only slots the
// `exec` arm emits are the signature's executable tokens, `before`, the loop
// condition, `loop_body`, and `after`. Every proof slot — `proof_first`,
// `proof_before`, `proof_loop`, `proof_after` — is emitted only by the
// `verified` arm, so it cannot change an executable branch, argument,
// conversion, binding, return value or effect. A change to any executable slot
// changes both expansions; a change confined to a proof slot changes neither
// executable expansion. Keep it that way: a new slot that both arms emit is a
// change to the shipped body and must be reviewed as one.
//
// Every caller writes the executable body once, at the invocation site in
// the source-span macro files. `source_span_kernel.rs` includes this file and
// invokes the `exec` arm; `verus/lib.rs` includes it and invokes the `verified`
// arm. The file is domain-owned and the proof reaches into `src/` for it, so
// production names no proof path.
//
// One consequence is deliberate: Whitaker's `bumpy_road_function` lint sees an
// opaque macro expansion, so nested conditionals inside these kernels are not
// inspected by it. `docs/adrs/adr-0003-shared-verified-kernels.md` records the
// limitation and the compensating controls.
/// Emit one function from one executable body.
///
/// The `exec` arm is ordinary Rust with the contracts, invariants and proof
/// slots removed. The `verified` arm is the same executable tokens with the
/// contracts and annotations added, wrapped in `verus!` so Verus parses the
/// result as its own syntax.
///
/// For example, a caller that writes
/// `verified_function!(exec ... fn f(x: usize) -> usize; requires(x <= 3);
/// ensures(result => result == x); proof_first { } before { } proof_before { }
/// proof_after { } after { x })` emits `fn f(x: usize) -> usize { x }`.
macro_rules! verified_function {
    (
        exec
        $(#[$attribute:meta])*
        $visibility:vis fn $name:ident($($arguments:tt)*) -> $result:ty;
        requires($($requirement:tt)*);
        ensures($result_name:ident => $($postcondition:tt)*);
        proof_first { $($proof_first:tt)* }
        before { $($before:tt)* }
        proof_before { $($proof_before:tt)* }
        $(while ($condition:expr)
            invariant($($invariant:tt)*)
            decreases($($decrease:tt)*) {
                $($loop_body:tt)*
            }
            proof_loop { $($proof_loop:tt)* }
            loop_step { $($loop_step:tt)* }
        )?
        proof_after { $($proof_after:tt)* }
        after { $($after:tt)* }
    ) => {
        $(#[$attribute])*
        $visibility fn $name($($arguments)*) -> $result {
            $($before)*
            $(while $condition {
                $($loop_body)*
                $($loop_step)*
            })?
            $($after)*
        }
    };
    (
        verified
        $(#[$attribute:meta])*
        $visibility:vis fn $name:ident($($arguments:tt)*) -> $result:ty;
        requires($($requirement:tt)*);
        ensures($result_name:ident => $($postcondition:tt)*);
        proof_first { $($proof_first:tt)* }
        before { $($before:tt)* }
        proof_before { $($proof_before:tt)* }
        $(while ($condition:expr)
            invariant($($invariant:tt)*)
            decreases($($decrease:tt)*) {
                $($loop_body:tt)*
            }
            proof_loop { $($proof_loop:tt)* }
            loop_step { $($loop_step:tt)* }
        )?
        proof_after { $($proof_after:tt)* }
        after { $($after:tt)* }
    ) => {
        verus! {
            $(#[$attribute])*
            $visibility fn $name($($arguments)*) -> ($result_name: $result)
                requires $($requirement)*
                ensures $($postcondition)*
            {
                proof { $($proof_first)* }
                $($before)*
                proof { $($proof_before)* }
                $(while $condition
                    invariant $($invariant)*
                    decreases $($decrease)*
                {
                    $($loop_body)*
                    proof { $($proof_loop)* }
                    $($loop_step)*
                })?
                proof { $($proof_after)* }
                $($after)*
            }
        }
    };
}