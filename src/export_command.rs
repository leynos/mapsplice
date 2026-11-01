//! Execution of the read-only export command.
//!
//! The export adapter in [`crate::roadmap::export`] is pure: it maps a parsed
//! roadmap to the versioned schema and compares two documents. This module
//! owns the surrounding infrastructure—reading the source and committed
//! export, serializing the fresh document, and turning a mismatch into an
//! [`MapspliceError::ExportDrift`] report—while never writing to disk.

use camino::Utf8PathBuf;

use crate::{
    RunOutcome,
    cli::ExportFormat,
    error::{MapspliceError, Result},
    fs::read_utf8,
    roadmap::{self, parse_roadmap},
};

/// Execute the read-only JSON export and optional committed-file comparison.
pub(crate) fn run_export(
    target: &camino::Utf8Path,
    format: ExportFormat,
    check: Option<&Utf8PathBuf>,
) -> Result<RunOutcome> {
    let span = tracing::info_span!(
        "run_export",
        operation = "export",
        format = format.as_str(),
        target = %target,
        check = check.map_or("", |path| path.as_str())
    );
    let _span_guard = span.enter();
    let fresh_schema = export_schema(target)?;
    let fresh_json = serialize_export(&fresh_schema, format)?;
    match check {
        Some(export_path) => verify_export(export_path, target, &fresh_schema, &fresh_json),
        None => Ok(emitted_export(fresh_json)),
    }
}

/// Return the freshly rendered export after recording the emitted decision.
fn emitted_export(fresh_json: String) -> RunOutcome {
    tracing::debug!(decision = "emitted", "export completed");
    RunOutcome::stdout(fresh_json)
}

/// Parse the roadmap and adapt it to the export schema without writing output.
fn export_schema(target: &camino::Utf8Path) -> Result<roadmap::export::RoadmapExport> {
    let target_text = read_utf8(target)?;
    let roadmap = parse_roadmap(&target_text)?;
    roadmap::export::to_schema(&roadmap)
}

/// Serialize an export schema in the requested format.
fn serialize_export(
    schema: &roadmap::export::RoadmapExport,
    format: ExportFormat,
) -> Result<String> {
    match format {
        ExportFormat::Json => roadmap::export::serialize(schema),
    }
}

/// Compare a committed export with the freshly rendered JSON bytes.
fn verify_export(
    export_path: &Utf8PathBuf,
    target: &camino::Utf8Path,
    fresh_schema: &roadmap::export::RoadmapExport,
    fresh_json: &str,
) -> Result<RunOutcome> {
    let committed_json = read_committed_export(export_path)?;
    if committed_json.as_bytes() == fresh_json.as_bytes() {
        return Ok(matched_export());
    }
    let committed_schema = parse_committed_export(export_path, &committed_json)?;
    Err(export_drift(
        export_path,
        target,
        fresh_schema,
        &committed_schema,
    ))
}

/// Read a committed export, attributing unreadable input to the committed file.
fn read_committed_export(export_path: &Utf8PathBuf) -> Result<String> {
    read_utf8(export_path).map_err(|error| MapspliceError::InvalidExport {
        path: export_path.to_owned(),
        message: error.to_string(),
    })
}

/// Return the successful outcome for a committed export that matches exactly.
fn matched_export() -> RunOutcome {
    tracing::debug!(decision = "match", "export check completed");
    RunOutcome::silent_success()
}

/// Build the drift error describing every semantic difference.
fn export_drift(
    export_path: &Utf8PathBuf,
    target: &camino::Utf8Path,
    fresh_schema: &roadmap::export::RoadmapExport,
    committed_schema: &roadmap::export::RoadmapExport,
) -> MapspliceError {
    let mut differences = roadmap::export::drift::differences(committed_schema, fresh_schema);
    if differences.is_empty() {
        differences.push("formatting-only drift: semantic JSON values are equal".to_owned());
    }
    tracing::debug!(
        decision = "drift",
        differences = differences.len(),
        "export check completed"
    );
    MapspliceError::ExportDrift {
        export: export_path.to_owned(),
        roadmap: target.to_owned(),
        differences: differences.join("\n"),
        regenerate: format!("mapsplice export {target} > {export_path}"),
    }
}

/// Decode a committed export, attributing malformed JSON to the committed file.
fn parse_committed_export(
    export_path: &Utf8PathBuf,
    committed_json: &str,
) -> Result<roadmap::export::RoadmapExport> {
    serde_json::from_str(committed_json).map_err(|error| MapspliceError::InvalidExport {
        path: export_path.to_owned(),
        message: error.to_string(),
    })
}
