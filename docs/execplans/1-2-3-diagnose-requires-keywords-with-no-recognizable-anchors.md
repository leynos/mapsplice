# 1.2.3 Diagnose `Requires` keywords with no recognizable anchors

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`,
`Decision log`, `Outcomes & retrospective`, `Conformance basis`, and
`Verification plan` must be kept up to date as work proceeds.

Status: DRAFT (awaiting design review and user approval)

Branch: `1-2-3-diagnose-requires-keywords-with-no-recognizable-anchors`

Roadmap item: `1.2.3` in `docs/roadmap.md`.

Implementation must not begin until the user explicitly approves this plan.

## Purpose / big picture

`mapsplice` is a command-line tool that makes structural edits (append,
insert, delete, replace) to numbered Markdown roadmaps such as
`docs/roadmap.md`. After an edit it renumbers every phase, step, task, and
addendum sub-task, and it rewrites *dependency references*: the anchor numbers
written after the keyword `Requires`, such as the `1.2.2` in
`- Requires 1.2.2.`. Only numbers in that dependency context are rewritten;
section references, versions, and prose numbers are left alone.

The scanner that finds dependency references works number-first. It looks for
anchor-shaped tokens (`8`, `8.2`, `8.2.3`, `8.2.3.1`) and then looks backwards
for a `Requires` keyword that governs them. A `Requires` keyword that governs no
recognizable token therefore leaves no trace at all. Today all of these are
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
for each `Requires` keyword that governs zero recognized anchor tokens, naming
the input file, the source line and column of the keyword, and the item that
contains it. The edit still succeeds, standard output still carries exactly the
rendered roadmap, `--in-place` still rewrites the target, and the exit status is
still zero. For example:

```plaintext
$ mapsplice delete roadmap.md 2 > new.md
warning: roadmap.md: line 9, column 5: task 1.2.3: `Requires` clause has no recognizable dependency anchors
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
  `has_section_sigil`) keeps its current accept and reject decisions. Every
  existing test in `src/roadmap/ops/dependency_text_tests.rs`,
  `tests/roadmap_golden.rs`, `tests/roadmap_golden/nested_requires.rs`, and
  `tests/roadmap_dependency_properties.rs` must pass unchanged in meaning.
- Preserve every guarantee in `docs/mapsplice-design.md` sections 5, 6, and 7
  (F1–F5, C1–C6, and the normative dependency-reference model). A warning is
  not a failure: it must never change standard output bytes, in-place file
  bytes, or the exit status of a successful edit (see `INV-non-interference`).
- Preserve F5 (fail closed). When an edit fails, report only the existing error.
  Warnings are computed only after the operation, rendering, and any in-place
  write have succeeded.
- Standard output carries only the rendered roadmap. Warnings go to standard
  error and nowhere else.
- Domain purity: code under `src/roadmap/` must stay free of filesystem,
  process, environment, and terminal access (`scripts/check-domain-purity.sh`,
  `make check-domain-purity`). The domain returns warnings as data; only
  `src/lib.rs` maps a source document to a file path, and only `src/main.rs`
  writes to standard error.
- No code file may exceed 400 lines (`AGENTS.md`). At the base revision
  `src/roadmap/ops/dependency_visitor.rs` is 399 lines and
  `src/roadmap/ops/mod.rs` is 383 lines, so new behaviour must not be added to
  either without first moving code out (see `EP-M1`).
- Every module starts with a `//!` comment; every `assert!`, `assert_eq!`, and
  `assert_ne!` carries a diagnostic message (`clippy::missing_assert_message`
  is denied in `Cargo.toml`); production code and shared fixtures do not use
  `expect` or `unwrap`.
- Authorized dependency changes: add `googletest = "0.14.4"` and
  `pretty_assertions = "1.4.1"` to `[dev-dependencies]` in `Cargo.toml`, with
  caret semantics. No other dependency may be added. No runtime dependency may
  be added.
- Authorized public API changes (the crate is version `0.1.0`, pre-1.0, with no
  release tag, so no source-compatibility machinery is required or permitted):
  add a `warnings: Vec<MapspliceWarning>` field to `mapsplice::RunOutcome`, and
  add the public type `mapsplice::MapspliceWarning` re-exported from the crate
  root. Every constructor and caller of `RunOutcome` is updated in the same
  change. `mapsplice::apply_command` keeps its signature.
- Do not modify completed ExecPlans under `docs/execplans/` other than this
  file.
- `typos.toml` is generated; put any spelling exceptions in `typos.local.toml`.
- Follow the gate order in `AGENTS.md`: `make check-fmt`, `make lint`, and
  `make test` run sequentially; never in parallel with each other for this
  worktree.

If satisfying the objective requires violating a constraint, stop, record the
conflict in `Decision log`, set the status to `BLOCKED`, and escalate.

## Tolerances (exception triggers)

The comparison base is `origin/main` at `622b71d` (the base revision of this
plan). Sizes are net lines (additions minus deletions) counted with:

```bash
git diff --numstat 622b71d -- . ':(exclude)docs/execplans/*' ':(exclude)Cargo.lock'
```

ExecPlan files, including this file, are excluded from every count.
`Cargo.lock` is generated by Cargo and excluded from the line counts, but any
lockfile change other than the entries pulled in by the two authorized
dev-dependencies is a dependency tolerance breach. `typos.toml` is generated
and excluded likewise.

Classification is whole-file by purpose:

- Code: everything under `src/` and `tests/`, including Markdown and text
  fixtures under `tests/fixtures/`, `tests/features/`, and snapshot files,
  because they are executable test specification.
- Configuration: `Cargo.toml`, `typos.local.toml`, and `.github/`.
- Documentation: everything under `docs/` except `docs/execplans/`, plus
  `README.md` and `skill/`.

