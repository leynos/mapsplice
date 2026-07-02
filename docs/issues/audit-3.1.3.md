# Post-step codebase audit after 3.1.3

## Scope and method

This audit reviewed `origin/main` after roadmap task 3.1.3, with the audit
record written from the `audit-3-1-3` worktree. The task was to find
refactoring opportunities, repeated code, complex conditionals, ergonomic
awkwardness, high similarity, inconsistencies, poor separation of concerns,
command-query segregation violations, and gaps in documentation comments,
developer or user documentation, and test coverage.

Primary references used:

- `AGENTS.md`
- `docs/mapsplice-design.md`
- `docs/developers-guide.md`
- `docs/users-guide.md`
- `docs/documentation-style-guide.md`
- `docs/scripting-standards.md`
- `docs/roadmap.md`
- `leta` skill
- `sem` skill
- `rust-router`, `domain-cli-and-daemons`, `rust-errors`, and
  `rust-unit-testing` skills

Tooling notes:

- Memtrace MCP calls to `list_indexed_repositories` were attempted twice and
  both returned `user cancelled MCP tool call`, so the audit continued with
  bounded branch-local evidence.
- `git donkey audit-3-1-3 origin/main` failed while its GitPython fetch step
  reported `fatal: Could not read from remote repository.` A manual `git fetch`
  worked, and the inspection worktree was created from `origin/main` with
  `git worktree add -b audit-3-1-3`.
- `leta workspace add` succeeded. Later `leta refs` and `leta daemon info`
  attempts failed with `Error: Failed to start daemon`, so exact local text
  search and `sem` were used for affected reference checks.

## Finding 1: Golden corpus still lacks a generated no-op round-trip property

- **Category:** test-gap
- **Severity:** high
- **Location:** `docs/roadmap.md`, task 3.1.2; `tests/roadmap_golden/contracts.rs`;
  `tests/roadmap_properties.rs`

Task 3.1.3 now asserts formatter-stable rendered outputs, but the broader F3
and C5 contract remains only partially covered. The current tests include a
single identity replacement fixture and an exact nested sub-task round-trip
unit test, while roadmap task 3.1.2 still calls for a generated no-op property
over conformant fixtures.

That leaves the fixture corpus without mechanical proof that every conformant
golden target can pass through the parser and renderer byte-identically under a
no-op edit, and without broad proof that the formatter remains a no-op on that
same no-op output.

**Proposed fix:** implement roadmap task 3.1.2 as a property or generated
corpus test over all conformant golden target fixtures. The test should apply a
stable no-op operation, compare rendered output to the original bytes, and run
the same `mdtablefix` plus `markdownlint-cli2 --fix` stability probe used by
the 3.1.3 format-gate helper.

## Finding 2: Task-number validation is duplicated between target and fragment parsers

- **Category:** duplication
- **Severity:** low
- **Location:** `src/roadmap/parse/document.rs:223` and
  `src/roadmap/parse/fragment.rs:239`

`validate_task_numbers` appears twice with the same loop, the same ownership
check, and the same error message. One copy validates target-roadmap tasks and
the other validates step-fragment tasks, but the rule itself is domain-level:
a task number must belong to its containing step.

Keeping the rule duplicated makes future diagnostic, telemetry, or grammar
changes easy to apply to one parser and forget in the other.

**Proposed fix:** move the shared predicate into a small parse-domain helper,
for example `validate_tasks_belong_to_step`, and call it from both target and
fragment parsing. Preserve the current error text and add a focused unit test
for the helper so the parser tests do not have to duplicate that branch.

## Finding 3: Rendering silently drops inconsistent sub-task child references

- **Category:** cqs
- **Severity:** medium
- **Location:** `src/roadmap/render.rs:90`

`render_task` treats `TaskChild::SubTask(identity)` as a lookup into
`task.sub_tasks`, but when the identity is missing it simply emits nothing for
that child. That converts an internal model inconsistency into silent data
loss in the rendered roadmap.

The parser normally creates both collections together, and sub-task operations
refresh child ordering, so this is an invariant breach rather than expected
user input. The renderer is still the fail-closed boundary for emitted bytes;
dropping content there conflicts with the design's collateral-corruption
threat model.

**Proposed fix:** make the lookup explicit and fallible. If a child identity is
absent from `task.sub_tasks`, return a typed `MapspliceError::InvalidRoadmap`
or introduce an internal invariant error variant with enough context to locate
the parent task and missing identity. Add a regression test that constructs an
inconsistent `TaskEntry` and asserts rendering fails instead of omitting the
child.

## Finding 4: Public API Rustdoc is missing required examples

- **Category:** docs-gap
- **Severity:** low
- **Location:** `src/lib.rs:20`, `src/lib.rs:51`, and `src/roadmap/parse/mod.rs:37`

`AGENTS.md` requires function documentation to include clear examples. Some
public APIs meet that bar, such as `parse_anchor`, but key exported APIs such
as `run_from_args`, `run_request`, and `parse_roadmap` only describe errors.

These functions are the library entry points listed in the developers' guide.
Without examples, downstream users and maintainers have to infer argument
shape, output mode, and failure handling from tests or implementation details.

**Proposed fix:** add compact doctest examples to the public entry points. Keep
the examples filesystem-light: use `parse_roadmap` for a pure parse example,
and use a temporary directory for `run_from_args` or `run_request` only where
the CLI workflow needs file paths. Gate with `cargo test --doc` and the normal
workspace tests.

## Finding 5: Markdown maintenance guidance conflicts with path-scoped workflow

- **Category:** inconsistency
- **Severity:** medium
- **Location:** `docs/developers-guide.md:126` and `docs/roadmap.md:161`

The developers' guide currently tells contributors to run `make fmt` for
Markdown changes. The active agent instructions and roadmap task 4.2.2 both
recognize that repository-wide Markdown formatting causes unrelated churn and
that contributors need path-scoped Markdown maintenance.

Until task 4.2.2 lands, the guide points maintainers at a command that the
workflow explicitly tells agents to avoid for narrow documentation changes.
That makes the documentation source of truth internally inconsistent.

**Proposed fix:** update the developers' guide after or alongside task 4.2.2
to document the path-scoped formatter and lint commands. If the Makefile target
does not exist yet, add it under task 4.2.2 and then replace the current
Markdown-change command block with the scoped workflow.

## Proposed roadmap items

### Complete corpus-wide no-op round-trip coverage

- **Severity:** high
- **Rationale:** The F3 and C5 guarantees remain unproven across the golden
  corpus until task 3.1.2 adds a generated no-op property.

### Consolidate parse-domain validation helpers

- **Severity:** low
- **Rationale:** Shared numbering invariants should have one implementation so
  target and fragment parsers cannot drift.

### Fail closed on renderer model-invariant breaches

- **Severity:** medium
- **Rationale:** Rendering must not silently omit model children when internal
  task and sub-task ordering data becomes inconsistent.

### Bring public API Rustdoc examples up to project standard

- **Severity:** low
- **Rationale:** The library APIs listed in the developers' guide should show
  executable examples, matching `AGENTS.md` documentation policy.

### Document path-scoped Markdown maintenance

- **Severity:** medium
- **Rationale:** Maintainers need guide-backed commands that avoid
  repository-wide formatter churn during narrow documentation edits.
