//! Dependency validation for export items using the rewriter's shared scanner.

use std::collections::HashSet;

use markdown::mdast::{ListItem, Node};

use super::text::render_text;
use crate::{
    error::{MapspliceError, Result, SourceLocation},
    roadmap::model::{MarkdownNodes, SubTaskEntry, TaskChild, TaskEntry},
};

/// Return whether paragraph content begins a dependency clause.
pub(super) fn is_requires_clause(nodes: &[Node]) -> Result<bool> {
    let rendered = render_text(nodes)?;
    let trimmed = rendered.trim_start();
    Ok(trimmed.starts_with("**Requires**") || begins_keyword(trimmed, "Requires"))
}

/// Collect and validate dependencies for phase or step content.
pub(super) fn from_markdown(
    title: &MarkdownNodes,
    content: &[&MarkdownNodes],
    known_anchors: &HashSet<String>,
) -> Result<Vec<String>> {
    let mut nodes = title.nodes().to_vec();
    for section in content {
        nodes.extend_from_slice(section.nodes());
    }
    dependencies(&nodes, known_anchors, true)
}

/// Collect dependencies owned by a task, excluding its nested checkbox items.
pub(super) fn from_task(task: &TaskEntry, known_anchors: &HashSet<String>) -> Result<Vec<String>> {
    let mut nodes = task.summary.nodes().to_vec();
    for child in task.children() {
        if let TaskChild::Body(body) = child {
            nodes.extend_from_slice(body.nodes());
        }
    }
    dependencies(&nodes, known_anchors, true)
}

/// Collect dependencies owned by a structural sub-task, excluding nested checkboxes.
pub(super) fn from_sub_task(
    sub_task: &SubTaskEntry,
    known_anchors: &HashSet<String>,
) -> Result<Vec<String>> {
    let mut nodes = sub_task.summary.nodes().to_vec();
    nodes.extend_from_slice(sub_task.body.nodes());
    dependencies(&nodes, known_anchors, true)
}

/// Collect dependencies owned by an unnumbered checkbox item.
pub(super) fn from_checkbox(
    item: &ListItem,
    known_anchors: &HashSet<String>,
) -> Result<Vec<String>> {
    let nodes = item.children.clone();
    dependencies(&nodes, known_anchors, true)
}

/// Scan candidates in source order, rejecting malformed and unresolved anchors.
fn dependencies(
    nodes: &[Node],
    known_anchors: &HashSet<String>,
    skip_checkbox_items: bool,
) -> Result<Vec<String>> {
    if let FormattedRequires::Found(location) = formatted_requires(nodes)? {
        return Err(MapspliceError::InvalidRoadmap {
            message: "formatted `Requires` keyword cannot be scanned for dependencies".to_owned(),
            location,
        });
    }

    let candidates = super::super::ops::collect_dependencies(nodes, skip_checkbox_items);
    let mut dependencies = Vec::new();
    for candidate in candidates {
        let location = candidate.location;
        let Some(anchor) = candidate.anchor else {
            return Err(MapspliceError::InvalidRoadmap {
                message: "invalid dependency anchor token".to_owned(),
                location,
            });
        };
        let number = anchor.to_string();
        if !known_anchors.contains(&number) {
            return Err(MapspliceError::DanglingDependency { anchor, location });
        }
        if !dependencies.contains(&number) {
            dependencies.push(number);
        }
    }
    Ok(dependencies)
}

/// Find a bold `Requires` token that the plain-text scanner cannot reach.
fn formatted_requires(nodes: &[Node]) -> Result<FormattedRequires> {
    for node in nodes {
        if let Node::Strong(strong) = node
            && render_text(&strong.children)?.trim() == "Requires"
        {
            return Ok(FormattedRequires::Found(
                node.position()
                    .map(|position| SourceLocation::from(&position.start)),
            ));
        }
        if let Some(children) = node_children(node)
            && let found @ FormattedRequires::Found(_) = formatted_requires(children)?
        {
            return Ok(found);
        }
    }
    Ok(FormattedRequires::NotFound)
}

/// Distinguish no bold keyword from a keyword whose AST position is absent.
enum FormattedRequires {
    /// No unreachable formatted keyword appeared in the subtree.
    NotFound,
    /// A formatted `Requires` keyword was found, with its optional location.
    Found(Option<SourceLocation>),
}

/// Return Markdown children for recursive formatted-keyword inspection.
fn node_children(node: &Node) -> Option<&[Node]> {
    match node {
        Node::Root(value) => Some(&value.children),
        Node::Paragraph(value) => Some(&value.children),
        Node::Heading(value) => Some(&value.children),
        Node::Blockquote(value) => Some(&value.children),
        Node::List(value) => Some(&value.children),
        Node::ListItem(value) => Some(&value.children),
        Node::Emphasis(value) => Some(&value.children),
        Node::Strong(value) => Some(&value.children),
        Node::Delete(value) => Some(&value.children),
        Node::Link(value) => Some(&value.children),
        Node::LinkReference(value) => Some(&value.children),
        Node::Table(value) => Some(&value.children),
        Node::TableRow(value) => Some(&value.children),
        Node::TableCell(value) => Some(&value.children),
        Node::FootnoteDefinition(value) => Some(&value.children),
        Node::MdxJsxFlowElement(value) => Some(&value.children),
        Node::MdxJsxTextElement(value) => Some(&value.children),
        _ => None,
    }
}

/// Return whether a rendered value starts with a whole keyword.
fn begins_keyword(value: &str, keyword: &str) -> bool {
    value.strip_prefix(keyword).is_some_and(|remainder| {
        remainder
            .chars()
            .next()
            .is_none_or(|character| !character.is_ascii_alphanumeric())
    })
}
