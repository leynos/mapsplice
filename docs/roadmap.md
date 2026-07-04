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

- [x] 3.1.4. Pin no-op behaviour for formatter-unstable accepted input.

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

- [x] 4.1.4. Fail closed on renderer model-invariant breaches.

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

- [x] 4.2.1. Make Mermaid validation deterministic under CI concurrency.

  - Requires 3.1.1.
  - Stabilize the `make nixie` path so unchanged diagrams do not time out under
    CI concurrency when serial validation passes.
  - Success: concurrent and serial Mermaid validation give repeatable pass/fail
    results on the existing documentation corpus.

- [x] 4.2.2. Add path-scoped Markdown maintenance targets.

  - Requires 4.2.1.
  - Add documented targets or variables that run Markdown formatting and linting
    on caller-supplied paths without repo-wide formatter churn.
  - Success: maintainers can format and lint one or more named Markdown files
    through Makefile-supported commands, and unchanged Markdown files outside
    those paths are left untouched.
  - [x] 4.2.2.1. Pin scoped Markdown target order and real-tool flags.
    - Addendum (from review:4.2.2; low). Add order/argument assertions and a
      gated real-tool smoke test for the scoped Markdown formatter/linter
      contracts. Lightweight addendum pass.
  - [x] 4.2.2.2. Guard load-bearing Markdown formatter flags.
    - Addendum (from review:4.2.2; low). Fail fast when
      `MARKDOWN_FORMAT_FLAGS` drops `--in-place` from the scoped formatter.
      Lightweight addendum pass.

### 4.3. Reconcile public API documentation with maintainer expectations

This step answers whether the library entry points documented for maintainers
are explained with executable examples rather than forcing readers to infer
usage from tests. Its outcome informs whether public API documentation can be
treated as part of the maintained contract.

- [x] 4.3.1. Bring public API Rustdoc examples up to project standard.

  - Requires 3.1.3.
  - Add compact executable Rustdoc examples for the public APIs listed in the
    developers' guide, keeping filesystem-heavy flows isolated to temporary
    paths.
  - Success: `run_from_args`, `run_request`, and `parse_roadmap` demonstrate
    typical usage and `cargo test --doc` passes.

### 4.4. Close audit-found failure and documentation gaps

This step answers whether the editor's hardening work covers the failure modes
surfaced by the post-4.2.2 audit, rather than only the already-gated happy
paths. Its outcome informs whether maintainers can trust escaped output, failed
in-place writes, and configuration discovery without reading source. See
docs/issues/audit-4.2.2.md.

