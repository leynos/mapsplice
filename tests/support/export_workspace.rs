//! Shared workspace and request helpers for the export property tests.

// These properties only write roadmap targets; the shared fixture also offers
// fragment helpers that the splice test binaries use.
#[expect(
    dead_code,
    reason = "the export property test writes only roadmap targets, not fragments"
)]
#[path = "workspace.rs"]
mod workspace;

use camino::Utf8Path;
use mapsplice::{
    CliRequest,
    CommandKind,
    ExportFormat,
    GlobalOptions,
    MapspliceError,
    run_request,
};
pub(crate) use workspace::{TestResult, Workspace, create_workspace};

/// Export one roadmap source through the library request path.
pub(crate) fn export_source(source: &str) -> TestResult<String> {
    let workspace = create_workspace()?;
    workspace.write_target(source)?;
    let outcome = run_request(CliRequest {
        global: GlobalOptions::default(),
        target: workspace.target.clone(),
        command: CommandKind::Export {
            format: ExportFormat::Json,
            check: None,
        },
    })?;
    outcome
        .stdout
        .ok_or_else(|| "export should write JSON to stdout".into())
}

/// Run the committed-export check for a workspace.
pub(crate) fn check_export(
    workspace: &Workspace,
    export_path: &Utf8Path,
) -> Result<mapsplice::RunOutcome, MapspliceError> {
    run_request(CliRequest {
        global: GlobalOptions::default(),
        target: workspace.target.clone(),
        command: CommandKind::Export {
            format: ExportFormat::Json,
            check: Some(export_path.to_owned()),
        },
    })
}
