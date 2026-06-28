//! Compile-time coverage for the public library API.

#[test]
fn public_api_compiles_for_basic_callers() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/public_api.rs");
}
