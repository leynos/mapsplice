# Validation and agent output design

## Status and scope

- **Status:** design for roadmap phase 6.
- **Scope:** `mapsplice validate`, miette-style human diagnostics, JSON output
  mode for every command, compact agent context, and a roadmap-maintenance
  agent skill.
- **Audience:** contributors implementing validation and JSON contracts,
  maintainers reviewing CLI compatibility, and agents consuming structured
  roadmap evidence.
- **Precedence:** `docs/roadmap.md` is the source of truth for planned phase 6
  scope; `docs/users-guide.md` remains the source of truth for currently
  released command behaviour; `AGENTS.md` governs quality gates, testing rules,
  and en-GB-oxendict spelling.
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
- `--json` turns every command into a strict machine contract with one compact
  JSON success document on stdout, or one JSON diagnostic document on stderr.

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

| Code | Class      | Meaning                                                         |
| ---- | ---------- | --------------------------------------------------------------- |
| 0    | success    | No validation findings at `error` severity.                     |
| 1    | validation | Error-severity findings or unrecoverable invalid roadmap state. |
| 2    | usage      | The command line or configuration is invalid.                   |
| 3    | io         | A target, linked file, or config file is missing.               |
| 4    | internal   | A model invariant or unexpected bug was hit.                    |

This is a deliberate pre-1.0 exit-code break from the current binary, where
non-clap failures collapse to exit code 1. The new taxonomy must be versioned
with the JSON schema and documented in the users' guide before release.

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

`status` is `ok` when there are no findings. It is `findings` when warnings or
notes are present but no error-severity finding exists; this remains exit code
0 so agents can distinguish "valid with advice" from "invalid". Validation
findings at `error` severity, or an invalid roadmap state that cannot be
recovered during validation, map to the validation exit class.

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

The binary must decide JSON mode before clap emits an error. The implementation
should use a lenient pre-scan for `--json` in `argv` and route clap usage
errors through the JSON diagnostic renderer when the flag is present. The
pre-scan must not accept unknown aliases; it only selects the renderer for
errors that happen before normal parsing.

The current binary installs tracing on stderr and reports failures through both
`tracing::error!` and `eprintln!`. JSON mode must replace that behaviour. The
JSON renderer must suppress stderr tracing, route diagnostic logs to a file or
other explicitly configured sink, and keep log previews out of diagnostic JSON.
If JSON output needs to mention that sink, it must use a separately versioned
field containing only sink identifiers or status that consumers can ignore. It
must never emit trace lines beside the JSON document on stderr, and it must not
embed localized trace previews in the main diagnostic payload.

Edit commands need an additional success payload because stdout is already used
for rewritten Markdown in human mode. JSON mode should return metadata and put
the artefact in one of two places:

- in-place edits report `artifact.kind = "target_rewritten"` and the target
  path;
- non-in-place JSON edits require `--output <path>` and report
  `artifact.kind = "written_file"` with the output path and content hash.

This keeps
`mapsplice insert --json --output <path> <target> <anchor> <fragment>` compact
and valid JSON while preserving the current human stdout behaviour. For example,
`mapsplice insert --json --output rewritten.md target.md 2 fragment.md` writes
the rewritten roadmap to `rewritten.md` and reports
`artifact.kind = "written_file"` in the JSON success document. A future explicit
`--include-document` mode may embed the rewritten Markdown in JSON for small
files, but it is not the default agent contract.

## Validation model

Validation should use the existing roadmap parser and model rather than a
parallel Markdown checker, but this is not a trivial wrapper around today's
fail-fast parser. The parser error channel must grow a multi-finding mode with
line and column ranges from mdast positions. A fatal syntax error still exits
with class `validation`; when recovery is impossible, the command emits one
fatal syntax finding and stops later passes.

Findings are collected in three passes.

1. **Syntax pass:** parse the roadmap grammar and report unsupported heading
   structure, task-without-step, step-without-phase, malformed anchors, and
   duplicate anchors.
2. **Dependency pass:** inspect `Requires` clauses using the existing
   dependency-reference predicate. Valid unresolved anchors are errors.
   Version-like or invalid dependency tokens are warnings when they appear in a
   dependency context.
3. **Link pass:** inspect local Markdown links. The document root is the git
   repository root containing the validated file. Relative file paths must stay
   inside that root and resolve from the linking document's directory. Fragment
   links must match generated heading slugs. Remote HTTP and HTTPS links are
   reported as unchecked notes, not fetched.

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
    },
    {
      "path": ["append"],
      "mutation": "write",
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
- CLI tests pin JSON usage errors that happen before clap completes parsing.
- CLI tests prove JSON mode suppresses or redirects tracing stderr output.
- Agent-context fixtures prove that the skill references real commands.
- Property tests cover span ranges, dependency-token classification, link-root
  containment, and JSON state transitions. Rust implementation work should use
  `proptest`; non-Rust ports should use the nearest equivalent such as
  Hypothesis or fast-check.
- Any explicit lemma or proof obligation introduced by the implementation must
  have an exhaustive proof, bounded model check, or documented finite-state
  argument before the relevant validator or JSON contract is accepted.

## Rollout

1. Add validation domain types and parsing-only checks.
2. Add the `validate` subcommand in human mode.
3. Add JSON diagnostics and validation-result schemas.
4. Add JSON success summaries for edit commands.
5. Add `context --json` and the agent skill.
6. Update the users' and developers' guides.

## References

- [Agent-native CLI assistance design](https://github.com/leynos/ortho-config/blob/main/docs/agent-native-cli-design.md).
- [miette `Diagnostic` documentation](https://docs.rs/miette/latest/miette/trait.Diagnostic.html).
- [miette `GraphicalReportHandler` documentation](https://docs.rs/miette/latest/miette/struct.GraphicalReportHandler.html).
