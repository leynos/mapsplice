//! Core library for the `mapsplice` roadmap splicing CLI.

mod cli;
mod error;
mod fs;
pub mod observability;
mod roadmap;

#[cfg(test)]
#[doc(hidden)]
pub mod doctest_support {
    //! Hidden helpers used by documentation examples.

    pub use crate::roadmap::source_preservation::original_node_source;
}

use camino::Utf8PathBuf;
pub use error::{MapspliceError, Result, SourceLocation};
use fs::{read_utf8, rewrite_utf8};
use roadmap::{
    RoadmapOperation as RoadmapOperationInner,
    apply_command_with_report,
    parse_fragment as parse_fragment_inner,
    parse_roadmap as parse_roadmap_inner,
    render_roadmap_with_report,
};

/// Execute `mapsplice` using command-line arguments.
///
/// # Examples
///
/// ```rust
/// use mapsplice::run_from_args;
///
/// # use std::{fs, io};
/// # use camino::Utf8PathBuf;
/// # use tempfile::{TempDir, tempdir};
/// #
/// # fn temp_path(temp: &TempDir, name: &str) -> Result<Utf8PathBuf, io::Error> {
/// #     Utf8PathBuf::from_path_buf(temp.path().join(name)).map_err(|_| {
/// #         io::Error::new(io::ErrorKind::InvalidData, "temporary path was not UTF-8")
/// #     })
/// # }
/// #
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let temp = tempdir()?;
/// let target = temp_path(&temp, "target.md")?;
/// let fragment = temp_path(&temp, "fragment.md")?;
///
/// # fs::write(
/// #     &target,
/// #     concat!(
/// #         "## 1. First phase\n\n### 1.1. First step\n\n- [ ] 1.1.1. Keep this task\n\n",
/// #         "## 2. Original second phase\n\n### 2.1. Original step\n\n- [ ] 2.1.1. Keep this task\n",
/// #     ),
/// # )?;
/// # fs::write(
/// #     &fragment,
/// #     "## 1. Inserted phase\n\n### 1.1. Inserted step\n\n- [ ] 1.1.1. Add this task\n",
/// # )?;
/// let args = vec![
///     "mapsplice".to_owned(),
///     "insert".to_owned(),
///     target.to_string(),
///     "2".to_owned(),
///     fragment.to_string(),
/// ];
///
/// let outcome = run_from_args(args)?;
///
/// assert!(outcome.written_path.is_none());
/// let rendered = outcome
///     .stdout
///     .as_deref()
///     .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "expected stdout"))?;
/// assert!(rendered.contains("## 2. Inserted phase"));
/// assert!(rendered.contains("## 3. Original second phase"));
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error when argument parsing, configuration loading, file I/O,
/// roadmap parsing, splice validation, or rendering fails.
pub fn run_from_args<I, T>(args: I) -> Result<RunOutcome>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    parse_cli_request(args)
        .and_then(run_request)
        .inspect_err(|error| {
            if should_record_failure(error) {
                observability::record_failure(error.class());
            }
        })
}

/// Exclude requested help and version output from failed-command metrics.
fn should_record_failure(error: &MapspliceError) -> bool {
    !matches!(
        error,
        MapspliceError::Clap(clap_error)
            if matches!(
                clap_error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            )
    )
}

/// Execute a parsed request.
///
/// # Examples
///
/// ```rust
/// use mapsplice::{CliRequest, CommandKind, GlobalOptions, run_request};
///
/// # use std::{fs, io};
/// # use camino::Utf8PathBuf;
/// # use tempfile::{TempDir, tempdir};
/// #
/// # fn temp_path(temp: &TempDir, name: &str) -> Result<Utf8PathBuf, io::Error> {
/// #     Utf8PathBuf::from_path_buf(temp.path().join(name)).map_err(|_| {
/// #         io::Error::new(io::ErrorKind::InvalidData, "temporary path was not UTF-8")
/// #     })
/// # }
/// #
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let temp = tempdir()?;
/// let target = temp_path(&temp, "target.md")?;
/// let fragment = temp_path(&temp, "fragment.md")?;
///
/// # fs::write(
/// #     &target,
/// #     "## 1. Existing phase\n\n### 1.1. Existing step\n\n- [ ] 1.1.1. Keep this task\n",
/// # )?;
/// # fs::write(
/// #     &fragment,
/// #     "## 1. Added phase\n\n### 1.1. Added step\n\n- [ ] 1.1.1. Add this task\n",
/// # )?;
/// let request = CliRequest {
///     global: GlobalOptions { in_place: false },
///     target,
///     command: CommandKind::Append { fragment },
/// };
///
/// let outcome = run_request(request)?;
///
/// assert!(outcome.written_path.is_none());
/// let rendered = outcome
///     .stdout
///     .as_deref()
///     .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "expected stdout"))?;
/// assert!(rendered.contains("## 2. Added phase"));
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error when the target or fragment cannot be read, when either
/// document is invalid, when the splice operation is not valid for the
/// addressed anchor, or when in-place rewriting fails.
pub fn run_request(request: CliRequest) -> Result<RunOutcome> {
    if let CommandKind::Export { format, check } = &request.command {
        return run_export(&request.target, *format, check.as_ref());
    }
    let edit = edit_command(&request.command).ok_or_else(|| MapspliceError::Configuration {
        message: "edit command conversion failed before mutation".to_owned(),
    })?;
    let operation = operation_from_command(edit);
    let operation_name = operation.name();
    let anchor = operation.anchor().map(|anchor| anchor.to_string());
    let span = tracing::info_span!(
        "run_request",
        operation = operation_name,
        anchor = anchor.as_deref().unwrap_or(""),
        target = %request.target,
        in_place = request.global.in_place
    );
    let _span_guard = span.enter();
    let target_text = read_utf8(&request.target)?;
    let mut roadmap = parse_roadmap_inner(&target_text)?;
    let fragment = load_fragment(&request)?;

    let (dependency_rewrites, mut preservation_report) =
        apply_command_with_report(&mut roadmap, operation, fragment)?;
    let (rendered, render_report) = render_roadmap_with_report(&roadmap)?;
    if request.global.in_place {
        rewrite_utf8(&request.target, &rendered)?;
        observability::record_in_place_rewrite();
    }
    observability::record_dependency_rewrites(dependency_rewrites);
    preservation_report.extend(render_report);
    observability::record_preservation_report(&preservation_report);

    if request.global.in_place {
        Ok(RunOutcome::in_place(request.target))
    } else {
        Ok(RunOutcome::stdout(rendered))
    }
}

