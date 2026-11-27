//! Property tests for completion byte preservation and idempotence.

use proptest::prelude::*;

use super::complete_item;
use crate::roadmap::{parse_anchor, parse_roadmap};

/// Suffix marker that generated titles cannot contain.
const SUFFIX: &str = "∎ marked";

/// Document header kept identical by every generated case.
const HEADER: &str = "## 1. Phase one\n\n### 1.1. Step one\n\n";

proptest! {
    #[test]
    fn completion_changes_only_the_checkbox_and_suffix(
        words in prop::collection::vec("[a-z]{1,6}", 1..5),
        trailing_period in any::<bool>(),
        use_suffix in any::<bool>(),
    ) {
        let core = words.join(" ");
        let title = if trailing_period { format!("{core}.") } else { core.clone() };
        let source = format!("{HEADER}- [ ] 1.1.1. {title}\n");
        let expected = if use_suffix {
            let marker = if trailing_period {
                format!("{core} {SUFFIX}.")
            } else {
                format!("{core} {SUFFIX}")
            };
            format!("{HEADER}- [x] 1.1.1. {marker}\n")
        } else {
            format!("{HEADER}- [x] 1.1.1. {title}\n")
        };
        let roadmap = parse_roadmap(&source).expect("generated roadmap should parse");
        let anchor = parse_anchor("1.1.1").expect("fixture anchor should parse");
        let suffix = use_suffix.then_some(SUFFIX);

        let first = complete_item(&roadmap, &source, anchor, suffix)
            .expect("completion should succeed");

        prop_assert_eq!(
            &first,
            &expected,
            "completion should change only the checkbox and the inserted suffix"
        );
        let reparsed = parse_roadmap(&first).expect("completed roadmap should parse");
        let second = complete_item(&reparsed, &first, anchor, suffix)
            .expect("second completion should succeed");
        prop_assert_eq!(second, first, "the second completion should be a no-op");
    }
}
