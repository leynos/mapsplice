//! Generated checks for mixed body-list and addendum mutations.

use std::fmt::Write;

use markdown::mdast::Node;
use proptest::prelude::*;

use super::{RoadmapDocument, TaskChild, TaskEntry};
use crate::roadmap::{
    RoadmapOperation,
    apply_command,
    parse_anchor,
    parse_fragment,
    parse_roadmap,
    render_roadmap,
};

const BODY_LIST_BOUNDARY_MARKER: &str = "<!-- mapsplice: preserve body list boundary -->";

#[derive(Clone, Copy)]
enum ChildShape {
    BodyList(usize),
    SubTask(usize),
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 24,
        .. ProptestConfig::default()
    })]

    /// Mixed tight-list runs retain order as addenda are inserted and deleted.
    #[test]
    fn mixed_body_runs_keep_order_across_sub_task_mutations(
        before in 0usize..4,
        between_first_and_second in 0usize..4,
        between_second_and_third in 0usize..4,
        after in 0usize..4,
    ) {
        let body_runs = [before, between_first_and_second, between_second_and_third, after];
        let source = mixed_list_roadmap(body_runs);
        let mut roadmap = parse_roadmap(&source)
            .expect("generated mixed-list roadmap should parse");
        let initial_parent = parent_task(&roadmap)?;
        assert_sub_task_numbers(initial_parent, &["1.1.1.1", "1.1.1.2", "1.1.1.3"])?;
        assert_child_layout(initial_parent, &[
            ChildShape::BodyList(before),
            ChildShape::SubTask(0),
            ChildShape::BodyList(between_first_and_second),
            ChildShape::SubTask(1),
            ChildShape::BodyList(between_second_and_third),
            ChildShape::SubTask(2),
            ChildShape::BodyList(after),
        ])?;

        apply_command(
            &mut roadmap,
            RoadmapOperation::Insert {
                anchor: parse_anchor("1.1.1.1")
                    .expect("first sub-task anchor should parse"),
                after: false,
            },
            Some(parse_fragment("  - [ ] 1.1.1.1. Inserted sub-task.\n")
                .expect("inserted sub-task fragment should parse")),
        )
        .expect("inserting before the first sub-task should succeed");

        let inserted_parent = parent_task(&roadmap)?;
        assert_sub_task_numbers(
            inserted_parent,
            &["1.1.1.1", "1.1.1.2", "1.1.1.3", "1.1.1.4"],
        )?;
        assert_child_layout(inserted_parent, &[
            ChildShape::BodyList(before),
            ChildShape::SubTask(0),
            ChildShape::SubTask(1),
            ChildShape::BodyList(between_first_and_second),
            ChildShape::SubTask(2),
            ChildShape::BodyList(between_second_and_third),
            ChildShape::SubTask(3),
            ChildShape::BodyList(after),
        ])?;
        let inserted_render = render_roadmap(&roadmap)
            .expect("roadmap after insertion should render");
        prop_assert!(inserted_render.contains("Requires 1.1.1.4."));
        assert_render_round_trip(
            &inserted_render,
            &["1.1.1.1", "1.1.1.2", "1.1.1.3", "1.1.1.4"],
            &[
                ChildShape::BodyList(before),
                ChildShape::SubTask(0),
                ChildShape::SubTask(1),
                ChildShape::BodyList(between_first_and_second),
                ChildShape::SubTask(2),
                ChildShape::BodyList(between_second_and_third),
                ChildShape::SubTask(3),
                ChildShape::BodyList(after),
            ],
        )?;
        roadmap = parse_roadmap(&inserted_render)
            .expect("rendered roadmap should be the next mutation's source");

        apply_command(
            &mut roadmap,
            RoadmapOperation::Delete {
                anchor: parse_anchor("1.1.1.3")
                    .expect("middle sub-task anchor should parse"),
            },
            None,
        )
        .expect("deleting the middle sub-task should succeed");

        let deleted_parent = parent_task(&roadmap)?;
        assert_sub_task_numbers(
            deleted_parent,
            &["1.1.1.1", "1.1.1.2", "1.1.1.3"],
        )?;
        assert_child_layout(deleted_parent, &[
            ChildShape::BodyList(before),
            ChildShape::SubTask(0),
            ChildShape::SubTask(1),
            ChildShape::BodyList(between_first_and_second),
            ChildShape::BodyList(between_second_and_third),
            ChildShape::SubTask(2),
            ChildShape::BodyList(after),
        ])?;

        let deleted_render = render_roadmap(&roadmap)
            .expect("roadmap after deletion should render");
        prop_assert!(deleted_render.contains("Requires 1.1.1.3."));
        prop_assert!(!deleted_render.contains(BODY_LIST_BOUNDARY_MARKER));
        if between_first_and_second > 0 && between_second_and_third > 0 {
            let joined_runs = format!(
                "  - Body between first and second item {}.\n  - Body between second and third item 0.",
                between_first_and_second - 1,
            );
            prop_assert!(deleted_render.contains(&joined_runs));
        }
        assert_render_round_trip(
            &deleted_render,
            &["1.1.1.1", "1.1.1.2", "1.1.1.3"],
            &[
                ChildShape::BodyList(before),
                ChildShape::SubTask(0),
                ChildShape::SubTask(1),
                ChildShape::BodyList(
                    between_first_and_second + between_second_and_third,
                ),
                ChildShape::SubTask(2),
                ChildShape::BodyList(after),
            ],
        )?;
    }
}

