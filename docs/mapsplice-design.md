# Mapsplice design

## Status and scope

- **Status:** living design document for the `mapsplice` CLI. It describes both
  the current implementation and the target contract the tool is evolving
  towards.
- **Scope:** the roadmap-splicing engine — parsing, the roadmap model, the four
  structural operations, in-place checkbox completion, renumbering,
  dependency-reference rewriting, and rendering, plus the read-only JSON export
  and drift contract. CLI plumbing (`ortho-config`, `--in-place`) is covered
  only where it bears on the guarantees below.
- **Audience:** contributors to `mapsplice`, and operators who run it
  unsupervised over real roadmaps (for example as part of a df12-build
  workshop).
- **Precedence:** `docs/users-guide.md` is the source of truth for command
  semantics and the accepted roadmap grammar; `AGENTS.md` governs the quality
  gates, testing rules, and en-GB-oxendict spelling. This document is the
  source of truth for the *fidelity* and *contract guarantees* and the *fixture
  requirements* that prove them. Where the implementation currently diverges
  from these guarantees, section 9 records the divergence and `docs/roadmap.md`
  tracks the remediation.

## 1. Problem statement

`mapsplice` applies one edit to a roadmap-shaped Markdown document — a
structural `append`, `insert`, `delete`, or `replace` that renumbers the
affected phases, steps, and tasks and rewrites the dependency references that
pointed at them, or an in-place `complete` that only checks one item off.

The defining hazard of such a tool is **collateral corruption**: silently
rewriting content it was never asked to touch, or mangling structure it does
not model. A roadmap editor that occasionally damages an unrelated section
reference, a version string, or a nested sub-task is worse than no tool at all,
because the damage is easy to miss in review and the operator loses the very
confidence the tool exists to provide.

This document therefore defines, as first-class contracts, the **fidelity**
guarantees that keep an edit minimal and faithful, the **functional contract**
each operation honours, the **dependency-reference model** that decides what is
and is not a reference, and the **fixture corpus** that pins all of it. The
guiding principle: *a splice changes only what it was asked to change, plus the
deterministic, documented consequences of that change — and nothing else.*

## 2. Non-negotiable constraints

These are assumed, not re-justified, by later sections (see the implementation
plan in `docs/execplans/initial-tool.md`):

- The supported roadmap syntax is a **constrained grammar**, not arbitrary
  Markdown. Documents outside it are rejected, never guessed at.
- Parsing is **mdast-based** (the `markdown` crate); edits run against a roadmap
  intermediate representation, never raw-string surgery.
- The default output is **standard output**; `--in-place` / `-i` rewrites the
  target file instead.
- **Level matching is strict**: a fragment must match the structural level of
  its anchor, or the operation fails with a typed error.
- The CLI is composed through `ortho-config`; filesystem access is
  capability-oriented (`cap_std`, `camino`).
- Tests are written with `rstest` (unit) and `rstest-bdd` (behavioural); modules
  stay below 400 lines.

## 3. Architecture overview

`mapsplice` is a pipeline from Markdown text to Markdown text through an
explicit roadmap model. Only text nodes inside the model are ever rewritten,
and only the supported grammar is ever rendered.

```mermaid
flowchart LR
  A[Markdown source] --> B[mdast parse]
  B --> C[Roadmap model]
  C --> D[Splice operation]
  D --> E[Renumber]
  E --> F[Reference rewrite]
  F --> G[Render grammar]
  G --> H[stdout or in-place]
  C --> J[JSON export adapter]
  J --> K[stdout or drift check]
```

*Figure 1: The mapsplice edit pipeline.*

The roadmap model (section 4) is the boundary that makes the guarantees
checkable: structural edits operate on typed phase, step, and task items;
renumbering is a pure function of position; and reference rewriting consults a
renumber plan rather than blindly substituting text.

