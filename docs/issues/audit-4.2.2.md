# Post-step codebase audit after 4.2.2

## Scope and method

This audit reviewed `origin/main` after roadmap task 4.2.2 ("Add path-scoped
Markdown maintenance targets"), with the audit record written from the
`audit-4-2-2` inspection worktree branched off `origin/main` (commit
`547eb9a`). The remit was to find refactoring opportunities, repeated code,
complex conditionals, ergonomic awkwardness, high similarity, inconsistencies,
poor separation of concerns, command-query segregation violations, and gaps in
documentation comments, developer or user documentation, and behavioural or
unit test coverage, then propose actionable fixes.

Primary references used:

- `AGENTS.md`
- `docs/mapsplice-design.md`
- `docs/developers-guide.md`
- `docs/users-guide.md`
- `docs/documentation-style-guide.md`
- `docs/roadmap.md`
- `leta` and `sem` skills
- `rust-router`, routing to `rust-errors`, `rust-types-and-apis`,
  `rust-unit-testing`, and `domain-cli-and-daemons`

Tooling notes:

- The Memtrace MCP server was unavailable in this planning session: the host
  rejected `mcp__memtrace__list_indexed_repositories` with a
  not-yet-granted-permission error and the session is non-interactive, so no
  Memtrace call could complete. Per the standing rules this tooling gap is
  recorded and the audit proceeded with bounded branch-local evidence gathered
  through direct file inspection and exact text search.
- The inspection worktree was created with the harness `EnterWorktree` tool
  (default `fresh` base ref branches from `origin/main`) after direct
  `git worktree add` and `git fetch` were blocked by the sandbox.

## The 4.2.2 change under review

Task 4.2.2 added three Makefile entries — `markdownfmt`, `markdownlint-paths`,
and the `require_markdown_paths` guard driven by `MARKDOWN_PATHS` — plus a
developers-guide section and the integration suite
`tests/makefile_markdown_maintenance.rs`. Findings 1 to 3 concern that change
directly; the remainder are broader codebase observations surfaced during the
audit.

## Finding 1: Scoped Markdown target names are inconsistent

- **Category:** inconsistency
- **Severity:** low
- **Location:** `Makefile:70` (`markdownfmt`), `Makefile:78`
  (`markdownlint-paths`), `Makefile:75` (`markdownlint`)

The path-scoped lint target is named `markdownlint-paths`, parallel to the
repository-wide `markdownlint`, but the path-scoped format target is named
`markdownfmt` with no `-paths` suffix and no repository-wide `markdownfmt`
counterpart (repository-wide Markdown formatting lives under `fmt` via
`mdformat-all`). A maintainer who has learned `markdownlint-paths` will
reasonably guess `markdownfmt-paths` and get a "no rule" error.

**Proposed fix:** rename the scoped format target to `markdownfmt-paths` (or
add it as an alias) so the scoped pair reads `markdownfmt-paths` /
`markdownlint-paths`, and update the developers-guide snippet accordingly.

## Finding 2: `MARKDOWN_FORMAT_FLAGS` is undocumented

- **Category:** docs-gap
- **Severity:** low
- **Location:** `Makefile:23`; `docs/developers-guide.md:127-136`

The developers-guide documents `MARKDOWN_PATHS` but not
`MARKDOWN_FORMAT_FLAGS`, the variable that controls the `mdtablefix` flag set
(`--wrap --renumber --breaks --ellipsis --fences --in-place`). A maintainer who
needs to change wrapping or renumbering behaviour for a scoped run has no
documented override and must read the Makefile.

**Proposed fix:** document `MARKDOWN_FORMAT_FLAGS` alongside `MARKDOWN_PATHS`,
noting its default and that `--in-place` is part of it.

## Finding 3: Scoped-format integration coverage does not assert flag pass-through

- **Category:** test-gap
- **Severity:** low
- **Location:** `tests/makefile_markdown_maintenance.rs:17-39`

The dry-run assertions check for `--in-place` but not the remaining
`MARKDOWN_FORMAT_FLAGS` tokens (`--wrap`, `--renumber`, `--breaks`,
`--ellipsis`, `--fences`), so a regression that drops or reorders those flags
would pass. There is also no case asserting the repository-wide `markdownlint`
glob still works alongside the scoped targets.

**Proposed fix:** extend the `markdownfmt` dry-run case to assert the full
default flag set reaches `mdtablefix`, and add a case confirming the
repository-wide `markdownlint` glob path is unaffected by the scoped targets.

## Finding 4: Unescaped backslash corrupts round-tripped text

- **Category:** inconsistency
- **Severity:** medium
- **Location:** `src/roadmap/render_text.rs:74` (`is_markdown_metacharacter`),
  `src/roadmap/render_text.rs:17` (`escape_markdown`)

`escape_markdown` prefixes a backslash before each Markdown metacharacter, but
the metacharacter set omits the backslash itself. Source text containing a
literal `\` therefore renders unescaped: `\x` in an input text node round-trips
to `\x`, which Markdown re-reads as an escaped `x`, silently dropping the
backslash. This breaks the round-trip fidelity the renderer is meant to
preserve. The set also omits `!` (image syntax) and does not special-case a
leading `-`/`.` that could read as a list ordinal.

**Proposed fix:** add `'\\'` to `is_markdown_metacharacter` (escaping it first
so ordering is safe), and add a unit test that round-trips text containing a
backslash and an exclamation mark. Consider the `!` and leading-punctuation
cases in the same pass.

## Finding 5: Failed in-place rewrite leaves an orphan temporary file

- **Category:** separation-of-concerns
- **Severity:** medium
- **Location:** `src/fs.rs:32-61` (`rewrite_utf8`)

`rewrite_utf8` creates a temporary sibling file, writes to it, then renames it
over the target. If `write_all` or the final `rename` fails, the function
returns the error without removing the temporary file, leaving an orphan
`.<name>.mapsplice.tmp.<pid>.<nanos>.<counter>` beside the target. Over
repeated failures these accumulate in the user's working directory.

**Proposed fix:** on the write and rename error paths, attempt
`cap.dir.remove_file(&temp_name)` (ignoring a secondary failure) before
returning the original error, and add a test that injects a write failure and
asserts no temporary file remains.

## Finding 6: Global config discovery is hand-rolled while the guide claims ortho-config

- **Category:** separation-of-concerns
- **Severity:** medium
- **Location:** `src/cli_config.rs:13-123`; `src/cli.rs:29-37`, `264-286`;
  `docs/developers-guide.md:59-68`

Global `in_place` defaults are discovered by a bespoke loader in
`cli_config.rs` (XDG `config.toml`, then `.mapsplice.toml`, then
`MAPSPLICE_IN_PLACE`), even though `GlobalCli` derives `OrthoConfig`. The
subcommand `after` default, by contrast, goes through `ortho-config`'s
`load_and_merge_subcommand_for`. The developers-guide states flatly that "The
CLI uses `ortho-config` for optional defaults", which is only true for the
subcommand path; the global path re-implements discovery and precedence by
hand. Two config mechanisms for one concern invite drift (for example, the
specific config-file locations and precedence are known only from the code).

**Proposed fix:** either route global defaults through `ortho-config` for a
single mechanism, or, if the manual loader is deliberate, document why in the
developers-guide and name the exact file locations and precedence there.

## Finding 7: Nested `Result<Result<String, ()>>` obscures the "file absent" signal

- **Category:** ergonomics
- **Severity:** low
- **Location:** `src/cli_config.rs:52` (`read_config_candidate`)

`read_config_candidate` returns `Result<std::result::Result<String, ()>>`,
using the inner `Err(())` purely to mean "candidate not found". A unit-typed
error carries no meaning and forces callers to read the body to understand the
convention.

**Proposed fix:** return `Result<Option<String>>` with `None` for the not-found
case; the call site at `global_config_file_default` reads more directly as
`if let Some(contents) = read_config_candidate(&path)?`.

## Finding 8: Config-discovery file locations are undocumented for users

- **Category:** docs-gap
- **Severity:** medium
- **Location:** `docs/users-guide.md:183-199`; `src/cli_config.rs:40-49`

The users' guide Configuration section shows the `MAPSPLICE_CMDS_INSERT_AFTER`
environment variable and a `[cmds.insert]` snippet but never states where a
global `in_place` default is read from. The implementation looks for
`$XDG_CONFIG_HOME/mapsplice/config.toml` and a project-local `.mapsplice.toml`
(local taking precedence), which a user cannot discover without reading the
source.

**Proposed fix:** document the global config file locations, their precedence
(local over XDG over none), and the `in_place = true` key in the users' guide
Configuration section.

## Finding 9: Duplicated re-exports create two public names per parser

- **Category:** inconsistency
- **Severity:** low
- **Location:** `src/lib.rs:255-258`

`parse_fragment` and `parse_fragment as parse_fragment_text`, and likewise
`parse_roadmap` / `parse_roadmap_text`, are both re-exported, so the public API
carries two names for each function. The `_text` synonyms are used only by the
integration tests (`tests/roadmap_parse.rs`, `tests/roadmap_sub_tasks.rs`,
`tests/ui/public_api.rs`), doubling the maintained surface without a caller
that needs the short form.

**Proposed fix:** pick one canonical name per function (the `_text` form
disambiguates "parses text, not a path") and drop the synonym, or document in
the public-API test why both are intentionally exported.

## Finding 10: `record_dependency_rewrites` is called in both branches of `run_request`

- **Category:** duplication
- **Severity:** low
- **Location:** `src/lib.rs:174-182` (`run_request`)

`observability::record_dependency_rewrites(dependency_rewrites)` appears in
both the in-place and stdout arms of the terminal `if`. The call is
branch-independent and can be hoisted above the conditional, leaving only the
in-place-specific `record_in_place_rewrite` and outcome construction inside the
arms.

**Proposed fix:** call `record_dependency_rewrites` once before the `if`, then
branch only on the in-place-specific work.

## Finding 11: Task and sub-task item parsers are near-duplicates

- **Category:** similarity
- **Severity:** medium
- **Location:** `src/roadmap/parse/mod.rs:155` (`parse_task_item`),
  `src/roadmap/parse/mod.rs:262` (`parse_sub_task_item_unchecked`); see also
  `mod.rs:315`/`325` (`parse_task_paragraph` / `parse_sub_task_paragraph`) and
  `mod.rs:89`/`336` (`strip_heading_prefix` / `parse_numbered_paragraph`)

Both item parsers share the same skeleton: `checked.is_none()` guard,
`children.first()` with `ok_or_else`, `let Node::Paragraph(..) else`, parse the
numbered paragraph, then `children.get(1..).unwrap_or(&[])` for the body. Only
the paragraph parser, error strings, and body handling differ. The paired
paragraph parsers and the heading/paragraph prefix strippers repeat the same
shape.

**Proposed fix:** extract a shared
`parse_checklist_item_head(item, kind) -> (&Paragraph, &[Node])` helper for the
item parsers and a single `strip_numbered_prefix(children, level, kind)` for
the prefix logic, letting the task and sub-task paths supply only the differing
pieces.

## Finding 12: Fragment-root parsers duplicate a single-list validation skeleton

- **Category:** similarity
- **Severity:** medium
- **Location:** `src/roadmap/parse/fragment.rs:169`
  (`parse_task_fragment_root`), `src/roadmap/parse/fragment.rs:192`
  (`parse_sub_task_fragment_root`)

The two functions are near-verbatim: the same `root.children.len() != 1` check,
`let Some(Node::List(list)) else`, list parse, empty check, sibling validation,
and wrap into a variant. Only the list parser, validator, and result variant
vary.

**Proposed fix:** factor a generic
`parse_single_list_fragment(root, parse_fn, validate_fn, wrap_fn, messages)`
that both roots call with their per-level closures.

## Finding 13: `parse_step_fragment_root` re-implements the DocumentParser step machine

- **Category:** separation-of-concerns
- **Severity:** medium
- **Location:** `src/roadmap/parse/fragment.rs:119`
  (`parse_step_fragment_root`); compare `src/roadmap/parse/document.rs:98-203`

`parse_step_fragment_root` inlines the step lifecycle already encoded in
`DocumentParser` (`begin_step`/`flush_step`/`push_non_structural_node`): manual
`StepSection` construction, take-previous-on-new-heading, and
tasks-versus-trailing body routing. Two divergent copies of the same state
machine risk drifting apart as roadmap rules evolve.

**Proposed fix:** extract a reusable step accumulator shared by the document
and fragment paths, or drive the fragment path through `DocumentParser`
directly.

## Finding 14: `validate_sub_task_numbers` is redundant on the only path that calls it

- **Category:** duplication
- **Severity:** low
- **Location:** `src/roadmap/parse/document.rs:165,223`
  (`validate_sub_task_numbers`); compare `src/roadmap/parse/mod.rs:258,297`
  (`validate_sub_task_number`)

The document path parses sub-tasks through the checked `parse_sub_task_item`
(`mod.rs:242`), which already calls `validate_sub_task_number` and rejects a
sub-task whose `task_number()` does not match its parent. `flush_tasks` then
calls `validate_sub_task_numbers` (`document.rs:165`), re-checking exactly that
invariant. The fragment path uses the unchecked parser and never calls
`validate_sub_task_numbers`, so the helper is both redundant on the document
path and absent on the fragment path — parent membership for fragment sub-tasks
is instead validated later at splice time.

**Proposed fix:** remove `validate_sub_task_numbers` and its call, relying on
the parse-time check; if a document-level cross-check is wanted as
defence-in-depth, add a comment stating it duplicates the parse-time invariant
deliberately.

## Finding 15: Phase-lookup-by-number is copied three times

- **Category:** duplication
- **Severity:** low
- **Location:** `src/roadmap/ops/mod.rs:152` (`insert_phases`), `:197`
  (`delete_anchor` phase arm), `:230` (`replace_anchor` phase arm)

The `phases.iter().position(|p| p.number == target).ok_or(AnchorNotFound…)`
idiom is written out three times, whereas step and task lookups share the
`find_step_parent_mut` / `find_task_parent_mut` helpers. The checkbox-marker
mapping (`Some(true) => "[x] "`, etc.) is similarly repeated at
`src/roadmap/render.rs:71`, `:106`, and `:265`.

**Proposed fix:** add a `find_phase_index(roadmap, target)` helper used by all
three phase sites, and a single
`checkbox_marker(checked: Option<bool>) -> &'static str` used by the three
render sites.

## Finding 16: `insert_sub_tasks` and `replace_sub_task` splice logic is untested

- **Category:** test-gap
- **Severity:** medium
- **Location:** `src/roadmap/ops/sub_task.rs:13` (`insert_sub_tasks`), `:40`
  (`replace_sub_task`); test module at `:105`

The colocated `#[cfg(test)]` module only exercises `delete_sub_task`. The two
operations that splice both the `sub_tasks` vector and the parallel `children`
vector — the alignment invariant most likely to break — have no direct unit
coverage.

**Proposed fix:** add table-driven cases asserting `sub_tasks` and `children`
stay aligned after insert-before, insert-after, and replace, including the
boundary ordinals.

## Finding 17: Only the first dangling dependency is reported

- **Category:** ergonomics
- **Severity:** low
- **Location:** `src/roadmap/ops/rewrite.rs:22,106-108`

Dependency rewriting accumulates every unresolved anchor into a `Vec` but then
reports only `unresolved.into_iter().next()` as a single `DanglingDependency`,
silently discarding the rest. A user fixing a roadmap with several dangling
references must re-run once per anchor.

**Proposed fix:** either early-return on the first unresolved anchor (dropping
the `Vec` entirely) or surface all dangling anchors in the error payload so the
user can fix them in one pass.

## Finding 18: `fragment_level` free function duplicates `RoadmapFragment::level`

- **Category:** duplication
- **Severity:** low
- **Location:** `src/roadmap/model.rs:225` (`fragment_level`), `:215`
  (`RoadmapFragment::level`); mixed use at `src/roadmap/ops/mod.rs:104` vs
  `:122`

`fragment_level` is a one-line wrapper over the already-public
`RoadmapFragment::level`, and both are used: `ops/mod.rs:104` calls the free
function while `:122` calls the method for the same value. Two names for one
query is redundant and the mixed usage reads as accidental.

**Proposed fix:** drop `fragment_level` and call `.level()` everywhere, or, if
the free function is kept for the public API, standardise internal call sites
on one form.

## Finding 19: Boolean routing probes allocate error strings on every non-match

- **Category:** ergonomics
- **Severity:** low
- **Location:** `src/roadmap/parse/mod.rs:61-67` (`is_phase_heading`,
  `is_step_heading`), `:378` (`looks_like_numbered_list`); `fragment.rs:94`
  (`is_step_fragment_start`)

Routing decisions use error-returning parsers as boolean probes via `.is_ok()`.
Each non-matching probe constructs and immediately discards `format!` error
strings and an owned remainder `String`, and these probes run on every heading
and list during document routing. `is_step_fragment_start` also inlines the
body of `is_step_heading` instead of delegating, unlike its `is_phase_*`
sibling.

**Proposed fix:** add a cheap `matches_numbered_prefix(value, level) -> bool`
that returns a boolean without building errors or owned strings, and have
`is_step_fragment_start` delegate to `is_step_heading`.

## Finding 20: Weak encapsulation on `RenumberPlan` and `TaskChildren`

- **Category:** separation-of-concerns
- **Severity:** low
- **Location:** `src/roadmap/model.rs:134-138` (`RenumberPlan::by_source`);
  `src/roadmap/parse/task_children.rs:5` (`TaskChildren` fields)

`RenumberPlan::by_source` is a `pub` field while its nested-map invariant is
meant to be maintained by the `pub(crate)` `insert` method (`model.rs:253`); a
public field lets callers bypass the guard. Similarly `TaskChildren` exposes
`body`, `sub_tasks`, and `ordered` as `pub(super)` and callers mutate them
directly (`mod.rs:208-217`, `243-246`) while also using `flush_body`, so the
`ordered`/`sub_tasks` alignment is easy to violate.

**Proposed fix:** make `by_source` private and expose only `resolve`/
`resolve_unique`/`insert`; give `TaskChildren` `push_body_node` and
`push_sub_task` methods that keep `ordered` in sync and drop the public fields.

## Proposed roadmap items

These are proposals only; adding them to `docs/roadmap.md` is reserved to the
root agent.

1. **Escape backslashes in `escape_markdown` and add round-trip coverage.**
   Finding 4 is a latent correctness bug in the render path; it warrants a
   small fix plus a regression test rather than being folded into unrelated
   work.
2. **Make in-place rewrite failure-safe and document config discovery.**
   Findings 5, 6, and 8 cluster around the CLI edge (orphan temp files, the
   split global-versus-subcommand config mechanism, and undocumented config
   locations) and would make a coherent hardening step.
3. **Consolidate the roadmap parser's duplicated item/fragment machinery.**
   Findings 11 to 14 and 19 describe a cluster of near-duplicate parser code
   and redundant validation; a single refactor step would reduce drift risk
   across the parse module.
4. **Close sub-task splice unit-test gaps.** Finding 16 leaves the highest-risk
   ops invariant (parallel `sub_tasks`/`children` vectors) unproven for insert
   and replace.
