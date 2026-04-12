//! Filesystem helpers that keep ambient access at the CLI edge.

use std::env;

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8::Dir};

use crate::error::{MapspliceError, Result};

/// Read a UTF-8 file through a capability directory.
pub fn read_utf8(path: &Utf8Path) -> Result<String> {
    let (dir, file_name, absolute) = open_parent_dir(path)?;
    dir.read_to_string(&file_name)
        .map_err(|source| MapspliceError::Io {
            action: "failed to read",
            path: absolute,
            source,
        })
}

/// Rewrite a UTF-8 file through a temporary sibling file and rename.
pub fn rewrite_utf8(path: &Utf8Path, contents: &str) -> Result<()> {
    let (dir, file_name, absolute) = open_parent_dir(path)?;
    let temp_name = format!(".{}.mapsplice.tmp.{}", file_name, std::process::id());

    dir.write(&temp_name, contents)
        .map_err(|source| MapspliceError::Io {
            action: "failed to write temporary file for",
            path: absolute.clone(),
            source,
        })?;

    dir.rename(&temp_name, &dir, &file_name)
        .map_err(|source| MapspliceError::Io {
            action: "failed to replace",
            path: absolute,
            source,
        })
}

fn open_parent_dir(path: &Utf8Path) -> Result<(Dir, String, Utf8PathBuf)> {
    let absolute = absolutize(path)?;
    let parent = absolute
        .parent()
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: format!("path `{absolute}` has no parent directory"),
        })?;
    let file_name = absolute
        .file_name()
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: format!("path `{absolute}` does not name a file"),
        })?;
    let dir = Dir::open_ambient_dir(parent, ambient_authority()).map_err(|source| {
        MapspliceError::Io {
            action: "failed to open parent directory for",
            path: parent.to_path_buf(),
            source,
        }
    })?;

    Ok((dir, file_name.to_owned(), absolute))
}

fn absolutize(path: &Utf8Path) -> Result<Utf8PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    let current_dir = env::current_dir().map_err(|source| MapspliceError::Io {
        action: "failed to read current working directory for",
        path: path.to_path_buf(),
        source,
    })?;
    let utf8_current_dir = Utf8PathBuf::from_path_buf(current_dir).map_err(|path_buf| {
        MapspliceError::InvalidRoadmap {
            message: format!(
                "current working directory is not valid UTF-8: {}",
                path_buf.display()
            ),
        }
    })?;

    Ok(utf8_current_dir.join(path))
}
