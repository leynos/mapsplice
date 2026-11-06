//! Integration smoke test for the public `mapsplice` API surface.

use mapsplice::{
    CommandKind,
    ExportFormat,
    GlobalOptions,
    RoadmapAnchor,
    parse_anchor,
    parse_fragment_text,
    parse_roadmap_text,
};

/// The format name must be usable where a `const` string is required.
const JSON_FORMAT: &str = ExportFormat::Json.as_str();

fn main() {
    assert_eq!(JSON_FORMAT, "json");
    let anchor: RoadmapAnchor = parse_anchor("1.2.3").expect("anchor should parse");
    let _command = CommandKind::Delete { anchor };
    let _export = CommandKind::Export {
        format: ExportFormat::Json,
        check: None,
    };
    let _global = GlobalOptions { in_place: true };
    let _roadmap = parse_roadmap_text(
        "## 1. Phase\n\n### 1.1. Step\n\n- [ ] 1.1.1. Task.\n",
    )
    .expect("roadmap should parse");
    let _fragment = parse_fragment_text("- [ ] 1.1.1. Task.\n")
        .expect("fragment should parse");
}
