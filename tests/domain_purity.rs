//! Contract tests for `scripts/check-domain-purity.sh`.
//!
//! The gate keeps the roadmap domain free of infrastructure, so the check that
//! keeps that true needs its own evidence. Each case materializes a small tree
//! with one roadmap source and asserts the checker's exit status, because a
//! gate that never fails is not a gate: the whole reason this script exists is
//! that `env!("CARGO_MANIFEST_DIR")` once reached the domain and nothing
//! deterministic noticed.
//!
//! Failures are reported as `Err` rather than by panicking, because
//! `clippy::panic_in_result_fn` is denied across the workspace.
//!
//! The exemption for test files is asserted too, from both sides: a test file
//! naming `std::process` must be tolerated, and a production file naming it
//! must not be. Without the second case the first would pass for the wrong
//! reason.

#![cfg(unix)]

use std::{error::Error, process::Command};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8::Dir};
use tempfile::TempDir;

/// Test-local result alias, matching the surrounding integration tests.
type TestResult<T = ()> = Result<T, Box<dyn Error>>;

/// The repository root, derived from the manifest directory.
fn manifest_dir() -> Utf8PathBuf { Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")) }

/// Reinterpret a temporary-directory path as UTF-8.
fn utf8(path: &std::path::Path) -> TestResult<&Utf8Path> {
    Utf8Path::from_path(path).ok_or_else(|| "temporary directory path should be UTF-8".into())
}

/// A scratch domain tree, kept alive for the length of one test.
struct Tree {
    _directory: TempDir,
    root: Utf8PathBuf,
}

/// Create a tree holding one roadmap source named `file`.
fn tree_with(file: &str, source: &str) -> TestResult<Tree> {
    let directory = TempDir::new().map_err(|error| format!("create scratch: {error}"))?;
    let root = utf8(directory.path())?.to_path_buf();
    let handle = Dir::open_ambient_dir(&root, ambient_authority())
        .map_err(|error| format!("open scratch {root}: {error}"))?;
    handle
        .create_dir_all("src/roadmap")
        .map_err(|error| format!("create src/roadmap: {error}"))?;
    write_source(&handle, file, source)?;
    Ok(Tree {
        _directory: directory,
        root,
    })
}

/// Write one roadmap source into an open tree.
fn write_source(handle: &Dir, file: &str, source: &str) -> TestResult {
    handle
        .write(format!("src/roadmap/{file}"), source)
        .map_err(|error| format!("write {file}: {error}").into())
}

/// The smallest production source: present, and reaching nothing.
const CLEAN_PRODUCTION: &str = "pub fn identity(text: &str) -> &str { text }\n";

/// Create a tree holding `CLEAN_PRODUCTION` plus one named file.
///
/// The extra file is not decoration. The checker refuses a domain tree whose
/// production-file count is zero, so a tree holding only a test file would be
/// rejected by the anti-vacuity guard rather than by the rule under test, and
/// an exemption case written that way would pass for the wrong reason.
fn tree_with_production_plus(file: &str, source: &str) -> TestResult<Tree> {
    let tree = tree_with("clean.rs", CLEAN_PRODUCTION)?;
    let handle = Dir::open_ambient_dir(&tree.root, ambient_authority())
        .map_err(|error| format!("open scratch {}: {error}", tree.root))?;
    write_source(&handle, file, source)?;
    Ok(tree)
}

/// Run the domain-purity checker against a scratch tree.
///
/// `RG` is cleared so the script resolves ripgrep from `PATH` rather than
/// inheriting an override from the developer's shell.
fn run_purity_check(root: &Utf8Path) -> TestResult<std::process::Output> {
    Command::new(manifest_dir().join("scripts/check-domain-purity.sh"))
        .arg(root)
        .env_remove("RG")
        .output()
        .map_err(|error| format!("execute check-domain-purity.sh: {error}").into())
}

/// Assert the checker rejects a domain source, naming the file it rejected.
fn assert_rejected(file: &str, source: &str) -> TestResult {
    let tree = tree_with(file, source)?;
    let output = run_purity_check(&tree.root)?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    if output.status.success() {
        return Err(format!(
            "a domain source naming infrastructure must be rejected, but the checker passed; \
             source was {source:?}"
        )
        .into());
    }
    if !stderr.contains(file) {
        return Err(format!(
            "rejection must name the offending file {file}; stderr was {stderr:?}"
        )
        .into());
    }
    Ok(())
}

/// Assert the checker accepts a domain source.
fn assert_accepted(file: &str, source: &str) -> TestResult {
    let tree = tree_with(file, source)?;
    let output = run_purity_check(&tree.root)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "a domain source reaching no infrastructure must be accepted; source was {source:?}, \
             stderr was {stderr:?}"
        )
        .into());
    }
    Ok(())
}

#[test]
fn a_domain_source_using_the_filesystem_is_rejected() -> TestResult {
    assert_rejected("render.rs", "use std::fs;\n")
}