The dependency direction is inward, and it is enforced rather than assumed. The
roadmap domain owns the Markdown model, the splice, and the rendering; whatever
reads a file, reads an environment variable, or walks the filesystem belongs to
an adapter that calls the domain. Filesystem access lives in `src/fs.rs`, and
`make check-domain-purity` fails `make lint` if a production source under
`src/roadmap` names infrastructure — through an import, a qualified path, a
process call, or a build-time macro. The import shape matches the
infrastructure name as a whole path segment rather than modelling the brace
grammar of a `use` tree, so nested groups such as
`use std::{fmt::{self, Display}, fs as files};` are caught where a
shape-per-form matcher was defeated three times. The rule needs a gate because
nothing in the compiler enforces it, and the temptation is concrete: the
resolved kernel is spliced with a macro, and the obvious way to locate a file
for `include!` is `env!("CARGO_MANIFEST_DIR")`. That construction also runs the
dependency the other way — from the domain to verification infrastructure —
which is why the shared kernel body is included *from* the domain-owned source
by both the production `const fn` and the proof in `verus/lib.rs`, rather than
the reverse. The gate and the roadmap proof boundary are described in
[developers-guide.md](developers-guide.md) and
[verification.md](verification.md).

The read-only export follows the same boundary. `src/lib.rs` dispatches a
`CommandKind::Export` request to `src/export_command/mod.rs`, which reads the
Markdown path and any committed export through an injected UTF-8 reader and
turns a mismatch into an `ExportError::Drift` report. The pure adapter in
`src/roadmap/export/` maps the existing roadmap model to an ordered, versioned
schema and compares decoded values for drift. The adapter owns no filesystem or
process operations, and the orchestration owns no parsing or comparison rules.
Exact bytes are checked first so a current export succeeds silently; semantic
comparison runs only when bytes differ. The grammar is enforced once at parse
time: a numbered checkbox item beneath a checkbox detail has no roadmap level,
so the parser rejects it with a source location for every command rather than
leaving the export adapter to discover it later.

## 4. The roadmap grammar (normative reference)

The accepted grammar is defined normatively in `docs/users-guide.md`. In
summary, a roadmap has an optional preamble followed by three structural levels:

- **Phases** — level-2 headings whose text begins with a phase number (`## 8.`).
- **Steps** — level-3 headings whose text begins with a step number
  (`### 8.2.`).
- **Tasks** — list items whose first inline text begins with a task number
  (`- [ ] 8.2.3.`).

A task may carry continuation paragraphs and nested bullet lists. Addendum
sub-tasks are recognized per nested-list item: an item whose number extends its
parent task by one level (`- [ ] 8.2.3.1.`) is an **addendum sub-task** — a
fourth structural level. Ordinary bullets may share that list and remain
non-structural body content. Anchors on the command line address these levels
directly: `8` a phase, `8.2` a step, `8.2.3` a task, `8.2.3.1` an addendum
sub-task.

## 5. Fidelity guarantees

These hold for every successful operation over conformant, gate-clean input.
Here, *gate-clean* means the input already passes the house Markdown formatter
and linter without a diff. Gate cleanliness is not itself a parse rule:
`mapsplice` may accept formatter-unstable Markdown that still matches the
roadmap grammar. When it does, F4 takes precedence over byte-for-byte
preservation for formatter-unstable syntax, so successful rendering may
normalize unstable spacing, list numbering, table alignment, or fence spelling
only as needed to produce gate-clean Markdown. Such normalization must be
deterministic, limited to formatter-unstable syntax, and pinned by fixtures.
Untouched gate-clean content remains byte-exact.

- **F1 — Content preservation.** Every node that the operation does not
  structurally target, and that is not renumbered or reference-rewritten as a
  documented consequence, is preserved exactly: text, formatting, list nesting,
  tables, and code blocks are unchanged. Task lists keep a step-level
  preserved-source span for the fast path and per-task source spans for dirty
  lists, so an edit can regenerate only tasks whose content, child ordering,
  numbering, or dependency text changed while leaving untouched tasks
  byte-exact, including loose-list spacing.
- **F2 — Minimal diff.** The only changes are the addressed item itself and the
  deterministic consequences of the edit — the renumbering of later items
  (section 6, C2) and the rewriting of dependency references to them (C3).
- **F3 — Round-trip stability.** Parsing a conformant document and rendering it
  under a no-op edit is the identity, modulo the one documented normalization:
  non-empty rendered roadmaps end in exactly one final newline. Rendering again
  does not add another final newline.
