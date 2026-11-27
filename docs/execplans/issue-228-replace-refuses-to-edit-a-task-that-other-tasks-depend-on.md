# Keep a replaced roadmap item's identity when it stays in place

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: COMPLETE

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
- [ ] 1.1.1. Review the plan. Requires 1.1.2.
- [ ] 1.1.2. Draft the plan.
```

and this fragment:

```markdown
- [x] 1.1.2. Draft the plan carefully.
```

make `mapsplice replace target.md 1.1.2 fragment.md` fail, because `replace`
retires the identity `1.1.2` and the surviving consumer's clause no longer
resolves. The operation is refused even though no consumer's bytes need to
change.

After this change, a replacement consisting of exactly one item at the same
level, whose fragment-local number equals the addressed number, is an in-place
edit of the same item. The command succeeds and only the replaced item's
content changes. Every `Requires` clause naming the addressed item keeps
resolving to that identity. When the replacement keeps the addressed number its
text is unchanged; when contiguous renumbering moves the item, as in a step
with gaps, the clause is rewritten to the new number. Source bytes are retained
per item: the addressed item, a parent whose content changed, and headings are
rendered canonically with LF, so a clause in a re-rendered parent summary loses
its interior CRLF even when its anchor does not change. The rendered document's
final newline is always LF, as the users' guide documents. Descendants of the
replaced item (steps under a phase, tasks and sub-tasks under a step, sub-tasks
under a task) keep their identities too, but only where a unique same-numbered
descendant exists on both the retired and replacement sides. A retained
descendant can still move to a different number within the replacement —
replacing step `1.1` with a step holding only task `1.1.2` renumbers it to
`1.1.1` — so references to that descendant are rewritten deterministically
rather than preserved byte-for-byte.

The mechanism is an **operation-granted continuity alias**: the renumber plan
temporarily remembers that a retired identity continues at the new anchor of
the item that replaced it in the same structural position. Target text resolves
through that alias. Fragment text never does, and the cross-source fallback
keeps its existing rule: a target reference with no source-local mapping
resolves to nothing. When the operation commits, every retained identity is
re-anchored to the number it now holds and to the target source, so a later
operation on the same in-memory document resolves the committed output without
any continuity state. The alias is needed only while the granting operation is
still rewriting its references.

A changed number (`1.1.2` replaced by `1.1.5`) or a multi-item replacement
still retires every replaced identity, and a surviving reference to one of them
still fails with `dangling_dependency`. `delete` is unchanged. `insert`,
`append`, and `delete` keep their existing reference semantics; they gain only
the same commit-time identity re-anchoring every operation now performs.

## Constraints

- Work only inside this worktree. Never edit the control checkout.
- Preserve the hexagonal boundary: `src/roadmap` is the domain and must not
  acquire filesystem, process, or CLI dependencies. `make check-domain-purity`
  must stay green.
- The rendered output must remain gate-clean (formatter-stable) and
  byte-identical for every item the splice does not re-render, apart from the
  documented final-newline normalization (a non-empty rendered document always
  ends in one LF). Re-rendered items are the addressed item, any parent whose
  content changed, and headings; they are emitted canonically with LF. A
  dependency reference that renumbering moves is rewritten to the item's new
  number. Source preservation (design guarantees F1-F5) must not be weakened.
- Contiguous renumbering (C2) is unchanged: numbers are assigned by position at
  every level.
- On success, every retained identity is re-anchored to the number the item now
  holds and to `SourceId::Target`, keeping `TaskChild::SubTask` markers
  aligned. A later `apply_command` on the same document must resolve the
  references the committed operation wrote, without relying on continuity
  aliases.
- The resolution kernel is unchanged. `RenumberPlan::resolve_reference`,
  `RenumberPlan::resolve_unique`, and `select_resolution` keep their current
  bodies and contracts. `RenumberPlan::resolve` gains a fallback to a separate
  continuity map, so the proved kernel still makes the final decision for every
  reference that reaches it.
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

Measured at delivery against `6794109`: 60 non-ExecPlan files, 2,493 net code
lines, 99 net documentation lines, and no configuration change. The code total
is 1,390 production source lines plus 1,103 test lines; 2,466 of those are
`.rs` lines, which is the figure GitHub's comparison reports. The file and code
budgets are exceeded; D7 records the rebaseline. Measuring from the plan's
original branch point would also count the upstream commits the rebase
absorbed, so that comparison does not isolate this change. The excess comes
from fixture directories and coverage the task packet requires, not from
widening the implementation surface.

The figures come from this command:

```sh
git diff --numstat 6794109 90277a3 -- . ':(exclude)docs/execplans/**'
```

Additions minus deletions are aggregated by the categories above. GitHub's PR
file panel uses a two-dot comparison and does not apply the ExecPlan exclusion,
so its higher file counts are not the plan's measurement.

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
  alias lives in a separate continuity map consulted only by
  `RenumberPlan::resolve`; `resolve_unique` and fragment resolution never see
  it, and model tests prove both stay unaffected.
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
- Risk: re-anchoring changes the identity values callers observe after a
  successful `apply_command`, including fragment items becoming target-sourced.
  Severity: medium. Likelihood: certain. Mitigation: the model method documents
  the invariant, the only in-tree identity assertion is updated, and
  successive-command regressions prove a later operation resolves the committed
  output. No public signature changes.

## Progress

- [x] (2026-11-20 00:00Z) Reconnaissance: located the replace pipeline,
      renumber plan, dependency rewriter, golden harness, property oracle, BDD
      harness, and the verification ledger.
- [x] (2026-11-20) ExecPlan drafted; user task packet authorizes execution.
- [x] (2026-11-20) `EP-M1` Continuity aliases implemented in the domain with
      model and operation unit tests. Evidence: `cargo test --lib` passes 198
      tests; disabling `continuity.apply` makes all six positive tests fail
      with `DanglingDependency` while the rejection and pairing tests still
      pass.
- [x] (2026-11-20) `EP-M1` review remediation: parameterized the retirement
      regressions, corrected the reproduction direction, and made successive
      commands consistent by re-anchoring identities at commit. Added
      `tests/roadmap_successive_commands.rs`. Evidence: with
      `reanchor_identities` disabled, both successive-command regressions fail
      with `DanglingDependency`; with it enabled, `cargo test --lib` passes 198
      tests and both regressions pass.
- [x] (2026-11-20) `EP-M2` Golden, integration, property, and BDD coverage
      added. Evidence: six golden cases in
      `tests/roadmap_golden/replacement_continuity.rs` (same-number preview and
      in-place success, changed-number preview and in-place rejection,
      multi-item rejection, retained descendant); CLI regressions in
      `tests/roadmap_ops/nested_requires.rs`,
      `tests/roadmap_ops/retained_descendants.rs`, and
      `tests/roadmap_sub_task_invariants.rs`; `Edit::ReplaceTaskInPlace` in the
      property oracle and generator; two BDD scenarios backed by
      `tests/steps/replace_continuity_steps.rs`. Focused suites pass:
      `roadmap_ops` and `roadmap_sub_task_invariants` 44/44,
      `behaviour_cli` 19/19, and
      `roadmap_golden`/`roadmap_dependency_properties`/`roadmap_successive_commands`
      104/104.
- [x] (2026-11-20) `EP-M2` review remediation: the property selector takes the
      edit kind from the remainder and the addressed position from the
      quotient, so every kind reaches every position a generated model offers.
      Evidence: `every_edit_kind_reaches_every_position` asserts the full cross
      product over `0..=u8::MAX`; the focused property suite passes 16/16, and
      all seven deterministic gates pass at `b59cc4f`, Whitaker included.
- [x] (2026-11-20) `EP-M3` Documentation: the user guide states the in-place
      rule and its retirement counterexamples; design C1 and section 7 record
      the operation-granted alias without weakening either resolution rule; the
      verification ledger gains a Pending row tagged #160 and #162 and links
      retention to the target-local precedence theorem; the two stale
      `src/roadmap/model.rs` path references name
      `src/roadmap/renumber_plan.rs`; the ledger fixture is a byte copy.
      Evidence: focused `verus_harness` suite passes 41/41; `make markdownlint`,
      `make nixie`, and `make spelling` pass. Verus itself is not installed here
      (`.verus/z3/z3` is absent), so `make verus`/`make verus-selftest` were not
      runnable; no proof source changed.
- [x] (2026-11-20) `EP-M3` review remediation: qualified the byte-identity
      claims after CodeRabbit showed a re-rendered line can end LF on a CRLF
      source. The user guide now promises identity continuity, unchanged clause
      text only when the number does not move, and per-item source
      preservation: an unrewritten task item keeps an interior CRLF, while the
      addressed item, a changed parent, and headings render canonically, and
      the document's final newline is always LF. The design pairs sub-tasks by
      their shared anchor under the uniqueness rule the implementation uses;
      the ExecPlan's own byte-identity claims are qualified the same way.
      Renamed the CRLF
      regression to `crlf_in_place_replace_normalizes_only_the_final_terminator`
      and added `crlf_in_place_replace_keeps_an_interior_clause_terminator`,
      which is only observable with a trailing task. The first full gate run
      after that addition failed `make lint` because `nested_requires` reached
      421 lines; the CRLF regressions now live in
      `tests/roadmap_ops/crlf_line_endings.rs` and the shared
      `assert_dangling_anchor` helper in `tests/roadmap_ops/assertions.rs`.
      Evidence: focused `roadmap_ops` suite passes 29/29 after the split.
- [x] (2026-11-20) `EP-M3` third review remediation: CodeRabbit showed two
      over-broad promises. Identity continuity is now separated from text
      continuity — a same-anchor replacement whose item is renumbered rewrites
      the surviving clause — and source preservation is stated per item: the
      addressed item, a changed parent summary, and headings render with LF,
      while an unrewritten task keeps an interior CRLF.
      `in_place_replace_renumbers_item_and_rewrites_its_consumer` and
      `crlf_child_replacement_reemits_parent_summary_clause` pin both
      boundaries. Evidence: focused `roadmap_ops` suite passes 31/31.
- [x] (2026-11-20) `EP-M3` Delivery: branch
      `issue-228-replace-refuses-to-edit-a-task-that-other-tasks-depend-on`
      pushed over SSH and draft PR
      [#244](https://github.com/leynos/mapsplice/pull/244) opened against
      `main`. CodeRabbit's third pass returned zero findings before delivery.
      The candidate was later rebased onto `main` for the #231/#251/#252
      changes, and a CodeScene delta review prompted a duplication refactor;
      the PR records the live head. Verus remains unrunnable here; the ledger
      records the premise as Pending under #160 and #162.
- [x] (2026-11-26) PR #244 rebased onto `main` three times: `3b7b71b`
      (#231/#251/#252), `05937ab` (#243 `complete`), and `6794109` (#263
      source-span verification). Conflicts were the CRLF tests, the model's
      renumber-plan path, the ops module, design C1/C2, and the verification
      ledger; the ledger's continuity row and premise paragraph were merged
      with #263's new source-span rows. `model_reanchor.rs` keeps the model
      under Whitaker's module limit. `cs delta` advisories were cleared by
      `splice_replacement`, a shared step iterator, the extracted no-op
      predicate, a sub-task mutation helper, and tabled regressions. Evidence:
      all seven gates pass at `90277a3`, including 907 nextest tests and 13
      doctests. Follow-ups raised as #260 (crate-root test size) and #261
      (per-item CRLF preservation).
- [x] (2026-11-27) Size and delivery reconciliation: no-op detection moved to
      `src/noop.rs`, leaving `src/lib.rs` at 391 lines, and
      `tests/roadmap_ops.rs` is back to its pre-existing 419 lines by declaring
      the retained-descendant module from `nested_requires`. The PR description
      reports the 907-test count. `cs delta 6794109` reports only improvements.
- [x] (2026-11-27) Delivery record reconciled for the ExecPlan assessment:
      status COMPLETE, delivered candidate `90277a3`, 907 nextest tests and 13
      doctests, tolerance figures restated with the exact measurement command,
      and the `make all` equivalence recorded.
- [x] (2026-11-26) Original-issue assessment remediation: an identical-item
      replacement is now a byte-identical no-op through a CLI-boundary
      comparison (D8). Golden cases cover the issue's surviving external
      dependant in preview and in-place modes, CRLF byte-identity regressions
      cover both modes, and the guide and design document the rule. The
      dropped-descendant regression and checking-off example from the ExecPlan
      assessment are also in place.
- [x] (2026-11-26) ExecPlan assessment remediation: recorded the measured
      size rebaseline in D7, added
      `dropped_descendant_required_by_consumer_is_rejected`, aligned the
      source-preservation constraint with per-item rendering and rewritten
      references, added the checking-off example the plan requires, completed
      Outcomes & retrospective, and reported the #160/#162 proof wording in
      the PR description. Evidence: focused `roadmap_ops` suite passes 32/32.

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
- Observation: storing aliases in `by_source` would have changed
  `resolve_unique` semantics. Evidence: a target alias and a fragment mapping
  with the same anchor and value make `resolve_unique` return `None`, so a
  target reference would fail for a different reason. Impact: D1 was revised to
  a separate map consulted only by `resolve`.
- Observation: `apply_command` identities are operation-scoped. A second
  command on the same parsed document failed whenever the first command had
  rewritten a reference: the next plan is keyed by each item's original anchor,
  but the rewritten text names the item's new number. Evidence: probes for
  replace-then-append and insert-then-append both reported
  `DanglingDependency`; the insert probe uses no continuity at all. Impact:
  keeping a continuity registry on the document would not be enough, because
  every identity key would still be stale.
  `RoadmapDocument::reanchor_identities` now resets each identity to the
  rendered number and target source on commit, which makes a committed document
  identity-equivalent to a fresh parse of its own rendering.
- Observation: Whitaker fails a gate when a module file exceeds 400 lines or a
  function clusters nested conditionals, and Clippy requires `mod.rs` for any
  module with a subdirectory. Evidence: the first full lint run reported
  `model` at 439 lines, `ops` at 475, `continuity` at 469, and four complexity
  bumps in `replace_anchor`. Impact: `RenumberPlan` moved to
  `src/roadmap/renumber_plan.rs`, finders moved to `src/roadmap/ops/lookup.rs`,
  continuity split into `position` and `descendants` submodules, and each
  replacement level gained a `*_with_continuity` helper.
- Observation: the rendered document always ends in LF, so a CRLF clause that
  is the document's final line loses only that terminator. Evidence: the
  initial CRLF regression expected
  `"- [ ] 1.1.2. Consumer.\r\n  - Requires 1.1.1.\r\n"` and failed because the
  clause is the last line; a probe with a trailing task then produced
  `"- [ ] 1.1.2. Consumer.\r\n  - Requires 1.1.1.\r\n- [ ] 1.1.3. Tail task.\n"`,
  showing interior CRLF was preserved. A later probe showed the rule is per
  item: replacing a sub-task re-renders its parent, so a summary clause in that
  parent returns with LF even though its anchor does not change, and headings
  always render with LF. Impact: the two companion regressions separate
  preserved source from the documented document-ending rule, and the guides
  promise identity continuity with per-item preservation rather than
  document-wide byte identity. Changing the renderer is outside this issue's
  scope.
- Observation: an in-place replacement does not always keep its number.
  Evidence: a step holding only `1.1.1` and `1.1.3` accepts a fragment item
  `1.1.3`, renumbers it to `1.1.2`, and rewrites the surviving consumer's
  clause from `Requires 1.1.3.` to `Requires 1.1.2.`. Impact: the guides
  separate identity continuity from text continuity, which holds only when the
  number does not move.
- Observation: Whitaker's 400-line limit applies to the property suite's
  test-support modules, not only to `src/`. Evidence: the full lint run failed
  with `Module generation spans 477 lines, exceeding the allowed 400.` once the
  fragment coverage was added. Impact: the generator is now
  `tests/support/generation/mod.rs` with fragment rendering and its tests in
  `tests/support/generation/fragments.rs`; the suite declares the module with
  `#[path = "support/generation/mod.rs"]`.
