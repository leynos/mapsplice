//! Integration smoke test for the public `mapsplice` API surface.

use camino::Utf8PathBuf;
use mapsplice::{
    CommandKind,
    ExportError,
    ExportFormat,
    GlobalOptions,
    MapspliceError,
    RoadmapAnchor,
    parse_anchor,
    parse_fragment_text,
    parse_roadmap_text,
};

/// The format name must be usable where a `const` string is required.
const JSON_FORMAT: &str = ExportFormat::Json.as_str();

/// `ExportError::class` must stay callable from a `const fn` body.
const fn export_class(error: &ExportError) -> &'static str { ExportError::class(error) }

/// `MapspliceError::class` must stay callable from a `const fn` body.
const fn mapsplice_class(error: &MapspliceError) -> &'static str { MapspliceError::class(error) }

fn main() {
    assert_eq!(JSON_FORMAT, "json");

    let drift = ExportError::Drift {
        export: Utf8PathBuf::from("roadmap.json"),
        roadmap: Utf8PathBuf::from("roadmap.md"),
        differences: String::new(),
        regenerate: String::new(),
    };
    assert_eq!(export_class(&drift), "export_drift");

    let invalid = ExportError::InvalidCommitted {
        path: Utf8PathBuf::from("roadmap.json"),
        message: String::new(),
    };
    assert_eq!(export_class(&invalid), "invalid_export");

    let serialization = ExportError::Serialization {
        message: String::new(),
    };
    assert_eq!(export_class(&serialization), "json_serialization");
    assert_eq!(
        mapsplice_class(&MapspliceError::Export(serialization)),
        "json_serialization"
    );
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
