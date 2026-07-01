# Scope reference rewriting to dependency contexts

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

Roadmap task 1.1.2 makes `mapsplice` honour the documented reference-rewrite
boundary: an edit may rewrite dependency references that point at renumbered
roadmap items, but it must not rewrite incidental numbers such as section
references, semantic versions, or prose quantities. A successful implementation
is visible by deleting or inserting a roadmap item and observing that
`Requires` references to moved items change while `§3.2`, `1.4.0`, and ordinary
numeric text remain byte-for-byte intact.

This plan covers only roadmap task 1.1.2, "Scope reference rewriting to
dependency contexts." It must not begin roadmap task 1.1.3 golden fixtures or
roadmap task 4.1.2 unresolved-reference diagnostics.

## Context and orientation

The repository is a Rust command-line tool. The relevant source files are:

- `src/roadmap/ops/mod.rs`, where `apply_command` stages the operation,
  renumbers the staged roadmap, rewrites dependencies, and commits the staged
  document back to the caller.
- `src/roadmap/ops/rewrite.rs`, where `renumber_document` builds the
  `RenumberPlan` and `rewrite_dependencies` walks roadmap model Markdown nodes.
- `src/roadmap/ops/dependency_text.rs`, where text-node anchor candidates are
  classified and rewritten.
- `src/roadmap/model.rs`, where `RenumberPlan::resolve` and
  `RenumberPlan::resolve_unique` define local and cross-source anchor mapping.
- `tests/roadmap_ops.rs`, `tests/roadmap_properties.rs`,
  `tests/features/mapsplice.feature`, and `tests/steps/cli_steps.rs`, which
  hold the current unit, property, and behaviour-driven test coverage.

The documented source of truth is:

- `AGENTS.md`, especially "Code Style and Structure", "Change Quality &
  Committing", "Rust Specific Guidance", "Testing", "Error Handling", and
  "Markdown Guidance".
- `docs/mapsplice-design.md` sections 3, 5, 6, 7, 8, and 9. Section 7 is the
  normative dependency-reference model. Section 9 records the current
  divergence D1, "Unscoped reference rewriting".
- `docs/developers-guide.md` sections 2, 3, 6, and 7. Section 6 specifically
  names `classify_dependency_reference` in `src/roadmap/ops/dependency_text.rs`
  as the layered coverage point for this family of work.
- `docs/users-guide.md` sections "The roadmap shape `mapsplice` expects",
  "Command overview", "Worked example", and "Validation rules and failure
  cases".
- `docs/contributing.md` section "Development gates".
- `docs/documentation-style-guide.md` sections "Spelling", "Markdown rules",
  and "Formatting".
- `docs/scripting-standards.md` only if a script is added or changed; this plan
  should not need script changes.
- `docs/roadmap.md` task 1.1.2 and its dependency on completed task 1.1.1.

The task depends on roadmap task 1.1.1. Branch-local inspection found the
classifier names already present in `src/roadmap/ops/dependency_text.rs`:
`DependencyReferenceClassification`, `classify_dependency_reference`,
`is_dependency_anchor`, `has_dependency_clause_separator`, and
`next_anchor_candidate`. Treat that as branch-local evidence only. At the start
of implementation, re-run the searches in this plan and run the red tests
before deciding whether production code still needs a change.

## Research and verified mechanisms

Memtrace is required as the primary main-branch code-search tool, but the MCP
call failed again in planning round 2. The attempted command was
`mcp__memtrace.list_indexed_repositories({})`; the host returned
`user cancelled MCP tool call`. This is recorded as a tooling failure, not a
product blocker. Implementation must try Memtrace again first. If repo_id
`mapsplice` appears, each production-edit work item must call `find_symbol`,
`get_symbol_context`, `get_impact`, and `get_timeline` for `rewrite_text_value`,
`classify_dependency_reference`, and `rewrite_dependencies` before editing
those load-bearing symbols. Use the `file_path` and `scope_path` returned by
`find_symbol` for `get_timeline`. If Memtrace fails again with the same host
cancellation, use bounded branch-local evidence from Leta or precise file
inspection and record the failure in this ExecPlan.

Leta is required for branch-local symbol navigation. Planning round 2 added the
workspace successfully and located the load-bearing symbols with:

```bash
leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-2
leta grep 'rewrite_text_value|classify_dependency_reference|rewrite_dependencies' \
  -k function,method --head 50
```

The returned locations were `src/roadmap/ops/dependency_text.rs:15` for
`rewrite_text_value`, `src/roadmap/ops/dependency_text.rs:62` for
`classify_dependency_reference`, and `src/roadmap/ops/rewrite.rs:68` for
`rewrite_dependencies`. Later `leta show`, `leta refs`, and `leta calls`
attempts failed with:

```plaintext
Error: Failed to start daemon
```

This is recorded as a tooling failure, not a product blocker. Implementation
must try Leta again first. If `leta show`, `refs`, or `calls` still fails, use
precise file inspection for the branch-local verification step and record the
failure in this ExecPlan.

Sem was available. `sem diff --from origin/main --to HEAD --format json`
reported only the already-committed ExecPlan file added on this branch during
planning round 2. That means this revision starts from a clean code diff against
`origin/main`, not that `origin/main` already satisfies task 1.1.2.

Firecrawl was required for external documentation research, but the MCP scrape
call failed again in planning round 2. The attempted call was
`mcp__firecrawl.firecrawl_scrape` for
`https://docs.rs/markdown/1.0.0/markdown/mdast/index.html`; the host returned
`user cancelled MCP tool call`. The plan therefore pins load-bearing external
behaviour to the locked crates' local rustdoc/source in the shared Cargo
registry. Implementation should retry Firecrawl for docs.rs pages before
editing, but must not block solely on the same host cancellation.

