# Validation and agent output design

## Status and scope

- **Status:** design for roadmap phase 6.
- **Scope:** `mapsplice validate`, miette-style human diagnostics, JSON output
  mode for every command, compact agent context, and a roadmap-maintenance
  agent skill.
- **Out of scope:** ExecPlan renumbering. That is covered by
  [execplan-renumbering-design.md](execplan-renumbering-design.md).

## Problem statement

`mapsplice` can already fail closed while applying edits, but agents need a
read-only way to prove that a roadmap is well formed before they ask for a
mutation. They also need structured output for every command so they can
consume bounded evidence rather than parse human terminal text.

Phase 6 therefore adds two related contracts:

- `mapsplice validate <target>` checks roadmap syntax, dependencies, and links
  without changing the target.
- `--json` turns every command into a strict machine contract with one JSON
  success document on stdout, or one JSON diagnostic document on stderr.

Human diagnostics stay rich and localizable. Machine diagnostics stay stable
and non-localized.

## Goals

- Validate the supported roadmap grammar without attempting an edit.
- Report unresolved `Requires` references and malformed dependency tokens.
- Check local Markdown links to files, headings, and fragments.
- Render human diagnostics with source spans, labels, help text, and stable
  codes using the miette diagnostic model.
- Provide JSON success and failure schemas for every subcommand.
- Publish compact agent context describing commands, output modes, mutation
  boundaries, examples, and exit classes.
- Add an agent skill that teaches validate-before-edit and validate-after-edit
  workflows.

## Non-goals

- Fetch remote links during validation.
- Accept arbitrary Markdown outside the roadmap grammar.
- Make JSON mode a stream for edit commands.
- Replace human miette diagnostics with JSON-only errors.
- Move domain execution into OrthoConfig.

## Command contract

The new command is:

```console
mapsplice validate [--json] <target>
```

Validation is read-only. It must not call the renderer except when a fixture is
explicitly testing render stability, and it must never emit rewritten Markdown.

Exit classes:

| Code | Class      | Meaning                                           |
| ---- | ---------- | ------------------------------------------------- |
| 0    | success    | No validation findings at `error` severity.       |
| 1    | validation | The roadmap is parseable but has findings.        |
| 2    | usage      | The command line or configuration is invalid.     |
| 3    | io         | A target, linked file, or config file is missing. |
| 4    | internal   | A model invariant or unexpected bug was hit.      |

Human mode writes diagnostics to stderr. Successful human validation may print
a short summary to stdout:

```text
docs/roadmap.md: ok
```

JSON mode writes exactly one success document to stdout and no stderr:

```json
{
  "schema_version": "1",
  "kind": "mapsplice.validation_result",
  "target": "docs/roadmap.md",
  "status": "ok",
  "summary": {
    "error": 0,
    "warning": 0,
    "note": 0
  },
  "findings": []
}
```

JSON failure writes no stdout and one diagnostic document to stderr:

```json
{
  "schema_version": "1",
  "kind": "mapsplice.diagnostic",
  "exit_class": "validation",
  "diagnostics": [
    {
      "code": "mapsplice::validate::missing_dependency",
      "severity": "error",
      "message": "dependency 8.2.3 does not resolve",
      "path": "docs/roadmap.md",
      "range": {
        "start": { "line": 42, "column": 18 },
        "end": { "line": 42, "column": 23 }
      },
      "help": "remove the dependency or update it to an existing roadmap item"
    }
  ]
}
```

## Output mode rules

The neighbouring OrthoConfig agent-native design treats `--json` as the
canonical structured-output flag and requires strict byte ownership for stdout
and stderr. `mapsplice` should adopt that contract directly:

- success in JSON mode writes one JSON document to stdout and nothing to
  stderr;
- failure in JSON mode writes no stdout and one JSON diagnostic document to
  stderr;
- human mode may use rich diagnostics on stderr;
- protocol identifiers, finding codes, exit classes, and schema versions are
  not localized;
- subprocess output is forbidden in JSON mode.

Edit commands need an additional success payload because stdout is already used
for rewritten Markdown in human mode. JSON mode should return metadata and put
the artefact in one of two places:

