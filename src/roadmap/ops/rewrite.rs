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

/// Renumber every roadmap item and return the old-to-new mapping.
pub(super) fn renumber_document(roadmap: &mut RoadmapDocument) -> Result<RenumberPlan> {
    let mut plan = RenumberPlan::default();

    for (phase_index, phase) in roadmap.phases.iter_mut().enumerate() {
        let new_phase = PhaseNumber::new(to_number(phase_index + 1, "phase")?)?;
        plan.insert(
            phase.identity.source,
            phase.identity.anchor,
            new_phase.into(),
        );
        phase.number = new_phase;

        for (step_index, step) in phase.steps.iter_mut().enumerate() {
            let new_step = StepNumber::new(new_phase, to_number(step_index + 1, "step")?)?;
            plan.insert(step.identity.source, step.identity.anchor, new_step.into());
            step.number = new_step;

            for (task_index, task) in step.tasks.iter_mut().enumerate() {
                let new_task = TaskNumber::new(new_step, to_number(task_index + 1, "task")?)?;
                plan.insert(task.identity.source, task.identity.anchor, new_task.into());
                task.number = new_task;
            }
        }
    }

    Ok(plan)
}

/// Rewrite anchor-like text references using a completed renumbering plan.
pub(super) fn rewrite_dependencies(
    roadmap: &mut RoadmapDocument,
    plan: &RenumberPlan,
) -> Result<u64> {
    let mut rewrite_count = 0;
    rewrite_nodes(
        roadmap.preamble.nodes_mut(),
        SourceId::Target,
        plan,
        &mut rewrite_count,
    )?;

    for phase in &mut roadmap.phases {
        rewrite_nodes(
            phase.title.nodes_mut(),
            phase.identity.source,
            plan,
            &mut rewrite_count,
        )?;
        rewrite_nodes(
            phase.body.nodes_mut(),
            phase.identity.source,
            plan,
            &mut rewrite_count,
        )?;
        rewrite_nodes(
            phase.trailing.nodes_mut(),
            phase.identity.source,
            plan,
            &mut rewrite_count,
        )?;
        for step in &mut phase.steps {
            rewrite_nodes(
                step.title.nodes_mut(),
                step.identity.source,
                plan,
                &mut rewrite_count,
            )?;
            rewrite_nodes(
                step.body.nodes_mut(),
                step.identity.source,
                plan,
                &mut rewrite_count,
            )?;
            rewrite_nodes(
                step.trailing.nodes_mut(),
                step.identity.source,
                plan,
                &mut rewrite_count,
            )?;
            for task in &mut step.tasks {
                rewrite_nodes(
                    task.summary.nodes_mut(),
                    task.identity.source,
                    plan,
                    &mut rewrite_count,
                )?;
                rewrite_nodes(
                    task.body.nodes_mut(),
                    task.identity.source,
                    plan,
                    &mut rewrite_count,
                )?;
            }
        }
    }
    Ok(rewrite_count)
}

/// Rewrite every eligible text node in a node slice.
fn rewrite_nodes(
    nodes: &mut [Node],
    source: SourceId,
    plan: &RenumberPlan,
    rewrite_count: &mut u64,
) -> Result<()> {
    for node in nodes {
        rewrite_node(node, source, plan, rewrite_count)?;
    }
    Ok(())
}

/// Rewrite one Markdown node, recursing into child-bearing nodes.
fn rewrite_node(
    node: &mut Node,
    source: SourceId,
    plan: &RenumberPlan,
    rewrite_count: &mut u64,
) -> Result<()> {
    if let Node::Text(text) = node {
        let (rewritten, count) = rewrite_text_value(&text.value, source, plan)?;
        text.value = rewritten;
        *rewrite_count += count;
        return Ok(());
    }

    rewrite_container_node(node, source, plan, rewrite_count)
}

/// Rewrite block-level container children.
fn rewrite_container_node(
    node: &mut Node,
    source: SourceId,
    plan: &RenumberPlan,
    rewrite_count: &mut u64,
) -> Result<()> {
    match node {
        Node::Root(root) => rewrite_nodes(&mut root.children, source, plan, rewrite_count),
        Node::Paragraph(paragraph) => {
            rewrite_nodes(&mut paragraph.children, source, plan, rewrite_count)
        }
        Node::Heading(heading) => rewrite_nodes(&mut heading.children, source, plan, rewrite_count),
        Node::Blockquote(blockquote) => {
            rewrite_nodes(&mut blockquote.children, source, plan, rewrite_count)
        }
        Node::List(list) => rewrite_nodes(&mut list.children, source, plan, rewrite_count),
        Node::ListItem(item) => rewrite_nodes(&mut item.children, source, plan, rewrite_count),
        Node::Table(table) => rewrite_nodes(&mut table.children, source, plan, rewrite_count),
        Node::TableRow(row) => rewrite_nodes(&mut row.children, source, plan, rewrite_count),
        Node::TableCell(cell) => rewrite_nodes(&mut cell.children, source, plan, rewrite_count),
        Node::FootnoteDefinition(definition) => {
            rewrite_nodes(&mut definition.children, source, plan, rewrite_count)
        }
        _ => rewrite_inline_container_node(node, source, plan, rewrite_count),
    }
}