/// Execute the read-only JSON export and optional committed-file comparison.
fn run_export(
    target: &camino::Utf8Path,
    format: cli::ExportFormat,
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
    let roadmap = parse_roadmap_inner(&target_text)?;
    roadmap::export::to_schema(&roadmap)
}

/// Serialize an export schema in the requested format.
fn serialize_export(
    schema: &roadmap::export::RoadmapExport,
    format: cli::ExportFormat,
) -> Result<String> {
    match format {
        cli::ExportFormat::Json => roadmap::export::serialize(schema),
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

/// Load a fragment only for commands that consume one.
fn load_fragment(request: &CliRequest) -> Result<Option<roadmap::RoadmapFragment>> {
    match request.command.fragment_path() {
        Some(path) => {
            tracing::debug!(path = %path, "loading roadmap fragment");
            let fragment_text = read_utf8(path)?;
            parse_fragment_inner(&fragment_text).map(Some)
        }
        None => Ok(None),
    }
}

/// Edit-only command values accepted by the mutating execution pipeline.
#[derive(Clone, Copy)]
enum EditCommand {
    /// Append a fragment to the roadmap.
    Append,
    /// Insert a fragment around an existing anchor.
    Insert {
        /// Existing roadmap location where the fragment is inserted.
        anchor: RoadmapAnchor,
        /// Insert after the anchor when true, before it otherwise.
        after: bool,
    },
    /// Delete an existing anchor.
    Delete {
        /// Roadmap location to remove.
        anchor: RoadmapAnchor,
    },
    /// Replace an existing anchor with a fragment.
    Replace {
        /// Roadmap location to replace.
        anchor: RoadmapAnchor,
    },
}

/// Convert a parsed command only when it belongs to the edit pipeline.
const fn edit_command(command: &CommandKind) -> Option<EditCommand> {
    match command {
        CommandKind::Append { .. } => Some(EditCommand::Append),
        CommandKind::Insert { anchor, after, .. } => Some(EditCommand::Insert {
            anchor: *anchor,
            after: *after,
        }),
        CommandKind::Delete { anchor } => Some(EditCommand::Delete { anchor: *anchor }),
        CommandKind::Replace { anchor, .. } => Some(EditCommand::Replace { anchor: *anchor }),
        CommandKind::Export { .. } => None,
    }
}

/// Translate an edit-only command into the roadmap operation to apply.
const fn operation_from_command(command: EditCommand) -> RoadmapOperationInner {
    match command {
        EditCommand::Append => RoadmapOperationInner::Append,
        EditCommand::Insert { anchor, after } => RoadmapOperationInner::Insert { anchor, after },
        EditCommand::Delete { anchor } => RoadmapOperationInner::Delete { anchor },
        EditCommand::Replace { anchor } => RoadmapOperationInner::Replace { anchor },
    }
}

/// Result of running the CLI.
#[derive(Debug, Eq, PartialEq)]
pub struct RunOutcome {
    /// Rewritten roadmap for standard output mode.
    pub stdout: Option<String>,
    /// Target rewritten in place, when requested.
    pub written_path: Option<Utf8PathBuf>,
}

impl RunOutcome {
    /// Wrap rendered text for a command that writes to standard output.
    const fn stdout(rendered: String) -> Self {
        Self {
            stdout: Some(rendered),
            written_path: None,
        }
    }

    /// Record the target path for a successful in-place rewrite.
    const fn in_place(path: Utf8PathBuf) -> Self {
        Self {
            stdout: None,
            written_path: Some(path),
        }
    }

    /// Return a successful outcome with no stdout or file mutation.
    const fn silent_success() -> Self {
        Self {
            stdout: None,
            written_path: None,
        }
    }
}

pub use cli::{CliRequest, CommandKind, ExportFormat, GlobalOptions, parse_cli_request};
pub use observability::{
    MetricsSnapshot,
    metrics_snapshot,
    record_dependency_rewrites,
    record_failure,
    record_in_place_rewrite,
};
pub use roadmap::{
    PhaseNumber,
    RoadmapAnchor,
    RoadmapDocument,
    RoadmapFragment,
    RoadmapItemLevel,
    RoadmapOperation,
    StepNumber,
    SubTaskEntry,
    SubTaskNumber,
    TaskNumber,
    apply_command,
    fragment_level,
    parse_anchor,
    parse_fragment,
    parse_fragment as parse_fragment_text,
    parse_roadmap,
    parse_roadmap as parse_roadmap_text,
    render_roadmap,
};
