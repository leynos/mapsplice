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

    #[test]
    fn generated_invalid_dependency_tokens_are_preserved(
        phase in 1u32..20,
        step in 1u32..20,
        task in 1u32..20,
        sub_task in 1u32..20,
        extra in 1u32..20,
    ) {
        let candidates = [
            format!("{phase}.0.{task}"),
            format!("{phase}.{step}.0"),
            format!("{phase}.0.{task}.{sub_task}"),
            format!("{phase}.{step}.{task}.0"),
            format!("{phase}.{step}.{task}.{sub_task}.{extra}"),
        ];

        for candidate in candidates {
            let workspace = create_workspace().expect("workspace fixture should initialize");
            workspace
                .write_target(&roadmap_with_dependency_text(&format!("Requires {candidate}, 2.1.1.")))
                .expect("target should be written");

            let outcome = run_from_args(["mapsplice", "delete", workspace.target.as_str(), "1"])
                .expect("delete command should succeed");
            let stdout = outcome.stdout.unwrap_or_default();

            prop_assert!(
                stdout.contains(&format!("Requires {candidate}, 1.1.1.")),
                "invalid dependency token changed or mapped reference missing: {stdout}"
            );
        }
    }

    #[test]
    fn generated_incidental_numeric_tokens_are_preserved(
        phase in 1u32..20,
        step in 1u32..20,
        task in 1u32..20,
    ) {
        let incidental_token = format!("{phase}.{step}.{task}");
        let workspace = create_workspace().expect("workspace fixture should initialize");
        workspace
            .write_target(&roadmap_with_dependency_text(&format!(
                "See {incidental_token}. Requires 2.1.1."
            )))
            .expect("target should be written");

        let outcome = run_from_args(["mapsplice", "delete", workspace.target.as_str(), "1"])
            .expect("delete command should succeed");
        let stdout = outcome.stdout.unwrap_or_default();

        prop_assert!(
            stdout.contains(&format!("See {incidental_token}. Requires 1.1.1.")),
            "incidental token changed or mapped reference missing: {stdout}"
        );
    }

    #[test]
    fn scoped_reference_generated_incidental_tokens_are_preserved(
        section_phase in 1u32..20,
        section_step in 1u32..20,
        version_major in 1u32..20,
        version_minor in 0u32..20,
        version_patch in 0u32..20,
        count in 1u32..200,
    ) {
        let section_reference = format!("§{section_phase}.{section_step}");
        let version = format!("{version_major}.{version_minor}.{version_patch}");
        let workspace = create_workspace().expect("workspace fixture should initialize");
        workspace
            .write_target(&roadmap_with_dependency_text(&format!(
                "See {section_reference}. Released {version}. Count {count}. Requires 2.1.1."
            )))
            .expect("target should be written");

        let outcome = run_from_args(["mapsplice", "delete", workspace.target.as_str(), "1"])
            .expect("delete command should succeed");
        let stdout = outcome.stdout.unwrap_or_default();

        prop_assert!(
            stdout.contains(&format!(
                "See {section_reference}. Released {version}. Count {count}. Requires 1.1.1."
            )),
            "scoped reference incidental text changed or mapped reference missing: {stdout}"
        );
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

fn roadmap_with_dependency_text(text: &str) -> String {
    format!(
        "# Example\n\n## 1. Phase one\n\n### 1.1. Step one\n\n- [ ] 1.1.1. First task.\n\n## 2. \
         Phase two\n\n### 2.1. Step two\n\n- [ ] 2.1.1. Second task. {text}\n"
    )
}