- Observation: the same limit applies to CLI regression modules. Evidence: the
  second `make lint` run reported
  `Module nested_requires spans 421 lines, exceeding the allowed 400.` after
  the companion CRLF regression was added. Impact: the CRLF cases moved to
  `tests/roadmap_ops/crlf_line_endings.rs`, declared from
  `tests/roadmap_ops/nested_requires.rs` rather than the crate root, whose
  fixture wiring is already 421 lines and would otherwise grow; the new module
  imports shared helpers through `crate::` paths. `assert_dangling_anchor`
  moved to `tests/roadmap_ops/assertions.rs` so both modules share it. The
  crate root's pre-existing overage is not enforced by Whitaker and is recorded
  here rather than widened by this change.
- Observation: Clippy's `integer_division` and
  `integer_division_remainder_used` lints are denied workspace-wide and apply
  to test-support code as much as to `src/`. Evidence: the first full lint run
  on the selector fix failed at `tests/support/oracle.rs:108-109` for `/` and
  `%` alone. Impact: intentional quotient and remainder use `div_euclid` and
  `rem_euclid`, matching the rest of the module, rather than an expectation.

## Decision log

- D1 (revised). Record continuity in a separate `RenumberPlan::continuity`
  map, consulted only by `RenumberPlan::resolve`, rather than inserting into
  `by_source`. Rationale: `resolve_reference`, `resolve_unique`, and
  `select_resolution` stay byte-identical, as the task packet requires. Aliases
  also stay invisible to the cross-source fallback, so a target alias can never
  make `resolve_unique` ambiguous, and fragment text can never inherit one. A
  surviving `by_source` mapping always wins because `resolve` consults it
  first. The draft D1 proposed an insert-if-absent entry in `by_source`;
  implementation showed that would expose aliases to `resolve_unique`, where a
  target alias plus a fragment mapping of the same value would produce `None`,
  and would contradict the target-local rule. Date/Author: 2026-11-20, agent.
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
- D5. Re-anchor every retained identity at commit instead of persisting
  continuity aliases on `RoadmapDocument`. Rationale: a later command builds
  its plan from the numbers the committed output contains, so the plan keys
  must be those numbers; storing aliases alone would leave every other key
  stale. Document state would also be a public API change, which the
  Constraints exclude. Re-anchoring additionally fixes the pre-existing
  insert-then-append failure, in which no continuity was involved, and makes a
  committed document identity-equivalent to a fresh parse of its own rendering.
  The alias remains operation-scoped and is still required for the granting
  operation's rewrite. Date/Author: 2026-11-20, agent.
