//! Assert one generated dependency-edit case against its oracle outcome.
//!
//! Split from `tests/support/dependency_case.rs`, which builds and runs the
//! case, so that each file stays under the 400-line limit.
//!
//! Two obligations meet here and must not be conflated. A check on what an item
//! *says* normalises `\r\n` to `\n` first, because a target may be generated
//! with either terminator and the renderer may change an item's terminators
//! while leaving its text alone. A check on what the file *holds* may not
//! normalise anything: a rejected edit has to leave the target with the exact
//! bytes it started from, which is why [`assert_target_unchanged`] is the one
//! assertion here that compares raw strings.

use std::collections::BTreeSet;

use mapsplice::MapspliceError;
use proptest::{
    prelude::*,
    test_runner::{TestCaseError, TestCaseResult},
};

use super::{
    dependency_case::{Case, Mode, prepare_workspace, run_case},
    generation::{
        FENCED_EXAMPLE,
        INCIDENTAL_TEXTS,
        fragment_label,
        phase_label,
        step_label,
        sub_task_label,
        task_label,
    },
    oracle::{
        Expectation,
        FRAGMENT_BASE,
        ItemId,
        Level,
        Model,
        SUB_TASKS_PER_TAIL,
        TASKS_PER_STEP,
    },
    rendered_block::{item_block, strip_fenced_regions},
    workspace_support::Workspace,
};

/// Assert that one generated edit matches its independently computed outcome.
pub fn assert_generated_edit(case: &Case, mode: Mode) -> TestCaseResult {
    let workspace = prepare_workspace(case)?;

    match (&case.expectation, run_case(&workspace, case, mode)) {
        (Expectation::Reject { stalled }, Err(MapspliceError::DanglingDependency { anchor })) => {
            prop_assert!(
                stalled.contains(&anchor.to_string()),
                "rejection must name a stranded anchor; named `{anchor}`, stranded {stalled:?}",
            );
            assert_target_unchanged(&workspace, &case.generated.target)?;
        }
        (Expectation::Reject { stalled }, Err(other)) => {
            prop_assert!(
                false,
                "expected a dangling-dependency rejection naming one of {stalled:?}, got `{other}`",
            );
        }
        (Expectation::Reject { stalled }, Ok(_)) => {
            prop_assert!(
                false,
                "edit must be rejected: surviving consumers still require {stalled:?}",
            );
        }
        (Expectation::Accept { .. }, Err(error)) => {
            prop_assert!(false, "edit must succeed, but failed with `{error}`");
        }
        (Expectation::Accept { .. }, Ok(rendered)) => {
            assert_accepted(&rendered, case)?;
        }
    }
    Ok(())
}

/// Assert a successful edit renders exactly the anchors the oracle predicts.
///
/// Every assertion is scoped to the block the owning item renders, not to the
/// whole document. A document-wide `contains` would be satisfied by another
/// item's text, and in particular by the fenced code example the generator
/// emits into every task body spelling `Requires 1.1.1.`, which would make a
/// clause assertion for that anchor vacuous.
fn assert_accepted(rendered: &str, case: &Case) -> TestCaseResult {
    let Some(anchors) = case.expectation.anchors() else {
        prop_assert!(false, "accepted case must carry expected anchors");
        return Ok(());
    };
    let retired = case.expectation.retired().cloned().unwrap_or_default();

    for (identity, anchor) in anchors {
        if identity.is_fragment() {
            continue;
        }
        let stem = item_stem(*identity);
        let Some(block) = item_block(rendered, &stem) else {
            prop_assert!(
                false,
                "surviving identity {identity:?} must render a summary containing \
                 `{stem}`\nrendered:\n{rendered}",
            );
            return Ok(());
        };
        let expected = format!("{anchor}. {stem}");
        prop_assert!(
            content_view(block).contains(&expected),
            "surviving identity {identity:?} must render as `{expected}`\nblock:\n{block}",
        );
    }

    for (consumer, prerequisite, expected_anchor) in case.expectation.rewritten() {
        let Some(block) = item_block(rendered, &item_stem(*consumer)) else {
            prop_assert!(
                false,
                "consumer {consumer:?} must render a summary to carry a clause",
            );
            return Ok(());
        };
        let clause = format!("Requires {expected_anchor}.");
        let block = content_view(block);
        prop_assert!(
            block.contains(&clause),
            "consumer {consumer:?} must carry a clause `{clause}` for \
             {prerequisite:?}\nblock:\n{block}",
        );
        assert_clause_is_not_incidental(&block, &clause, *consumer)?;
    }

    assert_no_retired_clause(rendered, &retired, case)?;
    Ok(())
}

/// Assert the clause appears outside the item's fenced code examples.
///
/// The generator writes a fenced block whose content is itself clause-shaped,
/// so `block.contains(clause)` alone cannot distinguish a real clause from that
/// example. Removing the fenced regions first is what makes the assertion bite.
fn assert_clause_is_not_incidental(block: &str, clause: &str, consumer: ItemId) -> TestCaseResult {
    let outside_fences = strip_fenced_regions(block);
    prop_assert!(
        outside_fences.contains(clause),
        "consumer {consumer:?} must carry `{clause}` outside its fenced code examples; the only \
         occurrence was inside one\nblock:\n{block}",
    );
    Ok(())
}

