//! Tests for body-list boundaries around roadmap task addenda.

use markdown::mdast::Node;
use rstest::rstest;

use super::super::{render_inline, render_roadmap};
use crate::roadmap::{
    RoadmapOperation,
    apply_command,
    model::TaskChild,
    parse_anchor,
    parse_roadmap,
};

#[test]
fn deleting_later_sub_task_separates_parent_paragraph_after_sub_task() {
    let source = concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  - [ ] 1.1.1.1. Kept addendum.\n",
        "  - [ ] 1.1.1.2. Addendum to remove.\n\n",
        "  Paragraph body.\n",
    );
    let mut roadmap = parse_roadmap(source).expect("roadmap should parse");
    let anchor = parse_anchor("1.1.1.2").expect("sub-task anchor should parse");

    apply_command(&mut roadmap, RoadmapOperation::Delete { anchor }, None)
        .expect("sub-task deletion should succeed");

    let rendered = render_roadmap(&roadmap).expect("roadmap should render");
    assert_eq!(
        rendered,
        concat!(
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. Parent task.\n",
            "  - [ ] 1.1.1.1. Kept addendum.\n\n",
            "  Paragraph body.\n",
        )
    );

    let reparsed = parse_roadmap(&rendered).expect("rendered roadmap should reparse");
    let task = reparsed
        .phases
        .first()
        .and_then(|phase| phase.steps.first())
        .and_then(|step| step.tasks().first())
        .expect("reparsed roadmap should contain its parent task");
    assert_eq!(task.children().len(), 2);
    assert!(matches!(
        task.children().first(),
        Some(TaskChild::SubTask(_))
    ));
    let Some(TaskChild::Body(body)) = task.children().get(1) else {
        panic!("the parent paragraph should follow its structural sub-task");
    };
    let Some(Node::Paragraph(paragraph)) = body.nodes().first() else {
        panic!("the parent body should start with a paragraph");
    };
    assert_eq!(
        render_inline(&paragraph.children).expect("paragraph body should render"),
        "Paragraph body."
    );
    let sub_task = task
        .sub_tasks()
        .first()
        .expect("the kept structural sub-task should remain");
    assert_eq!(
        render_inline(sub_task.summary.nodes()).expect("sub-task summary should render"),
        "Kept addendum."
    );
    assert_eq!(
        render_roadmap(&reparsed).expect("reparsed roadmap should render"),
        rendered,
        "the parent paragraph boundary should survive a parse-render round trip"
    );
}

#[rstest]
#[case::ordered_list_starting_above_one(
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  - [ ] 1.1.1.1. Addendum to remove.\n\n",
        "  3. Ordered body item.\n",
    ),
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n\n",
        "  3. Ordered body item.\n",
    ),
    vec![Some(3)],
)]
#[case::ordered_lists_on_each_side_of_sub_task(
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  1. Before body item.\n",
        "  - [ ] 1.1.1.1. Addendum to remove.\n",
        "  1. After body item.\n",
    ),
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n\n",
        "  1. Before body item.\n\n",
        "  <!-- mapsplice: preserve body list boundary -->\n\n",
        "  1. After body item.\n",
    ),
    vec![Some(1), Some(1)],
)]
#[case::ordered_lists_starting_above_one(
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n\n",
        "  3. Before body item.\n",
        "  - [ ] 1.1.1.1. Addendum to remove.\n",
        "  3. After body item.\n",
    ),
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n\n",
        "  3. Before body item.\n\n",
        "  <!-- mapsplice: preserve body list boundary -->\n\n",
        "  3. After body item.\n",
    ),
    vec![Some(3), Some(3)],
)]
fn deleting_sub_task_preserves_ordered_body_list_boundaries(
    #[case] source: &str,
    #[case] expected: &str,
    #[case] expected_ordered_starts: Vec<Option<u32>>,
) {
    let mut roadmap = parse_roadmap(source).expect("roadmap should parse");
    let anchor = parse_anchor("1.1.1.1").expect("sub-task anchor should parse");

    apply_command(&mut roadmap, RoadmapOperation::Delete { anchor }, None)
        .expect("sub-task deletion should succeed");

    let rendered = render_roadmap(&roadmap).expect("deleted roadmap should render");
    assert_eq!(rendered, expected);

    let reparsed = parse_roadmap(&rendered).expect("rendered roadmap should reparse");
    let task = reparsed
        .phases
        .first()
        .and_then(|phase| phase.steps.first())
        .and_then(|step| step.tasks().first())
        .expect("reparsed roadmap should contain its parent task");
    assert_eq!(
        render_inline(task.summary.nodes()).expect("task summary should render"),
        "Parent task."
    );
    let mut ordered_starts = Vec::new();
    for child in task.children() {
        if let TaskChild::Body(body) = child {
            for node in body.nodes() {
                if let Node::List(list) = node
                    && list.ordered
                {
                    ordered_starts.push(list.start);
                }
            }
        }
    }
    assert_eq!(ordered_starts, expected_ordered_starts);
    assert_eq!(
        render_roadmap(&reparsed).expect("reparsed roadmap should render"),
        expected,
        "ordered list boundaries should survive a parse-render round trip"
    );
}