The plan does not add dependencies. It uses these locked crates and verified
APIs:

- `markdown` is locked to `1.0.0` in `Cargo.lock`. The local crate source
  defines `markdown::to_mdast` as returning `mdast::Node` from a parsed source
  string, and its example shows headings containing `Text` nodes. The local
  source also defines `mdast::Node::Text(Text)`, parent nodes with mutable
  `children`, and `Text { value: String, position: Option<Position> }`.
  Verified files:
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/lib.rs`
  lines 140-164 and
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/markdown-1.0.0/src/mdast.rs`
  lines 168-221, 352-402, and 827-842. Therefore the implementation should
  keep using mdast traversal and mutate only `Node::Text.value`; it must not
  introduce raw Markdown string replacement across whole documents.
- `rstest` is locked to `0.26.1`. Its local crate rustdoc says `#[rstest]`
  supports fixture injection and parameterized cases with `#[case]`. Verified
  file:
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rstest-0.26.1/src/lib.rs`
  lines 20-27, 102-110, and 162-180. Therefore focused unit and operation
  tests should continue to use existing `rstest` patterns.
- `proptest` resolves to locked version `1.11.0` through `Cargo.lock`, while
  `Cargo.toml` asks for the caret requirement `proptest = "1.9.0"`. The local
  crate source exports `proptest!`, `prop_assert!`, and `prop_assert_eq!` from
  its prelude, and the macros return test-case failures instead of plain
  panics. Verified files:
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/prelude.rs`
  lines 23-30 and
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/sugar.rs`
  lines 99-150 and 708-810. Therefore generated preservation cases should use
  the existing `proptest::prelude::*` style in `tests/roadmap_properties.rs`.
- `rstest-bdd` and `rstest-bdd-macros` are locked to `0.5.0`. The local
  sources expose `StepContext`, step registry helpers, and the `#[scenario]`,
  `#[given]`, `#[when]`, and `#[then]` attribute macros. Verified files:
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rstest-bdd-0.5.0/src/lib.rs`
  lines 48-63 and
  `/home/leynos/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rstest-bdd-macros-0.5.0/src/lib.rs`
  lines 56-155. Therefore the behavioural test should follow the existing
  `tests/features/mapsplice.feature`, `tests/behaviour_cli.rs`, and
  `tests/steps/cli_steps.rs` pattern.

## Constraints

- Work only inside
  `/home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-2`.
- Do not edit the root/control worktree.
- Do not begin implementation until this draft ExecPlan is approved by the
  roadmap workflow.
- Preserve the existing public API unless an approved later revision says
  otherwise. In particular, do not change `run_from_args`, `run_request`,
  `parse_anchor`, `RoadmapOperation`, or `RenumberPlan` public visibility.
- Do not add an external dependency. Existing locked crates are enough.
- Preserve the design's current dependency-context limit: only `Requires` is a
  rewrite context. `Blocks`, `See`, and similar clauses are future extensions
  and must not be rewritten in this task.
- Keep unresolved valid dependency references unchanged. Reporting dangling
  dependencies belongs to roadmap task 4.1.2, not 1.1.2.
- Keep roadmap task 1.1.3 golden fixtures out of scope. This task may add
  focused unit, behavioural, and property tests, but the adversarial fixture
  corpus remains the next roadmap item.
- Follow Red-Green-Refactor inside each work item. The red step is allowed to
  fail locally before production changes, but only a gate-clean state may be
  committed.
- Use en-GB-oxendict spelling in comments, docs, and commits.
- Format only changed Markdown files. Do not run `make fmt`,
  `mdformat-all`, or any repo-global formatter during this task.
- Do not run formatters, lints, or tests in parallel. Capture long command
  output with `tee` under `/tmp`.
- Every command that pipes a gate through `tee` must run in a shell with
  `set -o pipefail` active in the same command block. Do not copy a tee'd gate
  pipeline out of its block without preserving `pipefail`; otherwise `tee` can
  mask a failing producer command.

## Tolerances

- Scope: if the implementation needs changes outside
  `src/roadmap/ops/dependency_text.rs`, `src/roadmap/ops/rewrite.rs`,
  `tests/roadmap_ops.rs`, `tests/roadmap_properties.rs`,
  `tests/features/mapsplice.feature`, `tests/behaviour_cli.rs`,
  `tests/steps/cli_steps.rs`, `docs/users-guide.md`, `docs/developers-guide.md`,
  `docs/roadmap.md`, and this ExecPlan, stop and record the reason before
  escalating.
- Size: if production Rust changes exceed 120 net lines, stop and escalate.
- Interface: if a public API signature or CLI syntax must change, stop and
  escalate.
- Dependencies: if a new dependency appears necessary, stop and escalate.
- Semantics: if the design appears to require rewriting any non-`Requires`
  context, stop and escalate because that conflicts with
  `docs/mapsplice-design.md` section 10.
- Testing: if a focused red test cannot be made to fail for the intended
  reason after two attempts, record the branch-local finding and proceed only
  if the existing production code already satisfies the asserted behaviour.
- Gates: if `make all`, `make markdownlint`, or `make nixie` fails twice for
  reasons attributable to this task, stop and escalate with the logs.

## Risks

- Risk: roadmap task 1.1.1 may already have implemented most of the production
  path needed for 1.1.2. Severity: medium. Likelihood: medium. Mitigation:
  start with red tests that express the full task 1.1.2 contract. If they
  already pass, record that as a discovery and make the commit a coverage and
  documentation commit rather than forcing production churn.

- Risk: section references, versions, and prose numbers can appear in the same
  text node as a valid `Requires` reference. Severity: high. Likelihood: high.
  Mitigation: include mixed-string tests such as
  `See §2.1. Released 1.4.0. Requires 2.1.1.` where only the `Requires` anchor
  changes.

- Risk: text scanning may accidentally treat a later sentence after `Requires`
  as part of the same clause. Severity: medium. Likelihood: medium. Mitigation:
  keep or add tests around sentence terminators, semicolons, and newlines
  before accepting a production change.

- Risk: formatter or Markdown lint commands could touch unrelated files.
  Severity: medium. Likelihood: medium. Mitigation: every work item below uses
  an explicit Markdown path list scoped to that work item. Do not replace those
  commands with a changed-Markdown discovery pipeline, because this worktree
  may already contain unrelated Markdown changes from another agent or a
  partially staged state.

## Plan of work

Work item 1: Pin dependency-text scoping at the library boundary.

Read before editing: `AGENTS.md` "Rust Specific Guidance" and "Testing";
`docs/mapsplice-design.md` sections 5, 6, 7, and 9; `docs/developers-guide.md`
sections 2, 3, and 6; `docs/roadmap.md` task 1.1.2; and this ExecPlan's
research section. Load skills: `leta`, `rust-router`, `rust-types-and-apis`, and
`rust-unit-testing`. Use Memtrace `list_indexed_repositories` first. If repo_id
`mapsplice` is available, run `find_symbol`, `get_symbol_context`,
`get_impact`, and `get_timeline` for all three load-bearing rewrite symbols
before production edits: `rewrite_text_value`, `classify_dependency_reference`,
and `rewrite_dependencies`. Use the `file_path` and `scope_path` returned by
`find_symbol` for each `get_timeline` call. If Memtrace is unavailable, record
the exact failure and continue with Leta. Use Leta `show`, `refs`, and `calls`
for branch-local confirmation if Leta works; if it does not, use bounded file
inspection.

Add focused `rstest` cases in `src/roadmap/ops/dependency_text.rs` for:

- a `Requires` clause that includes two moved dependency references,
- a section-sigil reference inside nearby prose that must classify as
  `NotDependencyReference`,
- a semantic-version-like token in `Requires` that must classify as
  `InvalidDependencyToken`,
- a valid unresolved `Requires 99.1.1` token that must classify as
  `Reference` but remain unchanged when no mapping exists,
- sentence and semicolon terminators that end the dependency context.

Add or extend a focused operation test in `tests/roadmap_ops.rs` named
`dependency_reference_delete_preserves_incidental_numbers_and_rewrites_requires`
that deletes phase 1 from a two-phase roadmap whose surviving task contains:

```markdown
See §2.1. Released 1.4.0. Count 27. Requires 2.1.1, 2.1.1.
```

The expected output must preserve `§2.1`, `1.4.0`, and `27`, while rewriting
both `Requires 2.1.1` occurrences to `1.1.1`.

Every new test added in this work item must include `dependency_reference` in
its test name or `rstest` case name. That makes the focused red and green
commands below concrete and ensures the operation regression is executed before
this work item is committed.

Red command, run before production edits:

```bash
set -o pipefail
cargo test --workspace --all-targets --all-features dependency_reference \
  2>&1 | tee /tmp/test-dependency-reference-red-mapsplice-roadmap-1-1-2.out
