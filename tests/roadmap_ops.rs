//! `rstest` coverage for roadmap splice operations and CLI config merging.

#[path = "support/ops.rs"]
mod support;

use std::fmt::Debug;

use mapsplice::{
    MapspliceError,
    RoadmapOperation,
    apply_command,
    metrics_snapshot,
    parse_fragment,
    parse_roadmap,
    run_from_args,
};
use rstest::rstest;
use support::{
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

fn assert_contains(haystack: &str, needle: &str) {
    assert!(haystack.contains(needle));
}

fn assert_equal<T>(actual: &T, expected: &T)
where
    T: Debug + PartialEq,
{
    assert_eq!(actual, expected);
}

fn assert_level_mismatch(error: &MapspliceError) {
    assert!(matches!(error, MapspliceError::LevelMismatch { .. }));
}

fn assert_anchor_not_found(error: &MapspliceError) {
    assert!(matches!(error, MapspliceError::AnchorNotFound { .. }));
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
        "- [ ] 3.1.1. Added task. Requires 3.1.1.\n",
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
fn unresolved_dependency_reference_is_reported(workspace: TestResult<Workspace>) -> TestResult {
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
    .expect_err("unresolved valid dependency references must fail");

    let MapspliceError::DanglingDependency { anchor } = error else {
        return Err(format!("expected dangling dependency error, got {error:?}").into());
    };
    assert_equal(&anchor.to_string(), &"99.1.1".to_owned());
    Ok(())
}

#[rstest]
fn apply_command_leaves_roadmap_unchanged_on_error() -> TestResult {
    let mut roadmap = parse_roadmap(concat!(
        "# Example\n\n",
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. First task. Requires 99.1.1.\n",
    ))?;
    let original = roadmap.clone();
    let fragment = parse_fragment(PHASE_FRAGMENT)?;

    let result = apply_command(&mut roadmap, RoadmapOperation::Append, Some(fragment));

    if !matches!(result, Err(MapspliceError::DanglingDependency { .. })) {
        return Err(format!("expected dangling dependency error, got {result:?}").into());
    }
    assert_equal(&roadmap, &original);
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
fn help_and_version_do_not_record_failures() {
    let before = metrics_snapshot();

    let help_error = run_from_args(["mapsplice", "--help"])
        .expect_err("help should be surfaced as clap display");
    let version_error = run_from_args(["mapsplice", "--version"])
        .expect_err("version should be surfaced as clap display");
    let after = metrics_snapshot();

    assert!(matches!(help_error, MapspliceError::Clap(_)));
    assert!(matches!(version_error, MapspliceError::Clap(_)));
    assert_equal(&after.failures, &before.failures);
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
fn dependency_reference_delete_preserves_incidental_numbers_and_rewrites_requires(
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
            "- [ ] 2.1.1. Second task. See §2.1. Released 1.4.0. Count 27. Requires 2.1.1, \
             2.1.1.\n",
        ))
        .expect("target should be written");

    let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "1"])
        .expect("delete command should succeed");
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(
        &stdout,
        "- [ ] 1.1.1. Second task. See §2.1. Released 1.4.0. Count 27. Requires 1.1.1, 1.1.1.",
    );
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn dependency_rewrite_ignores_blocks_clauses(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. First task.\n\n",
            "## 2. Phase two\n\n",
            "### 2.1. Step two\n\n",
            "- [ ] 2.1.1. Second task. Blocks 2.1.1.\n",
        ))
        .expect("target should be written");

    let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "1"])
        .expect("delete command should succeed");
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(&stdout, "Blocks 2.1.1.");
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
