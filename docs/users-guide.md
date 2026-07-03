# Mapsplice users' guide

This guide is for `mapsplice` users who need to edit roadmap-shaped Markdown
documents and understand the tool's supported grammar and constraints.
`mapsplice` edits roadmap-shaped Markdown documents by applying one structural
change at a time, then renumbering the affected roadmap items for the
maintainer. It is deliberately narrow: the tool understands a specific roadmap
grammar and will reject documents that drift outside it.

## Installation

Install `mapsplice` from the repository root:

```bash
cargo install --path .
```

For one-off local runs during development, use:

```bash
cargo run -- <subcommand> ...
```

## The roadmap shape `mapsplice` expects

`mapsplice` treats a roadmap as four structural levels:

- **Phases** are level-2 headings such as `## 8. Phase title`.
- **Steps** are level-3 headings such as `### 8.2. Step title`.
- **Tasks** are Markdown list items whose first paragraph begins with a number
  such as `- [ ] 8.2.3. Task title`.
- **Addendum sub-tasks** are nested task checklist items whose first paragraph
  begins with a fourth-level number such as `- [ ] 8.2.3.1. Sub-task title`.

Optional document preamble content may appear before the first numbered phase.
Tasks may also contain extra paragraphs or nested bullet lists beneath the
first numbered paragraph. Ordinary nested bullets remain task body Markdown;
numbered nested checklist items that use the fourth-level form are addendum
sub-tasks owned by their parent task.

Anchors on the command line map directly to those levels:

- `8` targets a phase
- `8.2` targets a step
- `8.2.3` targets a task
- `8.2.3.1` targets an addendum sub-task

Fragments supplied to `insert` and `replace` must contain one or more sibling
items at the same level as the target anchor.

## Command overview

`mapsplice` supports four operations:

```text
mapsplice append <target> <file-to-append>
mapsplice insert <target> <anchor> <file-to-insert>
mapsplice insert --after <target> <anchor> <file-to-insert>
mapsplice delete <target> <anchor>
mapsplice replace <target> <anchor> <file-to-replace-with>
```

By default the rewritten roadmap is written to standard output. Use
`--in-place` or `-i` to rewrite the target file instead.

## Worked example

Start with a target roadmap:

```markdown
## 1. Phase one

### 1.1. Step one

- [ ] 1.1.1. First task.

## 2. Phase two

### 2.1. Step two

- [ ] 2.1.1. Second task. Requires 2.1.1.
```

Create a fragment containing one or more sibling items. This fragment adds a
new phase:

```markdown
## 9. Inserted phase

### 9.1. Added step

- [ ] 9.1.1. Added task. Requires 9.1.1.
```

Insert the fragment before phase 2:

```bash
mapsplice insert target.md 2 fragment.md
```

The result will:

- insert the new phase before the original phase 2
- renumber the inserted phase to `2`
- renumber the original phase 2 to `3`
- rewrite dependency prose such as `Requires 2.1.1.` to `Requires 3.1.1.`

Only `Requires` dependency references are rewritten. Incidental numeric text
such as section references (`§2.1`), semantic versions (`1.4.0`), and prose
quantities is preserved.

## Command details

### `append`

```bash
mapsplice append <target> <file-to-append>
```

`append` is phase-only in the current release. The fragment file must contain
one or more phases, and those phases are appended to the end of the target
roadmap.

### `insert`

```bash
mapsplice insert <target> <anchor> <file-to-insert>
mapsplice insert --after <target> <anchor> <file-to-insert>
```

`insert` places sibling content before the addressed anchor by default. Pass
`--after` to place the fragment after the anchor instead.

Examples:

```bash
mapsplice insert docs/roadmap.md 8 new-phase.md
mapsplice insert --after docs/roadmap.md 8.2.3 new-task.md
```

### `delete`

```bash
mapsplice delete <target> <anchor>
```

`delete` removes exactly one addressed phase, step, or task. Any later items at
the same or deeper levels are renumbered as needed.

### `replace`

