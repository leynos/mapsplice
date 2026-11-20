# Keep a replaced roadmap item's identity when it stays in place

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: IN PROGRESS

Branch: `issue-228-replace-refuses-to-edit-a-task-that-other-tasks-depend-on`

Roadmap item: none. This plan closes GitHub issue
[#228](https://github.com/leynos/mapsplice/issues/228).

The user supplied this plan in the task packet and instructed its execution, so
the approval gate is satisfied by that standing instruction.

## Purpose / big picture

`mapsplice` edits a roadmap Markdown file in place while preserving every item
the edit does not touch. A roadmap item is a phase, step, task, or addendum
sub-task. Items can name other items in a `Requires` clause, and a `Requires`
clause is a dependency reference.

Today, replacing a task another task depends on fails with
`dangling_dependency`, even when the replacement is the same task with one word
changed. For example, this target:

```markdown
- [ ] 1.1.1. Draft the plan.
- [ ] 1.1.2. Review the plan. Requires 1.1.1.
```

and this fragment:

```markdown
- [x] 1.1.2. Review the plan carefully.
```

make `mapsplice replace target.md 1.1.2 fragment.md` fail, because `replace`
retires the identity `1.1.2` and the surviving consumer's clause no longer
resolves. The operation is refused even though no consumer's bytes need to
change.

After this change, a replacement consisting of exactly one item at the same
level, whose fragment-local number equals the addressed number, is an in-place
edit of the same item. The command succeeds, only the replaced item's line
changes, and every `Requires` clause stays byte-identical. Descendants of the
replaced item (steps under a phase, tasks and sub-tasks under a step, sub-tasks
under a task) keep their identities too, but only where a unique same-numbered
descendant exists on both the retired and replacement sides.

A changed number (`1.1.2` replaced by `1.1.5`) or a multi-item replacement
still retires every replaced identity, and a surviving reference to one of them
still fails with `dangling_dependency`. `delete` is unchanged.

The mechanism is an **operation-granted continuity alias**: the renumber plan
temporarily remembers that a retired identity continues at the new anchor of
the item that replaced it in the same structural position. Target text resolves
through that alias. Fragment text never does, and the cross-source fallback
keeps its existing rule: a target reference with no source-local mapping
resolves to nothing.

## Constraints

- Work only inside this worktree. Never edit the control checkout.
- Preserve the hexagonal boundary: `src/roadmap` is the domain and must not
  acquire filesystem, process, or CLI dependencies. `make check-domain-purity`
  must stay green.
- The rendered output must remain gate-clean (formatter-stable) and
  byte-identical outside the edited item; source preservation (design
  guarantees F1-F5) must not be weakened.
- Contiguous renumbering (C2) is unchanged: numbers are assigned by position at
  every level.
- The resolution kernel is unchanged. `RenumberPlan::resolve_reference`,
  `RenumberPlan::resolve_unique`, and `select_resolution` keep their current
  bodies and contracts; the alias is recorded through `RenumberPlan::resolve`
  so the proved kernel still makes the final decision.
- `insert`, `delete`, and `append` gain no continuity behaviour.
- `dangling_dependency` is preserved for every unpaired retired identity, and a
  failed operation must leave the caller's roadmap unchanged (staged clone
  discarded).
- Public API stays as it is; the new types are `pub(crate)` inside
  `src/roadmap`.
- Every module starts with `//!` documentation; production code contains no
  `expect`, `unwrap`, or panic; assertions carry custom diagnostics.
- Files stay under 400 lines. New logic goes in a new
  `src/roadmap/ops/continuity.rs` rather than growing `ops/mod.rs`.

## Tolerances (exception triggers)

Comparison base: `git diff --numstat` against commit `622b71d` (the branch
point), excluding this ExecPlan and generated artefacts. This repository has no
completed ExecPlan for a comparable dependency-identity change, so the budgets
below are a fresh survey of the affected callers, tests, fixtures, and docs.
Code counts production source, tests, and executable scripts. Configuration
counts manifests and workflow files (none expected). Documentation counts prose
outside `docs/execplans/`.

