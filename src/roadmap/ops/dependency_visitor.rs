//! Shared dependency traversal for rewrite and export collection.

use std::ops::Range;

use markdown::mdast::{Node, Text};

use super::{
    super::{
        RoadmapAnchor,
        model::{RenumberPlan, SourceId},
    },
    dependency_text::{dependency_candidates, rewrite_text_value},
};
use crate::error::SourceLocation;

/// A dependency candidate and the Markdown source position of its text node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CollectedDependency {
    /// Parsed anchor, or `None` for a malformed dependency token.
    pub(crate) anchor: Option<RoadmapAnchor>,
    /// Source location of the text node containing the candidate.
    pub(crate) location: Option<SourceLocation>,
}

/// Select whether the shared dependency visitor rewrites or only collects.
pub(crate) enum DependencyVisitMode<'a> {
    /// Resolve and replace candidates using an existing renumber plan.
    Rewrite {
        /// Source document used for plan resolution.
        source: SourceId,
        /// Completed anchor remapping plan.
        plan: &'a RenumberPlan,
        /// Number of changed references accumulated by the traversal.
        rewrite_count: &'a mut u64,
        /// Unresolved anchors accumulated by the traversal.
        unresolved: &'a mut Vec<RoadmapAnchor>,
        /// Location of the first unresolved reference.
        unresolved_location: &'a mut Option<SourceLocation>,
    },
    /// Collect candidates and text-node source positions without mutation.
    Collect {
        /// Destination for candidates found during the traversal.
        references: &'a mut Vec<CollectedDependency>,
        /// Whether nested checkbox items belong to another export item.
        skip_checkbox_items: bool,
    },
}

/// Collect dependency candidates from Markdown nodes without changing them.
pub(crate) fn collect_dependencies(
    nodes: &[Node],
    skip_checkbox_items: bool,
) -> Vec<CollectedDependency> {
    let mut cloned_nodes = nodes.to_vec();
    let mut references = Vec::new();
    visit_dependency_nodes(
        &mut cloned_nodes,
        &mut DependencyVisitMode::Collect {
            references: &mut references,
            skip_checkbox_items,
        },
    );
    references
}

/// Visit each Markdown node in order and rewrite eligible text descendants.
///
/// Errors report unresolved dependency anchors from the traversal.
pub(crate) fn visit_dependency_nodes(nodes: &mut [Node], mode: &mut DependencyVisitMode<'_>) {
    let mut index = 0;
    while index < nodes.len() {
        if should_skip_checkbox(nodes.get(index), mode) {
            index += 1;
            continue;
        }
        if matches!(nodes.get(index), Some(Node::Text(_) | Node::Break(_))) {
            let start = index;
            while index < nodes.len()
                && matches!(nodes.get(index), Some(Node::Text(_) | Node::Break(_)))
            {
                index += 1;
            }
            if let Some(group) = nodes.get_mut(start..index) {
                visit_text_group(group, mode);
            }
        } else if let Some(node) = nodes.get_mut(index) {
            visit_dependency_node(node, mode);
            index += 1;
        }
    }
}

/// Skip checked list items when collecting content for their parent item.
const fn should_skip_checkbox(node: Option<&Node>, mode: &DependencyVisitMode<'_>) -> bool {
    matches!(
        (node, mode),
        (
            Some(Node::ListItem(item)),
            DependencyVisitMode::Collect {
                skip_checkbox_items: true,
                ..
            }
        ) if item.checked.is_some()
    )
}

/// A source text node's span in the flattened inline sequence.
struct TextSegment {
    /// Position within the current sibling group.
    index: usize,
    /// Byte range occupied in the flattened text.
    range: Range<usize>,
}