- **F4 — Gate-clean output.** Rendered output passes the house Markdown gates
  (`make markdownlint`) and is stable under the house formatter (`make fmt`,
  which runs `mdtablefix` then `markdownlint-cli2 --fix`): a second formatting
  pass produces no diff.
- **F5 — Fail closed.** Malformed input is rejected with a typed, user-facing
  error before any output is produced. `mapsplice` never emits a partially
  rewritten or mangled document, and `--in-place` writes only on success.

## 6. Functional and contract guarantees

- **C1 — Operations.** `append` (phase-level), `insert` (before, or `--after`),
  `delete`, and `replace`, each addressed by an anchor, with strict level
  matching between fragment and anchor. `insert` and `replace` differ only in
  what happens to the addressed item's identity: an `insert` leaves it in place
  and splices the fragment around it, while a `replace` **retires** it. A
  retired identity is gone, so a dependency reference to it no longer resolves
  and the operation fails closed rather than redirecting the consumer; a
  fragment clause, which is written in the fragment's own numbering, still
  resolves through the cross-source fallback described in section 7. `complete`
  is not structural: it checks one addressed task or addendum sub-task off,
  changes no number, takes no fragment, and is specified in section 12.
- **C2 — Renumber contract.** After any structural edit (`append`, `insert`,
  `delete`, `replace`), phase, step, task, and addendum numbers are contiguous
  from 1, in document order, at every level. No gaps, no duplicates, no
  out-of-order numbering survives an operation. `complete` is exempt: it
  changes no number and must preserve the document's existing numbering
  byte-for-byte, so a roadmap that already carries a gap keeps it.
- **C3 — Reference-rewrite contract.** A *dependency reference* (section 7) to a
  renumbered item is rewritten to that item's new number. A valid dependency
  reference that does not resolve after an edit is reported as a dangling
  dependency diagnostic. A number that is **not** a dependency reference — a
  design-document section reference such as `§3.2`, a semantic version such as
  `1.4.0`, or an incidental quantity — is **never** rewritten, even when it
  coincides with a renumbered roadmap number.
- **C4 — Addenda contract.** An addendum sub-task is a first-class item: it
  renumbers with its parent (when task `8.2.3` becomes `9.2.3`, sub-task
  `8.2.3.1` becomes `9.2.3.1`), and its Markdown nesting and indentation are
  preserved on render.
- **C5 — Idempotence.** A no-op edit, and re-applying an already-applied
  numbering, leave the document unchanged (this is F3 viewed from the operation
  side).
- **C6 — Output modes.** Standard output by default; `--in-place` rewrites the
  target atomically and only when the operation and its gates succeed.

## 7. The dependency-reference model (normative)

This section is the source of truth for what counts as a reference, and is the
contract behind C3.

- An **anchor token** is a run of digits with one to three further `.<digits>`
  groups (a one- to four-level number such as `8`, `8.2`, `8.2.3`, or
  `8.2.3.1`). The token is consumed greedily: `1.2.17.1` is one token, never
  the prefix `1.2.17`.
- A **dependency reference** is an anchor token that appears in a **dependency
  context** — currently the `Requires` clause of a task body (and any future
  `Blocks` clause adopted by the grammar). Only dependency references are
  candidates for rewriting.
- **Clause positions.** A `Requires` clause is recognized in three source
  positions, and all three carry the same rewrite and dangling-dependency
  guarantees:

  | Position                            | Recognized when                        |
  | ----------------------------------- | -------------------------------------- |
  | Inline task or sub-task text        | `Requires` shares the numbered summary |
  | Continuation text of the summary    | `Requires` begins an indented line     |
  | Nested task or sub-task body bullet | `Requires` begins a nested bullet item |

  *Table 1: Recognized `Requires` clause positions.*

