//! Ordered task-child accumulator for roadmap parsing.

use markdown::mdast::Node;

use crate::{
    error::{MapspliceError, Result},
    roadmap::model::{MarkdownNodes, SubTaskEntry, TaskChild},
};

/// Accumulate task body blocks and sub-tasks in their original order.
pub(super) struct TaskChildren {
    /// Body nodes accumulated since the previous sub-task.
    body: MarkdownNodes,
    /// Parsed sub-tasks retained for the task model.
    sub_tasks: Vec<SubTaskEntry>,
    /// Body spans and sub-task identities in source order.
    ordered: Vec<TaskChild>,
}

impl TaskChildren {
    /// Create an empty task-child accumulator.
    pub(super) const fn new() -> Self {
        Self {
            body: MarkdownNodes::new(),
            sub_tasks: Vec::new(),
            ordered: Vec::new(),
        }
    }

    /// Preserve a non-structural node in the pending body span.
    pub(super) fn push_body_node(&mut self, node: Node, source_text: &str) {
        self.body.push_preserved(node, source_text);
    }

    /// Finish the pending body span before recording a sub-task.
    pub(super) fn push_sub_task(&mut self, sub_task: SubTaskEntry) {
        self.flush_body();
        self.ordered.push(TaskChild::SubTask(sub_task.identity));
        self.sub_tasks.push(sub_task);
    }

    /// Calculate the next one-based sub-task ordinal with overflow checks.
    pub(super) fn next_sub_task_ordinal(&self) -> Result<u32> {
        let expected =
            self.sub_tasks
                .len()
                .checked_add(1)
                .ok_or_else(|| MapspliceError::InvalidRoadmap {
                    message: "sub-task count exceeds supported numbering range".to_owned(),
                })?;
        u32::try_from(expected).map_err(|_| MapspliceError::InvalidRoadmap {
            message: "sub-task count exceeds supported numbering range".to_owned(),
        })
    }

    /// Flush pending body nodes and return the task's child collections.
    pub(super) fn finish(mut self) -> (MarkdownNodes, Vec<SubTaskEntry>, Vec<TaskChild>) {
        self.flush_body();
        (self.body, self.sub_tasks, self.ordered)
    }

    /// Add a pending body span to the ordered sequence when non-empty.
    fn flush_body(&mut self) {
        if !self.body.is_empty() {
            self.ordered.push(TaskChild::Body(self.body.clone()));
            self.body = MarkdownNodes::new();
        }
    }
}
