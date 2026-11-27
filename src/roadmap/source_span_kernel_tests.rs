//! Unit and property coverage for the source-span kernel.

use proptest::prelude::*;
use rstest::rstest;

use super::{
    ByteSpan,
    SpanError,
    checked_line_span,
    checked_line_start,
    extract_line_span,
    extract_span,
    task_item_spans,
};

/// One generated source: CR, LF, ASCII and multi-byte scalars.
fn source_text() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            Just('\r'),
            Just('\n'),
            Just('a'),
            Just(' '),
            Just('-'),
            Just('é'),
            Just('🙂'),
        ],
        0..24,
    )
    .prop_map(|characters| characters.into_iter().collect())
}

/// The line-start oracle used by the generated checks.
fn line_start_oracle(source: &str, offset: usize) -> Option<usize> {
    source
        .get(..offset)
        .map(|prefix| prefix.rfind(['\r', '\n']).map_or(0, |index| index + 1))
}

/// The extraction oracle used by the generated checks.
fn extract_oracle(source: &str, start: usize, end: usize) -> Option<String> {
    let prefix = source.get(..start)?;
    let widened = prefix.rfind(['\r', '\n']).map_or(0, |index| index + 1);
    source.get(widened..end).map(str::to_owned)
}

#[test]
fn empty_source_has_a_zero_line_start() {
    assert_eq!(
        checked_line_start("", 0),
        Some(0),
        "the empty source has one empty line starting at zero"
    );
    assert_eq!(
        extract_line_span("", 0, 0),
        Ok(String::new()),
        "the empty span of the empty source is an empty string"
    );
}

#[rstest]
#[case::crlf("a\r\nb", 4, Some(3), "a CRLF line starts after the LF")]
#[case::cr("a\rb", 3, Some(2), "a CR-only line starts after the CR")]
#[case::lf("a\nb", 3, Some(2), "an LF-only line starts after the LF")]
#[case::between_crlf("a\r\nb", 2, Some(2), "an offset between CR and LF starts a new line")]
#[case::usize_max("abc", usize::MAX, None, "an offset at usize::MAX is out of bounds")]
fn checked_line_start_recognizes_line_endings(
    #[case] source: &str,
    #[case] offset: usize,
    #[case] expected: Option<usize>,
    #[case] message: &str,
) {
    assert_eq!(checked_line_start(source, offset), expected, "{message}");
}

/// One extraction example.
struct ExtractionCase {
    /// The source text.
    source: &'static str,
    /// The requested start offset.
    start: usize,
    /// The requested end offset.
    end: usize,
    /// The expected owned text.
    expected: &'static str,
    /// The failure diagnostic.
    message: &'static str,
}

#[test]
fn extraction_widens_to_the_line_start() {
    const CASES: &[ExtractionCase] = &[
        ExtractionCase {
            source: "  - item\n",
            start: 4,
            end: 8,
            expected: "  - item",
            message: "extraction widens the start to the line and keeps exact bytes",
        },
        ExtractionCase {
            source: "a\r\nb",
            start: 3,
            end: 4,
            expected: "b",
            message: "extraction after a CRLF starts at the following line",
        },
        ExtractionCase {
            source: "a\rb",
            start: 2,
            end: 3,
            expected: "b",
            message: "extraction after a CR starts at the following line",
        },
        ExtractionCase {
            source: "a\r\nb",
            start: 4,
            end: 4,
            expected: "b",
            message: "an end offset at the source end yields the final line",
        },
    ];
    for case in CASES {
        assert_eq!(
            extract_line_span(case.source, case.start, case.end),
            Ok(case.expected.to_owned()),
            "{}",
            case.message
        );
    }
}

#[test]
fn unicode_offsets_are_bytes_not_scalars() {
    // "é" is two bytes; offset 1 is inside it.
    assert_eq!(
        checked_line_start("é\nb", 3),
        Some(3),
        "the line after a multi-byte scalar starts at its byte offset"
    );
    assert_eq!(
        checked_line_span("é\nb", 3, 4),
        Ok(ByteSpan { start: 3, end: 4 }),
        "a span after a multi-byte scalar is byte-addressed"
    );
    assert_eq!(
        extract_line_span("é\nb", 3, 4),
        Ok("b".to_owned()),
        "extraction returns the bytes after the multi-byte scalar"
    );
}

/// One rejected span example.
struct SpanCase {
    /// The source text.
    source: &'static str,
    /// The requested start offset.
    start: usize,
    /// The requested end offset.
    end: usize,
    /// The expected error.
    expected: SpanError,
    /// The failure diagnostic.
    message: &'static str,
}

