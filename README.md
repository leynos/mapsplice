# Mapsplice

`mapsplice` is a small CLI for splicing one roadmap into another without
hand-renumbering phases, steps, tasks, or dependency references.

By default the rewritten roadmap is written to standard output. Pass
`--in-place` or `-i` to rewrite the target file instead.

## Commands

```text
mapsplice append <target> <file-to-append>
mapsplice insert <target> <anchor> <file-to-insert>
mapsplice insert --after <target> <anchor> <file-to-insert>
mapsplice delete <target> <anchor>
mapsplice replace <target> <anchor> <file-to-replace-with>
```

Anchors determine the structural level of the operation:

- `8` targets a phase.
- `8.2` targets a step.
- `8.2.3` targets a task.

`append` is phase-only in the current release. It appends one or more phases to
the end of the target roadmap.

## Supported roadmap grammar

`mapsplice` intentionally supports a constrained roadmap format rather than
arbitrary Markdown surgery:

- Optional preamble content may appear before the first numbered phase.
- Phases are level-2 headings such as `## 8. Phase title`.
- Steps are level-3 headings such as `### 8.2. Step title`.
- Tasks are Markdown list items whose first paragraph starts with a task number
  such as `- [ ] 8.2.3. Task title`.
- Task bodies may contain extra paragraphs or nested bullet lists.

The tool renumbers every affected phase, step, and task after each operation.
Dependency prose inside text nodes is rewritten when it matches a renumbered
roadmap identifier. For example, `Requires 9.1.2.` becomes `Requires 10.1.2.`
when the referenced task moves.

Fragments supplied to `insert` or `replace` must contain one or more sibling
items at the same structural level as the target anchor. A task fragment cannot
be inserted at phase level, and a phase fragment cannot replace a task.

## Configuration

The CLI uses `ortho-config` for command parsing and optional per-subcommand
configuration discovery.

The initial release keeps required file paths and anchors on the command line.
Optional command flags can still come from configuration. For example, the
`insert --after` switch can be configured with either of the following:

```toml
[cmds.insert]
after = true
```

```bash
MAPSPLICE_CMDS_INSERT_AFTER=true
```

## Development

Run the standard quality gates before committing:

```bash
make check-fmt
make lint
make test
make markdownlint
make nixie
```
