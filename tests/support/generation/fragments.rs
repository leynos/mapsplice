//! Render the fragment each edit splices into the generated target.
//!
//! Most edits share one multi-task fragment, while an in-place replacement
//! supplies exactly the addressed task. Keeping the fragment rendering beside
//! its tests lets the generator stay within the module size the lint allows
//! without separating the fragment's bytes from the model that predicts them.

use std::fmt::{self, Write as _};

use super::{Shape, fragment_label, render_task, sub_task_indent};
use crate::oracle::{Edit, FRAGMENT_TASKS, ItemId, Level, Model};

/// Render the fragment an edit splices in.
pub(super) fn render_fragment(
    edit: Edit,
    shape: Shape,
    model: &Model,
) -> Result<String, fmt::Error> {
    match edit {
        Edit::ReplaceTaskInPlace { task } => render_in_place_fragment(task, shape, model),
        Edit::InsertTasksBefore { .. }
        | Edit::DeleteTask { .. }
        | Edit::ReplaceTask { .. }
        | Edit::DeleteStep { .. }
        | Edit::DeletePhase { .. } => render_fragment_tasks(),
    }
}

/// Render the multi-task fragment an insert or retiring replacement splices.
///
/// A task fragment is recognized by its three-level anchor, so the tasks are
/// spelled `1.1.n` even though the fragment is a standalone document. The
/// target's numbering replaces those numbers on splice, so they are only a
/// parse-time device, not an identity. No clause is generated inside the
/// fragment: a fragment clause would resolve differently depending on which
/// target it lands in and so could not be predicted from the model alone.
fn render_fragment_tasks() -> Result<String, fmt::Error> {
    let mut document = String::new();
    for index in 0..FRAGMENT_TASKS {
        writeln!(
            document,
            "- [ ] 1.1.{}. {}",
            index + 1,
            fragment_label(index)
        )?;
    }
    Ok(document)
}

/// Render the one-task fragment an in-place replacement splices.
///
/// The task is the addressed task rendered exactly as the target renders it:
/// the same anchor, summary, checkbox, clauses, incidental prose, and
/// sub-tasks. That is what makes the edit an identity-preserving one: the
/// fragment carries everything the retired item carried, so no edge can be
/// left dangling by text the replacement dropped. Nothing but the addressed
/// task is written, so the operation remains a single-item replacement.
fn render_in_place_fragment(
    task: usize,
    shape: Shape,
    model: &Model,
) -> Result<String, fmt::Error> {
    render_task(
        ItemId::new(Level::Task, task),
        shape,
        model,
        sub_task_indent(shape),
    )
}

#[cfg(test)]
mod tests {
    //! Tests that each edit's fragment carries what the oracle assumes.

    use super::super::{Shape, generate};
    use crate::oracle::Edit;

    /// The in-place fragment reproduces a tail task, sub-tasks and all.
    ///
    /// Task 5 is the tail task of step 1, so its target block owns two
    /// sub-tasks. The oracle keeps every identity beneath the addressed task
    /// alive for this edit; if the fragment dropped the sub-tasks the property
    /// would not find their blocks, so this test pins the reachable case rather
    /// than trusting the generator's sampling to hit it.
    #[test]
    fn in_place_fragment_reproduces_the_addressed_task() {
        let generated = generate(
            &[0],
            Shape { options: 0 },
            Edit::ReplaceTaskInPlace { task: 5 },
        )
        .expect("in-place fragment generation should succeed");

        assert!(
            generated
                .fragment
                .starts_with("- [ ] 1.2.3. Generated task 5\n"),
            "the fragment must open with the addressed task, got:\n{}",
            generated.fragment
        );
        for sub_task in [
            "\n  - [ ] 1.2.3.1. Generated sub-task 5-0\n",
            "\n  - [ ] 1.2.3.2. Generated sub-task 5-1\n",
        ] {
            assert!(
                generated.fragment.contains(sub_task),
                "the fragment must carry the sub-task `{sub_task:?}`, got:\n{}",
                generated.fragment
            );
        }
        assert_eq!(
            generated
                .fragment
                .lines()
                .filter(|line| line.starts_with("- [ ]"))
                .count(),
            1,
            "an in-place fragment must hold exactly the addressed task, got:\n{}",
            generated.fragment
        );
    }

    /// The retiring replacement keeps its multi-task fragment.
    #[test]
    fn retiring_replacement_still_splices_two_fragment_tasks() {
        let generated = generate(&[0], Shape { options: 0 }, Edit::ReplaceTask { task: 5 })
            .expect("retiring fragment generation should succeed");

        assert_eq!(
            generated.fragment.lines().count(),
            2,
            "the retiring variant must keep its two-task fragment, got:\n{}",
            generated.fragment
        );
        assert!(
            generated
                .fragment
                .contains("1.1.1. Generated fragment task 0"),
            "the fragment must use its own numbering, got:\n{}",
            generated.fragment
        );
    }
}
