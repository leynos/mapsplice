//! Renumbering and dependency rewrite support.

use markdown::mdast::Node;

use super::super::{
    PhaseNumber,
    RoadmapDocument,
    StepNumber,
    TaskNumber,
    model::{RenumberPlan, SourceId},
};
use crate::{
    error::{MapspliceError, Result},
    roadmap::parse_anchor,
};

pub(super) fn renumber_document(roadmap: &mut RoadmapDocument) -> Result<RenumberPlan> {
    let mut plan = RenumberPlan::default();

    for (phase_index, phase) in roadmap.phases.iter_mut().enumerate() {
        let new_phase = PhaseNumber(to_number(phase_index + 1, "phase")?);
        plan.insert(
            phase.identity.source,
            phase.identity.anchor,
            new_phase.into(),
        );
        phase.number = new_phase;

        for (step_index, step) in phase.steps.iter_mut().enumerate() {
            let new_step = StepNumber::new(new_phase, to_number(step_index + 1, "step")?);
            plan.insert(step.identity.source, step.identity.anchor, new_step.into());
            step.number = new_step;

            for (task_index, task) in step.tasks.iter_mut().enumerate() {
                let new_task = TaskNumber::new(new_step, to_number(task_index + 1, "task")?);
                plan.insert(task.identity.source, task.identity.anchor, new_task.into());
                task.number = new_task;
            }
        }
    }

    Ok(plan)
}

pub(super) fn rewrite_dependencies(roadmap: &mut RoadmapDocument, plan: &RenumberPlan) {
    rewrite_nodes(&mut roadmap.preamble, SourceId::Target, plan);

    for phase in &mut roadmap.phases {
        rewrite_nodes(&mut phase.title, phase.identity.source, plan);
        rewrite_nodes(&mut phase.body, phase.identity.source, plan);
        rewrite_nodes(&mut phase.trailing, phase.identity.source, plan);
        for step in &mut phase.steps {
            rewrite_nodes(&mut step.title, step.identity.source, plan);
            rewrite_nodes(&mut step.body, step.identity.source, plan);
            rewrite_nodes(&mut step.trailing, step.identity.source, plan);
            for task in &mut step.tasks {
                rewrite_nodes(&mut task.summary, task.identity.source, plan);
                rewrite_nodes(&mut task.body, task.identity.source, plan);
            }
        }
    }
}

fn rewrite_nodes(nodes: &mut [Node], source: SourceId, plan: &RenumberPlan) {
    for node in nodes {
        rewrite_node(node, source, plan);
    }
}

fn rewrite_node(node: &mut Node, source: SourceId, plan: &RenumberPlan) {
    match node {
        Node::Text(text) => text.value = rewrite_text_value(&text.value, source, plan),
        Node::Root(root) => rewrite_nodes(&mut root.children, source, plan),
        Node::Paragraph(paragraph) => rewrite_nodes(&mut paragraph.children, source, plan),
        Node::Heading(heading) => rewrite_nodes(&mut heading.children, source, plan),
        Node::Blockquote(blockquote) => rewrite_nodes(&mut blockquote.children, source, plan),
        Node::List(list) => rewrite_nodes(&mut list.children, source, plan),
        Node::ListItem(item) => rewrite_nodes(&mut item.children, source, plan),
        Node::Emphasis(emphasis) => rewrite_nodes(&mut emphasis.children, source, plan),
        Node::Strong(strong) => rewrite_nodes(&mut strong.children, source, plan),
        Node::Delete(delete) => rewrite_nodes(&mut delete.children, source, plan),
        Node::Link(link) => rewrite_nodes(&mut link.children, source, plan),
        Node::LinkReference(link) => rewrite_nodes(&mut link.children, source, plan),
        Node::Table(table) => rewrite_nodes(&mut table.children, source, plan),
        Node::TableRow(row) => rewrite_nodes(&mut row.children, source, plan),
        Node::TableCell(cell) => rewrite_nodes(&mut cell.children, source, plan),
        Node::FootnoteDefinition(definition) => {
            rewrite_nodes(&mut definition.children, source, plan);
        }
        Node::MdxJsxFlowElement(element) => rewrite_nodes(&mut element.children, source, plan),
        Node::MdxJsxTextElement(element) => rewrite_nodes(&mut element.children, source, plan),
        _ => {}
    }
}

fn rewrite_text_value(value: &str, source: SourceId, plan: &RenumberPlan) -> String {
    let mut result = String::with_capacity(value.len());
    let mut index = 0;

    while index < value.len() {
        let Some((start, end)) = next_anchor_candidate(value, index) else {
            if let Some(tail) = value.get(index..) {
                result.push_str(tail);
            }
            break;
        };
        if let Some(prefix) = value.get(index..start) {
            result.push_str(prefix);
        }
        let Some(candidate) = value.get(start..end) else {
            index = end;
            continue;
        };
        let Ok(anchor) = parse_anchor(candidate) else {
            result.push_str(candidate);
            index = end;
            continue;
        };
        let replacement = plan
            .resolve(source, anchor)
            .or_else(|| plan.resolve_unique(anchor))
            .map_or_else(|| candidate.to_owned(), |mapped| mapped.to_string());
        result.push_str(&replacement);
        index = end;
    }

    result
}

fn next_anchor_candidate(value: &str, start_at: usize) -> Option<(usize, usize)> {
    let bytes = value.as_bytes();
    let mut start = start_at;

    while let Some(byte) = bytes.get(start) {
        if is_anchor_start(bytes, start, *byte) {
            return Some((start, consume_anchor(bytes, start)));
        }
        start += 1;
    }

    None
}

fn is_anchor_start(bytes: &[u8], start: usize, byte: u8) -> bool {
    byte.is_ascii_digit()
        && !bytes
            .get(start.wrapping_sub(1))
            .is_some_and(u8::is_ascii_alphanumeric)
}

fn consume_anchor(bytes: &[u8], start: usize) -> usize {
    let mut end = consume_digits(bytes, start);
    while has_dot_digit(bytes, end) {
        end = consume_digits(bytes, end + 1);
    }
    end
}

fn consume_digits(bytes: &[u8], start: usize) -> usize {
    let mut end = start;
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    end
}

fn has_dot_digit(bytes: &[u8], dot_index: usize) -> bool {
    matches!(
        (bytes.get(dot_index), bytes.get(dot_index + 1)),
        (Some(b'.'), Some(next)) if next.is_ascii_digit()
    )
}

fn to_number(value: usize, label: &str) -> Result<u32> {
    u32::try_from(value).map_err(|_| MapspliceError::InvalidRoadmap {
        message: format!("{label} count exceeds supported numbering range"),
    })
}