- in-place edits report `artifact.kind = "target_rewritten"` and the target
  path;
- non-in-place edits report `artifact.kind = "roadmap_document"` and include
  the rewritten Markdown in a `content` string.

This keeps `mapsplice insert --json target.md 2 fragment.md` valid JSON while
preserving the current human stdout behaviour.

## Validation model

Validation should use the existing roadmap parser and model rather than a
parallel Markdown checker. Findings are collected in three passes.

1. **Syntax pass:** parse the roadmap grammar and report unsupported heading
   structure, task-without-step, step-without-phase, malformed anchors, and
   duplicate anchors.
2. **Dependency pass:** inspect `Requires` clauses using the existing
   dependency-reference predicate. Valid unresolved anchors are errors.
   Version-like or invalid dependency tokens are warnings when they appear in a
   dependency context.
3. **Link pass:** inspect local Markdown links. Relative file paths must
   resolve under the document root. Fragment links must match generated heading
   slugs. Remote HTTP and HTTPS links are reported as unchecked notes, not
   fetched.

Every finding carries:

- a stable `code`;
- a `severity` of `error`, `warning`, or `note`;
- a source path and optional source range;
- a short message;
- optional help text;
- optional related locations.

## Human diagnostics

Miette's `Diagnostic` trait models codes, severity, help, source code, labels,
URLs, and related diagnostics. Its graphical handler renders those diagnostics
with spans, wrapping, colour, and link controls, and exposes a render method
that can be tested in isolation from global state.

`mapsplice` should keep a domain-owned `MapspliceDiagnostic` enum and implement
the miette metadata from that enum. Tests should disable colour, URLs, and
terminal-width variance so snapshots stay deterministic.

Example human diagnostic:

```text
error[mapsplice::validate::missing_dependency]: dependency 8.2.3 does not resolve
  --> docs/roadmap.md:42:18
   |
42 | - [ ] 6.1.2. Check links. Requires 8.2.3.
   |                  ^^^^^ dependency target is missing
   |
   = help: remove the dependency or update it to an existing roadmap item
```

## Agent context

`mapsplice context --json` should emit a compact schema-versioned document:

```json
{
  "schema_version": "1",
  "kind": "mapsplice.agent_context",
  "commands": [
    {
      "path": ["validate"],
      "mutation": "read_only",
      "json": true,
      "exit_classes": ["success", "validation", "usage", "io", "internal"]
    }
  ]
}
```

The context should include every edit command, `validate`, and future
maintenance commands. It should describe whether each command mutates files,
whether `--in-place` applies, where JSON success data appears, and which
examples are safe for agents to run.

## Agent skill

The roadmap-maintenance skill should be generated or checked against the agent
context so it does not drift from the CLI. The skill must instruct agents to:

- run `mapsplice validate --json` before editing a roadmap;
- prefer `mapsplice` structural edits over hand edits;
- inspect JSON diagnostics by `code` and `exit_class`;
- run validation again after an edit;
- stop for maintainer judgement when diagnostics report ambiguous links,
  duplicate anchors, or failed postflight checks.

## Testing strategy

- Unit tests cover finding codes, ranges, dependency classification, and link
  target resolution.
- Behavioural tests cover `validate` success, syntax failure, dependency
  failure, link failure, and mixed finding severities.
- Snapshot tests pin miette human diagnostics with deterministic renderer
  settings.
- JSON golden fixtures pin success and failure schemas for every subcommand.
- CLI tests assert stdout and stderr byte ownership in human and JSON modes.
- Agent-context fixtures prove that the skill references real commands.

## Rollout

1. Add validation domain types and parsing-only checks.
2. Add the `validate` subcommand in human mode.
3. Add JSON diagnostics and validation-result schemas.
4. Add JSON success summaries for edit commands.
5. Add `context --json` and the agent skill.
6. Update the users' and developers' guides.

## References

- [Agent-native CLI assistance design](../ortho-config/docs/agent-native-cli-design.md).
- [miette `Diagnostic` documentation](https://docs.rs/miette/latest/miette/trait.Diagnostic.html).
- [miette `GraphicalReportHandler` documentation](https://docs.rs/miette/latest/miette/struct.GraphicalReportHandler.html).
