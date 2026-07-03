//! Renumbering and dependency rewrite support.

use markdown::mdast::Node;

use super::{
    super::{
        PhaseNumber,
        RoadmapAnchor,
        RoadmapDocument,
        StepNumber,
        SubTaskNumber,
        TaskNumber,
        model::{MarkdownNodes, RenumberPlan, SourceId, SubTaskEntry, TaskEntry},
    },
    dependency_text::rewrite_text_value,
};
use crate::error::{MapspliceError, Result};

/// Mutable state shared by one dependency-rewrite traversal.
struct DependencyRewriteContext<'plan> {
    plan: &'plan RenumberPlan,
    rewrite_count: u64,
    unresolved: Vec<RoadmapAnchor>,
}

impl<'plan> DependencyRewriteContext<'plan> {
    const fn new(plan: &'plan RenumberPlan) -> Self {
        Self {
            plan,
            rewrite_count: 0,
            unresolved: Vec::new(),
        }
    }
}

/// Renumber every roadmap item and return the old-to-new mapping.
pub(super) fn renumber_document(roadmap: &mut RoadmapDocument) -> Result<RenumberPlan> {
    let mut plan = RenumberPlan::default();

    for (phase_index, phase) in roadmap.phases.iter_mut().enumerate() {
        let new_phase = PhaseNumber::new(to_number(phase_index + 1, "phase")?)?;
        plan.record_mapping(
            phase.identity.source,
            phase.identity.anchor,
            new_phase.into(),
        );
        phase.number = new_phase;

        for (step_index, step) in phase.steps.iter_mut().enumerate() {
            let new_step = StepNumber::new(new_phase, to_number(step_index + 1, "step")?)?;
            plan.record_mapping(step.identity.source, step.identity.anchor, new_step.into());
            step.number = new_step;

            for (task_index, task) in step.tasks.iter_mut().enumerate() {
                let new_task = TaskNumber::new(new_step, to_number(task_index + 1, "task")?)?;
                plan.record_mapping(task.identity.source, task.identity.anchor, new_task.into());
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
    for (sub_task_index, sub_task) in task.sub_tasks_mut().iter_mut().enumerate() {
        let new_sub_task =
            SubTaskNumber::new(new_task, to_number(sub_task_index + 1, "sub-task")?)?;
        plan.record_mapping(
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
    let mut context = DependencyRewriteContext::new(plan);
    rewrite_markdown_nodes(&mut roadmap.preamble, SourceId::Target, &mut context)?;

    for phase in &mut roadmap.phases {
        rewrite_markdown_nodes(&mut phase.title, phase.identity.source, &mut context)?;
        rewrite_markdown_nodes(&mut phase.body, phase.identity.source, &mut context)?;
        rewrite_markdown_nodes(&mut phase.trailing, phase.identity.source, &mut context)?;
        for step in &mut phase.steps {
            rewrite_markdown_nodes(&mut step.title, step.identity.source, &mut context)?;
            rewrite_markdown_nodes(&mut step.body, step.identity.source, &mut context)?;
            rewrite_markdown_nodes(&mut step.trailing, step.identity.source, &mut context)?;
            for task in &mut step.tasks {
                rewrite_task_entry(task, &mut context)?;
            }
        }
    }
    if let Some(anchor) = context.unresolved.into_iter().next() {
        return Err(MapspliceError::DanglingDependency { anchor });
    }
    Ok(context.rewrite_count)
}

/// Rewrite dependencies inside one task and its ordered sub-tasks.
fn rewrite_task_entry(
    task: &mut TaskEntry,
    context: &mut DependencyRewriteContext<'_>,
) -> Result<()> {
    rewrite_markdown_nodes(&mut task.summary, task.identity.source, context)?;
    rewrite_markdown_nodes(&mut task.body, task.identity.source, context)?;
    for sub_task in task.sub_tasks_mut() {
        rewrite_sub_task_entry(sub_task, context)?;
    }
    Ok(())
}

/// Rewrite dependencies inside one sub-task.
fn rewrite_sub_task_entry(
    sub_task: &mut SubTaskEntry,
    context: &mut DependencyRewriteContext<'_>,
) -> Result<()> {
    rewrite_markdown_nodes(&mut sub_task.summary, sub_task.identity.source, context)?;
    rewrite_markdown_nodes(&mut sub_task.body, sub_task.identity.source, context)
}

/// Rewrite Markdown nodes and invalidate original snippets only on change.
fn rewrite_markdown_nodes(
    markdown: &mut MarkdownNodes,
    source: SourceId,
    context: &mut DependencyRewriteContext<'_>,
) -> Result<()> {
    let before = context.rewrite_count;
    rewrite_nodes(markdown.nodes_mut(), source, context)?;
    if context.rewrite_count > before {
        markdown.clear_original_blocks();
    }
    Ok(())
}

/// Rewrite every eligible text node in a node slice.
fn rewrite_nodes(
    nodes: &mut [Node],
    source: SourceId,
    context: &mut DependencyRewriteContext<'_>,
) -> Result<()> {
    for node in nodes {
        rewrite_node(node, source, context)?;
    }
    Ok(())
}

/// Rewrite one Markdown node, recursing into child-bearing nodes.
fn rewrite_node(
    node: &mut Node,
    source: SourceId,
    context: &mut DependencyRewriteContext<'_>,
) -> Result<()> {
    if let Node::Text(text) = node {
        let report = rewrite_text_value(&text.value, source, context.plan);
        context.unresolved.extend(report.unresolved);
        text.value = report.value;
        context.rewrite_count += report.rewrite_count;
        return Ok(());
    }

    rewrite_container_node(node, source, context)
}

/// Rewrite block-level container children.
fn rewrite_container_node(
    node: &mut Node,
    source: SourceId,
    context: &mut DependencyRewriteContext<'_>,
) -> Result<()> {
    match node {
        Node::Root(root) => rewrite_nodes(&mut root.children, source, context),
        Node::Paragraph(paragraph) => rewrite_nodes(&mut paragraph.children, source, context),
        Node::Heading(heading) => rewrite_nodes(&mut heading.children, source, context),
        Node::Blockquote(blockquote) => rewrite_nodes(&mut blockquote.children, source, context),
        Node::List(list) => rewrite_nodes(&mut list.children, source, context),
        Node::ListItem(item) => rewrite_nodes(&mut item.children, source, context),
        Node::Table(table) => rewrite_nodes(&mut table.children, source, context),
        Node::TableRow(row) => rewrite_nodes(&mut row.children, source, context),
        Node::TableCell(cell) => rewrite_nodes(&mut cell.children, source, context),
        Node::FootnoteDefinition(definition) => {
            rewrite_nodes(&mut definition.children, source, context)
        }
        _ => rewrite_inline_container_node(node, source, context),
    }
}

/// Rewrite inline container children.
fn rewrite_inline_container_node(
    node: &mut Node,
    source: SourceId,
    context: &mut DependencyRewriteContext<'_>,
) -> Result<()> {
    match node {
        Node::Emphasis(emphasis) => rewrite_nodes(&mut emphasis.children, source, context),
        Node::Strong(strong) => rewrite_nodes(&mut strong.children, source, context),
        Node::Delete(delete) => rewrite_nodes(&mut delete.children, source, context),
        Node::Link(link) => rewrite_nodes(&mut link.children, source, context),
        Node::LinkReference(link) => rewrite_nodes(&mut link.children, source, context),
        _ => rewrite_mdx_container_node(node, source, context),
    }
}

/// Rewrite MDX container children.
fn rewrite_mdx_container_node(
    node: &mut Node,
    source: SourceId,
    context: &mut DependencyRewriteContext<'_>,
) -> Result<()> {
    match node {
        Node::MdxJsxFlowElement(element) => rewrite_nodes(&mut element.children, source, context),
        Node::MdxJsxTextElement(element) => rewrite_nodes(&mut element.children, source, context),
        _ => Ok(()),
    }
}

/// Convert a collection index into a supported roadmap number.
fn to_number(value: usize, label: &str) -> Result<u32> {
    u32::try_from(value).map_err(|_| MapspliceError::InvalidRoadmap {
        message: format!("{label} count exceeds supported numbering range"),
    })
}
