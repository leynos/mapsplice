//! Generated coverage for dependency diagnostic coordinates on the export path.
//!
//! A clause that wraps across lines is parsed into one text node whose value
//! has the continuation indentation stripped, so the value offset is not the
//! source offset. These cases drive the real export command and compare its
//! reported coordinates against positions counted from the generated source.

use mapsplice::{MapspliceError, run_from_args};
use proptest::{prelude::*, test_runner::TestCaseError};

use crate::workspace_support::create_workspace;

/// Render a roadmap whose `Requires` clause wraps over three lines.
///
/// The valid anchor appears twice before the dangling token, so the mapping
/// has to pick the repeated occurrence rather than the first match. The
/// returned line and column are counted while building the source, so the
/// expectation never passes through the parser's normalization.
fn wrapped_dangling_source(
    line_ending: &str,
    indent: usize,
    has_preceding_text: bool,
    dangling: &str,
) -> (String, usize, usize) {
    let pad = " ".repeat(indent);
    let mut lines = vec![
        "## 1. Phase one".to_owned(),
        String::new(),
        "### 1.1. Step one".to_owned(),
        String::new(),
        "- [ ] 1.1.1. First task.".to_owned(),
        String::new(),
    ];
    if has_preceding_text {
        lines.push("  Background note.".to_owned());
        lines.push(String::new());
    }
    lines.push("  Requires 1.1.1,".to_owned());
    lines.push(format!("  {pad}and 1.1.1,"));
    let last_prefix = format!("  {pad}and ");
    lines.push(format!("{last_prefix}{dangling}."));

    let line = lines.len();
    let column = last_prefix.chars().count() + 1;
    let mut source = lines.join(line_ending);
    source.push_str(line_ending);
    (source, line, column)
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        .. ProptestConfig::default()
    })]

    /// A wrapped clause in a task body must report the dangling token's own
    /// source line and column for every generated layout.
    #[test]
    fn export_reports_wrapped_dependency_coordinates(
        line_ending in prop_oneof![Just("\n"), Just("\r\n")],
        indent in 0usize..4,
        has_preceding_text in any::<bool>(),
        dangling in prop_oneof![Just("99.1.1"), Just("12.3.4")],
    ) {
        let (source, line, column) =
            wrapped_dangling_source(line_ending, indent, has_preceding_text, dangling);
        let workspace = create_workspace()
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        workspace
            .write_target(&source)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        let error = run_from_args(["mapsplice", "export", workspace.target.as_str()])
            .expect_err("a dangling wrapped dependency must fail the export");
        let MapspliceError::DanglingDependency { anchor, location } = error else {
            return Err(TestCaseError::fail(format!(
                "expected a dangling dependency, got {error:?}"
            )));
        };
        prop_assert_eq!(anchor.to_string(), dangling.to_owned());
        let reported = location.ok_or_else(|| {
            TestCaseError::fail("the diagnostic should carry a source location")
        })?;
        prop_assert_eq!(reported.line, line);
        prop_assert_eq!(reported.column, column);
    }

    /// The same layout inside an unnumbered checkbox detail must report the
    /// detail's token, which the export path reaches through a different
    /// collector than a task body.
    #[test]
    fn export_reports_wrapped_checkbox_detail_coordinates(
        line_ending in prop_oneof![Just("\n"), Just("\r\n")],
        indent in 0usize..4,
        dangling in prop_oneof![Just("99.1.1"), Just("12.3.4")],
    ) {
        let pad = " ".repeat(indent);
        let mut lines = vec![
            "## 1. Phase one".to_owned(),
            String::new(),
            "### 1.1. Step one".to_owned(),
            String::new(),
            "- [ ] 1.1.1. First task.".to_owned(),
            "  - [x] Detail.".to_owned(),
            String::new(),
            "    Requires 1.1.1,".to_owned(),
            format!("    {pad}and 1.1.1,"),
        ];
        let last_prefix = format!("    {pad}and ");
        lines.push(format!("{last_prefix}{dangling}."));

        let line = lines.len();
        let column = last_prefix.chars().count() + 1;
        let mut source = lines.join(line_ending);
        source.push_str(line_ending);

        let workspace = create_workspace()
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        workspace
            .write_target(&source)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        let error = run_from_args(["mapsplice", "export", workspace.target.as_str()])
            .expect_err("a dangling wrapped dependency must fail the export");
        let MapspliceError::DanglingDependency { anchor, location } = error else {
            return Err(TestCaseError::fail(format!(
                "expected a dangling dependency, got {error:?}"
            )));
        };
        prop_assert_eq!(anchor.to_string(), dangling.to_owned());
        let reported = location.ok_or_else(|| {
            TestCaseError::fail("the diagnostic should carry a source location")
        })?;
        prop_assert_eq!(reported.line, line);
        prop_assert_eq!(reported.column, column);
    }
}
