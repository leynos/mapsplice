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
            open_temp: |dir: &Dir, name: &str, options: &OpenOptions| dir.open_with(name, options),
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
