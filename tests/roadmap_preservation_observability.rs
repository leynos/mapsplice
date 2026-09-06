//! Serial integration coverage for source-preservation metric reason paths.

#[path = "support/workspace.rs"]
mod workspace_support;

use mapsplice::{MetricsSnapshot, metrics_snapshot, run_from_args};
use workspace_support::{TestResult, create_workspace};

/// Verify stable task reuse increments preservation without mutation or
/// canonical-fallback counters.
#[test]
#[serial_test::serial(cli_env)]
fn stable_task_sources_record_preserved_rendering_only() {
    let before = metrics_snapshot();
    let stdout = run_preservation_operation(
        stable_task_roadmap(),
        "- [ ] 1.1.1. Inserted task.\n",
        ["insert", "1.1.2", "--after"],
    )
    .expect("stable task operation should succeed");
    let after = metrics_snapshot();

    assert!(stdout.contains("preserving its two-space continuation indentation."));
    assert_eq!(
        delta(
            before.preserved_source_renders,
            after.preserved_source_renders
        ),
        2
    );
    assert_no_preservation_mutation_or_fallback(before, after);
}

/// Verify a sub-task splice records only the parent's child-mutation reason.
#[test]
#[serial_test::serial(cli_env)]
fn sub_task_splice_records_child_mutation_only() {
    let before = metrics_snapshot();
    let stdout = run_preservation_operation(
        roadmap_with_sub_tasks(),
        "  - [ ] 1.1.1.1. Inserted sub-task.\n",
        ["insert", "1.1.1.2", "--after"],
    )
    .expect("sub-task splice should succeed");
    let after = metrics_snapshot();

    assert!(stdout.contains("- [ ] 1.1.1.3. Inserted sub-task."));
    assert_eq!(
        delta(
            before.invalidations_child_mutation,
            after.invalidations_child_mutation
        ),
        1
    );
    assert_eq!(
        delta(before.invalidations_renumber, after.invalidations_renumber),
        0
    );
    assert_eq!(
        delta(
            before.invalidations_dependency_rewrite,
            after.invalidations_dependency_rewrite,
        ),
        0
    );
    assert_no_canonical_fallback(before, after);
}

/// Verify inserting before existing tasks records only renumber invalidation.
#[test]
#[serial_test::serial(cli_env)]
fn task_insertion_before_existing_tasks_records_renumbering_only() {
    let before = metrics_snapshot();
    let stdout = run_preservation_operation(
        stable_task_roadmap(),
        "- [ ] 1.1.1. Inserted task.\n",
        ["insert", "1.1.1", ""],
    )
    .expect("task insertion should succeed");
    let after = metrics_snapshot();

    assert!(stdout.contains("- [ ] 1.1.3. Mutable sibling task."));
    assert_eq!(
        delta(before.invalidations_renumber, after.invalidations_renumber),
        2
    );
    assert_eq!(
        delta(
            before.invalidations_child_mutation,
            after.invalidations_child_mutation
        ),
        0
    );
    assert_eq!(
        delta(
            before.invalidations_dependency_rewrite,
            after.invalidations_dependency_rewrite,
        ),
        0
    );
    assert_no_canonical_fallback(before, after);
}

/// Verify a phase insertion records dependency-rewrite invalidation for the
/// affected unchanged task.
#[test]
#[serial_test::serial(cli_env)]
fn phase_insertion_records_dependency_rewrite_for_unchanged_task() {
    let before = metrics_snapshot();
    let stdout = run_preservation_operation(
        dependency_roadmap(),
        concat!(
            "## 9. Inserted phase\n\n",
            "### 9.1. Inserted step\n\n",
            "- [ ] 9.1.1. Inserted task.\n"
        ),
        ["insert", "1", "--after"],
    )
    .expect("phase insertion should succeed");
    let after = metrics_snapshot();

    assert!(stdout.contains("Requires 3.1.1."));
    assert_eq!(
        delta(
            before.invalidations_dependency_rewrite,
            after.invalidations_dependency_rewrite,
        ),
        1
    );
    assert_eq!(
        delta(
            before.invalidations_child_mutation,
            after.invalidations_child_mutation
        ),
        0
    );
    assert_no_canonical_fallback(before, after);
}

/// Verify list-marker and code-fence instability select their matching
/// canonical-fallback reason.
#[test]
#[serial_test::serial(cli_env)]
fn unstable_sources_record_only_their_canonical_fallback_reason() {
    assert_fallback_reason(
        concat!(
            "- [ ] 1.1.1. Unstable marker task.\n\n",
            "  1. First item.\n",
            "  3. Third item.\n"
        ),
        CanonicalFallback::ListMarker,
    );
    assert_fallback_reason(
        concat!(
            "- [ ] 1.1.1. Unstable fence task.\n\n",
            "  ~~~text\n",
            "  unstable fence\n",
            "  ~~~\n"
        ),
        CanonicalFallback::CodeFence,
    );
}

