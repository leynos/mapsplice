//! Adapter from Markdown parser positions to roadmap source coordinates.
//!
//! Roadmap diagnostics record [`SourceLocation`] values so that the error types
//! never depend on the Markdown AST. This module is the single place where a
//! parser [`Position`] crosses into the roadmap domain, which keeps the
//! conversion at the parser boundary instead of inside the shared error type.

use std::ops::Range;

use markdown::{
    mdast::Text,
    unist::{Point, Position},
};

use crate::error::SourceLocation;

/// Convert a Markdown parser position to one-based roadmap coordinates.
pub(crate) const fn source_location(position: &Position) -> SourceLocation {
    SourceLocation {
        line: position.start.line,
        column: position.start.column,
    }
}

/// Map a byte range inside a text node to the source coordinates of its start.
///
/// Markdown normalizes soft line endings and strips the indentation of
/// continuation lines, so a text node's value is not a slice of the document.
/// When the document is known, the token is located inside the node's own
/// source span, which keeps a diagnostic on the token even on a wrapped line.
/// Without the document the value is folded from the node's start, which is
/// exact for a single-line node and for the first line of a wrapped one.
///
/// # Examples
///
/// ```text
/// // Source, parsed into one text node whose value is
/// // "Requires 1.1.2\nand 1.0.1.":
/// //
/// //   line 7: "  Requires 1.1.2"
/// //   line 8: "  and 1.0.1."
/// //
/// // Offset 19..24 is "1.0.1"; the source location is line 8, column 7.
/// ```
pub(crate) fn source_location_at(
    text: &Text,
    source: Option<&str>,
    range: Range<usize>,
) -> Option<SourceLocation> {
    let position = text.position.as_ref()?;
    if let Some(document) = source
        && let Some(location) = locate_in_source(text, document, &range)
    {
        return Some(location);
    }
    fold_from_start(&position.start, &text.value, range.start)
}

/// Locate a token inside the node's own span of the original document.
fn locate_in_source(text: &Text, source: &str, range: &Range<usize>) -> Option<SourceLocation> {
    let position = text.position.as_ref()?;
    let token = text.value.get(range.clone())?;
    if token.is_empty() {
        return None;
    }
    let span = position.start.offset..position.end.offset;
    let node_source = source.get(span.clone())?;
    // Repeated anchors are common, so match the same occurrence the value has.
    let occurrence = text.value.get(..range.start)?.matches(token).count();
    let token_offset = node_source.match_indices(token).nth(occurrence)?.0;
    let prefix = source.get(span.start..span.start + token_offset)?;
    fold_from_start(&position.start, prefix, prefix.len())
}

/// Advance a start point over a prefix, counting line breaks and characters.
fn fold_from_start(start: &Point, value: &str, offset: usize) -> Option<SourceLocation> {
    let prefix = value.get(..offset)?;
    let (line, column) =
        prefix
            .chars()
            .fold((start.line, start.column), |(line, column), character| {
                if character == '\n' {
                    (line + 1, 1)
                } else {
                    (line, column + 1)
                }
            });
    Some(SourceLocation { line, column })
}

#[cfg(test)]
mod tests {
    //! Generated checks that a wrapped token reports its source coordinates.

    use markdown::{
        ParseOptions,
        mdast::{Node, Text},
        to_mdast,
    };
    use proptest::prelude::*;

    use super::source_location_at;

    /// Return the single text node of the document's wrapped paragraph.
    fn wrapped_text(source: &str) -> Text {
        let Node::Root(root) =
            to_mdast(source, &ParseOptions::gfm()).expect("fixture should parse")
        else {
            panic!("fixture should parse to a root");
        };
        let Some(Node::Paragraph(paragraph)) = root.children.get(1) else {
            panic!("the second block should be the wrapped paragraph");
        };
        let Some(Node::Text(text)) = paragraph.children.first() else {
            panic!("the wrapped paragraph should start with text");
        };
        text.clone()
    }

    /// Return the offset of the second occurrence of `token`, when present.
    fn second_occurrence(value: &str, token: &str) -> Option<usize> {
        let first = value.find(token)?;
        let rest = first + token.len();
        value.get(rest..)?.find(token).map(|offset| rest + offset)
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 32,
            .. ProptestConfig::default()
        })]

        /// The continuation line's indentation is stripped from the text value,
        /// so the value alone would place the token too early. The reported
        /// column must come from the source line instead.
        #[test]
        fn wrapped_token_reports_its_source_column(
            indent in 0usize..4,
            token in prop_oneof![Just("1.1.1"), Just("12.3.4"), Just("2.0.7")],
        ) {
            let padding = " ".repeat(indent);
            let source = format!("Intro.\n\nRequires {token}\n{padding}and {token}.\n");
            let text = wrapped_text(&source);
            let start = second_occurrence(&text.value, token)
                .expect("the generated clause should repeat its token");
            let location = source_location_at(&text, Some(&source), start..start + token.len())
                .expect("a token inside the node's span should have a location");
            prop_assert_eq!(location.line, 4);
            prop_assert_eq!(location.column, indent + 5);
        }
    }
}
