//! Regression tests for append preserving unchanged roadmap source.

#[path = "support/assertions.rs"]
mod assertions;
#[path = "support/workspace.rs"]
mod support;

use assertions::assert_contains;
use mapsplice::run_from_args;
use rstest::rstest;
use support::{TestResult, create_workspace};

const PHASE_FRAGMENT: &str = concat!(
    "## 9. Inserted phase\n\n",
    "### 9.1. Added step\n\n",
    "- [ ] 9.1.1. Added task. Requires 9.1.1.\n",
);

const TWO_SPACE_TARGET: &str = concat!(
    "# Roadmap\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [x] 1.1.1. Untouched task wraps across this source-authored line\n",
    "  without changing its continuation indentation.\n",
    "- [x] 1.1.2. Edited task wraps across this source-authored line\n",
    "  before the requested operation changes it.\n",
);

const UNTOUCHED_TWO_SPACE_TASK: &str = concat!(
    "- [x] 1.1.1. Untouched task wraps across this source-authored line\n",
    "  without changing its continuation indentation.",
);

const TWO_SPACE_FRAGMENT: &str = concat!(
    "- [ ] 9.9.9. New task wraps across a canonical continuation line\n",
    "  before it is inserted.\n",
);

const INDENTED_TASK_TARGET: &str = concat!(
    "# Roadmap\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "  - [x] 1.1.1. Surviving task.\n",
    "  - [x] 1.1.2. Deleted task.\n",
);

const INDENTED_TASK_DELETE_OUTPUT: &str = concat!(
    "# Roadmap\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "  - [x] 1.1.1. Surviving task.\n",
);

const CR_ONLY_TASK_TARGET: &str = concat!(
    "# Roadmap\r\r",
    "## 1. Phase one\r\r",
    "### 1.1. Step one\r\r",
    "- [x] 1.1.1. Surviving task.\r",
    "- [x] 1.1.2. Deleted task.\r",
);

const CR_ONLY_TASK_DELETE_OUTPUT: &str = concat!(
    "# Roadmap\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [x] 1.1.1. Surviving task.\r\n",
);

#[rstest]
#[serial_test::serial(cli_env)]
fn append_preserves_existing_loose_task_spacing() -> TestResult {
    let test_workspace = create_workspace()?;
    let preserved_phase = concat!(
        "## 1. Existing phase\n\n",
        "### 1.1. Existing step\n\n",
        "- [ ] 1.1.1. First existing task.\n",
        "\n",
        "  - Supporting note stays attached.\n",
        "\n",
        "- [ ] 1.1.2. Second existing task.\n",
    );
    test_workspace
        .write_target(&format!("# Example\n\n{preserved_phase}"))
        .expect("target should be written");
    test_workspace
        .write_fragment(PHASE_FRAGMENT)
        .expect("fragment should be written");

    run_from_args([
        "mapsplice",
        "--in-place",
        "append",
        test_workspace.target.as_str(),
        test_workspace.fragment.as_str(),
    ])
    .expect("in-place append command should succeed");

    assert_contains(
        &test_workspace.dir.read_to_string("target.md")?,
        preserved_phase,
    );
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn insert_preserves_untouched_two_space_task_source() -> TestResult {
    let test_workspace = create_workspace()?;
    test_workspace.write_target(TWO_SPACE_TARGET)?;
    test_workspace.write_fragment(TWO_SPACE_FRAGMENT)?;

    let output = run_from_args([
        "mapsplice",
        "insert",
        "--after",
        test_workspace.target.as_str(),
        "1.1.2",
        test_workspace.fragment.as_str(),
    ])?
    .stdout
    .unwrap_or_default();

    assert_contains(&output, UNTOUCHED_TWO_SPACE_TASK);
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn delete_preserves_surviving_two_space_task_source() -> TestResult {
    let test_workspace = create_workspace()?;
    test_workspace.write_target(TWO_SPACE_TARGET)?;

    let output = run_from_args([
        "mapsplice",
        "delete",
        test_workspace.target.as_str(),
        "1.1.2",
    ])?
    .stdout
    .unwrap_or_default();

    assert_contains(&output, UNTOUCHED_TWO_SPACE_TASK);
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn delete_does_not_leave_next_task_indentation_behind() -> TestResult {
    let test_workspace = create_workspace()?;
    test_workspace.write_target(INDENTED_TASK_TARGET)?;

    let output = run_from_args([
        "mapsplice",
        "delete",
        test_workspace.target.as_str(),
        "1.1.2",
    ])?
    .stdout
    .unwrap_or_default();

    if output != INDENTED_TASK_DELETE_OUTPUT {
        return Err(format!(
            concat!(
                "deleting an indented task left unexpected source:\n",
                "expected:\n{}\n",
                "actual:\n{}"
            ),
            INDENTED_TASK_DELETE_OUTPUT, output
        )
        .into());
    }
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn delete_cr_only_task_does_not_preserve_the_document_prefix() -> TestResult {
    let test_workspace = create_workspace()?;
    test_workspace.write_target(CR_ONLY_TASK_TARGET)?;

    let output = run_from_args([
        "mapsplice",
        "delete",
        test_workspace.target.as_str(),
        "1.1.2",
    ])?
    .stdout
    .unwrap_or_default();

    if output != CR_ONLY_TASK_DELETE_OUTPUT {
        return Err(format!(
            concat!(
                "deleting from a CR-only roadmap preserved unexpected source:\n",
                "expected:\n{}\n",
                "actual:\n{}"
            ),
            CR_ONLY_TASK_DELETE_OUTPUT, output
        )
        .into());
    }
    Ok(())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn replace_preserves_untouched_two_space_task_source() -> TestResult {
    let test_workspace = create_workspace()?;
    test_workspace.write_target(TWO_SPACE_TARGET)?;
    test_workspace.write_fragment(TWO_SPACE_FRAGMENT)?;

    let output = run_from_args([
        "mapsplice",
        "replace",
        test_workspace.target.as_str(),
        "1.1.2",
        test_workspace.fragment.as_str(),
    ])?
    .stdout
    .unwrap_or_default();

    assert_contains(&output, UNTOUCHED_TWO_SPACE_TASK);
    Ok(())
}
