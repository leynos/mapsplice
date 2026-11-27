//! Compile-fail check: every declared slot is required by the macro contract.

include!("../../src/roadmap/verified_function.macro.rs");

verified_function! {
    exec
    fn identity(value: usize) -> usize;
    requires(true);
    ensures(result => result == value);
    before {
    }
    proof_before {
    }
    proof_after {
    }
    after { value }
}

fn main() {}
