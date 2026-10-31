//! Unit tests for semantic export drift reporting.

use crate::roadmap::export::{
    drift::differences,
    schema::{
        ExportDetail,
        ExportDocument,
        ExportEntry,
        ExportPhase,
        ExportState,
        ExportStep,
        ExportTask,
        FORMAT_VERSION,
    },
};

/// Construct a plain export entry with nested entries.
fn entry(key: Option<&str>, value: &str, nested: Vec<ExportEntry>) -> ExportEntry {
    ExportEntry {
        key: key.map(str::to_owned),
        value: value.to_owned(),
        entries: nested,
    }
}

/// Construct an export detail.
fn detail(
    number: Option<&str>,
    title: &str,
    entries: Vec<ExportEntry>,
    nested: Vec<ExportDetail>,
) -> ExportDetail {
    ExportDetail {
        number: number.map(str::to_owned),
        title: title.to_owned(),
        state: ExportState::Open,
        requires: Vec::new(),
        entries,
        details: nested,
    }
}

/// Construct a task with one entry and one structural detail.
fn task(number: &str) -> ExportTask {
    ExportTask {
        number: number.to_owned(),
        title: "Task title".to_owned(),
        state: ExportState::Open,
        requires: Vec::new(),
        entries: vec![entry(None, "Task entry", Vec::new())],
        details: vec![detail(Some("1.1.1.1"), "Detail", Vec::new(), Vec::new())],
    }
}

/// Construct a step with one task.
fn step(number: &str) -> ExportStep {
    ExportStep {
        number: number.to_owned(),
        title: "Step title".to_owned(),
        question: Some("Question?".to_owned()),
        entries: Vec::new(),
        requires: Vec::new(),
        tasks: vec![task("1.1.1")],
        trailing: Vec::new(),
    }
}

/// Construct a phase with one step.
fn phase(number: &str) -> ExportPhase {
    ExportPhase {
        number: number.to_owned(),
        title: "Phase title".to_owned(),
        entries: Vec::new(),
        requires: Vec::new(),
        steps: vec![step("1.1")],
        trailing: Vec::new(),
    }
}

/// Construct a document with the supplied phases.
fn document(phases: Vec<ExportPhase>) -> ExportDocument {
    ExportDocument {
        format_version: FORMAT_VERSION,
        phases,
    }
}

/// Borrow the first phase of a fixture document.
fn phase_mut(document: &mut ExportDocument) -> &mut ExportPhase {
    document
        .phases
        .first_mut()
        .expect("fixture should have a phase")
}

/// Borrow the first step of the first phase.
fn step_mut(document: &mut ExportDocument) -> &mut ExportStep {
    phase_mut(document)
        .steps
        .first_mut()
        .expect("fixture should have a step")
}

/// Borrow the first task of the first step.
fn task_mut(document: &mut ExportDocument) -> &mut ExportTask {
    step_mut(document)
        .tasks
        .first_mut()
        .expect("fixture should have a task")
}

/// Borrow the first detail of the first task.
fn detail_mut(document: &mut ExportDocument) -> &mut ExportDetail {
    task_mut(document)
        .details
        .first_mut()
        .expect("fixture should have a detail")
}

/// Borrow the first nested detail of the first task detail.
fn nested_detail_mut(document: &mut ExportDocument) -> &mut ExportDetail {
    detail_mut(document)
        .details
        .first_mut()
        .expect("fixture should have a nested detail")
}

#[test]
fn identical_documents_have_no_differences() {
    let committed = document(vec![phase("1")]);
    let fresh = committed.clone();

    assert!(differences(&committed, &fresh).is_empty());
}

#[test]
fn a_format_version_change_is_reported_first() {
    let committed = document(vec![phase("1")]);
    let mut fresh = committed.clone();
    fresh.format_version = 2;

    let reported = differences(&committed, &fresh);

    assert_eq!(reported, vec!["~ format_version: 1 -> 2".to_owned()]);
}

