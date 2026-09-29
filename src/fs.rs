//! Filesystem helpers that keep ambient access at the CLI edge.

use std::{
    env,
    io::{self, Write},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs::OpenOptions, fs_utf8::Dir};

use crate::error::{MapspliceError, Result};

/// Distinguish temporary names created within the same process and timestamp.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Read a UTF-8 file through a capability directory.
#[tracing::instrument(skip_all, fields(path = %path))]
pub fn read_utf8(path: &Utf8Path) -> Result<String> {
    let cap = open_parent_dir(path)?;
    cap.dir
        .read_to_string(&cap.file_name)
        .map_err(|source| MapspliceError::Io {
            action: "failed to read",
            path: cap.absolute,
            source,
        })
}

/// Rewrite a UTF-8 file through a temporary sibling file and rename.
#[tracing::instrument(skip_all, fields(path = %path, bytes = contents.len()))]
pub fn rewrite_utf8(path: &Utf8Path, contents: &str) -> Result<()> {
    let cap = open_parent_dir(path)?;
    let temp_name = temp_file_name(&cap.file_name)?;

    rewrite_utf8_with_strategy(
        &cap,
        &temp_name,
        contents,
        RewriteStrategy {
            open_temp: |dir: &Dir, temp: &str, options: &OpenOptions| dir.open_with(temp, options),
            replace_target: |dir: &Dir, temp: &str, target: &str| dir.rename(temp, dir, target),
        },
    )
}

/// Write through a sibling file using injectable operations for failure tests.
fn rewrite_utf8_with_strategy<W, OpenTemp, ReplaceTarget>(
    cap: &FileCap,
    temp_name: &str,
    contents: &str,
    strategy: RewriteStrategy<OpenTemp, ReplaceTarget>,
) -> Result<()>
where
    W: Write,
    OpenTemp: FnOnce(&Dir, &str, &OpenOptions) -> io::Result<W>,
    ReplaceTarget: FnOnce(&Dir, &str, &str) -> io::Result<()>,
{
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);

    let mut temp = (strategy.open_temp)(&cap.dir, temp_name, &options).map_err(|source| {
        MapspliceError::Io {
            action: "failed to create temporary file for",
            path: cap.absolute.clone(),
            source,
        }
    })?;
    if let Err(source) = temp.write_all(contents.as_bytes()) {
        drop(temp);
        discard_temporary_file(cap, temp_name);
        return Err(MapspliceError::Io {
            action: "failed to write temporary file for",
            path: cap.absolute.clone(),
            source,
        });
    }
    drop(temp);

    if let Err(source) = (strategy.replace_target)(&cap.dir, temp_name, &cap.file_name) {
        discard_temporary_file(cap, temp_name);
        return Err(MapspliceError::Io {
            action: "failed to replace",
            path: cap.absolute.clone(),
            source,
        });
    }

    Ok(())
}

/// Operations used to create and replace a temporary sibling file.
struct RewriteStrategy<OpenTemp, ReplaceTarget> {
    /// Create a new temporary file for the pending contents.
    open_temp: OpenTemp,
    /// Replace the target with the completed temporary file.
    replace_target: ReplaceTarget,
}

/// Attempt to remove a temporary file after a failed rewrite.
fn discard_temporary_file(cap: &FileCap, temp_name: &str) {
    if let Err(source) = cap.dir.remove_file(temp_name) {
        tracing::debug!(
            path = %cap.absolute,
            temp_name,
            error = %source,
            "failed to remove temporary rewrite file after original failure"
        );
    }
}

/// Capability and path metadata for one file's parent directory.
struct FileCap {
    /// Directory capability used for file operations.
    dir: Dir,
    /// File name relative to the directory capability.
    file_name: String,
    /// Absolute path retained for error reporting.
    absolute: Utf8PathBuf,
}

/// Open the parent directory for a file path and retain display metadata.
fn open_parent_dir(path: &Utf8Path) -> Result<FileCap> {
    let absolute = absolutize(path)?;
    let parent = absolute.parent().ok_or_else(|| {
        path_shape_error(
            "failed to open parent directory for",
            &absolute,
            "path has no parent directory",
        )
    })?;
    let file_name = absolute.file_name().ok_or_else(|| {
        path_shape_error(
            "failed to identify file name for",
            &absolute,
            "path does not name a file",
        )
    })?;
    let dir = Dir::open_ambient_dir(parent, ambient_authority()).map_err(|source| {
        MapspliceError::Io {
            action: "failed to open parent directory for",
            path: parent.to_path_buf(),
            source,
        }
    })?;

    Ok(FileCap {
        dir,
        file_name: file_name.to_owned(),
        absolute,
    })
}

/// Convert a possibly relative UTF-8 path into an absolute UTF-8 path.
fn absolutize(path: &Utf8Path) -> Result<Utf8PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    let current_dir = env::current_dir().map_err(|source| MapspliceError::Io {
        action: "failed to read current working directory for",
        path: path.to_path_buf(),
        source,
    })?;
    let utf8_current_dir =
        Utf8PathBuf::from_path_buf(current_dir).map_err(|path_buf| MapspliceError::Io {
            action: "failed to read current working directory for",
            path: path.to_path_buf(),
            source: io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "current working directory is not valid UTF-8: {}",
                    path_buf.display()
                ),
            ),
        })?;

    Ok(utf8_current_dir.join(path))
}

/// Build a per-call temporary sibling filename for an atomic rewrite.
fn temp_file_name(file_name: &str) -> Result<String> {
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|source| MapspliceError::Io {
            action: "failed to create temporary file name for",
            path: Utf8PathBuf::from(file_name),
            source: io::Error::other(source),
        })?;
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    Ok(format!(
        ".{file_name}.mapsplice.tmp.{}.{}.{}",
        std::process::id(),
        since_epoch.as_nanos(),
        counter
    ))
}

/// Construct an I/O error for invalid path shape before filesystem access.
fn path_shape_error(
    action: &'static str,
    path: &Utf8Path,
    message: &'static str,
) -> MapspliceError {
    MapspliceError::Io {
        action,
        path: path.to_path_buf(),
        source: io::Error::new(io::ErrorKind::InvalidInput, message),
    }
}

#[cfg(test)]
#[path = "fs_tests.rs"]
mod tests;
