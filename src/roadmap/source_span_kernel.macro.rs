// The source-span kernel, shared verbatim between the proof and the product.
//
// Everything in this file is emitted by `source_span_kernel!`, once per build
// mode. `source_span_kernel.rs` invokes the `exec` mode; `verus/lib.rs`
// invokes the `verified` mode. The executable bodies are written once, here,
// and the mode token selects whether the ordinary compiler or Verus receives
// them; there is no second implementation to drift.
//
// `ByteSpan` measures UTF-8 *byte* offsets. Markdown's `Position` columns are
// byte columns in markdown 1.0.0, while `source_location.rs` derives scalar
// columns for its own reporting; this kernel is deliberately byte-based and
// never treats a byte offset as a display or scalar width.
//
// The `verified` arm declares the types `pub` because a `pub open spec fn` may
// not expose a `pub(crate)` datatype. Production re-exports only the total
// entry points, so the wider declaration does not widen the product's API.
//
// Every literal `fn` declaration is kept on one line of the invocation. This
// file is scanned textually by `scripts/check-verification-ledger.sh`, and a
// declaration that survives only as `fn $name` would not satisfy a ledger
// claim. The scan is textual only; review is what connects the claim to the
// body.
/// Emit the span and error types for one build mode.
macro_rules! byte_span_types {
    (exec) => {
        /// A half-open byte range in the roadmap source.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(crate) struct ByteSpan {
            /// The first byte of the span.
            pub(crate) start: usize,
            /// The first byte after the span.
            pub(crate) end: usize,
        }

        /// Why a requested span cannot name a substring of the source.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(crate) enum SpanError {
            /// The start offset is past the end of the source.
            StartOutOfBounds,
            /// The start offset is not on a UTF-8 character boundary.
            StartNotCharBoundary,
            /// The end offset is past the end of the source.
            EndOutOfBounds,
            /// The end offset is not on a UTF-8 character boundary.
            EndNotCharBoundary,
            /// The start offset lies after the end offset.
            Reversed,
        }
    };
    (verified) => {
        verus! {
            /// A half-open byte range in the roadmap source.
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub struct ByteSpan {
                /// The first byte of the span.
                pub start: usize,
                /// The first byte after the span.
                pub end: usize,
            }

            /// Why a requested span cannot name a substring of the source.
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub enum SpanError {
                /// The start offset is past the end of the source.
                StartOutOfBounds,
                /// The start offset is not on a UTF-8 character boundary.
                StartNotCharBoundary,
                /// The end offset is past the end of the source.
                EndOutOfBounds,
                /// The end offset is not on a UTF-8 character boundary.
                EndNotCharBoundary,
                /// The start offset lies after the end offset.
                Reversed,
            }
        }
    };
}

