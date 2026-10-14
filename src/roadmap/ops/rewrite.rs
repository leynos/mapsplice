//! Renumbering and dependency rewrite support.

use super::{
    super::{
        PhaseNumber,
        RoadmapAnchor,
        RoadmapDocument,
        StepNumber,
        SubTaskNumber,
        TaskNumber,
        model::{MarkdownNodes, RenumberPlan, SourceId, SubTaskEntry, TaskEntry},
        preservation_events::{PreservationInvalidationReason, PreservationReport},
    },
    dependency_visitor::{DependencyVisitMode, visit_dependency_nodes},
};
use crate::error::{MapspliceError, Result, SourceLocation};

/// Mutable state shared by one dependency-rewrite traversal.
struct DependencyRewriteContext<'plan> {
    /// Completed anchor remapping plan for this traversal.
    plan: &'plan RenumberPlan,
    /// Count of dependency references changed so far.
    rewrite_count: u64,
    /// Referenced anchors with no mapping found so far.
    unresolved: Vec<RoadmapAnchor>,
    /// Source location of the first unresolved reference.
    unresolved_location: Option<SourceLocation>,
}

impl<'plan> DependencyRewriteContext<'plan> {
    /// Create an empty traversal context for the supplied renumbering plan.
    const fn new(plan: &'plan RenumberPlan) -> Self {
        Self {
            plan,
            rewrite_count: 0,
            unresolved: Vec::new(),
            unresolved_location: None,
        }
    }
}

/// Renumber every roadmap item and return the old-to-new mapping.
pub(super) fn renumber_document(
    roadmap: &mut RoadmapDocument,
    report: &mut PreservationReport,
) -> Result<RenumberPlan> {
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
            if step.number != new_step {
                step.clear_task_list_source();
            }
            step.number = new_step;
            renumber_step_tasks(&mut step.tasks, new_step, &mut plan, report)?;
        }
    }

    Ok(plan)
}

/// Renumber tasks in one step and invalidate only tasks whose own number changes.
///
/// Returns an error when the collection exceeds the supported roadmap number
/// range.
fn renumber_step_tasks(
    tasks: &mut [TaskEntry],
    new_step: StepNumber,
    plan: &mut RenumberPlan,
    report: &mut PreservationReport,
) -> Result<()> {
    for (task_index, task) in tasks.iter_mut().enumerate() {
        let new_task = TaskNumber::new(new_step, to_number(task_index + 1, "task")?)?;
        plan.record_mapping(task.identity.source, task.identity.anchor, new_task.into());
        if task.number != new_task && task.set_number(new_task) {
            report.record_invalidation(PreservationInvalidationReason::Renumber);
        }
        renumber_sub_tasks(task, new_task, plan, report)?;
    }
    Ok(())
}

/// Renumber a task's sub-tasks and invalidate changed descendants and parents.
///
/// Returns an error when the sub-task collection exceeds the supported number
/// range.
fn renumber_sub_tasks(
    task: &mut TaskEntry,
    new_task: TaskNumber,
    plan: &mut RenumberPlan,
    report: &mut PreservationReport,
) -> Result<()> {
    let mut descendants_changed = false;
    for (sub_task_index, sub_task) in task.sub_tasks_mut().iter_mut().enumerate() {
        let new_sub_task =
            SubTaskNumber::new(new_task, to_number(sub_task_index + 1, "sub-task")?)?;
        plan.record_mapping(
            sub_task.identity.source,
            sub_task.identity.anchor,
            new_sub_task.into(),
        );
        if sub_task.number != new_sub_task && sub_task.clear_original_source() {
            report.record_invalidation(PreservationInvalidationReason::Renumber);
            descendants_changed = true;
        }
        sub_task.number = new_sub_task;
    }
    if descendants_changed && task.clear_original_source() {
        report.record_invalidation(PreservationInvalidationReason::Renumber);
    }
    Ok(())
}

/// Rewrite anchor-like text references using a completed renumbering plan.
pub(super) fn rewrite_dependencies(
    roadmap: &mut RoadmapDocument,
    plan: &RenumberPlan,
    report: &mut PreservationReport,
) -> Result<u64> {
    let mut context = DependencyRewriteContext::new(plan);
    rewrite_markdown_nodes(&mut roadmap.preamble, SourceId::Target, &mut context);

    for phase in &mut roadmap.phases {
        rewrite_markdown_nodes(&mut phase.title, phase.identity.source, &mut context);
        rewrite_markdown_nodes(&mut phase.body, phase.identity.source, &mut context);
        rewrite_markdown_nodes(&mut phase.trailing, phase.identity.source, &mut context);
        for step in &mut phase.steps {
            rewrite_markdown_nodes(&mut step.title, step.identity.source, &mut context);
            rewrite_markdown_nodes(&mut step.body, step.identity.source, &mut context);
            rewrite_markdown_nodes(&mut step.trailing, step.identity.source, &mut context);
            let before_tasks = context.rewrite_count;
            for task in &mut step.tasks {
                rewrite_task_entry(task, &mut context, report);
            }
            if context.rewrite_count > before_tasks {
                step.clear_task_list_source();
            }
        }
    }
    if let Some(anchor) = context.unresolved.into_iter().next() {
        return Err(MapspliceError::DanglingDependency {
            anchor,
            location: context.unresolved_location,
        });
    }
    Ok(context.rewrite_count)
}

