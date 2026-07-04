//! Property tests for roadmap numbering and dependency rewrite invariants.

#[path = "support/formatter_boundary.rs"]
mod formatter_boundary;
#[path = "support/properties.rs"]
mod support;

use formatter_boundary::{
    assert_house_format_noop,
    formatter_unstable_boundary_shape,
    gate_clean_boundary_shape,
};
use mapsplice::{MapspliceError, parse_anchor, run_from_args};
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
    fn generated_valid_dangling_dependency_references_are_reported(
        phase in 3u32..1_000,
        step in 1u32..1_000,
        task in 1u32..1_000,
    ) {
        let raw_anchor = format!("{phase}.{step}.{task}");
        let workspace = create_workspace().expect("workspace fixture should initialize");
        workspace
            .write_target(&roadmap_with_dependency_text(&format!("Requires {raw_anchor}.")))
            .expect("target should be written");

        let error = run_from_args(["mapsplice", "delete", workspace.target.as_str(), "1"])
            .expect_err("dangling dependency reference should fail");

        let MapspliceError::DanglingDependency { anchor } = error else {
            return Err(TestCaseError::fail(format!(
                "expected dangling dependency error for {raw_anchor}, got {error:?}"
            )));
        };
        prop_assert_eq!(anchor.to_string(), raw_anchor);
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

    #[test]
    fn gate_clean_noop_preserves_stable_boundary_bodies(
        shape in gate_clean_boundary_shape(),
    ) {
        let workspace = create_workspace().expect("workspace fixture should initialize");
        let target = shape.target();
        workspace
            .write_target(&target)
            .expect("target should be written");
        workspace
            .write_fragment("- [ ] 1.1.1. Existing task.\n")
            .expect("fragment should be written");

        let outcome = run_from_args([
            "mapsplice",
            "replace",
            workspace.target.as_str(),
            "1.1.1",
            workspace.fragment.as_str(),
        ])
        .expect("no-op replace command should succeed");
        let stdout = outcome.stdout.unwrap_or_default();

        prop_assert_eq!(stdout, target);
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        .. ProptestConfig::default()
    })]

    #[test]
    fn append_preserves_generated_untouched_task_lists(
        spacing in 0usize..3,
        child_order in 0usize..3,
        dependency in 0usize..3,
    ) {
        let workspace = create_workspace().expect("workspace fixture should initialize");
        let preserved_phase = preserved_append_phase(spacing, child_order, dependency);
        workspace
            .write_target(&format!(
                "# Example\n\n{preserved_phase}\n## 2. Existing phase two\n\n### 2.1. Existing step two\n\n- [ ] 2.1.1. Existing task two.\n"
            ))
            .expect("target should be written");
        workspace
            .write_fragment(PHASE_FRAGMENT)
            .expect("fragment should be written");

        let outcome = run_from_args([
            "mapsplice",
            "append",
            workspace.target.as_str(),
            workspace.fragment.as_str(),
        ])
        .expect("append command should succeed");
        let stdout = outcome.stdout.unwrap_or_default();

        prop_assert!(
            stdout.contains(&preserved_phase),
            "untouched phase changed after append:\nexpected fragment:\n{preserved_phase}\nrendered:\n{stdout}"
        );
        prop_assert!(
            stdout.contains("Requires 3.1.1."),
            "appended fragment dependency was not renumbered: {stdout}"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 12,
        .. ProptestConfig::default()
    })]

    #[test]
    fn formatter_unstable_noop_normalizes_to_documented_boundary(
        shape in formatter_unstable_boundary_shape(),
    ) {
        let workspace = create_workspace().expect("workspace fixture should initialize");
        let target = shape.target();
        let expected = shape.expected();
        workspace
            .write_target(&target)
            .expect("target should be written");
        workspace
            .write_fragment("- [ ] 1.1.1. Existing task.\n")
            .expect("fragment should be written");

        let outcome = run_from_args([
            "mapsplice",
            "replace",
            workspace.target.as_str(),
            "1.1.1",
            workspace.fragment.as_str(),
        ])
        .expect("no-op replace command should succeed");
        let stdout = outcome.stdout.unwrap_or_default();

        prop_assert_eq!(&stdout, &expected);
        assert_house_format_noop(&stdout).map_err(TestCaseError::fail)?;
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

fn preserved_append_phase(spacing: usize, child_order: usize, dependency: usize) -> String {
    format!(
        "## 1. Existing phase\n\n### 1.1. Existing step\n\n{}",
        preserved_task_list(spacing, child_order, dependency)
    )
}

fn preserved_task_list(spacing: usize, child_order: usize, dependency: usize) -> String {
    let incidental_text = match dependency {
        0 => "See §1.1.",
        1 => "Released 1.4.0.",
        _ => "Count 3.",
    };
    match (spacing, child_order) {
        (0, _) => format!(
            "- [ ] 1.1.1. First existing task. {incidental_text}\n- [ ] 1.1.2. Second existing \
             task.\n"
        ),
        (1, 0) => format!(
            "- [ ] 1.1.1. First existing task. {incidental_text}\n  - [ ] 1.1.1.1. First \
             sub-task.\n\n- [ ] 1.1.2. Second existing task.\n"
        ),
        (1, _) => format!(
            "- [ ] 1.1.1. First existing task. {incidental_text}\n  - [ ] 1.1.1.1. First \
             sub-task.\n  - [ ] 1.1.1.2. Second sub-task.\n\n- [ ] 1.1.2. Second existing task.\n"
        ),
        (_, 0) => format!(
            "- [ ] 1.1.1. First existing task. {incidental_text}\n\n  - Supporting note stays \
             attached.\n\n- [ ] 1.1.2. Second existing task.\n"
        ),
        (_, 1) => format!(
            "- [ ] 1.1.1. First existing task. {incidental_text}\n\n  1. Ordered body item.\n  2. \
             Second ordered body item.\n\n- [ ] 1.1.2. Second existing task.\n"
        ),
        _ => format!(
            "- [ ] 1.1.1. First existing task. {incidental_text}\n\n  - Supporting note before \
             sub-tasks.\n\n  - [ ] 1.1.1.1. First sub-task.\n\n- [ ] 1.1.2. Second existing \
             task.\n"
        ),
    }
}
