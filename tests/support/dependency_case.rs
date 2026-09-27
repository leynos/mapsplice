//! Build one generated dependency-edit case, and run it through the CLI.
//!
//! `tests/roadmap_dependency_properties.rs` declares the properties; building
//! the case and running it lives here, and the assertions it is checked against
//! live in `tests/support/dependency_assertions.rs`, so no file outgrows the
//! 400-line limit.
//!
//! `Mode` is a property dimension rather than a fixed choice. Preview and
//! in-place share a rendering path but not a writing one, so an assertion that
//! a rejected edit left the target alone only proves something in the in-place
//! mode: preview never opens the file for writing, and its byte-identity check
//! would hold even if the rejection path truncated the target on its way to a
//! write.

use mapsplice::{CliRequest, GlobalOptions, MapspliceError, parse_cli_request, run_request};
use proptest::test_runner::TestCaseError;

use super::{
    generation::{Generated, Shape, generate},
    oracle::{Edit, Expectation, expectation_for},
    workspace_support::{Workspace, create_workspace},
};

/// One generated case: the edit, the documents, and the expected outcome.
pub struct Case {
    /// The structural edit the case applies.
    pub(crate) edit: Edit,
    /// The generated target, fragment, and dependency graph.
    pub(crate) generated: Generated,
    /// The outcome the oracle predicts.
    pub(crate) expectation: Expectation,
}

/// How one case's edit reaches the filesystem.
///
/// The mode is not a convenience. Preview and in-place share a rendering path
/// but not a writing one, so an assertion that the target is unchanged after a
/// rejection only proves something in the in-place mode: preview never opens
/// the file for writing, and its byte-identity check would hold even if the
/// rejection path truncated the target on its way to a write.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    /// Print the rewrite to standard output, leaving the target alone.
    Preview,
    /// Rewrite the target file.
    InPlace,
}

impl From<bool> for Mode {
    /// Map a generated in-place flag onto the mode it selects.
    fn from(in_place: bool) -> Self {
        if in_place {
            Self::InPlace
        } else {
            Self::Preview
        }
    }
}

/// Property cases are slow enough that a modest count exercises them well.
pub const CASES: u32 = 256;

/// Build one generated case from the supplied selectors.
///
/// `crlf` is a parameter of its own rather than a bit inside `shape_options`,
/// so that every run generates both endings. Leaving it to a bit of a random
/// byte would let a run skip CRLF entirely and report success without having
/// exercised it.
///
/// # Examples
///
/// ```ignore
/// let case = build_case(&[0], 3, 0, true)?;
/// assert!(matches!(case.edit, Edit::DeleteStep { .. }));
/// ```
pub fn build_case(
    step_selectors: &[u8],
    edit_selector: u8,
    shape_options: u8,
    crlf: bool,
) -> Result<Case, TestCaseError> {
    let options = if crlf {
        shape_options | Shape::CRLF
    } else {
        shape_options & !Shape::CRLF
    };
    let shape = Shape { options };
    let generated =
        generate(step_selectors, shape).map_err(|error| TestCaseError::fail(error.to_string()))?;
    let edit = Edit::from_selector(edit_selector);
    let expectation = expectation_for(&generated.model, edit);
    Ok(Case {
        edit,
        generated,
        expectation,
    })
}

/// Write the generated target and fragment into a fresh workspace.
///
/// Helpers propagate with `?` rather than `expect`, because Clippy's
/// `expect_used` is denied and `allow-expect-in-tests` covers only the
/// `#[test]` bodies themselves, not the helpers they call.
pub fn prepare_workspace(case: &Case) -> Result<Workspace, TestCaseError> {
    let workspace = create_workspace().map_err(|error| TestCaseError::fail(error.to_string()))?;
    workspace
        .write_target(&case.generated.target)
        .map_err(|error| TestCaseError::fail(error.to_string()))?;
    workspace
        .write_fragment(&case.generated.fragment)
        .map_err(|error| TestCaseError::fail(error.to_string()))?;
    Ok(workspace)
}

/// Run the generated edit through the CLI, returning the rewritten roadmap.
///
/// The request is built and run in process: a rejected edit must never reach
/// the filesystem, and the in-process path lets the test inspect the target
/// afterwards without a shell.
///
/// The mode is assigned to the parsed request's `global` field rather than
/// passed as `--in-place`, because the flag is only one of three inputs to that
/// field: an environment variable and a discovered `.mapsplice.toml` also feed
/// it, and a property test sharing a process with the configuration suite would
/// otherwise be at their mercy. Which *arguments* set the field is covered by
/// the CLI regressions and `tests/cli_ui.rs`.
pub fn run_case(workspace: &Workspace, case: &Case, mode: Mode) -> Result<String, MapspliceError> {
    let mut args = vec!["mapsplice".to_owned()];
    match case.edit {
        Edit::InsertTasksBefore { .. } => {
            args.push("insert".to_owned());
            args.push(workspace.target.to_string());
            args.push(case.edit.anchor());
            args.push(workspace.fragment.to_string());
        }
        Edit::ReplaceTask { .. } => {
            args.push("replace".to_owned());
            args.push(workspace.target.to_string());
            args.push(case.edit.anchor());
            args.push(workspace.fragment.to_string());
        }
        Edit::DeleteTask { .. } | Edit::DeleteStep { .. } | Edit::DeletePhase { .. } => {
            args.push("delete".to_owned());
            args.push(workspace.target.to_string());
            args.push(case.edit.anchor());
        }
    }
    let parsed = parse_cli_request(args)?;
    let request = CliRequest {
        global: GlobalOptions {
            in_place: mode == Mode::InPlace,
        },
        ..parsed
    };
    let outcome = run_request(request)?;
    match mode {
        Mode::Preview => Ok(outcome.stdout.unwrap_or_default()),
        Mode::InPlace => read_target(workspace),
    }
}

/// Read the target file back through the workspace capability.
fn read_target(workspace: &Workspace) -> Result<String, MapspliceError> {
    workspace
        .dir
        .read_to_string("target.md")
        .map_err(|error| MapspliceError::Io {
            action: "reading",
            path: workspace.target.clone(),
            source: error,
        })
}
