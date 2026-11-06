//! Unit coverage for the export orchestration seam.

use std::sync::{Mutex, MutexGuard};

use super::*;
use crate::observability::metrics_snapshot;

/// Minimal roadmap accepted by the parser and export adapter.
const ROADMAP: &str = concat!(
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. Task one.\n",
);

/// Roadmap differing from [`ROADMAP`] only in its phase title.
///
/// A committed export of this document is valid JSON that differs
/// semantically from the fresh export, which is what drift requires.
const STALE_ROADMAP: &str = concat!(
    "## 1. Phase two\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. Task one.\n",
);

/// Export counters are process-global, so counter tests run one at a time.
static EXPORT_COUNTER_LOCK: Mutex<()> = Mutex::new(());

/// Serialize a test that asserts on process-global export counters.
fn export_counter_guard() -> MutexGuard<'static, ()> {
    EXPORT_COUNTER_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Read a roadmap that reports the supplied failure for any path.
fn failing_read(path: &Utf8Path) -> Result<String> {
    Err(MapspliceError::Io {
        action: "failed to read",
        path: path.to_owned(),
        source: std::io::Error::other("unreadable"),
    })
}

/// Read the fixture roadmap for the target and fail for every other path.
fn targeted_read(target: &Utf8Path) -> impl Fn(&Utf8Path) -> Result<String> + '_ {
    move |path: &Utf8Path| {
        if path == target {
            Ok(ROADMAP.to_owned())
        } else {
            failing_read(path)
        }
    }
}

/// Serialize the export for a roadmap fixture.
fn export_for(roadmap: &str) -> String {
    let schema = roadmap::export::to_schema(&parse_roadmap(roadmap).expect("fixture should parse"))
        .expect("fixture should adapt");
    roadmap::export::serialize(&schema).expect("fixture should serialize")
}

/// Read the fixture roadmap for the target and a supplied export otherwise.
fn committed_read<'reader>(
    target: &'reader Utf8Path,
    committed: &'reader str,
) -> impl Fn(&Utf8Path) -> Result<String> + 'reader {
    move |path: &Utf8Path| {
        if path == target {
            Ok(ROADMAP.to_owned())
        } else {
            Ok(committed.to_owned())
        }
    }
}

#[test]
fn run_export_emits_json_from_the_injected_reader() {
    let target = Utf8Path::new("target.md");
    let read = |_path: &Utf8Path| Ok(ROADMAP.to_owned());

    let outcome = run_export(target, ExportFormat::Json, None, &read)
        .expect("a readable roadmap should export");

    let stdout = outcome.stdout.expect("export should write to stdout");
    assert!(
        stdout.contains("\"format_version\""),
        "unexpected export: {stdout}"
    );
}

#[test]
fn run_export_propagates_a_source_read_failure() {
    let error = run_export(
        Utf8Path::new("target.md"),
        ExportFormat::Json,
        None,
        &failing_read,
    )
    .expect_err("an unreadable roadmap should fail");

    assert!(
        matches!(error, MapspliceError::Io { .. }),
        "a source read failure should surface unchanged: {error}"
    );
}

#[test]
fn run_export_attributes_a_committed_read_failure_to_the_export() {
    let target = Utf8Path::new("target.md");
    let export = Utf8PathBuf::from("roadmap.json");
    let read = targeted_read(target);

    let error = run_export(target, ExportFormat::Json, Some(&export), &read)
        .expect_err("an unreadable committed export should fail");

    assert!(
        matches!(
            error,
            MapspliceError::Export(ExportError::InvalidCommitted { .. })
        ),
        "a committed read failure should be attributed to the export: {error}"
    );
}

#[test]
fn run_export_attributable_invalid_json_keeps_its_own_error() {
    let target = Utf8Path::new("target.md");
    let export = Utf8PathBuf::from("roadmap.json");
    let read = |path: &Utf8Path| {
        if path == target {
            Ok(ROADMAP.to_owned())
        } else {
            Ok("{ not json".to_owned())
        }
    };

    let error = run_export(target, ExportFormat::Json, Some(&export), &read)
        .expect_err("malformed committed JSON should fail");

    assert!(
        matches!(
            error,
            MapspliceError::Export(ExportError::InvalidCommitted { .. })
        ),
        "malformed JSON should be attributed to the export: {error}"
    );
}

