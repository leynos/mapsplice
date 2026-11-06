//! Inline rendering, whitespace normalization, and label recognition.

use markdown::mdast::{Node, Paragraph};

use super::super::render::inline::render_inline_export;
use crate::error::Result;

/// Render inline Markdown while collapsing source whitespace to one space.
pub(super) fn render_text(nodes: &[Node]) -> Result<String> {
    render_inline_export(nodes).map(|rendered| collapse_whitespace(&rendered))
}

/// Render a paragraph as a labelled or unlabelled export entry.
pub(super) fn paragraph_entry(paragraph: &Paragraph) -> Result<(Option<String>, String)> {
    if let Some((label, remainder)) = split_label(&paragraph.children)? {
        return Ok((Some(label), render_text(&remainder)?));
    }
    Ok((None, render_text(&paragraph.children)?))
}

/// Split already-rendered paragraph text into its optional label and body.
///
/// This mirrors [`paragraph_entry`] for text that arrives as a rendered
/// string rather than as inline nodes, such as prose recovered after a
/// dependency clause. For example, `Success: keep this.` becomes
/// `(Some("Success"), "keep this.")`.
pub(super) fn entry_from_rendered(rendered: &str) -> (Option<String>, String) {
    let Some((label, remainder)) = rendered.split_once(':') else {
        return (None, rendered.to_owned());
    };
    let normalized = collapse_whitespace(label);
    if !is_label(&normalized) {
        return (None, rendered.to_owned());
    }
    (
        Some(normalized.trim().to_owned()),
        collapse_whitespace(remainder),
    )
}

/// Split a leading plain or strong label from inline nodes.
fn split_label(nodes: &[Node]) -> Result<Option<(String, Vec<Node>)>> {
    let Some(first) = nodes.first() else {
        return Ok(None);
    };
    match first {
        Node::Text(_) | Node::Break(_) => Ok(split_plain_label(nodes)),
        Node::Strong(strong) => {
            let rendered_label = render_text(&strong.children)?;
            let Some(label) = rendered_label.strip_suffix(':') else {
                return Ok(None);
            };
            if !is_label(label) {
                return Ok(None);
            }
            Ok(Some((
                label.trim().to_owned(),
                nodes.get(1..).unwrap_or_default().to_vec(),
            )))
        }
        _ => Ok(None),
    }
}

/// Split a plain label that may span a soft or hard line break.
///
/// Markdown keeps a soft line ending inside one text node but represents a
/// hard break as a separate node, so the label is read from the leading run of
/// text and break nodes rather than from the first node alone. Whitespace is
/// normalized before the label is validated, which makes a wrapped or
/// space-padded label export identically to its single-line form.
fn split_plain_label(nodes: &[Node]) -> Option<(String, Vec<Node>)> {
    let run_end = nodes
        .iter()
        .position(|node| !matches!(node, Node::Text(_) | Node::Break(_)))
        .unwrap_or(nodes.len());
    let run = nodes.get(..run_end)?;
    let flattened = flatten_text_run(run)?;
    let (label, _) = flattened.split_once(':')?;
    let normalized = collapse_whitespace(label);
    if !is_label(&normalized) {
        return None;
    }
    let (colon_index, local_offset) = locate_colon(run, label.len())?;
    let mut remainder = Vec::new();
    if let Some(Node::Text(node)) = run.get(colon_index) {
        let mut remainder_text = node.clone();
        node.value
            .get(local_offset + 1..)
            .unwrap_or_default()
            .clone_into(&mut remainder_text.value);
        remainder.push(Node::Text(remainder_text));
    }
    remainder.extend_from_slice(run.get(colon_index + 1..).unwrap_or_default());
    remainder.extend_from_slice(nodes.get(run_end..).unwrap_or_default());
    Some((normalized, remainder))
}

/// Flatten a run of text and break nodes, rendering each break as a space.
fn flatten_text_run(nodes: &[Node]) -> Option<String> {
    let mut flattened = String::new();
    for node in nodes {
        match node {
            Node::Text(text) => flattened.push_str(&text.value),
            Node::Break(_) => flattened.push(' '),
            _ => return None,
        }
    }
    Some(flattened)
}

/// Find the node and local byte offset that hold the colon.
fn locate_colon(nodes: &[Node], colon_offset: usize) -> Option<(usize, usize)> {
    let mut consumed = 0;
    for (index, node) in nodes.iter().enumerate() {
        let length = match node {
            Node::Text(text) => text.value.len(),
            Node::Break(_) => 1,
            _ => return None,
        };
        if colon_offset < consumed + length {
            return Some((index, colon_offset - consumed));
        }
        consumed += length;
    }
    None
}

/// Return whether a phrase is a supported short label.
fn is_label(label: &str) -> bool {
    let normalized = label.trim();
    !normalized.is_empty()
        && normalized.chars().all(|character| {
            character.is_ascii_alphabetic() || character == ' ' || character == '-'
        })
}

/// Collapse every whitespace run and trim both ends.
fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
