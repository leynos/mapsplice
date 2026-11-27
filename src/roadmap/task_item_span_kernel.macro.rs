// Task-item span resolution, shared verbatim between the proof and the product.
//
// This is the structured-data loop of the source-span kernel. It lives beside
// `source_span_kernel.macro.rs` rather than inside it so each file stays below
// the repository's 400-line limit. `source_span_kernel!` declares `ByteSpan`,
// `SpanError` and the validation helpers first; invoke it before
// `task_item_span_kernel!`.
//
// Each item resolves to the next item's checked line start, or to the list end
// when the next position is absent or invalid. An absent current start yields
// no span. The adapter owns the mdast conversion; this file sees only optional
// byte offsets.

/// Emit the task-item span kernel for one build mode.
macro_rules! task_item_span_kernel {
    ($mode:ident) => {
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
