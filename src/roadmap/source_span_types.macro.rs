// The mode-selected span and error types for the source-span kernels.
//
// `source_span_kernel!` invokes this macro once per build mode, so the product
// and the proof declare the same fields and variants. The verified arm wraps
// the declarations in `verus!`; the executable arm emits ordinary Rust.
//
// Byte offsets are the unit throughout. `ByteSpan` is a half-open byte range,
// never a scalar index or a display column.

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

