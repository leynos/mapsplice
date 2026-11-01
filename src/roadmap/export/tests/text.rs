//! Unit tests for export inline rendering and label recognition.

use markdown::mdast::{Node, Text};
use rstest::rstest;

use super::paragraph;
use crate::roadmap::export::text::{paragraph_entry, render_text};

#[test]
fn plain_paragraph_without_a_label_is_unlabelled() {
    let (key, value) =
        paragraph_entry(&paragraph("A plain sentence.")).expect("plain text should render");

    assert_eq!(key, None);
    assert_eq!(value, "A plain sentence.");
}

/// A colon in plain text divides the key from the value at the first split.
#[test]
fn plain_label_splits_at_the_first_colon() {
    let (key, value) =
        paragraph_entry(&paragraph("Question: what next?")).expect("labelled text should render");

    assert_eq!(key.as_deref(), Some("Question"));
    assert_eq!(value, "what next?");
}

/// Text that cannot form a label is kept whole in the value.
#[rstest]
#[case::digit_in_label("Step 1: continue.")]
#[case::punctuation_in_label("A/B: value.")]
#[case::empty_label(": value")]
#[case::strong_without_colon("**Note** body text")]
#[case::strong_unsupported_characters("**Note 2:** body")]
#[case::emphasis_first("*Emphasis*: body")]
fn unrecognized_label_keeps_the_original_text(#[case] source: &str) {
    let (key, value) = paragraph_entry(&paragraph(source)).expect("text should render");

    assert_eq!(key, None, "unexpected label for {source:?}");
    assert_eq!(value, source, "the original text should be preserved");
}

#[test]
fn strong_label_splits_and_is_removed_from_the_value() {
    let (key, value) =
        paragraph_entry(&paragraph("**Note:** body text")).expect("strong label should render");

    assert_eq!(key.as_deref(), Some("Note"));
    assert_eq!(value, "body text");
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
