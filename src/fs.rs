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
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);

    let mut temp =
        cap.dir
            .open_with(&temp_name, &options)
            .map_err(|source| MapspliceError::Io {
                action: "failed to create temporary file for",
                path: cap.absolute.clone(),
                source,
            })?;
    temp.write_all(contents.as_bytes())
        .map_err(|source| MapspliceError::Io {
            action: "failed to write temporary file for",
            path: cap.absolute.clone(),
            source,
        })?;
    drop(temp);

    cap.dir
        .rename(&temp_name, &cap.dir, &cap.file_name)
        .map_err(|source| MapspliceError::Io {
            action: "failed to replace",
            path: cap.absolute,
            source,
        })
}

struct FileCap {
    dir: Dir,
    file_name: String,
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
mod tests {
    //! Concurrency coverage for temporary-file name generation.

    use std::{collections::BTreeSet, thread};

    use super::temp_file_name;

    #[test]
    fn temporary_names_are_unique_under_concurrent_calls() {
        let handles = (0..16)
            .map(|_| thread::spawn(|| temp_file_name("target.md")))
            .collect::<Vec<_>>();
        let mut names = BTreeSet::new();

        for handle in handles {
            let name = handle
                .join()
                .expect("temporary-name worker should finish")
                .expect("temporary name should be generated");
            assert!(names.insert(name), "temporary name should be unique");
        }
    }
}