/// Rewrite inline container children.
fn rewrite_inline_container_node(
    node: &mut Node,
    source: SourceId,
    plan: &RenumberPlan,
    rewrite_count: &mut u64,
) -> Result<()> {
    match node {
        Node::Emphasis(emphasis) => {
            rewrite_nodes(&mut emphasis.children, source, plan, rewrite_count)
        }
        Node::Strong(strong) => rewrite_nodes(&mut strong.children, source, plan, rewrite_count),
        Node::Delete(delete) => rewrite_nodes(&mut delete.children, source, plan, rewrite_count),
        Node::Link(link) => rewrite_nodes(&mut link.children, source, plan, rewrite_count),
        Node::LinkReference(link) => rewrite_nodes(&mut link.children, source, plan, rewrite_count),
        _ => rewrite_mdx_container_node(node, source, plan, rewrite_count),
    }
}

/// Rewrite MDX container children.
fn rewrite_mdx_container_node(
    node: &mut Node,
    source: SourceId,
    plan: &RenumberPlan,
    rewrite_count: &mut u64,
) -> Result<()> {
    match node {
        Node::MdxJsxFlowElement(element) => {
            rewrite_nodes(&mut element.children, source, plan, rewrite_count)
        }
        Node::MdxJsxTextElement(element) => {
            rewrite_nodes(&mut element.children, source, plan, rewrite_count)
        }
        _ => Ok(()),
    }
}

/// Rewrite anchor candidates within a text value and return the rewrite count.
fn rewrite_text_value(value: &str, source: SourceId, plan: &RenumberPlan) -> Result<(String, u64)> {
    let mut result = String::with_capacity(value.len());
    let mut rewrite_count = 0;
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
        if !is_dependency_anchor(value, start) {
            result.push_str(candidate);
            index = end;
            continue;
        }
        let Ok(anchor) = parse_anchor(candidate) else {
            result.push_str(candidate);
            index = end;
            continue;
        };
        let mapped = plan
            .resolve(source, anchor)
            .or_else(|| plan.resolve_unique(anchor))
            .ok_or(MapspliceError::DanglingDependency { anchor })?;
        rewrite_count += 1;
        result.push_str(&mapped.to_string());
        index = end;
    }

    Ok((result, rewrite_count))
}

/// Return whether the candidate appears in a supported dependency clause.
fn is_dependency_anchor(value: &str, start: usize) -> bool {
    ["Requires", "Blocks"].into_iter().any(|keyword| {
        latest_keyword_before(value, start, keyword).is_some_and(|position| {
            is_keyword_boundary(value, position, keyword.len())
                && has_dependency_clause_separator(value, position + keyword.len(), start)
        })
    })
}

/// Find the latest dependency keyword before an anchor candidate.
fn latest_keyword_before(value: &str, start: usize, keyword: &str) -> Option<usize> {
    value
        .get(..start)?
        .rmatch_indices(keyword)
        .next()
        .map(|(position, _)| position)
}

/// Return whether a dependency keyword is bounded as a word.
fn is_keyword_boundary(value: &str, position: usize, length: usize) -> bool {
    let bytes = value.as_bytes();
    !bytes
        .get(position.wrapping_sub(1))
        .is_some_and(u8::is_ascii_alphanumeric)
        && !bytes
            .get(position + length)
            .is_some_and(u8::is_ascii_alphanumeric)
}

/// Return whether text between keyword and anchor still looks like a clause.
fn has_dependency_clause_separator(value: &str, after_keyword: usize, anchor_start: usize) -> bool {
    value
        .get(after_keyword..anchor_start)
        .is_some_and(|between| {
            !between.contains('\n')
                && !between.chars().any(is_dependency_clause_terminator)
                && (between.contains(':') || between.starts_with(' '))
        })
}

/// Return whether a character terminates a dependency clause.
const fn is_dependency_clause_terminator(character: char) -> bool { matches!(character, '.' | ';') }

/// Find the next anchor-shaped token with leading and trailing boundaries.
fn next_anchor_candidate(value: &str, start_at: usize) -> Option<(usize, usize)> {
    let bytes = value.as_bytes();
    let mut start = start_at;

    while let Some(byte) = bytes.get(start) {
        if is_anchor_start(bytes, start, *byte) {
            let end = consume_anchor(bytes, start);
            if is_anchor_end(bytes, end) {
                return Some((start, end));
            }
        }
        start += 1;
    }

    None
}

/// Return whether an anchor candidate ends before another alphanumeric byte.
fn is_anchor_end(bytes: &[u8], end: usize) -> bool {
    !bytes.get(end).is_some_and(u8::is_ascii_alphanumeric)
}

/// Return whether a byte can start an anchor candidate.
fn is_anchor_start(bytes: &[u8], start: usize, byte: u8) -> bool {
    byte.is_ascii_digit()
        && !bytes
            .get(start.wrapping_sub(1))
            .is_some_and(u8::is_ascii_alphanumeric)
}

/// Consume a dotted numeric anchor candidate from its starting byte.
fn consume_anchor(bytes: &[u8], start: usize) -> usize {
    let mut end = consume_digits(bytes, start);
    while has_dot_digit(bytes, end) {
        end = consume_digits(bytes, end + 1);
    }
    end
}

/// Consume consecutive ASCII digits.
fn consume_digits(bytes: &[u8], start: usize) -> usize {
    let mut end = start;
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    end
}

/// Return whether a dot is followed by another digit segment.
fn has_dot_digit(bytes: &[u8], dot_index: usize) -> bool {
    matches!(
        (bytes.get(dot_index), bytes.get(dot_index + 1)),
        (Some(b'.'), Some(next)) if next.is_ascii_digit()
    )
}

/// Convert a collection index into a supported roadmap number.
fn to_number(value: usize, label: &str) -> Result<u32> {
    u32::try_from(value).map_err(|_| MapspliceError::InvalidRoadmap {
        message: format!("{label} count exceeds supported numbering range"),
    })
}
