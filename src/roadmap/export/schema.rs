//! Versioned, ordered JSON types for roadmap exports.

use serde::{Deserialize, Serialize};

/// Current roadmap export contract version.
pub(crate) const FORMAT_VERSION: u32 = 1;

/// Supported task checkbox states in the versioned JSON contract.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ExportState {
    /// Checkbox is open.
    Open,
    /// Checkbox is done.
    Done,
}

impl ExportState {
    /// Return the stable lowercase spelling used in JSON and drift reports.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Done => "done",
        }
    }
}

/// Root document emitted by `mapsplice export --format json`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExportDocument {
    /// Version of the JSON field and semantic contract.
    pub(crate) format_version: u32,
    /// Ordered phases in the roadmap.
    pub(crate) phases: Vec<ExportPhase>,
}

/// Phase content and its ordered child steps.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExportPhase {
    /// Dotted phase number.
    pub(crate) number: String,
    /// Normalized phase title.
    pub(crate) title: String,
    /// Labelled or plain paragraphs before the steps.
    pub(crate) entries: Vec<ExportEntry>,
    /// Dependencies found in this phase's title and content.
    pub(crate) requires: Vec<String>,
    /// Ordered steps in this phase.
    pub(crate) steps: Vec<ExportStep>,
    /// Content after the final step.
    pub(crate) trailing: Vec<ExportEntry>,
}

/// Step question, content, and ordered tasks.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExportStep {
    /// Dotted step number.
    pub(crate) number: String,
    /// Normalized step title.
    pub(crate) title: String,
    /// First unlabelled paragraph, if the step has one.
    pub(crate) question: Option<String>,
    /// Other content before the task list.
    pub(crate) entries: Vec<ExportEntry>,
    /// Dependencies found in this step's title and content.
    pub(crate) requires: Vec<String>,
    /// Ordered tasks in this step.
    pub(crate) tasks: Vec<ExportTask>,
    /// Content after the task list.
    pub(crate) trailing: Vec<ExportEntry>,
}

/// Numbered task and its nested checkbox details.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExportTask {
    /// Dotted task number.
    pub(crate) number: String,
    /// Normalized task title.
    pub(crate) title: String,
    /// Checkbox state.
    pub(crate) state: ExportState,
    /// Ordered task-number dependencies.
    pub(crate) requires: Vec<String>,
    /// Labelled and plain task-body entries.
    pub(crate) entries: Vec<ExportEntry>,
    /// Ordered structural and unnumbered checkbox details.
    pub(crate) details: Vec<ExportDetail>,
}

/// Numbered sub-task or unnumbered nested checkbox detail.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExportDetail {
    /// Dotted number for a structural sub-task, or `null` for a nested detail.
    pub(crate) number: Option<String>,
    /// Normalized detail title.
    pub(crate) title: String,
    /// Checkbox state.
    pub(crate) state: ExportState,
    /// Ordered task-number dependencies.
    pub(crate) requires: Vec<String>,
    /// Labelled and plain detail-body entries.
    pub(crate) entries: Vec<ExportEntry>,
    /// Nested checkbox details in source order.
    pub(crate) details: Vec<Self>,
}

/// One normalized paragraph or list item, with optional nested entries.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExportEntry {
    /// Leading label, or `null` for unlabelled content.
    pub(crate) key: Option<String>,
    /// Normalized inline content after the label.
    pub(crate) value: String,
    /// Ordered entries nested beneath an empty labelled paragraph or bullet.
    pub(crate) entries: Vec<Self>,
}
