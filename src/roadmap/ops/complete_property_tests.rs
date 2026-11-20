//! Property tests for completion idempotence and anchor preservation.

use proptest::prelude::*;

use super::{collect_anchors, complete_item};
use crate::roadmap::{parse_anchor, parse_roadmap};

/// Suffix marker that generated titles cannot contain.
const SUFFIX: &str = "∎ marked";

proptest! {
    #[test]
    fn completion_is_idempotent_and_keeps_the_anchor_set(
        words in prop::collection::vec("[a-z]{1,10}", 1..8),
        trailing_period in any::<bool>(),
        use_suffix in any::<bool>(),
    ) {
        let mut title = words.join(" ");
        if trailing_period {
            title.push('.');
        }
        let source = format!(
            "## 1. Phase one\n\n### 1.1. Step one\n\n- [ ] 1.1.1. {title}\n"
        );
        let roadmap = parse_roadmap(&source).expect("generated roadmap should parse");
        let anchor = parse_anchor("1.1.1").expect("fixture anchor should parse");
        let suffix = use_suffix.then_some(SUFFIX);

        let first = complete_item(&roadmap, &source, anchor, suffix)
            .expect("completion should succeed");

        prop_assert!(
            first.contains("- [x] 1.1.1. "),
            "completed text should tick the item: {}",
            first
        );
        if use_suffix {
            let unwrapped = first.split_whitespace().collect::<Vec<_>>().join(" ");
            prop_assert_eq!(
                unwrapped.matches(SUFFIX).count(),
                1,
                "the suffix should appear exactly once: {}",
                first
            );
        }
        let reparsed = parse_roadmap(&first).expect("completed roadmap should parse");
        prop_assert_eq!(
            collect_anchors(&roadmap),
            collect_anchors(&reparsed),
            "the anchor set should not change"
        );
        let second = complete_item(&reparsed, &first, anchor, suffix)
            .expect("second completion should succeed");
        prop_assert_eq!(second, first, "the second completion should be a no-op");
    }
}
