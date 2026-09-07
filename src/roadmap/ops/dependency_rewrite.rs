//! Rendering helpers for individual rewritten dependency references.

use super::super::model::RenumberPlan;
use crate::roadmap::{RoadmapAnchor, model::SourceId};

/// Shared dependency-rewrite inputs for one text value.
pub(super) struct DependencyRewriteContext<'a> {
    pub(super) source: SourceId,
    pub(super) plan: &'a RenumberPlan,
}

/// Mutable dependency-rewrite output accumulated while scanning a text value.
pub(super) struct DependencyRewriteState<'a> {
    pub(super) result: &'a mut String,
    pub(super) rewrite_count: &'a mut u64,
    pub(super) unresolved: &'a mut Vec<RoadmapAnchor>,
}

/// Append a rewritten dependency reference, retaining identical mappings verbatim.
pub(super) fn rewrite_dependency_reference(
    context: &DependencyRewriteContext<'_>,
    anchor: RoadmapAnchor,
    candidate: &str,
    state: &mut DependencyRewriteState<'_>,
) {
    let Some(rewritten) = context
        .plan
        .resolve(context.source, anchor)
        .or_else(|| context.plan.resolve_unique(anchor))
        .map(|mapped| mapped.to_string())
    else {
        state.unresolved.push(anchor);
        state.result.push_str(candidate);
        return;
    };

    *state.rewrite_count += u64::from(rewritten != candidate);
    state.result.push_str(&rewritten);
}