#[test]
fn phase_additions_removals_and_order_changes_are_reported() {
    let committed = document(vec![phase("1"), phase("2")]);
    let mut fresh = document(vec![phase("2"), phase("3")]);
    phase_mut(&mut fresh).title = "Renamed".to_owned();

    let reported = differences(&committed, &fresh);

    assert!(
        reported.contains(&"~ roadmap phases order: [\"1\", \"2\"] -> [\"2\", \"3\"]".to_owned())
    );
    assert!(reported.contains(&"+ phase 3 added".to_owned()));
    assert!(reported.contains(&"- phase 1 removed".to_owned()));
    assert!(reported.contains(&"~ phase 2 title: \"Phase title\" -> \"Renamed\"".to_owned()));
}

#[test]
fn step_additions_removals_and_field_changes_are_reported() {
    let mut committed = document(vec![phase("1")]);
    phase_mut(&mut committed).steps = vec![step("1.1"), step("1.2")];
    let mut fresh = document(vec![phase("1")]);
    phase_mut(&mut fresh).steps = vec![step("1.1"), step("1.3")];
    step_mut(&mut fresh).question = None;
    step_mut(&mut fresh).requires = vec!["1.1.1".to_owned()];

    let reported = differences(&committed, &fresh);

    assert!(
        reported.contains(
            &"~ phase 1 steps order: [\"1.1\", \"1.2\"] -> [\"1.1\", \"1.3\"]".to_owned()
        )
    );
    assert!(reported.contains(&"+ step 1.3 added".to_owned()));
    assert!(reported.contains(&"- step 1.2 removed".to_owned()));
    assert!(reported.contains(&"~ step 1.1 question: Some(\"Question?\") -> None".to_owned()));
    assert!(reported.contains(&"~ step 1.1 requires: [] -> [\"1.1.1\"]".to_owned()));
}

#[test]
fn task_additions_removals_and_field_changes_are_reported() {
    let mut committed = document(vec![phase("1")]);
    step_mut(&mut committed).tasks = vec![task("1.1.1"), task("1.1.2")];
    let mut fresh = document(vec![phase("1")]);
    step_mut(&mut fresh).tasks = vec![task("1.1.1"), task("1.1.3")];
    task_mut(&mut fresh).title = "Changed".to_owned();
    task_mut(&mut fresh).state = ExportState::Done;
    task_mut(&mut fresh).requires = vec!["1.1.2".to_owned()];

    let reported = differences(&committed, &fresh);

    assert!(reported.contains(
        &"~ step 1.1 tasks order: [\"1.1.1\", \"1.1.2\"] -> [\"1.1.1\", \"1.1.3\"]".to_owned()
    ));
    assert!(reported.contains(&"+ task 1.1.3 added".to_owned()));
    assert!(reported.contains(&"- task 1.1.2 removed".to_owned()));
    assert!(reported.contains(&"~ task 1.1.1 title: \"Task title\" -> \"Changed\"".to_owned()));
    assert!(reported.contains(&"~ task 1.1.1 state: \"open\" -> \"done\"".to_owned()));
    assert!(reported.contains(&"~ task 1.1.1 requires: [] -> [\"1.1.2\"]".to_owned()));
}

#[test]
fn entry_changes_additions_and_removals_are_reported_at_every_level() {
    let mut committed = document(vec![phase("1")]);
    task_mut(&mut committed).entries = vec![
        entry(Some("Key"), "value", Vec::new()),
        entry(None, "second", Vec::new()),
    ];
    let mut fresh = document(vec![phase("1")]);
    task_mut(&mut fresh).entries = vec![
        entry(
            Some("Key"),
            "changed",
            vec![entry(None, "nested", Vec::new())],
        ),
        entry(None, "third", Vec::new()),
    ];

    let reported = differences(&committed, &fresh);

    assert!(reported.contains(&"~ task 1.1.1 entry 1 value: \"value\" -> \"changed\"".to_owned()));
    assert!(reported.contains(&"+ task 1.1.1 entry 1 entry 1 added".to_owned()));
    assert!(reported.contains(&"~ task 1.1.1 entry 2 value: \"second\" -> \"third\"".to_owned()));
}