#[test]
fn every_error_variant_is_reachable() {
    const CASES: &[SpanCase] = &[
        SpanCase {
            source: "abc",
            start: 4,
            end: 4,
            expected: SpanError::StartOutOfBounds,
            message: "a start past the source is rejected first",
        },
        SpanCase {
            source: "é",
            start: 1,
            end: 2,
            expected: SpanError::StartNotCharBoundary,
            message: "a start inside a scalar is rejected second",
        },
        SpanCase {
            source: "abc",
            start: 1,
            end: 4,
            expected: SpanError::EndOutOfBounds,
            message: "an end past the source is rejected third",
        },
        SpanCase {
            source: "aé",
            start: 0,
            end: 2,
            expected: SpanError::EndNotCharBoundary,
            message: "an end inside a scalar is rejected fourth",
        },
        SpanCase {
            source: "abc",
            start: 2,
            end: 1,
            expected: SpanError::Reversed,
            message: "a reversed span is rejected last",
        },
        SpanCase {
            source: "abc",
            start: usize::MAX,
            end: usize::MAX,
            expected: SpanError::StartOutOfBounds,
            message: "a start at usize::MAX is rejected before any arithmetic",
        },
        SpanCase {
            source: "abc",
            start: 0,
            end: usize::MAX,
            expected: SpanError::EndOutOfBounds,
            message: "an end at usize::MAX is rejected before any arithmetic",
        },
    ];
    for case in CASES {
        assert_eq!(
            checked_line_span(case.source, case.start, case.end),
            Err(case.expected),
            "{}",
            case.message
        );
    }
}

#[test]
fn validation_order_is_observable() {
    let source = "é";
    assert_eq!(
        checked_line_span(source, 5, 1),
        Err(SpanError::StartOutOfBounds),
        "the start range is checked before the end range and ordering"
    );
    assert_eq!(
        checked_line_span(source, 1, 9),
        Err(SpanError::StartNotCharBoundary),
        "the start boundary is checked before the end range"
    );
    assert_eq!(
        checked_line_span(source, 0, 1),
        Err(SpanError::EndNotCharBoundary),
        "the end boundary is checked before ordering"
    );
}

#[test]
fn opposite_unicode_endpoints_are_rejected() {
    // An offset inside a scalar can be past a smaller valid end.
    assert_eq!(
        checked_line_span("éa", 1, 0),
        Err(SpanError::StartNotCharBoundary),
        "the start boundary check precedes the reversed-span check"
    );
}

#[test]
fn extraction_rejects_invalid_and_reversed_spans() {
    assert_eq!(
        extract_line_span("abc", 4, 4),
        Err(SpanError::StartOutOfBounds),
        "an out-of-bounds start must not extract"
    );
    assert_eq!(
        extract_line_span("abc", 2, 1),
        Err(SpanError::Reversed),
        "a reversed span must not extract"
    );
    assert_eq!(
        extract_span("abc", ByteSpan { start: 2, end: 1 }),
        None,
        "the explicit-span entry point rejects a reversed span"
    );
}

#[test]
fn task_item_spans_use_the_next_line_start_or_the_list_end() {
    let source = "  - a\n  - b\n";
    let spans = task_item_spans(&[Some(2), Some(8)], Some(12), source);
    assert_eq!(
        spans,
        vec![
            Some(ByteSpan { start: 0, end: 6 }),
            Some(ByteSpan { start: 6, end: 12 }),
        ],
        "each item runs to the next item's line start, or the list end"
    );
    let extracted = spans
        .into_iter()
        .map(|candidate| candidate.and_then(|byte_span| extract_span(source, byte_span)))
        .collect::<Vec<_>>();
    assert_eq!(
        extracted,
        vec![Some("  - a\n".to_owned()), Some("  - b\n".to_owned())],
        "the extracted item sources are the original lines"
    );
}

#[test]
fn task_item_spans_fall_back_for_unpositioned_neighbours() {
    let source = "  - a\n  - b\n";
    assert_eq!(
        task_item_spans(&[Some(2), None], Some(12), source),
        vec![Some(ByteSpan { start: 0, end: 12 }), None],
        "an unpositioned next item falls back to the list end"
    );
    assert_eq!(
        task_item_spans(&[None, Some(8)], Some(12), source),
        vec![None, Some(ByteSpan { start: 6, end: 12 })],
        "an unpositioned item extracts nothing"
    );
}

#[test]
fn task_item_spans_handle_empty_and_unbounded_lists() {
    let source = "  - a\n  - b\n";
    assert_eq!(
        task_item_spans(&[], Some(0), source),
        Vec::new(),
        "an empty item list produces an empty result"
    );
    assert_eq!(
        task_item_spans(&[Some(2)], None, source),
        vec![None],
        "a final item with no list end has no span"
    );
}

#[test]
fn task_item_spans_fall_back_when_the_next_end_is_invalid() {
    let source = "  - a\n  - b\n";
    assert_eq!(
        task_item_spans(&[Some(2), Some(99)], Some(12), source),
        vec![Some(ByteSpan { start: 0, end: 12 }), None],
        "an invalid next start falls back to the list end"
    );
}

#[path = "source_span_kernel_property_tests.rs"]
mod property_tests;