```

Expected red result if the branch still has the D1 defect: the new operation
test fails because an incidental section reference, version, or prose number
changed. If the new tests already pass, record the exact command and outcome in
`Surprises & Discoveries`, and do not alter production code for this work item.

If red fails for the intended reason, make the minimal production change in
`src/roadmap/ops/dependency_text.rs`. Keep the rewrite mechanism as mdast
text-node traversal plus dependency-context classification. Do not move the
boundary into raw Markdown string rewriting. The expected final mechanism is:

1. `rewrite_dependencies` in `src/roadmap/ops/rewrite.rs` continues to walk the
   roadmap model's Markdown nodes.
2. `rewrite_node` continues to call `rewrite_text_value` only for
   `markdown::mdast::Node::Text`.
3. `rewrite_text_value` finds anchor-shaped candidates and asks
   `classify_dependency_reference` whether each candidate is a valid dependency
   reference.
4. `classify_dependency_reference` returns `Reference(anchor)` only when the
   candidate is in the current dependency context, not immediately preceded by
   `§`, and accepted by `parse_anchor`.
5. A `Reference(anchor)` is resolved through `RenumberPlan::resolve` and then
   `RenumberPlan::resolve_unique`; unresolved references are copied unchanged.

Green command:

```bash
set -o pipefail
cargo test --workspace --all-targets --all-features dependency_reference \
  2>&1 | tee /tmp/test-dependency-reference-green-mapsplice-roadmap-1-1-2.out
