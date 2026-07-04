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
