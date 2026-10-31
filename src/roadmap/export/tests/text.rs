//! Unit tests for export inline rendering and label recognition.

use markdown::mdast::{Node, Text};

use super::paragraph;
use crate::roadmap::export::text::{paragraph_entry, render_text};

#[test]
fn plain_paragraph_without_a_label_is_unlabelled() {
    let (key, value) =
        paragraph_entry(&paragraph("A plain sentence.")).expect("plain text should render");

    assert_eq!(key, None);
    assert_eq!(value, "A plain sentence.");
}

#[test]
fn plain_label_splits_at_the_first_colon() {
    let (key, value) =
        paragraph_entry(&paragraph("Question: what next?")).expect("labelled text should render");

    assert_eq!(key.as_deref(), Some("Question"));
    assert_eq!(value, "what next?");
}

#[test]
fn plain_label_rejects_digits_and_punctuation() {
    for source in ["Step 1: continue.", "A/B: value."] {
        let (key, value) =
            paragraph_entry(&paragraph(source)).expect("invalid labels stay in the value");

        assert_eq!(key, None, "unexpected label for {source:?}");
        assert_eq!(value, source, "the original text should be preserved");
    }
}

#[test]
fn empty_label_is_not_a_label() {
    let (key, value) = paragraph_entry(&paragraph(": value")).expect("text should render");

    assert_eq!(key, None);
    assert_eq!(value, ": value");
}

#[test]
fn strong_label_splits_and_is_removed_from_the_value() {
    let (key, value) =
        paragraph_entry(&paragraph("**Note:** body text")).expect("strong label should render");

    assert_eq!(key.as_deref(), Some("Note"));
    assert_eq!(value, "body text");
}

#[test]
fn strong_without_a_trailing_colon_is_not_a_label() {
    let (key, value) =
        paragraph_entry(&paragraph("**Note** body text")).expect("strong text should render");

    assert_eq!(key, None);
    assert_eq!(value, "**Note** body text");
}

#[test]
fn strong_label_with_unsupported_characters_is_not_a_label() {
    let (key, value) =
        paragraph_entry(&paragraph("**Note 2:** body")).expect("strong text should render");

    assert_eq!(key, None);
    assert_eq!(value, "**Note 2:** body");
}

#[test]
fn non_text_first_inline_is_not_a_label() {
    let (key, value) =
        paragraph_entry(&paragraph("*Emphasis*: body")).expect("emphasis should render");

    assert_eq!(key, None);
    assert_eq!(value, "*Emphasis*: body");
}

#[test]
fn render_text_collapses_soft_wrapped_lines() {
    let paragraph = paragraph("alpha\nbeta");

    let rendered = render_text(&paragraph.children).expect("text should render");

    assert_eq!(rendered, "alpha beta");
}

#[test]
fn render_text_trims_and_collapses_explicit_whitespace() {
    let nodes = [Node::Text(Text {
        value: "  alpha    beta  ".to_owned(),
        position: None,
    })];

    let rendered = render_text(&nodes).expect("text should render");

    assert_eq!(rendered, "alpha beta");
}
