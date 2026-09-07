//! Unit tests for task-list render-state observability.

use super::render_roadmap;
use crate::{
    observability::{MetricsSnapshot, metrics_snapshot},
    roadmap::{RoadmapDocument, model::StepSection, parse_roadmap},
};

#[test]
fn task_render_metrics_distinguish_preserved_and_canonical_items() {
    let source = concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. First task.\n",
        "  First continuation.\n",
        "- [ ] 1.1.2. Second task.\n",
        "  Second continuation.\n",
    );
    let mut roadmap = parse_roadmap(source).expect("roadmap should parse");
    let before = metrics_snapshot();

    let after_preserved = render_and_snapshot(&roadmap, "preserved task list should render");
    clear_task_source(&mut roadmap, 0);
    let after_mixed = render_and_snapshot(&roadmap, "mixed task list should render");
    clear_task_source(&mut roadmap, 1);
    let after_canonical = render_and_snapshot(&roadmap, "canonical task list should render");

    assert_task_render_delta(before, after_preserved, TaskRenderDelta::new(2, 0, 0));
    assert_task_render_delta(after_preserved, after_mixed, TaskRenderDelta::new(1, 1, 1));
    assert_task_render_delta(after_mixed, after_canonical, TaskRenderDelta::new(0, 2, 2));
}

fn render_and_snapshot(roadmap: &RoadmapDocument, error_message: &str) -> MetricsSnapshot {
    if let Err(error) = render_roadmap(roadmap) {
        panic!("{error_message}: {error}");
    }
    metrics_snapshot()
}

fn clear_task_source(roadmap: &mut RoadmapDocument, index: usize) {
    let step = first_step_mut(roadmap);
    step.clear_task_list_source();
    let Some(task) = step.tasks.get_mut(index) else {
        panic!("step should contain the requested task");
    };
    task.clear_task_source();
}

fn first_step_mut(roadmap: &mut RoadmapDocument) -> &mut StepSection {
    let Some(phase) = roadmap.phases.first_mut() else {
        panic!("roadmap should contain a phase");
    };
    let Some(step) = phase.steps.first_mut() else {
        panic!("phase should contain a step");
    };
    step
}

#[derive(Clone, Copy)]
struct TaskRenderDelta {
    preserved: u64,
    canonical: u64,
    fallbacks: u64,
}

impl TaskRenderDelta {
    const fn new(preserved: u64, canonical: u64, fallbacks: u64) -> Self {
        Self {
            preserved,
            canonical,
            fallbacks,
        }
    }
}

fn assert_task_render_delta(
    before: MetricsSnapshot,
    after: MetricsSnapshot,
    expected: TaskRenderDelta,
) {
    assert_eq!(
        after.preserved_task_items - before.preserved_task_items,
        expected.preserved
    );
    assert_eq!(
        after.canonical_task_items - before.canonical_task_items,
        expected.canonical
    );
    assert_eq!(
        after.task_source_fallbacks - before.task_source_fallbacks,
        expected.fallbacks
    );
}