#[derive(Clone, Copy)]
enum CanonicalFallback {
    ListMarker,
    CodeFence,
}

/// Assert that exactly the expected canonical-fallback reason is recorded for
/// a generated unstable task source.
fn assert_fallback_reason(source: &str, expected: CanonicalFallback) {
    let before = metrics_snapshot();
    let target = format!(
        "# Roadmap\n\n## 1. Phase\n\n### 1.1. Step\n\n{source}\n- [ ] 1.1.2. Anchor task.\n"
    );
    let _stdout = match run_preservation_operation(
        &target,
        "- [ ] 1.1.1. Inserted task.\n",
        ["insert", "1.1.2", "--after"],
    ) {
        Ok(stdout) => stdout,
        Err(error) => panic!("formatter fallback operation should succeed: {error}"),
    };
    let after = metrics_snapshot();

    let list_fallbacks = delta(
        before.canonical_fallbacks_unstable_list_marker,
        after.canonical_fallbacks_unstable_list_marker,
    );
    let fence_fallbacks = delta(
        before.canonical_fallbacks_unstable_code_fence,
        after.canonical_fallbacks_unstable_code_fence,
    );
    match expected {
        CanonicalFallback::ListMarker => {
            assert_eq!(list_fallbacks, 1);
            assert_eq!(fence_fallbacks, 0);
        }
        CanonicalFallback::CodeFence => {
            assert_eq!(list_fallbacks, 0);
            assert_eq!(fence_fallbacks, 1);
        }
    }
    assert_eq!(
        delta(before.canonical_fallbacks, after.canonical_fallbacks),
        1
    );
    assert_eq!(
        delta(
            before.preserved_source_invalidations,
            after.preserved_source_invalidations
        ),
        0
    );
}

/// Run one structural operation in a temporary workspace and return its output.
///
/// The command array contains the operation, anchor, and optional placement
/// flag; failures are propagated through the integration-test result type.
fn run_preservation_operation(
    target: &str,
    fragment: &str,
    command: [&str; 3],
) -> TestResult<String> {
    let workspace = create_workspace()?;
    workspace.write_target(target)?;
    workspace.write_fragment(fragment)?;
    let mut arguments = vec![
        "mapsplice",
        command[0],
        workspace.target.as_str(),
        command[1],
    ];
    if !command[2].is_empty() {
        arguments.push(command[2]);
    }
    arguments.push(workspace.fragment.as_str());
    run_from_args(arguments)?
        .stdout
        .ok_or_else(|| "stdout mode should return rendered roadmap".into())
}

/// Assert that a stable preservation operation records no invalidation or
/// canonical fallback.
fn assert_no_preservation_mutation_or_fallback(before: MetricsSnapshot, after: MetricsSnapshot) {
    assert_eq!(
        delta(
            before.preserved_source_invalidations,
            after.preserved_source_invalidations
        ),
        0
    );
    assert_eq!(
        delta(before.canonical_fallbacks, after.canonical_fallbacks),
        0
    );
}

/// Assert that a mutation operation records no canonical fallback of any kind.
fn assert_no_canonical_fallback(before: MetricsSnapshot, after: MetricsSnapshot) {
    assert_eq!(
        delta(before.canonical_fallbacks, after.canonical_fallbacks),
        0
    );
    assert_eq!(
        delta(
            before.canonical_fallbacks_unstable_list_marker,
            after.canonical_fallbacks_unstable_list_marker,
        ),
        0
    );
    assert_eq!(
        delta(
            before.canonical_fallbacks_unstable_code_fence,
            after.canonical_fallbacks_unstable_code_fence,
        ),
        0
    );
}

/// Return a saturating counter delta for a before-and-after snapshot pair.
const fn delta(before: u64, after: u64) -> u64 { after.saturating_sub(before) }

/// Return a roadmap containing a wrapped stable task and mutable sibling.
const fn stable_task_roadmap() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. Phase\n\n",
        "### 1.1. Step\n\n",
        "- [ ] 1.1.1. Stable task deliberately wraps onto a second line while\n",
        "  preserving its two-space continuation indentation.\n\n",
        "- [ ] 1.1.2. Mutable sibling task.\n"
    )
}

/// Return a roadmap containing a parent task with two stable sub-tasks.
const fn roadmap_with_sub_tasks() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. Phase\n\n",
        "### 1.1. Step\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  - [ ] 1.1.1.1. Stable first sub-task.\n",
        "  - [ ] 1.1.1.2. Stable second sub-task.\n"
    )
}

/// Return a two-phase roadmap with a dependency that a phase insertion rewrites.
const fn dependency_roadmap() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. First phase\n\n",
        "### 1.1. First step\n\n",
        "- [ ] 1.1.1. Task that Requires 2.1.1.\n\n",
        "## 2. Later phase\n\n",
        "### 2.1. Later step\n\n",
        "- [ ] 2.1.1. Later task.\n"
    )
}
