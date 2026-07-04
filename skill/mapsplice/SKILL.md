---
name: mapsplice
description: >
  Use the current `mapsplice` CLI to make structural edits to roadmap-shaped
  Markdown files. Trigger when an agent needs to append, insert, delete, or
  replace numbered roadmap phases, steps, tasks, or addendum sub-tasks while
  preserving renumbering and `Requires` references. This skill covers only the
  functionality available now; it is not a roadmap-grooming, roadmap-authoring,
  validation, JSON-output, link-checking, or ExecPlan-renumbering skill.
---

# Mapsplice

## Scope

Use this skill to operate the installed `mapsplice` command safely. It does not
decide what the roadmap should say. Use `$roadmap-grooming` for curation, GIST
judgement, task consolidation, or deciding whether roadmap work belongs in a
different phase.

Current `mapsplice` supports only these commands:

```text
mapsplice append <target> <fragment>
mapsplice insert [--after] <target> <anchor> <fragment>
mapsplice delete <target> <anchor>
mapsplice replace <target> <anchor> <fragment>
```

The only global edit mode is:

```text
--in-place, -i
```

Do not assume `mapsplice validate`, `--json`, link checking, agent context
output, or ExecPlan renumbering exists until the local `mapsplice --help` shows
those commands or flags.

## Roadmap shape

`mapsplice` edits a constrained roadmap grammar:

- phases are level-2 headings, such as `## 8. Phase title`;
- steps are level-3 headings, such as `### 8.2. Step title`;
- tasks are checklist items, such as `- [ ] 8.2.3. Task title`;
- addendum sub-tasks are nested checklist items, such as
  `- [ ] 8.2.3.1. Sub-task title`.

Anchors use the same dotted forms:

- `8` targets a phase;
- `8.2` targets a step;
- `8.2.3` targets a task;
- `8.2.3.1` targets an addendum sub-task.

Fragments for `insert` and `replace` must contain one or more sibling items at
the same structural level as the target anchor. `append` is phase-only in the
current release, so its fragment must contain one or more phases.

## Safe workflow

1. Check the live command surface first:

   ```bash
   mapsplice --help
   mapsplice insert --help
   ```

2. Start from a clean understanding of local changes:

   ```bash
   git status --short
   ```

3. Write the fragment as a temporary Markdown file. Keep its numbering
   structurally valid, but do not spend time making the numbers final;
   `mapsplice` renumbers inserted or appended content.

4. Preview to stdout before touching the target:

   ```bash
   mapsplice insert docs/roadmap.md 4.2 fragment.md > /tmp/roadmap.md
   diff -u docs/roadmap.md /tmp/roadmap.md
   ```

5. Inspect the diff. The intended changes should be the requested structural
   edit, downstream renumbering, and dependency-reference rewrites. If the diff
   rewrites unrelated prose, large untouched sections, or unexpected spacing,
   stop and review before using `--in-place`.

6. Apply in place only after the preview is acceptable:

   ```bash
   mapsplice --in-place insert docs/roadmap.md 4.2 fragment.md
   ```

7. Run the repository's Markdown gates after editing. In this repository, use
   the scoped targets when possible:

   ```bash
   make markdownfmt MARKDOWN_PATHS='docs/roadmap.md'
   make markdownlint
   ```

## Command notes

### Append

Use `append` only to add phase-level content to the end of a roadmap:

```bash
mapsplice append docs/roadmap.md phase-fragment.md
mapsplice --in-place append docs/roadmap.md phase-fragment.md
```

The fragment must contain one or more `## <number>.` phase headings.

### Insert

Use `insert` to place sibling content before an anchor:

```bash
mapsplice insert docs/roadmap.md 6 fragment.md
mapsplice insert docs/roadmap.md 6.2 fragment.md
mapsplice insert docs/roadmap.md 6.2.3 fragment.md
```

Use `--after` to place content after the anchor:

```bash
mapsplice insert --after docs/roadmap.md 6.2.3 fragment.md
```

Be aware that `insert --after` can also be enabled by configuration or the
`MAPSPLICE_CMDS_INSERT_AFTER` environment variable. For a predictable one-off
run, set it explicitly:

```bash
MAPSPLICE_CMDS_INSERT_AFTER=false mapsplice insert docs/roadmap.md 6 fragment.md
```

### Delete

Use `delete` to remove one addressed phase, step, task, or addendum sub-task:

```bash
mapsplice delete docs/roadmap.md 6.2.3
mapsplice --in-place delete docs/roadmap.md 6.2.3
```

Later items at the same or deeper levels are renumbered.

### Replace

Use `replace` to remove one addressed item and splice in one or more sibling
items from a fragment:

```bash
mapsplice replace docs/roadmap.md 6.2 replacement.md
mapsplice --in-place replace docs/roadmap.md 6.2 replacement.md
```

The replacement fragment must match the anchor's structural level.

## Output and configuration

By default, successful commands write the rewritten roadmap to stdout and leave
the target file unchanged. With `--in-place`, successful commands rewrite the
target and do not print the roadmap body.

`--in-place` can also be enabled by configuration or by
`MAPSPLICE_IN_PLACE=true`. For a predictable preview, disable it explicitly:

```bash
MAPSPLICE_IN_PLACE=false mapsplice replace docs/roadmap.md 6 replacement.md
```

The CLI has no command-line flag that forces `in_place = false`; use the
environment variable when a config file might enable in-place writes.

## Reference rewriting and failures

`mapsplice` rewrites only dependency references in `Requires` clauses. It
should preserve incidental numbers such as prose quantities, semantic versions,
and section references.

The command fails closed when the target or fragment does not match the
supported grammar, when the fragment level does not match the anchor, when the
anchor is missing, or when a valid `Requires` reference cannot resolve after
the edit. In `--in-place` mode, failed edits leave the target unchanged.

## Known caveat

Dogfooding on this repository observed that `append` can normalize spacing in
untouched roadmap task lists. Treat broad spacing-only churn as a reason to
stop, inspect the diff, and file or reference a product issue instead of
blindly committing the result.
