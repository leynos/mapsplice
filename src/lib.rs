//! Core library for the `mapsplice` roadmap splicing CLI.

mod cli;
mod error;
mod fs;
mod observability;
mod roadmap;

use camino::Utf8PathBuf;
pub use error::{MapspliceError, Result};
use fs::{read_utf8, rewrite_utf8};
use roadmap::{RoadmapOperation, apply_command, parse_fragment, parse_roadmap, render_roadmap};

/// Execute `mapsplice` using command-line arguments.
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
    let request = parse_cli_request(args)?;
    run_request(request).inspect_err(|error| observability::record_failure(error.class()))
}

/// Execute a parsed request.
///
/// # Errors
///
/// Returns an error when the target or fragment cannot be read, when either
/// document is invalid, when the splice operation is not valid for the
/// addressed anchor, or when in-place rewriting fails.
pub fn run_request(request: CliRequest) -> Result<RunOutcome> {
    let operation = operation_from_command(&request.command);
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
    let mut roadmap = parse_roadmap(&target_text)?;
    let fragment = load_fragment(&request)?;

    apply_command(&mut roadmap, operation, fragment.as_ref())?;

    let rendered = render_roadmap(&roadmap)?;
    if request.global.in_place {
        rewrite_utf8(&request.target, &rendered)?;
        observability::record_in_place_rewrite();
        Ok(RunOutcome::in_place(request.target))
    } else {
        Ok(RunOutcome::stdout(rendered))
    }
}

fn load_fragment(request: &CliRequest) -> Result<Option<roadmap::RoadmapFragment>> {
    match request.command.fragment_path() {
        Some(path) => {
            tracing::debug!(path = %path, "loading roadmap fragment");
            let fragment_text = read_utf8(path)?;
            parse_fragment(&fragment_text).map(Some)
        }
        None => Ok(None),
    }
}

const fn operation_from_command(command: &cli::CommandKind) -> RoadmapOperation {
    match command {
        CommandKind::Append { .. } => RoadmapOperation::Append,
        CommandKind::Insert { anchor, after, .. } => RoadmapOperation::Insert {
            anchor: *anchor,
            after: *after,
        },
        CommandKind::Delete { anchor } => RoadmapOperation::Delete { anchor: *anchor },
        CommandKind::Replace { anchor, .. } => RoadmapOperation::Replace { anchor: *anchor },
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
    const fn stdout(rendered: String) -> Self {
        Self {
            stdout: Some(rendered),
            written_path: None,
        }
    }

    const fn in_place(path: Utf8PathBuf) -> Self {
        Self {
            stdout: None,
            written_path: Some(path),
        }
    }
}

pub use cli::{CliRequest, CommandKind, GlobalOptions, parse_cli_request};
pub use observability::{MetricsSnapshot, metrics_snapshot};
pub use roadmap::{
    RoadmapAnchor,
    RoadmapDocument,
    RoadmapFragment,
    RoadmapItemLevel,
    fragment_level,
    parse_anchor,
    parse_fragment as parse_fragment_text,
    parse_roadmap as parse_roadmap_text,
};