#[rstest]
#[case::loose_ordered_then_tight_ordered(
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  1. Loose first item.\n\n",
        "  2. Loose second item.\n",
        "  - [ ] 1.1.1.1. Addendum to remove.\n",
        "  1. Tight list starts over.\n",
    ),
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n\n",
        "  1. Loose first item.\n\n",
        "  2. Loose second item.\n\n",
        "  <!-- mapsplice: preserve body list boundary -->\n\n",
        "  1. Tight list starts over.\n",
    ),
    vec![(true, Some(1), true), (true, Some(1), false)],
)]
#[case::loose_bullets_then_tight_bullets(
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  - Loose first item.\n\n",
        "  - Loose second item.\n",
        "  * [ ] 1.1.1.1. Addendum to remove.\n",
        "  - Tight body item.\n",
    ),
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n\n",
        "  - Loose first item.\n\n",
        "  - Loose second item.\n\n",
        "  <!-- mapsplice: preserve body list boundary -->\n\n",
        "  - Tight body item.\n",
    ),
    vec![(false, None, true), (false, None, false)],
)]
fn deleting_sub_task_preserves_mixed_spread_body_list_boundaries(
    #[case] source: &str,
    #[case] expected: &str,
    #[case] expected_list_shapes: Vec<(bool, Option<u32>, bool)>,
) {
    let mut roadmap = parse_roadmap(source).expect("roadmap should parse");
    let anchor = parse_anchor("1.1.1.1").expect("sub-task anchor should parse");

    apply_command(&mut roadmap, RoadmapOperation::Delete { anchor }, None)
        .expect("sub-task deletion should succeed");

    let rendered = render_roadmap(&roadmap).expect("roadmap should render");
    assert_eq!(rendered, expected);

    let reparsed = parse_roadmap(&rendered).expect("rendered roadmap should reparse");
    let task = reparsed
        .phases
        .first()
        .and_then(|phase| phase.steps.first())
        .and_then(|step| step.tasks().first())
        .expect("reparsed roadmap should contain its parent task");
    let mut list_shapes = Vec::new();
    for child in task.children() {
        if let TaskChild::Body(body) = child {
            for node in body.nodes() {
                if let Node::List(list) = node {
                    list_shapes.push((list.ordered, list.start, list.spread));
                }
            }
        }
    }
    assert_eq!(list_shapes, expected_list_shapes);
    assert_eq!(
        render_roadmap(&reparsed).expect("reparsed roadmap should render"),
        expected,
        "mixed-spread body lists should survive a parse-render round trip"
    );
}
