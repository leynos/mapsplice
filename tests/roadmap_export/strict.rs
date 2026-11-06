//! Invalid export and source-location integration coverage.

use super::*;

#[rstest]
#[serial_test::serial(cli_env)]
fn invalid_committed_export_preserves_json_location(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(BASIC_ROADMAP)?;
    let export_path = write_committed_export(
        &test_workspace,
        "{\n  \"format_version\": 1,\n  \"unknown\": true\n}\n",
    )?;

    let error = run_from_args([
        "mapsplice",
        "export",
        "--check",
        export_path.as_str(),
        test_workspace.target.as_str(),
    ])
    .expect_err("schema-invalid JSON should be rejected");

    let diagnostic = error.to_string();
    require_contains(&diagnostic, "invalid export")?;
    require_contains(&diagnostic, "line 3 column")?;
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn invalid_committed_checkbox_state_is_rejected(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(BASIC_ROADMAP)?;
    let canonical = export(&test_workspace)?;
    let invalid = canonical.replace("\"state\": \"open\"", "\"state\": \"pending\"");
    require(canonical != invalid, "the state mutation should apply")?;
    let export_path = write_committed_export(&test_workspace, &invalid)?;

    let error = run_from_args([
        "mapsplice",
        "export",
        "--check",
        export_path.as_str(),
        test_workspace.target.as_str(),
    ])
    .expect_err("schema-invalid checkbox states must be rejected");

    let diagnostic = error.to_string();
    require_contains(&diagnostic, "invalid export")?;
    require_contains(&diagnostic, "line ")?;
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn export_ignores_in_place_even_for_a_direct_request(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(BASIC_ROADMAP)?;
    let original_target = test_workspace.dir.read_to_string("target.md")?;

    let outcome = run_request(CliRequest {
        global: GlobalOptions { in_place: true },
        target: test_workspace.target.clone(),
        command: CommandKind::Export {
            format: ExportFormat::Json,
            check: None,
        },
    })?;

    require(outcome.stdout.is_some(), "export should write to stdout")?;
    require_equal(&outcome.written_path, &None, "export written path")?;
    require_equal(
        &test_workspace.dir.read_to_string("target.md")?,
        &original_target,
        "preserved target",
    )?;
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn dependency_bullet_exports_its_supported_children(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(concat!(
        "## 1. Planning\n\n### 1.1. Export\n\n",
        "- [ ] 1.1.1. Emit JSON.\n",
        "  - Requires 1.1.2.\n\n",
        "    Success: retain this criterion.\n",
        "- [ ] 1.1.2. Check JSON.\n",
    ))?;

    let exported = export(&test_workspace)?;

    require_contains(&exported, "\"key\": \"Success\"")?;
    require_contains(&exported, "\"value\": \"retain this criterion.\"")?;
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn prose_after_a_requires_clause_is_exported(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(concat!(
        "## 1. Planning\n\n### 1.1. Export\n\n",
        "- [ ] 1.1.1. Emit JSON.\n",
        "  - Requires 1.1.2. Success: retain this criterion.\n",
        "- [ ] 1.1.2. Check JSON.\n",
    ))?;

    let exported = export(&test_workspace)?;

    require_contains(&exported, "\"key\": \"Success\"")?;
    require_contains(&exported, "\"value\": \"retain this criterion.\"")?;
    Ok(())
}

#[rstest]
#[case::unsupported_task_block(
    "## 1. Phase\n\n### 1.1. Step\n\n- [ ] 1.1.1. Task.\n\n  ```text\n  body\n  ```\n",
    "unsupported block in task body",
    "line 7, column 3"
)]
#[case::malformed_requires_anchor(
    "## 1. Phase\n\n### 1.1. Step\n\n- [ ] 1.1.1. Task. Requires 1.0.1.\n",
    "invalid dependency anchor token",
    "line 5, column 29"
)]
#[case::wrapped_requires_anchor(
    concat!(
        "## 1. Phase\n\n### 1.1. Step\n\n- [ ] 1.1.1. Task.\n\n",
        "  Requires 1.1.2\n  and 1.0.1.\n\n",
        "- [ ] 1.1.2. Other task.\n",
    ),
    "invalid dependency anchor token",
    "line 8, column 5"
)]
#[case::malformed_numbered_checkbox(
    "## 1. Phase\n\n### 1.1. Step\n\n- [ ] 1.1.1. Task.\n  - [ ] 1.1.1.0. Malformed detail.\n",
    "malformed numbered checkbox detail prefix",
    "line 6, column 9"
)]
#[case::out_of_order_numbered_checkbox(
    concat!(
        "## 1. Phase\n\n### 1.1. Step\n\n- [ ] 1.1.1. Task.\n",
        "  - Context.\n",
        "  - [ ] 1.1.1.2. Skipped ordinal.\n",
    ),
    "sub-task `1.1.1.2` is not in document order",
    "line 7, column 3"
)]
#[case::formatted_requires_keyword(
    "## 1. Phase\n\n### 1.1. Step\n\n- [ ] 1.1.1. Task. **Requires** 1.1.1.\n",
    "formatted `Requires` keyword",
    "line 5, column 20"
)]
#[case::missing_task_checkbox(
    "## 1. Phase\n\n### 1.1. Step\n\n- 1.1.1. Task without a checkbox.\n",
    "roadmap task lists must be unordered checklist items",
    "line 5, column 1"
)]
#[case::task_list_without_step(
    "## 1. Phase\n\n- [ ] 1.1.1. Task appears too early.\n",
    "task list appeared without a current step",
    "line 3, column 1"
)]
#[case::checkbox_beneath_dependency_bullet(
    concat!(
        "## 1. Phase\n\n### 1.1. Step\n\n- [ ] 1.1.1. Task.\n",
        "  - Requires 1.1.1.\n",
        "    - [ ] nested checkbox\n",
    ),
    "checkbox detail cannot be nested beneath a non-checkbox entry",
    "line 7, column 5"
)]
#[serial_test::serial(cli_env)]
fn malformed_or_unsupported_exports_fail_with_source_locations(
    workspace: TestResult<Workspace>,
    #[case] source: &str,
    #[case] expected_message: &str,
    #[case] expected_location: &str,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace.write_target(source)?;

    let error = run_from_args(["mapsplice", "export", test_workspace.target.as_str()])
        .expect_err("invalid export structure should fail");
    let diagnostic = error.to_string();

    require_contains(&diagnostic, expected_message)?;
    require_contains(&diagnostic, expected_location)?;
    Ok(())
}
