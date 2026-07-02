# Mapsplice design

## Status and scope

- **Status:** living design document for the `mapsplice` CLI. It describes both
  the current implementation and the target contract the tool is evolving
  towards.
- **Scope:** the roadmap-splicing engine — parsing, the roadmap model, the four
  structural operations, renumbering, dependency-reference rewriting, and
  rendering. CLI plumbing (`ortho-config`, `--in-place`) is covered only where
  it bears on the guarantees below.
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

`mapsplice` applies one structural edit to a roadmap-shaped Markdown document —
`append`, `insert`, `delete`, or `replace` — then renumbers the affected
phases, steps, and tasks and rewrites the dependency references that pointed at
them.

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
```

*Figure 1: The mapsplice edit pipeline.*

The roadmap model (section 4) is the boundary that makes the guarantees
checkable: structural edits operate on typed phase, step, and task items;
renumbering is a pure function of position; and reference rewriting consults a
renumber plan rather than blindly substituting text.

## 4. The roadmap grammar (normative reference)

The accepted grammar is defined normatively in `docs/users-guide.md`. In
summary, a roadmap has an optional preamble followed by three structural levels:

- **Phases** — level-2 headings whose text begins with a phase number (`## 8.`).
- **Steps** — level-3 headings whose text begins with a step number
  (`### 8.2.`).
- **Tasks** — list items whose first inline text begins with a task number
  (`- [ ] 8.2.3.`).

A task may carry continuation paragraphs and nested bullet lists. A nested list
item whose number extends its parent task by one level (`- [ ] 8.2.3.1.`) is an
**addendum sub-task** — a fourth structural level. Anchors on the command line
address these levels directly: `8` a phase, `8.2` a step, `8.2.3` a task,
`8.2.3.1` an addendum sub-task.

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
  tables, and code blocks are unchanged.
- **F2 — Minimal diff.** The only changes are the addressed item itself and the
  deterministic consequences of the edit — the renumbering of later items
  (section 6, C2) and the rewriting of dependency references to them (C3).
- **F3 — Round-trip stability.** Parsing a conformant document and rendering it
  under a no-op edit is the identity, modulo the one documented normalization:
  non-empty rendered roadmaps end in exactly one final newline. Rendering again
  does not add another final newline.
- **F4 — Gate-clean output.** Rendered output passes the house Markdown gates
  (`make markdownlint`) and is stable under the house formatter
  (`mdformat-all`, which runs `mdtablefix` then `markdownlint-cli2 --fix`): a
  second formatting pass produces no diff.
- **F5 — Fail closed.** Malformed input is rejected with a typed, user-facing
  error before any output is produced. `mapsplice` never emits a partially
  rewritten or mangled document, and `--in-place` writes only on success.

## 6. Functional and contract guarantees

- **C1 — Operations.** `append` (phase-level), `insert` (before, or `--after`),
  `delete`, and `replace`, each addressed by an anchor, with strict level
  matching between fragment and anchor.
- **C2 — Renumber contract.** After any edit, phase, step, task, and addendum
  numbers are contiguous from 1, in document order, at every level. No gaps, no
  duplicates, no out-of-order numbering survives an operation.
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
- **Incidental numbers are preserved.** An anchor token that is not in a
  dependency context, or that is immediately preceded by a section sigil (`§`),
  is incidental: it is a section reference, a version, or prose, and it is left
  exactly as written. Scoping rewriting to dependency contexts — rather than
  substituting every number-shaped token in the document — is what upholds F1
  and C3.
- **Resolution.** A dependency reference is resolved against the renumber plan:
  the source-local mapping first, then a unique cross-source mapping when the
  anchor is defined exactly once across the target and the fragment. A valid
  dependency reference that does not resolve is a dangling dependency and the
  operation fails with a typed diagnostic before output is emitted or an
  in-place write occurs.

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

  *Table 1: Required adversarial fixtures for the fidelity and reference
  contracts.*

- **Test shapes.** `rstest` unit fixtures cover the model, renumbering, and the
  reference resolver in isolation; `rstest-bdd` features cover the CLI surface
  and output modes; golden comparison covers render fidelity end to end.
- **Round-trip property.** For any conformant document, a no-op edit renders
  byte-identical output (F3), and a second `mdformat-all` pass produces no diff
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