- **Numeric ranges.** A range such as `Requires 1.1.1-1.1.3.` is not a
  *supported* clause form and is never *expanded* into the anchors it spans.
  The endpoints are still rewritten, individually: a hyphen is not an anchor
  character, so the scanner reads `1.1.1` and `1.1.3` as two separate anchors,
  each in dependency context, and each follows its own item's new number. The
  rewritten clause therefore keeps exactly two anchors however many the range
  covered. Deleting `1.1.2` from a roadmap that requires `1.1.1-1.1.3` yields
  `Requires 1.1.1-1.1.2.`, and deleting an *endpoint* fails the operation as a
  dangling dependency like any other retired requirement. The form is tolerated
  rather than rejected because rejecting it would mean declining an edit over a
  stylistic choice in prose the author is entitled to write.
- **Hard-wrapped clauses.** A line ending or Markdown hard break between
  `Requires` and its anchors, or between anchors in one clause, is treated as
  whitespace by the shared dependency visitor. Rewriting and JSON export
  therefore see the same references. Semicolons and non-numeric periods still
  terminate a clause.
- **Incidental numbers are preserved.** An anchor token that is not in a
  dependency context, or that is immediately preceded by a section sigil (`§`),
  is incidental: it is a section reference, a version, or prose, and it is left
  exactly as written. Scoping rewriting to dependency contexts — rather than
  substituting every number-shaped token in the document — is what upholds F1
  and C3.
- **Resolution.** A dependency reference is resolved against the renumber plan:
  the source-local mapping first, then, for a reference written in the
  **fragment**, a unique cross-source mapping when the anchor is defined
  exactly once across the target and the fragment. The cross-source fallback is
  deliberately withheld from target text. A fragment item's identity is its
  fragment-local spelling, because that spelling is all the fragment file
  carries; a target item's identity is the anchor it held before the operation.
  Both are written the same way, so if target text could fall back to the
  fragment's mapping, a replacement item spelled like the item it superseded
  would absorb a surviving consumer's clause — the clause would keep its old
  digits and silently resolve to a different item. This is the identity
  preservation guarantee in its resolution form. A valid dependency reference
  that does not resolve is a dangling dependency and the operation fails with a
  typed diagnostic before output is emitted or an in-place write occurs.

## 8. Fixture and test requirements

The guarantees above are mechanical, so they are enforced mechanically, not by
inspection.

- **Golden-fixture corpus.** A corpus of `(input, expected-output)` Markdown
  pairs, one fixture per operation and per contract guarantee, compared
  exactly. A guarantee without a golden fixture is considered unproven.
- **Fixture EOF whitespace.** Golden fixtures are raw-byte evidence. Expected
  output fixtures should normally end in exactly one final newline, matching
  F3. Extra end-of-file whitespace may be kept only when the fixture is proving
  preservation of source bytes that already contain that whitespace. Reviewers
  should treat any other trailing whitespace change as accidental churn unless
  the fixture name, surrounding test, or nearby rationale makes the fidelity
  case explicit.
- **Task-list source boundary.** Parser code captures task-list source at the
  step boundary rather than through ordinary block rendering. Operation code
  must clear that span when it mutates the list, and renderer code must
  validate the task model before reusing preserved task-list bytes.
- **Per-item source boundary.** When parsing a target roadmap, the parser also
  captures each task and addendum sub-task's complete `ListItem` span in its
  `original_source` field. The span includes the header, lazy continuation
  lines, nested body nodes, and nested sub-tasks with their original
  indentation. Fragment parsing leaves this field empty because fragment
  content is new output rather than source to preserve.
- **Per-item invalidation.** Structural operations move untouched task and
  sub-task values without clearing their source. A value is invalidated only
  when its own number, summary, dependency text, body, or structural
  descendants change. The step-level task-list span remains the first-choice
  fast path and is cleared whenever the task list itself changes.
- **Preservation and rendering ownership.** Parsing owns source-span capture;
  mutation and renumbering own precise invalidation; rendering owns the
  formatter-stability check and the fallback to canonical Markdown. A stable
  preserved item is emitted verbatim, with trailing newlines trimmed and the
  required separator restored when it is joined to canonical output. Ordered or
  nested list markers and code-fence delimiters that are formatter-unstable
  force canonical rendering instead. Canonical rendering gives every
  paragraph-first body run a blank-line boundary, including runs after an
  addendum sub-task. When removing a structural child joins same-kind body-list
  runs, it inserts an invisible HTML comment between them unless both tight
  bullet runs are intentionally joined. This keeps `CommonMark` from merging
  distinct ordered lists or changing loose-list state. The marker reuses an
  existing separator where possible to avoid extra blank lines.
