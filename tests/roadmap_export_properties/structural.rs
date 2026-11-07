//! Property tests for structural drift on generated documents.
//!
//! Each case renders a generated roadmap, exports it, and mutates the decoded
//! export so the committed document stands in for a stale export: one that
//! predates a source that added, removed, or reordered whole items. The drift
//! report must name the change at the level where the shape changed, and must
//! match unnumbered nested details by their source ordinal.

use proptest::prelude::*;
use serde_json::{Value, json};

use super::{
    drift::write_committed_export,
    model::{COMPACT, render, roadmap_model, structural_model},
};
use crate::export_workspace::{check_export, create_workspace, export_source};

/// One structural mutation of a committed export.
struct StructuralCase {
    /// Short description used in assertion messages.
    description: &'static str,
    /// Fragments that a drift report must not contain.
    ///
    /// A permutation must be reported as an order change, not as one item
    /// being removed and another added.
    forbidden: &'static [&'static str],
    /// Mutator applied to the decoded committed export.
    ///
    /// Returns the drift fragment the mutation must produce, so a case whose
    /// expected ordinal depends on the generated document stays precise.
    /// Mutators also report the fixture path they need but cannot find, so a
    /// reshape of the exported document surfaces as a test failure rather than
    /// a panic.
    apply: fn(&mut Value, &str) -> Result<String, String>,
}

/// Borrow an array at `pointer`, reporting when the fixture shape drifts.
fn array_at<'value>(
    root: &'value mut Value,
    pointer: &str,
) -> Result<&'value mut Vec<Value>, String> {
    root.pointer_mut(pointer)
        .and_then(Value::as_array_mut)
        .ok_or_else(|| format!("{pointer} should be a JSON array"))
}

/// Replace one field of a JSON object, reporting when it is not an object.
fn set_field(item: &mut Value, field: &str, value: Value) -> Result<(), String> {
    item.as_object_mut()
        .ok_or_else(|| "generated export item should be a JSON object".to_owned())?
        .insert(field.to_owned(), value);
    Ok(())
}

/// Clone the first array element so a case can append a variant of it.
fn cloned_first(items: &[Value]) -> Result<Value, String> {
    items
        .first()
        .cloned()
        .ok_or_else(|| "generated export should contain an item".to_owned())
}

/// Borrow the first array element, reporting when the array is empty.
fn first_mut(items: &mut [Value]) -> Result<&mut Value, String> {
    items
        .first_mut()
        .ok_or_else(|| "generated export should contain an item".to_owned())
}

/// Drop the first task so the fresh export appears to have gained it.
fn remove_first_task(root: &mut Value, _replacement: &str) -> Result<String, String> {
    let tasks = array_at(root, "/phases/0/steps/0/tasks")?;
    if tasks.is_empty() {
        return Err("generated export should contain a task".to_owned());
    }
    tasks.remove(0);
    Ok("+ task 1.1.1 added".to_owned())
}

/// Append a task that the fresh export no longer contains.
fn append_vanished_task(root: &mut Value, _replacement: &str) -> Result<String, String> {
    let tasks = array_at(root, "/phases/0/steps/0/tasks")?;
    let mut extra = cloned_first(tasks)?;
    set_field(&mut extra, "number", json!("1.1.9"))?;
    tasks.push(extra);
    Ok("- task 1.1.9 removed".to_owned())
}

/// Swap the first two tasks so their document order changes.
fn permute_tasks(root: &mut Value, _replacement: &str) -> Result<String, String> {
    let tasks = array_at(root, "/phases/0/steps/0/tasks")?;
    if tasks.len() < 2 {
        return Err("generated export should contain at least two tasks".to_owned());
    }
    tasks.swap(0, 1);
    Ok("~ step 1.1 tasks order:".to_owned())
}

/// Append a task entry the fresh export no longer contains.
fn append_vanished_task_entry(root: &mut Value, replacement: &str) -> Result<String, String> {
    let entries = array_at(root, "/phases/0/steps/0/tasks/0/entries")?;
    let ordinal = entries.len() + 1;
    let mut extra = cloned_first(entries)?;
    set_field(&mut extra, "value", json!(replacement))?;
    entries.push(extra);
    Ok(format!("- task 1.1.1 entry {ordinal} removed"))
}