- Files: more than 40 non-ExecPlan files changed in total.
- Code: more than 900 net lines (additions minus deletions).
- Configuration: more than 0 net lines.
- Documentation: more than 350 net lines outside this ExecPlan.
- Interface: any public API change, or any change to the three resolution
  functions named in `Constraints`.
- Dependencies: any new dependency.
- Architecture: any need for the domain to read files or environment.
- Iterations: the same focused test still failing for the same reason after
  five fix attempts.
- Ambiguity: the descendant pairing rule admits two materially different
  readings.

Stop and report rather than working around a breached tolerance.

## Risks

- Risk: a continuity alias lets a target reference resolve onto a *different*
  item, recreating the alias bug the cross-source rule prevents. Severity:
  high. Likelihood: low. Mitigation: record an alias only for the same-level
  single replacement whose fragment anchor equals the addressed anchor, only
  when the retired identity has no existing source-local mapping, and pin the
  behaviour with target-text regressions plus the existing cross-source
  fallback test.
- Risk: fragment text starts resolving through target aliases, changing
  fragment-clause semantics. Severity: medium. Likelihood: low. Mitigation: the
  alias is recorded in the target's source-local map. Fragment-local lookup
  already contains every alias key for a paired replacement item, and
  `resolve_reference` consults the fragment map first; add a model test proving
  `resolve_unique` and fragment resolution ignore aliases.
- Risk: descendant pairing by anchor is ambiguous when the parser accepts
  duplicate anchors. Severity: medium. Likelihood: low. Mitigation: pair only
  anchors that occur exactly once on each side; add ambiguity tests that prove
  no alias is recorded.
- Risk: source preservation regresses because the replacement loses its
  preserved span cache. Severity: medium. Likelihood: medium. Mitigation:
  golden fixtures assert exact bytes and the fixture gate proves the output is
  formatter-clean.
- Risk: the property-suite oracle models replacement as retirement, so the
  suite would disagree with the new behaviour. Severity: high. Likelihood:
  high. Mitigation: extend the oracle with the one-item same-number variant and
  retain retirement for the two-item variant in the same milestone.

## Progress

- [x] (2026-11-20 00:00Z) Reconnaissance: located the replace pipeline,
      renumber plan, dependency rewriter, golden harness, property oracle, BDD
      harness, and the verification ledger.
- [x] (2026-11-20) ExecPlan drafted; user task packet authorizes execution.
- [ ] `EP-M1` Continuity aliases implemented in the domain with unit tests.
- [ ] `EP-M2` Golden, integration, property, and BDD coverage added.
- [ ] `EP-M3` User guide, design document, verification ledger, and PR closed.

## Surprises & discoveries

- Observation: the parser validates sub-task ordering but not duplicate
  anchors, so a document may hold two items with the same identity key.
  Evidence: `validate_sub_task_position` checks parent and ordinal only. Impact:
  `record_continuity` must refuse to overwrite an existing mapping, and
  descendant pairing must skip anchors that are not unique.
- Observation: `RenumberPlan::record_mapping` overwrites by design, because the
  renumber pass is authoritative for surviving items. Evidence:
  `src/roadmap/model.rs` uses `BTreeMap::insert`. Impact: continuity needs a
  separate insert-if-absent method rather than a second `record_mapping` call.
- Observation: the renumber pass only changes `number` fields; structure is
  fixed before it runs. Evidence: `renumber_document` in
  `src/roadmap/ops/rewrite.rs` iterates the staged tree in place. Impact: a
  structural position captured at splice time stays valid through renumbering,
  so the final anchor can be read from the staged tree afterwards.

## Decision log

