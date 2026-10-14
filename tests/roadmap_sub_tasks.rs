//! `rstest` coverage for structural roadmap sub-task handling.

#[path = "support/assertions.rs"]
mod assertions;
#[path = "support/phase.rs"]
mod phase_support;
#[path = "support/sub_tasks.rs"]
mod sub_task_support;
#[path = "support/roadmap_workspace.rs"]
mod workspace_support;

use assertions::assert_contains;
use mapsplice::{
    MapspliceError,
    RoadmapDocument,
    RoadmapOperation,
    SubTaskEntry,
    apply_command,
    parse_anchor,
    parse_fragment,
    parse_roadmap_text,
    render_roadmap,
    run_from_args,
};
use phase_support::PHASE_FRAGMENT;
use rstest::rstest;
use sub_task_support::TARGET_WITH_SUB_TASKS;
use workspace_support::{TestResult, Workspace, workspace};

const TARGET_PHASE_EIGHT_WITH_SUB_TASK: &str =
    include_str!("fixtures/roadmap_sub_tasks/target_phase_eight_with_sub_task.txt");

fn sub_task_numbers(sub_tasks: &[SubTaskEntry]) -> String {
    sub_tasks
        .iter()
        .map(|sub_task| sub_task.number.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn assert_render_order(rendered: &str, markers: &[&str]) {
    let positions = markers
        .iter()
        .map(|marker| {
            assert_eq!(
                rendered.matches(marker).count(),
                1,
                "each expected rendered marker should appear exactly once: {marker}\n{rendered}"
            );
            rendered
                .match_indices(*marker)
                .map(|(position, _)| position)
                .next()
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    assert!(
        positions
            .iter()
            .zip(positions.iter().skip(1))
            .all(|(previous, next)| previous < next),
        "canonical rendering should preserve mixed child order:\n{rendered}"
    );
}

fn render_after_inserting_before_parent(
    mut roadmap: RoadmapDocument,
) -> Result<String, MapspliceError> {
    let anchor = parse_anchor("1.1.2")?;
    let fragment = parse_fragment("- [ ] 1.1.2. Inserted task.\n")?;
    apply_command(
        &mut roadmap,
        RoadmapOperation::Insert {
            anchor,
            after: false,
        },
        Some(fragment),
    )?;
    render_roadmap(&roadmap)
}

fn assert_not_contains(haystack: &str, needle: &str) {
    assert!(
        !haystack.contains(needle),
        "unexpected `{needle}` in:\n{haystack}"
    );
}

fn assert_invalid_roadmap(error: &MapspliceError) {
    assert!(
        matches!(error, MapspliceError::InvalidRoadmap { .. }),
        "expected invalid roadmap"
    );
}

#[rstest]
fn parse_roadmap_keeps_nested_numbered_sub_tasks_structural() {
    let roadmap =
        parse_roadmap_text(TARGET_WITH_SUB_TASKS).expect("roadmap with sub-tasks should parse");
    let phase = roadmap.phases.first().expect("roadmap should have a phase");
    let step = phase.steps.first().expect("phase should have a step");
    let task = step.tasks().first().expect("step should have a task");
    let first_sub_task = task
        .sub_tasks()
        .first()
        .expect("task should have a first sub-task");
    let second_sub_task = task
        .sub_tasks()
        .get(1)
        .expect("task should have a second sub-task");

    assert_eq!(task.body.len(), 0);
    assert_eq!(task.sub_tasks().len(), 2);
    assert_eq!(first_sub_task.number.to_string(), "1.1.1.1");
    assert_eq!(second_sub_task.number.to_string(), "1.1.1.2");
}

#[rstest]
#[case::body_before(
    "  - A non-structural body bullet.\n  - [ ] 1.1.2.1. Sub one.\n",
    "1.1.2.1",
    "1.1.3.1",
    &["- [ ] 1.1.3. Parent.", "  - A non-structural body bullet.", "  - [ ] 1.1.3.1. Sub one."]
)]
#[case::body_between(
    "  - [ ] 1.1.2.1. Sub one.\n  - A non-structural body bullet.\n  - [ ] 1.1.2.2. Sub two.\n",
    "1.1.2.1, 1.1.2.2",
    "1.1.3.1, 1.1.3.2",
    &["- [ ] 1.1.3. Parent.", "  - [ ] 1.1.3.1. Sub one.", "  - A non-structural body bullet.", "  - [ ] 1.1.3.2. Sub two."]
)]
#[case::body_after(
    "  - [ ] 1.1.2.1. Sub one.\n  - A non-structural body bullet.\n",
    "1.1.2.1",
    "1.1.3.1",
    &["- [ ] 1.1.3. Parent.", "  - [ ] 1.1.3.1. Sub one.", "  - A non-structural body bullet."]
)]
fn mixed_nested_list_classification_survives_canonical_renumbering(
    #[case] nested_items: &str,
    #[case] expected_sub_tasks: &str,
    #[case] expected_rendered_sub_tasks: &str,
    #[case] expected_rendered_order: &[&str],
) {
    let roadmap = parse_roadmap_text(&format!(
        "# Example\n\n## 1. Phase one\n\n### 1.1. Step one\n\n- [ ] 1.1.1. First task.\n- [ ] \
         1.1.2. Parent.\n{nested_items}"
    ))
    .expect("mixed nested task list should parse");
    let task = roadmap
        .phases
        .first()
        .and_then(|phase| phase.steps.first())
        .and_then(|step| step.tasks().get(1))
        .expect("roadmap should contain the parent task");
    assert_eq!(sub_task_numbers(task.sub_tasks()), expected_sub_tasks);

    let rendered = render_after_inserting_before_parent(roadmap)
        .expect("inserting before the parent should render the renumbered children");
    assert_render_order(&rendered, expected_rendered_order);

    let reparsed = parse_roadmap_text(&rendered).expect("canonical output should reparse");
    let reparsed_task = reparsed
        .phases
        .first()
        .and_then(|phase| phase.steps.first())
        .and_then(|step| {
            step.tasks()
                .iter()
                .find(|candidate| candidate.number().to_string() == "1.1.3")
        })
        .expect("reparsed roadmap should contain the moved parent task");
    assert_eq!(
        sub_task_numbers(reparsed_task.sub_tasks()),
        expected_rendered_sub_tasks
    );
    assert_eq!(
        render_roadmap(&reparsed).expect("reparsed roadmap should render"),
        rendered,
        "canonical mixed-list output should be stable after reparsing"
    );
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
    assert_contains(&stdout, "  - [ ] 1.1.1.1. First sub-task. Requires 1.1.1.");
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
            "  - [ ] 1.1.2.1. Nested sub-task.\n",
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
    assert_contains(&stdout, "  - [ ] 1.1.1.1. Nested sub-task.");
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
            "  - [ ] 1.1.1.1. Removed sub-task.\n",
            "  - [x] 1.1.1.2. Second sub-task.\n",
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

    assert_contains(&stdout, "  - [x] 1.1.1.1. Second sub-task.");
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
            "  - [ ] 2.1.1.1. Nested sub-task. Requires 2.1.1.\n",
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
        "  - [ ] 1.1.2.1. Wrong parent.\n",
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
        "  - [ ] 1.1.2. Nested sibling task.\n",
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
        "  - [ ] 1.1.1.1. Nested sub-task.\n",
        "    - [ ] 1.1.2. Nested roadmap task.\n",
    ))
    .expect_err("sub-task body must reject roadmap-shaped lists");

    assert_invalid_roadmap(&error);
}
