# 1.2.3 Diagnose `Requires` keywords with no recognizable anchors

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: DRAFT (design review applied; awaiting user approval)

Branch: `1-2-3-diagnose-requires-keywords-with-no-recognizable-anchors`

Roadmap item: `1.2.3` in `docs/roadmap.md`.

Implementation must not begin until the user explicitly approves this plan.

## Purpose / big picture

`mapsplice` is a command-line tool that makes structural edits (append, insert,
delete, replace) to numbered Markdown roadmaps such as `docs/roadmap.md`. After
an edit it renumbers every phase, step, task, and addendum sub-task, and it
rewrites *dependency references*: the anchor numbers written after the keyword
`Requires`, such as the `1.2.2` in `- Requires 1.2.2.`. Only numbers in that
dependency context are rewritten; section references, versions, and prose
numbers are left alone.

The scanner that finds dependency references works number-first. It looks for
anchor-shaped tokens (`8`, `8.2`, `8.2.3`, `8.2.3.1`) and then looks backwards
for a `Requires` keyword that governs them. A `Requires` keyword that governs
no recognizable token therefore leaves no trace at all. Today all of these are
accepted silently and are never renumbered, validated, or reported:

```markdown
- [ ] 1.2.3. Ship the importer.

  - Requires none.
  - Requires: TBD
  - **Requires** 1.2.1.
  - Requires `1.2.2`.
  - Requires §1.2.
```

The last three look like dependencies to a reader. They are not, because the
emphasis, the inline code span, and the section sigil each hide the number from
the dependency context. When a later edit renumbers task `1.2.1`, the clause
keeps its stale number, which is exactly the silent corruption `mapsplice`
exists to prevent.

After this change, every successful edit reports one warning on standard error
for each `Requires` keyword that governs zero recognized anchor tokens. The
warning names the file and the line and column where the keyword was read, and
the item that contains it by its number *after* the edit, adding its number
before the edit when that differs. The edit still succeeds, standard output
still carries exactly the rendered roadmap, `--in-place` still rewrites the
target, and the exit status is still zero. For example, deleting task 1.1.1
from a roadmap whose task 1.1.2 contains `- Requires none.` prints:

```plaintext
$ mapsplice delete roadmap.md 1.1.1 > new.md
roadmap.md:10:5: warning: task 1.1.1 (was 1.1.2): `Requires` clause has no recognizable dependency anchors
$ echo $?
0
```

A roadmap whose clauses all carry at least one recognized token, including this
repository's own `docs/roadmap.md`, produces no warnings. Failure behaviour is
unchanged: when an edit fails, only the existing error diagnostic is printed.

This work closes roadmap item 1.2.3 and completes step 1.2, "Recognize
dependency clauses in every clause position".

## Constraints

- Implement only roadmap item 1.2.3. Do not change which numbers are rewritten,
  how a clause is recognized, or how dependency references resolve. The
  dependency-reference predicate in `src/roadmap/ops/dependency_text.rs`
  (`classify_dependency_reference`, `is_dependency_anchor`,
  `has_dependency_clause_separator`, `has_dependency_clause_terminator`,
  `has_section_sigil`, `next_anchor_candidate`) keeps its current accept and
  reject decisions. Every existing test in
  `src/roadmap/ops/dependency_text_tests.rs`, `tests/roadmap_golden.rs`,
  `tests/roadmap_golden/nested_requires.rs`, and
  `tests/roadmap_dependency_properties.rs` must pass unchanged in meaning.
- Do not change the JSON export path (`src/roadmap/export/`). Its existing
  handling of anchorless clauses is recorded as a known divergence (D12).
- Preserve every guarantee in `docs/mapsplice-design.md` sections 5, 6, and 7
  (F1–F5, C1–C6, and the normative dependency-reference model). A warning is
  not a failure: it must never change standard output bytes, in-place file
  bytes, or the exit status of a successful edit (`INV-non-interference`).
- Preserve F5 (fail closed). When an edit fails, report only the existing
  error. Warnings exist only inside a successful result.
- Standard output carries only the rendered roadmap. Warnings go to standard
  error and nowhere else.
- Domain purity: code under `src/roadmap/` must stay free of filesystem,
  process, environment, and terminal access (`scripts/check-domain-purity.sh`,
  `make check-domain-purity`). The domain returns warnings as data; only
  `src/lib.rs` maps a source document to a file path, and only `src/main.rs`
  writes to standard error.
- No code file may exceed 400 lines (`AGENTS.md`). At the base revision
  `src/roadmap/ops/dependency_visitor.rs` is 399 lines and
  `src/roadmap/ops/mod.rs` is 383 lines, so `EP-M1` moves code out of the
  visitor before any behaviour is added, and `src/roadmap/ops/mod.rs` may grow
  by at most 10 lines.
- Every module starts with a `//!` comment; every `assert!`, `assert_eq!`, and
  `assert_ne!` carries a diagnostic message (`clippy::missing_assert_message`
  is denied in `Cargo.toml`); production code and shared fixtures do not use
  `expect` or `unwrap`.
- Tests that start the binary pass `RUST_LOG=off` to the child process (the
  subprocess exemption in `AGENTS.md`), except the one process test that
  deliberately sets `RUST_LOG=warn`. No test mutates the environment of the
  test process.
- Authorized dependency changes: add `googletest = "0.14.4"` and
  `pretty_assertions = "1.4.1"` to `[dev-dependencies]` in `Cargo.toml`, with
  caret semantics, as the user directed. No other dependency may be added. No
  runtime dependency may be added.
- Authorized public API changes (the crate is version `0.1.0`, pre-1.0, with no
  release tag, so no source-compatibility machinery is required or permitted):
  add `#[non_exhaustive]` and a public `warnings: Vec<MapspliceWarning>` field
  to `mapsplice::RunOutcome`; add the public struct
  `mapsplice::MapspliceWarning` (private fields, public accessors, `Display`)
  and the public `#[non_exhaustive]` enum `mapsplice::WarningKind`, both
  re-exported from the crate root. Every constructor and reader of `RunOutcome`
  is updated in the same change. `mapsplice::apply_command` keeps its signature.
- Do not modify completed ExecPlans under `docs/execplans/` other than this
  file.
- `typos.toml` is generated; put any spelling exceptions in `typos.local.toml`.
- Follow the gate order in `AGENTS.md`: `make check-fmt`, `make lint`, and
  `make test` run sequentially, never in parallel with each other for this
  worktree.

If satisfying the objective requires violating a constraint, stop, record the
conflict in `Decision log`, set the status to `BLOCKED`, and escalate.

## Tolerances (exception triggers)

The comparison base is `origin/main` at `622b71d` (the base revision of this
plan). Sizes are net lines (additions minus deletions) counted with:

```bash
git diff --numstat 622b71d -- . ':(exclude)docs/execplans/*' ':(exclude)Cargo.lock' ':(exclude)typos.toml'
```

ExecPlan files, including this file, are excluded from every count.
`Cargo.lock` and `typos.toml` are generated and excluded from the line counts,
but any lockfile change other than the entries pulled in by the two authorized
dev-dependencies is a dependency tolerance breach.

Classification is whole-file by purpose:

- Code: everything under `src/` and `tests/`, including Markdown and text
  fixtures under `tests/fixtures/`, `tests/features/`, and snapshot files,
  because they are executable test specification.
- Configuration: `Cargo.toml`, `typos.local.toml`, and `.github/`.
- Documentation: everything under `docs/` except `docs/execplans/`, plus
  `README.md` and `skill/`.

