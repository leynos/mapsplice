//! Execution of the read-only export command.
//!
//! The export adapter in [`crate::roadmap::export`] is pure: it maps a parsed
//! roadmap to the versioned schema and compares two documents. This module
//! owns the surrounding infrastructure—reading the source and committed
//! export, serializing the fresh document, and turning a mismatch into an
//! [`ExportError::Drift`] report—while never writing to disk.

use camino::{Utf8Path, Utf8PathBuf};

use crate::{
    RunOutcome,
    cli::ExportFormat,
    error::{ExportError, MapspliceError, Result},
    observability::{ExportOutcome, record_export_attempt, record_export_outcome},
    roadmap::{self, parse_roadmap},
};

/// Fallible UTF-8 reader supplied to the export orchestration.
///
/// Injecting the reader keeps the read-only orchestration testable without a
/// temporary filesystem, and keeps filesystem policy in [`crate::fs`].
pub(crate) type ReadUtf8Fn<'reader> = &'reader dyn Fn(&Utf8Path) -> Result<String>;

/// Execute the read-only JSON export and optional committed-file comparison.
pub(crate) fn run_export(
    target: &Utf8Path,
    format: ExportFormat,
    check: Option<&Utf8PathBuf>,
    read: ReadUtf8Fn<'_>,
) -> Result<RunOutcome> {
    let span = tracing::info_span!(
        "run_export",
        operation = "export",
        format = format.as_str(),
        check = check.is_some()
    );
    let _span_guard = span.enter();
    record_export_attempt();
    let result = export_request(target, format, check, read);
    record_export_outcome(decision_for(&result, check.is_some()));
    result
}

/// Return the single outcome recorded for one completed export attempt.
///
/// Deriving the decision from the result keeps the accounting exhaustive: every
/// return path, including a source read failure, records exactly one outcome.
const fn decision_for(result: &Result<RunOutcome>, has_check: bool) -> ExportOutcome {
    match result {
        Ok(_) if has_check => ExportOutcome::Matched,
        Ok(_) => ExportOutcome::Emitted,
        Err(MapspliceError::Export(ExportError::Drift { .. })) => ExportOutcome::Drifted,
        Err(_) => ExportOutcome::Failed,
    }
}

/// Render the export and, when requested, compare it with the committed file.
fn export_request(
    target: &Utf8Path,
    format: ExportFormat,
    check: Option<&Utf8PathBuf>,
    read: ReadUtf8Fn<'_>,
) -> Result<RunOutcome> {
    let schema = export_schema(target, read)?;
    let rendered = RenderedExport {
        json: serialize_export(&schema, format)?,
        schema: &schema,
    };
    match check {
        Some(export_path) => verify_export(export_path, target, &rendered, read),
        None => Ok(emitted_export(rendered.json)),
    }
}

/// A freshly rendered export, kept with its schema for drift reporting.
struct RenderedExport<'schema> {
    /// Serialized document compared byte-for-byte with the committed export.
    json: String,
    /// Decoded schema used to describe semantic differences.
    schema: &'schema roadmap::export::RoadmapExport,
}

/// Return the freshly rendered export.
fn emitted_export(fresh_json: String) -> RunOutcome {
    tracing::debug!(decision = "emitted", "export completed");
    RunOutcome::stdout(fresh_json)
}

/// Parse the roadmap and adapt it to the export schema without writing output.
fn export_schema(
    target: &Utf8Path,
    read: ReadUtf8Fn<'_>,
) -> Result<roadmap::export::RoadmapExport> {
    let target_text = read(target)?;
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
    target: &Utf8Path,
    rendered: &RenderedExport<'_>,
    read: ReadUtf8Fn<'_>,
) -> Result<RunOutcome> {
    let committed_json = read_committed_export(export_path, read)?;
    if committed_json.as_bytes() == rendered.json.as_bytes() {
        return Ok(matched_export());
    }
    let committed_schema = parse_committed_export(export_path, &committed_json)?;
    Err(export_drift(export_path, target, rendered.schema, &committed_schema).into())
}

/// Read a committed export, attributing unreadable input to the committed file.
fn read_committed_export(export_path: &Utf8PathBuf, read: ReadUtf8Fn<'_>) -> Result<String> {
    read(export_path).map_err(|error| {
        ExportError::InvalidCommitted {
            path: export_path.to_owned(),
            message: error.to_string(),
        }
        .into()
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
    target: &Utf8Path,
    fresh_schema: &roadmap::export::RoadmapExport,
    committed_schema: &roadmap::export::RoadmapExport,
) -> ExportError {
    let mut differences = roadmap::export::drift::differences(committed_schema, fresh_schema);
    if differences.is_empty() {
        differences.push("formatting-only drift: semantic JSON values are equal".to_owned());
    }
    tracing::debug!(
        decision = "drift",
        differences = differences.len(),
        "export check completed"
    );
    ExportError::Drift {
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
    serde_json::from_str(committed_json).map_err(|error| {
        ExportError::InvalidCommitted {
            path: export_path.to_owned(),
            message: error.to_string(),
        }
        .into()
    })
}

#[cfg(test)]
mod tests;
