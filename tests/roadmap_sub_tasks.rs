//! `rstest` coverage for structural roadmap sub-task handling.

#[path = "support/phase.rs"]
mod phase_support;
#[path = "support/sub_tasks.rs"]
mod sub_task_support;
#[path = "support/roadmap_workspace.rs"]
mod workspace_support;

use mapsplice::{MapspliceError, parse_roadmap_text, run_from_args};
use phase_support::PHASE_FRAGMENT;
use rstest::rstest;
use sub_task_support::TARGET_WITH_SUB_TASKS;
use workspace_support::{TestResult, Workspace, workspace};

const TARGET_PHASE_EIGHT_WITH_SUB_TASK: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. First phase task.\n\n",
    "## 2. Phase two\n\n",
    "### 2.1. Step two\n\n",
    "- [ ] 2.1.1. Second phase task.\n\n",
    "## 3. Phase three\n\n",
    "### 3.1. Step three\n\n",
    "- [ ] 3.1.1. Third phase task.\n\n",
    "## 4. Phase four\n\n",
    "### 4.1. Step four\n\n",
    "- [ ] 4.1.1. Fourth phase task.\n\n",
    "## 5. Phase five\n\n",
    "### 5.1. Step five\n\n",
    "- [ ] 5.1.1. Fifth phase task.\n\n",
    "## 6. Phase six\n\n",
    "### 6.1. Step six\n\n",
    "- [ ] 6.1.1. Sixth phase task.\n\n",
    "## 7. Phase seven\n\n",
    "### 7.1. Step seven\n\n",
    "- [ ] 7.1.1. Seventh phase task.\n\n",
    "## 8. Phase eight\n\n",
    "### 8.1. Step eight one\n\n",
    "- [ ] 8.1.1. Earlier phase eight task.\n\n",
    "### 8.2. Step eight two\n\n",
    "- [ ] 8.2.1. First task.\n",
    "- [ ] 8.2.2. Second task.\n",
    "- [ ] 8.2.3. Parent task. Requires 8.2.3.1.\n",
    "    - [ ] 8.2.3.1. Nested sub-task. Requires 8.2.3.\n",
);

fn assert_contains(haystack: &str, needle: &str) {
    assert!(haystack.contains(needle));
}

fn assert_not_contains(haystack: &str, needle: &str) {
    assert!(!haystack.contains(needle));
}

fn assert_invalid_roadmap(error: &MapspliceError) {
    assert!(matches!(error, MapspliceError::InvalidRoadmap { .. }));
}

#[rstest]
fn parse_roadmap_keeps_nested_numbered_sub_tasks_structural() {
    let roadmap =
        parse_roadmap_text(TARGET_WITH_SUB_TASKS).expect("roadmap with sub-tasks should parse");
    let phase = roadmap.phases.first().expect("roadmap should have a phase");
    let step = phase.steps.first().expect("phase should have a step");
    let task = step.tasks.first().expect("step should have a task");
    let first_sub_task = task
        .sub_tasks
        .first()
        .expect("task should have a first sub-task");
    let second_sub_task = task
        .sub_tasks
        .get(1)
        .expect("task should have a second sub-task");

    assert_eq!(task.body.len(), 0);
    assert_eq!(task.sub_tasks.len(), 2);
    assert_eq!(first_sub_task.number.to_string(), "1.1.1.1");
    assert_eq!(second_sub_task.number.to_string(), "1.1.1.2");
}

