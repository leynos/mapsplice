//! Local outcomes produced while mutating and rendering preserved task source.

/// Closed mutation causes for preserved-source invalidation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PreservationInvalidationReason {
    /// The item's rendered number changed.
    Renumber,
    /// The item's dependency text changed.
    DependencyRewrite,
    /// A structural sub-task mutation changed a parent task.
    ChildMutation,
}

/// Closed formatter-stability causes for canonical item rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CanonicalFallbackReason {
    /// An ordered or nested list marker is formatter-unstable.
    UnstableListMarker,
    /// A code-fence delimiter is formatter-unstable.
    UnstableCodeFence,
}

/// The local outcome of rendering one task or sub-task with preserved source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PreservationRenderOutcome {
    /// The formatter-stable original source was emitted verbatim.
    PreservedSource,
    /// The original source required canonical rendering for the given cause.
    CanonicalFallback(CanonicalFallbackReason),
}

/// Bounded preservation outcomes collected during one command execution.
///
/// The report keeps rendering and roadmap mutation independent from
/// process-wide observability. The application boundary consumes it only after
/// the command has rendered and, for in-place requests, written successfully.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PreservationReport {
    preserved_source_renders: u64,
    invalidations_renumber: u64,
    invalidations_dependency_rewrite: u64,
    invalidations_child_mutation: u64,
    canonical_fallbacks_unstable_list_marker: u64,
    canonical_fallbacks_unstable_code_fence: u64,
}

impl PreservationReport {
    /// Record one source invalidation that actually removed preserved source.
    pub(crate) const fn record_invalidation(&mut self, reason: PreservationInvalidationReason) {
        match reason {
            PreservationInvalidationReason::Renumber => self.invalidations_renumber += 1,
            PreservationInvalidationReason::DependencyRewrite => {
                self.invalidations_dependency_rewrite += 1;
            }
            PreservationInvalidationReason::ChildMutation => {
                self.invalidations_child_mutation += 1;
            }
        }
    }

    /// Record the formatter decision made for one preserved task or sub-task.
    pub(crate) const fn record_render_outcome(&mut self, outcome: PreservationRenderOutcome) {
        match outcome {
            PreservationRenderOutcome::PreservedSource => self.preserved_source_renders += 1,
            PreservationRenderOutcome::CanonicalFallback(
                CanonicalFallbackReason::UnstableListMarker,
            ) => self.canonical_fallbacks_unstable_list_marker += 1,
            PreservationRenderOutcome::CanonicalFallback(
                CanonicalFallbackReason::UnstableCodeFence,
            ) => self.canonical_fallbacks_unstable_code_fence += 1,
        }
    }

    /// Combine outcomes produced by independent mutation and render phases.
    pub(crate) const fn extend(&mut self, other: Self) {
        self.preserved_source_renders += other.preserved_source_renders;
        self.invalidations_renumber += other.invalidations_renumber;
        self.invalidations_dependency_rewrite += other.invalidations_dependency_rewrite;
        self.invalidations_child_mutation += other.invalidations_child_mutation;
        self.canonical_fallbacks_unstable_list_marker +=
            other.canonical_fallbacks_unstable_list_marker;
        self.canonical_fallbacks_unstable_code_fence +=
            other.canonical_fallbacks_unstable_code_fence;
    }

    /// Return the count of formatter-stable preserved task or sub-task renders.
    pub(crate) const fn preserved_source_renders(self) -> u64 { self.preserved_source_renders }

    /// Return the count of invalidations caused by renumbering.
    pub(crate) const fn invalidations_renumber(self) -> u64 { self.invalidations_renumber }

    /// Return the count of invalidations caused by dependency rewriting.
    pub(crate) const fn invalidations_dependency_rewrite(self) -> u64 {
        self.invalidations_dependency_rewrite
    }

    /// Return the count of invalidations caused by sub-task mutation.
    pub(crate) const fn invalidations_child_mutation(self) -> u64 {
        self.invalidations_child_mutation
    }

    /// Return the count of canonical fallbacks caused by list markers.
    pub(crate) const fn canonical_fallbacks_unstable_list_marker(self) -> u64 {
        self.canonical_fallbacks_unstable_list_marker
    }

    /// Return the count of canonical fallbacks caused by code fences.
    pub(crate) const fn canonical_fallbacks_unstable_code_fence(self) -> u64 {
        self.canonical_fallbacks_unstable_code_fence
    }
}
