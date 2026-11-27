//! Compile-time coverage for the public library API.

#[test]
fn compile_time_contracts() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/public_api.rs");
    cases.pass("tests/ui/verified_function_exec.rs");
    cases.compile_fail("tests/ui/model_invariants.rs");
    cases.compile_fail("tests/ui/metrics_snapshot_non_exhaustive.rs");
    cases.compile_fail("tests/ui/verified_function_malformed.rs");
}