Budget survey. The closest precedents are the dangling-dependency report (commit
`84ab09e`, 15 files, 1,248 insertions) and roadmap item 1.2.1 (commit
`a566a4d`, pull request #86, 69 files, 6,876 insertions). Item 1.2.1 also
bootstrapped the Verus prover harness, the generated-roadmap oracle, and the
dependency property suite, so most of its size was approved infrastructure, not
missed scope; this plan reuses that infrastructure. The recent refactor plans
`docs/execplans/roadmap-5-1-4.md` (production budget 180 lines) and
`docs/execplans/roadmap-4-4-4.md` (production budget 80 lines) recorded no
budget increases. No comparable plan records a repeatedly raised budget. The
design review in this plan's `Decision log` (D4) removed an estimated 150
production and 150 test lines by collecting warnings in the existing rewrite
pass. The estimate below traces every affected file listed in
`Context and orientation`: about 280 production lines (scanner kernel 50,
rewrite and visitor plumbing 60, edit report 30, warning type 90, application
and binary wiring 50), about 920 test lines (unit tables 120, bounded
enumeration and property 200, golden harness extension 50, fixtures 150,
behavioural steps and features 100, process and snapshot tests 100, generated
property model 120, location regressions 80), and about 350 documentation lines
(ADR 80, design 70, users' guide 80, developers' guide 50, migration note 25,
validation design 10, skill 15, verification ledger prose 10, `README.md` 5,
roadmap 8).

- Files: stop and escalate if more than 40 non-ExecPlan files change.
- Code: stop and escalate if code changes exceed 1,400 net lines in total, or
  if production code under `src/` alone exceeds 450 net lines.
- Configuration: stop and escalate if configuration changes exceed 25 net
  lines.
- Documentation: stop and escalate if documentation changes exceed 500 net
  lines.
- Interface: stop and escalate if any public API change beyond those
  authorized in `Constraints` is needed, or if `apply_command`,
  `RoadmapDocument`, or `MapspliceError` must change.
- Dependencies: stop and escalate if any dependency beyond `googletest` and
  `pretty_assertions` (and their transitive lockfile entries) is needed.
- Architecture: stop and escalate if the warning cannot be computed without
  changing the dependency-reference predicate, the renumber plan, or the
  rendering pipeline, or if domain code would need to know file paths.
- Iterations: stop and escalate if a focused test still fails after three
  implementation attempts for the same cause.
- Time: stop and escalate if any single milestone takes more than six hours of
  active work.
- Ambiguity: stop and present options if implementation evidence shows that a
  class of clause listed in D2, D3, or D11 should behave differently.

Each category is enforced independently; spare capacity in one category does
not offset an overrun in another.

## Risks

- Risk: the warning fires on prose that legitimately starts with the word
  `Requires` outside inline code (for example, "Requires the merged importer").
  Severity: medium. Likelihood: medium. Mitigation: the warning is non-fatal
  and follows the existing case-sensitive keyword rule exactly, so the same
  prose is already a dependency context for any number that follows it. The
  users' guide tells authors to wrap the *keyword* in inline code or reword the
  sentence when it is deliberately not a dependency, and warns against
  un-quoting deliberately quoted numbers (an archived-task reference such as
  ``Requires archived task `1.2.2` `` would then fail as a dangling dependency).
  `docs/roadmap.md` already writes every prose mention as `` `Requires` ``,
  and a dogfood acceptance check proves it yields zero warnings.
- Risk: a user merges standard error into the output file
  (`mapsplice … > new.md 2>&1`), so a warning line lands in the roadmap.
  Because the warning quotes `` `Requires` `` in a code span, the stray line
  never warns again. Severity: medium. Likelihood: low. Mitigation: no test can
  prevent shell misuse; the migration note, users' guide, and
  `skill/mapsplice/SKILL.md` state that successful edits may now write to
  standard error and that the streams must not be merged.
- Risk: the reported line and column refer to the input file, which
  `--in-place` has already overwritten. Severity: low. Likelihood: medium.
  Mitigation: the item is named by its post-edit number, which is authoritative
  in the rewritten file; the "was" number and the coordinates are documented as
  describing the file as read; an in-place scenario with a renumbered
  anchorless item pins the text.
- Risk: splitting `src/roadmap/ops/dependency_visitor.rs` to make room changes
  behaviour. Severity: medium. Likelihood: low. Mitigation: `EP-M1` is a
  behaviour-preserving move committed on its own, gated by the full existing
  suite before any new behaviour lands.
- Risk: a clause that hides only some of its numbers, such as
  `Requires 1.2.1 and **1.2.2**`, stays silent because one token is recognized.
  Severity: medium. Likelihood: low. Mitigation: out of scope for this item
  (the roadmap text asks for clauses with zero recognized tokens); recorded as
  a known limitation in the design document and offered to the user as a
  follow-up roadmap candidate.
- Risk: scripts that treat any standard-error output as failure start failing
  on warnings. Severity: medium. Likelihood: low. Mitigation: exit status
  remains zero; the migration note and users' guide tell automation to rely on
  the exit status.
- Risk: the edit-time warning and the planned phase 6 `validate` finding for
  the same condition diverge. Severity: low. Likelihood: medium. Mitigation:
  the warning carries the stable code
  `mapsplice::dependency::anchorless_clause`, recorded in
  `docs/validation-and-agent-output-design.md` for phase 6 to reuse.
- Risk: the JSON export treats the same clauses differently (it rejects
  `**Requires**` and accepts `Requires none.` as clause prose). Severity: low.
  Likelihood: certain. Mitigation: no change in this item; the divergence is
  recorded in the ADR and design section 7 (D12).

## Progress

- [x] (2026-11-20 10:40Z) Branch renamed to
  `1-2-3-diagnose-requires-keywords-with-no-recognizable-anchors`, tracking
  `origin/1-2-3-diagnose-requires-keywords-with-no-recognizable-anchors`.
- [x] (2026-11-20 11:30Z) Reconnaissance completed with a Wyvern agent team
  (scanner, diagnostics plumbing, test and proof infrastructure, design and
  plan history).
- [x] (2026-11-20 12:30Z) Initial draft written.
- [x] (2026-11-20 14:10Z) Community-of-experts design review (structure and
  contracts; failure modes and verification; alternatives and viability)
  applied; see the revision note.
- [ ] Plan approved by the user.
- [ ] `EP-M0` feasibility probe recorded.
- [ ] `EP-M1` dependency visitor split without behaviour change.
- [ ] `EP-M2` keyword-ownership kernel with unit, bounded, and property
  evidence.
- [ ] `EP-M3` anchorless clauses collected by the rewrite pass.
- [ ] `EP-M4` warnings surfaced through `RunOutcome` and standard error.
- [ ] `EP-M5` documentation, ADR, and roadmap closure.

## Surprises & discoveries

- Observation: no non-fatal warning channel exists anywhere in the product.
  Evidence: `RunOutcome` in `src/lib.rs` has only `stdout` and `written_path`;
  `src/main.rs` writes to standard error only on failure; no `warn!` call
  exists under `src/`. Impact: this item introduces the first warning channel,
  which phase 6 will extend; see D5 and the ADR in `EP-M5`.
- Observation: `tracing` is initialized with
  `EnvFilter::from_default_env()`, whose default directive is `error`, so a
  `tracing::warn!` event is invisible unless `RUST_LOG` is set. Evidence:
  `init_tracing` in `src/main.rs`. Impact: the user-visible warning is written
  through `report_stderr_diagnostic`, not through `tracing`.
- Observation: an existing unit test already pins `Requires none.` as producing
  no dependency candidates. Evidence:
  `src/roadmap/ops/dependency_text_tests.rs` case `Count 27. Requires none.`.
  Impact: that expectation stays true; the warning is a new output alongside
  candidates.
- Observation: `src/roadmap/ops/dependency_visitor.rs` is 399 lines, one under
  the repository limit. Impact: `EP-M1` moves the text-group helpers and the
  container-descent functions out first.
- Observation: `rewrite_dependencies` does not visit regions in document
  order: it visits `phase.trailing` before the phase's steps, `step.trailing`
  before the step's tasks, and task body children before sub-tasks
  (`src/roadmap/ops/rewrite.rs`). Impact: warnings are sorted explicitly (D7).
- Observation: `RoadmapAnchor` derives `Ord` with the variant first, so
  `Phase(2) < Step(1.1)`; that ordering is not document order. Impact: D7 sorts
  by the anchor's numeric components.
- Observation: fragment `MarkdownNodes` carry the fragment's own source text
  (`src/roadmap/parse/fragment.rs`, `SourceText`), and each item's
  `identity.anchor` is the anchor as written in its own source, even after
  renumbering. Impact: fragment warnings report fragment coordinates and the
  fragment spelling as the "was" number.
- Observation: the golden workspace uses absolute temporary paths
  (`tests/golden/workspace.rs`, `create_workspace`). Impact: pinned warning
  text replaces the workspace root with a placeholder before comparison.
- Observation: the repository's ExecPlan file names follow
  `roadmap-N-N-N.md`, but the user's instructions name this plan after the
  branch. Impact: this plan uses the requested name; no existing plan is
  renamed.

## Decision log

- D1. Decision: a *dependency keyword occurrence* is an occurrence of the
  literal, case-sensitive `Requires` whose neighbouring bytes are not ASCII
  alphanumeric, exactly as `is_keyword_boundary` decides today. A keyword
  occurrence is *anchorless* when no dependency candidate recorded by the
  dependency visitor is owned by it, where the owner of a candidate is the
  keyword occurrence that `latest_keyword_before` already selects. Rationale:
  the roadmap item defines the condition relative to "the scanner", so the
  warning must be derived from the scanner's own ownership decision rather than
  from a second grammar. Rejected option: a keyword-first clause grammar, which
  would give the product two definitions of a dependency clause. Date/Author:
  2026-11-20, planning agent; confirmed by design review.
- D2. Decision: a clause whose only tokens are malformed dependency tokens,
  such as `Requires 1.4.0` or `Requires 1.2.3.4.5`, is *not* anchorless and
  does not warn. Rationale: the scanner does recognize these as anchor-shaped
  dependency tokens (`DependencyCandidate { anchor: None }`), and roadmap item
  1.1.1 deliberately preserves them. Warning about malformed tokens in
  dependency context is the separate phase 6 rule in
  `docs/validation-and-agent-output-design.md` ("version-like or invalid
  dependency tokens are warnings"), owned by roadmap item 6.1.2. The users'
  guide says so explicitly. Rejected option: treating "zero *valid* anchors" as
  anchorless, which would fold 6.1.2 scope into this item. Date/Author:
  2026-11-20, planning agent; confirmed by design review.
- D3. Decision: these are anchorless and warn: `Requires none.`,
  `Requires: TBD`, a bare `Requires` or `Requires.` at the end of a paragraph,
  `Requires §1.2`, `Requires-1.2`, `Requires v1.2`, `Requires none1.2`, a table
  cell `| Requires |`, a keyword whose number sits in a different inline group
  (`**Requires** 1.2.1`, ``Requires `1.2.1` ``, `Requires [1.2.1](#x)`), and
  `Requires none; see 1.2.1` (the semicolon ends the clause). These do not
  warn: lowercase `requires`, `PreRequires`, `Requirements`, any keyword inside
  inline code, fenced code, HTML, or a link destination (the visitor never
  enters them), and a clause with at least one valid or malformed token. Each
  follows mechanically from D1 and the existing scanner; `EP-M2` pins each one
  in a table. Rationale: the inline-group cases are precisely the "malformed or
  unparsable" clauses the roadmap item exists to surface. Date/Author:
  2026-11-20, planning agent; extended by design review.
- D4. Decision: collect anchorless keywords inside the existing
  `rewrite_dependencies` pass, which already runs over the edited (staged)
  document and visits every dependency region with the owning item at hand. The
  rewrite visitor computes anchorless keywords from the same candidate list it
  rewrites, and records each keyword's source coordinates from the text
  *before* replacements are applied. The result travels out of
  `apply_command_with_report` in an `EditReport`. Rationale: one traversal
  means the rewritten regions and the diagnosed regions cannot drift apart;
  locations cannot shift when a neighbouring token changes length; a failed
  rewrite returns `Err`, which drops the warnings, so F5 holds by construction;
  and both the item's new number and its identity are available. Rejected
  options: a separate read-only query that mirrors the rewrite walk (the first
  draft), whose parity test could not detect a region later added to the
  rewriter only; and scanning the inputs before the operation, which warns
  about deleted items and needs a second walk over the four `RoadmapFragment`
  variants. Trade-off accepted: the rewrite command returns one more report
  field, alongside the counts it already returns. Date/Author: 2026-11-20,
  design review (all three reviewers).
- D5. Decision: introduce a minimal warning channel. The domain returns
  `AnchorlessClause` values in `EditReport`; `src/lib.rs` maps each to a public
  `MapspliceWarning` carrying the file path; `RunOutcome` gains
  `warnings: Vec<MapspliceWarning>`; `src/main.rs` writes standard output first
  and, only once that succeeds (or meets a closed pipe), writes each warning's
  `Display` text as one line to standard error and emits one `tracing::warn!`
  event with the stable `warning_code`. Rationale: this mirrors the existing
  error path (`report_error` traces a class and writes the display text), keeps
  domain purity, keeps `INV-failure-silence` true even when standard output
  fails, and gives phase 6 a typed channel to extend. Rejected options:
  `tracing::warn!` alone (invisible by default), adding strings to the `Copy`
  `PreservationReport`, and a full severity and finding-code model (owned by
  phase 6). Date/Author: 2026-11-20, planning agent; ordering amended by design
  review.
- D6. Decision: each warning is one standard-error line, produced entirely by
  `MapspliceWarning`'s `Display` so the binary and every fixture share one
  formatter:

  ```plaintext
  <path>:<line>:<column>: warning: <kind> <current>[ (was <original>)]: <message>
  ```

  `<message>` is exactly
  `` `Requires` clause has no recognizable dependency anchors ``.

  `<path>` is the target or fragment path as given on the command line;
  `<line>` and `<column>` are one-based coordinates of the keyword in that file
  as it was read; `<kind>` is `RoadmapItemLevel`'s `Display` (`phase`, `step`,
  `task`, `sub-task`); `<current>` is the item's number after the edit;
  `(was <original>)` appears when the number written in that file differs. Text
  outside any item is labelled `preamble` with no number. When the location is
  unknown, `:<line>:<column>` is omitted. Rationale: this is the GNU layout for
  diagnostics with a source file (`file:line:column: message`, lower-case
  message, no final full stop), which editors and terminals can jump to[^gnu];
  the post-edit number is authoritative after `--in-place`; the "was" number
  connects it to the coordinates. The format is provisional until phase 6
  adopts a renderer (`docs/validation-and-agent-output-design.md`), and the ADR
  says so. Date/Author: 2026-11-20, design review.
- D7. Decision: warnings are ordered by a document-order key: preamble first,
  then by the post-edit anchor's numeric components compared lexicographically
  (`1 < 1.1 < 1.1.1 < 1.1.1.1 < 1.1.2 < 1.2 < 2`), then by line and column.
  There is one warning per anchorless keyword occurrence, with no
  de-duplication. Rationale: the rewrite traversal is not in document order (see
  `Surprises & discoveries`); this key is deterministic and matches the order
  in which a reader meets items. Text in a phase's trailing blocks sorts with
  its phase, before the phase's steps; the users' guide states this.
  Date/Author: 2026-11-20, design review.
- D8. Decision: no Verus proof and no Kani harness for this item. Evidence for
  the ownership kernel comes from an `rstest` partition table, a bounded
  exhaustive enumeration over a lexeme alphabet checked against a metamorphic
  oracle, and a `proptest` property using the same oracle. Rationale: see
  `Verification plan`, "Formal verification decision". Date/Author: 2026-11-20,
  planning agent; confirmed by design review.
- D9. Decision: warnings are not configurable in this item (no `--quiet` flag,
  no suppression comment, no cap on count). Rationale: no requirement asks for
  it; the warning is fixed by editing the clause; phase 6 owns output-mode
  policy and flood control. Date/Author: 2026-11-20, planning agent.
- D10. Decision: no new metric counter. Rationale: no usage, uptake, failure,
  or mitigation metric is required by the roadmap or design; the stable
  `warning_code` tracing field is enough for log-based correlation.
  Date/Author: 2026-11-20, planning agent.
- D11. Decision: warn in every region the rewriter scans (preamble, phase and
  step titles, bodies, and trailing blocks, task and sub-task text, and table
  cells), not only in task bodies. Rationale: those regions are already
  dependency contexts for the rewriter, which rewrites any anchored `Requires`
  number found there; an anchorless keyword in them is equally inert.
  Collecting in the rewrite pass (D4) makes this the natural, single rule.
  Rejected option: restricting warnings to task and sub-task bodies, which
  would leave inert clauses silent in regions the rewriter does act on.
  Date/Author: 2026-11-20, planning agent, raised as an open question by design
  review.
- D12. Decision: do not change `export` in this item. The export path already
  rejects `**Requires**` (`formatted_requires` in
  `src/roadmap/export/requires.rs`) and records `Requires none.` as a clause
  followed by prose, so an edit can warn where an export errors or stays
  silent. Rationale: the roadmap item is scoped to edit time; read-only checks
  belong to the phase 6 `validate` command. The divergence is recorded in the
  ADR and design section 7. Date/Author: 2026-11-20, design review.
- D13. Decision: warnings reach library callers through `run_request` and
  `run_from_args` (`RunOutcome::warnings`); the lower-level `apply_command`
  keeps returning only the rewrite count. Rationale: the authorized interface
  change is confined to `RunOutcome`; a public query can be added when a
  library consumer needs it. Date/Author: 2026-11-20, design review.
- D14. Decision: `MapspliceWarning` is a struct with private fields and
  accessors, and its kind is a `#[non_exhaustive]` enum `WarningKind` with a
  `code()` method returning a namespaced code. Rationale: phase 6 findings need
  extra fields (range, help, related items); private fields let them be added
  without breaking callers, which a public enum variant with named fields would
  not. Date/Author: 2026-11-20, design review.
- D15. Decision: use `googletest` matchers and `pretty_assertions::assert_eq!`
  in the tests this item adds, as the user directed; existing tests are not
  migrated. Rationale: the user authorized and asked for both; one design
  reviewer preferred to defer them to a repository-wide change, which is
  recorded here as a dissent. The developers' guide records when to reach for
  each. Date/Author: 2026-11-20, planning agent.

## Outcomes & retrospective

Not started. This section is completed at each milestone boundary and at
completion.

## Context and orientation

Inspected base revision: `622b71d` ("Bump rust-toolchain from
nightly-2026-11-18 to nightly-2026-11-19 (#241)"). Line numbers below refer to
that revision; relocate symbols with `rg -n '<symbol>' src tests`.

### How an edit runs today

`src/main.rs` `main` calls `mapsplice::run_from_args`, then `emit_outcome` on
success (writes `RunOutcome::stdout` to standard output, treating a closed pipe
as success) or `report_error` on failure (emits `tracing::error!` with
`error_class`, then writes the error's `Display` text and a newline to standard
error through `report_stderr_diagnostic`, and exits with status 1).

`src/lib.rs` `run_request` reads the target, parses it with
`parse_roadmap_inner`, loads the optional fragment with `load_fragment`, calls
`apply_command_with_report`, renders with `render_roadmap_with_report`,
optionally writes in place with `rewrite_utf8`, records observability, and
returns `RunOutcome::stdout(rendered)` or `RunOutcome::in_place(path)`.
`RunOutcome` (`src/lib.rs`, about line 258) is
`pub struct RunOutcome { pub stdout: Option<String>, pub written_path:
Option<Utf8PathBuf> }`
deriving `Debug, Eq, PartialEq`, with private constructors `stdout`,
`in_place`, and `silent_success`. The export command builds its outcomes
through those constructors in `src/export_command/mod.rs`, so it needs no
change.

`src/roadmap/ops/mod.rs` `apply_command_with_report` clones the roadmap,
applies the splice, calls `renumber_document`, then `rewrite_dependencies`
(`src/roadmap/ops/rewrite.rs`), and returns `(u64, PreservationReport)`.
`rewrite_dependencies` holds a `DependencyRewriteContext` (plan, rewrite count,
unresolved anchors, first unresolved location) and walks these *dependency
regions*, in this order: the preamble; each phase's `title`, `body`, and
`trailing`; then each step's `title`, `body`, and `trailing`; then each task's
`summary`, `body`, and `body_children_mut()` blocks, then its sub-tasks'
`summary` and `body`. It calls
`rewrite_markdown_nodes(markdown, source, context)` for each region. Each
region is a `MarkdownNodes` value carrying the source text it was parsed from
(`MarkdownNodes::source`), and each item carries
`identity: ItemIdentity { source: SourceId, anchor: RoadmapAnchor }`, the
source document (`SourceId::Target` or `SourceId::Fragment`) and the anchor as
written there. After `renumber_document`, each item's current number is
`phase.number`, `step.number`, `task.number()`, or `sub_task.number`.

### How the scanner recognizes a dependency reference

`src/roadmap/ops/dependency_text.rs` (262 lines):

- `dependency_candidates(value)` scans with `next_anchor_candidate`, which
  finds a maximal token (a digit not preceded by an ASCII alphanumeric,
  greedily extended by `.digits` groups, and not followed by an ASCII
  alphanumeric) and resumes after it. It keeps the token when
  `classify_dependency_reference` returns `Reference(anchor)` (kept as
  `anchor: Some`) or `InvalidDependencyToken` (kept as `anchor: None`), and
  returns `Vec<DependencyCandidate { range, anchor }>`.
- `is_dependency_anchor` checks the keyword list `["Requires"]` through
  `latest_keyword_before`, which takes the *last* `Requires` substring before
  the token (`rmatch_indices`), then requires `is_keyword_boundary`,
  `has_dependency_clause_separator` (a colon or whitespace in the slice between
  keyword and token) and no `has_dependency_clause_terminator` in that slice (a
  `;`, or a `.` not between two digits *within the slice*).
- `has_section_sigil` rejects a token immediately preceded by `§`.
- `rewrite_text_value` returns a `DependencyRewriteReport` (new value, rewrite
  count, unresolved anchors, first unresolved range in the original value).
- `DependencyReferenceClassification` derives `Copy`.

`src/roadmap/ops/dependency_visitor.rs` (399 lines) is the shared traversal
used by rewriting and by JSON export collection. `visit_dependency_nodes`
groups adjacent `Text` and `Break` nodes, flattens each group with
`flatten_text_group` (a `Break` becomes one space), and records only candidates
lying wholly inside one text node (`candidate_text_segment`). A single text
node takes the fast path `visit_single_text`, which calls `rewrite_text_value`
in `Rewrite` mode. Containers are descended by `visit_container_node`,
`visit_inline_container_node`, and `visit_mdx_container_node`; inline code,
fenced code, HTML, and link destinations are leaves and are never scanned. The
visit modes are `DependencyVisitMode::Rewrite` and
`DependencyVisitMode::Collect`. Replacements in a group are applied after all
candidates are recorded (`apply_replacements`). `source_location_at`
(`src/roadmap/source_location.rs`) maps a byte range in a text node to a
one-based `SourceLocation { line, column }`, matching the n-th occurrence of
the token text inside the node's own source span.

### Tests, proofs, and gates available

- Unit tests: `rstest` 0.28.0 tables in
  `src/roadmap/ops/dependency_text_tests.rs`.
- Golden fixtures: `tests/roadmap_golden.rs` and
  `tests/roadmap_golden/nested_requires.rs`, driven by `tests/golden/`
  (`case.rs`, `runner.rs`, `assertions.rs`, `workspace.rs`, `format_gate.rs`).
  Fixture directories are
  `tests/fixtures/golden/<case>/{target.md, fragment.md, expected.md}`; the
  workspace root is an absolute temporary directory. There are 38 success
  fixtures; all contain only numeric `Requires` clauses. No fixture pins
  standard error today.
- Behavioural tests: `rstest-bdd` 0.6.0. Feature file
  `tests/features/mapsplice.feature`, scenarios bound in
  `tests/behaviour_cli.rs`, steps in `tests/steps/cli_steps.rs`
  (`CliState::run` captures `stdout`, `stderr`, and `success` from the real
  binary and currently inherits the ambient environment).
- Process tests: `tests/cli_process_contract.rs` (Linux only) runs the built
  binary with logging switched off (`RUST_LOG=off`), colour output disabled,
  the C locale, and an isolated `HOME`.
- Property tests: `proptest` 1.9.0. `tests/roadmap_dependency_properties.rs`
  generates roadmaps from a model (`tests/support/generation.rs`, 351 lines)
  and checks them against an oracle that never calls the production recognizer
  (`tests/support/oracle.rs`), 256 cases per property.
- Snapshots: `insta` 1.44.0, currently one inline snapshot in
  `tests/cli_ui.rs`.
- Proofs: Verus proves only the resolution kernel `select_resolution`
  (`verus/lib.rs`, `src/roadmap/ops/select_resolution.macro.rs`), run by
  `make verus` and `make verus-selftest`; the ledger is `docs/verification.md`,
  checked by `make check-verification-ledger`. The clause recognizer is
  explicitly outside every proof (`docs/verification.md`, "What is proved, and
  what is not"). No Kani harnesses exist.
- Gates: `make check-fmt`, `make lint` (verification ledger, domain purity,
  rustdoc and Clippy with `-D warnings`, Whitaker), `make test` (nextest plus
  doctests), `make markdownlint`, `make nixie`, `make spelling`. Continuous
  integration also runs `markdownlint-cli2` separately from `make check-fmt`.

### Documentation and skills to consult

Documentation: `docs/mapsplice-design.md` (sections 5–8 and 10, especially
section 7, "The dependency-reference model (normative)"),
`docs/developers-guide.md` (sections 2, 3, 6, 7, and 8.2),
`docs/users-guide.md` ("Help and diagnostics", the dependency-rewriting text
under the worked example, "Validation rules and failure cases", and
"Compatibility and migration"), `docs/validation-and-agent-output-design.md`
(finding severities and the phase 6 dependency pass), `docs/verification.md`,
`docs/migrations/0.2.0.md`, `docs/documentation-style-guide.md` (ADR format),
`docs/rust-testing-with-rstest-fixtures.md`, `docs/rstest-bdd-users-guide.md`,
`docs/rust-doctest-dry-guide.md`,
`docs/reliable-testing-in-rust-via-dependency-injection.md`,
`docs/complexity-antipatterns-and-refactoring-strategies.md`, and
`docs/ortho-config-users-guide.md` (no configuration change is planned; consult
it only to confirm that no new option is needed).

Skills: `rust-router` (then `rust-unit-testing` for `rstest`, `googletest`,
`pretty_assertions`, and `insta`; `proptest` for the property; `rust-errors` and
`rust-types-and-apis` for the warning type), `hexagonal-architecture` (domain
finding versus application warning versus adapter output), `execplans`,
`nextest`, and `en-gb-oxendict-style` for documentation.

## Conformance basis

Upstream artefacts, all at revision `622b71d`:

- `docs/roadmap.md`, item 1.2.3 and step 1.2.
- `docs/mapsplice-design.md`, sections 5 (F1–F5), 6 (C3, C6), 7 (dependency
  reference model), 8 (fixture requirements), and 10 (risks).
- `docs/validation-and-agent-output-design.md`, finding severities and the
  dependency pass (phase 6; not implemented).
- `docs/adrs/adr-0001-rust-build-standard.md` (build gates only).
- `AGENTS.md` (code style, testing, and documentation rules).

No Terms of Reference document exists for this repository.

Requirements:

- `REQ-warn`: an edit over a roadmap containing a `Requires` keyword that
  governs zero recognized anchor tokens reports a warning (roadmap 1.2.3).
- `REQ-name`: the warning names the item containing the clause (roadmap
  1.2.3).
- `REQ-succeed`: the edit still succeeds; output bytes and exit status are
  those of the same edit without the warning (roadmap 1.2.3; design F1, F2, C6).
- `REQ-pin`: a fixture pins the diagnostic text (roadmap 1.2.3; design
  section 8 regression discipline).
- `REQ-scope`: no change to which numbers are rewritten (design C3,
  section 7).
- `REQ-failclosed`: failures still report only the typed error (design F5).

Gaps:

- `GAP-silent`: a keyword with zero recognized tokens is silently inert.
- `GAP-channel`: no non-fatal diagnostic channel exists.

Trace:

```plaintext
roadmap 1.2.3 -> REQ-warn, GAP-silent -> EP-M2 -> dependency_text_tests::anchorless_cases, anchorless_enumeration
roadmap 1.2.3 -> REQ-warn, REQ-name -> EP-M3 -> rewrite_anchorless_tests::region_owners
roadmap 1.2.3 -> REQ-name, REQ-pin, GAP-channel -> EP-M4 -> c3_anchorless_requires_warning/expected_warnings.txt
roadmap 1.2.3 -> REQ-succeed -> EP-M4 -> golden zero-warning sweep, behaviour_cli anchor-free scenarios, cli_process_contract
design C3 -> REQ-scope -> EP-M1..EP-M4 -> existing dependency suites unchanged
design F5 -> REQ-failclosed -> EP-M4 -> behaviour_cli failure-silence scenarios
```

## Verification plan

### Abstract model

Let `t` be the flattened string of one text group visited by
`visit_dependency_nodes` (adjacent `Text` values concatenated, each `Break`
replaced by one space). Define:

- `K(t)`: the set of byte ranges of keyword occurrences, that is, occurrences
  of `Requires` in `t` whose preceding and following bytes (when present) are
  not ASCII alphanumeric.
- `C(t)`: the candidates returned by `dependency_candidates(t)`. Each
  candidate `c` has a range and an *owner* `own(c)`, the range of the last
  `Requires` substring that starts before `c` (the occurrence
  `latest_keyword_before` selects). Every candidate in `C(t)` has
  `own(c) ∈ K(t)`, because classification rejects a token whose latest keyword
  fails the boundary test.
- `V(t) ⊆ C(t)`: the candidates the visitor records, that is, those lying
  wholly inside one text node of the group. For a single-node group,
  `V(t) = C(t)`.
- `A(t) = K(t) \ { own(c) | c ∈ V(t) }`: the anchorless keyword occurrences.

For the staged document `D` after a successful splice and renumbering, the
rewrite pass visits each dependency region `r` with an owner `o(r)` (the
containing item's identity and post-edit number, or none for the preamble). The
pass produces, alongside its rewrites, one
`AnchorlessClause { owner: o(r), location }` for each element of `A(t)` of
every text group `t` it visits, where `location` is `source_location_at` of the
keyword range within its text node, computed before any replacement in that
node. The emitted list is then sorted by the D7 key.

Preconditions: `D` is the staged document after a successful operation and
renumbering. If parsing, the operation, or rewriting fails, no warning list
exists (the `Result` is `Err`).

### Invariants and lemmas

- `INV-exact` (establishes `REQ-warn`): for every visited text group `t`, the
  rewrite pass reports exactly `A(t)`: no anchorless keyword occurrence is
  missed and no owning keyword occurrence is reported.
- `L-owner-total`: every recorded candidate has exactly one owner, and that
  owner is in `K(t)`.
- `L-owner-agrees`: the owner returned with each candidate is the keyword
  occurrence that made `classify_dependency_reference` accept it, and adding
  the owner leaves the candidate list and its accept or reject decisions
  unchanged (`REQ-scope`).
- `L-neutralize` (the oracle's correctness premise): for `k ∈ K(t)`, replacing
  the eight bytes of `k` with eight `;` bytes removes exactly the candidates
  owned by `k` from `C(t)` and leaves every other candidate unchanged. Argument:
  `;` is not ASCII alphanumeric, so no token boundary next to `k` changes
  (both neighbours of `k` are already non-alphanumeric); a candidate owned by
  `k` now has its latest keyword either absent or earlier, with `;` (a
  terminator) in between, so it is rejected; a candidate owned by a later
  keyword never includes `k` in its separator slice; a candidate before `k` is
  unaffected. The lemma is checked mechanically by the enumeration below (both
  directions are asserted on every string).
- `INV-owner-label` (establishes `REQ-name`): each warning's owner is the
  innermost item whose region contained the keyword, with its post-edit number
  and its identity's source and spelling.
- `INV-non-interference` (establishes `REQ-succeed`, `REQ-scope`): collecting
  anchorless keywords does not alter rewritten text; standard output bytes,
  in-place file bytes, and exit status equal those of the same edit at the base
  revision.
- `INV-stream-separation` (establishes `REQ-succeed`): warnings appear only on
  standard error; standard output contains only the rendered roadmap.
- `INV-failure-silence` (establishes `REQ-failclosed`): a failed edit, or a
  successful edit whose standard-output write fails, emits no warning lines.
- `INV-order` (establishes `REQ-pin`): warnings are emitted in the D7 order.
- `INV-location` (establishes `REQ-name`): the reported line and column are
  the position of the keyword's `R` in the item's own source file, including
  when a token earlier in the same node is rewritten to a different length,
  with CRLF line endings, after a `PreRequires` substring in the same node, and
  for fragment items.

### Axioms (trusted, not verified here)

- `AXIOM-mdast`: the `markdown` crate produces `Text`, `Break`,
  `InlineCode`, `Code`, `Html`, `Strong`, `Emphasis`, `Link`, and table nodes
  with correct `Position` offsets into the parsed source, as relied on by every
  existing location test.
- `AXIOM-scanner`: `dependency_candidates` is the specified dependency
  recognizer; its decisions are pinned by roadmap items 1.1.1–1.2.2 and their
  tests. This plan verifies ownership and anchorlessness relative to it, not
  the recognizer itself.
- `AXIOM-stderr`: `std::io::stderr` writes complete lines in order for a single
  thread; the existing `write_stderr_line` helper is reused.

### Methods per obligation

- Obligation: `L-owner-total` and `L-owner-agrees`.
  Method: `rstest` partition table plus the unchanged existing table.
  Rationale: ownership is a finite set of syntactic situations. Domain: one
  keyword with one token; two keywords each with tokens; a token owned by the
  second of two keywords; a `PreRequires` substring between a keyword and a
  token; a terminator between; a colon separator; a malformed token; a section
  sigil. Artefact: `src/roadmap/ops/dependency_text_tests.rs`, new
  `candidate_owner_cases`. Evidence: red against the stub described in `EP-M2`
  (the stub reports the first keyword for every candidate, so the
  second-keyword cases fail on an assertion); green after. Every existing
  `dependency_text_tests` case passes unchanged. Non-vacuity: the
  second-keyword case rejects an implementation that always picks the first
  keyword.
- Obligation: `INV-exact` at the text level and `L-neutralize`.
  Method: `rstest` table for every D3 class, a bounded exhaustive enumeration,
  and a `proptest` property, all compared against a metamorphic oracle.
  Rationale: the behaviour depends on a small lexeme vocabulary, so bounded
  exhaustive enumeration covers every interleaving up to the bound; `proptest`
  extends coverage to longer strings with shrinking. The oracle treats
  `dependency_candidates` as a trusted black box (`AXIOM-scanner`) and copies
  none of the grammar, so it cannot drift from the scanner. Oracle:
  `oracle_anchorless(value)` returns the keyword ranges `k` (found by a plain
  boundary-checked substring search in the test) for which
  `dependency_candidates(neutralize(value, k))` is exactly
  `dependency_candidates(value)` with no element removed; for each anchored `k`
  it also asserts that only candidates after `k` were removed (`L-neutralize`).
  Domain: the eleven lexemes `Requires`, `PreRequires`, ` `, `:`, `.`, `;`, `-`,
  `§`, `none`, `1.2`, `1.4.0`. (`requires` and `\n` are covered by the table;
  a line ending is a space at text level.) Exhaustive enumeration covers every
  sequence of one to five lexemes: 11 + 121 + 1,331 + 14,641 + 161,051 =
  177,155 strings. The property draws sequences of up to 24 lexemes for 256
  cases. Artefact: `src/roadmap/ops/anchorless_enumeration_tests.rs`
  (`#[cfg(test)]` module beside `dependency_text_tests.rs`, with a `//!`
  comment stating that the oracle depends only on `dependency_candidates` and
  must not reimplement clause rules). Evidence: red against the stub (which
  returns no anchorless keywords), green after; the test prints and asserts the
  string count. Non-vacuity: the enumeration asserts that both anchored and
  anchorless outcomes occur, that at least 10,000 strings contain a keyword,
  and that the strings `Requires;Requires 1.2` (a token owned by the second
  keyword) and `Requires 1.2 Requires` (an anchorless keyword after an anchored
  one) are among those enumerated. The property runs through `TestRunner::run`
  with local `Cell` counters for "has keyword", "anchorless", and "anchored",
  and asserts each is non-zero after the run. Seeded faults, each applied once
  and reverted: (a) report every keyword occurrence; (b) skip the boundary test
  in `dependency_keywords`; (c) treat a keyword as anchored when *any*
  candidate exists in the string. Each must be rejected by the enumeration
  (fault (c) by `Requires;Requires 1.2`); transcripts go in
  `Artefacts and notes`.
- Obligation: `INV-exact` across text groups (`V(t)` versus `C(t)`) and
  `INV-owner-label`. Method: named `rstest` cases through
  `apply_command_with_report`. Domain: a keyword followed by a hard break and
  then an anchor (anchored); a keyword in `Strong` with the anchor outside
  (anchorless); the anchor in inline code; the anchor in a link; two keywords
  split by a hard break; and one anchorless clause in each region kind the
  rewriter visits (preamble; phase title, body, trailing; step title, body,
  trailing; task summary, continuation, body bullet; sub-task summary, body; a
  table cell; a fragment task inserted by the operation). Artefact:
  `src/roadmap/ops/rewrite_anchorless_tests.rs`, including `region_owners`.
  Evidence: red against the stub, green after; each region yields exactly one
  `AnchorlessClause` with the expected owner kind, post-edit number, and
  identity. Non-vacuity: the hard-break case rejects an implementation that
  scans each `Text` node separately; the `Strong` case rejects one that scans
  the paragraph's concatenated plain text; a twin document with a valid
  `Requires` in every region yields no warnings and is rewritten, proving the
  regions are live. `task.body` itself is drained into body children by the
  parser, so it is not separately reachable; that is recorded, not tested.
- Obligation: `INV-location`.
  Method: named regression cases. Domain: `Requires 1.1.9; Requires none.` where
  `1.1.9` becomes `1.1.10`; the same clause in a CRLF document;
  `PreRequires x; Requires none.`; a fragment item. Artefact:
  `src/roadmap/ops/rewrite_anchorless_tests.rs`. Evidence: each reported line
  and column equals the keyword's position in its own source file. Non-vacuity:
  a column computed from the rewritten value would be one greater in the first
  case; counting `Requires` substrings rather than boundary-valid keywords
  would pick the wrong occurrence in the third.
- Obligation: `INV-non-interference`, `INV-stream-separation`, `INV-order`,
  `REQ-pin`. Method: golden fixtures, a golden zero-warning sweep, behavioural
  scenarios, process tests, and an `insta` snapshot. Artefacts:
  `tests/fixtures/golden/c3_anchorless_requires_warning/` (`target.md`,
  `fragment.md`, `expected.md`, `expected_warnings.txt`), an insert-before case
  in which the anchorless target item is renumbered and the fragment adds
  `**Requires** 1.1.1.` to a new task; and
  `tests/fixtures/golden/c3_requires_outside_dependency_text_no_warning/`,
  whose target holds `Requires` only in fenced code, inline code, HTML, a link
  destination, lowercase prose, `PreRequires`, and `Requirements`. The golden
  runner (`tests/golden/runner.rs` and `assertions.rs`) compares
  `expected_warnings.txt`, with the workspace root replaced by `<workspace>`,
  when present, and asserts an empty warning list otherwise, so all 38 existing
  success fixtures become zero-warning regressions. The `insta` snapshot in a
  new `tests/roadmap_warnings.rs` captures every warning for the region-owner
  document. `tests/cli_process_contract.rs` gains one case asserting exact
  standard output and standard error bytes for the golden fixture run in a
  temporary directory with relative paths, and one case under `RUST_LOG=warn`
  asserting exactly one line in the D6 format plus a tracing line carrying
  `warning_code`. Evidence: the process case fails on assertion before `EP-M4`
  (standard error is empty), then passes. Non-vacuity: the fixture's expected
  output contains the anchor-free clause unchanged and a renumbered neighbour,
  proving the edit ran; the process test compares standard output with
  `expected.md` byte for byte.
- Obligation: `INV-failure-silence`.
  Method: behavioural scenarios across three failure kinds. Domain: a target
  with an anchor-free clause and a dangling reference; a level mismatch whose
  fragment contains an anchor-free clause; a missing anchor in a target with an
  anchor-free clause. Evidence: each command fails, standard error contains its
  error diagnostic and no warning line. A unit test in `src/main.rs` confirms
  warnings are not written when the injected standard-output writer fails.
  Non-vacuity: the same targets without the failure cause do warn (main
  scenario), so the absence is caused by the failure.
- Obligation: generated-document false-positive freedom and `INV-owner-label`
  at scale. Method: extend the generated-roadmap property model. Domain: the
  generator in `tests/support/generation.rs` and the oracle in
  `tests/support/oracle.rs`, 256 cases. Evidence: the model gains an optional
  `anchorless` flag on one generated task, rendered as a `- Requires none.`
  body bullet (rendering in a new `tests/support/anchorless_case.rs` if
  `generation.rs` would pass 400 lines); the oracle expects the bullet
  preserved verbatim and exactly one warning naming the task's post-edit number
  when the task survives the edit, and none when it is deleted or the flag is
  unset. Non-vacuity: the property asserts that the flag is set in a non-zero
  number of cases, that both surviving and deleted flagged tasks occur, and
  that every generated case without the flag contains at least one anchored
  clause.
- Obligation: dogfood.
  Method: manual acceptance check recorded in `Artefacts and notes`. Evidence:
  `cargo run -- insert --after docs/roadmap.md 1.2.3 <task fragment>` in
  preview writes no warning lines.

### Formal verification decision

No Verus proof and no Kani harness are planned (D8). The new contractual logic
is the ownership kernel `A(t) = K(t) \ own(V(t))`. Once the owner is returned
by the same function that accepts the candidate, `L-owner-agrees` holds by
construction, and a Verus proof of the set difference would restate its
definition. Unlike `select_resolution`, there is no separate decision rule to
prove. The substantive risk is in string scanning and Markdown traversal, which
Verus cannot reach without re-modelling `&str` scanning and `mdast` outside the
shipped code; a proof over such a model would leave the shipped recognizer
unverified, as `docs/verification.md` already records. Kani over string
allocation would explore no more than the bounded enumeration, which is
exhaustive over every lexeme sequence up to length five, reaches every
ownership situation (two keywords plus a token need five lexemes), and needs no
new tooling; `proptest` extends it beyond the bound. The existing Verus proofs
over `select_resolution` are untouched; `make verus` is not an acceptance
criterion because no proved function changes, and
`make check-verification-ledger` (part of `make lint`) still runs.
`docs/verification.md` gains a sentence in "What is proved, and what is not"
naming the enumeration as the evidence for keyword ownership. Revisit this
decision if implementation needs a new resolution or renumbering rule.

### Suggested BDD features

Add to `tests/features/mapsplice.feature`, bound in `tests/behaviour_cli.rs`,
with new steps in `tests/steps/cli_steps.rs` (`CliState::run` gains
`.env("RUST_LOG", "off")`):

```gherkin
  Scenario: Anchor-free Requires clause warns without blocking an in-place edit
    Given the target roadmap with an anchor-free Requires clause in task 1.1.2
    When I delete task 1.1.1 in place
    Then the command succeeds
    And stdout is empty
    And stderr warns that task 1.1.1 (was 1.1.2) has an anchorless Requires clause at line 10, column 5
    And the target file keeps the anchor-free clause under task 1.1.1

  Scenario: Anchor-free Requires clause warns while previewing to stdout
    Given the target roadmap with an anchor-free Requires clause in task 1.1.2
    When I append the phase fragment
    Then the command succeeds
    And stdout contains the anchor-free clause unchanged
    And stderr warns that task 1.1.2 has an anchorless Requires clause at line 10, column 5

  Scenario: Anchor-free Requires clause in a fragment names the fragment
    Given the target roadmap with one step and two tasks
    And the task fragment roadmap with an anchor-free Requires clause
    When I insert the task fragment after task 1.1.1
    Then the command succeeds
    And stderr warns about the fragment file for task 1.1.2 (was 1.1.1)

  Scenario: Roadmap with only anchored Requires clauses emits no warnings
    Given the target roadmap with scoped reference text
    When I delete phase 1
    Then the command succeeds
    And stderr is empty

  Scenario: Failed edit reports only the error, not anchor-free warnings
    Given the target roadmap with an anchor-free clause and a dangling dependency reference
    And the phase fragment roadmap
    When I try to append the phase fragment in place
    Then the command fails
    And stderr mentions dangling dependency anchor 99.1.1
    And stderr contains no warning lines
    And the target file remains unchanged

  Scenario: Level mismatch with an anchor-free fragment reports only the error
    Given the target roadmap with two phases
    And the task fragment roadmap with an anchor-free Requires clause
    When I try to insert the mismatched fragment before phase 2
    Then the command fails
    And stderr mentions the phase versus task mismatch
    And stderr contains no warning lines
```

Each scenario traces to `REQ-warn`, `REQ-name`, `REQ-succeed`, or
`REQ-failclosed`, and is delivered in `EP-M4`. Exact step wording may be
adjusted to the existing step vocabulary during implementation; the asserted
outcomes may not.

## Plan of work

### `EP-M0` feasibility probe

Question: does today's binary accept the D3 classes silently? Build once
(`cargo build`), write a scratch target under `/tmp/mapsplice-probe/`
containing the five clauses in `Purpose / big picture`, run
`target/debug/mapsplice delete /tmp/mapsplice-probe/target.md 2`, and observe
empty standard error and exit status 0. Record the transcript in
`Artefacts and notes`. No repository file changes. (Fragment source text and
identity spelling were confirmed by reading code during design review.)

### `EP-M1` split the dependency visitor without behaviour change

Requirements: none directly; enables `EP-M3` under the 400-line limit.

Create `src/roadmap/ops/dependency_text_group.rs` with a `//!` comment
explaining that it owns the flattening of adjacent text and hard-break nodes
and the mapping of flattened ranges back to their text nodes. Move
`TextSegment`, `flatten_text_group`, and `candidate_text_segment` into it.
Create `src/roadmap/ops/dependency_containers.rs` with a `//!` comment
explaining that it owns which Markdown containers the dependency visitor
descends into and which nodes are leaves. Move `visit_container_node`,
`visit_inline_container_node`, and `visit_mdx_container_node` into it. Give
moved items `pub(super)` visibility, declare both modules in
`src/roadmap/ops/mod.rs` (two lines), and change nothing else. No tests change.
This is a refactor commit on its own.

Acceptance: `make check-fmt`, `make lint`, and `make test` pass;
`wc -l src/roadmap/ops/dependency_visitor.rs` reports 340 lines or fewer.

### `EP-M2` keyword-ownership kernel

Requirements: `REQ-warn`, `REQ-scope`; obligations `L-owner-total`,
`L-owner-agrees`, `L-neutralize`, `INV-exact` (text level).

Stub first, so the red step fails on behaviour rather than compilation. In
`src/roadmap/ops/dependency_text.rs` add `keyword: Range<usize>` to
`DependencyCandidate` and fill it with `0..0`; add
`pub(crate) fn dependency_keywords(value: &str) -> Vec<Range<usize>>` and
`pub(crate) fn anchorless_dependency_keywords(value: &str, recorded:
&[DependencyCandidate]) -> Vec<Range<usize>>`,
both returning `Vec::new()`; update the export path's construction sites, if
any, mechanically. Run `make test` to confirm nothing else changed.

Red: add `candidate_owner_cases` and `anchorless_cases` (every D3 row) to
`src/roadmap/ops/dependency_text_tests.rs`, and the enumeration and property to
`src/roadmap/ops/anchorless_enumeration_tests.rs`. Use `rstest` for tables,
`googletest` matchers (`assert_that!`, `elements_are!`, `is_empty()`) for
collection assertions, and `pretty_assertions::assert_eq!` with a message for
range lists. Run `cargo nextest run -p mapsplice dependency_text anchorless`
and observe assertion failures (owner `0..0`, empty anchorless lists).

Green: thread the keyword start out of `latest_keyword_before` through
`is_dependency_anchor` and `classify_dependency_reference` (carry it as a
`usize` in the `Reference` and `InvalidDependencyToken` variants so the
classification stays `Copy`), and fill `DependencyCandidate::keyword` with
`start..start + "Requires".len()`. Implement `dependency_keywords` with the
existing `is_keyword_boundary`, and `anchorless_dependency_keywords` as the
keyword ranges not owned by any recorded candidate, ascending. Re-run the
focused tests to green.

Refactor: keep the file under 400 lines; extract a predicate if
`classify_dependency_reference` grows beyond one screen. Run the three gates.

Seeded-fault check: apply each fault named in `Verification plan` in turn,
observe the enumeration fail, revert with `git checkout --`, and record the
transcripts. Delete any `proptest-regressions` entries the faults create.

### `EP-M3` anchorless clauses collected by the rewrite pass

Requirements: `REQ-warn`, `REQ-name`; obligations `INV-exact` (group level),
`INV-owner-label`, `INV-location`.

Stub first: create `src/roadmap/ops/edit_report.rs` with a `//!` comment,
defining:

```rust
/// A `Requires` keyword that owns no recognized dependency token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AnchorlessClause {
    /// Containing item's identity, or `None` for the preamble.
    pub(crate) identity: Option<ItemIdentity>,
    /// Containing item's number after the edit, or `None` for the preamble.
    pub(crate) current: Option<RoadmapAnchor>,
    /// Keyword position in the item's own source document, when known.
    pub(crate) location: Option<SourceLocation>,
}

/// Outcome of one successful staged edit.
#[derive(Debug, Default)]
pub(crate) struct EditReport {
    /// Dependency references whose anchor changed.
    pub(crate) dependency_rewrites: u64,
    /// Source-preservation outcomes for observability.
    pub(crate) preservation: PreservationReport,
    /// Anchorless `Requires` keywords, in the D7 order.
    pub(crate) anchorless: Vec<AnchorlessClause>,
}
```

Change `apply_command_with_report` to return `Result<EditReport>` with an empty
`anchorless` list, and update its two callers (`apply_command` and
`run_request`). Run `make test`.

Red: create `src/roadmap/ops/rewrite_anchorless_tests.rs` with the cross-group
cases, `region_owners`, the twin document, and the `INV-location` cases,
building documents by parsing Markdown literals with `parse_roadmap_inner` and
`parse_fragment_inner` and applying operations through
`apply_command_with_report`. Observe assertion failures (empty lists).

Green:

1. In `src/roadmap/ops/dependency_visitor.rs`, add
   `anchorless: &'a mut Vec<Option<SourceLocation>>` to
   `DependencyVisitMode::Rewrite`. In `record_group_candidates`, after
   recording candidates and before `apply_replacements`, compute
   `anchorless_dependency_keywords(&flattened, &recorded)`, map each keyword
   range to its text segment with `candidate_text_segment`, and push
   `source_location_at(text, source_text, local_range)`. In
   `visit_single_text`, add `anchorless_keywords: Vec<Range<usize>>` to
   `DependencyRewriteReport` (computed in `rewrite_text_value` from the same
   candidate list) and push their locations before assigning the new value.
2. In `src/roadmap/ops/rewrite.rs`, add `anchorless: Vec<AnchorlessClause>` to
   `DependencyRewriteContext`, and replace the `source: SourceId` parameter of
   `rewrite_markdown_nodes` with a small `Copy` struct
   `ClauseOwner { identity: Option<ItemIdentity>, current:
   Option<RoadmapAnchor> }`
   from which the source is derived (`SourceId::Target` for the preamble).
   After each visit, drain the new locations into `AnchorlessClause` values
   tagged with the owner. Each call site passes the owner it already has: the
   preamble passes none; phase, step, task, body-child, and sub-task sites pass
   the item's `identity` and current number. `rewrite_dependencies` sorts the
   list by the D7 key and returns it with the count, so
   `apply_command_with_report` can fill `EditReport`.
3. Add a doc comment to `rewrite_dependencies` stating that every region it
   visits is also diagnosed, because diagnosis happens in the same traversal.

Refactor and gates as before. `make check-domain-purity` must pass and
`src/roadmap/ops/mod.rs` must stay at or below 393 lines.

### `EP-M4` warnings surfaced through `RunOutcome` and standard error

Requirements: `REQ-name`, `REQ-succeed`, `REQ-pin`, `REQ-failclosed`;
obligations `INV-non-interference`, `INV-stream-separation`,
`INV-failure-silence`, `INV-order`.

Red: add the two golden cases, the runner's warning comparison and zero-warning
sweep, the process cases, the BDD scenarios and steps, the `insta` snapshot,
and the generated-property extension. Before production changes, the process
case and BDD warning steps fail on assertions (standard error is empty); the
golden and snapshot tests fail to compile on `RunOutcome::warnings`, which is
acceptable because the behavioural reds above already prove the missing
behaviour.

Green:

1. Create `src/warning.rs` with a `//!` comment, defining:

   ```rust
   /// Category of a non-fatal diagnostic.
   #[derive(Clone, Copy, Debug, Eq, PartialEq)]
   #[non_exhaustive]
   pub enum WarningKind {
       /// A `Requires` keyword governs no recognized dependency anchor token.
       AnchorlessDependencyClause,
   }

   /// A non-fatal diagnostic reported alongside a successful command.
   #[derive(Clone, Debug, Eq, PartialEq)]
   pub struct MapspliceWarning {
       kind: WarningKind,
       path: Utf8PathBuf,
       item: Option<RoadmapAnchor>,
       original_item: Option<RoadmapAnchor>,
       location: Option<SourceLocation>,
   }
   ```

   with `WarningKind::code(self) -> &'static str` returning
   `"mapsplice::dependency::anchorless_clause"`; public accessors `kind`,
   `path`, `item`, `original_item`, and `location`; a `pub(crate)` constructor;
   and a `Display` implementation producing the full D6 line. Include a doctest
   showing the display text of a warning obtained from `run_request` on a small
   roadmap written to a temporary directory, or, if that is too heavy for a
   doctest, documented with a `text` example and covered by unit tests.
   Re-export both types from `src/lib.rs` beside `MapspliceError`.
2. In `src/lib.rs`, add `#[non_exhaustive]` and
   `pub warnings: Vec<MapspliceWarning>` to `RunOutcome` with a doc comment;
   initialize it empty in `stdout`, `in_place`, and `silent_success`; add a
   private `with_warnings(self, warnings)`. In `run_request`, keep the fragment
   path, and after the in-place write (or after rendering in preview), map each
   `AnchorlessClause` from `EditReport` to `MapspliceWarning`, choosing
   `request.target` for `SourceId::Target` (and the preamble) and the fragment
   path for `SourceId::Fragment`. Put the mapping in a small named function so
   `run_request` stays short. If `src/lib.rs` would pass 380 lines, move
   `RunOutcome` into `src/run_outcome.rs` first.
3. In `src/main.rs`, change `emit_outcome` to write standard output first and
   then, when that returns success, write each warning with
   `report_stderr_diagnostic(&warning)` after
   `tracing::warn!(warning_code = warning.kind().code(), "mapsplice command warning")`.
   Add unit tests beside `diagnostic_tracing_tests` asserting the traced code,
   that the event omits the path, and that warnings are not written when the
   injected standard-output writer fails with an error other than a closed pipe.
4. In `tests/steps/cli_steps.rs`, add `.env("RUST_LOG", "off")` to
   `CliState::run`, the new targets as constants in `tests/support/cli.rs`, and
   the steps named in the BDD features. Warning steps compare the whole line
   after replacing the workspace root with `<workspace>`.

Refactor: run the three gates.

Dogfood: run the preview insert on `docs/roadmap.md` and record that standard
error is empty.

### `EP-M5` documentation, ADR, and roadmap closure

1. Add `docs/adrs/adr-0002-non-fatal-edit-warnings.md` (about 80 lines; status
   `Proposed` until the pull request is approved, then `Accepted` with the
   date), following the ADR section order in
   `docs/documentation-style-guide.md`: context (fail-closed tool, silent inert
   clauses), decision (D1, D4, D5, D6, D14), consequences (standard error may
   carry output on success; export divergence D12; phase 6 absorbs
   `MapspliceWarning` into its findings as `warning` severity with the same
   code), and rejected alternatives (fail the edit; tracing only; validate
   only; a separate diagnostic walk).
2. `docs/mapsplice-design.md`: in section 7 add a bullet "Anchorless clauses"
   stating D1–D3 and D11 normatively, that they are reported as warnings, not
   errors, and the export divergence (D12); in section 6 C6 state that warnings
   go to standard error after standard output in both output modes and never
   alter output or exit status; in section 5 note that a warning is not a
   failure under F5; add a Table 2 row "Anchorless `Requires`" proving "warned,
   preserved, edit succeeds"; in section 10 record the partially hidden clause
   limitation (`Requires 1.2.1 and **1.2.2**`); reference the ADR.
3. `docs/users-guide.md`: add `### Warnings` under "Help and diagnostics" with
   the exact line format, the meaning of "was" and of the coordinates after
   `--in-place`, the D7 order, an example, and the instruction not to merge
   standard error into the output; extend the dependency-rewriting text with
   the anchorless rule, how to fix each D3 class (wrap the *keyword* in inline
   code or reword when the clause is deliberately not a dependency; never
   un-quote deliberately quoted numbers), and that malformed-token clauses such
   as `Requires 1.4.0` remain silent until roadmap item 6.1.2; note in
   "Validation rules and failure cases" that warnings are non-fatal; add a
   "Compatibility and migration" subsection; align the existing note that
   `export --check` writes to standard error.
4. `docs/migrations/0.2.0.md`: record `RunOutcome` becoming
   `#[non_exhaustive]` with a `warnings` field, `MapspliceWarning` and
   `WarningKind`, and that successful edits may now write to standard error.
5. `docs/developers-guide.md`: section 2 (domain `AnchorlessClause` in
   `EditReport` → application `MapspliceWarning` → binary output), section 3
   (public `RunOutcome::warnings`, `MapspliceWarning`, `WarningKind`), section
   7 (the bounded enumeration and the metamorphic oracle, and when to use
   `googletest` and `pretty_assertions`), and a re-use policy:
   `MapspliceWarning` is the only user-facing warning type; new warnings are new
   `WarningKind` variants produced as domain data and mapped in `src/lib.rs`;
   domain code never formats paths or writes output; anchorless detection lives
   only in the rewrite traversal.
6. `docs/validation-and-agent-output-design.md`: note that the edit-time
   warning exists with code `mapsplice::dependency::anchorless_clause`, and
   that the phase 6 dependency pass should report the same condition as a
   `warning`-severity finding with that code.
7. `docs/verification.md`: add the sentence described in "Formal verification
   decision"; add no ledger row.
8. `skill/mapsplice/SKILL.md`: tell agents that warnings may appear on standard
   error after a successful edit, that streams must not be merged, and how to
   fix each class.
9. `README.md`: add a feature bullet for anchorless-clause warnings.
10. `docs/roadmap.md`: mark 1.2.3 `[x]` with an "Addressed:" note like 1.2.1.
11. Run `make fmt`, then `make check-fmt`, `make markdownlint`, `make nixie`,
    `make spelling`, `make lint`, and `make test`.

## Milestones and plateaus

- `EP-M1`: the visitor is split; behaviour identical. Requirements: none.
  Evidence: full suite green; visitor at or below 340 lines. Conformance:
  design unchanged; no interface or dependency change. Recovery: revert the
  single commit. Remaining gaps: all behaviour. Compatibility decision: none.
- `EP-M2`: the scanner reports owners and anchorless keywords for a string.
  Requirements: `REQ-scope` (held), `REQ-warn` (text level). Evidence:
  ownership and anchorless tables, enumeration, and property green; seeded
  faults rejected. Conformance: recognizer decisions unchanged
  (`dependency_text_tests` untouched in meaning); no public change. Recovery:
  revert the milestone commits. Remaining gaps: document level and output.
  Compatibility decision: none (`DependencyCandidate` is crate-private).
- `EP-M3`: a successful edit produces an ordered list of anchorless clauses.
  Requirements: `REQ-warn`, `REQ-name` (data). Evidence: cross-group,
  region-owner, twin, and location tests green; domain purity green.
  Conformance: one traversal for rewriting and diagnosis; output unchanged.
  Recovery: revert. Remaining gaps: user-visible output. Compatibility
  decision: none (`apply_command_with_report` is crate-private).
- `EP-M4`: users see warnings. Requirements: all `REQ-*`. Evidence: golden
  fixtures with `expected_warnings.txt`, golden zero-warning sweep, BDD
  scenarios, process tests, snapshot, generated property, dogfood. Conformance:
  F1–F5 and C1–C6 hold; only the authorized public API additions. Recovery:
  revert. Remaining gaps: documentation. Compatibility decision: none (pre-1.0,
  unreleased; migration note only).
- `EP-M5`: documentation, ADR, and roadmap reflect the behaviour. Evidence:
  Markdown, Mermaid, spelling, and code gates green. Recovery: revert
  documentation commits. Remaining gaps: none. Compatibility decision: none.

## Concrete steps

Working directory: the repository root of this worktree. Run gates
sequentially, through `tee`, with `pipefail`:

```bash
set -o pipefail
BRANCH=$(git branch --show-current)
make check-fmt 2>&1 | tee /tmp/check-fmt-mapsplice-$BRANCH.out
make lint 2>&1 | tee /tmp/lint-mapsplice-$BRANCH.out
make test 2>&1 | tee /tmp/test-mapsplice-$BRANCH.out
```

Documentation milestones add:

```bash
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-$BRANCH.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-$BRANCH.out
make spelling 2>&1 | tee /tmp/spelling-mapsplice-$BRANCH.out
```

Focused commands per milestone:

```plaintext
EP-M1 green:  cargo nextest run -p mapsplice dependency                 -> all pass, counts unchanged from base
EP-M2 red:    cargo nextest run -p mapsplice dependency_text anchorless -> assertion failures (owner 0..0, empty lists)
EP-M2 green:  same command                                               -> all pass; enumeration reports 177155 strings
EP-M3 red:    cargo nextest run -p mapsplice rewrite_anchorless           -> assertion failures (empty anchorless list)
EP-M3 green:  same command                                               -> all pass
EP-M4 red:    cargo nextest run -p mapsplice --test cli_process_contract --test behaviour_cli
              -> stderr assertions fail (standard error is empty)
EP-M4 green:  cargo nextest run -p mapsplice --test cli_process_contract --test behaviour_cli \
                --test roadmap_golden --test roadmap_warnings --test roadmap_dependency_properties
              -> all pass
```

Rust builds pass through the shared `build-limits-rustc` admission queue; a
queued build is not hung. Do not run gates in parallel, and do not create a
separate Cargo cache.

Commit after each milestone (and after each green step where useful) with an
imperative subject and a body explaining what and why.

## Validation and acceptance

Behavioural acceptance, from the repository root after `EP-M4`:

```plaintext
$ printf '%s\n' '# Roadmap' '' '## 1. One' '' '### 1.1. Step' '' \
    '- [ ] 1.1.1. First.' '- [ ] 1.1.2. Second.' '' '  - Requires none.' > /tmp/r.md
$ target/debug/mapsplice delete /tmp/r.md 1.1.1
# stdout: the roadmap with task 1.1.2 renumbered to 1.1.1 and "Requires none." unchanged
# stderr: /tmp/r.md:10:5: warning: task 1.1.1 (was 1.1.2): `Requires` clause has no recognizable dependency anchors
$ echo $?
0
```

Quality criteria:

- Tests: `make test` passes; the new tests named in `Verification plan` fail
  for the stated reason before their milestone's green step and pass after;
  every existing golden success fixture asserts zero warnings.
- Verification: `L-owner-total`, `L-owner-agrees`, `L-neutralize`,
  `INV-exact`, `INV-owner-label`, `INV-location`, `INV-non-interference`,
  `INV-stream-separation`, `INV-failure-silence`, and `INV-order` are each
  discharged by the artefacts named above; seeded-fault transcripts are
  recorded.
- Lint: `make lint` passes, including domain purity and the verification
  ledger.
- Formatting: `make check-fmt` and `make markdownlint` pass.
- Documentation: `make nixie` and `make spelling` pass.

## Idempotence and recovery

Every step is a source edit followed by a test run, so it can be repeated. Each
milestone ends in a commit that is a safe rollback point. Seeded faults are
applied to the working tree only, never committed, and reverted with
`git checkout -- <file>`; remove any `*.proptest-regressions` entries they
produce. The `EP-M0` probe writes only under `/tmp/mapsplice-probe/`. If a gate
fails, read its `/tmp` log before re-running it.

## Artefacts and notes

Reconnaissance observations at `622b71d` (planning time):

- `rg -n 'warn!' src` returns only a comment in
  `src/roadmap/ops/remap_kernel.rs`.
- `wc -l src/roadmap/ops/dependency_visitor.rs` reports 399.
- `rg -n 'Requires' docs/roadmap.md | rg -v 'Requires [0-9]'` shows every
  prose mention wrapped in inline code, so the dogfood check is expected to
  print no warnings.
- A design reviewer surveyed roadmaps in sibling repositories on the planning
  host and found at most four would-be warnings per roadmap, of forms such as
  `Requires no earlier task.`, `Requires the merged EP-M1…`, and
  ``Requires archived tasks `1.2.2` and `1.3.2` ``; these motivated the users'
  guide advice in `Risks`.

Implementation transcripts (probe, red and green evidence, seeded faults,
dogfood) are added here as milestones complete.

## Interfaces and dependencies

At the end of `EP-M4` these exist:

- In `crate::roadmap::ops::dependency_text`: the crate-private
  `DependencyCandidate { range, anchor, keyword }`, plus the `pub(crate)`
  functions `dependency_keywords` and `anchorless_dependency_keywords`, and
  `DependencyRewriteReport::anchorless_keywords`.
- In `crate::roadmap::ops::dependency_visitor`: an `anchorless` sink on
  `DependencyVisitMode::Rewrite` (crate-private).
- In `crate::roadmap::ops::edit_report`: `AnchorlessClause` and `EditReport`
  (crate-private), returned by `apply_command_with_report`.
- `mapsplice::MapspliceWarning` (public struct, private fields, accessors,
  `Display`) and `mapsplice::WarningKind` (public, `#[non_exhaustive]`, with
  `code()`).
- `mapsplice::RunOutcome` (now `#[non_exhaustive]`) with the public field
  `warnings: Vec<MapspliceWarning>`.

Reuse policy: `dependency_candidates` remains the only recognizer of dependency
tokens and the only source of ownership; `visit_dependency_nodes` remains the
only Markdown traversal for dependency text, and anchorless detection happens
only inside its `Rewrite` mode; no module re-parses Markdown or re-implements
keyword matching. `MapspliceWarning` is the only public warning type,
constructed only in `src/lib.rs`.

Dependencies: add to `[dev-dependencies]` in `Cargo.toml`:

```toml
googletest = "0.14.4"
pretty_assertions = "1.4.1"
```

No runtime dependency changes.

## Revision note

- 2026-11-20: initial draft from Wyvern reconnaissance at `622b71d`.
- 2026-11-20: applied the community-of-experts design review (structure and
  contracts; failure modes and verification; alternatives and viability; all
  three verdicts were "revise" or "proceed with conditions"). Changes:
  anchorless keywords are now collected inside the existing rewrite pass (D4),
  removing the separate query, its mirrored region walk, and the parity test;
  warnings name the item by its post-edit number with a "was" number, and use
  the GNU `file:line:column: warning:` layout produced by a single `Display`
  (D6); explicit document-order sorting replaces traversal order (D7);
  `MapspliceWarning` becomes a struct with a `#[non_exhaustive]` kind and a
  namespaced code, and `RunOutcome` becomes `#[non_exhaustive]` (D14); warnings
  are written after standard output succeeds; the forward-scanning oracle,
  which disagreed with the scanner on `Requires §1.2`, `Requires 1.2none`, and
  `Requires none1.2`, is replaced by a metamorphic keyword-neutralization
  oracle, and the enumeration bound rises to five lexemes so ownership cases
  and seeded fault (c) are reachable; red steps use stubs so they fail on
  assertions; behavioural tests pin `RUST_LOG=off`; the generated-property
  extension moves into the model; a no-warning fixture covers keywords outside
  dependency text; failure silence is tested across three failure kinds;
  D11–D15 record scope, export divergence, library reach, the warning type
  shape, and the test-library choice; documentation now also covers `README.md`,
  `docs/verification.md`, the merged-stream hazard, and the partially hidden
  clause limitation. The milestone order is unchanged; the production budget
  falls from 500 to 450 lines.

[^gnu]: GNU Coding Standards, section 4.4 "Formatting Error Messages",
    <https://www.gnu.org/prep/standards/html_node/Errors.html>.
