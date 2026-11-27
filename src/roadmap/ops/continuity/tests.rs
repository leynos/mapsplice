//! Unit tests for replacement identity continuity.

use rstest::rstest;

use super::{ItemPosition, RetiredSubtree, pair_descendants};
use crate::{
    error::{MapspliceError, Result},
    roadmap::{
        RoadmapAnchor,
        RoadmapDocument,
        RoadmapOperation,
        apply_command,
        model::{ItemIdentity, SourceId, TaskChild},
        parse_anchor,
        parse_fragment,
        parse_roadmap,
        render_roadmap,
    },
};

const TASK_TARGET: &str = concat!(
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. Consumer. Requires 1.1.2.\n",
    "- [x] 1.1.2. Depended-on task.\n",
);

const CHECKED_OFF_FRAGMENT: &str = "- [x] 1.1.2. Revised depended-on task.\n";
const IDENTICAL_FRAGMENT: &str = "- [x] 1.1.2. Depended-on task.\n";
const CHANGED_NUMBER_FRAGMENT: &str = "- [x] 1.1.5. Revised depended-on task.\n";
const MULTI_ITEM_FRAGMENT: &str = concat!(
    "- [x] 1.1.2. First replacement.\n",
    "- [ ] 1.1.3. Second replacement.\n",
);

const SUB_TASK_TARGET: &str = concat!(
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. Parent task.\n",
    "  - [ ] 1.1.1.1. Consumer sub-task. Requires 1.1.1.2.\n",
    "  - [ ] 1.1.1.2. Depended-on sub-task.\n",
);

const SUB_TASK_FRAGMENT: &str = "  - [x] 1.1.1.2. Revised sub-task.\n";

const STEP_TARGET: &str = concat!(
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. Depended-on task.\n\n",
    "### 1.2. Step two\n\n",
    "- [ ] 1.2.1. Task consumer. Requires 1.1.1.\n",
    "- [ ] 1.2.2. Step consumer. Requires 1.1.\n",
);

const STEP_FRAGMENT: &str = concat!(
    "### 1.1. Replacement step\n\n",
    "- [ ] 1.1.1. Revised depended-on task.\n",
);

const PHASE_TARGET: &str = concat!(
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. Depended-on task.\n\n",
    "## 2. Phase two\n\n",
    "### 2.1. Step one\n\n",
    "- [ ] 2.1.1. Task consumer. Requires 1.1.1.\n",
    "- [ ] 2.1.2. Step consumer. Requires 1.1.\n",
    "- [ ] 2.1.3. Phase consumer. Requires 1.\n",
);

const PHASE_FRAGMENT: &str = concat!(
    "## 1. Replacement phase\n\n",
    "### 1.1. Replacement step\n\n",
    "- [ ] 1.1.1. Revised depended-on task.\n",
);

const NESTED_FRAGMENT_TARGET: &str = concat!(
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. Consumer. Requires 1.1.2.1.\n",
    "- [ ] 1.1.2. Parent task.\n",
    "  - [ ] 1.1.2.1. Depended-on sub-task.\n",
);

const NESTED_FRAGMENT_REPLACEMENT: &str = concat!(
    "- [x] 1.1.2. Revised parent task.\n",
    "  - [ ] 1.1.2.1. Revised sub-task.\n",
);

/// One continuation case and the clauses it must preserve.
#[derive(Clone, Copy, Debug)]
struct ContinuationCase {
    target: &'static str,
    anchor: &'static str,
    fragment: &'static str,
    expected: &'static [&'static str],
}

