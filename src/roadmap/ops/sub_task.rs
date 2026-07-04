//! Addendum sub-task splice helpers.

use super::find_task_parent_mut;
use crate::{
    error::{MapspliceError, Result},
    roadmap::{
        RoadmapDocument,
        SubTaskNumber,
        model::{ItemIdentity, StepSection, SubTaskEntry, SubTaskSplice, TaskChild, TaskEntry},
    },
};

pub(super) fn insert_sub_tasks(
    roadmap: &mut RoadmapDocument,
    target: SubTaskNumber,
    after: bool,
    sub_tasks: Vec<SubTaskEntry>,
) -> Result<()> {
    let (step, task_index, sub_task_index) = find_sub_task_parent_mut(roadmap, target)?;
    {
        let task = step
            .tasks
            .get_mut(task_index)
            .ok_or(MapspliceError::AnchorNotFound {
                anchor: target.task_number().into(),
            })?;
        let target_identity = sub_task_identity(task, sub_task_index)?;
        let splice = find_sub_task_splice(task, sub_task_index, target_identity)?;
        task.insert_sub_tasks(splice, after, sub_tasks);
    }
    step.clear_task_list_source();
    Ok(())
}

pub(super) fn delete_sub_task(roadmap: &mut RoadmapDocument, target: SubTaskNumber) -> Result<()> {
    let (step, task_index, sub_task_index) = find_sub_task_parent_mut(roadmap, target)?;
    {
        let task = step
            .tasks
            .get_mut(task_index)
            .ok_or(MapspliceError::AnchorNotFound {
                anchor: target.task_number().into(),
            })?;
        let target_identity = sub_task_identity(task, sub_task_index)?;
        let splice = find_sub_task_splice(task, sub_task_index, target_identity)?;
        task.delete_sub_task(splice);
    }
    step.clear_task_list_source();
    Ok(())
}

pub(super) fn replace_sub_task(
    roadmap: &mut RoadmapDocument,
    target: SubTaskNumber,
    sub_tasks: Vec<SubTaskEntry>,
) -> Result<()> {
    let (step, task_index, sub_task_index) = find_sub_task_parent_mut(roadmap, target)?;
    {
        let task = step
            .tasks
            .get_mut(task_index)
            .ok_or(MapspliceError::AnchorNotFound {
                anchor: target.task_number().into(),
            })?;
        let target_identity = sub_task_identity(task, sub_task_index)?;
        let splice = find_sub_task_splice(task, sub_task_index, target_identity)?;
        task.replace_sub_task(splice, sub_tasks);
    }
    step.clear_task_list_source();
    Ok(())
}

fn find_sub_task_parent_mut(
    roadmap: &mut RoadmapDocument,
    target: SubTaskNumber,
) -> Result<(&mut StepSection, usize, usize)> {
    let (step, task_index) = find_task_parent_mut(roadmap, target.task_number())?;
    let task = step
        .tasks
        .get(task_index)
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.task_number().into(),
        })?;
    let sub_task_index =
        task.find_sub_task_index(target)
            .ok_or(MapspliceError::AnchorNotFound {
                anchor: target.into(),
            })?;
    Ok((step, task_index, sub_task_index))
}

fn sub_task_identity(task: &TaskEntry, sub_task_index: usize) -> Result<ItemIdentity> {
    task.sub_tasks()
        .get(sub_task_index)
        .map(|sub_task| sub_task.identity)
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: format!("sub-task `{}` is missing from parent task", task.number),
        })
}

fn find_sub_task_splice(
    task: &TaskEntry,
    sub_task_index: usize,
    identity: ItemIdentity,
) -> Result<SubTaskSplice> {
    let child_index = task
        .children()
        .iter()
        .position(|child| matches!(child, TaskChild::SubTask(candidate) if *candidate == identity))
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: format!(
                "sub-task `{}` is missing from task child order",
                task.number
            ),
        })?;
    Ok(SubTaskSplice {
        sub_task_index,
        child_index,
    })
}

#[cfg(test)]
mod tests {
    //! Unit tests for sub-task splice model invariants.

    use rstest::rstest;

    use crate::roadmap::{
        RoadmapOperation,
        apply_command,
        model::{ItemIdentity, TaskChild, TaskEntry},
        parse_anchor,
        parse_fragment,
        parse_roadmap,
    };

