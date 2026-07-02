# Mapsplice roadmap

This roadmap drives `mapsplice` to the fidelity and contract guarantees set out
in `mapsplice-design.md`. It is itself written in the roadmap grammar
`mapsplice` edits, so it doubles as a living conformance fixture. Work
fix-debt-first: the known reference-rewrite divergence comes before the corpus
that locks the guarantees in.

## 1. Guarantee reference-rewrite fidelity

Idea: a splice must never corrupt a number it was not asked to move. Only
dependency references are rewritten; section references, versions, and prose
numbers are preserved exactly.

### 1.1. Define and enforce the dependency-reference model

This step answers whether `mapsplice` can tell a dependency reference apart
from an incidental number — a section reference, a version, or a quantity —
rather than rewriting every number-shaped token.

- [x] 1.1.1. Specify the dependency-reference predicate in code.
  - Define the dependency context (the `Requires` clause) and the anchor-token
    rules from the design, as a single predicate that decides whether an anchor
    token is a dependency reference.
  - Keep invalid/version-like text separate from valid unresolved dependency
    references: `Requires 1.4.0` is preserved because `0` is not a positive
    anchor component, while `Requires 99.1.1` is a valid dependency-reference
    candidate that is later reported when no renumber-plan mapping exists.
  - See mapsplice-design.md, "The dependency-reference model".
  - Success: a documented predicate classifies anchor tokens; unit tests cover
    `Requires` clauses, invalid version-like values such as `1.4.0`, valid
    unresolved dependency references such as `99.1.1`, section references,
    versions, and prose numbers without treating those boundaries as the same
    classification path.
- [x] 1.1.2. Scope reference rewriting to dependency contexts.
  - Requires 1.1.1.
  - Rewrite only dependency references; preserve every incidental anchor token,
    including section references preceded by a sigil and version strings.
  - Success: an edit that shifts later numbers rewrites every `Requires`
    reference to the moved items, and leaves a section reference, a version such
    as 1.4.0, and prose numbers unchanged.
- [x] 1.1.3. Pin the corruption cases with regression fixtures.
  - Requires 1.1.2.
  - Add golden fixtures for the adversarial reference cases drawn from the
    design.
  - Success: fixtures cover section-reference preservation, version
    preservation,
    substring non-match, and multi-id `Requires` lists, each failing before the
    scope fix and passing after.

## 2. Model addenda as first-class items

Idea: nested sub-tasks are part of the roadmap structure, not opaque task body
text, so they must renumber and render like every other item.

### 2.1. Renumber and render addendum sub-tasks faithfully

This step answers whether a fourth-level sub-task tracks its parent through a
renumber and survives rendering with its nesting intact.

- [x] 2.1.1. Represent addendum sub-tasks in the roadmap model.
  - Extend the model so a task owns ordered fourth-level sub-tasks, each with
    its
    own identity and number, parsed from nested numbered list items.
  - See mapsplice-design.md, "The roadmap grammar".
  - Success: parsing a task with nested `8.2.3.1`-style items yields first-class
    sub-task items, and rendering them is byte-identical to the source.
- [x] 2.1.2. Renumber sub-tasks with their parent.
  - Requires 2.1.1.
  - Include the fourth level in the renumber plan so a sub-task follows its
    parent's new number, and rewrite dependency references to it.
  - Success: when a parent task moves from 8.2.3 to 9.2.3 its sub-task moves
    from
    8.2.3.1 to 9.2.3.1, and references to the sub-task are rewritten to match.
- [x] 2.1.3. Preserve sub-task nesting and indentation on render.
  - Requires 2.1.1.
  - Emit sub-tasks at the correct nesting depth without breaking them out of
    their parent list.
  - Success: a round-trip of a document containing nested sub-tasks is
    byte-identical and the rendered output is gate-clean.

## 3. Establish the fidelity and fixture corpus

Idea: every guarantee is pinned by a golden fixture and a property test, so a
regression is a failing test rather than a silent corruption.

### 3.1. Build the golden-fixture corpus and the round-trip property

This step answers whether the fidelity and contract guarantees are enforced
mechanically rather than by inspection.

- [x] 3.1.1. Assemble the grammar-surface and per-contract golden fixtures.
  - Requires 1.1.3 and 2.1.3.
  - Add one input-and-expected fixture per operation and per guarantee, covering
    the preamble, phases, steps, tasks, multi-line bodies, nested bullets,
    tables, and code blocks.
  - See mapsplice-design.md, "Fixture and test requirements".
  - Success: the corpus exercises every operation and every fidelity and
    contract
    guarantee, compared exactly.
  - [x] 3.1.1.1. Document exact fixture EOF whitespace policy.
    - Addendum (from review:3.1.1; low). Document when raw-byte golden
      fixtures may preserve EOF whitespace and how reviewers distinguish
      intentional fixture fidelity from accidental whitespace churn.
      Lightweight addendum pass.
  - [x] 3.1.1.2. Consolidate the golden fixture harness.
    - Addendum (from audit:3.1.1; medium). Refactor golden fixture case
      construction into a parameterized harness that preserves named cases
      while reducing repeated helper and assertion edits. Lightweight addendum
      pass.
