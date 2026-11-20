//! In-place completion of one task or addendum sub-task.
//!
//! `mapsplice complete` checks off a single roadmap item without a fragment.
//! Unlike the splice commands, it does not rebuild the document from the
//! model: every byte outside the addressed item's checkbox and inserted suffix
//! must survive, including title wrapping, nested bullets, `Requires` clauses,
//! and numbering. The command therefore patches the original text and treats
//! the parsed model only as a validated index into that text.
//!
//! # Markdown position facts
//!
//! The patch depends on the byte offsets `markdown` 1.0.0 reports, which were
//! confirmed against every shape the command accepts:
//!
//! - Offsets are bytes, not characters: a multibyte title keeps correct byte offsets while its
//!   columns count characters.
//! - A `ListItem` starts at its bullet marker; its opening `Paragraph` starts at the numbered
//!   prefix, after the checkbox, so `[ ]` always lies between the two.
//! - A paragraph end covers lazy continuation lines and excludes the trailing line ending; nested
//!   bullets are separate nodes after the paragraph.
//! - CRLF and bare-CR documents report the same relative structure, so the patch never has to
//!   normalize line endings.
//!
//! Suffix placement, existing-suffix detection, and formatter-stable wrapping
//! live in the sibling `complete_suffix` module.
//!
//! # Safety
//!
//! Every offset is checked against the source and the parser's checkbox state
//! before a byte is replaced. The patched text is reparsed and rejected with
//! [`MapspliceError::UnsafeCompletionEdit`] unless the anchor set is unchanged
//! and the addressed item is checked with its suffix at the title end.

use std::collections::BTreeSet;

use super::{
    super::{
        RoadmapAnchor,
        RoadmapDocument,
        SubTaskEntry,
        SubTaskNumber,
        TaskNumber,
        model::{ItemSourceSpan, MarkdownNodes, StepSection, TaskEntry},
        parse_roadmap,
    },
    complete_suffix::{insert_suffix, normalize_suffix, title_has_suffix},
};
use crate::error::{MapspliceError, Result};

#[cfg(test)]
#[path = "complete_property_tests.rs"]
mod property_tests;
#[cfg(test)]
#[path = "complete_suffix_tests.rs"]
mod suffix_tests;
#[cfg(test)]
#[path = "complete_test_support.rs"]
mod test_support;
#[cfg(test)]
#[path = "complete_tests.rs"]
mod tests;

/// Mark one task or addendum sub-task as done in the original document text.
///
/// `roadmap` must be the parse of `source`. The returned text is byte-identical
/// to `source` outside the addressed item's checkbox marker and, when a suffix
/// is configured, the title's final line. Completing an already checked item
/// returns `source` unchanged whatever suffix is configured: repeated runs are
/// no-ops, and a different suffix can never be appended a second time.
///
/// # Errors
///
/// Returns [`MapspliceError::UnsupportedAnchorLevel`] for phase and step
/// anchors, [`MapspliceError::AnchorNotFound`] when the anchor is absent, and
/// [`MapspliceError::UnsafeCompletionEdit`] when the retained span, the
/// checkbox marker, or the reparsed patch disagrees with the request.
///
/// # Examples
///
/// ```ignore
/// let roadmap = parse_roadmap(source)?;
/// let anchor = parse_anchor("1.3.1")?;
/// let completed = complete_item(&roadmap, source, anchor, Some("— done"))?;
/// assert!(completed.contains("- [x] 1.3.1."));
/// ```
pub(crate) fn complete_item(
    roadmap: &RoadmapDocument,
    source: &str,
    anchor: RoadmapAnchor,
    suffix: Option<&str>,
) -> Result<String> {
    let target = find_target(roadmap, anchor)?;
    let span = target
        .span
        .ok_or_else(|| unsafe_edit(anchor, "the item has no retained source span"))?;
    if !span.is_valid_in(source) {
        return Err(unsafe_edit(
            anchor,
            "the retained source span does not fit the target text",
        ));
    }
    let checked = target
        .checked
        .ok_or_else(|| unsafe_edit(anchor, "the item has no parsed checkbox state"))?;
    let marker = checkbox_marker_offset(source, span, checked, anchor)?;
    if checked {
        // Completing a checked item is a no-op even when a suffix is configured:
        // re-running with a different suffix must never append a second one.
        return Ok(source.to_owned());
    }
    let normalized_suffix = normalize_suffix(suffix);
    let has_suffix = normalized_suffix
        .as_deref()
        .is_some_and(|text| title_has_suffix(target.summary, text));
    let suffix_to_insert = normalized_suffix.as_deref().filter(|_| !has_suffix);

    let mut patched = source.to_owned();
    patched.replace_range(marker..=marker, "x");
    if let Some(text) = suffix_to_insert {
        insert_suffix(&mut patched, source, span, text)
            .ok_or_else(|| unsafe_edit(anchor, "the title span cannot hold the done suffix"))?;
    }
    validate_patch(&patched, roadmap, anchor, normalized_suffix.as_deref())?;
    Ok(patched)
}

/// Parsed facts needed to complete one addressed item.
struct CompletionTarget<'roadmap> {
    /// Checked state recorded by the parser.
    checked: Option<bool>,
    /// Retained byte span in the target source.
    span: Option<ItemSourceSpan>,
    /// Parsed title nodes, used for rendering-independent suffix detection.
    summary: &'roadmap MarkdownNodes,
}

