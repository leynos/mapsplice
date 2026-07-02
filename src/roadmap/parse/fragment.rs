//! Fragment-level roadmap parsing.

use markdown::mdast::{List, Node, Root};

use super::{
    document::parse_document_root,
    is_phase_heading,
    is_step_heading,
    looks_like_sub_task_list,
    looks_like_task_list,
    parse_root,
    parse_step_heading,
    parse_sub_task_fragment_list,
    parse_task_list,
};
use crate::{
    error::{MapspliceError, Result},
    roadmap::{
        RoadmapFragment,
        StepNumber,
        model::{ItemIdentity, MarkdownNodes, SourceId, StepSection, SubTaskEntry, TaskEntry},
    },
};

/// Parse a fragment file.
///
/// # Errors
///
/// Returns an error when the Markdown cannot be parsed or when the fragment
/// does not cleanly represent one or more sibling phases, steps, or tasks.
#[tracing::instrument(skip_all, fields(bytes = markdown.len()))]
pub fn parse_fragment(markdown: &str) -> Result<RoadmapFragment> {
    let root = parse_root(markdown)?;
    let first = root
        .children
        .first()
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: "fragment file is empty".to_owned(),
        })?;

    if is_phase_fragment_start(first) {
        return parse_phase_fragment(root, markdown);
    }

    if is_step_fragment_start(first) {
        return parse_step_fragment_root(root, markdown);
    }

    if is_task_fragment_start(first) {
        return parse_task_fragment_root(root, markdown);
    }

    if is_sub_task_fragment_start(first) {
        return parse_sub_task_fragment_root(root, markdown);
    }

    Err(MapspliceError::InvalidRoadmap {
        message: concat!(
            "fragment must start with a phase heading, step heading, numbered ",
            "task list, or numbered sub-task list"
        )
        .to_owned(),
    })
}

/// Return whether the first fragment node starts a phase fragment.
fn is_phase_fragment_start(node: &Node) -> bool {
    matches!(node, Node::Heading(heading) if is_phase_heading(heading))
}

/// Return whether the first fragment node starts a step fragment.
fn is_step_fragment_start(node: &Node) -> bool {
    matches!(node, Node::Heading(heading) if heading.depth == 3 && parse_step_heading(heading).is_ok())
}

/// Return whether the first fragment node starts a task fragment.
fn is_task_fragment_start(node: &Node) -> bool {
    matches!(node, Node::List(list) if looks_like_task_list(list))
}

/// Return whether the first fragment node starts an addendum sub-task fragment.
fn is_sub_task_fragment_start(node: &Node) -> bool {
    matches!(node, Node::List(list) if looks_like_sub_task_list(list))
}

/// Parse one or more sibling phases from a fragment root.
fn parse_phase_fragment(root: Root, source_text: &str) -> Result<RoadmapFragment> {
    let document = parse_document_root(root, SourceId::Fragment, source_text)?;
    if !document.preamble.is_empty() {
        return Err(MapspliceError::InvalidRoadmap {
            message: "phase fragments must not contain a preamble".to_owned(),
        });
    }
    Ok(RoadmapFragment::Phase(document.phases))
}

/// Parse one or more sibling steps directly from a fragment root.
fn parse_step_fragment_root(root: Root, source_text: &str) -> Result<RoadmapFragment> {
    let mut steps = Vec::new();
    let mut current_step = None;

    for node in root.children {
        match node {
            Node::Heading(heading) if is_step_heading(&heading) => {
                if let Some(step) = current_step.take() {
                    steps.push(step);
                }
                let (number, title) = parse_step_heading(&heading)?;
                current_step = Some(StepSection {
                    identity: ItemIdentity {
                        source: SourceId::Fragment,
                        anchor: number.into(),
                    },
                    number,
                    title: MarkdownNodes::from_nodes(title),
                    body: MarkdownNodes::new(),
                    tasks: Vec::new(),
                    trailing: MarkdownNodes::new(),
                });
            }
            Node::Heading(_) => {
                return Err(MapspliceError::InvalidRoadmap {
                    message: "step fragments must contain only step sections".to_owned(),
                });
            }
            Node::List(list) if looks_like_task_list(&list) => {
                append_step_fragment_tasks(&mut current_step, &list, source_text)?;
            }
            other => push_step_fragment_body(&mut current_step, other, source_text)?,
        }
    }

    if let Some(step) = current_step {
        steps.push(step);
    }

    if steps.is_empty() {
        return Err(MapspliceError::InvalidRoadmap {
            message: "step fragments must contain only step sections".to_owned(),
        });
    }
    validate_step_siblings(&steps)?;

    Ok(RoadmapFragment::Step(steps))
}

