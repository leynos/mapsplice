//! Unit tests for export inline rendering and label recognition.

use markdown::mdast::{Node, Text};
use proptest::prelude::*;
use rstest::rstest;

use super::paragraph;
use crate::roadmap::{
    export::text::{paragraph_entry, render_text},
    render::inline::render_inline_export,
};

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

/// Labels export identically however their internal whitespace is written.
#[rstest]
#[case::single_space("Proof first: demonstrate the invariant.")]
#[case::repeated_spaces("Proof   first: demonstrate the invariant.")]
#[case::soft_wrap("Proof\nfirst: demonstrate the invariant.")]
#[case::hard_break("Proof  \nfirst: demonstrate the invariant.")]
#[case::tab("Proof\tfirst: demonstrate the invariant.")]
fn label_whitespace_is_normalized_before_recognition(#[case] source: &str) {
    let (key, value) = paragraph_entry(&paragraph(source)).expect("text should render");

    assert_eq!(key.as_deref(), Some("Proof first"));
    assert_eq!(value, "demonstrate the invariant.");
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

/// Export text is data, not Markdown source, so it must not carry the escape
/// backslashes that the rewrite renderer needs to keep Markdown reparsable.
#[rstest]
#[case::parentheses("Cost (estimate) stays literal.")]
#[case::underscore("A snake_case name stays literal.")]
#[case::brackets("A [bracketed] phrase stays literal.")]
#[case::hash("Issue #144 stays literal.")]
fn render_text_emits_literal_metacharacters(#[case] source: &str) {
    let (key, value) = paragraph_entry(&paragraph(source)).expect("text should render");

    assert_eq!(key, None, "unexpected label for {source:?}");
    assert_eq!(value, source, "metacharacters should survive unescaped");
}

/// Supported formatting nodes still retain their Markdown delimiters; only
/// plain text is emitted literally.
#[test]
fn render_text_preserves_supported_formatting_nodes() {
    let (key, value) = paragraph_entry(&paragraph("Keep **bold** and `code` (literal)."))
        .expect("formatted text should render");

    assert_eq!(key, None);
    assert_eq!(value, "Keep **bold** and `code` (literal).");
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

/// Characters the Markdown renderer escapes, plus characters that begin
/// Markdown constructs and ordinary letters and digits to combine them.
const LITERAL_CHARACTERS: &[char] = &[
    '\\', '*', '_', '`', '(', ')', '[', ']', '#', '-', '+', '!', '<', '>', '&', '|', '{', '}',
    '\'', '"', ' ', '\t', 'a', 'Z', '0', '.',
];

/// Generate plain text from characters that invite escaping.
fn literal_text() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(LITERAL_CHARACTERS), 0..24)
        .prop_map(|characters| characters.into_iter().collect())
}

proptest! {
    /// Export rendering is the identity on plain text. It must not add,
    /// remove or transform characters, so literal backslashes and Markdown
    /// metacharacters reach consumers exactly as they appear in the source.
    /// This exercises the renderer on its own: whitespace normalization is a
    /// separate step, deliberately excluded here so its collapsing cannot
    /// mask a rendering change.
    #[test]
    fn export_rendering_keeps_literal_text_unchanged(value in literal_text()) {
        let nodes = [Node::Text(Text {
            value: value.clone(),
            position: None,
        })];

        let rendered = render_inline_export(&nodes).expect("text should render");

        prop_assert_eq!(rendered.as_str(), value.as_str());
    }
}
