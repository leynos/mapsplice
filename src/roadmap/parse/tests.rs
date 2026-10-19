//! Parser regression tests for ordered task-child classification.

use rstest::rstest;

use super::TaskChild;
use crate::roadmap::parse_roadmap;

const TASK_PREFIX: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.2. Parent.\n",
);

#[rstest]
#[case::body_before(
    "  - A non-structural body bullet.\n  - [ ] 1.1.2.1. Sub one.\n",
    "body, 1.1.2.1",
    "1.1.2.1"
)]
#[case::body_between(
    "  - [ ] 1.1.2.1. Sub one.\n  - A non-structural body bullet.\n  - [ ] 1.1.2.2. Sub two.\n",
    "1.1.2.1, body, 1.1.2.2",
    "1.1.2.1, 1.1.2.2"
)]
#[case::body_after(
    "  - [ ] 1.1.2.1. Sub one.\n  - A non-structural body bullet.\n",
    "1.1.2.1, body",
    "1.1.2.1"
)]
fn mixed_nested_list_keeps_body_runs_between_structural_sub_tasks(
    #[case] nested_items: &str,
    #[case] expected_child_order: &str,
    #[case] expected_sub_tasks: &str,
) {
    let roadmap = parse_roadmap(&format!("{TASK_PREFIX}{nested_items}"))
        .expect("mixed nested list should parse");
    let task = roadmap
        .phases
        .first()
        .and_then(|phase| phase.steps.first())
        .and_then(|step| step.tasks().first())
        .expect("roadmap should contain the parent task");
    let child_order = task
        .children()
        .iter()
        .map(|child| match child {
            TaskChild::Body(_) => "body".to_owned(),
            TaskChild::SubTask(identity) => identity.anchor.to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ");
    let sub_tasks = task
        .sub_tasks()
        .iter()
        .map(|sub_task| sub_task.number.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    assert_eq!(child_order, expected_child_order);
    assert_eq!(sub_tasks, expected_sub_tasks);
}
