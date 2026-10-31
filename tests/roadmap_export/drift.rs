//! Semantic export drift and dependency parity integration coverage.

use super::*;

#[rstest]
#[case::task_title("Emit **stable** JSON", "Emit changed JSON", "~ task 1.1.1 title")]
#[case::dependency("Requires 1.1.2.", "Requires 1.1.1.", "~ task 1.1.1 requires")]
#[case::checkbox("- [ ] 1.1.1.", "- [x] 1.1.1.", "~ task 1.1.1 state")]
#[serial_test::serial(cli_env)]
fn export_check_names_semantic_drift_and_preserves_both_files(
    workspace: TestResult<Workspace>,
    #[case] source: &str,
    #[case] replacement: &str,
    #[case] expected_difference: &str,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(BASIC_ROADMAP)?;
    let committed = export(&test_workspace)?;
    let export_path = write_committed_export(&test_workspace, &committed)?;
    let original_target = test_workspace.dir.read_to_string("target.md")?;
    let original_export = test_workspace.dir.read_to_string("roadmap.json")?;
    let changed = original_target.replacen(source, replacement, 1);
    require(changed != original_target, "test mutation should apply")?;
    test_workspace.write_target(&changed)?;
    let changed_target = test_workspace.dir.read_to_string("target.md")?;

    let error = run_from_args([
        "mapsplice",
        "export",
        "--check",
        export_path.as_str(),
        test_workspace.target.as_str(),
    ])
    .expect_err("semantic drift should fail the check");

    require_contains(&error.to_string(), expected_difference)?;
    require_equal(
        &test_workspace.dir.read_to_string("target.md")?,
        &changed_target,
        "preserved target",
    )?;
    require_equal(
        &test_workspace.dir.read_to_string("roadmap.json")?,
        &original_export,
        "preserved export",
    )?;
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn nested_detail_drift_names_its_owning_task(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(BASIC_ROADMAP)?;
    let committed = export(&test_workspace)?;
    let export_path = write_committed_export(&test_workspace, &committed)?;
    let original = test_workspace.dir.read_to_string("target.md")?;
    let changed = original.replacen(
        "Compare the export without writing files.",
        "Compare the export without rewriting files.",
        1,
    );
    require(changed != original, "the detail mutation should apply")?;
    test_workspace.write_target(&changed)?;

    let error = run_from_args([
        "mapsplice",
        "export",
        "--check",
        export_path.as_str(),
        test_workspace.target.as_str(),
    ])
    .expect_err("a changed detail title is semantic drift");

    require_contains(&error.to_string(), "~ detail 1 of task 1.1.1 title")?;
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn export_check_succeeds_silently_and_does_not_rewrite_files(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(BASIC_ROADMAP)?;
    let original_target = test_workspace.dir.read_to_string("target.md")?;
    let json = export(&test_workspace)?;
    let export_path = write_committed_export(&test_workspace, &json)?;

    let outcome = run_from_args([
        "mapsplice",
        "export",
        "--check",
        export_path.as_str(),
        test_workspace.target.as_str(),
    ])?;

    require_equal(&outcome.stdout, &None, "check stdout")?;
    require_equal(&outcome.written_path, &None, "check written path")?;
    require_equal(
        &test_workspace.dir.read_to_string("target.md")?,
        &original_target,
        "preserved target",
    )?;
    require_equal(
        &test_workspace.dir.read_to_string("roadmap.json")?,
        &json,
        "preserved export",
    )?;
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn export_check_reports_formatting_only_drift(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(BASIC_ROADMAP)?;
    let canonical = export(&test_workspace)?;
    let value = serde_json::from_str::<serde_json::Value>(&canonical)?;
    let compact = serde_json::to_string(&value)?;
    let export_path = write_committed_export(&test_workspace, &compact)?;

    let error = run_from_args([
        "mapsplice",
        "export",
        "--check",
        export_path.as_str(),
        test_workspace.target.as_str(),
    ])
    .expect_err("different bytes should be reported even when the schema matches");

    require_contains(&error.to_string(), "formatting-only drift")?;
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn export_check_reports_semantic_source_order_drift(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(BASIC_ROADMAP)?;
    let canonical = export(&test_workspace)?;
    let mut document: serde_json::Value = serde_json::from_str(&canonical)?;
    let tasks = task_array_mut(&mut document)?;
    let [first, second, ..] = tasks.as_mut_slice() else {
        return Err("the export should have at least two tasks".into());
    };
    std::mem::swap(first, second);
    let reordered = format!("{}\n", serde_json::to_string_pretty(&document)?);
    let export_path = write_committed_export(&test_workspace, &reordered)?;

    let error = run_from_args([
        "mapsplice",
        "export",
        "--check",
        export_path.as_str(),
        test_workspace.target.as_str(),
    ])
    .expect_err("source-order changes are semantic drift");

    require_contains(&error.to_string(), "step 1.1 tasks order")?;
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn nested_requires_export_matches_the_rewriter_after_renumbering(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(NESTED_REQUIRES_ROADMAP)?;
    test_workspace.write_fragment(INSERTED_TASK)?;

    let outcome = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "1.1.1",
        test_workspace.fragment.as_str(),
    ])?;
    let rewritten = outcome.stdout.ok_or("insert should render the roadmap")?;
    require_contains(&rewritten, "- Requires 1.1.2.")?;
    test_workspace.write_target(&rewritten)?;

    let document: serde_json::Value = serde_json::from_str(&export(&test_workspace)?)?;
    require_equal(
        task_requires(&document, 2)?,
        &serde_json::json!(["1.1.2"]),
        "nested Requires export",
    )?;
    Ok(())
}
