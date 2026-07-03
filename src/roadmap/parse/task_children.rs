//! Ordered task-child accumulator for roadmap parsing.

use markdown::mdast::Node;

use crate::{
    error::{MapspliceError, Result},
    roadmap::model::{MarkdownNodes, SubTaskEntry, TaskChild},
};

pub(super) struct TaskChildren {
    body: MarkdownNodes,
    sub_tasks: Vec<SubTaskEntry>,
    ordered: Vec<TaskChild>,
}

impl TaskChildren {
    pub(super) const fn new() -> Self {
        Self {
            body: MarkdownNodes::new(),
            sub_tasks: Vec::new(),
            ordered: Vec::new(),
        }
    }

    pub(super) fn push_body_node(&mut self, node: Node, source_text: &str) {
        self.body.push_preserved(node, source_text);
    }

    pub(super) fn push_sub_task(&mut self, sub_task: SubTaskEntry) {
        self.flush_body();
        self.ordered.push(TaskChild::SubTask(sub_task.identity));
        self.sub_tasks.push(sub_task);
    }

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

    pub(super) fn finish(mut self) -> (MarkdownNodes, Vec<SubTaskEntry>, Vec<TaskChild>) {
        self.flush_body();
        (self.body, self.sub_tasks, self.ordered)
    }

    fn flush_body(&mut self) {
        if !self.body.is_empty() {
            self.ordered.push(TaskChild::Body(self.body.clone()));
            self.body = MarkdownNodes::new();
        }
    }
}