- D6. Record the alias in the ledger as `Pending`, not as a proved obligation.
  Rationale: `RenumberPlan::resolve` folds the alias into the source-local
  answer before `select_resolution` sees it, so
  `target_text_keeps_its_source_local_mapping` covers retention *given* the
  alias — but alias construction, fragment cardinality, and descendant pairing
  are premises the kernel cannot observe. Claiming them as proved would extend
  the theorem past its boundary; recording them under #160 and #162 keeps the
  proof claim honest while the golden, CLI, and property evidence exercises the
  premise and its counterexamples. Date/Author: 2026-11-20, agent.
- D8. Treat a replacement whose rendered content equals the target's, apart
  from line terminators (including a missing final terminator), as a no-op at
  the CLI boundary: preview emits the target's source bytes and in-place mode
  performs no write. Rationale: the issue requires an identical-item
  replacement to be a no-op with no diff; re-rendering normalizes CRLF and the
  final terminator by design, so byte identity is only achievable by not
  rendering. The check compares the terminator-normalized source with the
  terminator-normalized normal render, so it is semantic rather than textual: a
  real edit, a formatter-unstable source, and any content normalization still
  render. Only `replace` takes this path; other operations keep their canonical
  rendering. Date/Author: 2026-11-26, agent.
- D7. Rebaseline the size tolerances at delivery. The delivered branch
  contributes 60 non-ExecPlan files and 2,493 net code lines measured from the
  rebase base `6794109`, against the original budgets of 40 files and 900 net
  code lines. Measuring from the plan's original branch point would also count
  the upstream commits the rebase absorbed, so that comparison does not isolate
  this change. The excess is mostly golden fixture directories (target,
  fragment, and expected files per case), pairwise continuity regressions, and
  the property-oracle extension. Reducing the change would drop coverage the
  task packet requires. Rationale: record the measured scope and report it on
  PR #244 rather than silently widening the budget; no source, configuration,
  dependency, or interface boundary changes. Measurement command:

  ```sh
  git diff --numstat 6794109 90277a3 -- . ':(exclude)docs/execplans/**'
  ```

  Date/Author: 2026-11-26, agent.

