//! `rstest` coverage for roadmap rendering preservation behaviour.

#[path = "support/phase.rs"]
mod phase_support;
#[path = "support/roadmap_workspace.rs"]
mod workspace_support;

use std::io;

use mapsplice::run_from_args;
use phase_support::PHASE_FRAGMENT;
use rstest::rstest;
use workspace_support::{TestResult, Workspace, workspace};

fn assert_contains(haystack: &str, needle: &str) {
    assert!(haystack.contains(needle));
}

fn assert_ordered(haystack: &str, first: &str, second: &str, third: &str) -> TestResult {
    let first_index = marker_index(haystack, first)?;
    let second_index = marker_index(haystack, second)?;
    let third_index = marker_index(haystack, third)?;
    if first_index >= second_index || second_index >= third_index {
        return Err(io::Error::other("markers should appear in order").into());
    }
    Ok(())
}

fn marker_index(haystack: &str, marker: &str) -> TestResult<usize> {
    haystack
        .find(marker)
        .ok_or_else(|| io::Error::other(format!("marker `{marker}` should exist")).into())
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
fn render_preserves_untouched_body_markdown_exactly(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "| Name  | Value |\n",
            "| :---- | ----: |\n",
            "| alpha |   10  |\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. First task.\n\n",
            "## 2. Phase two\n\n",
            "### 2.1. Step two\n\n",
            "- [ ] 2.1.1. Second task.\n",
        ))
        .expect("target should be written");

    let outcome = run_from_args(["mapsplice", "delete", test_workspace.target.as_str(), "2"])
        .expect("delete command should succeed");
    let stdout = outcome.stdout.unwrap_or_default();

    assert_contains(
        &stdout,
        "| Name  | Value |\n| :---- | ----: |\n| alpha |   10  |",
    );
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn render_preserves_task_body_and_sub_task_order(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. Parent task.\n",
            "    Body before.\n",
            "    - [ ] 1.1.1.1. Nested sub-task.\n",
            "    Body after.\n",
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

    assert_ordered(
        &stdout,
        "Body before.",
        "- [ ] 1.1.1.1. Nested sub-task.",
        "Body after.",
    )?;
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn render_preserves_nested_sub_task_block_exactly(workspace: TestResult<Workspace>) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. Parent task.\n",
            "    Body before.\n",
            "    - [ ] 1.1.1.1. Nested sub-task.\n",
            "    Body after.\n",
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

    assert_contains(
        &stdout,
        concat!(
            "- [ ] 1.1.1. Parent task.\n",
            "    Body before.\n",
            "    - [ ] 1.1.1.1. Nested sub-task.\n",
            "    Body after.",
        ),
    );
    Ok(())
}
