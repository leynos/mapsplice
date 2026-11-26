//! Verifiable source-span scanning and copying for preserved Markdown.
//!
//! A preserved task or list item is copied from the original source rather
//! than re-rendered. Two helpers make that copy: the scan that widens a span to
//! the start of its line, and the copy that returns the owned text. Both used
//! to be written directly in `super::source_preservation`, where a property
//! test could sample them and a proof could not reach them.
//!
//! This module owns the executable implementation of both. The body is written
//! once in `source_span_kernel.macro.rs`; this file expands the ordinary Rust
//! form, and `verus/lib.rs` expands the verified form from the same text. The
//! domain owns the kernel and the proof reaches into `src/` for it, so
//! production names no proof path.
//!
//! Byte offsets are the unit of the kernel. A `ByteSpan` is a half-open byte
//! range, never a scalar index or a display column. The scanner recognizes CR
//! and LF separately, so a CR-only line boundary is preserved as faithfully as
//! an LF or CRLF one, and every returned boundary is checked against Rust's
//! UTF-8 character boundaries before a span is copied.
//!
//! Only total entry points leave the module. `line_start_offset` and
//! `copy_span` carry caller-established preconditions and stay private;
//! `checked_line_start`, `checked_line_span`, `extract_span`,
//! `extract_line_span` and `task_item_spans` are the public surface, and
//! each validates its inputs before reaching a precondition-bearing callee.

// The macro definitions live beside this module. `verus/lib.rs` includes the
// same files and invokes the verified mode, so the two builds expand one text.
include!("verified_function.macro.rs");
include!("source_span_kernel.macro.rs");

source_span_kernel!(exec);
#[cfg(test)]
mod tests {
    //! Unit and property coverage for the source-span kernel.

    use proptest::prelude::*;

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

    #[test]
    fn crlf_lf_and_cr_boundaries_are_recognized() {
        assert_eq!(
            checked_line_start("a\r\nb", 4),
            Some(3),
            "a CRLF line starts after the LF"
        );
        assert_eq!(
            checked_line_start("a\rb", 3),
            Some(2),
            "a CR-only line starts after the CR"
        );
        assert_eq!(
            checked_line_start("a\nb", 3),
            Some(2),
            "an LF-only line starts after the LF"
        );
        assert_eq!(
            checked_line_start("a\r\nb", 2),
            Some(2),
            "an offset between CR and LF starts a new line"
        );
    }

    #[test]
    fn extraction_widens_to_the_line_start() {
        assert_eq!(
            extract_line_span("  - item\n", 4, 8),
            Ok("  - item".to_owned()),
            "extraction widens the start to the line and keeps exact bytes"
        );
        assert_eq!(
            extract_line_span("a\r\nb", 3, 4),
            Ok("b".to_owned()),
            "extraction after a CRLF starts at the following line"
        );
        assert_eq!(
            extract_line_span("a\rb", 2, 3),
            Ok("b".to_owned()),
            "extraction after a CR starts at the following line"
        );
        assert_eq!(
            extract_line_span("a\r\nb", 4, 4),
            Ok("b".to_owned()),
            "an end offset at the source end yields the final line"
        );
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

    #[test]
    fn every_error_variant_is_reachable() {
        assert_eq!(
            checked_line_span("abc", 4, 4),
            Err(SpanError::StartOutOfBounds),
            "a start past the source is rejected first"
        );
        assert_eq!(
            checked_line_span("é", 1, 2),
            Err(SpanError::StartNotCharBoundary),
            "a start inside a scalar is rejected second"
        );
        assert_eq!(
            checked_line_span("abc", 1, 4),
            Err(SpanError::EndOutOfBounds),
            "an end past the source is rejected third"
        );
        assert_eq!(
            checked_line_span("aé", 0, 2),
            Err(SpanError::EndNotCharBoundary),
            "an end inside a scalar is rejected fourth"
        );
        assert_eq!(
            checked_line_span("abc", 2, 1),
            Err(SpanError::Reversed),
            "a reversed span is rejected last"
        );
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
    fn task_item_spans_handle_absent_positions() {
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

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            .. ProptestConfig::default()
        })]

        /// The scan agrees with `rfind` at every offset.
        #[test]
        fn checked_line_start_matches_the_rfind_oracle(source in source_text()) {
            for offset in 0..=source.len() {
                let actual = checked_line_start(&source, offset);
                let expected = line_start_oracle(&source, offset);
                prop_assert_eq!(
                    actual,
                    expected,
                    "offset {} of {:?} must match the rfind oracle",
                    offset,
                    source,
                );
            }
        }

        /// Extraction agrees with the widened `str::get` oracle for every
        /// ordered pair of character-boundary offsets.
        #[test]
        fn extraction_matches_the_widened_get_oracle(source in source_text()) {
            for start in 0..=source.len() {
                if !source.is_char_boundary(start) {
                    continue;
                }
                for end in start..=source.len() {
                    if !source.is_char_boundary(end) {
                        continue;
                    }
                    let actual = extract_line_span(&source, start, end);
                    let expected = extract_oracle(&source, start, end)
                        .ok_or_else(|| TestCaseError::fail("oracle rejected a boundary span"))?;
                    prop_assert_eq!(
                        actual,
                        Ok(expected),
                        "span {}..{} of {:?} must match the widened get oracle",
                        start,
                        end,
                        source,
                    );
                }
            }
        }

        /// Non-boundary endpoints are rejected before any copy.
        #[test]
        fn non_boundary_endpoints_are_rejected(source in source_text()) {
            for offset in 0..=source.len() {
                if source.is_char_boundary(offset) {
                    continue;
                }
                let actual = checked_line_span(&source, offset, offset);
                prop_assert!(
                    actual.is_err(),
                    "offset {} of {:?} is not a character boundary",
                    offset,
                    source,
                );
            }
        }

        /// The byte-level check agrees with `str::is_char_boundary`.
        #[test]
        fn checked_line_start_accepts_exactly_the_char_boundaries(source in source_text()) {
            for offset in 0..=source.len() {
                let actual = checked_line_start(&source, offset).is_some();
                prop_assert_eq!(
                    actual,
                    source.is_char_boundary(offset),
                    "offset {} of {:?} must agree with str::is_char_boundary",
                    offset,
                    source,
                );
            }
        }
    }
}
