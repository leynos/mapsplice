//! Ordered task-child accumulator for roadmap parsing.

use crate::roadmap::model::{MarkdownNodes, SubTaskEntry, TaskChild};

pub(super) struct TaskChildren {
    pub(super) body: MarkdownNodes,
    pub(super) sub_tasks: Vec<SubTaskEntry>,
    pub(super) ordered: Vec<TaskChild>,
}

impl TaskChildren {
    pub(super) const fn new() -> Self {
        Self {
            body: MarkdownNodes::new(),
            sub_tasks: Vec::new(),
            ordered: Vec::new(),
        }
    }

    pub(super) fn flush_body(&mut self) {
        if !self.body.is_empty() {
            self.ordered.push(TaskChild::Body(self.body.clone()));
            self.body = MarkdownNodes::new();
        }
    }
}
