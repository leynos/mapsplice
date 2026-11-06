//! Unit coverage for the export orchestration seam.

use std::sync::{Mutex, MutexGuard};

use super::*;
use crate::observability::{MetricsSnapshot, metrics_snapshot};

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
fn export_request_emits_json_from_the_injected_reader() {
    let target = Utf8Path::new("target.md");
    let read = |_path: &Utf8Path| Ok(ROADMAP.to_owned());

    let outcome = export_request(target, ExportFormat::Json, None, &read)
        .expect("a readable roadmap should export");

    let stdout = outcome.stdout.expect("export should write to stdout");
    assert!(
        stdout.contains("\"format_version\""),
        "unexpected export: {stdout}"
    );
}

#[test]
fn export_request_propagates_a_source_read_failure() {
    let error = export_request(
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
fn export_request_attributes_a_committed_read_failure_to_the_export() {
    let target = Utf8Path::new("target.md");
    let export = Utf8PathBuf::from("roadmap.json");
    let read = targeted_read(target);

    let error = export_request(target, ExportFormat::Json, Some(&export), &read)
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
fn export_request_attributable_invalid_json_keeps_its_own_error() {
    let target = Utf8Path::new("target.md");
    let export = Utf8PathBuf::from("roadmap.json");
    let read = |path: &Utf8Path| {
        if path == target {
            Ok(ROADMAP.to_owned())
        } else {
            Ok("{ not json".to_owned())
        }
    };

    let error = export_request(target, ExportFormat::Json, Some(&export), &read)
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
fn export_request_rejects_an_unsupported_committed_format_version() {
    let target = Utf8Path::new("target.md");
    let export = Utf8PathBuf::from("roadmap.json");
    // A future version may also change the shape, so this document fails the
    // strict decode and must still name the version rather than a missing field.
    let read = committed_read(
        target,
        "{\"format_version\": 2, \"phases\": [{\"number\": \"1\"}]}\n",
    );

    let error = export_request(target, ExportFormat::Json, Some(&export), &read)
        .expect_err("an unsupported format version should fail");

    let MapspliceError::Export(ExportError::InvalidCommitted { message, .. }) = error else {
        panic!("an unsupported version should be an invalid committed export: {error}");
    };
    assert!(
        message.contains("format_version 2"),
        "the diagnostic should name the unsupported version: {message}"
    );
}

#[test]
fn export_request_reports_a_format_version_change_as_drift() {
    let target = Utf8Path::new("target.md");
    let export = Utf8PathBuf::from("roadmap.json");
    let bumped = export_for(ROADMAP).replace("\"format_version\": 1", "\"format_version\": 2");
    let read = committed_read(target, &bumped);

    let error = export_request(target, ExportFormat::Json, Some(&export), &read)
        .expect_err("a differing format version should drift");

    let MapspliceError::Export(ExportError::Drift { differences, .. }) = error else {
        panic!("a compatible shape should report drift rather than a decode error: {error}");
    };
    assert!(
        differences.contains("format_version"),
        "the drift report should name the version field: {differences}"
    );
}

#[test]
fn export_request_reports_drift_as_an_export_error() {
    let target = Utf8Path::new("target.md");
    let export = Utf8PathBuf::from("roadmap.json");
    let stale = export_for(STALE_ROADMAP);
    let read = committed_read(target, &stale);

    let error = export_request(target, ExportFormat::Json, Some(&export), &read)
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

/// Per-outcome counter deltas observed between two metric snapshots.
#[derive(Debug, Eq, PartialEq)]
struct OutcomeDeltas {
    /// Emitted-export counter delta.
    emitted: u64,
    /// Matched-export counter delta.
    matched: u64,
    /// Drifted-export counter delta.
    drifted: u64,
    /// Failed-export counter delta.
    failed: u64,
}

/// Compute the per-outcome deltas between two snapshots.
fn outcome_deltas(before: &MetricsSnapshot, after: &MetricsSnapshot) -> OutcomeDeltas {
    OutcomeDeltas {
        emitted: after.export_emitted - before.export_emitted,
        matched: after.export_matches - before.export_matches,
        drifted: after.export_drifts - before.export_drifts,
        failed: after.export_failures - before.export_failures,
    }
}

/// Return the deltas expected after recording exactly one outcome.
fn expected_deltas(outcome: ExportOutcome) -> OutcomeDeltas {
    OutcomeDeltas {
        emitted: u64::from(outcome == ExportOutcome::Emitted),
        matched: u64::from(outcome == ExportOutcome::Matched),
        drifted: u64::from(outcome == ExportOutcome::Drifted),
        failed: u64::from(outcome == ExportOutcome::Failed),
    }
}

/// Run one export and assert that only its own outcome counter advanced.
///
/// This catches a misclassified outcome, including a `check` flag inverted at
/// the call site, because an emitted export must not increment the matched
/// counter and vice versa.
fn assert_records_only_outcome<F>(expected: ExportOutcome, export: F)
where
    F: FnOnce() -> Result<RunOutcome>,
{
    let before = metrics_snapshot();
    let _outcome = export();
    let after = metrics_snapshot();

    assert_eq!(
        after.export_attempts,
        before.export_attempts + 1,
        "one export should record exactly one attempt"
    );
    assert_eq!(
        outcome_deltas(&before, &after),
        expected_deltas(expected),
        "only the {expected:?} counter should advance"
    );
}

#[test]
fn counters_record_exactly_one_outcome_per_completed_export() {
    let _guard = export_counter_guard();
    let target = Utf8Path::new("target.md");
    let export = Utf8PathBuf::from("roadmap.json");
    let fresh = export_for(ROADMAP);
    let stale = export_for(STALE_ROADMAP);

    assert_records_only_outcome(ExportOutcome::Emitted, || {
        run_export(target, ExportFormat::Json, None, &targeted_read(target))
    });
    assert_records_only_outcome(ExportOutcome::Matched, || {
        run_export(
            target,
            ExportFormat::Json,
            Some(&export),
            &committed_read(target, &fresh),
        )
    });
    assert_records_only_outcome(ExportOutcome::Drifted, || {
        run_export(
            target,
            ExportFormat::Json,
            Some(&export),
            &committed_read(target, &stale),
        )
    });
    assert_records_only_outcome(ExportOutcome::Failed, || {
        run_export(target, ExportFormat::Json, None, &failing_read)
    });
    assert_records_only_outcome(ExportOutcome::Failed, || {
        run_export(
            target,
            ExportFormat::Json,
            Some(&export),
            &targeted_read(target),
        )
    });
}

#[test]
fn export_request_has_no_command_side_effects() {
    let _guard = export_counter_guard();
    let target = Utf8Path::new("target.md");
    let export = Utf8PathBuf::from("roadmap.json");
    let fresh = export_for(ROADMAP);
    let stale = export_for(STALE_ROADMAP);
    let before = metrics_snapshot();

    export_request(target, ExportFormat::Json, None, &targeted_read(target))
        .expect("a readable roadmap should export");
    export_request(
        target,
        ExportFormat::Json,
        Some(&export),
        &committed_read(target, &fresh),
    )
    .expect("an identical committed export should match");
    export_request(
        target,
        ExportFormat::Json,
        Some(&export),
        &committed_read(target, &stale),
    )
    .expect_err("a stale committed export should drift");
    export_request(target, ExportFormat::Json, None, &failing_read)
        .expect_err("an unreadable roadmap should fail");

    let after = metrics_snapshot();
    assert_eq!(
        after.export_attempts, before.export_attempts,
        "the export query must not record attempts"
    );
    assert_eq!(
        outcome_deltas(&before, &after),
        OutcomeDeltas {
            emitted: 0,
            matched: 0,
            drifted: 0,
            failed: 0,
        },
        "the export query must not record outcomes"
    );
}