/// Build a roadmap whose four tight body-list runs surround three addenda.
fn mixed_list_roadmap(body_runs: [usize; 4]) -> String {
    let mut nested_items = String::new();
    push_body_run(&mut nested_items, "before", body_runs[0]);
    nested_items.push_str("  - [ ] 1.1.1.1. First addendum.\n");
    push_body_run(&mut nested_items, "between first and second", body_runs[1]);
    nested_items.push_str("  - [ ] 1.1.1.2. Middle addendum.\n");
    push_body_run(&mut nested_items, "between second and third", body_runs[2]);
    nested_items.push_str("  - [ ] 1.1.1.3. Last addendum.\n");
    push_body_run(&mut nested_items, "after", body_runs[3]);
    format!(
        concat!(
            "## 1. Phase\n\n",
            "### 1.1. Step\n\n",
            "- [ ] 1.1.1. Parent task.\n",
            "{nested_items}",
            "- [ ] 1.1.2. Consumer Requires 1.1.1.3.\n",
        ),
        nested_items = nested_items,
    )
}

/// Append one tight ordinary body-list run with distinct generated labels.
fn push_body_run(markdown: &mut String, label: &str, item_count: usize) {
    for item in 0..item_count {
        writeln!(markdown, "  - Body {label} item {item}.")
            .expect("writing to a String should succeed");
    }
}

/// Return the single parent task from a generated roadmap.
fn parent_task(roadmap: &RoadmapDocument) -> Result<&TaskEntry, TestCaseError> {
    roadmap
        .phases
        .first()
        .and_then(|phase| phase.steps.first())
        .and_then(|step| step.tasks().first())
        .ok_or_else(|| TestCaseError::fail("generated roadmap should contain a parent task"))
}

/// Check structural numbering and the exact interleaving of body runs.
fn assert_sub_task_numbers(task: &TaskEntry, expected: &[&str]) -> Result<(), TestCaseError> {
    let actual = task
        .sub_tasks()
        .iter()
        .map(|sub_task| sub_task.number.to_string())
        .collect::<Vec<_>>();
    let expected_numbers = expected
        .iter()
        .map(|number| (*number).to_owned())
        .collect::<Vec<_>>();
    prop_assert_eq!(actual, expected_numbers);
    Ok(())
}

/// Check child order and ensure every ordinary run remains one tight list.
fn assert_child_layout(task: &TaskEntry, expected: &[ChildShape]) -> Result<(), TestCaseError> {
    let expected_children = expected
        .iter()
        .filter(|shape| !matches!(**shape, ChildShape::BodyList(0)))
        .collect::<Vec<_>>();
    let mut sub_task_index = 0;
    prop_assert_eq!(task.children().len(), expected_children.len());
    for (child, shape) in task.children().iter().zip(expected_children) {
        match (child, shape) {
            (TaskChild::Body(body), ChildShape::BodyList(expected_items)) => {
                prop_assert!(*expected_items > 0);
                prop_assert_eq!(body.nodes().len(), 1);
                let Some(Node::List(list)) = body.nodes().first() else {
                    return Err(TestCaseError::fail("body run should remain a list"));
                };
                prop_assert!(!list.ordered);
                prop_assert!(!list.spread);
                prop_assert_eq!(list.children.len(), *expected_items);
                prop_assert!(list.children.iter().all(|node| matches!(
                    node,
                    Node::ListItem(item) if !item.spread
                )));
            }
            (TaskChild::SubTask(identity), ChildShape::SubTask(expected_index)) => {
                prop_assert_eq!(*expected_index, sub_task_index);
                prop_assert_eq!(
                    Some(identity),
                    task.sub_tasks()
                        .get(sub_task_index)
                        .map(|sub_task| &sub_task.identity),
                );
                sub_task_index += 1;
            }
            _ => {
                return Err(TestCaseError::fail(
                    "task child order differs from generated runs",
                ));
            }
        }
    }
    prop_assert_eq!(sub_task_index, task.sub_tasks().len());
    Ok(())
}

/// Ensure canonical output can be parsed and rendered without changing it.
fn assert_render_round_trip(
    rendered: &str,
    expected_numbers: &[&str],
    expected_children: &[ChildShape],
) -> Result<(), TestCaseError> {
    let reparsed =
        parse_roadmap(rendered).map_err(|error| TestCaseError::fail(error.to_string()))?;
    let parent = parent_task(&reparsed)?;
    assert_sub_task_numbers(parent, expected_numbers)?;
    assert_child_layout(parent, expected_children)?;
    let rendered_again =
        render_roadmap(&reparsed).map_err(|error| TestCaseError::fail(error.to_string()))?;
    prop_assert_eq!(rendered_again, rendered);
    Ok(())
}
