//! Unit tests for roadmap render fidelity.

use super::render_roadmap;
use crate::roadmap::parse_roadmap;

#[test]
fn exact_nested_sub_task_round_trip() {
    let source = concat!(
        "# Example\n\n",
        "## 8. Phase one\n\n",
        "### 8.2. Step one\n\n",
        "- [ ] 8.2.3. Parent task.\n",
        "    Body before.\n",
        "    - [ ] 8.2.3.1. Nested sub-task.\n",
        "    Body after.",
    );
    let roadmap = parse_roadmap(source).expect("nested sub-task roadmap should parse");

    let rendered = render_roadmap(&roadmap).expect("nested sub-task roadmap should render");

    assert_eq!(rendered, source);
}
