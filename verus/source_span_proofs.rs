// Source-span specifications, lemmas and the verified expansion.
//
// This file is included at the crate root by `verus/lib.rs`, which keeps the
// proof entry point below the repository's 400-line limit without changing the
// scope the specifications are declared in. The executable body remains
// domain-owned in `src/roadmap/`; only the proof-side definitions live here.

// ---------------------------------------------------------------------------
// Kernel: source-span scanning and copying
// ---------------------------------------------------------------------------
//
// Production callers: `original_position_source`, `task_item_sources` and the
// line-start adapter in `src/roadmap/source_preservation.rs`, through the total
// entry points in `src/roadmap/source_span_kernel.rs`.
//
// The kernel answers three questions in byte offsets: where the line
// containing an offset starts, whether a requested span names a valid
// substring, and how to copy that substring. CR and LF are recognized
// separately, so a CR-only boundary is preserved as faithfully as an LF or
// CRLF one.

verus! {

broadcast use {vstd::slice::group_slice_axioms, vstd::utf8::group_utf8_lib};

/// The two bytes Markdown treats as line terminators in this scan.
pub open spec fn is_line_terminator(byte: u8) -> bool {
    byte == b'\n' || byte == b'\r'
}

/// The start of the line containing `offset`, as a byte offset.
///
/// This is the recursive specification the executable backward scan is proved
/// equal to. It walks back from `offset` to the first preceding CR or LF and
/// returns the position after it, or zero when no terminator precedes it.
pub open spec fn spec_line_start(bytes: Seq<u8>, offset: int) -> int
    decreases offset,
{
    if offset <= 0 {
        0
    } else if is_line_terminator(bytes[offset - 1]) {
        offset
    } else {
        spec_line_start(bytes, offset - 1)
    }
}

/// True when `offset` can name a byte position in `bytes`.
pub open spec fn spec_offset_within(bytes: Seq<u8>, offset: int) -> bool {
    0 <= offset <= bytes.len() && is_char_boundary(bytes, offset)
}

/// The first failing span check, or `None` when the span is acceptable.
///
/// The order matches the executable validation: start range, start boundary,
/// end range, end boundary, ordering, and finally the widened line start's
/// boundary. The last check is redundant for any `&str`, whose bytes are valid
/// UTF-8 and whose line starts after a CR or LF are therefore boundaries, but
/// stating it keeps the successful span's boundary facts independent of a
/// deep UTF-8 lemma.
pub open spec fn spec_span_error(bytes: Seq<u8>, start: int, end: int) -> Option<SpanError> {
    if start > bytes.len() {
        Some(SpanError::StartOutOfBounds)
    } else if !is_char_boundary(bytes, start) {
        Some(SpanError::StartNotCharBoundary)
    } else if end > bytes.len() {
        Some(SpanError::EndOutOfBounds)
    } else if !is_char_boundary(bytes, end) {
        Some(SpanError::EndNotCharBoundary)
    } else if start > end {
        Some(SpanError::Reversed)
    } else if !is_char_boundary(bytes, spec_line_start(bytes, start)) {
        Some(SpanError::StartNotCharBoundary)
    } else {
        None
    }
}

/// Compare an actually produced optional span with its specification.
pub open spec fn spec_span_matches(actual: Option<ByteSpan>, expected: Option<(int, int)>) -> bool {
    match actual {
        Some(span) => expected == Some((span.start as int, span.end as int)),
        None => expected.is_none(),
    }
}

/// The successful result of `checked_line_start`.
pub open spec fn spec_checked_line_start(bytes: Seq<u8>, offset: int) -> Option<int> {
    if spec_offset_within(bytes, offset) {
        Some(spec_line_start(bytes, offset))
    } else {
        None
    }
}

/// The successful result of `checked_line_span`.
pub open spec fn spec_checked_line_span(
    bytes: Seq<u8>,
    start: int,
    end: int,
) -> Option<(int, int)> {
    if spec_span_error(bytes, start, end).is_none() {
        Some((spec_line_start(bytes, start), end))
    } else {
        None
    }
}

/// The end offset a task item's extraction uses.
pub open spec fn spec_task_item_end(
    starts: Seq<Option<usize>>,
    list_end: Option<usize>,
    bytes: Seq<u8>,
    index: int,
) -> Option<int> {
    let from_next = if index + 1 < starts.len() {
        match starts[index + 1] {
            Some(next) => spec_checked_line_start(bytes, next as int),
            None => None,
        }
    } else {
        None
    };
    match from_next {
        Some(value) => Some(value),
        None => match list_end {
            Some(value) => Some(value as int),
            None => None,
        },
    }
}

/// The span each task item extracts, or `None` when one cannot be formed.
pub open spec fn spec_task_item_span(
    starts: Seq<Option<usize>>,
    list_end: Option<usize>,
    bytes: Seq<u8>,
    index: int,
) -> Option<(int, int)> {
    if 0 <= index < starts.len() {
        match starts[index] {
            None => None,
            Some(start) => match spec_task_item_end(starts, list_end, bytes, index) {
                None => None,
                Some(end) => spec_checked_line_span(bytes, start as int, end),
            },
        }
    } else {
        None
    }
}

/// True when every produced span matches its specification.
pub open spec fn spec_all_spans_match(
    spans: Seq<Option<ByteSpan>>,
    starts: Seq<Option<usize>>,
    list_end: Option<usize>,
    bytes: Seq<u8>,
) -> bool {
    forall|position: int| 0 <= position < spans.len() ==> spec_span_matches(
        spans[position],
        spec_task_item_span(starts, list_end, bytes, position),
    )
}

/// Carry the per-item property across an append.
proof fn lemma_spans_match_push(
    spans: Seq<Option<ByteSpan>>,
    starts: Seq<Option<usize>>,
    list_end: Option<usize>,
    bytes: Seq<u8>,
    appended: Option<ByteSpan>,
)
    requires
        spec_all_spans_match(spans, starts, list_end, bytes),
        spec_span_matches(
            appended,
            spec_task_item_span(starts, list_end, bytes, spans.len() as int),
        ),
    ensures
        spec_all_spans_match(spans.push(appended), starts, list_end, bytes),
{
    assert forall|position: int|
        0 <= position < spans.push(appended).len() implies spec_span_matches(
        #[trigger] spans.push(appended)[position],
        spec_task_item_span(starts, list_end, bytes, position),
    ) by {
        if position < spans.len() {
            assert(spans.push(appended)[position] == spans[position]);
        }
    }
}

/// Relate the scan's stopping point to the recursive specification.
proof fn lemma_spec_line_start(bytes: Seq<u8>, offset: int, index: int)
    requires
        0 <= index <= offset <= bytes.len(),
        forall|i: int| index <= i < offset ==> !is_line_terminator(bytes[i]),
        index == 0 || is_line_terminator(bytes[index - 1]),
    ensures
        spec_line_start(bytes, offset) == index,
    decreases offset - index,
{
    if offset == index {
        if index > 0 {
            assert(is_line_terminator(bytes[offset - 1]));
        }
    } else {
        assert(!is_line_terminator(bytes[offset - 1]));
        lemma_spec_line_start(bytes, offset - 1, index);
    }
}

/// The line start is a byte position at or before the supplied offset.
proof fn lemma_line_start_bounds(bytes: Seq<u8>, offset: int)
    requires
        0 <= offset <= bytes.len(),
    ensures
        0 <= spec_line_start(bytes, offset) <= offset,
    decreases offset,
{
    if offset > 0 && !is_line_terminator(bytes[offset - 1]) {
        lemma_line_start_bounds(bytes, offset - 1);
    }
}

/// A byte after a one-byte scalar is a character boundary.
///
/// Valid UTF-8 decodes a one-byte scalar in isolation, so the next position
/// starts a fresh scalar (or is the end of the sequence). This is the fact the
/// widened-start boundary check needs; it is proved by peeling the first scalar
/// until the position is reached.
proof fn lemma_after_ascii_is_boundary(bytes: Seq<u8>, offset: int)
    requires
        valid_utf8(bytes),
        0 < offset <= bytes.len(),
        is_leading_byte_width_1(bytes[offset - 1]),
    ensures
        is_char_boundary(bytes, offset),
    decreases bytes.len(),
{
    assert(valid_first_scalar(bytes));
    let first = length_of_first_scalar(bytes);
    if offset <= first {
        assert(offset == first) by {
            assert(valid_leading_and_continuation_bytes_first_codepoint(bytes));
            if is_leading_byte_width_1(bytes[0]) {
                assert(first == 1);
            } else if is_leading_byte_width_2(bytes[0]) {
                assert(first == 2);
                assert(is_continuation_byte(bytes[1]));
            } else if is_leading_byte_width_3(bytes[0]) {
                assert(first == 3);
                assert(is_continuation_byte(bytes[1]));
                assert(is_continuation_byte(bytes[2]));
            } else {
                assert(is_leading_byte_width_4(bytes[0]));
                assert(first == 4);
                assert(is_continuation_byte(bytes[1]));
                assert(is_continuation_byte(bytes[2]));
                assert(is_continuation_byte(bytes[3]));
            }
        };
        reveal_with_fuel(is_char_boundary, 2);
    } else {
        let rest = pop_first_scalar(bytes);
        assert(valid_utf8(rest));
        assert(rest[offset - 1 - first] == bytes[offset - 1]);
        lemma_after_ascii_is_boundary(rest, offset - first);
        reveal_with_fuel(is_char_boundary, 2);
    }
}

/// The line start of any valid source is a character boundary.
proof fn lemma_line_start_is_boundary(bytes: Seq<u8>, offset: int)
    requires
        valid_utf8(bytes),
        0 <= offset <= bytes.len(),
    ensures
        is_char_boundary(bytes, spec_line_start(bytes, offset)),
    decreases offset,
{
    if offset <= 0 {
        is_char_boundary_start_end_of_seq(bytes);
    } else if is_line_terminator(bytes[offset - 1]) {
        assert(is_leading_byte_width_1(bytes[offset - 1]));
        lemma_after_ascii_is_boundary(bytes, offset);
    } else {
        lemma_line_start_is_boundary(bytes, offset - 1);
    }
}

/// A valid in-range ordered boundary pair is accepted.
///
/// This is the completeness half of the span contract. `spec_span_error`'s
/// final widened-boundary check is a defensive executable guard, and this
/// theorem proves it can never reject a span that the earlier checks accepted
/// in a valid `&str`. Without it the specification would describe only which
/// error is reported, not that every legitimate span succeeds.
proof fn lemma_span_error_complete(bytes: Seq<u8>, start: int, end: int)
    requires
        valid_utf8(bytes),
        0 <= start <= end <= bytes.len(),
        is_char_boundary(bytes, start),
        is_char_boundary(bytes, end),
    ensures
        spec_span_error(bytes, start, end).is_none(),
{
    lemma_line_start_is_boundary(bytes, start);
    lemma_line_start_bounds(bytes, start);
    assert(!(start > bytes.len()));
    assert(is_char_boundary(bytes, start));
    assert(!(end > bytes.len()));
    assert(is_char_boundary(bytes, end));
    assert(!(start > end));
    assert(is_char_boundary(bytes, spec_line_start(bytes, start)));
}

/// A character boundary inside a prefix is a boundary of that prefix.
///
/// This is what lets the copy split at the span start on the prefix returned
/// by the first split, using the boundary the validator established for the
/// whole source. The two boundaries agree because, away from the shared end,
/// both are characterized by the same leading byte.
proof fn lemma_boundary_prefix(bytes: Seq<u8>, prefix_len: int, index: int)
    requires
        valid_utf8(bytes),
        is_char_boundary(bytes, prefix_len),
        0 <= index <= prefix_len <= bytes.len(),
    ensures
        is_char_boundary(bytes.subrange(0, prefix_len), index) == is_char_boundary(bytes, index),
{
    let prefix = bytes.subrange(0, prefix_len);
    valid_utf8_split(bytes, prefix_len);
    if index == 0 {
        assert(is_char_boundary(prefix, 0));
        assert(is_char_boundary(bytes, 0));
    } else if index == prefix_len {
        is_char_boundary_start_end_of_seq(prefix);
    } else {
        lemma_seq_subrange_index(bytes, 0, prefix_len, index);
        is_char_boundary_iff_is_leading_byte(bytes, index);
        is_char_boundary_iff_is_leading_byte(prefix, index);
    }
}

} // verus!