#[rstest]
#[case::task(ContinuationCase {
    target: TASK_TARGET,
    anchor: "1.1.2",
    fragment: CHECKED_OFF_FRAGMENT,
    expected: &[
        "- [ ] 1.1.1. Consumer. Requires 1.1.2.\n",
        "- [x] 1.1.2. Revised depended-on task.\n",
    ],
})]
#[case::sub_task(ContinuationCase {
    target: SUB_TASK_TARGET,
    anchor: "1.1.1.2",
    fragment: SUB_TASK_FRAGMENT,
    expected: &[
        "  - [ ] 1.1.1.1. Consumer sub-task. Requires 1.1.1.2.\n",
        "  - [x] 1.1.1.2. Revised sub-task.\n",
    ],
})]
#[case::step(ContinuationCase {
    target: STEP_TARGET,
    anchor: "1.1",
    fragment: STEP_FRAGMENT,
    expected: &[
        "- [ ] 1.2.1. Task consumer. Requires 1.1.1.\n",
        "- [ ] 1.2.2. Step consumer. Requires 1.1.\n",
        "- [ ] 1.1.1. Revised depended-on task.\n",
    ],
})]
#[case::phase(ContinuationCase {
    target: PHASE_TARGET,
    anchor: "1",
    fragment: PHASE_FRAGMENT,
    expected: &[
        "- [ ] 2.1.1. Task consumer. Requires 1.1.1.\n",
        "- [ ] 2.1.2. Step consumer. Requires 1.1.\n",
        "- [ ] 2.1.3. Phase consumer. Requires 1.\n",
    ],
})]
fn replace_continues_the_addressed_identity(#[case] case: ContinuationCase) {
    let document = replace(ReplaceCase {
        target: case.target,
        anchor: case.anchor,
        fragment: case.fragment,
    })
    .expect("the in-place replacement should succeed");
    let output = rendered(&document);

    for clause in case.expected {
        assert!(
            output.contains(clause),
            "the reference `{clause}` should stay byte-identical, got:\n{output}"
        );
    }
}

#[test]
fn replace_task_with_identical_content_is_a_no_op() {
    let document = replace(ReplaceCase {
        target: TASK_TARGET,
        anchor: "1.1.2",
        fragment: IDENTICAL_FRAGMENT,
    })
    .expect("the identical replacement should succeed");

    assert_eq!(
        rendered(&document),
        TASK_TARGET,
        "an identical one-item replacement should not change the roadmap"
    );
}

#[rstest]
#[case::changed_number(CHANGED_NUMBER_FRAGMENT)]
#[case::multiple_items(MULTI_ITEM_FRAGMENT)]
fn replace_task_that_retires_the_identity_reports_a_dangling_dependency(#[case] fragment: &str) {
    let outcome = replace(ReplaceCase {
        target: TASK_TARGET,
        anchor: "1.1.2",
        fragment,
    });
    let message = format!("the {fragment:?} replacement retires the identity");
    assert_dangling(&outcome, anchor("1.1.2"), message);
}

#[test]
fn delete_task_grants_no_continuity() {
    let outcome = delete(TargetCase {
        target: TASK_TARGET,
        anchor: "1.1.2",
    });
    assert_dangling(&outcome, anchor("1.1.2"), "deletion retires the identity");
}

/// Assert the outcome is a dangling dependency on `expected`.
fn assert_dangling(
    outcome: &Result<RoadmapDocument>,
    expected: RoadmapAnchor,
    message: impl std::fmt::Display,
) {
    assert!(
        matches!(
            outcome,
            Err(MapspliceError::DanglingDependency { anchor: reported, .. })
                if *reported == expected
        ),
        "{message}, got {outcome:?}"
    );
}

#[test]
fn replace_task_continues_a_depended_on_sub_task() {
    let document = replace(ReplaceCase {
        target: NESTED_FRAGMENT_TARGET,
        anchor: "1.1.2",
        fragment: NESTED_FRAGMENT_REPLACEMENT,
    })
    .expect("the nested in-place replacement should succeed");
    let output = rendered(&document);

    assert!(
        output.contains("- [ ] 1.1.1. Consumer. Requires 1.1.2.1.\n"),
        "the sub-task reference should stay byte-identical, got:\n{output}"
    );

    let replacement_task = document
        .phases
        .first()
        .and_then(|phase| phase.steps.first())
        .and_then(|step| step.tasks().get(1))
        .expect("replacement task should exist");
    let replacement_sub_task = replacement_task
        .sub_tasks()
        .first()
        .expect("replacement sub-task should exist");

    assert_eq!(
        replacement_sub_task.identity,
        ItemIdentity {
            source: SourceId::Target,
            anchor: anchor("1.1.2.1"),
        },
        "committing the operation should re-anchor the replacement sub-task"
    );
    assert!(
        replacement_task.children().iter().any(|child| matches!(
            child,
            TaskChild::SubTask(candidate) if *candidate == replacement_sub_task.identity
        )),
        "the replacement task should list the re-anchored sub-task marker, got: {:?}",
        replacement_task.children()
    );
}