- D1. Record continuity as an ordinary source-local mapping in
  `RenumberPlan::by_source`, using insert-if-absent, rather than a separate
  alias map. Rationale: `resolve_reference` and `resolve_unique` stay
  byte-identical, as the task packet requires, and the kernel still decides
  between local and cross-source resolution. A separate map would also work but
  would special-case `resolve` and make the alias invisible to the model tests
  that pin resolution. Date/Author: 2026-11-20, agent.
- D2. Identify replacement items by structural position, not by fragment
  identity. Rationale: fragment anchors are not validated for uniqueness and
  the fragment map is keyed by anchor, so a duplicate could collapse; positions
  are unambiguous and survive renumbering. Date/Author: 2026-11-20, agent.
- D3. Pair descendants by exact `RoadmapAnchor` equality, requiring uniqueness
  on both sides. Rationale: an anchor already encodes its level and every
  parent ordinal, so equality is the same-number rule at every level; skipping
  duplicates avoids granting an identity the document itself could not
  distinguish. Date/Author: 2026-11-20, agent.
- D4. Apply aliases after `renumber_document` and before
  `rewrite_dependencies`. Rationale: the final anchor must come from the staged
  tree, and the rewriter must see a complete plan. A failure in the rewriter
  discards the staged clone as before. Date/Author: 2026-11-20, agent.

## Outcomes & retrospective

Pending. At completion this section records the observed before/after CLI
transcripts, the gate results, and any lesson that would change the next
identity-preserving operation.

## Context and orientation

The pipeline lives in `src/roadmap/ops/mod.rs`. `apply_command_with_report`
clones the parsed roadmap, applies the operation (`append_fragment`,
`insert_fragment`, `delete_anchor`, or `replace_anchor`), calls
`renumber_document` from `src/roadmap/ops/rewrite.rs` to assign contiguous
numbers and produce a `RenumberPlan`, and then calls `rewrite_dependencies`,
which resolves every dependency reference through
`RenumberPlan::resolve_reference`. The staged clone is assigned back only on
success.

`replace_anchor` dispatches to `replace_phase`, `replace_step`, `replace_task`
(in `src/roadmap/ops/level_edit.rs`) or `replace_sub_task` (in
`src/roadmap/ops/sub_task.rs`). Each removes the addressed item and splices the
fragment's items in its place. The fragment items carry
`ItemIdentity { source: SourceId::Fragment, anchor }`, where the anchor is the
number written in the fragment file.

`RenumberPlan` in `src/roadmap/model.rs` holds
`BTreeMap<SourceId, BTreeMap<RoadmapAnchor, RoadmapAnchor>>` and exposes
`resolve`, `resolve_unique`, `resolve_reference`, and `record_mapping`.
`resolve_reference` delegates to `select_resolution` (in
`src/roadmap/ops/remap_kernel.rs`, proved in `verus/lib.rs`): a local mapping
wins; the cross-source mapping is used only for fragment text. A target
reference with no local mapping therefore fails, which is why the retired
anchor dangles today.

Terms used below. A **retired identity** is the `ItemIdentity` of a target item
removed by the operation. A **continuity alias** is a plan entry mapping that
retired identity's anchor to the final anchor of the item that took its place.
A **structural position** is the tuple of collection indices (phase, step,
task, sub-task) that locates an item in the staged document; it is not a number.

Testing infrastructure. Unit tests use `rstest` inside `src/`; integration
tests live in `tests/`; golden fixtures under `tests/fixtures/golden/<case>/`
are registered in `tests/roadmap_golden/contracts.rs` or
`tests/roadmap_golden/nested_requires.rs`; the property suite in
`tests/roadmap_dependency_properties.rs` uses an independent oracle under
`tests/support/`; BDD scenarios live in `tests/features/mapsplice.feature` with
steps in `tests/steps/cli_steps.rs`. Gates: `make check-fmt`, `make lint`,
`make test`, `make markdownlint`, `make nixie`, `make spelling`, `make all`, and
`make verus` / `make verus-selftest` when the prover is installed.

## Conformance basis

No Terms of Reference or technical-design revision exists for this repository;
the governing upstream artefacts are:

