//! Regression tests for task source preservation during rendering.

use markdown::mdast::Node;
use rstest::rstest;

use super::super::{render_inline, render_roadmap};
use crate::{
    observability::metrics_snapshot,
    roadmap::{
        RoadmapOperation,
        apply_command,
        model::TaskChild,
        parse_anchor,
        parse_fragment,
        parse_roadmap,
    },
};

#[test]
fn task_lazy_continuation_survives_same_step_cache_invalidation() {
    let before = metrics_snapshot();
    let source = concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. This untouched task keeps its two-space lazy continuation\n",
        "  indentation after a sibling invalidates the step cache.\n",
        "- [ ] 1.1.2. Insert after this anchor.\n",
    );
    let mut roadmap = parse_roadmap(source).expect("roadmap should parse");
    let anchor = parse_anchor("1.1.2").expect("anchor should parse");
    let fragment =
        parse_fragment("- [ ] 1.1.3. Inserted sibling.\n").expect("fragment should parse");

    apply_command(
        &mut roadmap,
        RoadmapOperation::Insert {
            anchor,
            after: true,
        },
        Some(fragment),
    )
    .expect("sibling insertion should succeed");

    let rendered = render_roadmap(&roadmap).expect("roadmap should render");

    assert_eq!(
        rendered,
        concat!(
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. This untouched task keeps its two-space lazy continuation\n",
            "  indentation after a sibling invalidates the step cache.\n",
            "- [ ] 1.1.2. Insert after this anchor.\n",
            "- [ ] 1.1.3. Inserted sibling.\n",
        )
    );
    assert_eq!(
        metrics_snapshot(),
        before,
        "direct rendering must not record process-local preservation metrics"
    );
}

#[test]
fn deleting_sub_task_keeps_paragraph_body_separate_from_summary() {
    let source = concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n\n",
        "  Paragraph body.\n\n",
        "  - [ ] 1.1.1.1. Addendum to remove.\n",
        "  - Ordinary body list item.\n",
        "  - [ ] 1.1.1.2. Later addendum.\n",
    );
    let mut roadmap = parse_roadmap(source).expect("roadmap should parse");
    let anchor = parse_anchor("1.1.1.1").expect("sub-task anchor should parse");

    apply_command(&mut roadmap, RoadmapOperation::Delete { anchor }, None)
        .expect("sub-task deletion should succeed");

    let rendered = render_roadmap(&roadmap).expect("deleted roadmap should render");
    assert_eq!(
        rendered,
        concat!(
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. Parent task.\n\n",
            "  Paragraph body.\n\n",
            "  - Ordinary body list item.\n",
            "  - [ ] 1.1.1.1. Later addendum.\n",
        )
    );

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
    let Some(TaskChild::Body(paragraph_body)) = task.children().first() else {
        panic!("the first task child should contain the paragraph body");
    };
    let Some(Node::Paragraph(paragraph)) = paragraph_body.nodes().first() else {
        panic!("the first task body block should be a paragraph");
    };
    assert_eq!(
        render_inline(&paragraph.children).expect("paragraph body should render"),
        "Paragraph body."
    );
}

#[rstest]
#[case::body_bullet_beside_addendum(
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. First task.\n",
        "- [ ] 1.1.2. Parent task.\n",
        "  - A non-structural body bullet.\n",
        "  - [ ] 1.1.2.1. Sub one.\n",
    ),
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. First task.\n",
        "- [ ] 1.1.2. Inserted task.\n",
        "- [ ] 1.1.3. Parent task.\n",
        "  - A non-structural body bullet.\n",
        "  - [ ] 1.1.3.1. Sub one.\n",
    ),
)]
#[case::tight_body_list_without_addendum(
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. First task.\n",
        "- [ ] 1.1.2. Parent task.\n",
        "  - Ordinary body bullet.\n",
        "  - Another body bullet.\n",
        "- [ ] 1.1.3. Later task.\n",
    ),
    concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. First task.\n",
        "- [ ] 1.1.2. Inserted task.\n",
        "- [ ] 1.1.3. Parent task.\n\n",
        "  - Ordinary body bullet.\n",
        "  - Another body bullet.\n",
        "- [ ] 1.1.4. Later task.\n",
    ),
)]
fn inserting_before_parent_renders_body_separators(#[case] source: &str, #[case] expected: &str) {
    let mut roadmap = parse_roadmap(source).expect("roadmap should parse");
    let anchor = parse_anchor("1.1.2").expect("anchor should parse");
    let fragment = parse_fragment("- [ ] 1.1.2. Inserted task.\n").expect("fragment should parse");

    apply_command(
        &mut roadmap,
        RoadmapOperation::Insert {
            anchor,
            after: false,
        },
        Some(fragment),
    )
    .expect("task insertion should succeed");

    assert_eq!(
        render_roadmap(&roadmap).expect("roadmap should render"),
        expected
    );
}
