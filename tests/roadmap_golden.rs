//! Golden fixture coverage for roadmap operations and contracts.

mod golden;

use golden::{
    FixturePath,
    GoldenCase,
    GoldenCommand,
    GoldenExpectation,
    GoldenWorkspace,
    SuccessOutput,
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

fn golden_success_case(
    name: &'static str,
    command: GoldenCommand,
    has_fragment: bool,
) -> GoldenCase {
    GoldenCase {
        name,
        command,
        target: golden_fixture(name, "target.md"),
        fragment: has_fragment.then_some(golden_fixture(name, "fragment.md")),
        expectation: GoldenExpectation::Success {
            output: SuccessOutput::Stdout {
                expected: golden_fixture(name, "expected.md"),
            },
        },
    }
}

const fn golden_fixture(case: &'static str, file: &'static str) -> FixturePath {
    FixturePath::Golden { case, file }
}

#[rstest]
#[serial_test::serial(cli_env)]
fn append_phase(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case("append_phase", GoldenCommand::Append, true),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_phase_before(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "insert_phase_before",
            GoldenCommand::InsertBefore { anchor: "2" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_step_after(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "insert_step_after",
            GoldenCommand::InsertAfter { anchor: "1.1" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_task_before(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "insert_task_before",
            GoldenCommand::InsertBefore { anchor: "1.1.2" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_sub_task_after(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "insert_sub_task_after",
            GoldenCommand::InsertAfter { anchor: "1.1.1.1" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn delete_task(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "delete_task",
            GoldenCommand::Delete { anchor: "1.1.2" },
            false,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn replace_step(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "replace_step",
            GoldenCommand::Replace { anchor: "1.2" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn replace_sub_task(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "replace_sub_task",
            GoldenCommand::Replace { anchor: "1.1.1.2" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn preamble_preserved(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "preamble_preserved",
            GoldenCommand::InsertAfter { anchor: "1.1.1" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn phase_step_task_surface(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "phase_step_task_surface",
            GoldenCommand::InsertAfter { anchor: "1" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn multi_line_task_body(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "multi_line_task_body",
            GoldenCommand::InsertAfter { anchor: "1.1.2" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn nested_bullets(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "nested_bullets",
            GoldenCommand::InsertAfter { anchor: "1.1.2" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn tables_preserved(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "tables_preserved",
            GoldenCommand::InsertAfter { anchor: "1.1.2" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn code_blocks_preserved(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "code_blocks_preserved",
            GoldenCommand::InsertAfter { anchor: "1.1.2" },
            true,
        ),
    )
}

#[rstest]
#[serial_test::serial(cli_env)]
fn addendum_body_surface(workspace: TestResult<GoldenWorkspace>) -> TestResult {
    assert_golden_case(
        &workspace?,
        golden_success_case(
            "addendum_body_surface",
            GoldenCommand::InsertAfter { anchor: "1.1.1" },
            true,
        ),
    )
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