/// Visit adjacent text and hard-break nodes as one whitespace-normalized run.
///
/// Markdown represents a hard line break as a separate node, so combining the
/// run keeps dependency clauses consistent with soft-wrapped paragraphs.
fn visit_text_group(nodes: &mut [Node], mode: &mut DependencyVisitMode<'_>) {
    if let [Node::Text(text)] = nodes {
        visit_single_text(text, mode);
        return;
    }

    let mut flattened = String::new();
    let mut segments = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        match node {
            Node::Text(text) => {
                let start = flattened.len();
                flattened.push_str(&text.value);
                segments.push(TextSegment {
                    index,
                    range: start..flattened.len(),
                });
            }
            Node::Break(_) => flattened.push(' '),
            _ => {}
        }
    }

    let node_count = nodes.len();
    let mut visitor = TextGroupVisitor {
        nodes,
        replacements: vec![Vec::new(); node_count],
        mode,
    };
    for candidate in dependency_candidates(&flattened) {
        let Some((segment_index, local_range)) =
            candidate_text_segment(&candidate.range, &segments)
        else {
            continue;
        };
        visitor.record_candidate(&candidate, segment_index, local_range);
    }
    visitor.apply_replacements();
}

/// Find the text node containing a whole dependency token and its local span.
fn candidate_text_segment(
    candidate: &Range<usize>,
    segments: &[TextSegment],
) -> Option<(usize, Range<usize>)> {
    let segment = segments.iter().find(|segment| {
        candidate.start >= segment.range.start && candidate.end <= segment.range.end
    })?;
    Some((
        segment.index,
        candidate.start - segment.range.start..candidate.end - segment.range.start,
    ))
}

/// Visitor state for a flattened adjacent-text group.
struct TextGroupVisitor<'nodes, 'mode, 'context> {
    /// Mutable Markdown nodes in the adjacent text group being visited.
    nodes: &'nodes mut [Node],
    /// Replacements collected by source node index.
    replacements: Vec<Vec<(Range<usize>, String)>>,
    /// Collector or rewrite policy for this traversal.
    mode: &'mode mut DependencyVisitMode<'context>,
}

/// Source segment and byte range targeted by a dependency rewrite.
struct ReplacementTarget {
    /// Index of the owning text node in the visited group.
    segment_index: usize,
    /// Byte range occupied by the dependency token in that text node.
    range: Range<usize>,
}

impl TextGroupVisitor<'_, '_, '_> {
    /// Collect a candidate or record its rewrite outcome at its source node.
    fn record_candidate(
        &mut self,
        candidate: &super::dependency_text::DependencyCandidate,
        segment_index: usize,
        local_range: Range<usize>,
    ) {
        let Some(Node::Text(text)) = self.nodes.get_mut(segment_index) else {
            return;
        };
        let location = text
            .position
            .as_ref()
            .map(|position| SourceLocation::from(&position.start));
        match self.mode {
            DependencyVisitMode::Rewrite { .. } => {
                self.record_rewrite(
                    candidate.anchor,
                    ReplacementTarget {
                        segment_index,
                        range: local_range,
                    },
                    location,
                );
            }
            DependencyVisitMode::Collect { references, .. } => {
                references.push(CollectedDependency {
                    anchor: candidate.anchor,
                    location,
                });
            }
        }
    }

    /// Resolve one rewrite candidate or retain it as an unresolved dependency.
    fn record_rewrite(
        &mut self,
        candidate_anchor: Option<RoadmapAnchor>,
        target: ReplacementTarget,
        location: Option<SourceLocation>,
    ) {
        let DependencyVisitMode::Rewrite {
            source,
            plan,
            rewrite_count,
            unresolved,
            unresolved_location,
        } = self.mode
        else {
            return;
        };
        let Some(anchor) = candidate_anchor else {
            return;
        };
        if let Some(mapped) = plan.resolve_reference(*source, anchor) {
            **rewrite_count += u64::from(mapped != anchor);
            if mapped != anchor
                && let Some(replacement_group) = self.replacements.get_mut(target.segment_index)
            {
                replacement_group.push((target.range, mapped.to_string()));
            }
        } else {
            unresolved.push(anchor);
            if (**unresolved_location).is_none() {
                **unresolved_location = location;
            }
        }
    }

    /// Apply collected replacements from right to left in each source segment.
    fn apply_replacements(&mut self) {
        for (index, ranges) in self.replacements.iter_mut().enumerate() {
            if let Some(Node::Text(text)) = self.nodes.get_mut(index) {
                apply_text_replacements(text, ranges);
            }
        }
    }
}