#[test]
fn entry_removals_beyond_the_new_length_are_reported() {
    let mut committed = document(vec![phase("1")]);
    phase_mut(&mut committed).entries = vec![
        entry(None, "first", Vec::new()),
        entry(None, "second", Vec::new()),
    ];
    let mut fresh = document(vec![phase("1")]);
    phase_mut(&mut fresh).entries = vec![entry(None, "first", Vec::new())];

    let reported = differences(&committed, &fresh);

    assert!(reported.contains(&"- phase 1 entry 2 removed".to_owned()));
}

#[test]
fn numbered_detail_changes_additions_and_removals_are_reported() {
    let mut committed = document(vec![phase("1")]);
    task_mut(&mut committed).details = vec![
        detail(Some("1.1.1.1"), "First", Vec::new(), Vec::new()),
        detail(Some("1.1.1.2"), "Second", Vec::new(), Vec::new()),
    ];
    let mut fresh = document(vec![phase("1")]);
    task_mut(&mut fresh).details = vec![
        detail(Some("1.1.1.1"), "Renamed", Vec::new(), Vec::new()),
        detail(Some("1.1.1.3"), "Third", Vec::new(), Vec::new()),
    ];
    detail_mut(&mut fresh).state = ExportState::Done;
    detail_mut(&mut fresh).requires = vec!["1.1.2".to_owned()];

    let reported = differences(&committed, &fresh);

    assert!(
        reported.contains(
            &"~ task 1.1.1 details order: [\"1.1.1.1\", \"1.1.1.2\"] -> [\"1.1.1.1\", \"1.1.1.3\"]"
                .to_owned()
        )
    );
    assert!(reported.contains(&"~ sub-task 1.1.1.1 title: \"First\" -> \"Renamed\"".to_owned()));
    assert!(reported.contains(&"~ sub-task 1.1.1.1 state: \"open\" -> \"done\"".to_owned()));
    assert!(reported.contains(&"~ sub-task 1.1.1.1 requires: [] -> [\"1.1.2\"]".to_owned()));
    assert!(reported.contains(&"+ sub-task 1.1.1.3 added".to_owned()));
    assert!(reported.contains(&"- sub-task 1.1.1.2 removed".to_owned()));
}

#[test]
fn unnumbered_details_match_by_source_ordinal() {
    let mut committed = document(vec![phase("1")]);
    task_mut(&mut committed).details = vec![
        detail(None, "First", Vec::new(), Vec::new()),
        detail(None, "Second", Vec::new(), Vec::new()),
    ];
    let mut fresh = document(vec![phase("1")]);
    task_mut(&mut fresh).details = vec![detail(None, "First", Vec::new(), Vec::new())];

    let reported = differences(&committed, &fresh);

    assert!(reported.contains(&"- detail 2 of task 1.1.1 removed".to_owned()));
}

#[test]
fn added_unnumbered_details_are_reported_by_ordinal() {
    let mut committed = document(vec![phase("1")]);
    task_mut(&mut committed).details = vec![detail(None, "First", Vec::new(), Vec::new())];
    let mut fresh = document(vec![phase("1")]);
    task_mut(&mut fresh).details = vec![
        detail(None, "First", Vec::new(), Vec::new()),
        detail(None, "Second", Vec::new(), Vec::new()),
    ];

    let reported = differences(&committed, &fresh);

    assert!(reported.contains(&"+ detail 2 of task 1.1.1 added".to_owned()));
}

#[test]
fn nested_detail_drift_names_its_preformatted_owner() {
    let mut committed = document(vec![phase("1")]);
    detail_mut(&mut committed).details = vec![detail(None, "Nested", Vec::new(), Vec::new())];
    let mut fresh = committed.clone();
    nested_detail_mut(&mut fresh).title = "Changed".to_owned();

    let reported = differences(&committed, &fresh);

    assert!(
        reported.contains(
            &"~ detail 1 of sub-task 1.1.1.1 title: \"Nested\" -> \"Changed\"".to_owned()
        )
    );
}