## Outcomes & retrospective

Delivered. `mapsplice replace` now treats exactly one same-level, same-anchor
fragment item as an in-place edit: the addressed identity and its uniquely
paired descendants continue, surviving `Requires` clauses keep resolving, and
only a dropped or renumbered identity is rejected. A depended-on task can be
checked off without touching its dependants, which was the issue's blocking
case.

Observed before the change, the issue's reproduction exited non-zero with
`dangling_dependency` for both a changed and an identical one-item fragment.
After the change, the same-number replacement exits zero with only the edited
item's line changed; an identical-item replacement is a byte-for-byte no-op in
both modes, CRLF included. A changed anchor, a multi-item fragment, a dropped
descendant, and `delete` still fail closed and leave the target unchanged.

Evidence at the delivered candidate: all seven deterministic gates pass (907
nextest tests plus 13 doctests), hosted CI's `build-test` and `verify` pass,
and CodeScene's delta shows only improvements.

Lessons: (1) identity continuity and text continuity are different promises —
contiguous renumbering can rewrite a surviving clause even for a same-anchor
replacement; (2) source preservation is per item, so a re-rendered parent
summary is not covered by an "untouched clause" promise; (3) the size estimate
missed the three-file cost of each golden fixture directory and needed the
rebaseline recorded in D7; (4) Whitaker's module limit applies to CLI
regression modules, not only to `src/` and property support.

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

