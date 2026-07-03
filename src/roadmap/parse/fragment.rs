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
    step_accumulator::StepAccumulator,
};
use crate::{
    error::{MapspliceError, Result},
    roadmap::{
        RoadmapFragment,
        model::{SourceId, StepSection, SubTaskEntry, TaskEntry},
    },
};

/// Parse a fragment file.
///
/// # Examples
///
/// ```rust
/// use mapsplice::{RoadmapFragment, parse_fragment};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let fragment = parse_fragment(
///     "## 1. Added phase\n\n### 1.1. Added step\n\n- [ ] 1.1.1. Add the first task\n",
/// )?;
///
/// let RoadmapFragment::Phase(phases) = fragment else {
///     return Err("expected a phase fragment".into());
/// };
/// let task = &phases[0].steps[0].tasks[0];
///
/// assert_eq!(phases[0].number.get(), 1);
/// assert_eq!(task.number.to_string(), "1.1.1");
/// # Ok(())
/// # }
/// ```
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
    let mut accumulator = StepAccumulator::new(SourceId::Fragment, source_text);

    for node in root.children {
        match node {
            Node::Heading(heading) if is_step_heading(&heading) => {
                let (number, title) = parse_step_heading(&heading)?;
                accumulator.begin_step(number, title, &mut steps);
            }
            Node::Heading(_) => {
                return Err(MapspliceError::InvalidRoadmap {
                    message: "step fragments must contain only step sections".to_owned(),
                });
            }
            Node::List(list) if looks_like_task_list(&list) => {
                accumulator.append_task_list(&list)?;
            }
            other => accumulator.push_non_structural_node(other)?,
        }
    }

    accumulator.flush_into(&mut steps);

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
    parse_single_list_fragment(
        root,
        source_text,
        SingleListFragmentParser {
            messages: SingleListFragmentMessages {
                single_list: "task fragments must contain only a single task list",
                empty_list: "task fragment list is empty",
            },
            parse_list: parse_task_fragment_list,
            validate: validate_task_siblings,
            wrap: RoadmapFragment::Task,
        },
    )
}

/// Parse a single top-level checklist as an addendum sub-task fragment.
fn parse_sub_task_fragment_root(root: Root, source_text: &str) -> Result<RoadmapFragment> {
    parse_single_list_fragment(
        root,
        source_text,
        SingleListFragmentParser {
            messages: SingleListFragmentMessages {
                single_list: "sub-task fragments must contain only a single sub-task list",
                empty_list: "sub-task fragment list is empty",
            },
            parse_list: parse_sub_task_fragment_list,
            validate: validate_sub_task_siblings,
            wrap: RoadmapFragment::SubTask,
        },
    )
}

fn parse_task_fragment_list(list: &List, source_text: &str) -> Result<Vec<TaskEntry>> {
    parse_task_list(list, SourceId::Fragment, source_text)
}

struct SingleListFragmentParser<T> {
    messages: SingleListFragmentMessages,
    parse_list: fn(&List, &str) -> Result<Vec<T>>,
    validate: fn(&[T]) -> Result<()>,
    wrap: fn(Vec<T>) -> RoadmapFragment,
}

struct SingleListFragmentMessages {
    single_list: &'static str,
    empty_list: &'static str,
}

/// Parse one top-level list with fragment-level validation and wrapping.
fn parse_single_list_fragment<T>(
    root: Root,
    source_text: &str,
    parser: SingleListFragmentParser<T>,
) -> Result<RoadmapFragment> {
    let SingleListFragmentParser {
        messages,
        parse_list,
        validate,
        wrap,
    } = parser;

    if root.children.len() != 1 {
        return Err(MapspliceError::InvalidRoadmap {
            message: messages.single_list.to_owned(),
        });
    }

    let Some(Node::List(list)) = root.children.into_iter().next() else {
        return Err(MapspliceError::InvalidRoadmap {
            message: messages.single_list.to_owned(),
        });
    };
    let items = parse_list(&list, source_text)?;
    if items.is_empty() {
        return Err(MapspliceError::InvalidRoadmap {
            message: messages.empty_list.to_owned(),
        });
    }
    validate(&items)?;
    Ok(wrap(items))
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
