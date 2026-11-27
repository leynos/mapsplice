// The source-span kernel, shared verbatim between the proof and the product.
//
// Everything in this file is emitted by `source_span_kernel!`, once per build
// mode. `source_span_kernel.rs` invokes the `exec` mode; `verus/lib.rs`
// invokes the `verified` mode. The executable bodies are written once, here,
// and the mode token selects whether the ordinary compiler or Verus receives
// them; there is no second implementation to drift.
//
// The task-item helpers live in `task_item_span_kernel.macro.rs` because this
// file, together with the type declarations it invokes, must stay below the
// repository's 400-line limit. `source_span_kernel!` defines the types first,
// so invoke it before `task_item_span_kernel!`.
//
// Byte offsets are the unit. The scanner recognizes CR and LF separately, and
// every returned boundary is checked against Rust's UTF-8 character boundaries
// before a span is copied. Markdown's `Position` columns are byte columns in
// markdown 1.0.0, while `source_location.rs` derives scalar columns for its own
// reporting; this kernel is deliberately byte-based.
//
// Every literal `fn` declaration is kept on one line of the invocation. This
// file is scanned textually by `scripts/check-verification-ledger.sh`, and a
// declaration that survives only as `fn $name` would not satisfy a ledger
// claim. The scan is textual only; review is what connects the claim to the
// body.

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
                // `get` is the checked form of the slice read, so this function
                // needs no lint suppression. The `None` arms are unreachable
                // under the caller's index bound; they return the same absence
                // the kernel reports for a missing item.
                let item_start = match starts.get(index) {
                    Some(value) => *value,
                    None => return None,
                };
                let next_start = if index + 1 < starts.len() {
                    match starts.get(index + 1) {
                        Some(value) => *value,
                        None => return None,
                    }
                } else {
                    None
                };
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
    };
}
