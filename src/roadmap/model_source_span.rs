//! Byte offsets locating one checklist item in its original source.

/// Byte offsets locating one checklist item's marker and opening paragraph.
///
/// The completion command edits the original document text rather than the
/// rendered model, so it needs the exact bytes of the checkbox marker and the
/// title. The Markdown parser reports offsets in bytes while columns count
/// characters, so these spans stay byte offsets; `title_start` is the start of
/// the numbered prefix, which is also the start of the opening paragraph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ItemSourceSpan {
    /// Offset of the list item's bullet marker.
    pub(crate) item_start: usize,
    /// Offset of the item's numbered title text, after the checkbox.
    pub(crate) title_start: usize,
    /// Offset just past the item's opening paragraph.
    pub(crate) paragraph_end: usize,
}

impl ItemSourceSpan {
    /// Return whether the span is ordered and lands on character boundaries.
    #[must_use]
    pub(crate) const fn is_valid_in(self, source: &str) -> bool {
        self.item_start <= self.title_start
            && self.title_start <= self.paragraph_end
            && self.paragraph_end <= source.len()
            && source.is_char_boundary(self.item_start)
            && source.is_char_boundary(self.title_start)
            && source.is_char_boundary(self.paragraph_end)
    }
}
