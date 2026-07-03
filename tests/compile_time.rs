//! Compile-time coverage for the public library API.

#[test]
fn compile_time_contracts() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/public_api.rs");
    cases.compile_fail("tests/ui/model_invariants.rs");
}
