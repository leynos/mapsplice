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

/// Export the workspace roadmap and return the JSON written to standard output.
fn export(workspace: &Workspace) -> TestResult<String> {
    run_from_args(["mapsplice", "export", workspace.target.as_str()])?
        .stdout
        .ok_or_else(|| "export should write JSON to stdout".into())
}

/// Write a committed export beside the workspace roadmap and return its path.
fn write_committed_export(workspace: &Workspace, contents: &str) -> TestResult<Utf8PathBuf> {
    workspace.dir.write("roadmap.json", contents)?;
    Ok(workspace.target.with_file_name("roadmap.json"))
}

/// Fail the test with `message` when `condition` does not hold.
fn require(condition: bool, message: impl Into<String>) -> TestResult {
    if condition {
        Ok(())
    } else {
        Err(message.into().into())
    }
}

/// Fail the test when `actual` and `expected` differ.
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

/// Fail the test when `actual` does not contain `expected`.
fn require_contains(actual: &str, expected: &str) -> TestResult {
    require(
        actual.contains(expected),
        format!("expected {actual:?} to contain {expected:?}"),
    )
}

/// Assert one exported object carries every required key.
fn require_keys(node: &serde_json::Value, keys: &[&str], kind: &str) -> TestResult {
    let object = node
        .as_object()
        .ok_or_else(|| format!("{kind}: expected an object, got {node:?}"))?;
    for key in keys {
        require(
            object.contains_key(*key),
            format!("{kind}: missing `{key}` in {node:?}"),
        )?;
    }
    Ok(())
}

/// Borrow every array element of one key, treating an absent key as empty.
fn members<'a>(node: &'a serde_json::Value, key: &str) -> &'a [serde_json::Value] {
    node.get(key)
        .and_then(serde_json::Value::as_array)
        .map_or(&[], Vec::as_slice)
}

/// Borrow the ordered task array of the first exported step.
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

/// Borrow the dependency list of one exported task.
fn task_requires(
    document: &serde_json::Value,
    task_index: usize,
) -> TestResult<&serde_json::Value> {
    task_value(document, task_index)?
        .get("requires")
        .ok_or_else(|| "the requested task should have dependencies".into())
}

/// Borrow one task object from the first exported step.
fn task_value(document: &serde_json::Value, task_index: usize) -> TestResult<&serde_json::Value> {
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
        .ok_or_else(|| "the requested task should be exported".into())
}

/// Borrow the dependency list of one checkbox detail on an exported task.
fn detail_requires(
    document: &serde_json::Value,
    task_index: usize,
    detail_index: usize,
) -> TestResult<&serde_json::Value> {
    task_value(document, task_index)?
        .get("details")
        .and_then(serde_json::Value::as_array)
        .and_then(|details| details.get(detail_index))
        .and_then(|detail| detail.get("requires"))
        .ok_or_else(|| "the requested checkbox detail should have dependencies".into())
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

/// The repository roadmap is the largest real document, so exporting it twice
/// must be byte-stable and its structure must satisfy the documented schema.
///
/// This deliberately avoids a full-document snapshot: routine roadmap edits
/// would otherwise churn a thousand-line artefact without testing export
/// behaviour. Byte-exact coverage stays with the fixed golden pair.
#[rstest]
#[serial_test::serial(cli_env)]
fn export_of_repository_roadmap_is_deterministic_and_well_formed(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(REPOSITORY_ROADMAP)?;

    let first = export(&test_workspace)?;
    let second = export(&test_workspace)?;
    require_equal(
        &first.as_bytes(),
        &second.as_bytes(),
        "repository export bytes",
    )?;

    let document: serde_json::Value = serde_json::from_str(&first)
        .map_err(|error| format!("the export should be valid JSON: {error}"))?;
    require_equal(
        &document.get("format_version"),
        &Some(&serde_json::json!(1)),
        "format version",
    )?;

    let phases = members(&document, "phases");
    require(!phases.is_empty(), "the roadmap should export phases")?;
    let steps = phases
        .iter()
        .flat_map(|phase| members(phase, "steps"))
        .collect::<Vec<_>>();
    require(!steps.is_empty(), "the roadmap should export steps")?;
    let tasks = steps
        .iter()
        .flat_map(|step| members(step, "tasks"))
        .collect::<Vec<_>>();
    require(!tasks.is_empty(), "the roadmap should export tasks")?;

    for phase in phases {
        require_keys(phase, &["number", "title", "steps"], "phase")?;
    }
    for step in &steps {
        require_keys(step, &["number", "title", "tasks"], "step")?;
    }
    let mut numbers = std::collections::BTreeSet::new();
    for task in &tasks {
        require_keys(
            task,
            &["number", "title", "state", "requires", "entries", "details"],
            "task",
        )?;
        let number = task
            .get("number")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "every task should export a string number".to_owned())?;
        require(
            numbers.insert(number.to_owned()),
            format!("task number `{number}` should be unique"),
        )?;
    }
    require(
        numbers.contains("1.1.1"),
        "the export should include task `1.1.1`",
    )?;
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn hard_wrapped_and_listed_dependencies_export_as_exact_arrays(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(concat!(
        "## 1. Planning\n\n### 1.1. Export\n\n",
        // A hard-wrapped clause and an anchor list on the task itself.
        "- [ ] 1.1.1. Consumer. Requires\n  1.1.2,\n  1.1.3\n",
        "- [ ] 1.1.2. First prerequisite.\n",
        // The same two layouts inside an unnumbered checkbox detail.
        "- [ ] 1.1.3. Second prerequisite.\n",
        "  - [x] Detail. Requires 1.1.1,\n    1.1.2\n",
    ))?;

    let document: serde_json::Value = serde_json::from_str(&export(&test_workspace)?)
        .map_err(|error| format!("the export should be valid JSON: {error}"))?;

    require_equal(
        task_requires(&document, 0)?,
        &serde_json::json!(["1.1.2", "1.1.3"]),
        "hard-wrapped task dependencies",
    )?;
    require_equal(
        detail_requires(&document, 2, 0)?,
        &serde_json::json!(["1.1.1", "1.1.2"]),
        "wrapped checkbox-detail dependencies",
    )?;
    Ok(())
}

#[rstest]
#[case::soft_wrap(concat!(
    "## 1. Planning\n\n### 1.1. Export\n\n",
    "- [ ] 1.1.1. Emit JSON. Requires\n  1.1.2.\n",
    "- [ ] 1.1.2. Check JSON.\n",
))]
#[case::crlf(concat!(
    "## 1. Planning\r\n\r\n### 1.1. Export\r\n\r\n",
    "- [ ] 1.1.1. Emit JSON. Requires 1.1.2.\r\n",
    "- [ ] 1.1.2. Check JSON.\r\n",
))]
#[case::trailing_spaces(concat!(
    "## 1. Planning   \n\n### 1.1. Export\n\n",
    "- [ ] 1.1.1. Emit JSON. Requires 1.1.2.   \n",
    "- [ ] 1.1.2. Check JSON.\n",
))]
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

#[path = "roadmap_export/diagnostics.rs"]
mod diagnostics;
#[path = "roadmap_export/drift.rs"]
mod drift;
#[path = "roadmap_export/strict.rs"]
mod strict;
