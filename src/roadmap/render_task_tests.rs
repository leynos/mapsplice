//! Regression tests for task source preservation during rendering.

use super::super::render_roadmap;
use crate::roadmap::{
    RoadmapOperation,
    apply_command,
    parse_anchor,
    parse_fragment,
    parse_roadmap,
};

#[test]
fn task_lazy_continuation_survives_same_step_cache_invalidation() {
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
}
