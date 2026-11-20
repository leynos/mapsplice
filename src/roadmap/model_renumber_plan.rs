//! Old-to-new anchor mappings recorded while a document is renumbered.

use std::collections::BTreeMap;

use super::{
    super::{RoadmapAnchor, ops::select_resolution},
    SourceId,
};

/// Old-to-new renumbering lookup.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RenumberPlan {
    /// Mapping by source document.
    by_source: BTreeMap<SourceId, BTreeMap<RoadmapAnchor, RoadmapAnchor>>,
}

impl RenumberPlan {
    /// Resolve a rewritten anchor for the given source and original anchor.
    #[must_use]
    pub fn resolve(&self, source: SourceId, anchor: RoadmapAnchor) -> Option<RoadmapAnchor> {
        self.by_source
            .get(&source)
            .and_then(|mapping| mapping.get(&anchor).copied())
    }

    /// Resolve a dependency reference written in `source` to its new anchor.
    ///
    /// The source-local mapping is consulted first. A unique cross-source
    /// mapping is then accepted only for fragment text.
    ///
    /// The asymmetry is a correctness requirement rather than an optimization.
    /// A fragment item's identity is its fragment-local spelling, because that
    /// spelling is all the fragment file carries; a target item's identity is
    /// the anchor it held before the operation. Those are the same kind of
    /// value, so if target text could fall back to the fragment's mapping, a
    /// replacement item spelled like the item it superseded would absorb a
    /// surviving consumer's clause — the clause would keep its old digits and
    /// silently resolve to a different item. A fragment clause, by contrast,
    /// genuinely is written in the fragment's own numbering when it arrives, so
    /// the fallback is exactly what makes it resolve.
    ///
    /// The choice between the two mappings is delegated to
    /// [`select_resolution`], a pure kernel that `verus/lib.rs` proves
    /// identity preservation over. Keeping the decision there — and only
    /// there — is what makes the rule a theorem rather than a comment: were
    /// the caller to withhold the cross-source value itself, the kernel's
    /// fragment guard would be dead in production and the proof would be
    /// about unreachable logic.
    ///
    /// [`select_resolution`]: super::super::ops::select_resolution
    #[must_use]
    pub fn resolve_reference(
        &self,
        source: SourceId,
        anchor: RoadmapAnchor,
    ) -> Option<RoadmapAnchor> {
        let local = self.resolve(source, anchor);
        let cross_unique = self.resolve_unique(anchor);
        select_resolution(local, cross_unique, source == SourceId::Fragment)
    }

    /// Resolve a unique mapping across all sources when local lookup is absent.
    #[must_use]
    pub fn resolve_unique(&self, anchor: RoadmapAnchor) -> Option<RoadmapAnchor> {
        let mut matches = self
            .by_source
            .values()
            .filter_map(|mapping| mapping.get(&anchor).copied());
        let first = matches.next()?;
        if matches.next().is_some() {
            None
        } else {
            Some(first)
        }
    }

    /// Record a source-local old-to-new anchor mapping.
    pub(crate) fn record_mapping(
        &mut self,
        source: SourceId,
        old: RoadmapAnchor,
        new: RoadmapAnchor,
    ) {
        self.by_source.entry(source).or_default().insert(old, new);
    }
}