    const ROADMAP_WITH_TWO_SUB_TASKS: &str = concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  - [ ] 1.1.1.1. Removed sub-task.\n",
        "  - [x] 1.1.1.2. Remaining sub-task.\n",
    );

    const ROADMAP_WITH_THREE_SUB_TASKS: &str = concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  Parent body before sub-tasks.\n",
        "  - [ ] 1.1.1.1. First sub-task.\n",
        "  - [x] 1.1.1.2. Second sub-task.\n",
        "  - [ ] 1.1.1.3. Third sub-task.\n",
    );

    const ONE_SUB_TASK_FRAGMENT: &str = "  - [ ] 1.1.1.1. Inserted sub-task.\n";
    const ONE_REPLACEMENT_SUB_TASK_FRAGMENT: &str = "  - [x] 1.1.1.1. Replacement sub-task.\n";
    const TWO_REPLACEMENT_SUB_TASK_FRAGMENT: &str = concat!(
        "  - [x] 1.1.1.1. Replacement sub-task A.\n",
        "  - [ ] 1.1.1.2. Replacement sub-task B.\n",
    );

    #[rstest]
    #[case::before_first(
        "1.1.1.1",
        false,
        &["1.1.1.1", "1.1.1.2", "1.1.1.3", "1.1.1.4"]
    )]
    #[case::after_last(
        "1.1.1.3",
        true,
        &["1.1.1.1", "1.1.1.2", "1.1.1.3", "1.1.1.4"]
    )]
    fn sub_task_insert_keeps_structural_and_child_vectors_aligned(
        #[case] target: &str,
        #[case] after: bool,
        #[case] expected_numbers: &[&str],
    ) {
        let mut roadmap =
            parse_roadmap(ROADMAP_WITH_THREE_SUB_TASKS).expect("sub-task roadmap should parse");
        let anchor = parse_anchor(target).expect("sub-task anchor should parse");
        let fragment =
            parse_fragment(ONE_SUB_TASK_FRAGMENT).expect("sub-task fragment should parse");

        apply_command(
            &mut roadmap,
            RoadmapOperation::Insert { anchor, after },
            Some(fragment),
        )
        .expect("sub-task insert should succeed");

        let task = parent_task(&roadmap).expect("roadmap should keep the parent task");
        assert_sub_task_numbers(task, expected_numbers);
        assert_sub_task_child_identity_alignment(task);
    }

    #[rstest]
    #[case::replace_first(
        "1.1.1.1",
        ONE_REPLACEMENT_SUB_TASK_FRAGMENT,
        &["1.1.1.1", "1.1.1.2", "1.1.1.3"],
        &[true, true, false]
    )]
    #[case::replace_last_with_multiple(
        "1.1.1.3",
        TWO_REPLACEMENT_SUB_TASK_FRAGMENT,
        &["1.1.1.1", "1.1.1.2", "1.1.1.3", "1.1.1.4"],
        &[false, true, true, false]
    )]
    fn sub_task_replace_keeps_structural_and_child_vectors_aligned(
        #[case] target: &str,
        #[case] fragment_source: &str,
        #[case] expected_numbers: &[&str],
        #[case] expected_checked: &[bool],
    ) {
        let mut roadmap =
            parse_roadmap(ROADMAP_WITH_THREE_SUB_TASKS).expect("sub-task roadmap should parse");
        let anchor = parse_anchor(target).expect("sub-task anchor should parse");
        let fragment = parse_fragment(fragment_source).expect("sub-task fragment should parse");

        apply_command(
            &mut roadmap,
            RoadmapOperation::Replace { anchor },
            Some(fragment),
        )
        .expect("sub-task replace should succeed");

        let task = parent_task(&roadmap).expect("roadmap should keep the parent task");
        assert_sub_task_numbers(task, expected_numbers);
        assert_sub_task_checked_states(task, expected_checked);
        assert_sub_task_child_identity_alignment(task);
    }

    #[test]
    fn delete_sub_task_removes_matching_child_reference() {
        let mut roadmap =
            parse_roadmap(ROADMAP_WITH_TWO_SUB_TASKS).expect("sub-task roadmap should parse");
        let anchor = parse_anchor("1.1.1.1").expect("sub-task anchor should parse");

        apply_command(&mut roadmap, RoadmapOperation::Delete { anchor }, None)
            .expect("sub-task delete should succeed");

        let task = parent_task(&roadmap).expect("roadmap should keep the parent task");

        assert_eq!(task.sub_tasks().len(), 1);
        let remaining_sub_task = task
            .sub_tasks()
            .first()
            .expect("task should keep the remaining sub-task");
        assert_eq!(
            sub_task_child_identities(task),
            vec![remaining_sub_task.identity]
        );
    }

    fn parent_task(roadmap: &crate::roadmap::RoadmapDocument) -> Option<&TaskEntry> {
        roadmap
            .phases
            .first()
            .and_then(|phase| phase.steps.first())
            .and_then(|step| step.tasks.first())
    }

    fn assert_sub_task_numbers(task: &TaskEntry, expected: &[&str]) {
        let actual = task
            .sub_tasks()
            .iter()
            .map(|sub_task| sub_task.number.to_string())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    fn assert_sub_task_checked_states(task: &TaskEntry, expected: &[bool]) {
        let actual = task
            .sub_tasks()
            .iter()
            .map(|sub_task| sub_task.checked)
            .collect::<Vec<_>>();
        let expected_states = expected.iter().copied().map(Some).collect::<Vec<_>>();
        assert_eq!(actual, expected_states);
    }

    fn assert_sub_task_child_identity_alignment(task: &TaskEntry) {
        let sub_task_identities = task
            .sub_tasks()
            .iter()
            .map(|sub_task| sub_task.identity)
            .collect::<Vec<_>>();
        assert_eq!(sub_task_child_identities(task), sub_task_identities);
    }

    fn sub_task_child_identities(task: &TaskEntry) -> Vec<ItemIdentity> {
        task.children()
            .iter()
            .filter_map(|child| match child {
                TaskChild::SubTask(identity) => Some(*identity),
                TaskChild::Body(_) => None,
            })
            .collect()
    }
}
