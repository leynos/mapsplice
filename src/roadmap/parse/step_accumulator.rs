//! Shared active-step accumulation for document and fragment parsing.

use markdown::mdast::{List, Node};

use super::{parse_task_list, validate_tasks_belong_to_step};
use crate::{
    error::{MapspliceError, Result},
    roadmap::{
        StepNumber,
        model::{ItemIdentity, MarkdownNodes, SourceId, StepSection},
        source_preservation::original_position_source,
    },
};

pub(super) struct StepAccumulator<'source> {
    current: Option<StepSection>,
    source: SourceId,
    source_text: &'source str,
}

impl<'source> StepAccumulator<'source> {
    pub(super) const fn new(source: SourceId, source_text: &'source str) -> Self {
        Self {
            current: None,
            source,
            source_text,
        }
    }

    pub(super) const fn has_active_step(&self) -> bool { self.current.is_some() }

    pub(super) fn begin_step(
        &mut self,
        number: StepNumber,
        title: Vec<Node>,
        completed: &mut Vec<StepSection>,
    ) {
        self.flush_into(completed);
        self.current = Some(StepSection {
            identity: ItemIdentity {
                source: self.source,
                anchor: number.into(),
            },
            number,
            title: MarkdownNodes::from_nodes(title),
            body: MarkdownNodes::new(),
            tasks: Vec::new(),
            task_list_source: None,
            trailing: MarkdownNodes::new(),
        });
    }

    pub(super) fn append_task_list(&mut self, list: &List) -> Result<()> {
        let current = self
            .current
            .as_mut()
            .ok_or_else(|| MapspliceError::InvalidRoadmap {
                message: "task list appeared without a current step".to_owned(),
            })?;
        if !current.trailing.is_empty() {
            return Err(MapspliceError::InvalidRoadmap {
                message: format!(
                    "task list for step `{}` cannot appear after trailing step content",
                    current.number
                ),
            });
        }

        let mut tasks = parse_task_list(list, self.source, self.source_text)?;
        validate_tasks_belong_to_step(current.number, &tasks)?;
        if current.tasks.is_empty() {
            current.set_task_list_source(
                list.position
                    .as_ref()
                    .and_then(|position| original_position_source(position, self.source_text)),
            );
        } else {
            current.clear_task_list_source();
        }
        current.tasks.append(&mut tasks);
        Ok(())
    }

    pub(super) fn push_non_structural_node(&mut self, node: Node) -> Result<()> {
        let current = self
            .current
            .as_mut()
            .ok_or_else(|| MapspliceError::InvalidRoadmap {
                message: "step fragments must contain only step sections".to_owned(),
            })?;
        if current.tasks.is_empty() {
            current.body.push_preserved(node, self.source_text);
        } else {
            current.trailing.push_preserved(node, self.source_text);
        }
        Ok(())
    }

    pub(super) fn flush_into(&mut self, completed: &mut Vec<StepSection>) {
        if let Some(step) = self.current.take() {
            completed.push(step);
        }
    }
}
