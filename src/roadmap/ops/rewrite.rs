//! Renumbering and dependency rewrite support.

use markdown::mdast::Node;

use super::{
    super::{
        PhaseNumber,
        RoadmapDocument,
        StepNumber,
        SubTaskNumber,
        TaskNumber,
        model::{MarkdownNodes, RenumberPlan, SourceId, SubTaskEntry, TaskEntry},
    },
    dependency_text::rewrite_text_value,
};
use crate::error::{MapspliceError, Result};

/// Renumber every roadmap item and return the old-to-new mapping.
pub(super) fn renumber_document(roadmap: &mut RoadmapDocument) -> Result<RenumberPlan> {
    let mut plan = RenumberPlan::default();

    for (phase_index, phase) in roadmap.phases.iter_mut().enumerate() {
        let new_phase = PhaseNumber::new(to_number(phase_index + 1, "phase")?)?;
        plan.insert(
            phase.identity.source,
            phase.identity.anchor,
            new_phase.into(),
        );
        phase.number = new_phase;

        for (step_index, step) in phase.steps.iter_mut().enumerate() {
            let new_step = StepNumber::new(new_phase, to_number(step_index + 1, "step")?)?;
            plan.insert(step.identity.source, step.identity.anchor, new_step.into());
            step.number = new_step;

            for (task_index, task) in step.tasks.iter_mut().enumerate() {
                let new_task = TaskNumber::new(new_step, to_number(task_index + 1, "task")?)?;
                plan.insert(task.identity.source, task.identity.anchor, new_task.into());
                task.number = new_task;
                renumber_sub_tasks(task, new_task, &mut plan)?;
            }
        }
    }

    Ok(plan)
}

/// Renumber ordered sub-tasks beneath one task.
fn renumber_sub_tasks(
    task: &mut TaskEntry,
    new_task: TaskNumber,
    plan: &mut RenumberPlan,
) -> Result<()> {
    for (sub_task_index, sub_task) in task.sub_tasks.iter_mut().enumerate() {
        let new_sub_task =
            SubTaskNumber::new(new_task, to_number(sub_task_index + 1, "sub-task")?)?;
        plan.insert(
            sub_task.identity.source,
            sub_task.identity.anchor,
            new_sub_task.into(),
        );
        sub_task.number = new_sub_task;
    }
    Ok(())
}

/// Rewrite anchor-like text references using a completed renumbering plan.
pub(super) fn rewrite_dependencies(
    roadmap: &mut RoadmapDocument,
    plan: &RenumberPlan,
) -> Result<u64> {
    let mut rewrite_count = 0;
    rewrite_markdown_nodes(
        &mut roadmap.preamble,
        SourceId::Target,
        plan,
        &mut rewrite_count,
    )?;

    for phase in &mut roadmap.phases {
        rewrite_markdown_nodes(
            &mut phase.title,
            phase.identity.source,
            plan,
            &mut rewrite_count,
        )?;
        rewrite_markdown_nodes(
            &mut phase.body,
            phase.identity.source,
            plan,
            &mut rewrite_count,
        )?;
        rewrite_markdown_nodes(
            &mut phase.trailing,
            phase.identity.source,
            plan,
            &mut rewrite_count,
        )?;
        for step in &mut phase.steps {
            rewrite_markdown_nodes(
                &mut step.title,
                step.identity.source,
                plan,
                &mut rewrite_count,
            )?;
            rewrite_markdown_nodes(
                &mut step.body,
                step.identity.source,
                plan,
                &mut rewrite_count,
            )?;
            rewrite_markdown_nodes(
                &mut step.trailing,
                step.identity.source,
                plan,
                &mut rewrite_count,
            )?;
            for task in &mut step.tasks {
                rewrite_task_entry(task, plan, &mut rewrite_count)?;
            }
        }
    }
    Ok(rewrite_count)
}

/// Rewrite dependencies inside one task and its ordered sub-tasks.
fn rewrite_task_entry(
    task: &mut TaskEntry,
    plan: &RenumberPlan,
    rewrite_count: &mut u64,
) -> Result<()> {
    rewrite_markdown_nodes(&mut task.summary, task.identity.source, plan, rewrite_count)?;
    rewrite_markdown_nodes(&mut task.body, task.identity.source, plan, rewrite_count)?;
    for sub_task in &mut task.sub_tasks {
        rewrite_sub_task_entry(sub_task, plan, rewrite_count)?;
    }
    Ok(())
}

/// Rewrite dependencies inside one sub-task.
fn rewrite_sub_task_entry(
    sub_task: &mut SubTaskEntry,
    plan: &RenumberPlan,
    rewrite_count: &mut u64,
) -> Result<()> {
    rewrite_markdown_nodes(
        &mut sub_task.summary,
        sub_task.identity.source,
        plan,
        rewrite_count,
    )?;
    rewrite_markdown_nodes(
        &mut sub_task.body,
        sub_task.identity.source,
        plan,
        rewrite_count,
    )
}

