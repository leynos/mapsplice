//! Compile-fail coverage for roadmap model invariants.

use mapsplice::parse_roadmap_text;

fn main() {
    let mut roadmap = parse_roadmap_text(concat!(
        "## 1. Phase\n\n",
        "### 1.1. Step\n\n",
        "- [ ] 1.1.1. Task.\n",
        "  - [ ] 1.1.1.1. Sub-task.\n",
    ))
    .expect("roadmap should parse");

    let task = &mut roadmap.phases[0].steps[0].tasks[0];
    task.sub_tasks.clear();
}
