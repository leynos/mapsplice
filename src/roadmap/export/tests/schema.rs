//! Unit tests for the versioned export schema contract.

use crate::roadmap::export::schema::{ExportDocument, ExportState, FORMAT_VERSION};

#[test]
fn export_state_serializes_with_the_lowercase_vocabulary() {
    assert_eq!(
        serde_json::to_string(&ExportState::Open).expect("state should serialize"),
        "\"open\""
    );
    assert_eq!(
        serde_json::to_string(&ExportState::Done).expect("state should serialize"),
        "\"done\""
    );
    assert_eq!(ExportState::Open.as_str(), "open");
    assert_eq!(ExportState::Done.as_str(), "done");
}

#[test]
fn export_state_rejects_unknown_spellings() {
    let error = serde_json::from_str::<ExportState>("\"blocked\"")
        .expect_err("unknown states are rejected");

    assert!(!error.to_string().is_empty());
}

#[test]
fn export_document_round_trips_through_json() {
    let document = ExportDocument {
        format_version: FORMAT_VERSION,
        phases: Vec::new(),
    };

    let encoded = serde_json::to_string(&document).expect("document should serialize");
    let decoded: ExportDocument =
        serde_json::from_str(&encoded).expect("document should deserialize");

    assert_eq!(decoded, document);
}

#[test]
fn export_document_rejects_unknown_fields() {
    let error = serde_json::from_str::<ExportDocument>(
        r#"{"format_version":1,"phases":[],"unexpected":true}"#,
    )
    .expect_err("unknown fields are rejected");

    assert!(
        error.to_string().contains("unknown field"),
        "unexpected error: {error}"
    );
}

#[test]
fn export_document_requires_the_format_version() {
    let error = serde_json::from_str::<ExportDocument>(r#"{"phases":[]}"#)
        .expect_err("version is required");

    assert!(
        error.to_string().contains("format_version"),
        "unexpected error: {error}"
    );
}
