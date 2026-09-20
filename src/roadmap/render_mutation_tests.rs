//! Regression tests for source-cache invalidation during task mutation.

use super::render_roadmap;
use crate::roadmap::{RoadmapAnchor, parse_anchor, parse_roadmap};

#[test]
fn task_mutation_invalidates_preserved_task_and_list_source() {
    let mut roadmap = parse_roadmap(concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Task.\n",
        "- [ ] 1.1.2. Removed task.\n",
    ))
    .expect("roadmap should parse");
    let step = roadmap
        .phases
        .first_mut()
        .and_then(|phase| phase.steps.first_mut())
        .expect("roadmap should contain a step");
    let task = step
        .tasks_mut()
        .first_mut()
        .expect("step should contain a task");
    task.set_checked(Some(true));
    let RoadmapAnchor::Task(number) = parse_anchor("1.1.9").expect("task anchor should parse")
    else {
        panic!("expected a task anchor");
    };
    task.set_number(number);
    let removed = step
        .tasks_mut()
        .pop()
        .expect("step should contain a second task");

    let rendered = render_roadmap(&roadmap).expect("mutated roadmap should render");

    assert!(rendered.contains("- [x] 1.1.9. Task."));
    assert!(!rendered.contains("- [ ] 1.1.1. Task."));
    assert!(!rendered.contains("Removed task."));
    assert_eq!(removed.number().to_string(), "1.1.2");
}
