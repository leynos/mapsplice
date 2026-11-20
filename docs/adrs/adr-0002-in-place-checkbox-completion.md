# Architectural decision record (ADR) 0002: Patch checkbox completion in place

## Status

Accepted on 20 November 2026. `mapsplice complete` checks one task or addendum
sub-task off by patching the addressed item's source bytes instead of rendering
the roadmap canonically.

## Date

2026-11-20.

## Context and problem statement

Recording finished work previously meant either editing the checkbox by hand or
round-tripping a fragment through `replace`, which forces the caller to extract
the item first and makes an agent risk a structural edit for a one-byte status
change. Issue #229 asks for `mapsplice complete <target> <anchor>`: preview the
patched roadmap on standard output, or rewrite the file with `--in-place`.

The structural operations re-render any item whose number, summary, dependency
text, body, or descendants change. That is appropriate for a splice and wrong
for completion: a canonical re-render may reflow a wrapped title, restate
nested bullets, or touch adjacent list spacing, so a status change would
produce unrelated diff noise. The repository also treats formatter stability as
a contract (design F4): Markdown paragraphs and bullets wrap at 80 columns, and
a suffix appended to a title must not be reflowed by `make fmt` into a shape
`mapsplice` would not itself have produced.

The done marker itself is a project convention, not a property of the grammar.
Some teams want `— done`; others want a bare checkbox. Forcing any marker would
make `mapsplice` opinionated about prose, and applying a marker twice would
corrupt the title.

## Decision drivers

- Only the addressed item's checkbox and optional marker may change; every
  other byte keeps its source spelling (F1, F2).
- Re-running the command, with or without a configured marker, must be a
  no-op that exits successfully (C5).
- Completion output must be stable under the house formatter, so the inserted
  marker has to predict `mdtablefix --wrap`'s layout.
- Marker configuration should follow the existing `ortho-config` precedence:
  flag, environment, local file, XDG file, home file, default.
- Phase and step anchors have no checkbox and must fail with a typed,
  inspectable error rather than a silent success.
- The operation must be usable from the library as well as the CLI, on the same
  request/outcome boundary as the other commands.

## Options considered

### Option A: Canonical re-render of the addressed item

Parse the roadmap, mark the model checked, and render the item (or its task
list) canonically. This reuses the existing renderer and mutation machinery,
but it rewrites the item's title wrapping, continuation indentation, and
nested-list spacing. The golden fixture for a wrapped title with nested bullets
would show churn well beyond the checkbox, and F1/F2 would fail for the item
the edit was asked to touch.

### Option B: Source-span patch (chosen)

Capture the addressed item's span from mdast positions, replace the single
checkbox byte inside the marker, and insert an optional suffix at the end of
the title paragraph before its terminal punctuation. Only those bytes change.
The cost is a new span type, an explicit post-condition check, and local
wrapping logic for the suffix.

### Option C: Parse, modify, and print through a CommonMark renderer

Round-trip the whole document through a renderer. This is the simplest model
but loses original bytes everywhere, violating F1 for every untouched node and
making the command unusable on roadmaps whose formatting the renderer would
normalize.

| Topic                     | Option A       | Option B      | Option C       |
| ------------------------- | -------------- | ------------- | -------------- |
| Untouched bytes preserved | No             | Yes           | No             |
| New code required         | Minimal        | Span and wrap | Minimal        |
| Formatter stability       | Renderer-owned | Local rule    | Renderer-owned |
| F1/F2 for the edited item | Fails          | Holds         | Fails          |

_Table 1: Comparison of completion strategies._

## Decision outcome / proposed direction

Option B. `src/roadmap/model_source_span.rs` carries an `ItemSourceSpan` with
the item start, title start, and title paragraph end; the parser populates it
for tasks and addendum sub-tasks, and the completion operation rejects a span
that is out of order, out of range, or not on character boundaries.

`src/roadmap/ops/complete.rs` resolves the anchor, rejecting phases and steps
with `UnsupportedAnchorLevel`, agrees the parsed checked state with the marker
byte, and returns the original text unchanged when the item is already checked.
Otherwise it normalizes the configured suffix, checks whether the title already
ends with it, replaces the checkbox byte, inserts the suffix, and re-parses the
patched text. The post-condition requires the same anchor set, a checked
target, and a suffix present at the title's end; failure returns
`UnsafeCompletionEdit` and nothing is written.

`src/roadmap/ops/complete_suffix.rs` owns marker placement. A suffix without
terminal punctuation is inserted before the title's trailing punctuation run so
the full stop stays terminal; a suffix that supplies its own punctuation
replaces that run. The suffix is appended to the title's final line, and only
its own words wrap, greedily at 80 columns, onto continuation lines indented to
the title's content column. Line endings follow the document.

The operation is exposed as `CommandKind::Complete { anchor, suffix }` and
configured through `--suffix`, `MAPSPLICE_CMDS_COMPLETE_SUFFIX`, and a
`[cmds.complete]` table, matching the `insert` `after` precedent. An empty
configured value selects no marker; a value containing a line break is a
configuration error.

## Goals and non-goals

- Goals:
  - Complete a leaf task, a task other tasks depend on, and an addendum
    sub-task without changing any other byte.
  - Support preview and in-place modes with the existing outcome types.
  - Support an optional, configurable done marker placed before terminal
    punctuation.
  - Make a second run, and a run on an already-checked item, a successful
    no-op.
- Non-goals:
  - Guarding completion when addendum sub-tasks remain open
    (`--allow-incomplete-subtasks`); recorded as future roadmap work.
  - A `reopen` command; recorded as future roadmap work.
  - Renumbering, dependency rewriting, or accepting a fragment file. A
    completed item keeps its anchor, so no reference changes.

## Known risks and limitations

- The suffix wrap rule duplicates the formatter's 80-column policy in one
  constant. If the house wrap width or hanging-indent rule changes, completed
  titles may need a formatter pass; the golden fixtures pin the current rule.
- Completion deliberately does not validate dependencies. Dependants of the
  addressed item never block the edit, and completing a prerequisite that is
  itself dangling remains the concern of the structural commands.
- Because an already-checked item is always a no-op, `complete` cannot change
  an existing marker. That is intentional: `reopen` is the future operation
  that removes one.

## Architectural rationale

The domain layer gains a pure function from `(roadmap, source, anchor, suffix)`
to patched text, with no filesystem, process, or environment access. The CLI
adapter only merges configuration and validates the suffix, and the composition
root in `src/lib.rs` chooses stdout versus `--in-place` and records the rewrite
metric. This keeps domain policy separate from adapters, as the hexagonal
boundary requires, and leaves the ordinary renderer and preservation machinery
untouched for structural edits.
