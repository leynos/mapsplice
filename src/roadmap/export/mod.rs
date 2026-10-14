//! Pure adapter from the roadmap model to its versioned JSON contract.

mod blocks;
mod details;
pub(crate) mod drift;
mod requires;
mod schema;
mod text;

use std::collections::HashSet;

use markdown::mdast::{ListItem, Node};
pub(crate) use schema::ExportDocument as RoadmapExport;

use self::schema::{
    ExportDocument,
    ExportPhase,
    ExportState,
    ExportStep,
    ExportTask,
    FORMAT_VERSION,
};
use super::{
    RoadmapAnchor,
    RoadmapDocument,
    model::{StepSection, TaskEntry},
};
use crate::error::{MapspliceError, Result, SourceLocation};

/// Serialize one already validated export schema in its stable byte form.
pub(crate) fn serialize(document: &ExportDocument) -> Result<String> {
    let mut output = serde_json::to_string_pretty(document).map_err(|error| {
        MapspliceError::JsonSerialization {
            message: error.to_string(),
        }
    })?;
    output.push('\n');
    Ok(output)
}

/// Adapt roadmap model values to the explicit schema without performing I/O.
pub(crate) fn to_schema(roadmap: &RoadmapDocument) -> Result<ExportDocument> {
    let known_anchors = known_anchor_numbers(roadmap);
    let phases = roadmap
        .phases
        .iter()
        .map(|phase| {
            let content = [&phase.body, &phase.trailing];
            Ok(ExportPhase {
                number: phase.number.to_string(),
                title: text::render_text(phase.title.nodes())?,
                entries: blocks::entries(&phase.body)?,
                requires: requires::from_markdown(&phase.title, &content, &known_anchors)?,
                steps: phase
                    .steps
                    .iter()
                    .map(|step| export_step(step, &known_anchors))
                    .collect::<Result<Vec<_>>>()?,
                trailing: blocks::entries(&phase.trailing)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(ExportDocument {
        format_version: FORMAT_VERSION,
        phases,
    })
}

/// Adapt one step with its first unlabelled paragraph separated as a question.
fn export_step(step: &StepSection, known_anchors: &HashSet<String>) -> Result<ExportStep> {
    let content = [&step.body, &step.trailing];
    let (question, entries) = blocks::step_content(&step.body)?;
    Ok(ExportStep {
        number: step.number.to_string(),
        title: text::render_text(step.title.nodes())?,
        question,
        entries,
        requires: requires::from_markdown(&step.title, &content, known_anchors)?,
        tasks: step
            .tasks()
            .iter()
            .map(|task| export_task(task, known_anchors))
            .collect::<Result<Vec<_>>>()?,
        trailing: blocks::entries(&step.trailing)?,
    })
}

/// Adapt one checked task and its source-ordered body content.
fn export_task(task: &TaskEntry, known_anchors: &HashSet<String>) -> Result<ExportTask> {
    let location = task
        .summary
        .nodes()
        .first()
        .and_then(Node::position)
        .map(|position| SourceLocation::from(&position.start));
    let state = task.checked.map_or_else(
        || {
            Err(MapspliceError::InvalidRoadmap {
                message: format!("task `{}` has no checkbox state", task.number()),
                location,
            })
        },
        |is_done| Ok(export_state(is_done)),
    )?;
    let (entries, details) = blocks::task_content(task, known_anchors)?;
    Ok(ExportTask {
        number: task.number().to_string(),
        title: text::render_text(task.summary.nodes())?,
        state,
        requires: requires::from_task(task, known_anchors)?,
        entries,
        details,
    })
}

/// Map a parser checkbox marker to its closed JSON state vocabulary.
const fn export_state(is_done: bool) -> ExportState {
    if is_done {
        ExportState::Done
    } else {
        ExportState::Open
    }
}

/// Gather all numbered roadmap items that may be dependency endpoints.
fn known_anchor_numbers(roadmap: &RoadmapDocument) -> HashSet<String> {
    let mut anchors = HashSet::new();
    for phase in &roadmap.phases {
        anchors.insert(phase.number.to_string());
        for step in &phase.steps {
            anchors.insert(step.number.to_string());
            for task in step.tasks() {
                add_task_anchor_numbers(task, &mut anchors);
            }
        }
    }
    anchors
}

/// Add anchors from parser-owned and mixed-list sub-tasks beneath one task.
fn add_task_anchor_numbers(task: &TaskEntry, anchors: &mut HashSet<String>) {
    anchors.insert(task.number().to_string());
    anchors.extend(
        task.sub_tasks()
            .iter()
            .map(|sub_task| sub_task.number.to_string()),
    );
    for child in task.children() {
        if let super::model::TaskChild::Body(body) = child {
            add_mixed_sub_task_anchors(body.nodes(), anchors);
        }
    }
}

/// Add numbered checkbox anchors that appear in mixed prose and sub-task lists.
fn add_mixed_sub_task_anchors(nodes: &[Node], anchors: &mut HashSet<String>) {
    for list in nodes.iter().filter_map(|node| match node {
        Node::List(list) => Some(list),
        _ => None,
    }) {
        for item in list.children.iter().filter_map(|child| match child {
            Node::ListItem(item) => Some(item),
            _ => None,
        }) {
            add_checkbox_sub_task_anchor(item, anchors);
            add_mixed_sub_task_anchors(&item.children, anchors);
        }
    }
}

/// Add a checked item's valid numbered sub-task prefix, when it has one.
fn add_checkbox_sub_task_anchor(item: &ListItem, anchors: &mut HashSet<String>) {
    if item.checked.is_none() {
        return;
    }
    let Some(Node::Paragraph(paragraph)) = item.children.first() else {
        return;
    };
    let Some(Node::Text(text)) = paragraph.children.first() else {
        return;
    };
    let Some((prefix, _)) = text.value.split_once(". ") else {
        return;
    };
    if let Ok(RoadmapAnchor::SubTask(number)) = super::parse_anchor(prefix) {
        anchors.insert(number.to_string());
    }
}
