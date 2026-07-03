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

struct RewriteStrategy<OpenTemp, ReplaceTarget> {
    open_temp: OpenTemp,
    replace_target: ReplaceTarget,
}

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

    use std::{
        collections::BTreeSet,
        io::{self, Write},
        thread,
    };

    use camino::{Utf8Path, Utf8PathBuf};
    use cap_std::ambient_authority;

    use super::{
        Dir,
        MapspliceError,
        OpenOptions,
        RewriteStrategy,
        open_parent_dir,
        rewrite_utf8_with_strategy,
        temp_file_name,
    };

    const ORIGINAL_CONTENTS: &str = "original roadmap\n";
    const REPLACEMENT_CONTENTS: &str = "replacement roadmap\n";

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("injected write failure"))
        }

        fn flush(&mut self) -> io::Result<()> { Ok(()) }
    }

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

    #[test]
    fn write_failure_removes_temporary_sibling() {
        let tempdir = tempfile::tempdir().expect("temporary directory should be created");
        let target = target_path(&tempdir);
        let test_dir = test_dir(&tempdir).expect("temporary directory should open");
        test_dir
            .write("target.md", ORIGINAL_CONTENTS)
            .expect("target should be seeded");
        let cap = open_parent_dir(&target).expect("target parent should open");
        let temp_name = ".target.md.mapsplice.tmp.write-failure-test";

        let error = rewrite_utf8_with_strategy(
            &cap,
            temp_name,
            REPLACEMENT_CONTENTS,
            RewriteStrategy {
                open_temp: |dir: &Dir, name: &str, options: &OpenOptions| {
                    let _temp = dir
                        .open_with(name, options)
                        .expect("temporary sibling should be created");
                    Ok(FailingWriter)
                },
                replace_target: unexpected_replace_after_write_failure,
            },
        )
        .expect_err("injected write failure should be returned");

        io_source(&error, "failed to write temporary file for");
        assert_eq!(
            test_dir
                .read_to_string("target.md")
                .expect("target should remain readable"),
            ORIGINAL_CONTENTS,
            "target contents should be unchanged"
        );
        assert_no_temporary_siblings(&cap, "write failure");
    }

    #[test]
    fn rename_failure_removes_temporary_sibling() {
        let tempdir = tempfile::tempdir().expect("temporary directory should be created");
        let target = target_path(&tempdir);
        let test_dir = test_dir(&tempdir).expect("temporary directory should open");
        test_dir
            .write("target.md", ORIGINAL_CONTENTS)
            .expect("target should be seeded");
        let cap = open_parent_dir(&target).expect("target parent should open");
        let temp_name = ".target.md.mapsplice.tmp.rename-failure-test";

        let error = rewrite_utf8_with_strategy(
            &cap,
            temp_name,
            REPLACEMENT_CONTENTS,
            RewriteStrategy {
                open_temp: |dir: &Dir, name: &str, options: &OpenOptions| {
                    dir.open_with(name, options)
                },
                replace_target: failing_replace,
            },
        )
        .expect_err("injected rename failure should be returned");

        assert_eq!(
            io_source(&error, "failed to replace").kind(),
            io::ErrorKind::PermissionDenied
        );
        let target_contents = test_dir
            .read_to_string("target.md")
            .expect("target should remain readable");
        assert_eq!(
            target_contents, ORIGINAL_CONTENTS,
            "target contents should be unchanged"
        );
        assert_ne!(
            target_contents, REPLACEMENT_CONTENTS,
            "replacement contents should not reach target after rename failure"
        );
        assert_no_temporary_siblings(&cap, "rename failure");
    }

    fn target_path(tempdir: &tempfile::TempDir) -> Utf8PathBuf {
        let Some(path) = Utf8Path::from_path(tempdir.path()) else {
            panic!("temporary path should be valid UTF-8");
        };
        path.join("target.md")
    }

    fn test_dir(tempdir: &tempfile::TempDir) -> io::Result<Dir> {
        let path = Utf8Path::from_path(tempdir.path()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "temporary path should be valid UTF-8",
            )
        })?;
        Dir::open_ambient_dir(path, ambient_authority())
    }

    fn io_source<'error>(
        error: &'error MapspliceError,
        expected_action: &'static str,
    ) -> &'error io::Error {
        match error {
            MapspliceError::Io { action, source, .. } => {
                assert_eq!(*action, expected_action);
                source
            }
            other => panic!("expected I/O error, got {other:?}"),
        }
    }

    fn failing_replace(_dir: &Dir, _temp_name: &str, _target_name: &str) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::PermissionDenied))
    }

    fn unexpected_replace_after_write_failure(
        _dir: &Dir,
        _temp_name: &str,
        _target_name: &str,
    ) -> io::Result<()> {
        Err(io::Error::other(
            "replace should not run after write failure",
        ))
    }

    fn assert_no_temporary_siblings(cap: &super::FileCap, failure: &str) {
        let temporary_siblings = match temporary_siblings(cap) {
            Ok(names) => names,
            Err(error) => panic!("temporary siblings should be queried: {error}"),
        };
        assert!(
            temporary_siblings.is_empty(),
            "temporary sibling should be removed after {failure}"
        );
    }

    fn temporary_siblings(cap: &super::FileCap) -> io::Result<Vec<String>> {
        cap.dir.entries()?.try_fold(Vec::new(), |mut names, entry| {
            let name = entry?.file_name()?;
            if name.contains(".mapsplice.tmp") {
                names.push(name);
            }
            Ok(names)
        })
    }
}