/// Emit the source-span kernel for one build mode.
macro_rules! source_span_kernel {
    ($mode:ident) => {
        byte_span_types!($mode);

        verified_function! {
            $mode
            /// Return the byte offset at which the line containing `offset`
            /// starts.
            #[expect(
                clippy::indexing_slicing,
                reason = "the guard proves index > 0 and the invariant proves index <= bytes.len()"
            )]
            #[must_use]
            fn line_start_offset(source: &str, offset: usize) -> usize;
            requires(offset <= source.spec_bytes().len());
            ensures(result => result as int == spec_line_start(source.spec_bytes(), offset as int));
            proof_first {
            }
            before {
                let bytes = source.as_bytes();
                let mut index = offset;
            }
            proof_before {
                assert(bytes@ == source.spec_bytes());
                assert(bytes.len() == bytes@.len());
            }
            while (index > 0 && !(bytes[index - 1] == b'\n' || bytes[index - 1] == b'\r'))
            invariant(
                index <= offset,
                index <= bytes.len(),
                forall|i: int| index <= i < offset as int ==> !is_line_terminator(bytes@[i]),
            )
            decreases(index) {
                index -= 1;
            }
            proof_loop {
            }
            loop_step {
            }
            proof_after {
                lemma_spec_line_start(bytes@, offset as int, index as int);
            }
            after { index }
        }

        verified_function! {
            $mode
            /// Return the start of the line containing `offset`, or `None`
            /// when `offset` is out of bounds or not a character boundary.
            #[must_use]
            pub(crate) fn checked_line_start(source: &str, offset: usize) -> Option<usize>;
            requires(true);
            ensures(result => match result {
                Some(value) => {
                    &&& spec_offset_within(source.spec_bytes(), offset as int)
                    &&& value as int == spec_line_start(source.spec_bytes(), offset as int)
                }
                None => !spec_offset_within(source.spec_bytes(), offset as int),
            });
            proof_first {
            }
            before {
                let bytes = source.as_bytes();
                if offset > bytes.len() {
                    return None;
                }
                if !source.is_char_boundary(offset) {
                    return None;
                }
            }
            proof_before {
                assert(bytes@ == source.spec_bytes());
                assert(bytes.len() == bytes@.len());
            }
            proof_after {
            }
            after { Some(line_start_offset(source, offset)) }
        }

        verified_function! {
            $mode
            /// Validate a requested span and widen its start to the start of
            /// its line.
            #[must_use]
            pub(crate) fn checked_line_span(
                source: &str,
                start: usize,
                end: usize,
            ) -> Result<ByteSpan, SpanError>;
            requires(true);
            ensures(result => match result {
                Ok(span) => {
                    &&& spec_span_error(source.spec_bytes(), start as int, end as int).is_none()
                    &&& span.start as int == spec_line_start(source.spec_bytes(), start as int)
                    &&& span.end == end
                    &&& span.start <= span.end
                    &&& span.end <= source.spec_bytes().len()
                    &&& is_char_boundary(source.spec_bytes(), span.start as int)
                    &&& is_char_boundary(source.spec_bytes(), span.end as int)
                    &&& is_char_boundary(
                        source.spec_bytes().subrange(0, span.end as int),
                        span.start as int,
                    )
                }
                Err(error) => spec_span_error(source.spec_bytes(), start as int, end as int)
                    == Some(error),
            });
            proof_first {
            }
            before {
                let bytes = source.as_bytes();
                if start > bytes.len() {
                    return Err(SpanError::StartOutOfBounds);
                }
                if !source.is_char_boundary(start) {
                    return Err(SpanError::StartNotCharBoundary);
                }
                if end > bytes.len() {
                    return Err(SpanError::EndOutOfBounds);
                }
                if !source.is_char_boundary(end) {
                    return Err(SpanError::EndNotCharBoundary);
                }
                if start > end {
                    return Err(SpanError::Reversed);
                }
                let widened_start = line_start_offset(source, start);
                if !source.is_char_boundary(widened_start) {
                    return Err(SpanError::StartNotCharBoundary);
                }
            }
            proof_before {
                assert(bytes@ == source.spec_bytes());
                assert(bytes.len() == bytes@.len());
            }
            proof_after {
                lemma_line_start_bounds(source.spec_bytes(), start as int);
                lemma_boundary_prefix(source.spec_bytes(), end as int, widened_start as int);
            }
            after { Ok(ByteSpan { start: widened_start, end }) }
        }

        verified_function! {
            $mode
            /// Copy the bytes of a validated span out of the source.
            fn copy_span(source: &str, span: ByteSpan) -> String;
            requires(
                span.start <= span.end,
                span.end <= source.spec_bytes().len(),
                is_char_boundary(source.spec_bytes(), span.end as int),
                is_char_boundary(
                    source.spec_bytes().subrange(0, span.end as int),
                    span.start as int,
                ),
            );
            ensures(result => encode_utf8(result@) == source.spec_bytes().subrange(
                span.start as int,
                span.end as int,
            ));
            proof_first {
            }
            before {
                let (prefix, _suffix) = source.split_at(span.end);
                let (_head, selected) = prefix.split_at(span.start);
                let copied = selected.to_owned();
            }
            proof_before {
            }
            proof_after {
                assert(prefix.spec_bytes() =~= source.spec_bytes().subrange(0, span.end as int));
                assert(selected.spec_bytes() =~= prefix.spec_bytes().subrange(
                    span.start as int,
                    span.end as int,
                ));
                assert(prefix.spec_bytes().len() == span.end as int);
                assert(copied@ == selected@);
                assert(encode_utf8(copied@) =~= selected.spec_bytes());
                lemma_seq_subrange_composition(
                    source.spec_bytes(),
                    0,
                    span.end as int,
                    span.start as int,
                    span.end as int,
                );
            }
            after { copied }
        }

        verified_function! {
            $mode
            /// Extract the source for a span that has already been validated.
            #[must_use]
            pub(crate) fn extract_span(source: &str, span: ByteSpan) -> Option<String>;
            requires(true);
            ensures(result => match result {
                Some(text) => {
                    &&& spec_span_error(source.spec_bytes(), span.start as int, span.end as int)
                        .is_none()
                    &&& encode_utf8(text@) == source.spec_bytes().subrange(
                        spec_line_start(source.spec_bytes(), span.start as int),
                        span.end as int,
                    )
                }
                None => spec_span_error(source.spec_bytes(), span.start as int, span.end as int)
                    .is_some(),
            });
            proof_first {
            }
            before {
                let Ok(validated) = checked_line_span(source, span.start, span.end) else {
                    return None;
                };
            }
            proof_before {
            }
            proof_after {
            }
            after { Some(copy_span(source, validated)) }
        }

        verified_function! {
            $mode
            /// Extract the source for the line-widened span `start..end`.
            #[must_use]
            pub(crate) fn extract_line_span(
                source: &str,
                start: usize,
                end: usize,
            ) -> Result<String, SpanError>;
            requires(true);
            ensures(result => match result {
                Ok(text) => {
                    &&& spec_span_error(source.spec_bytes(), start as int, end as int).is_none()
                    &&& encode_utf8(text@) == source.spec_bytes().subrange(
                        spec_line_start(source.spec_bytes(), start as int),
                        end as int,
                    )
                }
                Err(error) => spec_span_error(source.spec_bytes(), start as int, end as int)
                    == Some(error),
            });
            proof_first {
            }
            before {
                let validated = match checked_line_span(source, start, end) {
                    Ok(validated) => validated,
                    Err(error) => return Err(error),
                };
            }
            proof_before {
            }
            proof_after {
            }
            after { Ok(copy_span(source, validated)) }
        }

        verified_function! {
            $mode
            /// Resolve one item's validated span.
            #[expect(
                clippy::indexing_slicing,
                reason = "the caller proves index < starts.len(), and the next-item read is \
                          guarded by index + 1 < starts.len()"
            )]
            fn task_item_span(
                starts: &[Option<usize>],
                list_end: Option<usize>,
                source: &str,
                index: usize,
            ) -> Option<ByteSpan>;
            requires(index < starts.len());
            ensures(result =>
                spec_span_matches(
                    result,
                    spec_task_item_span(starts@, list_end, source.spec_bytes(), index as int),
                ),
            );
            proof_first {
                assert(starts.len() as int == starts@.len());
            }
            before {
                let item_start = starts[index];
                let next_start = if index + 1 < starts.len() { starts[index + 1] } else { None };
                let next_end = match next_start {
                    Some(next) => checked_line_start(source, next),
                    None => None,
                };
                let end = match next_end {
                    Some(value) => Some(value),
                    None => list_end,
                };
                let span = match item_start {
                    None => None,
                    Some(start) => match end {
                        None => None,
                        Some(end_offset) => match checked_line_span(source, start, end_offset) {
                            Ok(span) => Some(span),
                            Err(_) => None,
                        },
                    },
                };
            }
            proof_before {
                assert(item_start == spec_slice_index(starts, index as int));
                assert(spec_slice_index(starts, index as int) == starts@[index as int]);
                assert(next_start == if index as int + 1 < starts@.len() {
                    starts@[index as int + 1]
                } else {
                    None
                });
            }
            proof_after {
                assert(spec_task_item_end(starts@, list_end, source.spec_bytes(), index as int)
                    == match end {
                        Some(value) => Some(value as int),
                        None => match list_end {
                            Some(value) => Some(value as int),
                            None => None,
                        },
                    });
                assert(spec_task_item_span(starts@, list_end, source.spec_bytes(), index as int)
                    == match item_start {
                        None => None,
                        Some(start) => match end {
                            None => None,
                            Some(end_offset) => spec_checked_line_span(
                                source.spec_bytes(),
                                start as int,
                                end_offset as int,
                            ),
                        },
                    });
                assert(spec_span_matches(
                    span,
                    spec_task_item_span(starts@, list_end, source.spec_bytes(), index as int),
                ));
            }
            after { span }
        }

        verified_function! {
            $mode
            /// Map each item's optional start to its validated source span.
            #[must_use]
            pub(crate) fn task_item_spans(
                starts: &[Option<usize>],
                list_end: Option<usize>,
                source: &str,
            ) -> Vec<Option<ByteSpan>>;
            requires(true);
            ensures(result =>
                result@.len() == starts@.len(),
                spec_all_spans_match(result@, starts@, list_end, source.spec_bytes()),
            );
            proof_first {
            }
            before {
                let mut spans = Vec::with_capacity(starts.len());
                let mut index = 0;
            }
            proof_before {
            }
            while (index < starts.len())
            invariant(
                index <= starts@.len(),
                spans@.len() == index,
                spec_all_spans_match(spans@, starts@, list_end, source.spec_bytes()),
            )
            decreases(starts.len() - index) {
                // Capture the item under inspection before advancing, so the
                // proof slot below can name it. The invariant's post-state is
                // restored before the appended element is pushed.
                let position = index;
                index += 1;
                let span = task_item_span(starts, list_end, source, position);
            }
            proof_loop {
                assert(position < starts.len());
                assert(spans@.len() == position);
                lemma_spans_match_push(
                    spans@,
                    starts@,
                    list_end,
                    source.spec_bytes(),
                    span,
                );
            }
            loop_step {
                spans.push(span);
            }
            proof_after {
            }
            after { spans }
        }
    };
}