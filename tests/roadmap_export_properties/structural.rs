//! Property tests for structural drift: additions, removals, and reordering.
//!
//! Unlike the field-level property, these cases mutate the shape of the
//! committed export. A stale export can differ from the source by gaining,
//! losing, or reordering whole items, and the drift report must name that
//! change at the level where it happened.

use proptest::prelude::*;
use serde_json::{Value, json};

use super::drift::{NESTED_ROADMAP, write_committed_export};
use crate::export_workspace::{check_export, create_workspace, export_source};

/// One structural mutation of the committed export and the drift it must report.
///
/// The committed document stands in for a stale export: mutating it models a
/// source that has added, removed, or reordered items since the export was
/// written.
struct StructuralCase {
    /// Short description used in assertion messages.
    description: &'static str,
    /// Fragment that the drift report must contain.
    expected: &'static str,
    /// Mutator applied to the decoded committed export.
    ///
    /// A mutator reports the fixture path it needs but cannot find, so a
    /// reshape of the exported document surfaces as a test failure rather
    /// than a panic.
    apply: fn(&mut Value, &str) -> Result<(), String>,
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
        .ok_or_else(|| "fixture export item should be a JSON object".to_owned())?
        .insert(field.to_owned(), value);
    Ok(())
}

/// Clone the first array element so a case can append a variant of it.
fn cloned_first(items: &[Value]) -> Result<Value, String> {
    items
        .first()
        .cloned()
        .ok_or_else(|| "fixture export should contain an item".to_owned())
}

/// Drop the first task so the fresh export appears to have gained it.
fn remove_first_task(root: &mut Value, _replacement: &str) -> Result<(), String> {
    let tasks = array_at(root, "/phases/0/steps/0/tasks")?;
    if tasks.is_empty() {
        return Err("fixture export should contain a task".to_owned());
    }
    tasks.remove(0);
    Ok(())
}

/// Append a task that the fresh export no longer contains.
fn append_vanished_task(root: &mut Value, _replacement: &str) -> Result<(), String> {
    let tasks = array_at(root, "/phases/0/steps/0/tasks")?;
    let mut extra = cloned_first(tasks)?;
    set_field(&mut extra, "number", json!("1.1.9"))?;
    tasks.push(extra);
    Ok(())
}

/// Insert extra tasks around the original so the task order changes.
fn reorder_tasks(root: &mut Value, _replacement: &str) -> Result<(), String> {
    let tasks = array_at(root, "/phases/0/steps/0/tasks")?;
    let mut trailing = cloned_first(tasks)?;
    set_field(&mut trailing, "number", json!("1.1.8"))?;
    let mut leading = cloned_first(tasks)?;
    set_field(&mut leading, "number", json!("1.1.9"))?;
    tasks.insert(0, leading);
    tasks.push(trailing);
    Ok(())
}

/// Append a task entry the fresh export no longer contains.
fn append_vanished_task_entry(root: &mut Value, replacement: &str) -> Result<(), String> {
    let entries = array_at(root, "/phases/0/steps/0/tasks/0/entries")?;
    let mut extra = cloned_first(entries)?;
    set_field(&mut extra, "value", json!(replacement))?;
    entries.push(extra);
    Ok(())
}

/// Append a sub-task that the fresh export no longer contains.
fn append_vanished_sub_task(root: &mut Value, _replacement: &str) -> Result<(), String> {
    let details = array_at(root, "/phases/0/steps/0/tasks/0/details")?;
    let mut extra = cloned_first(details)?;
    set_field(&mut extra, "number", json!("1.1.1.9"))?;
    set_field(&mut extra, "details", json!([]))?;
    details.push(extra);
    Ok(())
}

/// Append a phase that the fresh export no longer contains.
fn append_vanished_phase(root: &mut Value, _replacement: &str) -> Result<(), String> {
    let phases = array_at(root, "/phases")?;
    let mut extra = cloned_first(phases)?;
    set_field(&mut extra, "number", json!("2"))?;
    set_field(&mut extra, "steps", json!([]))?;
    phases.push(extra);
    Ok(())
}

/// Structural mutations covering additions, removals, and reordering.
const STRUCTURAL_CASES: &[StructuralCase] = &[
    StructuralCase {
        description: "task added to the source",
        expected: "+ task 1.1.1 added",
        apply: remove_first_task,
    },
    StructuralCase {
        description: "task removed from the source",
        expected: "- task 1.1.9 removed",
        apply: append_vanished_task,
    },
    StructuralCase {
        description: "task order changed",
        expected: "~ step 1.1 tasks order:",
        apply: reorder_tasks,
    },
    StructuralCase {
        description: "task entry removed from the source",
        expected: "- task 1.1.1 entry 2 removed",
        apply: append_vanished_task_entry,
    },
    StructuralCase {
        description: "sub-task removed from the source",
        expected: "- sub-task 1.1.1.9 removed",
        apply: append_vanished_sub_task,
    },
    StructuralCase {
        description: "phase removed from the source",
        expected: "- phase 2 removed",
        apply: append_vanished_phase,
    },
];

/// An export that matches the source exactly must not report drift.
#[test]
fn identical_committed_export_reports_no_drift() {
    let workspace = create_workspace().expect("workspace fixture should initialize");
    workspace
        .write_target(NESTED_ROADMAP)
        .expect("fixture roadmap should be written");
    let committed = export_source(NESTED_ROADMAP).expect("fixture export should succeed");
    let export_path =
        write_committed_export(&workspace, &committed).expect("committed export should be written");

    let outcome = check_export(&workspace, &export_path)
        .expect("an identical committed export should not drift");

    assert!(
        outcome.stdout.is_none(),
        "a matching export should not report differences: {:?}",
        outcome.stdout
    );
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 8,
        .. ProptestConfig::default()
    })]

    /// Each structural mutation is named at the level where the shape changed.
    #[test]
    fn drift_reports_structural_changes(replacement in "[a-z]{4,10}") {
        let workspace = create_workspace().expect("workspace fixture should initialize");
        workspace
            .write_target(NESTED_ROADMAP)
            .expect("fixture roadmap should be written");
        let committed = export_source(NESTED_ROADMAP).expect("fixture export should succeed");

        for case in STRUCTURAL_CASES {
            let mut mutated: Value =
                serde_json::from_str(&committed).expect("export should decode as JSON");
            (case.apply)(&mut mutated, &replacement)
                .expect("structural mutation should apply to the fixture export");
            let encoded =
                serde_json::to_string(&mutated).expect("mutated export should encode as JSON");
            let export_path = write_committed_export(&workspace, &encoded)
                .expect("committed export should be written");

            let error = check_export(&workspace, &export_path)
                .expect_err("a structurally changed committed export should drift");
            let message = error.to_string();
            prop_assert!(
                message.contains(case.expected),
                "{}: expected {:?} in drift report: {message}",
                case.description,
                case.expected
            );
        }
    }
}
