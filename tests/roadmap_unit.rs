//! `rstest` coverage for roadmap parsing and splice semantics.

#[path = "support/unit.rs"]
mod support;

use std::fmt::Debug;

use mapsplice::{
    MapspliceError,
    RoadmapItemLevel,
    fragment_level,
    parse_anchor,
    parse_fragment_text,
    parse_roadmap_text,
    run_from_args,
};
use rstest::rstest;
use support::{
    EnvVarGuard,
    PHASE_FRAGMENT,
    REPLACEMENT_FRAGMENT,
    TARGET_THREE_PHASES,
    TARGET_TWO_PHASES,
    TARGET_TWO_TASKS,
    TASK_FRAGMENT,
    TestResult,
    Workspace,
    workspace,
};

#[rstest]
#[case("8", "8")]
#[case("8.2", "8.2")]
#[case("8.2.3", "8.2.3")]
#[case("8.2.3.4", "8.2.3.4")]
fn parse_anchor_accepts_supported_forms(#[case] raw: &str, #[case] expected: &str) {
    let anchor = parse_anchor(raw).expect("supported anchors should parse");
    assert_eq!(anchor.to_string(), expected);
}

#[rstest]
#[case("8.")]
#[case("8.2.")]
#[case("0")]
#[case("01")]
#[case("8.02")]
#[case("8.2.0")]
#[case("8.2.3.0")]
#[case("a.b")]
#[case("8.2.3.4.5")]
fn parse_anchor_rejects_invalid_forms(#[case] raw: &str) {
    let error = parse_anchor(raw).expect_err("invalid anchors must be rejected");
    assert!(matches!(error, MapspliceError::InvalidAnchor { .. }));
}

#[rstest]
#[case(
    "## 9. Phase\n\n### 9.1. Step\n\n- [ ] 9.1.1. Task.\n",
    RoadmapItemLevel::Phase
)]
#[case("### 9.2. Step\n\n- [ ] 9.2.1. Task.\n", RoadmapItemLevel::Step)]
#[case("- [ ] 9.9.9. Task.\n", RoadmapItemLevel::Task)]
fn parse_fragment_detects_supported_level(
    #[case] fragment: &str,
    #[case] expected: RoadmapItemLevel,
) {
    let parsed = parse_fragment_text(fragment).expect("supported fragment should parse");
    assert_eq!(fragment_level(&parsed), expected);
}

#[rstest]
fn parse_roadmap_keeps_preamble_and_structure() {
    let roadmap = parse_roadmap_text(concat!(
        "# Example\n\n",
        "## Guiding principles\n\n",
        "- Be careful.\n\n",
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. First task.\n",
    ))
    .expect("supported roadmap should parse");

    assert_eq!(roadmap.preamble.len(), 3);
    assert_eq!(roadmap.phases.len(), 1);
    let first_phase = roadmap
        .phases
        .first()
        .expect("roadmap should contain one phase");
    assert_eq!(first_phase.steps.len(), 1);
    let first_step = first_phase
        .steps
        .first()
        .expect("phase should contain one step");
    assert_eq!(first_step.tasks.len(), 1);
}

fn assert_contains(haystack: &str, needle: &str) {
    assert!(haystack.contains(needle));
}

fn assert_equal<T>(actual: &T, expected: &T)
where
    T: Debug + PartialEq,
{
    assert_eq!(actual, expected);
}

fn assert_dangling_dependency(error: &MapspliceError) {
    assert!(matches!(error, MapspliceError::DanglingDependency { .. }));
}

fn assert_level_mismatch(error: &MapspliceError) {
    assert!(matches!(error, MapspliceError::LevelMismatch { .. }));
}

fn assert_anchor_not_found(error: &MapspliceError) {
    assert!(matches!(error, MapspliceError::AnchorNotFound { .. }));
}

fn assert_configuration_error(error: &MapspliceError) {
    assert!(matches!(error, MapspliceError::Configuration { .. }));
}

fn assert_invalid_roadmap(error: &MapspliceError) {
    assert!(matches!(error, MapspliceError::InvalidRoadmap { .. }));
}