#[test]
fn a_grouped_or_aliased_infrastructure_import_is_rejected() -> TestResult {
    // A brace group puts the infrastructure a level further in and an alias
    // hides it again at the call site, so neither the line-anchored `use` shape
    // nor the qualified-path shape matches. The import itself is the violation,
    // whatever local name it is bound to, so the gate must catch it before any
    // alias is resolved.
    assert_rejected("render.rs", "use std::{fs as files};\n")?;
    assert_rejected("render.rs", "use std::{fs};\n")?;
    assert_rejected("render.rs", "use std::{path, fs};\n")?;
    assert_rejected("render.rs", "use std::{io as stream, process as proc};\n")?;
    assert_rejected("render.rs", "pub use std::env;\n")?;
    // The near misses: a domain module may group imports freely, and a name
    // that merely starts with an infrastructure name is not infrastructure.
    assert_accepted("render.rs", "use std::{collections::HashMap};\n")?;
    assert_accepted("render.rs", "use crate::roadmap::{fs_helpers};\n")?;
    assert_accepted("render.rs", "use std::fmt::{self, Display};\n")
}

#[test]
fn the_root_of_an_infrastructure_import_is_not_what_matters() -> TestResult {
    // The offence is reaching the filesystem, not spelling the crate root one
    // way. A rule that only knew `std::` would let the crate-rooted form land.
    assert_rejected("render.rs", "use crate::fs;\n")
}

#[test]
fn a_domain_source_reading_the_build_environment_is_rejected() -> TestResult {
    // The historical defect, in the shape it actually took: locating the proof
    // kernel through the build environment.
    assert_rejected(
        "render.rs",
        "const KERNEL: &str = env!(\"CARGO_MANIFEST_DIR\");\n",
    )?;
    assert_rejected("render.rs", "fn f() { let _ = std::env::args(); }\n")?;
    assert_rejected("render.rs", "fn f() { let _ = current_dir(); }\n")
}

#[test]
fn a_domain_source_reading_a_file_at_build_time_is_rejected() -> TestResult {
    assert_rejected(
        "render.rs",
        "const AGENTS: &str = include_str!(\"AGENTS.md\");\n",
    )
}

#[test]
fn a_domain_source_that_reaches_nothing_is_accepted() -> TestResult {
    assert_accepted(
        "render.rs",
        "use std::fmt;\n\npub fn render(text: &str) -> Result<String, fmt::Error> {\n    let mut \
         out = String::new();\n    out.push_str(text);\n    Ok(out)\n}\n",
    )
}

#[test]
fn a_test_file_may_name_process_because_it_drives_the_binary() -> TestResult {
    // `src/roadmap/render_tests.rs` runs the built binary, so the exemption is
    // load-bearing rather than a convenience. The clean production file beside
    // it is what lets this case exercise the exemption rather than the
    // empty-tree guard.
    let tree = tree_with_production_plus(
        "render_tests.rs",
        "use std::process::Command;\n\nfn run() { let _ = Command::new(\"mapsplice\"); }\n",
    )?;
    let output = run_purity_check(&tree.root)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(
            format!("a test file may drive the built binary; stderr was {stderr:?}").into(),
        );
    }
    Ok(())
}

#[test]
fn a_production_file_may_not_name_process() -> TestResult {
    // The counterpart to the exemption above: the same text in a production
    // file must fail, so the exemption is doing the work rather than the
    // pattern simply not matching.
    assert_rejected(
        "render.rs",
        "use std::process::Command;\n\nfn run() { let _ = Command::new(\"mapsplice\"); }\n",
    )
}

#[test]
fn an_empty_domain_tree_is_not_silently_accepted() -> TestResult {
    // A scan that finds no sources must not read as success, or the gate would
    // pass for a tree that had lost its domain entirely.
    let directory = TempDir::new().map_err(|error| format!("create scratch: {error}"))?;
    let root = utf8(directory.path())?.to_path_buf();
    Dir::open_ambient_dir(&root, ambient_authority())
        .map_err(|error| format!("open scratch {root}: {error}"))?
        .create_dir_all("src/roadmap")
        .map_err(|error| format!("create src/roadmap: {error}"))?;
    let output = run_purity_check(&root)?;
    if output.status.success() {
        return Err("a domain directory holding no sources must not pass".into());
    }
    Ok(())
}

#[test]
fn the_gate_is_wired_into_the_lint_target() -> TestResult {
    // The script is only a gate if `make lint` runs it. Reading the Makefile is
    // the check the target contract already uses for the Verus targets.
    let makefile = Dir::open_ambient_dir(manifest_dir(), ambient_authority())
        .map_err(|error| format!("open repository root: {error}"))?
        .read_to_string("Makefile")
        .map_err(|error| format!("read Makefile: {error}"))?;
    if !makefile.contains("\nlint: check-verification-ledger check-domain-purity") {
        return Err("the domain-purity gate must be a prerequisite of `make lint`".into());
    }
    if !makefile.contains("\ncheck-domain-purity: check-ripgrep") {
        return Err("the domain-purity gate must require ripgrep like its sibling check".into());
    }
    Ok(())
}