- **Preservation observability.** `MetricsSnapshot` exposes aggregate counts
  for preserved-source renders, preserved-source invalidations, and canonical
  fallbacks, together with reason counters for `renumber`, `dependency_rewrite`,
  `child_mutation`, `unstable_list_marker`, and `unstable_code_fence`. These
  causes form closed `PreservationInvalidationReason` and
  `CanonicalFallbackReason` sets. The counters are bounded, atomic, and
  process-local; they are diagnostic state, not durable telemetry. Invalidation
  is recorded at mutation points, while preservation and fallback outcomes are
  recorded at the rendering decision. Canonical rendering of invalidated or
  formatter-unstable task content uses the two-space continuation convention.
- **Required coverage.** The corpus must exercise the whole grammar surface
  (preamble; phases, steps, tasks; multi-line task bodies; nested bullets;
  tables; code blocks) and, as adversarial cases, every way collateral
  corruption could occur:

  | Fixture class                   | Must prove                                 |
  | ------------------------------- | ------------------------------------------ |
  | Section-reference preservation  | `§3.2` survives a renumber unchanged (C3)  |
  | Version / quantity preservation | `1.4.0` and prose numbers survive (C3, F1) |
  | Substring non-match             | `1.2.17.1` is not partially rewritten      |
  | Addendum renumber               | `8.2.3.1` tracks its parent task (C4)      |
  | Addendum render fidelity        | nesting and indentation preserved (C4, F1) |
  | `Requires` lists                | every id in a multi-id clause is rewritten |
  | Dangling `Requires`             | unresolved valid anchors fail closed       |
  | Completion preservation         | only checkbox and marker bytes change (F2) |

  *Table 2: Required adversarial fixtures for the fidelity and reference
  contracts.*

- **Test shapes.** `rstest` unit fixtures cover the model, renumbering, and the
  reference resolver in isolation; `rstest-bdd` features cover the CLI surface
  and output modes; golden comparison covers render fidelity end to end.
- **Round-trip property.** For any conformant document, a no-op edit renders
  byte-identical output (F3), and a second `make fmt` pass produces no diff
  (F4).
- **Regression discipline.** Every fixed defect lands with a fixture that fails
  before the fix and passes after it.

## 9. Known divergences from the target contract

There are no known divergences from the dependency-reference contract in
sections 5–7. The remaining roadmap work in `docs/roadmap.md` expands the
fixture corpus and round-trip guarantees that pin the contract mechanically.

## 10. Risks, trade-offs, and future extensions

- **Constrained grammar (accepted).** `mapsplice` deliberately rejects a fully
  general Markdown round-trip writer (see the decision log in
  `docs/execplans/initial-tool.md`). The trade-off is reach for safety:
  documents must conform, but conformant edits are predictable.
- **Reference contexts (current limit).** Only `Requires` is a dependency
  context today. `Blocks`, `See`, and similar clauses are future extensions and
  must each be added to section 7 before the rewriter acts on them.
- **One edit per invocation (current limit).** Batched multi-edit transactions
  are a possible future extension; until then, compose edits as a pipeline and
  review the standard-output diff between steps.

## 11. Versioned JSON export

Markdown remains the canonical roadmap. `mapsplice export --format json`
derives a stable JSON view for downstream design-contract checkers, while
`--check EXPORT.json` reports drift without writing either input. The
schema-versioned fields, normalization rules, dependency endpoints, strict
block allow-list and example are specified in
[roadmap-export-format.md](roadmap-export-format.md).

The export adapter consumes typed phases, steps, tasks and ordered task
children. It shares dependency candidate recognition with the rewriter, so a
hard-wrapped or nested-bullet `Requires` clause has one classification and one
source order. Unsupported content fails with a located `InvalidRoadmap`
diagnostic rather than disappearing from the export. JSON schema and semantic
changes require a `FORMAT_VERSION` increment.

