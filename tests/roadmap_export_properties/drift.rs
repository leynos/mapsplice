//! Property test for drift reporting at every nesting level.

use proptest::prelude::*;
use serde_json::{Value, json};

use crate::export_workspace::{check_export, create_workspace, export_source};

/// One JSON mutation and the drift fragment it must produce.
struct DriftCase {
    /// Short description used in assertion messages.
    description: &'static str,
    /// JSON pointer to the field that the mutation replaces.
    pointer: &'static str,
    /// Fragment that the drift report must contain.
    expected: &'static str,
    /// Mutator that returns whether its target path existed.
    apply: fn(&mut Value, &str, &str) -> bool,
}

/// Replace a string field.
fn set_string(root: &mut Value, pointer: &str, replacement: &str) -> bool {
    root.pointer_mut(pointer).is_some_and(|slot| {
        *slot = Value::String(replacement.to_owned());
        true
    })
}

/// Replace a checkbox state field.
fn set_state(root: &mut Value, pointer: &str, _replacement: &str) -> bool {
    root.pointer_mut(pointer).is_some_and(|slot| {
        *slot = json!("done");
        true
    })
}

/// Replace a dependency list with an anchor that is not in the roadmap.
fn set_requires(root: &mut Value, pointer: &str, _replacement: &str) -> bool {
    root.pointer_mut(pointer).is_some_and(|slot| {
        *slot = json!(["9.9.9"]);
        true
    })
}

/// Replace the export format version.
fn set_format_version(root: &mut Value, _pointer: &str, _replacement: &str) -> bool {
    root.pointer_mut("/format_version").is_some_and(|slot| {
        *slot = json!(2);
        true
    })
}

/// Mutations covering every nesting level of the exported document.
const DRIFT_CASES: &[DriftCase] = &[
    DriftCase {
        description: "format version",
        pointer: "/format_version",
        expected: "~ format_version:",
        apply: set_format_version,
    },
    DriftCase {
        description: "phase title",
        pointer: "/phases/0/title",
        expected: "~ phase 1 title",
        apply: set_string,
    },
    DriftCase {
        description: "phase requires",
        pointer: "/phases/0/requires",
        expected: "~ phase 1 requires",
        apply: set_requires,
    },
    DriftCase {
        description: "phase entry",
        pointer: "/phases/0/entries/0/value",
        expected: "~ phase 1 entry 1 value",
        apply: set_string,
    },
    DriftCase {
        description: "step title",
        pointer: "/phases/0/steps/0/title",
        expected: "~ step 1.1 title",
        apply: set_string,
    },
    DriftCase {
        description: "step question",
        pointer: "/phases/0/steps/0/question",
        expected: "~ step 1.1 question",
        apply: set_string,
    },
    DriftCase {
        description: "step requires",
        pointer: "/phases/0/steps/0/requires",
        expected: "~ step 1.1 requires",
        apply: set_requires,
    },
    DriftCase {
        description: "step entry",
        pointer: "/phases/0/steps/0/entries/0/value",
        expected: "~ step 1.1 entry 1 value",
        apply: set_string,
    },
    DriftCase {
        description: "step trailing entry",
        pointer: "/phases/0/steps/0/trailing/0/value",
        expected: "~ step 1.1 trailing entry 1 value",
        apply: set_string,
    },
    DriftCase {
        description: "task title",
        pointer: "/phases/0/steps/0/tasks/0/title",
        expected: "~ task 1.1.1 title",
        apply: set_string,
    },
    DriftCase {
        description: "task state",
        pointer: "/phases/0/steps/0/tasks/0/state",
        expected: "~ task 1.1.1 state",
        apply: set_state,
    },
    DriftCase {
        description: "task requires",
        pointer: "/phases/0/steps/0/tasks/0/requires",
        expected: "~ task 1.1.1 requires",
        apply: set_requires,
    },
    DriftCase {
        description: "task entry",
        pointer: "/phases/0/steps/0/tasks/0/entries/0/value",
        expected: "~ task 1.1.1 entry 1 value",
        apply: set_string,
    },
    DriftCase {
        description: "sub-task title",
        pointer: "/phases/0/steps/0/tasks/0/details/0/title",
        expected: "~ sub-task 1.1.1.1 title",
        apply: set_string,
    },
    DriftCase {
        description: "sub-task state",
        pointer: "/phases/0/steps/0/tasks/0/details/0/state",
        expected: "~ sub-task 1.1.1.1 state",
        apply: set_state,
    },
    DriftCase {
        description: "sub-task requires",
        pointer: "/phases/0/steps/0/tasks/0/details/0/requires",
        expected: "~ sub-task 1.1.1.1 requires",
        apply: set_requires,
    },
    DriftCase {
        description: "sub-task entry",
        pointer: "/phases/0/steps/0/tasks/0/details/0/entries/0/value",
        expected: "~ sub-task 1.1.1.1 entry 1 value",
        apply: set_string,
    },
    DriftCase {
        description: "nested detail title",
        pointer: "/phases/0/steps/0/tasks/0/details/0/details/0/title",
        expected: "~ detail 1 of sub-task 1.1.1.1 title",
        apply: set_string,
    },
    DriftCase {
        description: "nested detail state",
        pointer: "/phases/0/steps/0/tasks/0/details/0/details/0/state",
        expected: "~ detail 1 of sub-task 1.1.1.1 state",
        apply: set_state,
    },
    DriftCase {
        description: "nested detail entry",
        pointer: "/phases/0/steps/0/tasks/0/details/0/details/0/entries/0/value",
        expected: "~ detail 1 of sub-task 1.1.1.1 entry 1 value",
        apply: set_string,
    },
];

/// A roadmap with content at every export nesting level.
const NESTED_ROADMAP: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "Phase note: phase value.\n\n",
    "### 1.1. Step one\n\n",
    "What question?\n\n",
    "Step note: step value.\n\n",
    "- [ ] 1.1.1. Task one.\n\n",
    "  Task note: task value.\n\n",
    "  - [ ] 1.1.1.1. Detail one.\n\n",
    "    Detail note: detail value.\n\n",
    "    - [ ] Nested detail.\n\n",
    "      Nested note: nested value.\n\n",
    "Step close: step trailing value.\n",
);

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 8,
        .. ProptestConfig::default()
    })]

    #[test]
    fn drift_reports_mutations_at_every_nesting_level(replacement in "[a-z]{4,10}") {
        let workspace = create_workspace().expect("workspace fixture should initialize");
        workspace
            .write_target(NESTED_ROADMAP)
            .expect("fixture roadmap should be written");
        let committed = export_source(NESTED_ROADMAP).expect("fixture export should succeed");
        let export_path = workspace.target.with_file_name("roadmap.json");

        for case in DRIFT_CASES {
            let mut mutated: Value =
                serde_json::from_str(&committed).expect("export should decode as JSON");
            prop_assert!(
                (case.apply)(&mut mutated, case.pointer, &replacement),
                "{}: pointer {:?} is missing from the export",
                case.description,
                case.pointer
            );
            let encoded =
                serde_json::to_string(&mutated).expect("mutated export should encode as JSON");
            workspace
                .dir
                .write("roadmap.json", &encoded)
                .expect("mutated export should be written");

            let error = check_export(&workspace, &export_path)
                .expect_err("a mutated committed export should drift");
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
