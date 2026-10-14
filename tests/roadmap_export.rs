//! CLI export, deterministic JSON and drift-check integration coverage.

#[path = "support/roadmap_workspace.rs"]
mod workspace_support;

use camino::Utf8PathBuf;
use mapsplice::{CliRequest, CommandKind, ExportFormat, GlobalOptions, run_from_args, run_request};
use rstest::rstest;
use workspace_support::{TestResult, Workspace, workspace};

const BASIC_ROADMAP: &str = include_str!("roadmap_golden/export_basic.md");
const GOLDEN_JSON: &str = include_str!("roadmap_golden/export_basic.json");
const REPOSITORY_ROADMAP: &str = include_str!("../docs/roadmap.md");
const NESTED_REQUIRES_ROADMAP: &str = concat!(
    "## 1. Phase\n\n",
    "### 1.1. Step\n\n",
    "- [ ] 1.1.1. Prerequisite.\n",
    "- [ ] 1.1.2. Consumer.\n",
    "  - Requires 1.1.1.\n",
);
const INSERTED_TASK: &str = "- [ ] 1.1.1. Inserted task.\n";

fn export(workspace: &Workspace) -> TestResult<String> {
    run_from_args(["mapsplice", "export", workspace.target.as_str()])?
        .stdout
        .ok_or_else(|| "export should write JSON to stdout".into())
}

fn write_committed_export(workspace: &Workspace, contents: &str) -> TestResult<Utf8PathBuf> {
    workspace.dir.write("roadmap.json", contents)?;
    Ok(workspace.target.with_file_name("roadmap.json"))
}

fn require(condition: bool, message: impl Into<String>) -> TestResult {
    if condition {
        Ok(())
    } else {
        Err(message.into().into())
    }
}

fn require_equal<T: std::fmt::Debug + PartialEq>(
    actual: &T,
    expected: &T,
    description: &str,
) -> TestResult {
    require(
        actual == expected,
        format!("{description}: expected {expected:?}, got {actual:?}"),
    )
}

fn require_contains(actual: &str, expected: &str) -> TestResult {
    require(
        actual.contains(expected),
        format!("expected {actual:?} to contain {expected:?}"),
    )
}

fn task_array_mut(document: &mut serde_json::Value) -> TestResult<&mut Vec<serde_json::Value>> {
    document
        .get_mut("phases")
        .and_then(serde_json::Value::as_array_mut)
        .and_then(|phases| phases.get_mut(0))
        .and_then(|phase| phase.get_mut("steps"))
        .and_then(serde_json::Value::as_array_mut)
        .and_then(|steps| steps.get_mut(0))
        .and_then(|step| step.get_mut("tasks"))
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| "the export should have an ordered task array".into())
}

fn task_requires(
    document: &serde_json::Value,
    task_index: usize,
) -> TestResult<&serde_json::Value> {
    document
        .get("phases")
        .and_then(serde_json::Value::as_array)
        .and_then(|phases| phases.first())
        .and_then(|phase| phase.get("steps"))
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("tasks"))
        .and_then(serde_json::Value::as_array)
        .and_then(|tasks| tasks.get(task_index))
        .and_then(|task| task.get("requires"))
        .ok_or_else(|| "the requested task should have dependencies".into())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn export_is_byte_stable_and_matches_the_golden_pair(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(BASIC_ROADMAP)?;

    let first = export(&test_workspace)?;
    let second = export(&test_workspace)?;

    require_equal(&first.as_bytes(), &second.as_bytes(), "export bytes")?;
    require(first == GOLDEN_JSON, "export should match the golden JSON")?;
    require(
        !first.contains("Preamble excluded"),
        "preamble must be excluded",
    )?;
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn export_of_repository_roadmap_matches_snapshot(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(REPOSITORY_ROADMAP)?;

    insta::assert_snapshot!(export(&test_workspace)?);
    Ok(())
}

#[rstest]
#[case::soft_wrap(
    "## 1. Planning\n\n### 1.1. Export\n\n- [ ] 1.1.1. Emit JSON. Requires\n  1.1.2.\n- [ ] \
     1.1.2. Check JSON.\n"
)]
#[case::crlf(
    "## 1. Planning\r\n\r\n### 1.1. Export\r\n\r\n- [ ] 1.1.1. Emit JSON. Requires 1.1.2.\r\n- [ \
     ] 1.1.2. Check JSON.\r\n"
)]
#[case::trailing_spaces(
    "## 1. Planning   \n\n### 1.1. Export\n\n- [ ] 1.1.1. Emit JSON. Requires 1.1.2.   \n- [ ] \
     1.1.2. Check JSON.\n"
)]
#[serial_test::serial(cli_env)]
fn export_normalizes_wrapping_and_whitespace(
    workspace: TestResult<Workspace>,
    #[case] variant: &str,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(variant)?;
    let variant_output = export(&test_workspace)?;

    test_workspace.write_target(concat!(
        "## 1. Planning\n\n### 1.1. Export\n\n",
        "- [ ] 1.1.1. Emit JSON. Requires 1.1.2.\n",
        "- [ ] 1.1.2. Check JSON.\n",
    ))?;
    require_equal(
        &variant_output,
        &export(&test_workspace)?,
        "normalized export",
    )?;
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn export_normalizes_subtask_indentation(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(concat!(
        "## 1. Planning\n\n### 1.1. Export\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  - [ ] 1.1.1.1. Structural detail.\n",
        "    - [x] Nested checkbox detail.\n",
    ))?;
    let expected = export(&test_workspace)?;

    test_workspace.write_target(concat!(
        "## 1. Planning\n\n### 1.1. Export\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "    - [ ] 1.1.1.1. Structural detail.\n",
        "        - [x] Nested checkbox detail.\n",
    ))?;
    require_equal(&export(&test_workspace)?, &expected, "indented export")?;
    Ok(())
}

#[path = "roadmap_export/drift.rs"]
mod drift;
#[path = "roadmap_export/strict.rs"]
mod strict;