Export text is literal while the rewriter still escapes: `render_inline_export`
emits plain text without the backslash escapes that keep a rewritten roadmap
reparsable, because the JSON is data rather than Markdown source. Dependency
clauses are classified from the Markdown nodes, not from that rendered text, so
an escaped literal `\*\*Requires\*\*` paragraph is not dropped as a clause. The
dependency scanner still reads its text, though, so an anchor-shaped token such
as `9.9.9` is collected as a dependency: export fails with `DanglingDependency`
when that anchor is absent, and records a dependency edge when it is present.

## 12. In-place checkbox completion

`complete` marks one addressed task or addendum sub-task done without
restructuring the document. It is the minimum edit the grammar allows, and it
is specified as a source patch rather than a re-render: the parser's mdast
positions yield an `ItemSourceSpan` for the addressed item, the checkbox byte
inside the marker becomes `x`, and an optional configured done marker is
inserted at the end of the title paragraph before its terminal punctuation,
keeping closing emphasis, strong, and strike markers after the inserted words.
The parser's inline node positions locate that point: an emphasis container
ends before its closing markers, while inline code and links end after their
closing delimiter, so the suffix follows those literal constructs instead of
joining them. A suffix that itself ends with an inline marker is placed after
the title's closing markers, because joining two marker runs would produce one
ambiguous delimiter sequence; a title without a closing run keeps the usual
placement before its terminal punctuation. Existing-suffix detection compares
the rendered plain text of the title's parsed inline nodes, so literal trailing
markers such as the `*` in `build*` are never mistaken for closing markup. The
configured suffix is rendered as inline paragraph content, so a leading `#` or
list marker stays literal text instead of being read as block syntax, a suffix
carrying a literal marker still matches itself, and punctuation separates a
suffix word just as whitespace does. Terminal punctuation written as a
character reference such as `&#46;` or `&hellip;` is classified by what it
renders as, so it stays last after the inserted words and a punctuated suffix
replaces it; an escaped ampersand keeps the reference literal. Inline HTML
contributes a word boundary only when it renders a break, meaning an element
whose whole tag name is `br`, so `un<span></span>done` and
`un<br-widget></br-widget>done` do not satisfy a `done` suffix while
`un<br>done` does. When a generated break would follow an unpaired backslash,
whether it ends the title or an inserted suffix word, the break is paired with
one more backslash so the literal character survives. A backslash ending the
suffix itself is paired the same way before the terminal punctuation, closing
marker, or line ending that would otherwise consume it, and stays single at the
end of the document, where nothing follows. A line already at the width limit
fails closed instead. Every other byte — title wrapping, continuation lines,
nested bullets, `Requires` references, and all numbering — is retained, which
satisfies F1 and F2 without invoking the renderer at all. Section 5's
*gate-clean* caveat does not arise from syntax, because no unstable syntax is
normalized. It still bounds line width: an input paragraph that is already
wider than 80 columns stays that wide, because completion never reflows
existing title text.

The patched text is re-parsed and compared with the original before it is
returned or written: the anchor set must be unchanged, the addressed item must
parse as checked, and a configured suffix must be present at the end of its
title. A span that does not fit the source, a checkbox state that disagrees
with the parsed model, or a failed post-condition fails closed with
`MapspliceError::UnsafeCompletionEdit`. Phase and step anchors fail with
`MapspliceError::UnsupportedAnchorLevel`, because no checkbox exists at those
levels. An already-checked item returns the original text, so C5 holds and a
configured suffix can never be applied twice. The composition root prints that
unchanged text in preview mode, so a preview redirected to another file is a
complete document even when nothing changed; redirecting onto the target itself
truncates it before it can be read, which is why same-file updates use
`--in-place`. An in-place no-op reports success without a file write.

Done-marker placement keeps the repository Markdown formatter stable when the
title's existing wrapping already satisfies `make check-fmt`: the marker is
appended to the title's final line, and only its own words wrap, at 80 columns,
onto continuation lines indented to the title's content column. The decision to
patch source spans instead of rendering canonically, and to keep the marker
optional, is recorded in
[ADR 0002](adrs/adr-0002-in-place-checkbox-completion.md).
