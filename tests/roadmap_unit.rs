//! `rstest` coverage for roadmap parsing and splice semantics.

mod support;

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
#[case("8.2.3.4")]
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

fn workspace_fixture(workspace: TestResult<Workspace>) -> Workspace {
    match workspace {
        Ok(test_workspace) => test_workspace,
        Err(error) => panic!("workspace fixture should initialize: {error}"),
    }
}

#[rstest]
#[serial_test::serial(cli_env)]
fn append_emits_stdout_and_keeps_target_unchanged(workspace: TestResult<Workspace>) {
    let test_workspace = workspace_fixture(workspace);
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
    assert_eq!(outcome.stdout.as_deref(), Some(expected));
    assert_eq!(
        test_workspace
            .read_target()
            .expect("target should still be readable"),
        TARGET_TWO_PHASES
    );
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_before_phase_renumbers_later_phases_and_dependencies(workspace: TestResult<Workspace>) {
    let test_workspace = workspace_fixture(workspace);
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

    assert!(
        outcome
            .stdout
            .as_deref()
            .is_some_and(|stdout| stdout.contains("## 3. Phase two"))
    );
    assert!(
        outcome
            .stdout
            .as_deref()
            .is_some_and(|stdout| stdout.contains("Requires 3.1.1."))
    );
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_after_task_renumbers_later_tasks_within_the_step(workspace: TestResult<Workspace>) {
    let test_workspace = workspace_fixture(workspace);
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
    assert!(stdout.contains("- [ ] 1.1.2. Inserted task. Requires 1.1.2."));
    assert!(stdout.contains("- [ ] 1.1.3. Second task. Depends on 1.1.1 and 1.1.2."));
}

#[rstest]
#[serial_test::serial(cli_env)]
fn dangling_dependency_is_rejected(workspace: TestResult<Workspace>) {
    let test_workspace = workspace_fixture(workspace);
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

    assert!(matches!(error, MapspliceError::DanglingDependency { .. }));
}

#[rstest]
#[serial_test::serial(cli_env)]
fn delete_phase_rewrites_downstream_identifiers(workspace: TestResult<Workspace>) {
    let test_workspace = workspace_fixture(workspace);
    test_workspace
        .write_target(TARGET_THREE_PHASES)
        .expect("target should be written");

    let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "2"])
        .expect("delete command should succeed");
    let stdout = outcome.stdout.unwrap_or_default();
    assert!(stdout.contains("## 2. Phase three"));
    assert!(stdout.contains("Requires 2.1.1."));
}

#[rstest]
#[serial_test::serial(cli_env)]
fn replace_phase_with_multiple_phases(workspace: TestResult<Workspace>) {
    let test_workspace = workspace_fixture(workspace);
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
    assert!(stdout.contains("## 2. Replacement phase A"));
    assert!(stdout.contains("## 3. Replacement phase B"));
    assert!(stdout.contains("Requires 3.1.1."));
}

#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_mode_rewrites_target_and_emits_no_stdout(workspace: TestResult<Workspace>) {
    let test_workspace = workspace_fixture(workspace);
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

    assert_eq!(outcome.stdout, None);
    assert!(
        test_workspace
            .read_target()
            .expect("target should still be readable")
            .contains("## 1. Phase two")
    );
}

#[rstest]
#[serial_test::serial(cli_env)]
fn level_mismatch_is_rejected(workspace: TestResult<Workspace>) {
    let test_workspace = workspace_fixture(workspace);
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

    assert!(matches!(error, MapspliceError::LevelMismatch { .. }));
}

#[rstest]
#[serial_test::serial(cli_env)]
fn missing_anchor_is_rejected(workspace: TestResult<Workspace>) {
    let test_workspace = workspace_fixture(workspace);
    test_workspace
        .write_target(TARGET_TWO_PHASES)
        .expect("target should be written");

    let error = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "99"])
        .expect_err("missing anchor must fail");

    assert!(matches!(error, MapspliceError::AnchorNotFound { .. }));
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_after_can_default_from_environment(workspace: TestResult<Workspace>) {
    let test_workspace = workspace_fixture(workspace);
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
    assert!(stdout.contains("- [ ] 1.1.2. Inserted task. Requires 1.1.2."));
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_after_can_default_from_config_file(workspace: TestResult<Workspace>) {
    let test_workspace = workspace_fixture(workspace);
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
    assert!(stdout.contains("- [ ] 1.1.2. Inserted task. Requires 1.1.2."));
}

#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_can_default_from_environment(workspace: TestResult<Workspace>) {
    let test_workspace = workspace_fixture(workspace);
    let _in_place =
        EnvVarGuard::set("MAPSPLICE_IN_PLACE", "true").expect("environment guard should install");
    test_workspace
        .write_target(TARGET_TWO_PHASES)
        .expect("target should be written");

    let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "1"])
        .expect("delete command should succeed with in-place environment default");

    assert_eq!(outcome.stdout, None);
    assert!(
        test_workspace
            .read_target()
            .expect("target should still be readable")
            .contains("## 1. Phase two")
    );
}

#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_can_default_from_config_file(workspace: TestResult<Workspace>) {
    let test_workspace = workspace_fixture(workspace);
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

    assert_eq!(outcome.stdout, None);
    assert!(
        test_workspace
            .read_target()
            .expect("target should still be readable")
            .contains("## 1. Phase two")
    );
}

#[rstest]
#[serial_test::serial(cli_env)]
fn in_place_can_default_from_local_config_file(workspace: TestResult<Workspace>) {
    let test_workspace = workspace_fixture(workspace);
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

    assert_eq!(outcome.stdout, None);
    assert!(
        test_workspace
            .read_target()
            .expect("target should still be readable")
            .contains("## 1. Phase two")
    );
}

#[rstest]
#[serial_test::serial(cli_env)]
fn invalid_insert_config_surfaces_configuration_error(workspace: TestResult<Workspace>) {
    let test_workspace = workspace_fixture(workspace);
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

    assert!(matches!(error, MapspliceError::Configuration { .. }));
}
