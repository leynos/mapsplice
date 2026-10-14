//! Regression tests for task source preservation during rendering.

use super::super::render_roadmap;
use crate::{
    observability::metrics_snapshot,
    roadmap::{RoadmapOperation, apply_command, parse_anchor, parse_fragment, parse_roadmap},
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
fn mixed_nested_body_bullet_keeps_renumbered_addendum_tight() {
    let source = concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. First task.\n",
        "- [ ] 1.1.2. Parent task.\n",
        "  - A non-structural body bullet.\n",
        "  - [ ] 1.1.2.1. Sub one.\n",
    );
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

    let rendered = render_roadmap(&roadmap).expect("roadmap should render");

    assert_eq!(
        rendered,
        concat!(
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. First task.\n",
            "- [ ] 1.1.2. Inserted task.\n",
            "- [ ] 1.1.3. Parent task.\n",
            "  - A non-structural body bullet.\n",
            "  - [ ] 1.1.3.1. Sub one.\n",
        )
    );
}

#[test]
fn tight_body_list_keeps_separators_without_adjacent_sub_task() {
    let source = concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. First task.\n",
        "- [ ] 1.1.2. Parent task.\n",
        "  - Ordinary body bullet.\n",
        "  - Another body bullet.\n",
        "- [ ] 1.1.3. Later task.\n",
    );
    let mut roadmap = parse_roadmap(source).expect("roadmap should parse");
    let anchor = parse_anchor("1.1.2").expect("anchor should parse");
    let fragment =
        parse_fragment("- [ ] 1.1.2. Inserted task.\n").expect("task fragment should parse");

    apply_command(
        &mut roadmap,
        RoadmapOperation::Insert {
            anchor,
            after: false,
        },
        Some(fragment),
    )
    .expect("task insertion should succeed");

    let rendered = render_roadmap(&roadmap).expect("roadmap should render");

    assert_eq!(
        rendered,
        concat!(
            "## 1. Phase one\n\n",
            "### 1.1. Step one\n\n",
            "- [ ] 1.1.1. First task.\n",
            "- [ ] 1.1.2. Inserted task.\n",
            "- [ ] 1.1.3. Parent task.\n\n",
            "  - Ordinary body bullet.\n",
            "  - Another body bullet.\n",
            "- [ ] 1.1.4. Later task.\n",
        )
    );
}
