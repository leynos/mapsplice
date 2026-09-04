//! Unit tests for preserving original per-task source chunks.

use super::parse_roadmap;

#[test]
fn parsed_tasks_preserve_their_original_source_chunks() {
    let roadmap = parse_roadmap(concat!(
        "# Roadmap\n\n",
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. First task.\n",
        "  First continuation.\n\n",
        "- [ ] 1.1.2. Second task.\n",
        "  Second continuation.\n",
    ))
    .expect("roadmap should parse");
    let phase = roadmap
        .phases
        .first()
        .expect("roadmap should contain a phase");
    let step = phase.steps.first().expect("phase should contain a step");
    let first_task = step
        .tasks
        .first()
        .expect("step should contain the first task");
    let second_task = step
        .tasks
        .get(1)
        .expect("step should contain the second task");

    assert_eq!(
        first_task.task_source(),
        Some("- [ ] 1.1.1. First task.\n  First continuation.\n\n")
    );
    assert_eq!(
        second_task.task_source(),
        Some("- [ ] 1.1.2. Second task.\n  Second continuation.")
    );
}

#[test]
fn parsed_indented_tasks_do_not_share_marker_indentation() {
    let roadmap = parse_roadmap(concat!(
        "# Roadmap\n\n",
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "  - [ ] 1.1.1. First task.\n",
        "  - [ ] 1.1.2. Second task.\n",
    ))
    .expect("roadmap should parse");
    let phase = roadmap
        .phases
        .first()
        .expect("roadmap should contain a phase");
    let step = phase.steps.first().expect("phase should contain a step");
    let first_task = step
        .tasks
        .first()
        .expect("step should contain the first task");
    let second_task = step
        .tasks
        .get(1)
        .expect("step should contain the second task");

    assert_eq!(
        first_task.task_source(),
        Some("  - [ ] 1.1.1. First task.\n"),
    );
    assert_eq!(
        second_task.task_source(),
        Some("  - [ ] 1.1.2. Second task."),
    );
}
