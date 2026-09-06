//! Generated coverage for roadmap item-source preservation invariants.

#[path = "support/workspace.rs"]
mod workspace_support;

use std::fmt::Write;

use mapsplice::run_from_args;
use proptest::prelude::*;
use workspace_support::create_workspace;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        .. ProptestConfig::default()
    })]

    /// Check that changing a sibling leaves a formatter-stable generated task
    /// source byte-identical while adding the new task.
    #[test]
    fn sibling_task_mutations_preserve_stable_generated_source(
        body_shape in 0u8..4,
        sub_task_count in 0usize..4,
    ) {
        let workspace = create_workspace().expect("workspace fixture should initialize");
        let stable_source = stable_task_source(body_shape, sub_task_count)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        workspace
            .write_target(&roadmap_with_stable_task(&stable_source))
            .expect("target should be written");
        workspace
            .write_fragment("- [ ] 1.1.1. Inserted sibling task.\n")
            .expect("fragment should be written");

        let stdout = render_command([
            "mapsplice",
            "insert",
            workspace.target.as_str(),
            "1.1.2",
            "--after",
            workspace.fragment.as_str(),
        ])?;

        prop_assert!(
            stdout.contains(&stable_source),
            "stable target source changed after a sibling task mutation:\n{stdout}"
        );
        prop_assert!(stdout.contains("- [ ] 1.1.3. Inserted sibling task."));
    }

    /// Exercise bounded sub-task insert, delete, and replace sequences while
    /// checking that each rendered structural marker remains contiguous.
    #[test]
    fn generated_sub_task_splices_keep_one_contiguous_child_sequence(
        initial_sub_task_count in 1usize..4,
        operations in prop::collection::vec(0u8..3, 1..6),
    ) {
        let workspace = create_workspace().expect("workspace fixture should initialize");
        workspace
            .write_target(
                &roadmap_with_sub_tasks(initial_sub_task_count)
                    .map_err(|error| TestCaseError::fail(error.to_string()))?,
            )
            .expect("target should be written");
        let mut sub_task_count = initial_sub_task_count;

        for (operation_index, operation) in operations.into_iter().enumerate() {
            let output = match operation {
                0 => {
                    workspace
                        .write_fragment(&format!(
                            "  - [ ] 1.1.1.1. Inserted sub-task {operation_index}.\n"
                        ))
                        .expect("sub-task fragment should be written");
                    render_command([
                        "mapsplice",
                        "insert",
                        workspace.target.as_str(),
                        &sub_task_anchor(sub_task_count),
                        "--after",
                        workspace.fragment.as_str(),
                    ])?
                }
                1 if sub_task_count > 1 => render_command([
                    "mapsplice",
                    "delete",
                    workspace.target.as_str(),
                    &sub_task_anchor(sub_task_count),
                ])?,
                _ => {
                    workspace
                        .write_fragment(&format!(
                            "  - [ ] 1.1.1.1. Replacement sub-task {operation_index}.\n"
                        ))
                        .expect("sub-task fragment should be written");
                    render_command([
                        "mapsplice",
                        "replace",
                        workspace.target.as_str(),
                        "1.1.1.1",
                        workspace.fragment.as_str(),
                    ])?
                }
            };

            sub_task_count = next_sub_task_count(sub_task_count, operation);
            assert_contiguous_sub_task_markers(&output, sub_task_count)?;
            workspace
                .write_target(&output)
                .expect("rendered roadmap should become the next valid target");
        }
    }

    /// Check that generated items with unstable markers or fences use the
    /// canonical renderer after a sibling mutation.
    #[test]
    fn formatter_unstable_generated_items_use_canonical_rendering(
        unstable_shape in 0u8..3,
    ) {
        let workspace = create_workspace().expect("workspace fixture should initialize");
        workspace
            .write_target(&roadmap_with_unstable_task(unstable_shape))
            .expect("target should be written");
        workspace
            .write_fragment("- [ ] 1.1.1. Inserted sibling task.\n")
            .expect("fragment should be written");

        let stdout = render_command([
            "mapsplice",
            "insert",
            workspace.target.as_str(),
            "1.1.2",
            "--after",
            workspace.fragment.as_str(),
        ])?;

        match unstable_shape {
            0 => {
                prop_assert!(stdout.contains("  1. First ordered body item."));
                prop_assert!(stdout.contains("  2. Canonical ordered body item."));
                prop_assert!(!stdout.contains("  3. Canonical ordered body item."));
            }
            1 => {
                prop_assert!(stdout.contains("  ```text"));
                prop_assert!(!stdout.contains("~~~text"));
            }
            _ => {
                prop_assert!(stdout.contains("  ```text"));
                prop_assert!(!stdout.contains("  ````text"));
            }
        }
    }

    /// Check that dependency rewrites update the affected generated text while
    /// leaving an unrelated sibling task present.
    #[test]
    fn dependency_rewrites_update_generated_task_and_sub_task_text(
        dependency_in_sub_task in any::<bool>(),
    ) {
        let workspace = create_workspace().expect("workspace fixture should initialize");
        workspace
            .write_target(&roadmap_with_rewritable_dependency(dependency_in_sub_task))
            .expect("target should be written");

        let stdout = render_command([
            "mapsplice",
            "delete",
            workspace.target.as_str(),
            "1",
        ])?;

        prop_assert!(stdout.contains("Requires 1.1.1."));
        prop_assert!(!stdout.contains("Requires 2.1.1."));
        prop_assert!(stdout.contains("Unchanged sibling task."));
    }
}