#[rstest]
#[serial_test::serial(cli_env)]
fn append_renumbers_sub_tasks_and_their_dependencies(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(TARGET_WITH_SUB_TASKS)
        .expect("target should be written");
    test_workspace
        .write_fragment(PHASE_FRAGMENT)
        .expect("fragment should be written");

    let outcome = run_from_args([
        "mapsplice",
        "append",
        test_workspace.target.as_str(),
        test_workspace.fragment.as_str(),
    ])
    .expect("append command should succeed");
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "1.1.1. Parent task. Requires 1.1.1.1.");
    assert_contains(
        &stdout,
        "    - [ ] 1.1.1.1. First sub-task. Requires 1.1.1.",
    );
    assert_contains(&stdout, "- [ ] 1.1.2. Sibling task. Requires 1.1.1.2.");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_before_phase_moves_sub_task_and_rewrites_sub_task_references(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(TARGET_PHASE_EIGHT_WITH_SUB_TASK)
        .expect("target should be written");
    test_workspace
        .write_fragment(PHASE_FRAGMENT)
        .expect("fragment should be written");

    let outcome = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "8",
        test_workspace.fragment.as_str(),
    ])
    .expect("insert command should succeed");
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "- [ ] 9.2.3. Parent task. Requires 9.2.3.1.");
    assert_contains(&stdout, "- [ ] 9.2.3.1. Nested sub-task. Requires 9.2.3.");
    assert_not_contains(&stdout, "Requires 8.2.3.1.");
    assert_not_contains(&stdout, "- [ ] 8.2.3.1. Nested sub-task. Requires 8.2.3.");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn delete_renumbers_sub_tasks_with_parent_task(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. Removed task.\n",
            "- [ ] 1.1.2. Parent task.\n",
            "    - [ ] 1.1.2.1. Nested sub-task.\n",
        ))
        .expect("target should be written");

    let outcome = run_from_args([
        "mapsplice",
        "delete",
        test_workspace.target.as_str(),
        "1.1.1",
    ])
    .expect("delete command should succeed");
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "- [ ] 1.1.1. Parent task.");
    assert_contains(&stdout, "    - [ ] 1.1.1.1. Nested sub-task.");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn delete_sub_task_renumbers_later_sub_tasks(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. Parent task.\n",
            "    - [ ] 1.1.1.1. Removed sub-task.\n",
            "    - [x] 1.1.1.2. Second sub-task.\n",
            "- [ ] 1.1.2. Sibling task. Requires 1.1.1.2.\n",
        ))
        .expect("target should be written");

    let outcome = run_from_args([
        "mapsplice",
        "delete",
        test_workspace.target.as_str(),
        "1.1.1.1",
    ])
    .expect("sub-task delete should succeed");
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "    - [x] 1.1.1.1. Second sub-task.");
    assert_contains(&stdout, "- [ ] 1.1.2. Sibling task. Requires 1.1.1.1.");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn dependency_rewrites_inside_sub_task_text(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. Removed task.\n\n",
            "## 2. Phase two\n\n",
            "### 2.1. Step two\n\n",
            "- [ ] 2.1.1. Parent task.\n",
            "    - [ ] 2.1.1.1. Nested sub-task. Requires 2.1.1.\n",
        ))
        .expect("target should be written");

    let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "1"])
        .expect("delete command should succeed");
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "1.1.1.1. Nested sub-task. Requires 1.1.1.");
    Ok(())
}

#[rstest]
fn malformed_sub_task_parent_is_rejected() {
    let error = parse_roadmap_text(concat!(
        "# Example\n\n",
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "    - [ ] 1.1.2.1. Wrong parent.\n",
    ))
    .expect_err("sub-task must belong to parent task");

    assert_invalid_roadmap(&error);
}

#[rstest]
fn nested_roadmap_task_list_under_task_is_rejected() {
    let error = parse_roadmap_text(concat!(
        "# Example\n\n",
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "    - [ ] 1.1.2. Nested sibling task.\n",
    ))
    .expect_err("nested roadmap task lists should fail");

    assert_invalid_roadmap(&error);
}

#[rstest]
fn nested_roadmap_task_list_under_sub_task_is_rejected() {
    let error = parse_roadmap_text(concat!(
        "# Example\n\n",
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "    - [ ] 1.1.1.1. Nested sub-task.\n",
        "        - [ ] 1.1.2. Nested roadmap task.\n",
    ))
    .expect_err("sub-task body must reject roadmap-shaped lists");

    assert_invalid_roadmap(&error);
}
