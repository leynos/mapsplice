//! Dependency validation for export items using the rewriter's shared scanner.

use std::{collections::HashSet, sync::Arc};

use markdown::mdast::{ListItem, Node, Paragraph};

use super::text::{entry_from_rendered, paragraph_entry, render_text};
use crate::{
    error::{MapspliceError, Result, SourceLocation},
    roadmap::{
        model::{MarkdownNodes, SubTaskEntry, TaskChild, TaskEntry},
        source_location::source_location,
    },
};

/// How a paragraph relates to a `Requires` dependency clause.
#[derive(Debug, Eq, PartialEq)]
pub(super) enum RequiresParagraph {
    /// The paragraph is ordinary content.
    Content,
    /// The paragraph contains only a dependency clause.
    ClauseOnly,
    /// The paragraph contains a dependency clause followed by prose.
    ClauseWithProse(String),
}

/// Classify a paragraph's dependency clause and any prose that follows it.
///
/// The clause ends at its last recognized anchor, so prose after that point is
/// ordinary content. Without this split, a paragraph such as
/// `Requires 1.1.1. Success: keep this.` would silently lose its `Success`
/// entry. Text between the keyword and the final anchor remains part of the
/// clause, matching the shared rewriter's scanner.
pub(super) fn classify_paragraph(nodes: &[Node]) -> Result<RequiresParagraph> {
    let rendered = render_text(nodes)?;
    let trimmed = rendered.trim_start();
    if opens_with_formatted_keyword(nodes)? {
        // `dependencies` rejects this form; classification only keeps the
        // paragraph from being exported as content on the way to that error.
        return Ok(RequiresParagraph::ClauseOnly);
    }
    if !opens_with_plain_keyword(nodes, "Requires") {
        return Ok(RequiresParagraph::Content);
    }
    let clause_end = super::super::ops::dependency_candidates(trimmed)
        .last()
        .map_or("Requires".len(), |candidate| candidate.range.end);
    let prose = trimmed
        .get(clause_end..)
        .unwrap_or_default()
        .trim_start_matches(|character: char| {
            character.is_whitespace() || matches!(character, '.' | ',' | ';' | ':')
        });
    Ok(if prose.is_empty() {
        RequiresParagraph::ClauseOnly
    } else {
        RequiresParagraph::ClauseWithProse(prose.to_owned())
    })
}

/// Return the entry a paragraph contributes, or `None` for a dependency clause.
pub(super) fn paragraph_entry_or_clause(
    paragraph: &Paragraph,
) -> Result<Option<(Option<String>, String)>> {
    match classify_paragraph(&paragraph.children)? {
        RequiresParagraph::Content => paragraph_entry(paragraph).map(Some),
        RequiresParagraph::ClauseOnly => Ok(None),
        RequiresParagraph::ClauseWithProse(prose) => Ok(Some(entry_from_rendered(&prose))),
    }
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
    let source = first_source(std::iter::once(title).chain(content.iter().copied()));
    dependencies(&nodes, source.as_deref(), known_anchors, true)
}

/// Collect dependencies owned by a task, excluding its nested checkbox items.
pub(super) fn from_task(task: &TaskEntry, known_anchors: &HashSet<String>) -> Result<Vec<String>> {
    let mut nodes = task.summary.nodes().to_vec();
    let mut bodies = Vec::new();
    for child in task.children() {
        if let TaskChild::Body(body) = child {
            nodes.extend_from_slice(body.nodes());
            bodies.push(body);
        }
    }
    let source = first_source(std::iter::once(&task.summary).chain(bodies));
    dependencies(&nodes, source.as_deref(), known_anchors, true)
}

/// Collect dependencies owned by a structural sub-task, excluding nested checkboxes.
pub(super) fn from_sub_task(
    sub_task: &SubTaskEntry,
    known_anchors: &HashSet<String>,
) -> Result<Vec<String>> {
    let mut nodes = sub_task.summary.nodes().to_vec();
    nodes.extend_from_slice(sub_task.body.nodes());
    let source = first_source([&sub_task.summary, &sub_task.body]);
    dependencies(&nodes, source.as_deref(), known_anchors, true)
}