/// Apply one text node's replacements from right to left.
fn apply_text_replacements(text: &mut Text, replacements: &mut Vec<(Range<usize>, String)>) {
    replacements.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    for (range, replacement) in replacements.drain(..) {
        text.value.replace_range(range, &replacement);
    }
}

/// Visit one non-text Markdown node, descending into containers as needed.
fn visit_dependency_node(node: &mut Node, mode: &mut DependencyVisitMode<'_>) {
    visit_container_node(node, mode);
}

/// Visit one text node with the same scanner used for mixed hard-break runs.
fn visit_single_text(text: &mut Text, mode: &mut DependencyVisitMode<'_>) {
    match mode {
        DependencyVisitMode::Rewrite {
            source,
            plan,
            rewrite_count,
            unresolved,
            unresolved_location,
        } => {
            let report = rewrite_text_value(&text.value, *source, plan);
            if !report.unresolved.is_empty() && unresolved_location.is_none() {
                **unresolved_location = text
                    .position
                    .as_ref()
                    .map(|position| SourceLocation::from(&position.start));
            }
            unresolved.extend(report.unresolved);
            text.value = report.value;
            **rewrite_count += report.rewrite_count;
        }
        DependencyVisitMode::Collect { references, .. } => {
            let location = text
                .position
                .as_ref()
                .map(|position| SourceLocation::from(&position.start));
            references.extend(
                dependency_candidates(&text.value)
                    .into_iter()
                    .map(|candidate| CollectedDependency {
                        anchor: candidate.anchor,
                        location,
                    }),
            );
        }
    }
}

/// Visit children of a Markdown container while respecting excluded leaves.
///
/// Inline code, HTML, and link destinations are leaves and are never visited.
fn visit_container_node(node: &mut Node, mode: &mut DependencyVisitMode<'_>) {
    match node {
        Node::Root(root) => visit_dependency_nodes(&mut root.children, mode),
        Node::Paragraph(paragraph) => visit_dependency_nodes(&mut paragraph.children, mode),
        Node::Heading(heading) => visit_dependency_nodes(&mut heading.children, mode),
        Node::Blockquote(blockquote) => visit_dependency_nodes(&mut blockquote.children, mode),
        Node::List(list) => visit_dependency_nodes(&mut list.children, mode),
        Node::ListItem(item) => visit_dependency_nodes(&mut item.children, mode),
        Node::Table(table) => visit_dependency_nodes(&mut table.children, mode),
        Node::TableRow(row) => visit_dependency_nodes(&mut row.children, mode),
        Node::TableCell(cell) => visit_dependency_nodes(&mut cell.children, mode),
        Node::FootnoteDefinition(definition) => {
            visit_dependency_nodes(&mut definition.children, mode);
        }
        Node::Break(_) | Node::Text(_) => {}
        _ => visit_inline_container_node(node, mode),
    }
}

/// Visit children of an inline Markdown container.
///
/// Link children are visible text; destinations remain excluded.
fn visit_inline_container_node(node: &mut Node, mode: &mut DependencyVisitMode<'_>) {
    match node {
        Node::Emphasis(emphasis) => visit_dependency_nodes(&mut emphasis.children, mode),
        Node::Strong(strong) => visit_dependency_nodes(&mut strong.children, mode),
        Node::Delete(delete) => visit_dependency_nodes(&mut delete.children, mode),
        Node::Link(link) => visit_dependency_nodes(&mut link.children, mode),
        Node::LinkReference(link) => visit_dependency_nodes(&mut link.children, mode),
        _ => visit_mdx_container_node(node, mode),
    }
}

/// Visit children of an MDX container, leaving leaf nodes unchanged.
///
/// The visitor never inspects JSX attributes or HTML source.
fn visit_mdx_container_node(node: &mut Node, mode: &mut DependencyVisitMode<'_>) {
    match node {
        Node::MdxJsxFlowElement(element) => visit_dependency_nodes(&mut element.children, mode),
        Node::MdxJsxTextElement(element) => visit_dependency_nodes(&mut element.children, mode),
        _ => {}
    }
}