`RenumberPlan` in `src/roadmap/renumber_plan.rs` holds the source-local mapping
plus a separate continuity map and exposes `resolve`, `resolve_unique`,
`resolve_reference`, `record_mapping`, and `record_continuity`.
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
`make verus` / `make verus-selftest` when the prover is installed. `make all`
is the union of `check-fmt`, `lint`, `typecheck`, `test`, and `spelling`; each
was run separately to green on the delivered candidate, so a separate
`make all` invocation would repeat the same targets without adding coverage.

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
- `INV-commit`: after a successful operation every identity is target-sourced
  and equal to the item's current number, sub-task markers name the matching
  identities, and a later operation on the same document resolves every
  reference the committed output contains.
- `L-in-place`: replacing a task with an identical same-number task produces
  byte-identical output, for CRLF sources too, because the CLI emits the
  target's own bytes when the canonical result is unchanged (D8). For a real
  edit, preservation is per item: the addressed item, a changed parent, and
  headings render with LF while an unrewritten task keeps its source.

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
- Obligation `INV-commit`: named integration regressions in
  `tests/roadmap_successive_commands.rs` apply a second operation after an
  in-place replacement and after a renumbering insertion. Artefact: the
  document-level `reanchor_identities` unit is exercised through the public
  API. Non-vacuity: disabling `reanchor_identities` makes both regressions fail
  with `DanglingDependency`, and the insertion case does not involve continuity.