Budget survey. The closest precedent is roadmap item 1.2.1 (commit `a566a4d`,
pull request #86), which changed 69 files with 6,876 insertions. That change
also bootstrapped the Verus prover harness, the generated-roadmap oracle, and
the dependency property suite, so most of its size was approved
infrastructure, not missed scope; this plan reuses that infrastructure. The
recent refactor plans `docs/execplans/roadmap-5-1-4.md` (production budget 180
lines) and `docs/execplans/roadmap-4-4-4.md` (production budget 80 lines)
recorded no budget increases. No comparable plan records a repeatedly raised
budget. The estimate below traces every affected file listed in
`Context and orientation`: about 330 production lines (scanner kernel 50,
visitor mode 50, region walk 120, warning type 70, application and binary
wiring 40), about 750 test lines (unit tables 110, bounded enumeration and
oracle 260, golden harness extension 40, fixtures 120, behavioural steps and
features 90, process and snapshot tests 80, property extension 50), and about
330 documentation lines (ADR 130, design 60, users' guide 60, developers' guide
40, migration note 20, validation design 10, skill 10).

- Files: stop and escalate if more than 40 non-ExecPlan files change.
- Code: stop and escalate if code changes exceed 1,400 net lines in total, or
  if production code under `src/` alone exceeds 500 net lines.
- Configuration: stop and escalate if configuration changes exceed 25 net
  lines.
- Documentation: stop and escalate if documentation changes exceed 500 net
  lines.
- Interface: stop and escalate if any public API change beyond the two
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
  class of clause listed in `Decision log` D2 or D3 should behave differently.

Each category is enforced independently; spare capacity in one category does
not offset an overrun in another.

## Risks

- Risk: the warning fires on prose that legitimately starts with the word
  `Requires` outside inline code (for example, "Requires careful thought").
  Severity: low. Likelihood: medium. Mitigation: the warning is non-fatal and
  follows the existing case-sensitive keyword rule exactly, so the same prose
  is already treated as a dependency context for any number that follows it.
  The users' guide explains the fix (wrap the word in inline code or rephrase).
  `docs/roadmap.md` already writes every prose mention as `` `Requires` ``, and
  a dogfood acceptance check proves it yields zero warnings.
- Risk: the diagnostic traversal drifts from the rewrite traversal, so a region
  the rewriter scans is not diagnosed or vice versa. Severity: high.
  Likelihood: medium. Mitigation: the warning reuses the shared
  `visit_dependency_nodes` container traversal through a new visit mode, so
  node-level coverage is shared by construction; the item-level region walk is
  pinned by a region-parity test (`L-region-parity`) that places a clause in
  every region `rewrite_dependencies` visits.
- Risk: source coordinates drift after dependency rewriting changes the length
  of an earlier token in the same text node (for example, `1.2.9` becoming
  `1.2.10` before an anchor-free keyword). Severity: medium. Likelihood: low.
  Mitigation: locate the keyword with the existing
  `source_location_at`, which matches the n-th occurrence of the token text in
  the node's own source span, so length changes elsewhere do not move it. A
  named regression fixture pins this case.
- Risk: splitting `src/roadmap/ops/dependency_visitor.rs` to make room changes
  behaviour. Severity: medium. Likelihood: low. Mitigation: `EP-M1` is a
  behaviour-preserving move committed on its own, gated by the full existing
  suite before any new behaviour lands.
- Risk: adding a public field to `RunOutcome` breaks struct-literal
  construction by library consumers. Severity: low. Likelihood: low.
  Mitigation: the crate is pre-1.0 and unreleased; the migration note in
  `docs/migrations/0.2.0.md` records the change.
- Risk: scripts that treat any standard-error output as failure start failing
  on warnings. Severity: medium. Likelihood: low. Mitigation: exit status
  remains zero; the migration note and users' guide tell automation to rely on
  the exit status, and that warnings never appear on standard output.
- Risk: the edit-time warning and the planned phase 6 `validate` finding for
  the same condition diverge in naming. Severity: low. Likelihood: medium.
  Mitigation: the warning carries a stable class string
  `anchorless_dependency_clause`, recorded in
  `docs/validation-and-agent-output-design.md` for phase 6 to adopt as its
  finding code suffix.

## Progress

- [x] (2026-11-20 10:40Z) Branch renamed to
  `1-2-3-diagnose-requires-keywords-with-no-recognizable-anchors`, tracking
  `origin/1-2-3-diagnose-requires-keywords-with-no-recognizable-anchors`.
- [x] (2026-11-20 11:30Z) Reconnaissance completed with a Wyvern agent team
  (scanner, diagnostics plumbing, test and proof infrastructure, design and
  plan history).
- [x] (2026-11-20 12:30Z) Initial draft written.
- [ ] Community-of-experts design review applied.
- [ ] Plan approved by the user.
- [ ] `EP-M0` feasibility probe recorded.
- [ ] `EP-M1` dependency visitor split without behaviour change.
- [ ] `EP-M2` keyword-ownership kernel with unit, bounded, and property
  evidence.
- [ ] `EP-M3` anchorless-clause query over a roadmap document.
- [ ] `EP-M4` warnings surfaced through `RunOutcome` and standard error.
- [ ] `EP-M5` documentation, ADR, and roadmap closure.

## Surprises & discoveries

- Observation: no non-fatal warning channel exists anywhere in the product.
  Evidence: `RunOutcome` in `src/lib.rs` has only `stdout` and `written_path`;
  `src/main.rs` writes to standard error only on failure; no `warn!` call
  exists under `src/`. Impact: this item introduces the first warning channel,
  which phase 6 will extend; see `Decision log` D5 and the ADR in `EP-M5`.
- Observation: `tracing` is initialized with
  `EnvFilter::from_default_env()`, whose default directive is `error`, so a
  `tracing::warn!` event is invisible unless `RUST_LOG` is set. Evidence:
  `init_tracing` in `src/main.rs`. Impact: the user-visible warning must be
  written through `report_stderr_diagnostic`, not through `tracing`.
- Observation: an existing unit test already pins `Requires none.` as producing
  no dependency candidates. Evidence: `src/roadmap/ops/dependency_text_tests.rs`
  case `Count 27. Requires none.`. Impact: that expectation stays true; the
  warning is computed by a separate query and does not change candidates.
- Observation: `src/roadmap/ops/dependency_visitor.rs` is 399 lines, one under
  the repository limit. Impact: `EP-M1` must move code out before adding the
  diagnostic visit mode.
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
  warning must be derived from the scanner's own ownership decision rather
  than from a second grammar. A separately specified clause grammar would
  create two definitions of a dependency clause that could disagree.
  Date/Author: 2026-11-20, planning agent.
- D2. Decision: a clause whose only tokens are malformed dependency tokens,
  such as `Requires 1.4.0`, is *not* anchorless and does not warn. Rationale:
  the scanner does recognize `1.4.0` as an anchor-shaped dependency token
  (`DependencyCandidate { anchor: None }`), and roadmap item 1.1.1 deliberately
  preserves it. Warning about malformed tokens in dependency context is the
  separate phase 6 rule in `docs/validation-and-agent-output-design.md`
  ("version-like or invalid dependency tokens are warnings"), owned by roadmap
  item 6.1.2. Rejected option: treating "zero *valid* anchors" as anchorless,
  which would fold 6.1.2 scope into this item.
  Date/Author: 2026-11-20, planning agent.
- D3. Decision: these are anchorless and warn: `Requires none.`,
  `Requires: TBD`, a bare `Requires` or `Requires.` at the end of a paragraph,
  `Requires §1.2`, `Requires-1.2`, a keyword whose number sits in a different
  inline group (`**Requires** 1.2.1`, ``Requires `1.2.1` ``,
  `Requires [1.2.1](#x)`), and `Requires none; see 1.2.1` (the semicolon ends
  the clause). These do not warn: lowercase `requires`, `PreRequires`,
  `Requirements`, any keyword inside inline code, fenced code, or HTML (the
  visitor never enters them), and a clause with at least one valid or
  malformed token. Rationale: each follows mechanically from D1 and the
  existing scanner; the inline-group cases are precisely the "malformed or
  unparsable" clauses the roadmap item exists to surface.
  Date/Author: 2026-11-20, planning agent.
- D4. Decision: diagnose the edited document after the operation succeeds, not
  the inputs before it. Items are named by their identity (source document and
  the anchor as written in that document), and locations are source
  coordinates in that document. Rationale: the warning concerns clauses the
  user is about to keep; a clause in a deleted item is no longer a hazard.
  Post-operation scanning also covers fragment items without a second walk over
  the four `RoadmapFragment` variants. Identity and source coordinates both
  refer to the file as the user wrote it, so the line, column, and item number
  agree with each other. Rejected option: scanning target and fragment before
  the operation, which warns about deleted and replaced items and needs a
  separate fragment walk. Date/Author: 2026-11-20, planning agent.
- D5. Decision: introduce a minimal warning channel. The domain returns
  `AnchorlessClause` values; `src/lib.rs` maps each to a public
  `MapspliceWarning` carrying the file path; `RunOutcome` gains
  `warnings: Vec<MapspliceWarning>`; `src/main.rs` writes each warning as one
  `warning: <display>` line to standard error before writing standard output,
  and emits one `tracing::warn!` event per warning with a stable
  `warning_class`. Rationale: this mirrors the existing error path
  (`report_error` traces a class and writes the display text), keeps domain
  purity, and gives phase 6 a typed channel to extend. Rejected options:
  `tracing::warn!` alone (invisible by default), adding strings to the
  `Copy` `PreservationReport`, and a full severity and finding-code model
  (owned by phase 6). Date/Author: 2026-11-20, planning agent.
- D6. Decision: the warning text is
  `` <path>: line <L>, column <C>: <kind> <anchor>: `Requires` clause has no recognizable dependency anchors ``,
  printed after the prefix `warning: `. `<kind>` is `phase`, `step`, `task`, or
  `sub-task` from the anchor depth; text outside any item is labelled
  `preamble` with no anchor. When a location is unknown the
  `line <L>, column <C>: ` segment is omitted. Rationale: it matches the
  existing error convention (`line N, column M: ...`, lower-case message, no
  final full stop), follows the GNU Coding Standards guidance for
  non-interactive diagnostics (file first, lower-case message, no trailing
  period)[^gnu], and names the item as the roadmap item requires. The path is
  the path as given on the command line, so output is deterministic in tests.
  Date/Author: 2026-11-20, planning agent.
- D7. Decision: one warning per anchorless keyword occurrence, in document
  order of the edited roadmap, without de-duplication. Rationale: each
  occurrence is a separate fix at a separate location, and document order is
  deterministic. Date/Author: 2026-11-20, planning agent.
- D8. Decision: no Verus proof and no Kani harness for this item. Evidence for
  the ownership kernel comes from an `rstest` partition table, a bounded
  exhaustive enumeration over a lexeme alphabet checked against an independent
  forward-scanning oracle, and a `proptest` property using the same oracle.
  Rationale: see `Verification plan`, "Formal verification decision".
  Date/Author: 2026-11-20, planning agent.
- D9. Decision: warnings are not configurable in this item (no `--quiet` flag,
  no suppression comment). Rationale: no requirement asks for it, and the
  warning is fixed by editing the clause; phase 6 owns output-mode policy.
  Date/Author: 2026-11-20, planning agent.
- D10. Decision: no new metric counter. Rationale: no usage, uptake, failure,
  or mitigation metric is required by the roadmap or design; the stable
  `warning_class` tracing field is enough for log-based correlation.
  Date/Author: 2026-11-20, planning agent.

## Outcomes & retrospective

Not started. This section is completed at each milestone boundary and at
completion.

## Context and orientation

Inspected base revision: `622b71d` ("Bump rust-toolchain from
nightly-2026-11-18 to nightly-2026-11-19 (#241)"). Line numbers below refer to
that revision; relocate symbols with `rg -n '<symbol>' src tests`.

### How an edit runs today

`src/main.rs` `main` calls `mapsplice::run_from_args`, then `emit_outcome` on
success (writes `RunOutcome::stdout` to standard output) or `report_error` on
failure (emits `tracing::error!` with `error_class`, then writes the error's
`Display` text and a newline to standard error through
`report_stderr_diagnostic`, and exits with status 1).

`src/lib.rs` `run_request` reads the target, parses it with
`parse_roadmap_inner`, loads the optional fragment with `load_fragment`, calls
`roadmap::ops::apply_command_with_report`, renders with
`render_roadmap_with_report`, optionally writes in place with `rewrite_utf8`,
records observability, and returns `RunOutcome::stdout(rendered)` or
`RunOutcome::in_place(path)`. `RunOutcome` (`src/lib.rs`, about line 258) is
`pub struct RunOutcome { pub stdout: Option<String>, pub written_path:
Option<Utf8PathBuf> }` deriving `Debug, Eq, PartialEq`, with private
constructors `stdout`, `in_place`, and `silent_success`. The export command
builds its outcomes in `src/export_command/mod.rs`.

`src/roadmap/ops/mod.rs` `apply_command_with_report` clones the roadmap,
applies the splice, calls `renumber_document`, then `rewrite_dependencies`
(`src/roadmap/ops/rewrite.rs`), and returns the rewrite count and a
`PreservationReport`. `rewrite_dependencies` walks these *dependency regions*,
in order: the preamble; each phase's `title`, `body`, and `trailing`; each
step's `title`, `body`, and `trailing`; each task's `summary`, `body`, and
`body_children_mut()` blocks; and each sub-task's `summary` and `body`. Each
region is a `MarkdownNodes` value, which carries the source text it was parsed
from (`MarkdownNodes::source`, an `Arc<str>`), and each item carries an
`identity: ItemIdentity { source: SourceId, anchor: RoadmapAnchor }` recording
the source document (`SourceId::Target` or `SourceId::Fragment`) and the
anchor as written there.

### How the scanner recognizes a dependency reference

`src/roadmap/ops/dependency_text.rs` (262 lines):

- `dependency_candidates(value)` scans for anchor-shaped tokens with
  `next_anchor_candidate` and keeps a token when
  `classify_dependency_reference` returns `Reference(anchor)` (kept as
  `anchor: Some`) or `InvalidDependencyToken` (kept as `anchor: None`).
  It returns `Vec<DependencyCandidate { range, anchor }>`.
- `is_dependency_anchor` checks the keyword list `["Requires"]` through
  `latest_keyword_before`, which takes the *last* `Requires` substring before
  the token (`rmatch_indices`), then requires `is_keyword_boundary` (no ASCII
  alphanumeric neighbour), `has_dependency_clause_separator` (a colon or
  whitespace between keyword and token) and no
  `has_dependency_clause_terminator` (a `;`, or a `.` not between two digits).
- `has_section_sigil` rejects a token immediately preceded by `§`.

`src/roadmap/ops/dependency_visitor.rs` (399 lines) is the shared traversal used
by rewriting and by JSON export collection. `visit_dependency_nodes` groups
adjacent `Text` and `Break` nodes, flattens each group with
`flatten_text_group` (a `Break` becomes one space), and records only candidates
lying wholly inside one text node (`candidate_text_segment`). A single text node
takes the fast path `visit_single_text`. Containers are descended by
`visit_container_node`, `visit_inline_container_node`, and
`visit_mdx_container_node`; inline code, fenced code, HTML, and link
destinations are leaves and are never scanned. The visit modes are
`DependencyVisitMode::Rewrite` and `DependencyVisitMode::Collect`.
`source_location_at` (`src/roadmap/source_location.rs`) maps a byte range in a
text node to a one-based `SourceLocation { line, column }`, matching the n-th
occurrence of the token text inside the node's own source span.

### Tests, proofs, and gates available

- Unit tests: `rstest` 0.28.0 tables in
  `src/roadmap/ops/dependency_text_tests.rs`.
- Golden fixtures: `tests/roadmap_golden.rs` and
  `tests/roadmap_golden/nested_requires.rs`, driven by `tests/golden/`
  (`case.rs`, `runner.rs`, `assertions.rs`, `workspace.rs`, `format_gate.rs`).
  Fixture directories are `tests/fixtures/golden/<case>/{target.md,
  fragment.md, expected.md}`. Success cases assert `RunOutcome` stdout; no
  fixture pins standard error today.
- Behavioural tests: `rstest-bdd` 0.6.0. Feature file
  `tests/features/mapsplice.feature`, scenarios bound in
  `tests/behaviour_cli.rs`, steps in `tests/steps/cli_steps.rs` (`CliState`
  captures `stdout`, `stderr`, and `success` from the real binary).
- Process tests: `tests/cli_process_contract.rs` (Linux only) runs the built
  binary with `RUST_LOG=off`, `NO_COLOR`, `LANG=C`, and an isolated `HOME`.
- Property tests: `proptest` 1.9.0. `tests/roadmap_dependency_properties.rs`
  generates roadmaps from a model (`tests/support/generation.rs`) and checks
  them against an oracle that never calls the production recognizer
  (`tests/support/oracle.rs`), 256 cases per property.
- Snapshots: `insta` 1.44.0, currently one inline snapshot in
  `tests/cli_ui.rs`.
- Proofs: Verus proves only the resolution kernel `select_resolution`
  (`verus/lib.rs`, `src/roadmap/ops/select_resolution.macro.rs`), run by
  `make verus` and `make verus-selftest`; the ledger is `docs/verification.md`,
  checked by `make check-verification-ledger`. The clause recognizer is
  explicitly outside every proof (`docs/verification.md`). No Kani harnesses
  exist.
- Gates: `make check-fmt`, `make lint` (verification ledger, domain purity,
  rustdoc and Clippy with `-D warnings`, Whitaker), `make test` (nextest plus
  doctests), `make markdownlint`, `make nixie`, `make spelling`. Continuous
  integration also runs `markdownlint-cli2` separately from `make check-fmt`.

### Documentation and skills to consult

Documentation: `docs/mapsplice-design.md` (sections 5–8, especially section 7,
"The dependency-reference model (normative)"), `docs/developers-guide.md`
(sections 2, 3, 6, 7, and 8.2), `docs/users-guide.md` ("Help and
diagnostics", the dependency-rewriting text under the worked example,
"Validation rules and failure cases", and "Compatibility and migration"),
`docs/validation-and-agent-output-design.md` (finding severities and the phase
6 dependency pass), `docs/migrations/0.2.0.md`,
`docs/documentation-style-guide.md` (ADR format),
`docs/rust-testing-with-rstest-fixtures.md`, `docs/rstest-bdd-users-guide.md`,
`docs/rust-doctest-dry-guide.md`,
`docs/reliable-testing-in-rust-via-dependency-injection.md`,
`docs/complexity-antipatterns-and-refactoring-strategies.md`, and
`docs/ortho-config-users-guide.md` (no configuration change is planned; consult
it only to confirm that no new option is needed).

Skills: `rust-router` (then `rust-unit-testing` for `rstest`, `googletest`,
`pretty_assertions`, and `insta`; `proptest` for the property; `rust-errors`
for the warning type), `hexagonal-architecture` (domain finding versus
application warning versus adapter output), `execplans`, `nextest`, and
`en-gb-oxendict-style` for documentation.

## Conformance basis

Upstream artefacts, all at revision `622b71d`:

- `docs/roadmap.md`, item 1.2.3 and step 1.2.
- `docs/mapsplice-design.md`, sections 5 (F1–F5), 6 (C3, C6), 7 (dependency
  reference model), and 8 (fixture requirements).
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
  those of the same edit without the warning (roadmap 1.2.3, design F1, F2,
  C6).
- `REQ-pin`: a fixture pins the diagnostic text (roadmap 1.2.3, design
  section 8 regression discipline).
- `REQ-scope`: no change to which numbers are rewritten (design C3, section 7).
- `REQ-failclosed`: failures still report only the typed error (design F5).

Gaps:

- `GAP-silent`: a keyword with zero recognized tokens is silently inert.
- `GAP-channel`: no non-fatal diagnostic channel exists.

Trace:

```plaintext
roadmap 1.2.3 -> REQ-warn, GAP-silent -> EP-M2, EP-M3 -> dependency_text_tests::anchorless_*, anchorless_clause_tests::region_parity
roadmap 1.2.3 -> REQ-name, REQ-pin, GAP-channel -> EP-M4 -> tests/fixtures/golden/c3_anchorless_requires_warning/expected_warnings.txt
roadmap 1.2.3 -> REQ-succeed -> EP-M4 -> golden zero-warning sweep, behaviour_cli anchor-free scenarios
design C3 -> REQ-scope -> EP-M1..EP-M4 -> existing dependency suites unchanged
design F5 -> REQ-failclosed -> EP-M4 -> behaviour_cli dangling-plus-anchorless scenario
```

## Verification plan

### Abstract model

Let `t` be the flattened string of one text group visited by
`visit_dependency_nodes` (adjacent `Text` values concatenated, each `Break`
replaced by one space). Define:

- `K(t)`: the set of byte ranges of keyword occurrences, that is, occurrences
  of `Requires` in `t` whose preceding and following bytes (when present) are
  not ASCII alphanumeric.
- `C(t)`: the candidates returned by `dependency_candidates(t)`. Each candidate
  `c` has a range and an *owner* `own(c)`, the range of the last `Requires`
  substring that starts before `c` (the occurrence `latest_keyword_before`
  selects). Every candidate in `C(t)` has `own(c) ∈ K(t)`, because
  classification rejects a token whose latest keyword fails the boundary test.
- `V(t) ⊆ C(t)`: the candidates the visitor records, that is, those lying
  wholly inside one text node of the group. For a single-node group,
  `V(t) = C(t)`.
- `A(t) = K(t) \ { own(c) | c ∈ V(t) }`: the anchorless keyword occurrences.

For a roadmap document `D` after an edit, let `R(D)` be the ordered list of
dependency regions that `rewrite_dependencies` visits, each paired with its
owner (the containing item's `ItemIdentity`, or none for the preamble). The
diagnostic result is the concatenation, over regions in order and text groups in
traversal order, of one `AnchorlessClause { owner, location }` per element of
`A(t)`, where `location` is `source_location_at` of the keyword range within its
text node.

Preconditions: `D` is the staged document after a successful operation,
renumbering, and rewriting. Malformed roadmaps never reach the query because
parsing or the operation has already failed (F5).

### Invariants and lemmas

- `INV-exact` (establishes `REQ-warn`): for every visited text group `t`, the
  query reports exactly `A(t)`: no anchorless keyword occurrence is missed and
  no owning keyword occurrence is reported.
- `L-owner-total`: every recorded candidate has exactly one owner, and that
  owner is in `K(t)`.
- `L-owner-agrees`: the owner returned alongside each candidate is the same
  keyword occurrence that made `classify_dependency_reference` accept it; the
  candidate list and its accept or reject decisions are unchanged by adding the
  owner (`REQ-scope`).
- `L-region-parity` (establishes `REQ-warn` at document level): the query
  visits exactly the regions in `R(D)`, with the same node traversal as the
  rewriter, and attributes each region to the item `rewrite_dependencies`
  would treat as its owner.
- `INV-non-interference` (establishes `REQ-succeed`, `REQ-scope`): computing
  warnings does not mutate the roadmap; standard output bytes, in-place file
  bytes, and exit status equal those of the same edit with the query removed.
- `INV-stream-separation` (establishes `REQ-succeed`): warnings appear only on
  standard error; standard output contains only the rendered roadmap.
- `INV-failure-silence` (establishes `REQ-failclosed`): a failed edit emits no
  warning lines.
- `INV-order` (establishes `REQ-pin`): warnings are emitted in region order,
  then text order, deterministically.
- `INV-location` (establishes `REQ-name`): the reported line and column are
  the position of the keyword's `R` in the item's own source file, including
  after a same-node token was rewritten to a different length, and including
  fragment items.

### Axioms (trusted, not verified here)

- `AXIOM-mdast`: the `markdown` crate produces `Text`, `Break`,
  `InlineCode`, `Code`, `Html`, `Strong`, `Emphasis`, and `Link` nodes with
  correct `Position` offsets into the parsed source, as relied on by every
  existing location test.
- `AXIOM-scanner`: `dependency_candidates` is the specified dependency
  recognizer; its decisions are pinned by roadmap items 1.1.1–1.2.2 and their
  tests. This plan verifies ownership and anchorlessness relative to it, not
  the recognizer itself.
- `AXIOM-stderr`: `std::io::stderr` writes complete lines in order for a single
  thread; the existing `write_stderr_line` helper is reused.
- `AXIOM-gnu`: the GNU Coding Standards message layout is a convention, not a
  contract; this plan's own fixture is the contract.

### Methods per obligation

- Obligation: `L-owner-total` and `L-owner-agrees`.
  Method: `rstest` partition table plus assertions on the existing table.
  Rationale: ownership is a finite set of syntactic situations.
  Domain: one keyword with one token; two keywords each with tokens; a
  `PreRequires` substring between keyword and token; a terminator between; a
  colon separator; a malformed token; a section sigil.
  Artefact: `src/roadmap/ops/dependency_text_tests.rs`, new
  `candidate_owner_cases`.
  Evidence: `cargo nextest run -p mapsplice candidate_owner` fails to compile
  before `DependencyCandidate::keyword` exists (red), then passes (green).
  Every existing `dependency_text_tests` case passes unchanged.
  Non-vacuity: the table includes a case with two keywords where the token
  belongs to the second, which rejects an implementation that always picks the
  first keyword; a seeded fault that returns the first keyword must fail it.
- Obligation: `INV-exact` at the text level.
  Method: `rstest` partition table for the classes in D3, a bounded exhaustive
  enumeration, and a `proptest` property, all compared against an independent
  oracle.
  Rationale: the input space is open-ended strings, but the behaviour depends
  on a small lexeme vocabulary; bounded exhaustive enumeration over that
  vocabulary covers every interleaving up to the bound, and `proptest` extends
  coverage to longer strings with shrinking.
  Domain: the lexemes `Requires`, `requires`, `PreRequires`, ` `, `:`, `.`,
  `;`, `-`, `§`, `none`, `1.2`, `1.4.0`, `\n`. Exhaustive enumeration covers
  every sequence of up to four lexemes (13^4 + 13^3 + 13^2 + 13 = 30,940
  strings). The property draws sequences of up to 24 lexemes, 512 cases.
  Artefact: `src/roadmap/ops/anchorless_oracle_tests.rs` (unit-test module,
  `#[cfg(test)]`), with the oracle `oracle_anchorless(value: &str) ->
  Vec<Range<usize>>` implemented as a forward scan: for each boundary-valid
  keyword occurrence, the clause extends to the first terminator (`;`, or `.`
  not between digits) or the next `Requires` substring; the occurrence is
  anchored when the clause contains a token start (a digit not preceded by an
  ASCII alphanumeric and not preceded by `§`), whose separator text from the
  keyword end contains a colon or whitespace. The oracle never calls
  `dependency_candidates` or `latest_keyword_before`.
  Evidence: `cargo nextest run -p mapsplice anchorless` fails before the kernel
  exists (red), then passes with 30,940 enumerated strings and 512 property
  cases (green). Any disagreement is recorded in `Surprises & discoveries`;
  if the production scanner is wrong the work stops, because changing the
  scanner is outside scope.
  Non-vacuity: the enumeration asserts that both anchored and anchorless
  outcomes occur, that at least 1,000 strings contain a keyword, and that each
  D3 class appears; the property records `prop_assert!` coverage counters for
  "has keyword", "anchorless", and "anchored" and fails if any is zero. Seeded
  faults to run once and revert: (a) report every keyword occurrence; (b)
  ignore the boundary test; (c) count candidates without ownership. Each must
  be rejected by the enumeration; the transcript is recorded in
  `Artefacts and notes`.
- Obligation: `INV-exact` across text groups (`V(t)` versus `C(t)`).
  Method: named `rstest` cases through the visitor.
  Domain: a keyword followed by a hard break and then an anchor; a keyword in
  `Strong` with the anchor outside; the anchor in inline code; the anchor in a
  link; two keywords split by a hard break.
  Artefact: `src/roadmap/ops/anchorless_clause_tests.rs`.
  Evidence: red before the `Diagnose` mode exists, green after.
  Non-vacuity: the hard-break case is anchored (no warning) while the `Strong`
  case is anchorless; an implementation that scans each `Text` node separately
  fails the first, and one that scans the paragraph's concatenated plain text
  fails the second.
- Obligation: `L-region-parity` and `INV-location`.
  Method: one table-driven document test.
  Domain: a target roadmap with an anchorless clause in each of the 13 region
  kinds (preamble; phase title, body, trailing; step title, body, trailing;
  task summary, continuation, body bullet; sub-task summary, body; plus a
  fragment task), and a twin roadmap with a valid `Requires` in the same
  regions.
  Artefact: `src/roadmap/ops/anchorless_clause_tests.rs`, `region_parity`.
  Evidence: the anchorless document yields exactly one warning per region with
  the expected owner and line and column; the twin yields none, and an edit over
  the twin rewrites every clause (proving the regions are ones the rewriter
  visits).
  Non-vacuity: removing any one region from the query's walk fails the test
  for that region; the twin rejects an implementation that warns on every
  keyword.
- Obligation: `INV-location` after a same-node rewrite.
  Method: named regression case.
  Domain: `Requires 1.1.9; Requires none.` where `1.1.9` is renumbered to
  `1.1.10`.
  Artefact: `src/roadmap/ops/anchorless_clause_tests.rs`.
  Evidence: the reported column is the source column of the second `Requires`.
  Non-vacuity: a fold over the rewritten value would report a column one
  greater; the test asserts the exact column.
- Obligation: `INV-non-interference`, `INV-stream-separation`, `INV-order`,
  `REQ-pin`.
  Method: golden fixture, golden zero-warning sweep, behavioural scenarios,
  process test, and `insta` snapshot.
  Artefacts: `tests/fixtures/golden/c3_anchorless_requires_warning/`
  (`target.md`, `fragment.md`, `expected.md`, `expected_warnings.txt`);
  the golden runner (`tests/golden/runner.rs` and `assertions.rs`) asserts
  `expected_warnings.txt` when present and an empty warning list otherwise, so
  every existing success fixture becomes a zero-warning regression;
  `tests/features/mapsplice.feature` scenarios; a process case in
  `tests/cli_process_contract.rs`; an `insta` snapshot of all warnings for the
  region-parity document in `tests/cli_ui.rs` or a new
  `tests/roadmap_warnings.rs`.
  Evidence: red before `RunOutcome::warnings` exists; green after; the golden
  sweep passes over all existing fixtures.
  Non-vacuity: the fixture's expected output contains the anchor-free clause
  unchanged and a renumbered neighbour, proving the edit ran; the process test
  asserts standard output bytes equal `expected.md` and standard error equals
  the pinned warning line exactly.
- Obligation: `INV-failure-silence`.
  Method: behavioural scenario.
  Domain: a target with both an anchor-free clause and a dangling reference.
  Evidence: the command fails, standard error contains the dangling-dependency
  diagnostic, and contains no line starting with `warning:`.
  Non-vacuity: the same target without the dangling reference does warn
  (covered by the main scenario), so the absence is caused by the failure.
- Obligation: generated-document false-positive freedom.
  Method: extend the existing dependency property.
  Domain: the 256 generated cases per property in
  `tests/roadmap_dependency_properties.rs`.
  Evidence: every successful generated edit returns an empty
  `RunOutcome::warnings`, and a second property injects `- Requires none.` into
  one generated task body and asserts exactly one warning naming that task's
  source anchor, with stdout otherwise satisfying the existing oracle.
  Non-vacuity: the injected property fails if the warning list is empty, and
  the existing generator always emits at least one anchored clause.
- Obligation: dogfood.
  Method: manual acceptance check recorded in `Artefacts and notes`.
  Evidence: `cargo run -- insert --after docs/roadmap.md 1.2.3 <task fragment>`
  in preview prints no warning lines.

### Formal verification decision

No Verus proof and no Kani harness are planned. The new contractual logic is
the ownership kernel `A(t) = K(t) \ own(V(t))` and the region walk. The kernel
is a set difference over facts the existing scanner already computes; once the
owner is returned by the same function that accepts the candidate,
`L-owner-agrees` holds by construction, and a Verus proof of the set difference
would restate its definition. The substantive risk is in string scanning and
Markdown traversal, which Verus cannot reach without re-modelling `&str`
scanning and `mdast` outside the shipped code; a proof over such a model would
leave the shipped recognizer unverified, as `docs/verification.md` already
records for the recognizer. Bounded exhaustive enumeration over the lexeme
alphabet gives exhaustive evidence within a stated bound without new tooling,
and `proptest` extends it beyond the bound. The existing Verus proofs over
`select_resolution` are untouched; `make verus` is not an acceptance criterion
because no proved function changes, and `make check-verification-ledger` (part
of `make lint`) still runs. Revisit this decision if implementation needs a
new resolution or renumbering rule.

### Suggested BDD features

Add to `tests/features/mapsplice.feature`, bound in `tests/behaviour_cli.rs`,
with new steps in `tests/steps/cli_steps.rs`:

```gherkin
  Scenario: Anchor-free Requires clause warns without blocking an in-place edit
    Given the target roadmap with an anchor-free Requires clause
    And the phase fragment roadmap
    When I append the phase fragment in place
    Then the command succeeds
    And stdout is empty
    And stderr warns that task 1.1.2 has a Requires clause with no recognizable anchors
    And the target file contains the appended phase and the unchanged anchor-free clause

  Scenario: Anchor-free Requires clause warns while previewing to stdout
    Given the target roadmap with an anchor-free Requires clause
    When I delete phase 2
    Then the command succeeds
    And stdout contains the rendered roadmap with the unchanged anchor-free clause
    And stderr warns that task 1.1.2 has a Requires clause with no recognizable anchors

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
```

Each scenario traces to `REQ-warn`, `REQ-name`, `REQ-succeed`, or
`REQ-failclosed`, and is delivered in `EP-M4`.

## Plan of work

### `EP-M0` feasibility probe

Question: does today's binary accept the D3 classes silently, and do fragment
`MarkdownNodes` carry fragment source text so a fragment warning can report a
location? Build once (`cargo build`), write a scratch target under `/tmp`
containing the five clauses in `Purpose / big picture`, run
`target/debug/mapsplice delete /tmp/probe/target.md 2`, and observe empty
standard error and exit status 0. Read `parse_fragment_inner` and confirm that
it passes the fragment text into each `MarkdownNodes` (search for
`SourceText` in `src/roadmap/parse/`). Record the transcript in
`Artefacts and notes`. No repository file changes. If fragment nodes carry no
source, D6's location segment is omitted for fragment items and the plan is
revised before `EP-M3`.

### `EP-M1` split the dependency visitor without behaviour change

Requirements: none directly; enables `EP-M3` under the 400-line limit.

Create `src/roadmap/ops/dependency_text_group.rs` with a `//!` comment
explaining that it owns the flattening of adjacent text and hard-break nodes
and the mapping of flattened ranges back to their text nodes. Move
`TextSegment`, `flatten_text_group`, and `candidate_text_segment` from
`src/roadmap/ops/dependency_visitor.rs` into it, unchanged except for
`pub(super)` visibility, and declare the module in `src/roadmap/ops/mod.rs`.
No behaviour changes; no tests change. This is a refactor commit on its own.

Acceptance: `make check-fmt`, `make lint`, and `make test` pass;
`wc -l src/roadmap/ops/dependency_visitor.rs` reports fewer than 370 lines.

### `EP-M2` keyword-ownership kernel

Requirements: `REQ-warn`, `REQ-scope`; obligations `L-owner-total`,
`L-owner-agrees`, `INV-exact` (text level).

Red: add `candidate_owner_cases` and the anchorless partition table to
`src/roadmap/ops/dependency_text_tests.rs`, and the oracle, bounded
enumeration, and property to the new `src/roadmap/ops/anchorless_oracle_tests.rs`
(declared under `#[cfg(test)]` beside the existing test module). Use `rstest`
for tables, `googletest` matchers (`assert_that!`, `elements_are!`,
`is_empty()`) for collection assertions, and `pretty_assertions::assert_eq!`
with a message for range lists. Run
`cargo nextest run -p mapsplice dependency_text anchorless` and observe the
compile failure for the missing `DependencyCandidate::keyword` field and the
missing `anchorless_dependency_keywords` function.

Green: in `src/roadmap/ops/dependency_text.rs`, add
`keyword: Range<usize>` to `DependencyCandidate`, filled from the occurrence
`latest_keyword_before` selected (thread it out of `is_dependency_anchor` and
`classify_dependency_reference` by returning the keyword range alongside the
`Reference` and `InvalidDependencyToken` classifications); add
`dependency_keywords(value: &str) -> Vec<Range<usize>>`, which enumerates
boundary-valid `Requires` occurrences using `is_keyword_boundary`; and add
`anchorless_dependency_keywords(value: &str, recorded: &[DependencyCandidate])
-> Vec<Range<usize>>`, returning the keyword ranges not owned by any recorded
candidate, in ascending order. Re-run the focused tests to green.

Refactor: keep the file under 400 lines; extract a predicate if
`classify_dependency_reference` grows beyond one screen. Run the three gates.

Seeded-fault check: apply each fault named in `Verification plan` in turn,
observe the enumeration fail, revert, and record the transcripts. Delete any
`proptest-regressions` file the faults create.

### `EP-M3` anchorless-clause query over a roadmap document

Requirements: `REQ-warn`, `REQ-name`; obligations `INV-exact` (group level),
`L-region-parity`, `INV-location`.

Red: create `src/roadmap/ops/anchorless_clause_tests.rs` with the cross-group
cases, `region_parity`, and the rewrite-shift location case. Build documents by
parsing Markdown literals with `parse_roadmap_inner` (and a fragment with
`parse_fragment_inner` plus an `insert` through `apply_command_with_report`, so
the fragment item reaches the staged document as it does at run time). Observe
the compile failure for the missing query.

Green:

1. In `src/roadmap/ops/dependency_visitor.rs`, add
   `DependencyVisitMode::Diagnose { anchorless: &'a mut Vec<Option<SourceLocation>>, source_text: Option<&'a str> }`.
   Extend `mode_source_text`. In `visit_text_group`, for `Diagnose`, record the
   group candidates (reusing `candidate_text_segment`), compute
   `anchorless_dependency_keywords(&flattened, &recorded)`, map each keyword
   range to its text segment, and push
   `source_location_at(text, source_text, local_range)`. In
   `visit_single_text`, do the same with `dependency_candidates(&text.value)`.
   `should_skip_checkbox` stays false for `Diagnose`, matching `Rewrite`.
2. Create `src/roadmap/ops/anchorless_clause.rs` with a `//!` comment stating
   that it is the read-only query reporting `Requires` keywords that own no
   recognized dependency token, that its region walk must mirror
   `rewrite_dependencies`, and that it is the only producer of
   `AnchorlessClause`. Define:

   ```rust
   /// A `Requires` keyword that owns no recognized dependency token.
   #[derive(Clone, Debug, Eq, PartialEq)]
   pub(crate) struct AnchorlessClause {
       /// Item containing the clause, or `None` for the preamble.
       pub(crate) owner: Option<ItemIdentity>,
       /// Source position of the keyword in the owner's own source document.
       pub(crate) location: Option<SourceLocation>,
   }

   /// Report anchorless `Requires` keywords in rewrite-traversal order.
   pub(crate) fn anchorless_dependency_clauses(roadmap: &RoadmapDocument) -> Vec<AnchorlessClause>;
   ```

   The function walks the same regions, in the same order, as
   `rewrite_dependencies`, cloning each region's nodes (as
   `collect_dependencies` already does) and visiting them in `Diagnose` mode.
   Task body children need read access; add `TaskEntry::body_children()`
   beside `body_children_mut()` in `src/roadmap/model_task_entry.rs` if no
   immutable accessor exists.
3. Re-export the query and type for the crate from `src/roadmap/ops/mod.rs`
   (`pub(crate) use`). Add a doc comment to `rewrite_dependencies` stating that
   a change to its region list must be mirrored in
   `anchorless_dependency_clauses`, guarded by `region_parity`.

Refactor and gates as before. `make check-domain-purity` must pass.

### `EP-M4` warnings surfaced through `RunOutcome` and standard error

Requirements: `REQ-name`, `REQ-succeed`, `REQ-pin`, `REQ-failclosed`;
obligations `INV-non-interference`, `INV-stream-separation`,
`INV-failure-silence`, `INV-order`.

Red: add the golden case `c3_anchorless_requires_warning` (a task-level insert
whose target holds `- Requires none.` in task 1.1.2's body and a renumbered
neighbour with a valid clause, and whose fragment adds a task with
`**Requires** 1.1.1.`), the zero-warning sweep in the golden runner, the four
BDD scenarios, the process test, the `insta` snapshot, and the two property
assertions. Observe the compile failure for `RunOutcome::warnings`.

Green:

1. Create `src/warning.rs` with a `//!` comment, defining:

   ```rust
   /// A non-fatal diagnostic reported alongside a successful command.
   #[derive(Clone, Debug, Eq, PartialEq)]
   #[non_exhaustive]
   pub enum MapspliceWarning {
       /// A `Requires` keyword governs no recognized dependency anchor token.
       AnchorlessDependencyClause {
           /// Input file containing the clause, as given on the command line.
           path: Utf8PathBuf,
           /// Containing item as written in that file, or `None` for the preamble.
           item: Option<RoadmapAnchor>,
           /// Source position of the keyword, when known.
           location: Option<SourceLocation>,
       },
   }
   ```

   with `pub const fn class(&self) -> &'static str` returning
   `"anchorless_dependency_clause"` and a `Display` implementation producing
   the D6 text without the `warning: ` prefix. Include a doctest that
   constructs the variant and checks its display text. Re-export it from
   `src/lib.rs` beside `MapspliceError`.
2. In `src/lib.rs`, add `pub warnings: Vec<MapspliceWarning>` to `RunOutcome`
   with a doc comment, initialize it empty in `stdout`, `in_place`, and
   `silent_success`, and add a private `with_warnings(self, warnings)`.
   In `run_request`, keep the fragment path, and after the in-place write (or
   after rendering in preview) call `anchorless_dependency_clauses(&roadmap)`
   and map each `AnchorlessClause` to `MapspliceWarning`, choosing
   `request.target` for `SourceId::Target` and the fragment path for
   `SourceId::Fragment`. Extract the mapping into a small named function so
   `run_request` stays short. Export outcomes in `src/export_command/mod.rs`
   carry an empty list.
3. In `src/main.rs`, before writing standard output, emit each warning: one
   `tracing::warn!(warning_class = warning.class(), "mapsplice command warning")`
   and one `report_stderr_diagnostic(&format_args!("warning: {warning}"))`.
   Add a unit test beside `diagnostic_tracing_tests` asserting the traced
   class and that the event does not contain the file path.
4. Add steps to `tests/steps/cli_steps.rs`: the new `Given` targets (as
   constants in `tests/support/cli.rs`), `When I append the phase fragment in
   place`, `Then stderr warns that task 1.1.2 has a Requires clause with no
   recognizable anchors` (exact line match), `Then stderr is empty`,
   `Then stderr contains no warning lines`, and the two content assertions.

Refactor: keep `src/lib.rs` under 400 lines; if needed, move `RunOutcome` and
its constructors into `src/run_outcome.rs`. Run the three gates.

Dogfood: run the preview insert on `docs/roadmap.md` and record that standard
error is empty.

### `EP-M5` documentation, ADR, and roadmap closure

1. Add `docs/adrs/adr-0002-non-fatal-edit-warnings.md` (status `Proposed`
   until the pull request is approved, then `Accepted` with the date),
   following the ADR section order in `docs/documentation-style-guide.md`:
   context (fail-closed tool, silent inert clauses), decision (D1, D4, D5,
   D6), consequences (stderr may carry output on success; phase 6 extends the
   channel), and rejected alternatives (fail the edit; tracing only; validate
   only).
2. `docs/mapsplice-design.md`: in section 7 add a bullet "Anchorless clauses"
   stating D1–D3 normatively and that they are reported as warnings, not
   errors; in section 6 C6 state that warnings go to standard error in both
   output modes and never alter output or exit status; in section 5 note that
   a warning is not a failure under F5; add a Table 2 row "Anchorless
   `Requires`" proving "warned, preserved, edit succeeds"; reference the ADR.
3. `docs/users-guide.md`: add `### Warnings` under "Help and diagnostics"
   with the exact line format and an example; extend the dependency-rewriting
   text with the anchorless rule and how to fix each D3 class; note in
   "Validation rules and failure cases" that warnings are non-fatal; add a
   "Compatibility and migration" subsection.
4. `docs/migrations/0.2.0.md`: record the new `RunOutcome::warnings` field,
   `MapspliceWarning`, and the possibility of standard-error output on
   success.
5. `docs/developers-guide.md`: section 2 (domain `AnchorlessClause` →
   application `MapspliceWarning` → binary output), section 3 (public
   `RunOutcome::warnings` and `MapspliceWarning`), section 7 (the bounded
   enumeration and oracle), and a re-use policy: `MapspliceWarning` is the only
   user-facing warning type; new warnings are new variants produced by
   domain queries and mapped in `src/lib.rs`; domain code never formats paths
   or writes output; `anchorless_dependency_clauses` must mirror
   `rewrite_dependencies`.
6. `docs/validation-and-agent-output-design.md`: note that the edit-time
   warning exists with class `anchorless_dependency_clause`, and that the
   phase 6 dependency pass should reuse it as a `warning`-severity finding.
7. `skill/mapsplice/SKILL.md`: tell agents that warnings may appear on standard
   error after a successful edit and how to fix them.
8. `docs/roadmap.md`: mark 1.2.3 `[x]` with an "Addressed:" note like 1.2.1.
9. Run `make fmt`, then `make check-fmt`, `make markdownlint`, `make nixie`,
   `make spelling`, `make lint`, and `make test`.

## Milestones and plateaus

- `EP-M1`: the visitor is split; behaviour identical. Requirements: none.
  Evidence: full suite green. Conformance: design unchanged; no interface or
  dependency change. Recovery: revert the single commit. Remaining gaps: all
  behaviour. Compatibility decision: none.
- `EP-M2`: the scanner reports owners and anchorless keywords for a string.
  Requirements: `REQ-scope` (held), `REQ-warn` (text level). Evidence:
  ownership and anchorless tables, enumeration, and property green; seeded
  faults rejected. Conformance: recognizer decisions unchanged
  (`dependency_text_tests` untouched in meaning); no public change.
  Recovery: revert the milestone commits. Remaining gaps: document query and
  output. Compatibility decision: none (`DependencyCandidate` is
  crate-private).
- `EP-M3`: the crate can list anchorless clauses for a document.
  Requirements: `REQ-warn`, `REQ-name` (data). Evidence: cross-group,
  region-parity, and location tests green; domain purity green.
  Conformance: query is read-only; region walk mirrors the rewriter.
  Recovery: revert. Remaining gaps: user-visible output. Compatibility
  decision: none.
- `EP-M4`: users see warnings. Requirements: all `REQ-*`. Evidence: golden
  fixture with `expected_warnings.txt`, golden zero-warning sweep, BDD
  scenarios, process test, snapshot, properties, dogfood. Conformance: F1–F5
  and C1–C6 hold; the two authorized public API additions only.
  Recovery: revert. Remaining gaps: documentation. Compatibility decision:
  none (pre-1.0, unreleased; migration note only).
- `EP-M5`: documentation, ADR, and roadmap reflect the behaviour.
  Evidence: Markdown, Mermaid, spelling, and code gates green.
  Recovery: revert documentation commits. Remaining gaps: none.
  Compatibility decision: none.

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
EP-M1 green:  cargo nextest run -p mapsplice dependency            -> all pass, counts unchanged from base
EP-M2 red:    cargo nextest run -p mapsplice dependency_text anchorless -> E0560/E0425 for keyword / anchorless_dependency_keywords
EP-M2 green:  same command                                          -> all pass; enumeration reports 30940 strings
EP-M3 red:    cargo nextest run -p mapsplice anchorless_clause      -> E0425 anchorless_dependency_clauses
EP-M3 green:  same command                                          -> all pass
EP-M4 red:    cargo nextest run -p mapsplice --test roadmap_golden --test behaviour_cli -> E0609 no field `warnings`
EP-M4 green:  same command, plus --test cli_process_contract --test roadmap_dependency_properties -> all pass
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
# stderr: warning: /tmp/r.md: line 10, column 5: task 1.1.2: `Requires` clause has no recognizable dependency anchors
$ echo $?
0
```

Quality criteria:

- Tests: `make test` passes; the new tests named in `Verification plan` fail
  before their milestone's green step and pass after; every existing golden
  success fixture asserts zero warnings.
- Verification: `L-owner-total`, `L-owner-agrees`, `INV-exact`,
  `L-region-parity`, `INV-location`, `INV-non-interference`,
  `INV-stream-separation`, `INV-failure-silence`, and `INV-order` are each
  discharged by the artefacts named above; seeded-fault transcripts are
  recorded.
- Lint: `make lint` passes, including domain purity and the verification
  ledger.
- Formatting: `make check-fmt` and `make markdownlint` pass.
- Documentation: `make nixie` and `make spelling` pass.

## Idempotence and recovery

Every step is a source edit followed by a test run, so it can be repeated.
Each milestone ends in a commit that is a safe rollback point. Seeded faults are
applied to the working tree only, never committed, and reverted with
`git checkout -- <file>`; remove any `*.proptest-regressions` entries they
produce. The `EP-M0` probe writes only under `/tmp`. If a gate fails, read its
`/tmp` log before re-running it.

## Artefacts and notes

Reconnaissance observations at `622b71d` (planning time):

- `rg -n 'warn!' src` returns only a comment in
  `src/roadmap/ops/remap_kernel.rs`.
- `wc -l src/roadmap/ops/dependency_visitor.rs` reports 399.
- `rg -n 'Requires' docs/roadmap.md | rg -v 'Requires [0-9]'` shows every
  prose mention wrapped in inline code, so the dogfood check is expected to
  print no warnings.

Implementation transcripts (probe, red and green evidence, seeded faults,
dogfood) are added here as milestones complete.

## Interfaces and dependencies

At the end of `EP-M4` these exist:

- `crate::roadmap::ops::dependency_text::DependencyCandidate { range,
  anchor, keyword }` (crate-private), plus `dependency_keywords` and
  `anchorless_dependency_keywords` (`pub(crate)`).
- `crate::roadmap::ops::dependency_visitor::DependencyVisitMode::Diagnose`
  (crate-private).
- `crate::roadmap::ops::anchorless_clause::{AnchorlessClause,
  anchorless_dependency_clauses}` (crate-private).
- `mapsplice::MapspliceWarning` (public, `#[non_exhaustive]`) with `class()`
  and `Display`.
- `mapsplice::RunOutcome::warnings: Vec<MapspliceWarning>` (public field).

Reuse policy: `dependency_candidates` remains the only recognizer of
dependency tokens and the only source of ownership; `visit_dependency_nodes`
remains the only Markdown traversal for dependency text; no module re-parses
Markdown or re-implements keyword matching. `anchorless_dependency_clauses` is
the only producer of anchorless findings and is called only from
`run_request`. `MapspliceWarning` is the only public warning type.

Dependencies: add to `[dev-dependencies]` in `Cargo.toml`:

```toml
googletest = "0.14.4"
pretty_assertions = "1.4.1"
```

No runtime dependency changes.

## Revision note

- 2026-11-20: initial draft from Wyvern reconnaissance at `622b71d`.

[^gnu]: GNU Coding Standards, section 4.4 "Formatting Error Messages",
    <https://www.gnu.org/prep/standards/html_node/Errors.html>.
