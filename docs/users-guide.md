# Mapsplice users' guide

`mapsplice` edits roadmap-shaped Markdown documents by applying one structural
change at a time, then renumbering the affected roadmap items for you. It is
deliberately narrow: the tool understands a specific roadmap grammar and will
reject documents that drift outside it.

## Installation

Install `mapsplice` from the repository root:

```bash
cargo install --path .
```

For one-off local runs during development, you can also use:

```bash
cargo run -- <subcommand> ...
```

## The roadmap shape `mapsplice` expects

`mapsplice` treats a roadmap as three structural levels:

- **Phases** are level-2 headings such as `## 8. Phase title`.
- **Steps** are level-3 headings such as `### 8.2. Step title`.
- **Tasks** are Markdown list items whose first paragraph begins with a number
  such as `- [ ] 8.2.3. Task title`.

Optional document preamble content may appear before the first numbered phase.
Tasks may also contain extra paragraphs or nested bullet lists beneath the
first numbered paragraph.

Anchors on the command line map directly to those levels:

- `8` targets a phase
- `8.2` targets a step
- `8.2.3` targets a task

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

Use in-place mode when you want the target rewritten directly:

```bash
mapsplice --in-place delete docs/roadmap.md 8
```

When `--in-place` is used, `mapsplice` rewrites the target and does not emit
the roadmap body on standard output.

## Configuration

`mapsplice` uses `ortho-config` for optional configuration loading. Required
inputs such as file paths and anchors remain command-line arguments, while
optional subcommand settings can come from configuration.

For example, the `insert --after` behaviour can be set in configuration:

```toml
[cmds.insert]
after = true
```

Or through the corresponding environment variable:

```bash
MAPSPLICE_CMDS_INSERT_AFTER=true
```

This is most useful when you have a local default editing style and want to
keep the command line shorter.

## Validation rules and failure cases

`mapsplice` will fail fast when:

- the target roadmap does not follow the supported phase/step/task grammar
- a fragment starts at the wrong structural level for the requested anchor
- a step appears before the first phase
- a task list appears without a containing step
- unsupported heading structure appears inside the roadmap body

This strictness is intentional. The tool is designed to produce predictable
roadmap edits, not to guess what a malformed document might have meant.

## Development gates

If you are working on the tool itself, the standard repository gates are:

```bash
make check-fmt
make lint
make test
make markdownlint
make nixie
```