- GitHub issue #228, "replace refuses to edit a task that other tasks depend
  on", inspected at branch point `622b71d`.
- `docs/mapsplice-design.md`, guarantees C1 (operations) and C3 (dependency
  rewriting), section 7 (dependency-resolution rationale), and section 8
  (fixture requirements), inspected at `622b71d`.
- `docs/users-guide.md`, replace section, inspected at `622b71d`.
- `docs/verification.md` and `verus/lib.rs`, proof claims for
  `select_resolution`, inspected at `622b71d`.

Derived requirements and gaps:

- `REQ-continuity`: one-item, same-number replacement of an item is an in-place
  edit that keeps the item's identity.
- `REQ-descendants`: uniquely pairable same-numbered descendants keep their
  identities.
- `REQ-retirement`: changed-number and multi-item replacements retire every
  removed identity; unpaired references still fail.
- `REQ-scope`: `insert`, `delete`, and `append` are unchanged.
- `REQ-evidence`: regressions, golden fixtures, property cases, BDD scenarios,
  and documentation all state the same identity-retention rule.
- `GAP-228`: today no replacement keeps an identity, so `REQ-continuity` is
  false in production.

Trace:

```plaintext
GAP-228 -> REQ-continuity/REQ-descendants -> EP-M1 -> model and ops unit tests
REQ-retirement -> EP-M1/EP-M2 -> dangling-dependency regressions
REQ-scope -> EP-M2 -> property suite and existing delete/insert tests
REQ-evidence -> EP-M2/EP-M3 -> golden, BDD, users-guide, design, verification
```

## Verification plan

### Abstract model

An operation edits an identity tree. Every item has an identity
`(source, original anchor)` and a position. Applying an edit removes the
addressed subtree and splices the fragment's subtree at the same position. The
renumber pass assigns each surviving position its contiguous number and maps
its identity to that number. A dependency edge is a pair (consumer text, named
anchor). A target edge resolves iff the consumer source has a local mapping for
the named anchor; a fragment edge may also use a unique mapping from another
source.

Continuity adds one rule to the plan: for a retirement that is structurally an
in-place continuation, the retired identity maps to the new number of the item
occupying the addressed position, when the plan has no other mapping for it.

### Invariants and lemmas

- `INV-continuity`: a one-item same-number replacement records exactly one
  addressed-item alias, and every uniquely paired descendant alias points at
  the descendant's final anchor.
- `INV-retirement`: no alias is recorded when the fragment item's number
  differs, when more than one fragment item is supplied, or on any non-replace
  operation.
- `INV-priority`: an existing source-local mapping is never overwritten by an
  alias.
- `INV-resolution`: target text resolves through aliases; fragment text and
  `resolve_unique` observe the plan exactly as before.
- `INV-failure`: an unpaired identity produces `DanglingDependency` and the
  original roadmap is unchanged.
- `L-in-place`: replacing a task with an identical same-number task produces
  byte-identical output (modulo the intended edit), because the replacement
  keeps its number and every consumer keeps its clause.

### Axioms (trusted, not verified here)

- `AXIOM-renumber`: `renumber_document` numbers items by position and never
  reorders them. Evidence: the existing renumber tests and the property
  oracle's independent numbering.
- `AXIOM-parser`: parsing a fragment yields its literal anchors and identities.
  Evidence: existing parser tests, including sub-task ordering.
- `AXIOM-render`: rendering is a function of the model and preserved spans.
  Evidence: existing golden fixtures and formatter-boundary tests.
- `AXIOM-verus`: `select_resolution` satisfies the three theorems in
  `verus/lib.rs`. This change does not edit them.

### Methods per obligation

- Obligation `INV-continuity`, `INV-retirement`, `INV-priority`: parameterized
  `rstest` tables over all four levels plus ambiguity and no-overwrite cases.
  Artefact: `src/roadmap/model_tests.rs` and `src/roadmap/ops/continuity.rs`
  tests. Evidence: `cargo test` focused on roadmap modules. Non-vacuity: each
  table includes a success case and a rejection case, and asserts the resolved
  anchor, not just an error code.
