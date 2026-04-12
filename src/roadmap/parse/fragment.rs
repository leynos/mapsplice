//! Fragment-level roadmap parsing.

use markdown::mdast::{Heading, List, Node, Root};

use super::{
    document::parse_document_root,
    is_phase_heading,
    looks_like_task_list,
    parse_root,
    parse_step_heading,
    parse_task_list,
};
use crate::{
    error::{MapspliceError, Result},
    roadmap::{
        RoadmapDocument,
        RoadmapFragment,
        model::{PhaseSection, SourceId, StepSection},
    },
};

/// Parse a fragment file.
///
/// # Errors
///
/// Returns an error when the Markdown cannot be parsed or when the fragment
/// does not cleanly represent one or more sibling phases, steps, or tasks.
pub fn parse_fragment(markdown: &str) -> Result<RoadmapFragment> {
    let root = parse_root(markdown)?;
    let first = root
        .children
        .first()
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: "fragment file is empty".to_owned(),
        })?;

    if is_phase_fragment_start(first) {
        return parse_phase_fragment(root);
    }

    if let Some(heading) = step_fragment_heading(first) {
        return parse_step_fragment(markdown, heading);
    }

    if let Some(list) = task_fragment_list(first) {
        return parse_task_fragment(markdown, list);
    }

    Err(MapspliceError::InvalidRoadmap {
        message: "fragment must start with a phase heading, step heading, or numbered task list"
            .to_owned(),
    })
}

fn is_phase_fragment_start(node: &Node) -> bool {
    matches!(node, Node::Heading(heading) if is_phase_heading(heading))
}

fn is_step_fragment_start(node: &Node) -> bool {
    matches!(node, Node::Heading(heading) if heading.depth == 3)
        && matches!(node, Node::Heading(heading) if parse_step_heading(heading).is_ok())
}

fn is_task_fragment_start(node: &Node) -> bool {
    matches!(node, Node::List(list) if looks_like_task_list(list))
}

fn step_fragment_heading(node: &Node) -> Option<&Heading> {
    match node {
        Node::Heading(heading) if is_step_fragment_start(node) => Some(heading),
        _ => None,
    }
}

fn task_fragment_list(node: &Node) -> Option<&List> {
    match node {
        Node::List(list) if is_task_fragment_start(node) => Some(list),
        _ => None,
    }
}

fn parse_phase_fragment(root: Root) -> Result<RoadmapFragment> {
    let document = parse_document_root(root, SourceId::Fragment)?;
    if !document.preamble.is_empty() {
        return Err(MapspliceError::InvalidRoadmap {
            message: "phase fragments must not contain a preamble".to_owned(),
        });
    }
    Ok(RoadmapFragment::Phase(document.phases))
}

fn parse_step_fragment(markdown: &str, heading: &Heading) -> Result<RoadmapFragment> {
    let step = parse_step_heading(heading)?.0;
    let wrapped = format!("## {}. Placeholder\n\n{markdown}", step.phase);
    let document = parse_wrapped_fragment(&wrapped)?;
    let phase = take_wrapped_phase(document, "wrapped step fragment did not produce a phase")?;
    if step_fragment_has_only_steps(&phase) {
        Ok(RoadmapFragment::Step(phase.steps))
    } else {
        Err(MapspliceError::InvalidRoadmap {
            message: "step fragments must contain only steps".to_owned(),
        })
    }
}

fn parse_task_fragment(markdown: &str, list: &List) -> Result<RoadmapFragment> {
    let first_task = parse_task_list(list, SourceId::Fragment)?
        .into_iter()
        .next()
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: "task fragment list is empty".to_owned(),
        })?;
    let step_number = first_task.number.step;
    let wrapped = format!(
        "## {}. Placeholder\n\n### {}. Placeholder\n\n{markdown}",
        step_number.phase, step_number
    );
    let document = parse_wrapped_fragment(&wrapped)?;
    let phase = take_wrapped_phase(document, "wrapped task fragment did not produce a phase")?;
    if !step_fragment_has_only_steps(&phase) {
        return Err(MapspliceError::InvalidRoadmap {
            message: "task fragments must contain only tasks".to_owned(),
        });
    }
    let step = phase
        .steps
        .into_iter()
        .next()
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: "wrapped task fragment did not produce a step".to_owned(),
        })?;
    if step_has_only_tasks(&step) {
        Ok(RoadmapFragment::Task(step.tasks))
    } else {
        Err(MapspliceError::InvalidRoadmap {
            message: "task fragments must contain only tasks".to_owned(),
        })
    }
}

fn parse_wrapped_fragment(markdown: &str) -> Result<RoadmapDocument> {
    let wrapped_root = parse_root(markdown)?;
    parse_document_root(wrapped_root, SourceId::Fragment)
}

fn take_wrapped_phase(
    document: RoadmapDocument,
    missing_message: &'static str,
) -> Result<PhaseSection> {
    document
        .phases
        .into_iter()
        .next()
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: missing_message.to_owned(),
        })
}

const fn step_fragment_has_only_steps(phase: &PhaseSection) -> bool {
    phase.body.is_empty() && phase.trailing.is_empty()
}

const fn step_has_only_tasks(step: &StepSection) -> bool {
    step.body.is_empty() && step.trailing.is_empty()
}