#[test]
fn run_export_reports_drift_as_an_export_error() {
    let target = Utf8Path::new("target.md");
    let export = Utf8PathBuf::from("roadmap.json");
    let stale = export_for(STALE_ROADMAP);
    let read = committed_read(target, &stale);

    let error = run_export(target, ExportFormat::Json, Some(&export), &read)
        .expect_err("a differing committed export should drift");

    assert!(
        matches!(error, MapspliceError::Export(ExportError::Drift { .. })),
        "a stale export should report drift: {error}"
    );
}

#[test]
fn decision_for_classifies_every_completed_export() {
    let emitted = decision_for(&Ok(RunOutcome::stdout(String::new())), false);
    let matched = decision_for(&Ok(RunOutcome::silent_success()), true);
    let drifted = decision_for(
        &Err(ExportError::Drift {
            export: Utf8PathBuf::from("roadmap.json"),
            roadmap: Utf8PathBuf::from("roadmap.md"),
            differences: String::new(),
            regenerate: String::new(),
        }
        .into()),
        true,
    );
    let source_read_error =
        failing_read(Utf8Path::new("roadmap.md")).expect_err("the fixture reader always fails");
    let source_read = decision_for(&Err(source_read_error), false);
    let committed_read = decision_for(
        &Err(ExportError::InvalidCommitted {
            path: Utf8PathBuf::from("roadmap.json"),
            message: "denied".to_owned(),
        }
        .into()),
        true,
    );
    let invalid_export = decision_for(
        &Err(ExportError::InvalidCommitted {
            path: Utf8PathBuf::from("roadmap.json"),
            message: "expected value".to_owned(),
        }
        .into()),
        true,
    );

    assert_eq!(emitted, ExportOutcome::Emitted);
    assert_eq!(matched, ExportOutcome::Matched);
    assert_eq!(drifted, ExportOutcome::Drifted);
    assert_eq!(source_read, ExportOutcome::Failed);
    assert_eq!(committed_read, ExportOutcome::Failed);
    assert_eq!(invalid_export, ExportOutcome::Failed);
}

#[test]
fn counters_record_exactly_one_outcome_per_completed_export() {
    let _guard = export_counter_guard();
    let before = metrics_snapshot();
    let target = Utf8Path::new("target.md");
    let export = Utf8PathBuf::from("roadmap.json");
    let fresh = export_for(ROADMAP);
    let stale = export_for(STALE_ROADMAP);

    // Emitted: no committed comparison.
    run_export(target, ExportFormat::Json, None, &targeted_read(target))
        .expect("a readable roadmap should export");
    // Matched: the committed bytes equal the fresh export.
    run_export(
        target,
        ExportFormat::Json,
        Some(&export),
        &committed_read(target, &fresh),
    )
    .expect("an identical committed export should match");
    // Drifted: the committed bytes differ from the fresh export.
    run_export(
        target,
        ExportFormat::Json,
        Some(&export),
        &committed_read(target, &stale),
    )
    .expect_err("a stale committed export should drift");
    // Failed: the source cannot be read.
    run_export(target, ExportFormat::Json, None, &failing_read)
        .expect_err("an unreadable roadmap should fail");
    // Failed: the committed export cannot be read.
    run_export(
        target,
        ExportFormat::Json,
        Some(&export),
        &targeted_read(target),
    )
    .expect_err("an unreadable committed export should fail");

    let after = metrics_snapshot();
    assert_eq!(after.export_attempts, before.export_attempts + 5);
    assert_eq!(after.export_emitted, before.export_emitted + 1);
    assert_eq!(after.export_matches, before.export_matches + 1);
    assert_eq!(after.export_drifts, before.export_drifts + 1);
    assert_eq!(after.export_failures, before.export_failures + 2);
}