- Obligation `INV-resolution`: model tests proving target aliases resolve,
  fragment lookup is unchanged, and `resolve_unique` ignores alias-only keys.
  Artefact: `src/roadmap/model_tests.rs`. Non-vacuity: include a control where
  no alias exists and resolution fails.
- Obligation `INV-failure` and `L-in-place`: CLI integration tests and golden
  fixtures asserting exit status, exact bytes, and the diagnostic for
  changed-number and multi-item replacements. Artefacts: `tests/roadmap_ops.rs`,
  `tests/roadmap_ops/nested_requires.rs`, and `tests/fixtures/golden/`.
  Negative control: the retired-number replace cases must fail before the fix
  if written as successes, and must still fail after it.
- Obligation broad input coverage: the independent property oracle gains a
  one-item same-number replacement modelled as identity continuation, while the
  two-item replacement keeps retirement. Artefact:
  `tests/roadmap_dependency_properties.rs` and `tests/support/`. Non-vacuity:
  both edit variants must occur across the generated cases; the oracle is
  independent of the implementation.
- Obligation user-visible behaviour: BDD scenarios for a successful in-place
  check-off and a changed-number rejection. Artefact:
  `tests/features/mapsplice.feature` and `tests/steps/cli_steps.rs`.
  Non-vacuity: the success scenario asserts the consumer's clause is unchanged.

### Formal verification decision

No new theorem is added. The change does not alter `select_resolution`; the
alias is an ordinary local mapping, so the existing target-local precedence
theorem already covers the decision. The construction and pairing rules are
finite structural transformations with examples and property coverage; a proof
would require a model of the parser and tree splice that the repository does
not have, and `docs/verification.md` records them as a pending claim tied to
issues #160 and #162 rather than pretending a vacuous theorem covers them.

### Suggested BDD features

Adopted in `tests/features/mapsplice.feature` (existing runner):

```gherkin
Scenario: Replace a depended-on task in place
  Given a roadmap whose second task requires the first task
  And a one-task fragment that checks off the first task
  When I replace the first task in place
  Then the command succeeds
  And the second task's Requires clause is unchanged

Scenario: Replacement that changes the number still dangles
  Given a roadmap whose second task requires the first task
  And a one-task fragment numbered for another position
  When I replace the first task in place
  Then the command fails with a dangling dependency
```

## Plan of work

### `EP-M1` Continuity aliases in the domain

Add
`RenumberPlan::record_continuity(&mut self, identity: ItemIdentity, new:
RoadmapAnchor) -> bool`
in `src/roadmap/model.rs`. It inserts only when the source-local map has no
entry for `identity.anchor` and returns whether it inserted. `resolve`,
`resolve_unique`, and `resolve_reference` keep their contracts (only `resolve`
may gain a doc note if needed).

Add `src/roadmap/ops/continuity.rs` with a `pub(crate)` `ItemPosition` enum
(phase/step/task/sub-task indices), a `ContinuityPlan` holding
`Vec<(ItemIdentity, ItemPosition)>`, constructors for each replacement level,
and `apply(&self, roadmap: &RoadmapDocument, plan: &mut RenumberPlan)` which
reads each position's final anchor and calls `record_continuity`.

In `replace_anchor`, return the plan. Trigger continuity only when the fragment
holds exactly one item at the anchor's level and that item's `identity.anchor`
equals the addressed number. For the other three replacement levels, gather the
retired subtree's identities and the fragment subtree's positions, pair by
exact anchor when each anchor occurs once on each side, and include the
addressed pair. Return an empty plan for changed-number, multi-item, and
non-replace operations.

Make `replace_phase` and `replace_step` return the first spliced position, and
expose the splice index from `replace_task` and `replace_sub_task` so the
position is exact. Preserve fragment identities and `TaskChild::SubTask`
markers; do not renumber or rebuild them.

