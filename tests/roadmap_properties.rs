//! Property tests for roadmap numbering and dependency rewrite invariants.

#[path = "support/properties.rs"]
mod support;

use mapsplice::{parse_anchor, run_from_args};
use proptest::prelude::*;
use support::{PHASE_FRAGMENT, create_workspace};

proptest! {
    #[test]
    fn canonical_anchors_roundtrip(
        phase in 1u32..1_000,
        step in 1u32..1_000,
        task in 1u32..1_000,
        level in 1u8..=3,
    ) {
        let raw = match level {
            1 => phase.to_string(),
            2 => format!("{phase}.{step}"),
            _ => format!("{phase}.{step}.{task}"),
        };

        let anchor = parse_anchor(&raw).expect("generated canonical anchor should parse");

        prop_assert_eq!(anchor.to_string(), raw);
    }

    #[test]
    fn noncanonical_positive_anchors_are_rejected(
        phase in 1u32..1_000,
        step in 1u32..1_000,
        task in 1u32..1_000,
    ) {
        let candidates = [
            format!("0.{step}.{task}"),
            format!("{phase}.0.{task}"),
            format!("{phase}.{step}.0"),
            format!("0{phase}"),
            format!("{phase}.0{step}"),
            format!("{phase}.{step}.0{task}"),
        ];

        for candidate in candidates {
            prop_assert!(
                parse_anchor(&candidate).is_err(),
                "noncanonical anchor parsed: {candidate}"
            );
        }
    }

    #[test]
    fn inserting_phase_rewrites_generated_dependency_references(
        phase_count in 2usize..8,
        insert_before in 1usize..8,
    ) {
        let insertion_anchor = insert_before.min(phase_count);
        let workspace = create_workspace().expect("workspace fixture should initialize");
        workspace
            .write_target(&numbered_phase_roadmap(phase_count))
            .expect("target should be written");
        workspace
            .write_fragment(PHASE_FRAGMENT)
            .expect("fragment should be written");

        let outcome = run_from_args([
            "mapsplice",
            "insert",
            workspace.target.as_str(),
            &insertion_anchor.to_string(),
            workspace.fragment.as_str(),
        ])
        .expect("insert command should succeed");
        let stdout = outcome.stdout.unwrap_or_default();

        for original in insertion_anchor..=phase_count {
            let new_phase = original + 1;
            prop_assert!(
                stdout.contains(&format!("Requires {new_phase}.1.1.")),
                "missing rewritten dependency for original phase {original}: {stdout}"
            );
        }
    }
}

fn numbered_phase_roadmap(phase_count: usize) -> String {
    (1..=phase_count)
        .map(|phase| {
            format!(
                "## {phase}. Original phase {phase}\n\n### {phase}.1. Step {phase}\n\n- [ ] \
                 {phase}.1.1. Task {phase}. Requires {phase}.1.1.\n"
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
