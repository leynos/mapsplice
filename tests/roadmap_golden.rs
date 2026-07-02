//! Golden fixture coverage for roadmap operations and contracts.

mod golden;

use golden::{
    GoldenWorkspace,
    TestResult,
    assert_golden_case,
    create_workspace,
    reference_delete_case,
};
use rstest::{fixture, rstest};

#[fixture]
fn workspace() -> TestResult<GoldenWorkspace> {
    let workspace = create_workspace()?;
    Ok(workspace)
}

#[rstest]
#[serial_test::serial(cli_env)]
fn section_reference(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(&workspace?, reference_delete_case("section_reference", "1"))
}

#[rstest]
#[serial_test::serial(cli_env)]
fn version_quantity(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(&workspace?, reference_delete_case("version_quantity", "1"))
}

#[rstest]
#[serial_test::serial(cli_env)]
fn section_reference_outside_requires(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        reference_delete_case("section_reference_outside_requires", "1"),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn substring_non_match(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        reference_delete_case("substring_non_match", "1"),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn multi_id_requires(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(&workspace?, reference_delete_case("multi_id_requires", "1"))
}