```bash
mapsplice replace <target> <anchor> <file-to-replace-with>
```

`replace` removes the addressed item and splices in one or more sibling items
from the fragment file. Replacing a phase with multiple phases is supported,
provided the fragment itself contains phases.

## Output modes

The default mode writes the updated roadmap to standard output:

```bash
mapsplice replace docs/roadmap.md 8 replacement.md > rewritten.md
```

Non-empty rendered roadmaps end in exactly one final newline. A second
`mapsplice` render of conformant output does not add another final newline.

Use in-place mode when the target should be rewritten directly:

```bash
mapsplice --in-place delete docs/roadmap.md 8
```

When `--in-place` is used, `mapsplice` rewrites the target and does not emit
the roadmap body on standard output.

If validation fails in `--in-place` mode, the target file is left unchanged and
no roadmap body is emitted on standard output.

## Configuration

Required inputs such as target paths, anchors, and fragment paths remain
command-line arguments. Configuration only supplies optional defaults for the
global `in_place` setting and the `insert` command's `after` setting.

Boolean configuration values use `true` or `false`.

### Global `in_place`

`in_place` controls whether successful edits rewrite the target file directly.
It defaults to `false`, so `mapsplice` writes the updated roadmap to standard
output unless another source enables in-place mode.

Set the file default with a top-level TOML key:

```toml
in_place = true
```

On Unix-like systems, `mapsplice` searches these files for global defaults, in
increasing precedence order:

1. `$XDG_CONFIG_HOME/mapsplice/config.toml`, when `XDG_CONFIG_HOME` is set and
   is valid Unicode.
2. `./.mapsplice.toml` in the current working directory.

The environment variable is:

```bash
MAPSPLICE_IN_PLACE=true
```

Final precedence for `in_place` is:

1. `--in-place` or `-i`, which can only force `true`.
2. `MAPSPLICE_IN_PLACE`.
3. Local `./.mapsplice.toml`.
4. XDG `$XDG_CONFIG_HOME/mapsplice/config.toml`.
5. Default `false`.

There is no command-line flag that forces `in_place = false`; use
`MAPSPLICE_IN_PLACE=false` when a file default should be disabled for one
process.

### Insert `after`

`after` controls whether `mapsplice insert` places the fragment after the
anchor. It defaults to before-anchor insertion, matching
`mapsplice insert <target> <anchor> <file-to-insert>` without `--after`.

Set the file default under the insert command table:

```toml
[cmds.insert]
after = true
```

On Unix-like systems, `mapsplice` searches these files for `insert` defaults,
in increasing precedence order:

1. `~/.mapsplice.toml`.
2. `$XDG_CONFIG_HOME/mapsplice/config.toml`, when present.
3. `./.mapsplice.toml` in the current working directory.

The environment variable is:

```bash
MAPSPLICE_CMDS_INSERT_AFTER=true
```

Final precedence for `after` is:

1. `--after`.
2. `MAPSPLICE_CMDS_INSERT_AFTER`.
3. Local `./.mapsplice.toml`.
4. XDG `$XDG_CONFIG_HOME/mapsplice/config.toml`.
5. Home `~/.mapsplice.toml`.
6. Default before-anchor insertion.

The command-line flag can only force `after = true`. There is no command-line
flag that forces `after = false`; use `MAPSPLICE_CMDS_INSERT_AFTER=false` when
a file default should be disabled for one process.

## Validation rules and failure cases

`mapsplice` will fail fast when:

- the target roadmap does not follow the supported phase/step/task grammar
- a fragment starts at the wrong structural level for the requested anchor
- a step appears before the first phase
- a task list appears without a containing step
- unsupported heading structure appears inside the roadmap body
- a valid `Requires` dependency reference cannot be resolved to a roadmap item
  after the edit

This strictness is intentional. The tool is designed to produce predictable
roadmap edits, not to guess what a malformed document might have meant.

## Contributing

Maintainer workflows and repository gates are documented in the
[contributing guide](contributing.md).