/// Resolve the addressed task or sub-task, rejecting anchors without checkboxes.
fn find_target(roadmap: &RoadmapDocument, anchor: RoadmapAnchor) -> Result<CompletionTarget<'_>> {
    match anchor {
        RoadmapAnchor::Task(target) => find_task(roadmap, target).map(|task| CompletionTarget {
            checked: task.checked(),
            span: task.source_span(),
            summary: &task.summary,
        }),
        RoadmapAnchor::SubTask(target) => {
            find_sub_task(roadmap, target).map(|sub_task| CompletionTarget {
                checked: sub_task.checked,
                span: sub_task.source_span(),
                summary: &sub_task.summary,
            })
        }
        RoadmapAnchor::Phase(_) | RoadmapAnchor::Step(_) => {
            Err(MapspliceError::UnsupportedAnchorLevel {
                anchor,
                level: anchor.level(),
            })
        }
    }
}

/// Iterate over every task in document order.
fn tasks(roadmap: &RoadmapDocument) -> impl Iterator<Item = &TaskEntry> {
    roadmap
        .phases
        .iter()
        .flat_map(|phase| phase.steps.iter())
        .flat_map(StepSection::tasks)
}

/// Find a task by its rendered number.
fn find_task(roadmap: &RoadmapDocument, target: TaskNumber) -> Result<&TaskEntry> {
    tasks(roadmap)
        .find(|task| task.number() == target)
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.into(),
        })
}

/// Find a sub-task by its rendered number beneath its parent task.
///
/// A missing parent is reported as the requested sub-task anchor, so the
/// diagnostic names the anchor the caller asked for rather than its prefix.
fn find_sub_task(roadmap: &RoadmapDocument, target: SubTaskNumber) -> Result<&SubTaskEntry> {
    let task = tasks(roadmap)
        .find(|task| task.number() == target.task_number())
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.into(),
        })?;
    task.sub_tasks()
        .iter()
        .find(|sub_task| sub_task.number == target)
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.into(),
        })
}

/// Locate the checkbox state byte and confirm it matches the parsed state.
///
/// Only the region between the bullet and the numbered title is examined, so a
/// `[ ]`-like sequence inside the title cannot be mistaken for the checkbox.
fn checkbox_marker_offset(
    source: &str,
    span: ItemSourceSpan,
    checked: bool,
    anchor: RoadmapAnchor,
) -> Result<usize> {
    let region = source
        .get(span.item_start..span.title_start)
        .ok_or_else(|| unsafe_edit(anchor, "the item marker lies outside the target text"))?;
    let bracket = span.item_start
        + region
            .find('[')
            .ok_or_else(|| unsafe_edit(anchor, "the item has no checkbox marker"))?;
    let state_offset = bracket + 1;
    let state = source.as_bytes().get(state_offset).copied();
    let closing = source.as_bytes().get(state_offset + 1).copied();
    let agrees = closing == Some(b']')
        && match state {
            // The Markdown parser accepts a space or a tab as an unchecked
            // marker, so both byte values must agree with the parsed state.
            Some(b' ' | b'\t') => !checked,
            Some(b'x' | b'X') => checked,
            _ => false,
        };
    if agrees {
        Ok(state_offset)
    } else {
        Err(unsafe_edit(
            anchor,
            "the checkbox marker disagrees with the parsed checked state",
        ))
    }
}

/// Reparse the patched text and confirm the edit achieved exactly its intent.
fn validate_patch(
    patched: &str,
    roadmap: &RoadmapDocument,
    anchor: RoadmapAnchor,
    suffix: Option<&str>,
) -> Result<()> {
    let reparsed = parse_roadmap(patched).map_err(|error| {
        unsafe_edit(
            anchor,
            format!("the patched document no longer parses: {error}"),
        )
    })?;
    if collect_anchors(roadmap) != collect_anchors(&reparsed) {
        return Err(unsafe_edit(
            anchor,
            "the patch changed the roadmap's anchor set",
        ));
    }
    let target = find_target(&reparsed, anchor)
        .map_err(|_| unsafe_edit(anchor, "the patch lost the addressed item"))?;
    if target.checked != Some(true) {
        return Err(unsafe_edit(
            anchor,
            "the patch did not mark the item as checked",
        ));
    }
    if let Some(expected) = suffix
        && !title_has_suffix(target.summary, expected)
    {
        return Err(unsafe_edit(
            anchor,
            "the patch did not place the done suffix at the title end",
        ));
    }
    Ok(())
}

/// Collect every item anchor in the document.
fn collect_anchors(roadmap: &RoadmapDocument) -> BTreeSet<RoadmapAnchor> {
    let mut anchors = BTreeSet::new();
    for phase in &roadmap.phases {
        anchors.insert(phase.number.into());
        for step in &phase.steps {
            anchors.insert(step.number.into());
            collect_task_anchors(step, &mut anchors);
        }
    }
    anchors
}

/// Add every task and addendum sub-task anchor in one step.
fn collect_task_anchors(step: &StepSection, anchors: &mut BTreeSet<RoadmapAnchor>) {
    for task in step.tasks() {
        anchors.insert(task.number().into());
        for sub_task in task.sub_tasks() {
            anchors.insert(sub_task.number.into());
        }
    }
}

/// Build the fail-closed error for an edit that cannot be applied safely.
fn unsafe_edit(anchor: RoadmapAnchor, reason: impl Into<String>) -> MapspliceError {
    MapspliceError::UnsafeCompletionEdit {
        anchor,
        reason: reason.into(),
    }
}