/// Replace a task's dependencies so the drift report names that field.
fn replace_task_requires(root: &mut Value, _replacement: &str) -> Result<String, String> {
    let tasks = array_at(root, "/phases/0/steps/0/tasks")?;
    let task = first_mut(tasks)?;
    set_field(task, "requires", json!(["9.9.9"]))?;
    Ok("~ task 1.1.1 requires:".to_owned())
}

/// Append a sub-task that the fresh export no longer contains.
fn append_vanished_sub_task(root: &mut Value, _replacement: &str) -> Result<String, String> {
    let details = array_at(root, "/phases/0/steps/0/tasks/0/details")?;
    let mut extra = cloned_first(details)?;
    set_field(&mut extra, "number", json!("1.1.1.9"))?;
    set_field(&mut extra, "details", json!([]))?;
    details.push(extra);
    Ok("- sub-task 1.1.1.9 removed".to_owned())
}

/// Swap the first two sub-tasks so their document order changes.
fn permute_sub_tasks(root: &mut Value, _replacement: &str) -> Result<String, String> {
    let details = array_at(root, "/phases/0/steps/0/tasks/0/details")?;
    if details.len() < 2 {
        return Err("generated export should contain at least two sub-tasks".to_owned());
    }
    details.swap(0, 1);
    Ok("~ task 1.1.1 details order:".to_owned())
}

/// Replace a sub-task's dependencies so the drift report names that field.
fn replace_sub_task_requires(root: &mut Value, _replacement: &str) -> Result<String, String> {
    let details = array_at(root, "/phases/0/steps/0/tasks/0/details")?;
    let detail = first_mut(details)?;
    set_field(detail, "requires", json!(["9.9.9"]))?;
    Ok("~ sub-task 1.1.1.1 requires:".to_owned())
}

/// Drop the last nested detail so the fresh export appears to have gained it.
fn remove_last_nested_detail(root: &mut Value, _replacement: &str) -> Result<String, String> {
    let nested = array_at(root, "/phases/0/steps/0/tasks/0/details/0/details")?;
    let ordinal = nested.len();
    if ordinal == 0 {
        return Err("generated export should contain a nested detail".to_owned());
    }
    nested.pop();
    Ok(format!("+ detail {ordinal} of sub-task 1.1.1.1 added"))
}

/// Append a nested detail that the fresh export no longer contains.
fn append_vanished_nested_detail(root: &mut Value, _replacement: &str) -> Result<String, String> {
    let nested = array_at(root, "/phases/0/steps/0/tasks/0/details/0/details")?;
    let ordinal = nested.len() + 1;
    let extra = cloned_first(nested)?;
    nested.push(extra);
    Ok(format!("- detail {ordinal} of sub-task 1.1.1.1 removed"))
}

/// Swap the first two nested details so their ordinal contents change.
///
/// Unnumbered details have no stable identifier, so the drift report matches
/// them by source ordinal. A permutation therefore changes what each ordinal
/// holds rather than adding or removing an item. The first ordinal's title is
/// overwritten with a digit-bearing marker, which generated titles cannot
/// contain, so the content change is always observable.
fn swap_nested_details(root: &mut Value, _replacement: &str) -> Result<String, String> {
    let nested = array_at(root, "/phases/0/steps/0/tasks/0/details/0/details")?;
    if nested.len() < 2 {
        return Err("generated export should contain at least two nested details".to_owned());
    }
    nested.swap(0, 1);
    set_field(first_mut(nested)?, "title", json!("ordinal-1"))?;
    Ok("~ detail 1 of sub-task 1.1.1.1 title:".to_owned())
}

/// Append a step that the fresh export no longer contains.
fn append_vanished_step(root: &mut Value, _replacement: &str) -> Result<String, String> {
    let steps = array_at(root, "/phases/0/steps")?;
    let mut extra = cloned_first(steps)?;
    set_field(&mut extra, "number", json!("1.9"))?;
    set_field(&mut extra, "tasks", json!([]))?;
    steps.push(extra);
    Ok("- step 1.9 removed".to_owned())
}

/// Append a phase that the fresh export no longer contains.
fn append_vanished_phase(root: &mut Value, _replacement: &str) -> Result<String, String> {
    let phases = array_at(root, "/phases")?;
    let mut extra = cloned_first(phases)?;
    set_field(&mut extra, "number", json!("9"))?;
    set_field(&mut extra, "steps", json!([]))?;
    phases.push(extra);
    Ok("- phase 9 removed".to_owned())
}

