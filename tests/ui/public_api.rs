//! Integration smoke test for the public `mapsplice` API surface.

use mapsplice::{
    CommandKind,
    GlobalOptions,
    RoadmapAnchor,
    parse_anchor,
    parse_fragment_text,
    parse_roadmap_text,
};

fn main() {
    let anchor: RoadmapAnchor = parse_anchor("1.2.3").expect("anchor should parse");
    let _command = CommandKind::Delete { anchor };
    let _global = GlobalOptions { in_place: true };
    let _roadmap = parse_roadmap_text(
        "## 1. Phase\n\n### 1.1. Step\n\n- [ ] 1.1.1. Task.\n",
    )
    .expect("roadmap should parse");
    let _fragment = parse_fragment_text("- [ ] 1.1.1. Task.\n")
        .expect("fragment should parse");
}
