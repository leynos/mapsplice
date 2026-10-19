//! Mixed-list mutation cases for ordinary body bullets and addendum sub-tasks.

use super::*;

fn assert_stdout(stdout: &str, expected: &str) -> TestResult {
    if stdout != expected {
        return Err(format!(
            "unexpected rendered roadmap\nexpected:\n{expected}\nactual:\n{stdout}"
        )
        .into());
    }
    Ok(())
}

#[rstest]
#[case::body_before(
    concat!(
        "  - A non-structural body bullet.\n",
        "  - [ ] 1.1.1.1. First sub-task.\n",
        "  - [ ] 1.1.1.2. Second sub-task. Requires 1.1.1.1.\n",
    ),
    concat!(
        "  - A non-structural body bullet.\n",
        "  - [ ] 1.1.2.1. First sub-task.\n",
        "  - [ ] 1.1.2.2. Second sub-task. Requires 1.1.2.1.\n",
    ),
)]
#[case::body_between(
    concat!(
        "  - [ ] 1.1.1.1. First sub-task.\n",
        "  - A non-structural body bullet.\n",
        "  - [ ] 1.1.1.2. Second sub-task. Requires 1.1.1.1.\n",
    ),
    concat!(
        "  - [ ] 1.1.2.1. First sub-task.\n",
        "  - A non-structural body bullet.\n",
        "  - [ ] 1.1.2.2. Second sub-task. Requires 1.1.2.1.\n",
    ),
)]
#[case::body_after(
    concat!(
        "  - [ ] 1.1.1.1. First sub-task.\n",
        "  - [ ] 1.1.1.2. Second sub-task. Requires 1.1.1.1.\n",
        "  - A non-structural body bullet.\n",
    ),
    concat!(
        "  - [ ] 1.1.2.1. First sub-task.\n",
        "  - [ ] 1.1.2.2. Second sub-task. Requires 1.1.2.1.\n",
        "  - A non-structural body bullet.\n",
    ),
)]
#[serial_test::serial(cli_env)]
fn insert_task_before_mixed_nested_body_renumbers_parent_and_siblings(
    workspace: TestResult<Workspace>,
    #[case] nested_items: &str,
    #[case] expected_nested_items: &str,
) -> TestResult {
    let test_workspace = workspace?;
    let target = format!(
        concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. Parent task. Requires 1.1.1.2.\n",
            "{}",
            "- [ ] 1.1.2. Sibling task. Requires 1.1.1.2.\n",
        ),
        nested_items
    );
    test_workspace
        .write_target(&target)
        .expect("target should be written");
    test_workspace
        .write_fragment("- [ ] 1.1.1. Inserted task.\n")
        .expect("task fragment should be written");

    let outcome = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "1.1.1",
        test_workspace.fragment.as_str(),
    ])
    .expect("task insertion should succeed");
    let stdout = outcome.stdout.unwrap_or_default();
    let expected = format!(
        concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. Inserted task.\n",
            "- [ ] 1.1.2. Parent task. Requires 1.1.2.2.\n",
            "{}",
            "- [ ] 1.1.3. Sibling task. Requires 1.1.2.2.\n",
        ),
        expected_nested_items
    );

    assert_stdout(&stdout, &expected)
}

#[rstest]
#[case::body_before(
    concat!(
        "  - A non-structural body bullet.\n",
        "  - [ ] 1.1.1.1. First sub-task.\n",
        "  - [ ] 1.1.1.2. Second sub-task.\n",
    ),
    concat!(
        "  - A non-structural body bullet.\n",
        "  - [ ] 1.1.1.1. Second sub-task.\n",
    ),
)]
#[case::body_between(
    concat!(
        "  - [ ] 1.1.1.1. First sub-task.\n",
        "  - A non-structural body bullet.\n",
        "  - [ ] 1.1.1.2. Second sub-task.\n",
    ),
    concat!(
        "  - A non-structural body bullet.\n",
        "  - [ ] 1.1.1.1. Second sub-task.\n",
    ),
)]
#[case::body_after(
    concat!(
        "  - [ ] 1.1.1.1. First sub-task.\n",
        "  - [ ] 1.1.1.2. Second sub-task.\n",
        "  - A non-structural body bullet.\n",
    ),
    concat!(
        "  - [ ] 1.1.1.1. Second sub-task.\n",
        "  - A non-structural body bullet.\n",
    ),
)]
#[serial_test::serial(cli_env)]
fn delete_sub_task_from_mixed_nested_body_preserves_order_and_rewrites_dependencies(
    workspace: TestResult<Workspace>,
    #[case] nested_items: &str,
    #[case] expected_nested_items: &str,
) -> TestResult {
    let test_workspace = workspace?;
    let target = format!(
        concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. Parent task. Requires 1.1.1.2.\n",
            "{}",
            "- [ ] 1.1.2. Sibling task. Requires 1.1.1.2.\n",
        ),
        nested_items
    );
    test_workspace
        .write_target(&target)
        .expect("target should be written");

    let outcome = run_from_args([
        "mapsplice",
        "delete",
        test_workspace.target.as_str(),
        "1.1.1.1",
    ])
    .expect("sub-task deletion should succeed");
    let stdout = outcome.stdout.unwrap_or_default();
    let expected = format!(
        concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. Parent task. Requires 1.1.1.1.\n",
            "{}",
            "- [ ] 1.1.2. Sibling task. Requires 1.1.1.1.\n",
        ),
        expected_nested_items
    );

    assert_stdout(&stdout, &expected)
}

