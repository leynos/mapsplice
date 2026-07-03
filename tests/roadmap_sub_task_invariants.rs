//! Integration tests for ordered sub-task mutation invariants.

#[path = "support/roadmap_workspace.rs"]
mod workspace_support;

use mapsplice::run_from_args;
use rstest::rstest;
use workspace_support::{TestResult, Workspace, workspace};

const TARGET_WITH_INTERLEAVED_SUB_TASK_BODY: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. Parent task. Requires 1.1.1.2.\n",
    "  Body before sub-tasks.\n\n",
    "  - [ ] 1.1.1.1. First sub-task. Requires 1.1.1.\n",
    "\n",
    "  Body between sub-tasks.\n\n",
    "  - [ ] 1.1.1.2. Second sub-task. Requires 1.1.1.1.\n",
    "\n",
    "  Body after sub-tasks.\n",
    "- [ ] 1.1.2. Sibling task. Requires 1.1.1.2.\n",
);

const INSERTED_SUB_TASK_FRAGMENT: &str = "  - [x] 1.1.1.1. Inserted sub-task.\n";

const REPLACEMENT_SUB_TASK_FRAGMENT: &str = concat!(
    "  - [x] 1.1.1.1. Replacement sub-task A.\n",
    "  - [ ] 1.1.1.2. Replacement sub-task B.\n",
);

fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "missing `{needle}` in:\n{haystack}"
    );
}

fn assert_in_order(haystack: &str, needles: &[&str]) {
    let mut remainder = haystack;
    for needle in needles {
        let Some((_, tail)) = remainder.split_once(needle) else {
            panic!("missing `{needle}` in:\n{haystack}");
        };
        remainder = tail;
    }
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_sub_task_preserves_interleaved_body_order(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(TARGET_WITH_INTERLEAVED_SUB_TASK_BODY)
        .expect("target should be written");
    test_workspace
        .write_fragment(INSERTED_SUB_TASK_FRAGMENT)
        .expect("fragment should be written");

    let outcome = run_from_args([
        "mapsplice",
        "insert",
        test_workspace.target.as_str(),
        "1.1.1.1",
        test_workspace.fragment.as_str(),
    ])
    .expect("sub-task insert should succeed");
    let stdout = outcome.stdout.unwrap_or_default();

    assert_in_order(
        &stdout,
        &[
            "Body before sub-tasks.",
            "1.1.1.1. Inserted sub-task.",
            "1.1.1.2. First sub-task. Requires 1.1.1.",
            "Body between sub-tasks.",
            "1.1.1.3. Second sub-task. Requires 1.1.1.2.",
            "Body after sub-tasks.",
        ],
    );
    assert_contains(&stdout, "Parent task. Requires 1.1.1.3.");
    assert_contains(&stdout, "Sibling task. Requires 1.1.1.3.");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn delete_sub_task_preserves_interleaved_body_order(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    let target = TARGET_WITH_INTERLEAVED_SUB_TASK_BODY
        .replace("Second sub-task. Requires 1.1.1.1.", "Second sub-task.");
    test_workspace
        .write_target(&target)
        .expect("target should be written");

    let outcome = run_from_args([
        "mapsplice",
        "delete",
        test_workspace.target.as_str(),
        "1.1.1.1",
    ])
    .expect("sub-task delete should succeed");
    let stdout = outcome.stdout.unwrap_or_default();

    assert_in_order(
        &stdout,
        &[
            "Body before sub-tasks.",
            "Body between sub-tasks.",
            "1.1.1.1. Second sub-task.",
            "Body after sub-tasks.",
        ],
    );
    assert_contains(&stdout, "Parent task. Requires 1.1.1.1.");
    assert_contains(&stdout, "Sibling task. Requires 1.1.1.1.");
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn replace_sub_task_preserves_interleaved_body_order(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(TARGET_WITH_INTERLEAVED_SUB_TASK_BODY)
        .expect("target should be written");
    test_workspace
        .write_fragment(REPLACEMENT_SUB_TASK_FRAGMENT)
        .expect("fragment should be written");

    let outcome = run_from_args([
        "mapsplice",
        "replace",
        test_workspace.target.as_str(),
        "1.1.1.2",
        test_workspace.fragment.as_str(),
    ])
    .expect("sub-task replace should succeed");
    let stdout = outcome.stdout.unwrap_or_default();

    assert_in_order(
        &stdout,
        &[
            "Body before sub-tasks.",
            "1.1.1.1. First sub-task. Requires 1.1.1.",
            "Body between sub-tasks.",
            "1.1.1.2. Replacement sub-task A.",
            "1.1.1.3. Replacement sub-task B.",
            "Body after sub-tasks.",
        ],
    );
    assert_contains(&stdout, "Parent task. Requires 1.1.1.3.");
    assert_contains(&stdout, "Sibling task. Requires 1.1.1.3.");
    Ok(())
}