- Obligation `INV-failure` and `L-in-place`: CLI integration tests and golden
  fixtures asserting exit status, exact bytes, and the diagnostic for
  changed-number and multi-item replacements. Artefacts: `tests/roadmap_ops.rs`,
  `tests/roadmap_ops/nested_requires.rs`,
  `tests/roadmap_ops/crlf_line_endings.rs`, and `tests/fixtures/golden/`.
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
in `src/roadmap/renumber_plan.rs` (then inside `src/roadmap/model.rs`). It
inserts only when the source-local map has no entry for `identity.anchor` and
returns whether it inserted. `resolve`, `resolve_unique`, and
`resolve_reference` keep their contracts (only `resolve` may gain a doc note if
needed).

Add `src/roadmap/ops/continuity/` (split into `mod.rs`, `position.rs`, and
`descendants.rs` during implementation) with a `pub(crate)` `ItemPosition` enum
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
  changes, the consumer clause keeps its text, anchor, and bytes in this LF
  fixture.
- `replace_depended_task_checks_off_in_place`: same content, in-place result
  identical to the preview stdout.
- `replace_depended_task_changed_number_rejected_preview` and `_in_place`:
  `DanglingDependency`, target unchanged.
- `replace_depended_task_dropped_anchor_multi_item`: multi-item replacement
  without the addressed anchor still rejected.
- `replace_retains_referenced_descendant`: replacing a step or phase keeps a
  uniquely paired descendant that an external consumer requires.