const TARGET_WITH_SUB_TASKS: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. Parent task. Requires 1.1.1.1.\n",
    "    - [ ] 1.1.1.1. First sub-task. Requires 1.1.1.\n",
    "    - [x] 1.1.1.2. Second sub-task.\n",
    "- [ ] 1.1.2. Sibling task. Requires 1.1.1.2.\n",
);

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
        "    - [ ] 1.1.1.1. First sub\\-task. Requires 1.1.1.",
    );
    assert_contains(&stdout, "- [ ] 1.1.2. Sibling task. Requires 1.1.1.2.");
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
    assert_contains(&stdout, "    - [ ] 1.1.1.1. Nested sub\\-task.");
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

    assert_contains(&stdout, "    - [x] 1.1.1.1. Second sub\\-task.");
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

    assert_contains(&stdout, "1.1.1.1. Nested sub\\-task. Requires 1.1.1.");
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
#[serial_test::serial(cli_env)]
fn render_preserves_code_metadata_and_blockquote_spacing(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "> First paragraph.\n",
            ">\n",
            "> Second paragraph.\n\n",
            "```rust ignore\n",
            "fn main() {}\n",
            "```\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. First task.\n",
        ))
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

    assert_contains(&stdout, "> First paragraph.\n>\n> Second paragraph.");
    assert_contains(&stdout, "```rust ignore\nfn main() {}\n```");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn append_emits_stdout_and_keeps_target_unchanged(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(TARGET_TWO_PHASES)
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

    let expected = concat!(
        "# Example\n\n",
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. First task.\n\n",
        "## 2. Phase two\n\n",
        "### 2.1. Step two\n\n",
        "- [ ] 2.1.1. Second task. Requires 2.1.1.\n\n",
        "## 3. Inserted phase\n\n",
        "### 3.1. Added step\n\n",
        "- [ ] 3.1.1. Added task. Requires 3.1.1.",
    );
    assert_equal(&outcome.stdout.as_deref(), &Some(expected));
    assert_equal(
        &test_workspace.read_target()?,
        &TARGET_TWO_PHASES.to_owned(),
    );
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_before_phase_renumbers_later_phases_and_dependencies(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(TARGET_TWO_PHASES)
        .expect("target should be written");
    test_workspace
        .write_fragment(PHASE_FRAGMENT)
        .expect("fragment should be written");

    let outcome = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "2",
        test_workspace.fragment.as_str(),
    ])
    .expect("insert command should succeed");

    let stdout = outcome.stdout.unwrap_or_default();
    assert_contains(&stdout, "## 3. Phase two");
    assert_contains(&stdout, "Requires 3.1.1.");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_after_task_renumbers_later_tasks_within_the_step(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(TARGET_TWO_TASKS)
        .expect("target should be written");
    test_workspace
        .write_fragment(TASK_FRAGMENT)
        .expect("fragment should be written");

    let outcome = run_from_args([
        "mapsplice",
        "insert",
        "--after",
        test_workspace.target.as_str(),
        "1.1.1",
        test_workspace.fragment.as_str(),
    ])
    .expect("insert-after command should succeed");

    let stdout = outcome.stdout.unwrap_or_default();
    assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
    assert_contains(
        &stdout,
        "- [ ] 1.1.3. Second task. Depends on 1.1.1 and 1.1.2.",
    );
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn dangling_dependency_is_rejected(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. First task. Requires 99.1.1.\n",
        ))
        .expect("target should be written");
    test_workspace
        .write_fragment(PHASE_FRAGMENT)
        .expect("fragment should be written");

    let error = run_from_args([
        "mapsplice",
        "append",
        test_workspace.target.as_str(),
        test_workspace.fragment.as_str(),
    ])
    .expect_err("dangling dependency references must fail");

    assert_dangling_dependency(&error);
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn delete_phase_rewrites_downstream_identifiers(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(TARGET_THREE_PHASES)
        .expect("target should be written");

    let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "2"])
        .expect("delete command should succeed");
    let stdout = outcome.stdout.unwrap_or_default();
    assert_contains(&stdout, "## 2. Phase three");
    assert_contains(&stdout, "Requires 2.1.1.");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn dependency_rewrite_skips_dotted_versions_but_rewrites_later_anchors(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. First task.\n\n",
            "## 2. Phase two\n\n",
            "### 2.1. Step two\n\n",
            "- [ ] 2.1.1. Second task. Requires 1.0.0, 2.1.1, and 2.1.1.\n",
        ))
        .expect("target should be written");

    let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "1"])
        .expect("delete command should succeed");
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "Requires 1.0.0, 1.1.1, and 1.1.1.");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn replace_phase_with_multiple_phases(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(TARGET_TWO_PHASES)
        .expect("target should be written");
    test_workspace
        .write_fragment(REPLACEMENT_FRAGMENT)
        .expect("fragment should be written");

    let outcome = run_from_args([
        "mapsplice",
        "replace",
        test_workspace.target.as_str(),
        "2",
        test_workspace.fragment.as_str(),
    ])
    .expect("replace command should succeed");

    let stdout = outcome.stdout.unwrap_or_default();
    assert_contains(&stdout, "## 2. Replacement phase A");
    assert_contains(&stdout, "## 3. Replacement phase B");
    assert_contains(&stdout, "Requires 3.1.1.");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_mode_rewrites_target_and_emits_no_stdout(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(TARGET_TWO_PHASES)
        .expect("target should be written");

    let outcome = run_from_args([
        "mapsplice",
        "--in-place",
        "delete",
        test_workspace.target.as_str(),
        "1",
    ])
    .expect("in-place delete should succeed");

    assert_equal(&outcome.stdout, &None);
    assert_contains(&test_workspace.read_target()?, "## 1. Phase two");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn level_mismatch_is_rejected(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(TARGET_TWO_PHASES)
        .expect("target should be written");
    test_workspace
        .write_fragment(TASK_FRAGMENT)
        .expect("fragment should be written");

    let error = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "2",
        test_workspace.fragment.as_str(),
    ])
    .expect_err("mismatched fragment level must fail");

    assert_level_mismatch(&error);
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn missing_anchor_is_rejected(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(TARGET_TWO_PHASES)
        .expect("target should be written");

    let error = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "99"])
        .expect_err("missing anchor must fail");

    assert_anchor_not_found(&error);
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_after_can_default_from_environment(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let _after = EnvVarGuard::set("MAPSPLICE_CMDS_INSERT_AFTER", "true")
        .expect("environment guard should install");
    test_workspace
        .write_target(TARGET_TWO_TASKS)
        .expect("target should be written");
    test_workspace
        .write_fragment(TASK_FRAGMENT)
        .expect("fragment should be written");

    let outcome = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "1.1.1",
        test_workspace.fragment.as_str(),
    ])
    .expect("insert command should succeed with environment default");

    let stdout = outcome.stdout.unwrap_or_default();
    assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_after_can_default_from_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace
        .write_xdg_config("[cmds.insert]\nafter = true\n")
        .expect("config should be written");
    let _config =
        EnvVarGuard::set("XDG_CONFIG_HOME", xdg_home.as_str()).expect("xdg guard should install");
    test_workspace
        .write_target(TARGET_TWO_TASKS)
        .expect("target should be written");
    test_workspace
        .write_fragment(TASK_FRAGMENT)
        .expect("fragment should be written");

    let outcome = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "1.1.1",
        test_workspace.fragment.as_str(),
    ])
    .expect("insert command should succeed with config default");

    let stdout = outcome.stdout.unwrap_or_default();
    assert_contains(&stdout, "- [ ] 1.1.2. Inserted task. Requires 1.1.2.");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_can_default_from_environment(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let _in_place =
        EnvVarGuard::set("MAPSPLICE_IN_PLACE", "true").expect("environment guard should install");
    test_workspace
        .write_target(TARGET_TWO_PHASES)
        .expect("target should be written");

    let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "1"])
        .expect("delete command should succeed with in-place environment default");

    assert_equal(&outcome.stdout, &None);
    assert_contains(&test_workspace.read_target()?, "## 1. Phase two");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_can_default_from_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace
        .write_xdg_config("in_place = true\n")
        .expect("config should be written");
    let _config =
        EnvVarGuard::set("XDG_CONFIG_HOME", xdg_home.as_str()).expect("xdg guard should install");
    test_workspace
        .write_target(TARGET_TWO_PHASES)
        .expect("target should be written");

    let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "1"])
        .expect("delete command should succeed with in-place config default");

    assert_equal(&outcome.stdout, &None);
    assert_contains(&test_workspace.read_target()?, "## 1. Phase two");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_can_default_from_local_config_file(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_local_config("in_place = true\n")
        .expect("local config should be written");
    let _cwd = test_workspace
        .enter_root()
        .expect("current directory guard should install");
    test_workspace
        .write_target(TARGET_TWO_PHASES)
        .expect("target should be written");

    let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "1"])
        .expect("delete command should succeed with local in-place config default");

    assert_equal(&outcome.stdout, &None);
    assert_contains(&test_workspace.read_target()?, "## 1. Phase two");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn invalid_insert_config_surfaces_configuration_error(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    let xdg_home = test_workspace
        .write_xdg_config("[cmds.insert]\nafter = \"later\"\n")
        .expect("config should be written");
    let _config =
        EnvVarGuard::set("XDG_CONFIG_HOME", xdg_home.as_str()).expect("xdg guard should install");
    test_workspace
        .write_target(TARGET_TWO_TASKS)
        .expect("target should be written");
    test_workspace
        .write_fragment(TASK_FRAGMENT)
        .expect("fragment should be written");

    let error = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "1.1.1",
        test_workspace.fragment.as_str(),
    ])
    .expect_err("invalid config should fail");

    assert_configuration_error(&error);
    Ok(())
}
