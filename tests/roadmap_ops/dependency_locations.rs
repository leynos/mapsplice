//! Source-coordinate regressions for dependency diagnostics.
//!
//! A clause that wraps across lines is parsed into one text node whose value
//! has the continuation indentation stripped. Diagnostics must still name the
//! token's column in the original Markdown, not in the normalized value.

use mapsplice::{MapspliceError, run_from_args};
use rstest::rstest;

use super::{
    assert_equal,
    support::{PHASE_FRAGMENT, TestResult, Workspace, workspace},
};

#[rstest]
#[serial_test::serial(cli_env)]
fn wrapped_dangling_dependency_reports_its_source_column(
    workspace: TestResult<Workspace>,
) -> TestResult {
    let test_workspace = workspace?;
    test_workspace
        .write_target(concat!(
            "# Example\n\n",
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. First task.\n\n",
            "  Requires 1.1.1\n  and 99.1.1.\n",
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
    .expect_err("an unresolved wrapped dependency must fail");

    let MapspliceError::DanglingDependency { anchor, location } = error else {
        return Err(format!("expected dangling dependency error, got {error:?}").into());
    };
    assert_equal(&anchor.to_string(), &"99.1.1".to_owned());
    let Some(reported) = location else {
        return Err("the dangling reference should carry its source location".into());
    };
    // The continuation line is indented by two spaces, so the token sits at
    // column 7 of line 10 even though the text value starts it at column 5.
    assert_equal(&reported.to_string(), &"line 10, column 7".to_owned());
    Ok(())
}