```

After the focused green command passes, run the required full code gate before
committing this code/test work item:

```bash
set -o pipefail
make all 2>&1 | tee /tmp/all-work-item-1-mapsplice-roadmap-1-1-2.out
```

If this work item also updates this ExecPlan or another Markdown file, format
only the known Markdown path for this work item and run the Markdown gates
before committing. Work item 1's only planned Markdown path is this ExecPlan:

```bash
mdtablefix docs/execplans/roadmap-1-1-2.md
markdownlint-cli2 --fix docs/execplans/roadmap-1-1-2.md
set -o pipefail
make markdownlint 2>&1 | tee /tmp/markdownlint-work-item-1-mapsplice-roadmap-1-1-2.out
make nixie 2>&1 | tee /tmp/nixie-work-item-1-mapsplice-roadmap-1-1-2.out
```

Commit when the focused tests, `make all`, any required Markdown gates, and
`sem diff --format json` show only the intended entities. Suggested commit
subject:

```plaintext
Scope dependency text rewrites
```

Work item 2: Pin scoped rewriting through CLI and property coverage.

Read before editing: `docs/mapsplice-design.md` sections 7 and 8;
`docs/developers-guide.md` section 6; `docs/users-guide.md` "Worked example";
`docs/rstest-bdd-users-guide.md` sections on scenarios, fixtures, and step
definitions if step wiring is unclear; and this ExecPlan's `rstest-bdd` and
`proptest` research notes. Load skills: `leta`, `rust-router`,
`rust-verification`, `rust-unit-testing`, and `proptest`. If this work item
requires any production edit to `rewrite_text_value`,
`classify_dependency_reference`, or `rewrite_dependencies`, repeat the Memtrace
pre-edit sequence for all three symbols when Memtrace is available:
`find_symbol`, `get_symbol_context`, `get_impact`, and `get_timeline`.

Add one behaviour-driven scenario to `tests/features/mapsplice.feature` with
this exact name:

```plaintext
Delete preserves scoped_reference incidental numbers while rewriting Requires dependencies
```

Register that scenario in `tests/behaviour_cli.rs`, and add the smallest new
Given/When/Then step definitions in `tests/steps/cli_steps.rs`. Reuse existing
`CliFixture` and target-writing helpers. The scenario should exercise the
compiled binary path through `run_from_args`, not just the library helper. It
must assert that:

- the command succeeds,
- the moved `Requires` references are rewritten,
- `§2.1`, `1.4.0`, and prose quantity text remain unchanged in standard
  output.

Extend `tests/roadmap_properties.rs` with a generated preservation property
named `scoped_reference_generated_incidental_tokens_are_preserved` for
incidental section references and versions that coexist with a mapped
`Requires` reference. Construct valid roadmaps directly instead of filtering
invalid generated inputs after the fact. Promote any meaningful shrunk failure
to a named `rstest` regression in `tests/roadmap_ops.rs` whose name also
contains `scoped_reference`.

No snapshot test is required for task 1.1.2. The roadmap assigns golden
fixtures and snapshot-like exact fixture comparison to task 1.1.3, so this work
item should not add the fixture corpus prematurely.

Red command, run before production edits from this work item:

```bash
set -o pipefail
cargo test --workspace --all-targets --all-features scoped_reference \
  2>&1 | tee /tmp/test-scoped-reference-red-mapsplice-roadmap-1-1-2.out
```

All new tests in this work item must share the `scoped_reference` substring. If
a framework-generated test name cannot contain that substring, run the concrete
generated test name explicitly and record the command in this plan. The command
should fail before any remaining production fix and pass after the production
state from work item 1 is correct.

Green command:

```bash
set -o pipefail
cargo test --workspace --all-targets --all-features scoped_reference \
  2>&1 | tee /tmp/test-scoped-reference-green-mapsplice-roadmap-1-1-2.out
```

After the focused green command passes, run the required full code gate before
committing this code/test work item:

```bash
set -o pipefail
make all 2>&1 | tee /tmp/all-work-item-2-mapsplice-roadmap-1-1-2.out
```

If this work item also updates this ExecPlan or another Markdown file, format
only the known Markdown path for this work item and run the Markdown gates
before committing. Work item 2's only planned Markdown path is this ExecPlan;
the BDD feature file is not Markdown and must not be passed to Markdown
formatters:

```bash
mdtablefix docs/execplans/roadmap-1-1-2.md
markdownlint-cli2 --fix docs/execplans/roadmap-1-1-2.md
set -o pipefail
make markdownlint 2>&1 | tee /tmp/markdownlint-work-item-2-mapsplice-roadmap-1-1-2.out
make nixie 2>&1 | tee /tmp/nixie-work-item-2-mapsplice-roadmap-1-1-2.out
```

Commit only after the focused behaviour and property tests, `make all`, and any
required Markdown gates pass. Suggested commit subject:

```plaintext
Cover scoped reference rewrites
```

Work item 3: Update user-facing and maintainer documentation.

Read before editing: `docs/documentation-style-guide.md` sections "Spelling",
"Markdown rules", and "Formatting"; `docs/users-guide.md` sections "Worked
example" and "Validation rules and failure cases"; `docs/developers-guide.md`
sections 6 and 7; `docs/roadmap.md` task 1.1.2; and `AGENTS.md` "Documentation
Maintenance". Load skills: `execplans` for keeping this plan current and
`en-gb-oxendict-style` for prose.

Update documentation only after the behaviour is proven by tests. Make the
smallest documentation changes needed:

- In `docs/users-guide.md`, add a short note near the worked example or command
  details that only `Requires` dependency references are rewritten. State that
  section references such as `§2.1`, semantic versions such as `1.4.0`, and
  prose quantities are preserved.
- In `docs/developers-guide.md` section 6, update the layered coverage
  description only if the test layout changed materially.
- In `docs/roadmap.md`, mark task 1.1.2 complete only after work items 1 and 2
  pass their focused tests.
- Keep this ExecPlan's `Progress`, `Surprises & Discoveries`, `Decision Log`,
  and `Outcomes & Retrospective` current.

Format only Markdown files changed by this work item. Work item 3 always updates
`docs/users-guide.md`, `docs/roadmap.md`, and this ExecPlan. It updates
`docs/developers-guide.md` only if the test layout changed materially, so the
developers-guide formatter commands are guarded by a worktree diff check for
that exact file instead of a repository-wide Markdown file discovery pipeline.
The path-safe commands are:

```bash
mdtablefix docs/users-guide.md docs/roadmap.md docs/execplans/roadmap-1-1-2.md
if ! git diff --quiet -- docs/developers-guide.md; then
  mdtablefix docs/developers-guide.md
