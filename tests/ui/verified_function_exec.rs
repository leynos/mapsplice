//! Compile-time check: the exec arm emits the function and ignores proof slots.

include!("../../src/roadmap/verified_function.macro.rs");

verified_function! {
    exec
    /// Return the input unchanged.
    #[must_use]
    fn identity(value: usize) -> usize;
    requires(value <= usize::MAX);
    ensures(result => result == value);
    proof_first {
        assert(true);
    }
    before {
    }
    proof_before {
        assert(true);
    }
    proof_after {
        assert(true);
    }
    after { value }
}

fn main() {
    assert_eq!(identity(7), 7);
}