In `apply_command_with_report`, capture the plan from `replace_anchor`, run
`renumber_document`, apply the aliases, then run `rewrite_dependencies`. The
staged clone continues to be discarded on error.

Red: model tests for `record_continuity` (records once, refuses overwrite,
resolves for target text, invisible to `resolve_unique`) and operation tests
for each level, descendant pairing, ambiguity, and unchanged sub-task markers
must fail before the implementation. Green: implement the helpers. Refactor:
move position helpers into `continuity.rs` and keep `mod.rs` under 400 lines.

### `EP-M2` Regressions, golden fixtures, properties, and BDD

Golden fixtures, registered in `tests/roadmap_golden/contracts.rs` and
`tests/roadmap_golden/nested_requires.rs`:

- `replace_depended_task_checks_off`: preview success, only the item line
  changes, consumer clause byte-identical.
- `replace_depended_task_checks_off_in_place`: same content, in-place result
  identical to the preview stdout.
- `replace_depended_task_changed_number_rejected_preview` and `_in_place`:
  `DanglingDependency`, target unchanged.
- `replace_depended_task_dropped_anchor_multi_item`: multi-item replacement
  without the addressed anchor still rejected.
- `replace_retains_referenced_descendant`: replacing a step or phase keeps a
  uniquely paired descendant that an external consumer requires.

Integration tests in `tests/roadmap_ops.rs` and
`tests/roadmap_ops/nested_requires.rs`: one-item same-number sub-task success
(keeping the existing multi-item rejection test unchanged), nested-bullet
`Requires` in-place, CRLF in-place preserving terminators, and a step- or
phase-level retained descendant.

Property suite: add a replacement variant whose fragment is one task generated
from the selected pre-edit anchor; the oracle models the addressed identity and
its still-present sub-tasks as continuing, and the two-item variant keeps
retirement. Confirm consumer-scoped checks assert continued edges.

BDD: add the two scenarios above and step definitions that reuse the existing
CLI helpers.

### `EP-M3` Documentation and delivery

Update `docs/users-guide.md` with the continuity rule, its retirement
counterexamples, and a checking-off example. Update `docs/mapsplice-design.md`
C1 and the section 7 resolution rationale with the operation-granted alias,
preserving "no target-to-fragment fallback" and "no unrelated number
inheritance". Update `docs/verification.md` with a `Pending` entry for alias
construction and descendant pairing, tagged #160 and #162, citing the
regression and property tests, and link retention to the target-local
precedence theorem in `verus/lib.rs` with the continuity-alias premise. Re-copy
the ledger into `tests/data/verification_ledger/real.md`. Draft revised wording
for issues #160 and #162 and report it to the maintainer rather than editing
the issues. Run all gates.

## Milestones and plateaus

- `EP-M1` Domain continuity. Requirements: `REQ-continuity`,
  `REQ-descendants`, `REQ-retirement`, `REQ-scope`. Acceptance: focused unit
  tests pass; `make check-fmt` and `make lint` pass. Conformance: no public
  API, dependency, or domain-purity change; trace links current. Recovery:
  revert the single commit; the staged clone design keeps failure atomic.
  Remaining: CLI, fixtures, properties, docs. Compatibility: none.
- `EP-M2` Observable behaviour. Requirements: `REQ-evidence`,
  `REQ-retirement`. Acceptance: golden, integration, property, and BDD suites
  pass; the new success cases fail before `EP-M1` and pass after. Conformance:
  fixtures follow design section 8; no test-only API leaks into production.
  Recovery: fixtures are additive. Remaining: documentation. Compatibility:
  none.
- `EP-M3` Documentation and delivery. Requirements: all. Acceptance: `make all`,
  `make markdownlint`, `make nixie`, `make spelling`, and the focused suites
  pass; the ledger copy test passes; the draft PR states `Closes #228`.
  Conformance: design and user guide agree with observed output. Remaining:
  merge. Compatibility: none.

## Concrete steps