fi
markdownlint-cli2 --fix docs/users-guide.md docs/roadmap.md docs/execplans/roadmap-1-1-2.md
if ! git diff --quiet -- docs/developers-guide.md; then
  markdownlint-cli2 --fix docs/developers-guide.md
fi
```

Then run:

```bash
set -o pipefail
make markdownlint 2>&1 | tee /tmp/markdownlint-mapsplice-roadmap-1-1-2.out
make nixie 2>&1 | tee /tmp/nixie-mapsplice-roadmap-1-1-2.out
```

Commit when Markdown formatting and documentation gates pass. Suggested commit
subject:

```plaintext
Document scoped dependency rewrites
```

Work item 4: Run final repository gates and record completion evidence.

Read before running: `docs/contributing.md` "Development gates",
`docs/developers-guide.md` section 7, `AGENTS.md` "Change Quality &
Committing", and this ExecPlan's validation section. Load skills: `execplans`,
`leta`, `rust-router`, and `sem`.

Run the full required gates sequentially:

```bash
set -o pipefail
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-1-1-2.out
make markdownlint 2>&1 | tee /tmp/markdownlint-final-mapsplice-roadmap-1-1-2.out
make nixie 2>&1 | tee /tmp/nixie-final-mapsplice-roadmap-1-1-2.out
```

Use `sem diff --format json` and `git diff --check` before the final commit or
handoff. If `make all` fails because of an environment-only issue, record the
exact command, log path, and error text in this ExecPlan before escalating. Do
not mark the task complete on failed gates.

## Concrete steps

1. Confirm the worktree and branch:

   ```bash
   pwd
   git branch --show-current
   git status --short
   ```

   Expected output includes:

   ```plaintext
   /home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-2
   roadmap-1-1-2
   ```

2. Re-attempt Memtrace orientation:

   ```plaintext
   mcp__memtrace.list_indexed_repositories()
   ```

   If `mapsplice` appears, use repo_id `mapsplice` and call `find_code` or
   `find_symbol` for orientation. Before any production edit to the rewrite
   path, call `find_symbol`, `get_symbol_context`, `get_impact`, and
   `get_timeline` for each of `rewrite_text_value`,
   `classify_dependency_reference`, and `rewrite_dependencies`. Pass the
   returned `file_path` and `scope_path` to `get_timeline`. If the host returns
   `user cancelled MCP tool call`, record it in `Surprises & Discoveries` and
   continue with Leta or precise file inspection.

3. Re-attempt Leta setup and branch-local symbol orientation:

   ```bash
   leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-2
   leta grep 'rewrite_text_value|classify_dependency_reference|rewrite_dependencies' \
     -k function,method --head 50
   leta show rewrite_text_value
   leta show classify_dependency_reference
   leta refs rewrite_text_value
   leta refs classify_dependency_reference
   leta show rewrite_dependencies
   leta refs rewrite_dependencies
   leta calls --from apply_command
   ```

   If Leta fails with `Error: Failed to start daemon` or another tooling error,
   record it and use exact file inspection.

4. Run `sem diff --from origin/main --to HEAD --format json` to capture the
   starting semantic diff.

5. Implement work items 1 through 3, committing after each gate-clean work
   item.

6. Run the final validation in work item 4 and update this ExecPlan's living
   sections before the final handoff.

## Validation and acceptance

Task 1.1.2 is accepted when all of the following are true:

- A focused library or operation test proves that deleting or inserting a
  roadmap item rewrites every moved `Requires` reference in the tested clause.
- The same test proves that a nearby section reference such as `§2.1`, a
  semantic version such as `1.4.0`, and ordinary prose numbers are preserved.
- A behaviour-driven CLI scenario proves the same guarantee through the command
  surface.
- A property test generates valid documents that combine incidental numeric
  tokens with a mapped `Requires` reference and proves that only the dependency
  reference changes.
- No test expects `Blocks`, `See`, or arbitrary prose to rewrite.
- Unresolved valid dependency references remain unchanged.
- `docs/roadmap.md` marks 1.1.2 complete only after the tests and production
  state satisfy the contract.

Final validation commands:

```bash
set -o pipefail
make all 2>&1 | tee /tmp/all-mapsplice-roadmap-1-1-2.out
make markdownlint 2>&1 | tee /tmp/markdownlint-final-mapsplice-roadmap-1-1-2.out
make nixie 2>&1 | tee /tmp/nixie-final-mapsplice-roadmap-1-1-2.out
```

`make all` includes `check-fmt`, `lint`, `typecheck`, and `test` on current
`origin/main` policy. Markdown changes additionally require `make markdownlint`
and `make nixie`.

## Idempotence and recovery

All planned edits are source edits and test additions. They are safe to retry
from a clean worktree. If a formatter modifies unrelated Markdown, park that
churn with a named stash:

```bash
git stash push \
  -m 'df12-stash v1 task=1.1.2 kind=discard reason="formatter churn outside scoped rewrite plan"'