- [x] 4.4.1. Escape literal Markdown backslashes without losing text.

  - Requires 3.1.3 and 4.1.4.
  - Include literal backslash and exclamation-mark cases in renderer
    round-trip coverage so text nodes cannot be re-read with characters
    silently dropped.
  - Success: rendered roadmap text containing `\` and `!` round-trips through
    the parser without loss, and the formatter-stability corpus remains green.

- [x] 4.4.2. Make failed in-place rewrites leave no temporary files.

  - Requires 4.1.1.
  - Clean up sibling temporary files on write and rename failures while
    preserving the original error.
  - Success: an injected write or rename failure leaves no
    `.mapsplice.tmp` sibling behind and still reports the original failure.

- [x] 4.4.3. Document configuration discovery truthfully.

  - Requires 4.3.1.
  - Reconcile the developers' and users' guides with the implemented global
    and subcommand default-loading paths, including local versus XDG
    precedence and the supported environment variables.
  - Success: the guides name the actual config files, precedence, and
    environment variables without implying every default is loaded through the
    same mechanism.

- [x] 4.4.4. Cover sub-task splice vector alignment.

  - Requires 2.1.2 and 4.1.4.
  - Add focused insert-before, insert-after, and replace cases for sub-task
    operations that mutate both the structural sub-task vector and the
    parallel child ordering.
  - Success: unit tests fail if `sub_tasks` and `children` diverge after
    inserting or replacing a sub-task at boundary ordinals.

## 5. Consolidate parser and model internals

Idea: if the duplicated parser, renderer, and model mutation seams are
single-sourced after the strictness work lands, later roadmap-grammar changes
can be made once instead of revalidating several near-identical code paths.
This is refactor and consolidation work: its value is reduced drift risk and a
lower cost for the next grammar change, not new user-facing behaviour. See
docs/issues/audit-4.2.2.md.

### 5.1. Single-source repeated roadmap-structure machinery

This step answers whether the parser and model share one representation of
checklist-item parsing, fragment-root validation, lookup helpers, and mutation
invariants. Its outcome informs whether future grammar work can be reviewed at
one seam rather than across duplicated branches.

- [x] 5.1.1. Share task and sub-task checklist parsing.

  - Requires 4.4.4.
  - Extract common checklist-head and numbered-prefix parsing helpers for task
    and sub-task items while preserving current diagnostics.
  - Success: task and sub-task item parsers use the same helper path, and the
    existing parse diagnostics and golden fixtures are unchanged.

- [x] 5.1.2. Share fragment-root validation and step accumulation.

  - Requires 5.1.1.
  - Remove the duplicated single-list fragment-root skeleton and reconcile step
    fragment parsing with the document parser's step lifecycle.
  - Success: task, sub-task, and step fragment parsing exercise shared
    validation machinery and all fragment-level behavioural tests pass without
    diagnostic drift.

- [x] 5.1.3. Collapse duplicated lookup and rendering helpers.

  - Requires 5.1.1.
  - Single-source phase lookup, checkbox marker rendering, fragment-level
    routing, and dependency-rewrite recording where the current code repeats
    branch-independent logic.
  - Success: internal call sites use one helper per concept, public behaviour
    and metrics output remain unchanged, and `make all` passes.

- [x] 5.1.4. Encapsulate roadmap mutation invariants.

  - Requires 5.1.1 and 5.1.3.
  - Hide direct mutation of `RenumberPlan` and `TaskChildren` internals behind
    methods that preserve nested-map and ordered-child invariants.
  - Success: callers cannot bypass the invariant-preserving methods, and tests
    pin task-body, sub-task, and dependency-renumber behaviour after the
    encapsulation.

## 6. Add agent-native validation and output contracts

Idea: `mapsplice` should be usable safely by agents and humans before it edits
roadmaps. A dedicated validator, miette-style human diagnostics, and strict
JSON contracts make syntax errors, dependency drift, and broken links visible
without requiring an edit attempt.

### 6.1. Validate roadmap structure, links, and dependencies

This step answers whether maintainers can run one command that proves a roadmap
is syntactically valid, internally linked, and dependency-consistent. Its
outcome informs whether agents can treat validation output as bounded machine
evidence rather than prose-only terminal text.

- [ ] 6.1.1. Add a `validate` subcommand and validation result model.

  - Requires 5.1.4.
  - Extend the parser error channel for multi-finding validation with
    source ranges, then reuse the roadmap model to classify syntax,
    dependency, and link findings without mutating the target document.
  - See docs/validation-and-agent-output-design.md.
  - Success: `mapsplice validate docs/roadmap.md` exits successfully for a
    valid roadmap, reports typed findings for invalid roadmaps, and never
    emits rewritten roadmap Markdown.

- [ ] 6.1.2. Check dependency references and Markdown links.

  - Requires 6.1.1.
  - Validate every `Requires` anchor against the parsed roadmap and validate
    local Markdown links against headings, files, and fragments, with file and
    fragment resolution confined to the workspace repository root. Link checks
    must not follow `../` paths that escape that root or external targets.
  - Success: missing anchors, malformed dependency tokens, missing files, and
    unresolved local heading fragments are reported with stable finding codes.

- [ ] 6.1.3. Render miette-style human diagnostics.

  - Requires 6.1.1.
  - Use source spans, labels, help text, and stable diagnostic codes so human
    output identifies the relevant roadmap line and the repair action.
  - Success: fixtures pin graphical diagnostics for syntax, dependency, and
    link findings, with colour and URL variance disabled for deterministic
    tests.

### 6.2. Align every command with agent-native output contracts

This step answers whether `mapsplice` has one coherent output contract across
edit, validate, and future maintenance commands. Its outcome informs whether
agents can call the tool without scraping human prose or guarding against mixed
stdout payloads.

- [ ] 6.2.1. Add global `--json` output mode.

  - Requires 6.1.1.
  - Return exactly one JSON document on success and one JSON diagnostic
    document on failure, keeping human diagnostics on stderr in non-JSON mode.
  - See
    <https://github.com/leynos/ortho-config/blob/main/docs/agent-native-cli-design.md>
    and
    docs/validation-and-agent-output-design.md.
  - Success: every subcommand has a pinned JSON success schema, failure schema,
    stdout contract, stderr contract, exit-code class, clap-usage fallback,
    and tracing policy.

- [ ] 6.2.2. Emit structured edit summaries for mutating commands.

  - Requires 6.2.1.
  - Summarize operation kind, target path, in-place status, rewritten target,
    inserted or removed anchors, renumber mappings, dependency rewrites, and
    emitted artefact hash.
  - Success: append, insert, delete, and replace can be run with `--json`
    using `--in-place` without losing the rewritten roadmap body or leaking
    non-JSON bytes.

- [ ] 6.2.3. Publish compact agent context for the CLI.

  - Requires 6.2.1.
  - Add a command or documented output that lists supported commands,
    mutation boundaries, output modes, exit classes, and examples in a compact
    machine-readable shape.
  - Success: the generated context names `validate`, all edit commands, and
    the `--json` contract in a schema-versioned payload.

- [ ] 6.2.4. Package an agent skill for roadmap maintenance.

  - Requires 6.1.3 and 6.2.3.
  - Author a skill that tells agents when to run validation, how to prefer
    JSON output, how to interpret miette diagnostics, and when to refuse
    unsafe roadmap edits.
  - Success: the skill is tested against the generated agent context, requires
    validate-before-edit and validate-after-edit, directs agents to inspect
    JSON diagnostics by `code` and `exit_class`, and stops for maintainer
    judgement on ambiguous links, duplicate anchors, or failed postflight
    checks.

## 7. Renumber ExecPlans after roadmap updates

Idea: roadmap task numbers are embedded in ExecPlan filenames, headings, links,
and historical references. When `mapsplice` renumbers a roadmap, it should be
able to plan and apply the corresponding ExecPlan renumbering without erasing
history or creating ambiguous references.

### 7.1. Discover and plan ExecPlan renumbering

This step answers whether `mapsplice` can identify which ExecPlans are tied to
renumbered roadmap tasks before it changes any files. Its outcome informs the
boundary between safe automated maintenance and cases that require maintainer
review.

- [ ] 7.1.1. Model roadmap-to-ExecPlan identity.

  - Requires 6.2.2.
  - Define how task anchors map to `docs/execplans/roadmap-*.md` filenames,
    titles, references, and in-document roadmap citations.
  - See docs/execplan-renumbering-design.md.
  - Success: fixtures classify matching, missing, duplicate, conflicting,
    stale, and manually named ExecPlans without applying changes.

- [ ] 7.1.2. Produce a dry-run renumber plan.

  - Requires 7.1.1.
  - Convert a roadmap edit's renumber map into an ExecPlan rename and rewrite
    plan, including conflicts, unchanged files, and review-required cases.
  - Success: `mapsplice` can emit a JSON dry-run plan that lists every proposed
    rename, content rewrite, skipped file, conflict, diagnostic, content hash,
    and destination anchor.

- [ ] 7.1.3. Define history-preservation rules.

  - Requires 7.1.1.
  - Decide which historical references remain as old numbers, which links are
    rewritten, and how tombstones or redirects are represented when necessary.
  - Success: the design and fixtures distinguish live roadmap references from
    historical notes, audit evidence, and intentionally stale citations.

### 7.2. Decide whether to automate ExecPlan renumbering apply

This step answers whether dry-run evidence justifies building an apply engine
instead of keeping ExecPlan renumbering as guided manual work. Its outcome
informs whether the project accepts the extra recovery and history-preservation
surface of automated writes.

- [ ] 7.2.1. Measure dry-run demand before committing to apply automation.

  - Requires 7.1.2 and 7.1.3.
  - Record dry-run reports from real roadmap updates and classify whether
    guided manual renames are rare, cheap, or repeatedly error-prone.
  - Success: proceed only if at least three dry-run reports show recurring
    manual pain; defer if reports are mixed or too sparse; reject if the
    reports show the manual flow is usually cheap and correct.

- [ ] 7.2.2. Specify apply preconditions before any mutation.

  - Requires 7.2.1.
  - Refuse dirty git worktrees unless `--allow-dirty` is supplied, require
    matching dry-run content hashes, destination anchors present in the on-disk
    roadmap, no plan error diagnostics, no path conflicts, and no unresolved
    `review_needed` diagnostics unless explicitly overridden. Target-state and
    conflict checks happen before any mutation.
  - Success: the design can prove an apply attempt cannot target a phantom
    roadmap state, cannot mutate after a failed preflight check, and can be
    recovered through git if interrupted.

- [ ] 7.2.3. Implement guarded apply only if demand evidence justifies it.

  - Requires 7.2.2.
  - Apply stages, in order: preflight, prepare, rename, rewrite, postflight.
  - Preflight fails closed before any partial writes if source-hash,
    destination-anchor, or conflict checks fail.
  - Rename ExecPlan files and rewrite only allowlisted live links, titles,
    metadata fields, and current-roadmap references using capability-oriented
    filesystem operations.
  - Success: if built, apply preserves historical sections by default, and a
    second run returns a clear stale-plan diagnostic.

- [ ] 7.2.4. Document operator recovery and agent boundaries.

  - Requires 7.2.3.
  - Explain dry-run review, guided manual renames, clean-worktree recovery,
    conflict handling, and when an agent must stop for maintainer judgement.
  - Success: the users' guide and agent skill cover normal dry-run
    renumbering, review-required conflicts, optional apply, and recovery from
    interrupted applies.