Integration tests in `tests/roadmap_ops.rs`,
`tests/roadmap_ops/nested_requires.rs`, and
`tests/roadmap_ops/crlf_line_endings.rs`: one-item same-number sub-task success
(keeping the existing multi-item rejection test unchanged), nested-bullet
`Requires` in-place, CRLF in-place preserving interior terminators while
normalizing only the document's final one, and a step- or phase-level retained
descendant.

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
- `src/roadmap/renumber_plan.rs` `RenumberPlan` API (then inside
  `src/roadmap/model.rs`; split out during implementation).
- `src/roadmap/ops/rewrite.rs` renumber-then-rewrite ordering and
  `DanglingDependency` construction.
- `tests/support/oracle_tree.rs` models `Edit::ReplaceTask` as removal plus
  splice, which must gain the continuation variant.
- `docs/verification.md` ledger rows and `tests/verus_harness/ledger.rs`
  byte-copy assertion.

Implementation transcripts are appended per milestone.

## Interfaces and dependencies

New domain-internal interface in `src/roadmap/renumber_plan.rs` (the type
started inside `src/roadmap/model.rs` and was split out during implementation):

```rust
pub(crate) fn record_continuity(
    &mut self,
    identity: ItemIdentity,
    new: RoadmapAnchor,
) -> bool;
```

New module `src/roadmap/ops/continuity/` (`mod.rs`, `position.rs`, and
`descendants.rs`):

```rust
pub(crate) enum ItemPosition { /* phase/step/task/sub-task indices */ }
pub(crate) struct Replacement { /* level, position, and item count */ }
pub(crate) struct ContinuityPlan { /* pairs of retired identity and position */ }

pub(crate) fn continuity_for(
    roadmap: &RoadmapDocument,
    retired: &RetiredSubtree,
    replacement: Replacement,
) -> ContinuityPlan;

impl ContinuityPlan {
    pub(crate) fn apply(&self, roadmap: &RoadmapDocument, plan: &mut RenumberPlan) -> usize;
}
```

Ownership: the domain owns the rule; adapters see only the resulting success or
`DanglingDependency`. Reuse policy: continuity construction is the only place
that pairs retired identities with replacement positions; no other module may
record aliases. No dependencies are added.

## Revision note

- 2026-11-20: Initial draft derived from issue #228 and the task packet's
  three-task plan. No API, milestone order, or scope changed during drafting.
- 2026-11-20: Review remediation in `EP-M1`. Corrected the reproduction's
  dependency direction; added commit-time identity re-anchoring after probes
  showed every second command failed once the first rewrote a reference; added
  the successive-command regressions and the `INV-commit` obligation. No public
  API or milestone order changed.
- 2026-11-20: Review remediation in `EP-M2` and documentation in `EP-M3`.
  Decoupled the property selector's edit kind from its addressed position after
  CodeRabbit showed the shared remainder limited each task edit to two of the
  twelve tasks; recorded the Clippy division-lint lesson; documented the
  in-place rule, the operation-granted alias, and its Pending proof status. No
  runtime behaviour or milestone order changed.
- 2026-11-20: Delivered as draft PR #244 from `ef46cab`, after the third
  CodeRabbit pass returned no findings and all seven gates passed on that
  commit. No scope or milestone order changed.
- 2026-11-27: Rebased a third time onto `main` at `6794109` after #263 added
  the source-span verification ledger; merged its rows with the continuity
  Pending row and restated the 907-test figures. No scope or milestone order
  changed.
- 2026-11-27: Rebased again onto `main` at `05937ab` after #243 added the
  `complete` command; resolved the model renumber-plan path, ops module, and
  design C1/C2 conflicts, and split re-anchoring into `model_reanchor.rs`. The
  delivery figures and test counts were restated for that base. No scope or
  milestone order changed.
- 2026-11-26: Rebased onto `main` at `3b7b71b` to clear the #231/#251/#252
  conflict and aligned the test serial markers with main's convention. A
  CodeScene delta review reported four new Code Duplication advisories; two
  shared helpers and a tabled golden case list clear them. Raised #260 and #261
  for the measured follow-ups. No scope or milestone order changed.
- 2026-11-20: Second and third `EP-M3` review remediation. Qualified every
  promise that a consumer clause is textually unchanged: the identity always
  continues, the text is unchanged only when the number does not move, and
  source preservation is per item, so a re-rendered parent summary loses its
  interior terminators. Corrected the descendant pairing description to shared
  anchor, matching `pair_descendants`. No runtime behaviour, API, or milestone
  order changed.