Working directory: this worktree.

```plaintext
Red:    cargo test --lib roadmap::ops::continuity -> missing module/type
        cargo test --lib roadmap::model -> record_continuity missing
Green:  cargo test --lib roadmap -> all roadmap unit tests pass
Focused CLI: cargo test --test roadmap_ops -> new regressions pass
Golden: cargo test --test roadmap_golden -> all golden cases pass
Property: cargo test --test roadmap_dependency_properties -> pass
BDD:    cargo test --test behaviour_cli -> pass
Gates:  make check-fmt; make lint; make test; make markdownlint; make nixie;
        make spelling; make all (sequential, tee each log to /tmp)
Proof:  make verus; make verus-selftest (when installed)
```

Use `set -o pipefail` with `tee`; log paths follow
`/tmp/$ACTION-$(basename $(pwd))-$(git branch --show-current).out`.

## Validation and acceptance

A user can run `mapsplice replace target.md 1.1.2 fragment.md` where the
fragment contains the single task `1.1.2` with changed text, and observe exit
status 0, a rewritten task line, and every `Requires 1.1.2.` clause unchanged.
An identical replacement produces no diff. Replacing with a fragment numbered
`1.1.5`, or with two tasks, exits with `dangling_dependency` and writes nothing
in place. `delete 1.1.2` still fails when a consumer requires it. `cargo test`,
`make lint`, and the golden suite pass; the new fixtures fail before the change.

## Idempotence and recovery

Every step is a normal edit or test run; re-running a focused test is safe.
Fixtures are additive; a bad fixture is fixed in place. Commits are atomic per
milestone, so `git revert <commit>` restores the previous plateau. No step
writes outside the worktree except `/tmp` logs.

## Artefacts and notes

Reconnaissance evidence at `622b71d`:

- `src/roadmap/ops/mod.rs` pipeline and `replace_anchor` dispatch.
- `src/roadmap/model.rs` `RenumberPlan` API.
- `src/roadmap/ops/rewrite.rs` renumber-then-rewrite ordering and
  `DanglingDependency` construction.
- `tests/support/oracle_tree.rs` models `Edit::ReplaceTask` as removal plus
  splice, which must gain the continuation variant.
- `docs/verification.md` ledger rows and `tests/verus_harness/ledger.rs`
  byte-copy assertion.

Implementation transcripts are appended per milestone.

## Interfaces and dependencies

New domain-internal interface in `src/roadmap/model.rs`:

```rust
pub(crate) fn record_continuity(
    &mut self,
    identity: ItemIdentity,
    new: RoadmapAnchor,
) -> bool;
```

New module `src/roadmap/ops/continuity.rs`:

```rust
pub(crate) enum ItemPosition { /* phase/step/task/sub-task indices */ }
pub(crate) struct ContinuityPlan { /* pairs of retired identity and position */ }

pub(crate) fn phase_continuity(retired: &PhaseSection, fragment: &[PhaseSection], position: ItemPosition) -> ContinuityPlan;
pub(crate) fn step_continuity(retired: &StepSection, fragment: &[StepSection], position: ItemPosition) -> ContinuityPlan;
pub(crate) fn task_continuity(retired: &TaskEntry, fragment: &[TaskEntry], position: ItemPosition) -> ContinuityPlan;
pub(crate) fn sub_task_continuity(retired: &SubTaskEntry, fragment: &[SubTaskEntry], position: ItemPosition) -> ContinuityPlan;

impl ContinuityPlan {
    pub(crate) fn apply(&self, roadmap: &RoadmapDocument, plan: &mut RenumberPlan);
}
```

Ownership: the domain owns the rule; adapters see only the resulting success or
`DanglingDependency`. Reuse policy: continuity construction is the only place
that pairs retired identities with replacement positions; no other module may
record aliases. No dependencies are added.

## Revision note

- 2026-11-20: Initial draft derived from issue #228 and the task packet's
  three-task plan. No API, milestone order, or scope changed during drafting.