/// Rewrite dependency references in a task and its structural descendants.
///
/// The task's preserved source is cleared when its own text or a descendant
/// changes. Errors report unresolved dependency anchors.
fn rewrite_task_entry(
    task: &mut TaskEntry,
    context: &mut DependencyRewriteContext<'_>,
    report: &mut PreservationReport,
) {
    let summary_changed = rewrite_markdown_nodes(&mut task.summary, task.identity.source, context);
    let body_changed = rewrite_markdown_nodes(&mut task.body, task.identity.source, context);
    let children_changed = rewrite_task_body_children(task, context);
    let task_text_changed = summary_changed || body_changed || children_changed;
    if task_text_changed && task.clear_original_source() {
        report.record_invalidation(PreservationInvalidationReason::DependencyRewrite);
    }
    let mut descendant_changed = false;
    for sub_task in task.sub_tasks_mut() {
        if rewrite_sub_task_entry(sub_task, context, report) {
            descendant_changed = true;
        }
    }
    if descendant_changed && task.clear_original_source() {
        report.record_invalidation(PreservationInvalidationReason::DependencyRewrite);
    }
}

/// Rewrite nested task-body blocks, leaving structural sub-tasks untouched.
///
/// Body-child blocks carry the nested body Markdown that
/// [`TaskEntry::body`] has already had drained into them by the parser, so a
/// dependency clause inside a body bullet is only reachable here. Sub-task
/// children are skipped because [`TaskEntry::sub_tasks_mut`] rewrites the same
/// content and visiting both would double-process it.
///
/// Returns whether any body block changed, and reports unresolved dependency
/// anchors through `context`.
fn rewrite_task_body_children(
    task: &mut TaskEntry,
    context: &mut DependencyRewriteContext<'_>,
) -> bool {
    let source = task.identity.source;
    let mut changed = false;
    for body in task.body_children_mut() {
        changed |= rewrite_markdown_nodes(body, source, context);
    }
    changed
}

/// Rewrite dependency references in one sub-task and report whether it changed.
///
/// Errors report unresolved dependency anchors encountered in the sub-task.
fn rewrite_sub_task_entry(
    sub_task: &mut SubTaskEntry,
    context: &mut DependencyRewriteContext<'_>,
    report: &mut PreservationReport,
) -> bool {
    let summary_changed =
        rewrite_markdown_nodes(&mut sub_task.summary, sub_task.identity.source, context);
    let body_changed =
        rewrite_markdown_nodes(&mut sub_task.body, sub_task.identity.source, context);
    let changed = summary_changed || body_changed;
    if changed && sub_task.clear_original_source() {
        report.record_invalidation(PreservationInvalidationReason::DependencyRewrite);
    }
    changed
}

/// Rewrite dependency references in Markdown nodes and report whether they changed.
///
/// Changed nodes lose their preserved block source; errors report unresolved
/// dependency anchors.
fn rewrite_markdown_nodes(
    markdown: &mut MarkdownNodes,
    source: SourceId,
    context: &mut DependencyRewriteContext<'_>,
) -> bool {
    let before = context.rewrite_count;
    visit_dependency_nodes(
        markdown.nodes_mut(),
        &mut DependencyVisitMode::Rewrite {
            source,
            plan: context.plan,
            rewrite_count: &mut context.rewrite_count,
            unresolved: &mut context.unresolved,
            unresolved_location: &mut context.unresolved_location,
        },
    );
    if context.rewrite_count > before {
        markdown.clear_original_blocks();
    }
    context.rewrite_count > before
}

/// Convert a collection index to a supported roadmap number.
///
/// Returns [`MapspliceError::InvalidRoadmap`] when `value` cannot fit in a
/// `u32`.
fn to_number(value: usize, label: &str) -> Result<u32> {
    u32::try_from(value).map_err(|_| MapspliceError::InvalidRoadmap {
        message: format!("{label} count exceeds supported numbering range"),
        location: None,
    })
}
