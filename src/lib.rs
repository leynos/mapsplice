//! Core library for the `mapsplice` roadmap splicing CLI.

mod cli;
mod error;
mod fs;
pub mod observability;
mod roadmap;

use camino::Utf8PathBuf;
pub use error::{MapspliceError, Result};
use fs::{read_utf8, rewrite_utf8};
use roadmap::{
    RoadmapOperation as RoadmapOperationInner,
    apply_command as apply_command_inner,
    parse_fragment as parse_fragment_inner,
    parse_roadmap as parse_roadmap_inner,
    render_roadmap,
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
    let mut roadmap = parse_roadmap_inner(&target_text)?;
    let fragment = load_fragment(&request)?;

    let dependency_rewrites = apply_command_inner(&mut roadmap, operation, fragment)?;
    let rendered = render_roadmap(&roadmap)?;
    if request.global.in_place {
        rewrite_utf8(&request.target, &rendered)?;
        observability::record_in_place_rewrite();
    }
    observability::record_dependency_rewrites(dependency_rewrites);

    if request.global.in_place {
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
            parse_fragment_inner(&fragment_text).map(Some)
        }
        None => Ok(None),
    }
}

const fn operation_from_command(command: &cli::CommandKind) -> RoadmapOperationInner {
    match command {
        CommandKind::Append { .. } => RoadmapOperationInner::Append,
        CommandKind::Insert { anchor, after, .. } => RoadmapOperationInner::Insert {
            anchor: *anchor,
            after: *after,
        },
        CommandKind::Delete { anchor } => RoadmapOperationInner::Delete { anchor: *anchor },
        CommandKind::Replace { anchor, .. } => RoadmapOperationInner::Replace { anchor: *anchor },
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
};