/// Parse a single top-level checklist as a task fragment.
fn parse_task_fragment_root(root: Root, source_text: &str) -> Result<RoadmapFragment> {
    if root.children.len() != 1 {
        return Err(MapspliceError::InvalidRoadmap {
            message: "task fragments must contain only a single task list".to_owned(),
        });
    }

    let Some(Node::List(list)) = root.children.into_iter().next() else {
        return Err(MapspliceError::InvalidRoadmap {
            message: "task fragments must contain only a single task list".to_owned(),
        });
    };
    let tasks = parse_task_list(&list, SourceId::Fragment, source_text)?;
    if tasks.is_empty() {
        return Err(MapspliceError::InvalidRoadmap {
            message: "task fragment list is empty".to_owned(),
        });
    }
    validate_task_siblings(&tasks)?;
    Ok(RoadmapFragment::Task(tasks))
}

/// Parse a single top-level checklist as an addendum sub-task fragment.
fn parse_sub_task_fragment_root(root: Root, source_text: &str) -> Result<RoadmapFragment> {
    if root.children.len() != 1 {
        return Err(MapspliceError::InvalidRoadmap {
            message: "sub-task fragments must contain only a single sub-task list".to_owned(),
        });
    }

    let Some(Node::List(list)) = root.children.into_iter().next() else {
        return Err(MapspliceError::InvalidRoadmap {
            message: "sub-task fragments must contain only a single sub-task list".to_owned(),
        });
    };
    let sub_tasks = parse_sub_task_fragment_list(&list, source_text)?;
    if sub_tasks.is_empty() {
        return Err(MapspliceError::InvalidRoadmap {
            message: "sub-task fragment list is empty".to_owned(),
        });
    }
    validate_sub_task_siblings(&sub_tasks)?;
    Ok(RoadmapFragment::SubTask(sub_tasks))
}

/// Append task list entries to the active step fragment.
fn append_step_fragment_tasks(
    step: &mut Option<StepSection>,
    list: &List,
    source_text: &str,
) -> Result<()> {
    let current = step
        .as_mut()
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: "task list appeared without a current step".to_owned(),
        })?;
    if !current.trailing.is_empty() {
        return Err(MapspliceError::InvalidRoadmap {
            message: format!(
                "task list for step `{}` cannot appear after trailing step content",
                current.number
            ),
        });
    }

    let mut tasks = parse_task_list(list, SourceId::Fragment, source_text)?;
    validate_task_numbers(current.number, &tasks)?;
    current.tasks.append(&mut tasks);
    Ok(())
}

/// Preserve non-structural nodes within the active step fragment.
fn push_step_fragment_body(
    step: &mut Option<StepSection>,
    node: Node,
    source_text: &str,
) -> Result<()> {
    let current = step
        .as_mut()
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: "step fragments must contain only step sections".to_owned(),
        })?;
    if current.tasks.is_empty() {
        current.body.push_preserved(node, source_text);
    } else {
        current.trailing.push_preserved(node, source_text);
    }
    Ok(())
}

/// Ensure task fragment numbers belong to the active step.
fn validate_task_numbers(step_number: StepNumber, tasks: &[TaskEntry]) -> Result<()> {
    for task in tasks {
        if task.number.step_number() != step_number {
            return Err(MapspliceError::InvalidRoadmap {
                message: format!(
                    "task `{}` does not belong to step `{}`",
                    task.number, step_number
                ),
            });
        }
    }
    Ok(())
}

/// Ensure step fragments contain siblings from the same phase.
fn validate_step_siblings(steps: &[StepSection]) -> Result<()> {
    let Some(first) = steps.first() else {
        return Ok(());
    };
    let phase = first.number.phase();
    for step in steps {
        if step.number.phase() != phase {
            return Err(MapspliceError::InvalidRoadmap {
                message: "step fragments must contain steps from one phase".to_owned(),
            });
        }
    }
    Ok(())
}

/// Ensure task fragments contain siblings from the same step.
fn validate_task_siblings(tasks: &[TaskEntry]) -> Result<()> {
    let Some(first) = tasks.first() else {
        return Ok(());
    };
    let step_number = first.number.step_number();
    for task in tasks {
        if task.number.step_number() != step_number {
            return Err(MapspliceError::InvalidRoadmap {
                message: "task fragments must contain tasks from one step".to_owned(),
            });
        }
    }
    Ok(())
}

/// Ensure sub-task fragments contain siblings from the same parent task.
fn validate_sub_task_siblings(sub_tasks: &[SubTaskEntry]) -> Result<()> {
    let Some(first) = sub_tasks.first() else {
        return Ok(());
    };
    let parent = first.number.task_number();
    for sub_task in sub_tasks {
        if sub_task.number.task_number() != parent {
            return Err(MapspliceError::InvalidRoadmap {
                message: "sub-task fragments must contain sub-tasks from one task".to_owned(),
            });
        }
    }
    Ok(())
}
