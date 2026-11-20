//! Old-to-new anchor lookups used while renumbering a roadmap.
//!
//! The plan holds two target-local statements. `by_source` records where each
//! surviving item landed. `continuity` records a retired identity that an
//! in-place replacement continued. They are kept apart because the cross-source
//! fallback must never observe a continuity alias: an alias says "this target
//! item is still the item at this position", not "fragment numbering applies
//! here". See [`RenumberPlan::resolve_reference`] for why that distinction is a
//! correctness requirement.

use std::collections::BTreeMap;

use super::{
    RoadmapAnchor,
    model::{ItemIdentity, SourceId},
};

/// Old-to-new renumbering lookup.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RenumberPlan {
    /// Mapping by source document.
    by_source: BTreeMap<SourceId, BTreeMap<RoadmapAnchor, RoadmapAnchor>>,
    /// Identities an in-place replacement continued, by source document.
    ///
    /// These are kept apart from `by_source` so the cross-source lookup cannot
    /// observe them. An alias is a target-local statement — "this retired item
    /// is still the item at this position" — not a renumbering of surviving
    /// text, and letting a fragment reference fall through to it would grant
    /// the fragment numbering it never wrote.
    continuity: BTreeMap<SourceId, BTreeMap<RoadmapAnchor, RoadmapAnchor>>,
}

impl RenumberPlan {
    /// Resolve a rewritten anchor for the given source and original anchor.
    ///
    /// A surviving identity's renumbering wins. A retired identity that an
    /// in-place replacement continued resolves through its continuity alias.
    #[must_use]
    pub fn resolve(&self, source: SourceId, anchor: RoadmapAnchor) -> Option<RoadmapAnchor> {
        self.by_source
            .get(&source)
            .and_then(|mapping| mapping.get(&anchor).copied())
            .or_else(|| {
                self.continuity
                    .get(&source)
                    .and_then(|mapping| mapping.get(&anchor).copied())
            })
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
    /// [`select_resolution`]: super::ops::select_resolution
    #[must_use]
    pub fn resolve_reference(
        &self,
        source: SourceId,
        anchor: RoadmapAnchor,
    ) -> Option<RoadmapAnchor> {
        let local = self.resolve(source, anchor);
        let cross_unique = self.resolve_unique(anchor);
        super::ops::select_resolution(local, cross_unique, source == SourceId::Fragment)
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

    /// Record that a retired identity continues at `new`.
    ///
    /// A replacement containing exactly the addressed item, numbered as it was,
    /// edits that item in place: the item is retired from the tree and a
    /// fragment item takes its position and number. Its identity has not gone
    /// away, so references to it must keep resolving.
    ///
    /// Returns whether the alias was recorded. An identity with any existing
    /// mapping is left alone: a surviving item with the same source-local
    /// anchor genuinely owns that identity, and a retired one must not displace
    /// it. The alias also stays invisible to [`Self::resolve_unique`], so the
    /// fragment cross-source fallback is unchanged.
    pub(crate) fn record_continuity(&mut self, identity: ItemIdentity, new: RoadmapAnchor) -> bool {
        if self.resolve(identity.source, identity.anchor).is_some() {
            return false;
        }
        self.continuity
            .entry(identity.source)
            .or_default()
            .insert(identity.anchor, new);
        true
    }
}
