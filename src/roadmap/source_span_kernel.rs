//! Verifiable source-span scanning and copying for preserved Markdown.
//!
//! A preserved task or list item is copied from the original source rather
//! than re-rendered. Two helpers make that copy: the scan that widens a span to
//! the start of its line, and the copy that returns the owned text. Both used
//! to be written directly in `super::source_preservation`, where a property
//! test could sample them and a proof could not reach them.
//!
//! This module owns the executable implementation of both. The bodies are
//! written once in the shared macro files beside this module; this file expands
//! the ordinary Rust form, and `verus/lib.rs` expands the verified form from the
//! same text. The domain owns the kernel and the proof reaches into `src/` for
//! it, so production names no proof path.
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
// same files and invokes the verified modes, so the two builds expand one text.
// The type declarations are invoked by `source_span_kernel!`, so that kernel
// must expand before the task-item kernel.
include!("verified_function.macro.rs");
include!("source_span_types.macro.rs");
include!("source_span_kernel.macro.rs");
include!("task_item_span_kernel.macro.rs");

source_span_kernel!(exec);
task_item_span_kernel!(exec);

#[cfg(test)]
#[path = "source_span_kernel_tests.rs"]
mod tests;
