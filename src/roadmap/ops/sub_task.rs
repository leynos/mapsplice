//! Addendum sub-task splice helpers.

use super::find_task_parent_mut;
use crate::{
    error::{MapspliceError, Result},
    roadmap::{
        RoadmapDocument,
        SubTaskNumber,
        model::{ItemIdentity, SubTaskEntry, TaskChild, TaskEntry},
    },
};

pub(super) fn insert_sub_tasks(
    roadmap: &mut RoadmapDocument,
    target: SubTaskNumber,
    after: bool,
    sub_tasks: Vec<SubTaskEntry>,
) -> Result<()> {
    let (task, sub_task_index) = find_sub_task_parent_mut(roadmap, target)?;
    let target_identity = sub_task_identity(task, sub_task_index)?;
    let child_index = find_sub_task_child_index(task, target_identity)?;
    let new_children = sub_task_children(&sub_tasks);
    let insert_at = sub_task_index + usize::from(after);
    let child_insert_at = child_index + usize::from(after);
    task.sub_tasks.splice(insert_at..insert_at, sub_tasks);
    task.children
        .splice(child_insert_at..child_insert_at, new_children);
    Ok(())
}

pub(super) fn delete_sub_task(roadmap: &mut RoadmapDocument, target: SubTaskNumber) -> Result<()> {
    let (task, sub_task_index) = find_sub_task_parent_mut(roadmap, target)?;
    let target_identity = sub_task_identity(task, sub_task_index)?;
    let child_index = find_sub_task_child_index(task, target_identity)?;
    task.sub_tasks.remove(sub_task_index);
    task.children.remove(child_index);
    Ok(())
}

pub(super) fn replace_sub_task(
    roadmap: &mut RoadmapDocument,
    target: SubTaskNumber,
    sub_tasks: Vec<SubTaskEntry>,
) -> Result<()> {
    let (task, sub_task_index) = find_sub_task_parent_mut(roadmap, target)?;
    let target_identity = sub_task_identity(task, sub_task_index)?;
    let child_index = find_sub_task_child_index(task, target_identity)?;
    let new_children = sub_task_children(&sub_tasks);
    task.sub_tasks
        .splice(sub_task_index..=sub_task_index, sub_tasks);
    task.children
        .splice(child_index..=child_index, new_children);
    Ok(())
}

fn find_sub_task_parent_mut(
    roadmap: &mut RoadmapDocument,
    target: SubTaskNumber,
) -> Result<(&mut TaskEntry, usize)> {
    let (step, task_index) = find_task_parent_mut(roadmap, target.task_number())?;
    let task = step
        .tasks
        .get_mut(task_index)
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.task_number().into(),
        })?;
    let sub_task_index = task
        .sub_tasks
        .iter()
        .position(|sub_task| sub_task.number == target)
        .ok_or(MapspliceError::AnchorNotFound {
            anchor: target.into(),
        })?;
    Ok((task, sub_task_index))
}

fn sub_task_identity(task: &TaskEntry, sub_task_index: usize) -> Result<ItemIdentity> {
    task.sub_tasks
        .get(sub_task_index)
        .map(|sub_task| sub_task.identity)
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: format!("sub-task `{}` is missing from parent task", task.number),
        })
}

fn find_sub_task_child_index(task: &TaskEntry, identity: ItemIdentity) -> Result<usize> {
    task.children
        .iter()
        .position(|child| matches!(child, TaskChild::SubTask(candidate) if *candidate == identity))
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: format!(
                "sub-task `{}` is missing from task child order",
                task.number
            ),
        })
}

fn sub_task_children(sub_tasks: &[SubTaskEntry]) -> Vec<TaskChild> {
    sub_tasks
        .iter()
        .map(|sub_task| TaskChild::SubTask(sub_task.identity))
        .collect()
}

#[cfg(test)]
mod tests {
    //! Unit tests for sub-task splice model invariants.

    use crate::roadmap::{
        RoadmapOperation,
        apply_command,
        model::{TaskChild, TaskEntry},
        parse_anchor,
        parse_roadmap,
    };

    const ROADMAP_WITH_TWO_SUB_TASKS: &str = concat!(
        "## 1. Phase one\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Parent task.\n",
        "  - [ ] 1.1.1.1. Removed sub-task.\n",
        "  - [x] 1.1.1.2. Remaining sub-task.\n",
    );

    #[test]
    fn delete_sub_task_removes_matching_child_reference() {
        let mut roadmap =
            parse_roadmap(ROADMAP_WITH_TWO_SUB_TASKS).expect("sub-task roadmap should parse");
        let anchor = parse_anchor("1.1.1.1").expect("sub-task anchor should parse");

        apply_command(&mut roadmap, RoadmapOperation::Delete { anchor }, None)
            .expect("sub-task delete should succeed");

        let task = parent_task(&roadmap).expect("roadmap should keep the parent task");
        let child_identities: Vec<_> = task
            .children
            .iter()
            .filter_map(|child| match child {
                TaskChild::SubTask(identity) => Some(*identity),
                TaskChild::Body(_) => None,
            })
            .collect();

        assert_eq!(task.sub_tasks.len(), 1);
        let remaining_sub_task = task
            .sub_tasks
            .first()
            .expect("task should keep the remaining sub-task");
        assert_eq!(child_identities, vec![remaining_sub_task.identity]);
    }

    fn parent_task(roadmap: &crate::roadmap::RoadmapDocument) -> Option<&TaskEntry> {
        roadmap
            .phases
            .first()
            .and_then(|phase| phase.steps.first())
            .and_then(|step| step.tasks.first())
    }
}