#[test]
fn descendant_pairing_skips_a_duplicated_retired_anchor() {
    let retired = retired_with(&[anchor("1.1.1.1"), anchor("1.1.1.1")]);
    let replacement = replacement_with(&[(anchor("1.1.1.1"), 0)]);

    assert!(
        pair_descendants(&retired, &replacement).is_empty(),
        "a retired anchor held by two items cannot be paired"
    );
}

#[test]
fn descendant_pairing_skips_a_duplicated_replacement_anchor() {
    let retired = retired_with(&[anchor("1.1.1.1")]);
    let replacement = replacement_with(&[(anchor("1.1.1.1"), 0), (anchor("1.1.1.1"), 1)]);

    assert!(
        pair_descendants(&retired, &replacement).is_empty(),
        "a replacement anchor held by two items cannot be paired"
    );
}

#[test]
fn descendant_pairing_keeps_only_unique_matches() {
    let retired = retired_with(&[anchor("1.1.1.1"), anchor("1.1.1.2")]);
    let replacement = replacement_with(&[(anchor("1.1.1.2"), 1), (anchor("1.1.1.3"), 2)]);

    let pairs = pair_descendants(&retired, &replacement);

    assert_eq!(
        pairs.len(),
        1,
        "only the anchor present on both sides should pair, got {pairs:?}"
    );
    let (paired_identity, paired_position) = pairs
        .first()
        .expect("the unique match should produce one pair");
    assert_eq!(
        paired_identity.anchor,
        anchor("1.1.1.2"),
        "the shared anchor should pair the retired identity"
    );
    assert_eq!(
        *paired_position,
        sub_task_position(1),
        "the pair should name the replacement's own position"
    );
}

/// One replacement case: a document, an addressed anchor, and a fragment.
#[derive(Clone, Copy, Debug)]
struct ReplaceCase<'a> {
    target: &'a str,
    anchor: &'a str,
    fragment: &'a str,
}

/// A document and the anchor a mutation addresses.
#[derive(Clone, Copy, Debug)]
struct TargetCase<'a> {
    target: &'a str,
    anchor: &'a str,
}

/// Parse a test anchor, failing the test with its spelling on error.
fn anchor(raw: &str) -> RoadmapAnchor { parse_anchor(raw).expect("test anchor should parse") }

/// Build an identity for one source and anchor.
fn identity(source: SourceId, anchor: RoadmapAnchor) -> ItemIdentity {
    ItemIdentity { source, anchor }
}

/// Build a retired subtree whose addressed item is task `1.1.1`.
fn retired_with(descendants: &[RoadmapAnchor]) -> RetiredSubtree {
    RetiredSubtree {
        addressed: identity(SourceId::Target, anchor("1.1.1")),
        descendants: descendants
            .iter()
            .map(|value| identity(SourceId::Target, *value))
            .collect(),
    }
}

/// Build replacement positions inside the first task's sub-task list.
fn replacement_with(entries: &[(RoadmapAnchor, usize)]) -> Vec<(ItemIdentity, ItemPosition)> {
    entries
        .iter()
        .map(|(value, index)| {
            (
                identity(SourceId::Fragment, *value),
                sub_task_position(*index),
            )
        })
        .collect()
}

/// Build a position inside the first task's sub-task list.
fn sub_task_position(sub_task: usize) -> ItemPosition {
    ItemPosition::SubTask {
        phase: 0,
        step: 0,
        task: 0,
        sub_task,
    }
}

/// Apply one replace operation and return the mutated document.
fn replace(case: ReplaceCase<'_>) -> Result<RoadmapDocument> {
    let mut document = parse_roadmap(case.target).expect("test roadmap should parse");
    let fragment_document = parse_fragment(case.fragment).expect("test fragment should parse");

    apply_command(
        &mut document,
        RoadmapOperation::Replace {
            anchor: anchor(case.anchor),
        },
        Some(fragment_document),
    )?;
    Ok(document)
}

/// Apply one delete operation and return the mutated document.
fn delete(case: TargetCase<'_>) -> Result<RoadmapDocument> {
    let mut document = parse_roadmap(case.target).expect("test roadmap should parse");

    apply_command(
        &mut document,
        RoadmapOperation::Delete {
            anchor: anchor(case.anchor),
        },
        None,
    )?;
    Ok(document)
}

/// Render a document, failing the test on a rendering error.
fn rendered(document: &RoadmapDocument) -> String {
    render_roadmap(document).expect("mutated roadmap should render")
}