/// Collect dependencies owned by an unnumbered checkbox item.
///
/// A bare list item carries no document handle of its own, so the caller
/// supplies the document the item was parsed from. Without it, a diagnostic on
/// a wrapped clause would report the normalized text position instead of the
/// token's source column.
pub(super) fn from_checkbox(
    item: &ListItem,
    source_text: Option<&str>,
    known_anchors: &HashSet<String>,
) -> Result<Vec<String>> {
    let nodes = item.children.clone();
    dependencies(&nodes, source_text, known_anchors, true)
}

/// Return the first document text available among the given collections.
fn first_source<'a>(collections: impl IntoIterator<Item = &'a MarkdownNodes>) -> Option<Arc<str>> {
    collections.into_iter().find_map(MarkdownNodes::source)
}

/// Scan candidates in source order, rejecting malformed and unresolved anchors.
fn dependencies(
    nodes: &[Node],
    source_text: Option<&str>,
    known_anchors: &HashSet<String>,
    skip_checkbox_items: bool,
) -> Result<Vec<String>> {
    if let FormattedRequires::Found(location) = formatted_requires(nodes)? {
        return Err(MapspliceError::InvalidRoadmap {
            message: "formatted `Requires` keyword cannot be scanned for dependencies".to_owned(),
            location,
        });
    }

    let candidates =
        super::super::ops::collect_dependencies(nodes, source_text, skip_checkbox_items);
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
        let found = formatted_requires_in_node(node)?;
        if matches!(found, FormattedRequires::Found(_)) {
            return Ok(found);
        }
    }
    Ok(FormattedRequires::NotFound)
}

/// Check one node, then its descendants, for a bold `Requires` token.
fn formatted_requires_in_node(node: &Node) -> Result<FormattedRequires> {
    if let Node::Strong(strong) = node
        && render_text(&strong.children)?.trim() == "Requires"
    {
        return Ok(FormattedRequires::Found(
            node.position().map(source_location),
        ));
    }
    node_children(node).map_or_else(|| Ok(FormattedRequires::NotFound), formatted_requires)
}

/// Distinguish no bold keyword from a keyword whose AST position is absent.
enum FormattedRequires {
    /// No unreachable formatted keyword appeared in the subtree.
    NotFound,
    /// A formatted `Requires` keyword was found, with its optional location.
    Found(Option<SourceLocation>),
}

/// Return Markdown children for recursive formatted-keyword inspection.
pub(super) fn node_children(node: &Node) -> Option<&[Node]> {
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

/// Return whether the paragraph opens with a bold `Requires` keyword.
///
/// Classification reads the Markdown nodes rather than the rendered prefix.
/// Export rendering emits plain text literally, so escaped source such as
/// `\*\*Requires\*\*` produces the same characters as a bold keyword; a
/// rendered-prefix test would mistake that prose for a dependency clause and
/// drop it from the export without an error. Only whitespace between the
/// paragraph start and the strong node is ignored, matching `trim_start`.
fn opens_with_formatted_keyword(nodes: &[Node]) -> Result<bool> {
    let Some(node) = nodes.iter().find(|node| !is_ignorable_prefix(node)) else {
        return Ok(false);
    };
    let Node::Strong(strong) = node else {
        return Ok(false);
    };
    Ok(render_text(&strong.children)?.trim() == "Requires")
}

/// Return whether a node adds no content before a formatted keyword.
fn is_ignorable_prefix(node: &Node) -> bool {
    match node {
        Node::Text(text) => text.value.trim().is_empty(),
        Node::Break(_) => true,
        _ => false,
    }
}

/// Return whether the leading plain-text run opens with the keyword.
///
/// Only the leading run of text and break nodes is inspected, so a bold
/// keyword is never mistaken for plain text. Leading whitespace is trimmed
/// but not collapsed; the keyword is a single word, so both forms agree.
fn opens_with_plain_keyword(nodes: &[Node], keyword: &str) -> bool {
    leading_text_run(nodes).is_some_and(|leading| begins_keyword(leading.trim_start(), keyword))
}

/// Flatten the leading run of text and break nodes, stopping at anything else.
fn leading_text_run(nodes: &[Node]) -> Option<String> {
    let mut leading = String::new();
    for node in nodes {
        append_plain_text(node, &mut leading)?;
    }
    Some(leading)
}

/// Append one node's contribution to a plain-text run.
///
/// Returns `None` for a node that ends the run.
fn append_plain_text(node: &Node, leading: &mut String) -> Option<()> {
    match node {
        Node::Text(text) => leading.push_str(&text.value),
        Node::Break(_) => leading.push(' '),
        _ => return None,
    }
    Some(())
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