/// Structural mutations covering additions, removals, and permutations.
const STRUCTURAL_CASES: &[StructuralCase] = &[
    StructuralCase {
        description: "task added to the source",
        forbidden: &[],
        apply: remove_first_task,
    },
    StructuralCase {
        description: "task removed from the source",
        forbidden: &[],
        apply: append_vanished_task,
    },
    StructuralCase {
        description: "task order changed",
        forbidden: &[
            "task 1.1.1 added",
            "task 1.1.1 removed",
            "task 1.1.2 added",
            "task 1.1.2 removed",
        ],
        apply: permute_tasks,
    },
    StructuralCase {
        description: "task entry removed from the source",
        forbidden: &[],
        apply: append_vanished_task_entry,
    },
    StructuralCase {
        description: "task dependencies changed",
        forbidden: &[],
        apply: replace_task_requires,
    },
    StructuralCase {
        description: "sub-task removed from the source",
        forbidden: &[],
        apply: append_vanished_sub_task,
    },
    StructuralCase {
        description: "sub-task order changed",
        forbidden: &[
            "sub-task 1.1.1.1 added",
            "sub-task 1.1.1.1 removed",
            "sub-task 1.1.1.2 added",
            "sub-task 1.1.1.2 removed",
        ],
        apply: permute_sub_tasks,
    },
    StructuralCase {
        description: "sub-task dependencies changed",
        forbidden: &[],
        apply: replace_sub_task_requires,
    },
    StructuralCase {
        description: "nested detail added to the source",
        forbidden: &[],
        apply: remove_last_nested_detail,
    },
    StructuralCase {
        description: "nested detail removed from the source",
        forbidden: &[],
        apply: append_vanished_nested_detail,
    },
    StructuralCase {
        description: "unnumbered nested details matched by ordinal",
        forbidden: &[
            "detail 1 of sub-task 1.1.1.1 added",
            "detail 1 of sub-task 1.1.1.1 removed",
            "detail 2 of sub-task 1.1.1.1 added",
            "detail 2 of sub-task 1.1.1.1 removed",
        ],
        apply: swap_nested_details,
    },
    StructuralCase {
        description: "step removed from the source",
        forbidden: &[],
        apply: append_vanished_step,
    },
    StructuralCase {
        description: "phase removed from the source",
        forbidden: &[],
        apply: append_vanished_phase,
    },
];

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 8,
        .. ProptestConfig::default()
    })]

    /// A generated document exported twice must not report drift.
    #[test]
    fn identical_generated_export_reports_no_drift(model in roadmap_model()) {
        let workspace = create_workspace().expect("workspace fixture should initialize");
        let source = render(&model, COMPACT);
        workspace
            .write_target(&source)
            .expect("generated roadmap should be written");
        let committed = export_source(&source).expect("generated export should succeed");
        let export_path = write_committed_export(&workspace, &committed)
            .expect("committed export should be written");

        let outcome = check_export(&workspace, &export_path)
            .expect("an identical generated export should not drift");

        prop_assert!(
            outcome.stdout.is_none(),
            "a matching export should not report differences: {:?}",
            outcome.stdout
        );
    }

    /// Each structural mutation is named at the level where the shape changed.
    #[test]
    fn drift_reports_structural_changes(
        model in structural_model(),
        replacement in "[a-z]{4,10}",
    ) {
        let workspace = create_workspace().expect("workspace fixture should initialize");
        let source = render(&model, COMPACT);
        workspace
            .write_target(&source)
            .expect("generated roadmap should be written");
        let committed = export_source(&source).expect("generated export should succeed");

        for case in STRUCTURAL_CASES {
            let mut mutated: Value =
                serde_json::from_str(&committed).expect("export should decode as JSON");
            let expected = (case.apply)(&mut mutated, &replacement)
                .expect("structural mutation should apply to the generated export");
            let encoded =
                serde_json::to_string(&mutated).expect("mutated export should encode as JSON");
            let export_path = write_committed_export(&workspace, &encoded)
                .expect("committed export should be written");

            let error = check_export(&workspace, &export_path)
                .expect_err("a structurally changed committed export should drift");
            let message = error.to_string();
            prop_assert!(
                message.contains(&expected),
                "{}: expected {:?} in drift report: {message}",
                case.description,
                expected
            );
            for fragment in case.forbidden {
                prop_assert!(
                    !message.contains(fragment),
                    "{}: unexpected {:?} in drift report: {message}",
                    case.description,
                    fragment
                );
            }
        }
    }
}