#[rstest]
#[case::joins_body_runs(
    concat!(
        "- [ ] 1.1.1. Parent task.\n",
        "  - Body item before the addendum.\n",
        "  - [ ] 1.1.1.1. Addendum to remove.\n",
        "  - Body item after the addendum.\n",
    ),
    concat!(
        "- [ ] 1.1.1. Parent task.\n\n",
        "  - Body item before the addendum.\n",
        "  - Body item after the addendum.\n",
    ),
)]
#[case::paragraph_separator(
    concat!(
        "- [ ] 1.1.1. Parent task.\n\n",
        "  Paragraph body.\n\n",
        "  - [ ] 1.1.1.1. Addendum to remove.\n",
        "  - Ordinary body list item.\n",
        "  - [ ] 1.1.1.2. Later addendum.\n",
    ),
    concat!(
        "- [ ] 1.1.1. Parent task.\n\n",
        "  Paragraph body.\n\n",
        "  - Ordinary body list item.\n",
        "  - [ ] 1.1.1.1. Later addendum.\n",
    ),
)]
#[serial_test::serial(cli_env)]
fn delete_sub_task_renders_mixed_body_separators(
    workspace: TestResult<Workspace>,
    #[case] task: &str,
    #[case] expected_task: &str,
) -> TestResult {
    const HEADER: &str = concat!(
        "# Example\n\n",
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
    );
    let test_workspace = workspace?;
    test_workspace.write_target(&format!("{HEADER}{task}"))?;

    let outcome = run_from_args([
        "mapsplice",
        "delete",
        test_workspace.target.as_str(),
        "1.1.1.1",
    ])?;
    assert_stdout(
        &outcome.stdout.unwrap_or_default(),
        &format!("{HEADER}{expected_task}"),
    )
}

#[rstest]
#[case::body_before(
    concat!(
        "  - A non-structural body bullet.\n",
        "  - [ ] 1.1.1.1. First sub-task.\n",
        "  - [ ] 1.1.1.2. Second sub-task. Requires 1.1.1.1.\n",
    ),
    concat!(
        "  - A non-structural body bullet.\n",
        "  - [ ] 1.1.1.1. Inserted sub-task.\n",
        "  - [ ] 1.1.1.2. First sub-task.\n",
        "  - [ ] 1.1.1.3. Second sub-task. Requires 1.1.1.2.\n",
    ),
)]
#[case::body_between(
    concat!(
        "  - [ ] 1.1.1.1. First sub-task.\n",
        "  - A non-structural body bullet.\n",
        "  - [ ] 1.1.1.2. Second sub-task. Requires 1.1.1.1.\n",
    ),
    concat!(
        "  - [ ] 1.1.1.1. Inserted sub-task.\n",
        "  - [ ] 1.1.1.2. First sub-task.\n",
        "  - A non-structural body bullet.\n",
        "  - [ ] 1.1.1.3. Second sub-task. Requires 1.1.1.2.\n",
    ),
)]
#[case::body_after(
    concat!(
        "  - [ ] 1.1.1.1. First sub-task.\n",
        "  - [ ] 1.1.1.2. Second sub-task. Requires 1.1.1.1.\n",
        "  - A non-structural body bullet.\n",
    ),
    concat!(
        "  - [ ] 1.1.1.1. Inserted sub-task.\n",
        "  - [ ] 1.1.1.2. First sub-task.\n",
        "  - [ ] 1.1.1.3. Second sub-task. Requires 1.1.1.2.\n",
        "  - A non-structural body bullet.\n",
    ),
)]
#[serial_test::serial(cli_env)]
fn insert_sub_task_before_mixed_nested_body_preserves_order_and_rewrites_dependencies(
    workspace: TestResult<Workspace>,
    #[case] nested_items: &str,
    #[case] expected_nested_items: &str,
) -> TestResult {
    let test_workspace = workspace?;
    let target = format!(
        concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. Parent task. Requires 1.1.1.2.\n",
            "{}",
            "- [ ] 1.1.2. Sibling task. Requires 1.1.1.2.\n",
        ),
        nested_items
    );
    test_workspace
        .write_target(&target)
        .expect("target should be written");
    test_workspace
        .write_fragment("  - [ ] 1.1.1.1. Inserted sub-task.\n")
        .expect("sub-task fragment should be written");

    let outcome = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "1.1.1.1",
        test_workspace.fragment.as_str(),
    ])
    .expect("sub-task insertion should succeed");
    let stdout = outcome.stdout.unwrap_or_default();
    let expected = format!(
        concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. Parent task. Requires 1.1.1.3.\n",
            "{}",
            "- [ ] 1.1.2. Sibling task. Requires 1.1.1.3.\n",
        ),
        expected_nested_items
    );

    assert_stdout(&stdout, &expected)
}