/// Assert no rewritten clause points at an anchor a removed identity vacated.
///
/// This is the identity-preservation obligation in observable form: a clause
/// reading a retired anchor would mean a consumer had silently resolved to a
/// different item that inherited the old number.
fn assert_no_retired_clause(
    rendered: &str,
    retired: &BTreeSet<String>,
    case: &Case,
) -> TestCaseResult {
    let live_anchors = case
        .expectation
        .anchors()
        .map(|anchors| anchors.values().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default();

    for anchor in retired {
        if live_anchors.contains(anchor) {
            continue;
        }
        for spelling in clause_spellings(rendered, anchor) {
            prop_assert!(
                !content_view(rendered).contains(&spelling),
                "retired anchor `{anchor}` must not survive in `{spelling}`\nrendered:\n{rendered}",
            );
        }
    }
    Ok(())
}

/// Return every clause spelling that names `anchor`, in a blind scan.
///
/// The scan is deliberately naive and does not reuse the production clause
/// recognizer: it splits on whitespace and reports tokens whose punctuation
/// stripped equals the anchor. A false positive is resolved by the caller
/// checking whether the anchor is live, so the scan can afford to be coarse.
/// `str::split_whitespace` treats `\r` as a separator, so the scan needs no
/// line-ending adaptation of its own.
fn clause_spellings(rendered: &str, anchor: &str) -> Vec<String> {
    let with_dot = format!("{anchor}.");
    rendered
        .split_whitespace()
        .filter(|token| token.trim_matches(['-', '*', '`']) == with_dot)
        .map(|_| format!("Requires {anchor}."))
        .collect()
}

/// Assert unrelated numeric text is preserved verbatim, whatever the outcome.
///
/// Each task's incidental prose and fenced example are checked inside that
/// task's own block, so a surviving stray copy elsewhere in the document
/// cannot satisfy the assertion. A delete whose anchor precedes a task
/// legitimately removes that task's block, so only blocks still present are
/// inspected. The comparison normalizes terminators because this obligation is
/// about the *text* surviving a rewrite, not about which terminator the
/// renderer chose for the item it sits in; the byte-level claim is made
/// separately, by [`assert_target_unchanged`] on the rejection path.
pub fn assert_incidental_text_survives(case: &Case, mode: Mode) -> TestCaseResult {
    let workspace = prepare_workspace(case)?;

    let Ok(rendered) = run_case(&workspace, case, mode) else {
        assert_target_unchanged(&workspace, &case.generated.target)?;
        return Ok(());
    };
    let rendered = content_view(&rendered);

    let mut inspected = 0_usize;
    for identity in Model::per_task_identities() {
        if identity.is_fragment() {
            continue;
        }
        let Some(block) = item_block(&rendered, &item_stem(identity)) else {
            continue;
        };
        inspected += 1;
        for fragment in &INCIDENTAL_TEXTS[..3] {
            prop_assert!(
                block.contains(fragment),
                "incidental text `{fragment}` must survive inside {identity:?}'s \
                 block\nblock:\n{block}",
            );
        }
        prop_assert!(
            block.contains(FENCED_EXAMPLE),
            "task {identity:?} must keep its fenced example untouched\nblock:\n{block}",
        );
    }
    prop_assert!(
        inspected > 0,
        "at least one task block must survive to inspect\nrendered:\n{rendered}",
    );
    Ok(())
}

/// Return the summary stem the generator renders for one identity.
fn item_stem(identity: ItemId) -> String {
    match identity.level {
        Level::Phase => phase_label(identity.index),
        Level::Step => step_label(identity.index),
        Level::Task if identity.is_fragment() => fragment_label(identity.index - FRAGMENT_BASE),
        Level::Task => task_label(identity.index),
        Level::SubTask => sub_task_label(
            identity.index.div_euclid(SUB_TASKS_PER_TAIL) * TASKS_PER_STEP + TASKS_PER_STEP - 1,
            identity.index.rem_euclid(SUB_TASKS_PER_TAIL),
        ),
    }
}

/// Assert the target file still holds exactly the generated bytes.
fn assert_target_unchanged(workspace: &Workspace, expected: &str) -> TestCaseResult {
    let after = workspace
        .dir
        .read_to_string("target.md")
        .map_err(|error| TestCaseError::fail(error.to_string()))?;
    prop_assert!(
        after == expected,
        "target must be left byte-identical\nexpected:\n{expected}\ngot:\n{after}",
    );
    Ok(())
}

/// Return `text` with CRLF terminators reduced to LF for a content comparison.
///
/// Only the checks that ask *what an item says* use this. Nothing that asks
/// *what bytes the file holds* may, which is why the preservation assertion
/// compares raw strings.
fn content_view(text: &str) -> String { text.replace("\r\n", "\n") }
