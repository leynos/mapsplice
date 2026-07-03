//! Document-level roadmap parsing.

use markdown::mdast::{Heading, List, Node, Root};

use super::{
    is_phase_heading,
    is_step_heading,
    looks_like_sub_task_list,
    looks_like_task_list,
    parse_phase_heading,
    parse_step_heading,
    step_accumulator::StepAccumulator,
};
use crate::{
    error::{MapspliceError, Result},
    roadmap::{
        RoadmapAnchor,
        RoadmapDocument,
        model::{ItemIdentity, MarkdownNodes, PhaseSection, SourceId},
    },
};

/// Parse an mdast root into a roadmap document for the given source.
pub(crate) fn parse_document_root(
    root: Root,
    source: SourceId,
    source_text: &str,
) -> Result<RoadmapDocument> {
    let mut parser = DocumentParser::new(source, source_text);
    for node in root.children {
        parser.parse_node(node)?;
    }
    parser.finish()
}

struct DocumentParser<'source> {
    document: RoadmapDocument,
    current_phase: Option<PhaseSection>,
    steps: StepAccumulator<'source>,
    source: SourceId,
    source_text: &'source str,
}

impl<'source> DocumentParser<'source> {
    /// Create a parser for one source document.
    const fn new(source: SourceId, source_text: &'source str) -> Self {
        Self {
            document: RoadmapDocument::new(),
            current_phase: None,
            steps: StepAccumulator::new(source, source_text),
            source,
            source_text,
        }
    }

    /// Route one top-level Markdown node into roadmap structure or body text.
    fn parse_node(&mut self, node: Node) -> Result<()> {
        match node {
            Node::Heading(heading) if is_phase_heading(&heading) => self.begin_phase(&heading),
            Node::Heading(heading) if is_step_heading(&heading) => self.begin_step(&heading),
            Node::Heading(heading) => self.handle_non_roadmap_heading(heading),
            Node::List(list) if looks_like_task_list(&list) => self.append_task_list(&list),
            Node::List(list) if looks_like_sub_task_list(&list) => {
                Err(MapspliceError::InvalidRoadmap {
                    message: "sub-task list appeared without a parent task".to_owned(),
                })
            }
            other => self.push_non_structural_node(other),
        }
    }

    /// Start a new phase, flushing any previous phase state.
    fn begin_phase(&mut self, heading: &Heading) -> Result<()> {
        self.flush_step()?;
        if let Some(phase) = self.current_phase.take() {
            self.document.phases.push(phase);
        }
        let (number, title) = parse_phase_heading(heading)?;
        self.current_phase = Some(PhaseSection {
            identity: ItemIdentity {
                source: self.source,
                anchor: RoadmapAnchor::Phase(number),
            },
            number,
            title: MarkdownNodes::from_nodes(title),
            body: MarkdownNodes::new(),
            steps: Vec::new(),
            trailing: MarkdownNodes::new(),
        });
        Ok(())
    }

    /// Start a new step inside the current phase.
    fn begin_step(&mut self, heading: &Heading) -> Result<()> {
        let phase = self
            .current_phase
            .as_ref()
            .ok_or_else(|| MapspliceError::InvalidRoadmap {
                message: "step heading appeared before the first phase heading".to_owned(),
            })?;
        let (number, title) = parse_step_heading(heading)?;
        if number.phase() != phase.number {
            return Err(MapspliceError::InvalidRoadmap {
                message: format!(
                    "step heading `{number}` does not belong to phase `{}`",
                    phase.number
                ),
            });
        }
        let current_phase =
            self.current_phase
                .as_mut()
                .ok_or_else(|| MapspliceError::InvalidRoadmap {
                    message: "step heading appeared before the first phase heading".to_owned(),
                })?;
        self.steps
            .begin_step(number, title, &mut current_phase.steps);
        Ok(())
    }

    /// Reject unsupported headings once roadmap parsing has started.
    fn handle_non_roadmap_heading(&mut self, heading: Heading) -> Result<()> {
        if self.current_phase.is_none() {
            self.document
                .preamble
                .push_preserved(Node::Heading(heading), self.source_text);
            return Ok(());
        }

        Err(MapspliceError::InvalidRoadmap {
            message: format!(
                "unsupported non-roadmap heading `{}` inside the roadmap body",
                Node::Heading(heading).to_string()
            ),
        })
    }

    /// Append a validated checklist task list to the current step.
    fn append_task_list(&mut self, list: &List) -> Result<()> { self.steps.append_task_list(list) }

    /// Move the current step into its parent phase.
    fn flush_step(&mut self) -> Result<()> {
        if self.steps.has_active_step() {
            let phase =
                self.current_phase
                    .as_mut()
                    .ok_or_else(|| MapspliceError::InvalidRoadmap {
                        message: "step flush attempted without a phase".to_owned(),
                    })?;
            self.steps.flush_into(&mut phase.steps);
        }
        Ok(())
    }

    /// Preserve non-structural Markdown in the nearest roadmap body.
    fn push_non_structural_node(&mut self, node: Node) -> Result<()> {
        if self.steps.has_active_step() {
            self.steps.push_non_structural_node(node)?;
        } else if let Some(phase) = self.current_phase.as_mut() {
            if phase.steps.is_empty() {
                phase.body.push_preserved(node, self.source_text);
            } else {
                phase.trailing.push_preserved(node, self.source_text);
            }
        } else {
            self.document
                .preamble
                .push_preserved(node, self.source_text);
        }
        Ok(())
    }

    /// Finish parsing and return a validated roadmap document.
    fn finish(mut self) -> Result<RoadmapDocument> {
        self.flush_step()?;
        if let Some(phase) = self.current_phase.take() {
            self.document.phases.push(phase);
        }

        if self.document.phases.is_empty() {
            return Err(MapspliceError::InvalidRoadmap {
                message: "roadmap must contain at least one numbered phase".to_owned(),
            });
        }

        Ok(self.document)
    }
}