```

Do not use a bare `git stash`. Do not discard user changes. If a gate fails,
inspect the relevant `/tmp/*mapsplice-roadmap-1-1-2.out` log and update this
plan before trying again.

## Interfaces and dependencies

The final implementation should keep these internal interfaces intact:

```rust
pub(super) fn rewrite_text_value(
    value: &str,
    source: SourceId,
    plan: &RenumberPlan,
) -> (String, u64)
```

```rust
pub(super) fn rewrite_dependencies(
    roadmap: &mut RoadmapDocument,
    plan: &RenumberPlan,
) -> Result<u64>
```

No public API or CLI signature change is expected. If an implementation appears
to need a public change, stop under the `Interface` tolerance.

## Progress

- [x] 2026-07-01T13:04:30Z Drafted the first planning-round ExecPlan for
  roadmap task 1.1.2.
- [x] 2026-07-01T13:12:00Z Formatted and gated this draft plan with
  `make all`, `make markdownlint`, and `make nixie`.
- [x] 2026-07-01T14:40:00Z Revised the plan for design-review round 2:
  per-work-item `make all` gates, complete Memtrace pre-edit calls, and
  concrete focused test names.
- [x] 2026-07-01T15:35:00Z Revised the plan for design-review round 3:
  replaced dirty-worktree Markdown formatter discovery pipelines with explicit
  per-work-item path lists and a guarded `docs/developers-guide.md` command.
- [x] 2026-07-01T15:45:00Z Formatted and gated the planning-round 3 revision
  with scoped `mdtablefix`, scoped `markdownlint-cli2 --fix`,
  `make markdownlint`, `make nixie`, and `make all`.
- [x] 2026-07-01T16:25:00Z Revised the plan for design-review round 4: every
  tee'd focused, per-work-item, and final validation command now sets
  `pipefail` in the same command block.
- [x] 2026-07-01T16:35:00Z Formatted and gated the planning-round 4 revision
  with scoped `mdtablefix`, scoped `markdownlint-cli2 --fix`,
  `make markdownlint`, `make nixie`, and `make all`, all through fail-closed
  `tee` pipelines.
- [x] 2026-07-01T13:51:18Z Re-attempted Memtrace, Firecrawl, Leta, and
  Sem at implementation start. Memtrace and Firecrawl still returned
  `user cancelled MCP tool call`; Leta still failed with
  `Read-only file system (os error 30)` and `Failed to start daemon`; Sem
  reported only this branch's ExecPlan addition against `origin/main`.
- [x] 2026-07-01T14:12:00Z Work item 1: pinned dependency-text scoping at
  the library boundary.
  Added dependency-reference classifier, text-rewrite, and operation
  regressions. The focused red command unexpectedly passed, so no production
  code was changed for this work item. Focused `dependency_reference`,
  `make all`, `make markdownlint`, and `make nixie` gates passed before commit.
- [x] 2026-07-01T14:24:00Z Work item 2: pinned scoped rewriting through CLI
  and property coverage.
  Added a BDD scenario through the compiled binary and a generated property for
  incidental section references, semantic versions, and prose counts beside a
  mapped `Requires` reference. Focused `scoped_reference` and `make all` gates
  passed before commit.
- [x] 2026-07-01T14:30:00Z Work item 3: updated user-facing and maintainer
  documentation.
  Documented the `Requires`-only rewrite scope, updated the maintainer coverage
  map, and marked roadmap task 1.1.2 complete.
- [x] 2026-07-01T14:10:00Z Work item 4: ran final gates and recorded
  completion evidence.
  Final `make all`, `make markdownlint`, `make nixie`, and CodeRabbit review
  passed; CodeRabbit reported zero findings.

## Surprises & Discoveries

- Observation: Memtrace was unavailable in planning rounds 1, 2, 3, and 4.
  Evidence: `mcp__memtrace.list_indexed_repositories({})` returned
  `user cancelled MCP tool call`. Impact: implementation must retry Memtrace,
  but this plan is not blocked.

- Observation: Leta partially worked in planning round 2, then failed during
  planning round 3 workspace setup. Evidence:
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-2`
  succeeded, and
  `leta grep 'rewrite_text_value|classify_dependency_reference|`
  `rewrite_dependencies' -k function,method --head 50` returned the three
  expected symbols. Later `leta show`, `leta refs`, and `leta calls` attempts
  returned `Error: Failed to start daemon`. In planning rounds 3 and 4,
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-2`
  failed with `Error: IO error: Read-only file system (os error 30)`, and
  `leta files` failed with `Error: Failed to start daemon`. Impact:
  implementation must retry Leta; exact file inspection remains an acceptable
  fallback if the daemon still fails.

- Observation: Firecrawl was unavailable in planning rounds 1, 2, and 4.
  Evidence: `mcp__firecrawl.firecrawl_scrape` for the docs.rs markdown `mdast`
  page returned `user cancelled MCP tool call`. Impact: implementation should
  retry Firecrawl; this plan pins external API behaviour to locked local crate
  source and rustdoc in the meantime.

- Observation: current branch-local source already contains
  `DependencyReferenceClassification` and `classify_dependency_reference`.
  Evidence: `leta grep` and exact file inspection of
  `src/roadmap/ops/dependency_text.rs`. Impact: implementation may find that
  work item 1 is a coverage-only change.

- Observation: Memtrace and Firecrawl were unavailable again at implementation
  start. Evidence: `mcp__memtrace.list_indexed_repositories({})` and
  `mcp__firecrawl.firecrawl_agent` for the docs.rs `markdown` 1.0.0 mdast page
  both returned `user cancelled MCP tool call`. Impact: implementation
  continued with bounded branch-local inspection and the locked local
  crate-source evidence already recorded in this plan.

- Observation: Leta remained unavailable at implementation start.
  Evidence:
  `leta workspace add /home/leynos/Projects/mapsplice.worktrees/roadmap-1-1-2`
  returned `Error: IO error: Read-only file system (os error 30)`, and
  `leta grep`, `leta show`, `leta refs`, and `leta calls` returned
  `Error: Failed to start daemon`. Impact: implementation used exact file
  inspection for the named local symbols and tests.

- Observation: work item 1's focused red command passed before any production
  edit. Evidence:
  `cargo test --workspace --all-targets --all-features dependency_reference`
  passed with 18 focused tests across unit, operation, and property binaries
  after adding the planned coverage. Impact: the existing branch-local rewrite
  implementation already scoped rewrites to dependency contexts; work item 1 is
  a coverage-only change.

## Decision Log

- Decision: keep the rewrite mechanism inside mdast text-node traversal and
  dependency-context classification. Rationale: `docs/mapsplice-design.md`
  requires mdast-based parsing and rejects raw-string surgery; locked
  `markdown` 1.0.0 exposes mutable text-node values and parent-node children
  needed by the existing traversal. Date/Author: 2026-07-01, planning agent.

- Decision: do not rewrite `Blocks` or `See` clauses in task 1.1.2.
  Rationale: `docs/mapsplice-design.md` section 7 says the current dependency
  context is `Requires`; section 10 names other clauses as future extensions.
  Date/Author: 2026-07-01, planning agent.

- Decision: do not add golden fixture corpus work in this task.
  Rationale: `docs/roadmap.md` assigns adversarial golden fixtures to task
  1.1.3, which depends on 1.1.2. Date/Author: 2026-07-01, planning agent.

- Decision: record Memtrace, Leta, and Firecrawl failures without setting
  status to BLOCKED. Rationale: the workflow instructions state advisory-tool
  unavailability is not a valid reason to block when bounded local evidence can
  support planning. Date/Author: 2026-07-01, planning agent.

- Decision: require `make all` before each code/test work-item commit, not only
  at final handoff. Rationale: AGENTS.md and the roadmap workflow require every
  code commit to pass all code commit gateways. Focused red/green tests prove
  the local behaviour, but `make all` is the required commit gate. Date/Author:
  2026-07-01, planning round 2 agent.

- Decision: make every load-bearing Memtrace pre-edit check explicit for
  `rewrite_text_value`, `classify_dependency_reference`, and
  `rewrite_dependencies`. Rationale: these symbols are the rewrite path for
  this task. When Memtrace is available, each must have `get_symbol_context`,
  `get_impact`, and `get_timeline` evidence before production edits.
  Date/Author: 2026-07-01, planning round 2 agent.

- Decision: require concrete focused-test names for the work item 1 operation
  regression. Rationale: the focused `dependency_reference` command must run
  the new `tests/roadmap_ops.rs` regression before the work item 1 commit.
  Naming the test
  `dependency_reference_delete_preserves_incidental_numbers_and_rewrites_requires`
  makes that command path-safe and deterministic. Date/Author: 2026-07-01,
  planning round 2 agent.

- Decision: replace Markdown formatter discovery pipelines with explicit
  per-work-item path lists. Rationale: the worktree is already dirty, and
  changed-Markdown discovery can format Markdown files that belong to another
  work item or another agent. Work items 1 and 2 format only this ExecPlan;
  work item 3 formats its known documentation paths and guards the optional
  `docs/developers-guide.md` path with an exact diff check. Date/Author:
  2026-07-01, planning round 3 agent.

- Decision: make every tee'd gate command fail closed with `set -o pipefail`.
  Rationale: a plain `gate 2>&1 | tee log` pipeline can return success when the
  gate fails and `tee` succeeds. Setting `pipefail` in each validation block
  preserves the producer's failing exit status for focused tests, per-work-item
  gates, Markdown gates, and final gates. Date/Author: 2026-07-01, planning
  round 4 agent.

## Outcomes & Retrospective

Implementation is complete. Work item 1 added dependency-reference coverage for
section-sigil incidental numbers, semicolon clause termination, multiple mapped
`Requires` references, unresolved valid references, and a delete operation that
preserves `§2.1`, `1.4.0`, and `27` while rewriting both `Requires 2.1.1`
occurrences. The focused command
`cargo test --workspace --all-targets --all-features dependency_reference`
passed before any production edit and again after formatting, so no production
code was changed for work item 1.

Work item 2 added compiled-binary BDD coverage and property coverage for the
same scoped rewrite contract. Work item 3 documented the `Requires`-only scope,
updated the maintainer coverage map, and marked roadmap task 1.1.2 complete.
The final deterministic gates and CodeRabbit review passed.

## Artifacts and notes

Planning evidence captured in this draft:

```plaintext
sem diff --from origin/main --to HEAD --format json
{"summary":{"fileCount":0,"added":0,"modified":0,"deleted":0,"moved":0,"renamed":0,"reordered":0,"orphan":0,"total":0},"changes":[]}
```

`Cargo.toml` uses caret requirements; `Cargo.lock` currently resolves
`markdown` to 1.0.0, `rstest` to 0.26.1, `rstest-bdd` and `rstest-bdd-macros`
to 0.5.0, and `proptest` to 1.11.0.

Planning-round validation logs:

```plaintext
/tmp/all-mapsplice-roadmap-1-1-2.out
/tmp/markdownlint-mapsplice-roadmap-1-1-2.out
/tmp/nixie-mapsplice-roadmap-1-1-2.out
```

All three commands completed successfully on 2026-07-01.

Planning-round 2 validation logs:

```plaintext
/tmp/mdtablefix-mapsplice-roadmap-1-1-2.out
/tmp/markdownlint-fix-mapsplice-roadmap-1-1-2.out
/tmp/markdownlint-mapsplice-roadmap-1-1-2-round2.out
/tmp/nixie-mapsplice-roadmap-1-1-2-round2.out
/tmp/mdtablefix-mapsplice-roadmap-1-1-2-round2-final.out
/tmp/markdownlint-fix-mapsplice-roadmap-1-1-2-round2-final.out
/tmp/markdownlint-mapsplice-roadmap-1-1-2-round2-final.out
/tmp/nixie-mapsplice-roadmap-1-1-2-round2-final.out
/tmp/nixie-mapsplice-roadmap-1-1-2-round2-final-retry.out
/tmp/nixie-file-mapsplice-roadmap-1-1-2-round2.out
```

Markdown formatting and Markdown lint completed successfully on 2026-07-01.
`make nixie` passed once in planning round 2, then failed twice while
validating the pre-existing sequence diagram in
`docs/rstest-bdd-users-guide.md` with `diagram 1 timed out`. The changed
ExecPlan file was reached and accepted before that timeout in both failed runs,
and `nixie --no-sandbox docs/execplans/roadmap-1-1-2.md` passed for the changed
file.

Planning-round 3 validation logs:

```plaintext
/tmp/mdtablefix-mapsplice-roadmap-1-1-2-round3.out
/tmp/markdownlint-fix-mapsplice-roadmap-1-1-2-round3.out
/tmp/markdownlint-mapsplice-roadmap-1-1-2-round3.out
/tmp/nixie-mapsplice-roadmap-1-1-2-round3.out
/tmp/all-mapsplice-roadmap-1-1-2-round3.out
```

All five commands completed successfully on 2026-07-01. `make all` ran
format-checking, Rustdoc, Clippy, Whitaker typechecking, `cargo check`,
nextest, and doctests; the test gate reported 77 nextest tests passed and 8
doctests passed with 2 ignored.

Planning-round 4 validation logs:

```plaintext
/tmp/mdtablefix-mapsplice-roadmap-1-1-2-round4.out
/tmp/markdownlint-fix-mapsplice-roadmap-1-1-2-round4.out
/tmp/markdownlint-mapsplice-roadmap-1-1-2-round4.out
/tmp/nixie-mapsplice-roadmap-1-1-2-round4.out
/tmp/all-mapsplice-roadmap-1-1-2-round4.out
```

All five commands completed successfully on 2026-07-01 with `set -o pipefail`
active for every tee'd command. `make all` ran format-checking, Rustdoc,
Clippy, Whitaker typechecking, `cargo check`, nextest, and doctests; the test
gate reported 77 nextest tests passed and 8 doctests passed with 2 ignored.

Revision note: Initial draft for roadmap task 1.1.2. It records unavailable
advisory tools, pins the verified local crate APIs, and decomposes the task
into gate-clean work items for scoped rewrite tests, any minimal production
correction, documentation, and final validation.

Revision note: Planning round 2 resolves the design-review blockers by requiring
`make all` before each code/test work-item commit, adding `get_timeline` and
the missing `get_impact` requirement for every load-bearing rewrite symbol when
Memtrace is available, and naming the work item 1 operation regression
`dependency_reference_delete_preserves_incidental_numbers_and_rewrites_requires`
so the focused `dependency_reference` command must execute it.

Revision note: Planning round 3 resolves the remaining design-review blocker by
removing dirty-worktree Markdown formatter discovery pipelines from work items
1, 2, and 3. The plan now uses explicit Markdown paths per work item, and the
only conditional documentation path, `docs/developers-guide.md`, is handled
with an exact file diff guard.

Revision note: Planning round 4 resolves the design-review blocker about
tee-masked gate failures. Every focused test, per-work-item gate, Markdown
gate, and final validation block that pipes through `tee` now sets
`set -o pipefail` in the same command block, so the plan fails closed when the
producer command fails.

Revision note: Implementation start records repeated Memtrace, Firecrawl, and
Leta tooling failures, then records that work item 1's newly added focused
coverage passed without any production edit. Work item 1 remains open until
delegated gates, CodeRabbit, and the atomic commit are complete.

Manual completion evidence after the workflow implementation adapter failed:

```plaintext
/tmp/test-dependency-reference-green-mapsplice-roadmap-1-1-2-manual.out
/tmp/all-work-item-1-mapsplice-roadmap-1-1-2-manual.out
/tmp/markdownlint-work-item-1-mapsplice-roadmap-1-1-2-manual.out
/tmp/nixie-work-item-1-mapsplice-roadmap-1-1-2-manual.out
/tmp/test-scoped-reference-green-mapsplice-roadmap-1-1-2-manual.out
/tmp/all-work-item-2-mapsplice-roadmap-1-1-2-manual.out
/tmp/markdownlint-work-item-2-mapsplice-roadmap-1-1-2-manual.out
/tmp/nixie-work-item-2-mapsplice-roadmap-1-1-2-manual.out
/tmp/markdownlint-docs-mapsplice-roadmap-1-1-2-manual.out
/tmp/nixie-docs-mapsplice-roadmap-1-1-2-manual.out
/tmp/all-mapsplice-roadmap-1-1-2-manual.out
/tmp/markdownlint-final-mapsplice-roadmap-1-1-2-manual.out
/tmp/nixie-final-mapsplice-roadmap-1-1-2-manual.out
/tmp/coderabbit-manual-mapsplice-roadmap-1-1-2.out
```

Final validation completed on 2026-07-01. `make all` reported 84 nextest tests
passed and 8 doctests passed with 2 ignored. `make markdownlint` reported zero
errors, `make nixie` validated all diagrams, and CodeRabbit completed with
zero findings.
