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

/// Split a leading plain or strong label from inline nodes.
fn split_label(nodes: &[Node]) -> Result<Option<(String, Vec<Node>)>> {
    let Some(first) = nodes.first() else {
        return Ok(None);
    };
    match first {
        Node::Text(text) => {
            let Some((label, suffix)) = split_label_text(&text.value) else {
                return Ok(None);
            };
            let mut remainder = nodes.to_vec();
            if let Some(Node::Text(first_text)) = remainder.first_mut() {
                first_text.value = suffix;
            }
            Ok(Some((label, remainder)))
        }
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

/// Split a text prefix at a colon when its label uses the accepted alphabet.
fn split_label_text(value: &str) -> Option<(String, String)> {
    let (label, remainder) = value.split_once(':')?;
    is_label(label).then(|| (label.trim().to_owned(), remainder.to_owned()))
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