/// Rewrite Markdown nodes and invalidate original snippets only on change.
fn rewrite_markdown_nodes(
    markdown: &mut MarkdownNodes,
    source: SourceId,
    plan: &RenumberPlan,
    rewrite_count: &mut u64,
) -> Result<()> {
    let before = *rewrite_count;
    rewrite_nodes(markdown.nodes_mut(), source, plan, rewrite_count)?;
    if *rewrite_count > before {
        markdown.clear_original_blocks();
    }
    Ok(())
}

/// Rewrite every eligible text node in a node slice.
fn rewrite_nodes(
    nodes: &mut [Node],
    source: SourceId,
    plan: &RenumberPlan,
    rewrite_count: &mut u64,
) -> Result<()> {
    for node in nodes {
        rewrite_node(node, source, plan, rewrite_count)?;
    }
    Ok(())
}

/// Rewrite one Markdown node, recursing into child-bearing nodes.
fn rewrite_node(
    node: &mut Node,
    source: SourceId,
    plan: &RenumberPlan,
    rewrite_count: &mut u64,
) -> Result<()> {
    if let Node::Text(text) = node {
        let (rewritten, count) = rewrite_text_value(&text.value, source, plan);
        text.value = rewritten;
        *rewrite_count += count;
        return Ok(());
    }

    rewrite_container_node(node, source, plan, rewrite_count)
}

/// Rewrite block-level container children.
fn rewrite_container_node(
    node: &mut Node,
    source: SourceId,
    plan: &RenumberPlan,
    rewrite_count: &mut u64,
) -> Result<()> {
    match node {
        Node::Root(root) => rewrite_nodes(&mut root.children, source, plan, rewrite_count),
        Node::Paragraph(paragraph) => {
            rewrite_nodes(&mut paragraph.children, source, plan, rewrite_count)
        }
        Node::Heading(heading) => rewrite_nodes(&mut heading.children, source, plan, rewrite_count),
        Node::Blockquote(blockquote) => {
            rewrite_nodes(&mut blockquote.children, source, plan, rewrite_count)
        }
        Node::List(list) => rewrite_nodes(&mut list.children, source, plan, rewrite_count),
        Node::ListItem(item) => rewrite_nodes(&mut item.children, source, plan, rewrite_count),
        Node::Table(table) => rewrite_nodes(&mut table.children, source, plan, rewrite_count),
        Node::TableRow(row) => rewrite_nodes(&mut row.children, source, plan, rewrite_count),
        Node::TableCell(cell) => rewrite_nodes(&mut cell.children, source, plan, rewrite_count),
        Node::FootnoteDefinition(definition) => {
            rewrite_nodes(&mut definition.children, source, plan, rewrite_count)
        }
        _ => rewrite_inline_container_node(node, source, plan, rewrite_count),
    }
}

/// Rewrite inline container children.
fn rewrite_inline_container_node(
    node: &mut Node,
    source: SourceId,
    plan: &RenumberPlan,
    rewrite_count: &mut u64,
) -> Result<()> {
    match node {
        Node::Emphasis(emphasis) => {
            rewrite_nodes(&mut emphasis.children, source, plan, rewrite_count)
        }
        Node::Strong(strong) => rewrite_nodes(&mut strong.children, source, plan, rewrite_count),
        Node::Delete(delete) => rewrite_nodes(&mut delete.children, source, plan, rewrite_count),
        Node::Link(link) => rewrite_nodes(&mut link.children, source, plan, rewrite_count),
        Node::LinkReference(link) => rewrite_nodes(&mut link.children, source, plan, rewrite_count),
        _ => rewrite_mdx_container_node(node, source, plan, rewrite_count),
    }
}

/// Rewrite MDX container children.
fn rewrite_mdx_container_node(
    node: &mut Node,
    source: SourceId,
    plan: &RenumberPlan,
    rewrite_count: &mut u64,
) -> Result<()> {
    match node {
        Node::MdxJsxFlowElement(element) => {
            rewrite_nodes(&mut element.children, source, plan, rewrite_count)
        }
        Node::MdxJsxTextElement(element) => {
            rewrite_nodes(&mut element.children, source, plan, rewrite_count)
        }
        _ => Ok(()),
    }
}

/// Convert a collection index into a supported roadmap number.
fn to_number(value: usize, label: &str) -> Result<u32> {
    u32::try_from(value).map_err(|_| MapspliceError::InvalidRoadmap {
        message: format!("{label} count exceeds supported numbering range"),
    })
}