- [x] 3.1.2. Add a no-op round-trip property test.
  - Requires 3.1.1.
  - For any conformant fixture, a no-op edit must render byte-identical output.
  - Success: the property holds across the corpus, and a second formatter pass
    on
    rendered output produces no diff.
- [x] 3.1.3. Assert gate-clean rendered output.
  - Requires 3.1.1.
  - Rendered fixtures must pass the house Markdown gates and be stable under the
    house formatter.
  - Success: `make markdownlint` passes on rendered fixtures and `mdformat-all`
    is a no-op on them.
  - [x] 3.1.3.1. Add ordered-list body gate fixture.
    - Addendum (from review:3.1.3; low). Add a rendered-output gate fixture
      with an ordinary ordered list in a task body, pinning the
      `mdtablefix --renumber` interaction. Lightweight addendum pass.
  - [x] 3.1.3.2. Clarify F1/F4 normalization boundaries.
    - Addendum (from review:3.1.3; medium). Document whether exact
      preservation applies only to gate-clean input or whether successful
      rendering may normalize formatter-unstable input. Lightweight addendum
      pass.
- [ ] 3.1.4. Pin no-op behaviour for formatter-unstable accepted input.
  - Requires 3.1.2 and 3.1.3.2.
  - Add adversarial accepted-input cases that make the F1/F4 trade-off explicit
    when `markdownlint-cli2 --fix` would change indentation or spacing.
  - Success: the no-op corpus records whether such input is preserved
    byte-for-byte or normalized, and the generated property enforces the
    documented boundary.

## 4. Harden strictness and diagnostics

Idea: a roadmap editor that cannot do what was asked must fail clearly, never
emit a damaged document.

### 4.1. Strengthen fail-closed behaviour and reference diagnostics

This step answers whether malformed input and unresolved edits produce clear,
typed errors rather than partial or mangled output.

- [x] 4.1.1. Audit the operations for fail-closed behaviour.
  - Requires 3.1.1.
  - Confirm each operation rejects malformed grammar and level mismatches before
    emitting output, and writes in place only on success.
  - Success: behavioural tests assert typed errors for malformed grammar, a
    level
    mismatch, and a missing anchor, with no partial output and no in-place write
    on failure.
- [x] 4.1.2. Report unresolved dependency references.
  - Requires 1.1.2.
  - Surface a `Requires` reference that resolves to no known item after an edit,
    so a dangling dependency is visible rather than silent.
  - Success: an edit that leaves a dangling `Requires` reference reports it, and
    a fixture pins the diagnostic.
- [x] 4.1.3. Single-source parse-domain task-number validation.
  - Requires 4.1.1.
  - Extract the shared task-belongs-to-step invariant used by target and
    fragment parsing into one parse-domain helper while preserving current
    diagnostics.
  - Success: target and fragment parsers use the same helper, and focused tests
    pin the shared invariant and unchanged error text.
- [ ] 4.1.4. Fail closed on renderer model-invariant breaches.
  - Requires 4.1.1.
  - Make rendering fail with a typed error when task child ordering references a
    missing sub-task instead of silently omitting content.
  - Success: an inconsistent task model produces a diagnostic and no rendered
    roadmap bytes are emitted.

### 4.2. Make documentation gates deterministic and scope-aware

This step answers whether documentation validation can remain a trustworthy
signal when CI runs Mermaid checks concurrently and contributors need narrow
Markdown maintenance without unrelated formatter churn. Its outcome informs how
later documentation-heavy tasks prove gate cleanliness without destabilizing
review.

- [ ] 4.2.1. Make Mermaid validation deterministic under CI concurrency.
  - Requires 3.1.1.
  - Stabilize the `make nixie` path so unchanged diagrams do not time out under
    CI concurrency when serial validation passes.
  - Success: concurrent and serial Mermaid validation give repeatable pass/fail
    results on the existing documentation corpus.
- [ ] 4.2.2. Add path-scoped Markdown maintenance targets.
  - Requires 4.2.1.
  - Add documented targets or variables that run Markdown formatting and linting
    on caller-supplied paths without repo-wide formatter churn.
  - Success: maintainers can format and lint one or more named Markdown files
    through Makefile-supported commands, and unchanged Markdown files outside
    those paths are left untouched.

### 4.3. Reconcile public API documentation with maintainer expectations

This step answers whether the library entry points documented for maintainers
are explained with executable examples rather than forcing readers to infer
usage from tests. Its outcome informs whether public API documentation can be
treated as part of the maintained contract.

- [ ] 4.3.1. Bring public API Rustdoc examples up to project standard.
  - Requires 3.1.3.
  - Add compact executable Rustdoc examples for the public APIs listed in the
    developers' guide, keeping filesystem-heavy flows isolated to temporary
    paths.
  - Success: `run_from_args`, `run_request`, and `parse_roadmap` demonstrate
    typical usage and `cargo test --doc` passes.