/// Run a CLI argument sequence and return its rendered standard output.
///
/// The property cases use non-in-place operations; an error is returned when
/// the command fails or does not produce standard output.
fn render_command<I, T>(args: I) -> Result<String, TestCaseError>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let outcome = run_from_args(args).map_err(|error| TestCaseError::fail(error.to_string()))?;
    outcome
        .stdout
        .ok_or_else(|| TestCaseError::fail("non-in-place operation should render stdout"))
}

/// Build a stable task source with the requested body family and sub-tasks.
///
/// The returned text includes a wrapped summary and preserves its original
/// two-space continuation indentation.
fn stable_task_source(body_shape: u8, sub_task_count: usize) -> Result<String, std::fmt::Error> {
    let mut source = concat!(
        "- [ ] 1.1.1. Stable target task deliberately wraps onto a second line while\n",
        "  preserving its two-space continuation indentation.\n"
    )
    .to_owned();
    source.push_str(stable_body(body_shape));
    for sub_task_index in 1..=sub_task_count {
        writeln!(
            source,
            "\n  - [ ] 1.1.1.{sub_task_index}. Stable nested sub-task {sub_task_index}."
        )?;
    }
    Ok(source.trim_end().to_owned())
}

/// Return one bounded body shape used by the generated preservation cases.
const fn stable_body(body_shape: u8) -> &'static str {
    match body_shape {
        0 => "\n\n  First stable paragraph.\n\n  Second stable paragraph.\n",
        1 => "\n\n  - Stable body list item.\n  - Another stable body list item.\n",
        2 => concat!(
            "\n\n  | Column | Value |\n",
            "  | --- | --- |\n",
            "  | stable | table |\n"
        ),
        _ => "\n\n  ```text\n  stable fenced content\n  ```\n",
    }
}

/// Embed a preserved task source and a mutable sibling in a valid roadmap.
fn roadmap_with_stable_task(stable_source: &str) -> String {
    format!(
        concat!(
            "# Roadmap\n\n",
            "## 1. Phase\n\n",
            "### 1.1. Step\n\n",
            "{}\n\n",
            "- [ ] 1.1.2. Mutable sibling task.\n"
        ),
        stable_source
    )
}

/// Build a roadmap whose first task owns the requested number of sub-tasks.
fn roadmap_with_sub_tasks(sub_task_count: usize) -> Result<String, std::fmt::Error> {
    let source = stable_task_source(0, sub_task_count)?;
    Ok(format!(
        "# Roadmap\n\n## 1. Phase\n\n### 1.1. Step\n\n{source}\n\n- [ ] 1.1.2. Sibling task.\n"
    ))
}

/// Return the rendered anchor for the final sub-task in a task.
fn sub_task_anchor(sub_task_count: usize) -> String { format!("1.1.1.{sub_task_count}") }

/// Compute the expected sub-task count after one generated operation.
const fn next_sub_task_count(current: usize, operation: u8) -> usize {
    match operation {
        0 => current + 1,
        1 if current > 1 => current - 1,
        _ => current,
    }
}

/// Assert that each expected sub-task marker occurs exactly once and no extra
/// marker follows the generated sequence.
fn assert_contiguous_sub_task_markers(output: &str, count: usize) -> Result<(), TestCaseError> {
    for sub_task_index in 1..=count {
        let marker = format!("- [ ] 1.1.1.{sub_task_index}.");
        let occurrences = output.matches(&marker).count();
        if occurrences != 1 {
            return Err(TestCaseError::fail(format!(
                "expected one `{marker}` marker, found {occurrences}:\n{output}"
            )));
        }
    }
    let next_marker = format!("- [ ] 1.1.1.{}.", count + 1);
    if output.contains(&next_marker) {
        return Err(TestCaseError::fail(format!(
            "unexpected non-contiguous marker `{next_marker}`:\n{output}"
        )));
    }
    Ok(())
}

/// Build a task whose body exercises one formatter-unstable shape.
fn roadmap_with_unstable_task(shape: u8) -> String {
    let body = match shape {
        0 => concat!(
            "\n\n  1. First ordered body item.\n",
            "  3. Canonical ordered body item.\n"
        ),
        1 => "\n\n  ~~~text\n  unstable tilde fence\n  ~~~\n",
        _ => "\n\n  ````text\n  unstable long fence\n  ````\n",
    };
    format!(
        concat!(
            "# Roadmap\n\n",
            "## 1. Phase\n\n",
            "### 1.1. Step\n\n",
            "- [ ] 1.1.1. Formatter-unstable target task.{}\n",
            "- [ ] 1.1.2. Mutable sibling task.\n"
        ),
        body
    )
}

/// Build a roadmap with a dependency on the task or sub-task in a later phase.
fn roadmap_with_rewritable_dependency(dependency_in_sub_task: bool) -> String {
    let dependency = "Requires 2.1.1.";
    let phase_two_task = if dependency_in_sub_task {
        format!(
            concat!(
                "- [ ] 2.1.1. Parent task.\n",
                "  - [ ] 2.1.1.1. Nested dependency task. {}\n",
                "- [ ] 2.1.2. Unchanged sibling task.\n"
            ),
            dependency
        )
    } else {
        format!(
            "- [ ] 2.1.1. Parent dependency task. {dependency}\n\n- [ ] 2.1.2. Unchanged sibling \
             task.\n"
        )
    };
    format!(
        concat!(
            "# Roadmap\n\n",
            "## 1. Deleted phase\n\n",
            "### 1.1. Deleted step\n\n",
            "- [ ] 1.1.1. Deleted task.\n\n",
            "## 2. Remaining phase\n\n",
            "### 2.1. Remaining step\n\n",
            "{}"
        ),
        phase_two_task
    )
}
