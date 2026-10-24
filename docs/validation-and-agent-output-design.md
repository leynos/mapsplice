# Validation and agent output design

## 1. Status and scope

- **Status:** target design for roadmap phase 6, not a description of released
  commands. The documentation-set decisions remain proposed for review.
- **Scope:** read-only roadmap and documentation-set validation, human
  diagnostics, global JSON output, graph export, compact agent context, and a
  roadmap-maintenance skill.
- **Audience:** implementers of validation and output contracts, maintainers
  reviewing compatibility, and agents consuming structured roadmap evidence.
- **Precedence:** [the roadmap](roadmap.md) owns planned work and completion;
  [the users' guide](users-guide.md) owns released behaviour; `AGENTS.md` owns
  quality gates. [ADR 002](adr-002-documentation-set-discovery.md) and
  [ADR 003](adr-003-documentation-namespaces.md), once accepted, own discovery
  and namespace decisions. This design applies their proposed contracts.
- **Related work:**
  [ExecPlan renumbering](execplan-renumbering-design.md) belongs to phase 7.
  Export work in [PR #153](https://github.com/leynos/mapsplice/pull/153)
  addresses [issue #144](https://github.com/leynos/mapsplice/issues/144), but
  does not by itself deliver the global JSON or graph-validation contracts.

### 1.1. Roadmap-to-design coverage

| Roadmap work | Design contract |
| --- | --- |
| 1.2.2 and 1.2.3 | Shared hard-wrap-aware dependency scanning and empty-clause warnings, section 4.2. |
| 6.1.1 | Read-only command, recoverable syntax findings, and pass prerequisites, sections 3 and 4. |
| 6.1.2 | Dependency resolution and workspace-bounded Markdown links, sections 4.2 and 4.5. |
| 6.1.3 | Cycles, self-references, and complete cycle witnesses, section 4.3. |
| 6.1.4 | Redundant dependencies and completion-status advice, section 4.4. |
| 6.1.5 | Stable finding model and deterministic human rendering, section 5. |
| 6.2.1 and 6.2.2 | JSON byte ownership, usage fallback, exit classes, and edit summaries, sections 3 and 6. |
| 6.2.3 | Versioned graph export and export-profile compatibility, section 7. |
| 6.2.4 and 6.2.5 | Agent context and validate-before/after-edit skill, section 8. |
| 6.3.1 to 6.3.3 | Documentation-set discovery, scoped uniqueness, and aggregate reporting, section 9 and ADRs 002/003. |

_Table 1: Planned work and its design coverage. No row asserts implementation._

## 2. Problem, goals, and boundaries

The editor can fail closed while applying edits, but maintainers need a
read-only way to check a roadmap before requesting mutation. They also need
structured results rather than terminal-text scraping. A duplicate record number
across files, as reported in
[issue #150](https://github.com/leynos/mapsplice/issues/150),
requires explicit inventory and namespace boundaries in addition to the
existing single-document duplicate-anchor check.

Phase 6 must validate supported syntax, local dependencies, cycles, dependency
hygiene, and local Markdown links. It must preserve the editor's accepted
roadmap grammar, use the same parser and dependency classifier, and emit
source-located findings without rendering or rewriting Markdown. JSON contracts
must remain compact, stable, non-localized, and complete enough for agents to
identify both findings and the scope actually checked.

The following remain outside this design:

- Remote fetching, arbitrary Markdown validation through the roadmap grammar,
  automatic repair, and ExecPlan mutation.
- Qualified cross-roadmap dependencies, durable task identifiers, and inference
  of completion from descendants.
- ADR/RFC contiguity, historical number allocation, filename/title agreement,
  index completeness, supersession semantics, or proof and traceability
  adequacy.
- Rule suppression, severity overrides, baselines, and warnings-as-errors.
  Version 1 has fixed severities; it must not imply these controls exist.
- Moving domain execution into OrthoConfig, making edit JSON a stream, or
  replacing human diagnostics with JSON-only errors.

## 3. Invocation, scope, and exit contract

The two validation forms are mutually exclusive:

```console
mapsplice validate [--json] <target>
mapsplice validate [--json] --documentation-set <manifest>
```

The first checks only the named roadmap. It never discovers a manifest or adds
neighbouring roadmaps, ADRs, or RFCs. Its link workspace is the nearest
containing Git worktree root, recognizing both `.git` directories and worktree
`.git` files; outside Git it is the target's parent directory. A missing or
ambiguous root is an error, not permission to inspect the invocation directory
instead.

The second uses ADR 002's explicit manifest and its parent directory as the
workspace. All roadmap dependency resolution remains document-local under
ADR 003. Reading a link target does not add it to the selected documentation
inventory. Both forms operate on immutable source buffers and use
capability-relative filesystem access.

Validation must not call the renderer or a mutation operation. A separate
render-stability fixture may exercise rendering, but the command must never
emit rewritten Markdown, create temporary files, update exports, or fetch URLs.

| Code | Class | Meaning |
| --- | --- | --- |
| 0 | success | No error-severity finding and all required inspection completed. |
| 1 | validation | Invalid document content, duplicate identities, unsafe targets, or export drift. |
| 2 | usage | Invalid arguments, manifest schema, or configuration. |
| 3 | io | Requested input or declared inventory cannot be read completely. |
| 4 | internal | A production model invariant or unexpected implementation failure. |

_Table 2: Shared exit classes for validation and agent-native commands._

A missing linked file or fragment is a document defect and therefore a
validation finding. A missing requested roadmap, manifest, or declared set
directory is an I/O failure. Permission errors and interrupted reads are I/O
failures, not evidence that a document reference is invalid. This distinction
supersedes the earlier design's ambiguous treatment of every missing linked
file as I/O.

Reject usage errors before inspection. During inspection, preserve collected
findings but let an internal failure take precedence over I/O, and I/O over
validation, when selecting the single exit class. Warnings and notes alone
retain exit code 0. A pass skipped because its prerequisites failed must not
produce a claim that the skipped property holds.

This is a deliberate pre-1.0 change from the current binary's non-clap failure
code. Version the JSON contract and document the exit change in the users'
guide before release. The draft export command's current exit behaviour is not
an exception to that migration.

## 4. Shared validation architecture

The input adapter obtains source buffers and stable document identities. The
existing parser produces the roadmap model, positions, and recoverable syntax
findings. One dependency visitor then produces classified tokens, owner
identities, resolved edges, and provenance. Pure domain passes consume those
objects; the human and JSON adapters render the same collected findings.

Extend the parser's error channel rather than build a parallel Markdown checker.
Recovery may continue at a trustworthy structural boundary, but it must not
invent anchors, parentage, checkbox state, or graph edges. Fatal syntax stops
dependent passes for that document. Other selected documents and independent
filename-uniqueness checks can still run. Retain invalid declarations and their
locations long enough to report all duplicate claimants.

Run syntax, dependency resolution, cycle analysis, hygiene, and link inspection
as distinct passes with explicit prerequisites. Record each pass as `completed`
or `skipped`, with a reason for a skipped pass. A recoverable local syntax issue
may leave link inspection possible; ambiguous ownership or duplicate anchors
blocks graph construction for that document. Report the syntax findings rather
than cascade guessed missing-dependency findings from a corrupt model.

Keep graph and uniqueness kernels pure and shared with production invocation.
Any verification work must exercise those kernels and their actual adapters,
not a second model that merely resembles the running checker.

### 4.1. Syntax and identity

Check unsupported heading structure, tasks without steps, steps without phases,
malformed anchors, and duplicate anchors using the supported roadmap grammar.
Preserve accepted body blocks, including tables and code blocks; export-format
restrictions in section 7 must not narrow general roadmap validity.

Within a roadmap, numbered phases, steps, tasks, and structural sub-tasks retain
their existing anchor rules. Unnumbered checkbox details have source-local owner
identities and their own status, but cannot become targets of bare anchors.
The documentation-set adapter contributes filename claims to the shared
uniqueness kernel without parsing ADR/RFC bodies as roadmap syntax.

### 4.2. Dependency classification and graph construction

Use the same candidate scanner and ownership visitor for rewriting, validation,
and export. Roadmap 1.2.2 supplies hard-wrapped clause recognition; 1.2.3
supplies warnings for `Requires` clauses with no recognizable anchors. Joining
logical text must retain a mapping back to original source ranges, including a
clause whose keyword and targets occupy different lines.

Retain the dependency predicate's exclusions for incidental prose numbers,
versions outside dependency contexts, section references, code, and link
destinations. A requirement belongs to the nearest containing item or checkbox
detail, not both that detail and its parent. Scan nested body bullets as well as
inline and continuation text.

Classify before applying command policy:

- A valid anchor with a unique local target creates an edge.
- A valid anchor with no local target produces `missing_dependency`, an error.
- Invalid or version-like tokens in dependency context produce
  `invalid_dependency_token`, a warning in general validation.
- A clause with no recognizable anchor produces `empty_requires`, a warning.
  When token warnings already explain that clause, do not add a redundant
  empty-clause warning. Preserve every original occurrence as provenance.

The graph is directed from dependent to prerequisite. Nodes carry document
identity, anchor or source-local owner identity, source range, and checkbox
state where one exists. Phase/step nodes without a checkbox have unknown state;
do not derive a state from their children. Only explicit `Requires` edges
participate. Structural containment does not create implicit dependencies.

Deduplicate repeated edges for graph algorithms while retaining all occurrence
locations. Resolve bare anchors only within the owner document, even during
multi-document validation. Malformed and unresolved references never create
phantom vertices. Graph construction records whether reference resolution was
complete; a graph with unresolved references cannot certify dependency validity.

### 4.3. Cycles and self-references

Cycle detection applies to every numbered level accepted by the dependency
predicate, not only third-level tasks. A self-edge produces
`mapsplice::validate::self_dependency` at error severity with the closed path
`A, A`. Each strongly connected component containing more than one vertex
produces one `mapsplice::validate::dependency_cycle` error.

Include a complete, actual directed cycle witness with its repeated starting
vertex, such as `A, B, C, A`, in structured evidence and human output. Include
all affected component vertices and related clause locations separately; do not
pretend that one simple witness necessarily visits every vertex in the
component. For a multi-vertex component, select a non-self cycle even if a
member also has a self-edge. Report self-edges separately.

Make witnesses deterministic: traverse nodes and neighbours in source order,
choose the earliest component vertex as the starting point, and select the
first simple return path in that order. Sort component findings by their first
source location. Do not enumerate all simple cycles, which is unnecessary for
locating the defect and can produce unboundedly verbose output.

Known resolved edges can prove a positive cycle finding even when another
reference is unresolved. They cannot prove absence of all cycles in that
incomplete document. Syntax that prevents trustworthy node ownership skips
cycle analysis entirely. An acyclic, fully resolved graph produces no cycle or
self-dependency findings.

### 4.4. Advisory dependency hygiene

A transitively redundant direct edge `A -> C` has an alternative path of at
least two edges from `A` to `C` after removing that direct edge from the graph.
For example, `A -> B -> C` makes a separate `A -> C` redundant. Emit
`mapsplice::validate::redundant_dependency` at note severity, located at the
redundant clause occurrence, with a deterministic alternate-path witness.

Evaluate each candidate against the original graph with only that edge removed;
do not iteratively delete edges while collecting findings. Run redundancy
analysis only after the document has an unambiguous, fully resolved, acyclic
graph. Otherwise record this pass as skipped and retain the prerequisite errors.
Repeated mentions of one direct edge alone are not transitive redundancy.

For each resolved direct edge from a checked item to an unchecked item, emit
`mapsplice::validate::completed_requires_incomplete` at note severity. Attach
the dependency occurrence and both checkbox locations. Do not infer phase or
step completion, propagate status through ancestors, or turn transitive open
prerequisites into additional direct-status findings. Unnumbered checkbox
details use their own status. Unknown status at either endpoint produces no
status finding. This check can run on trustworthy resolved edges even when
redundancy analysis must skip a cyclic or incomplete graph.

Neither advisory changes the exit class on its own or authorizes an automatic
rewrite. A clean graph produces neither finding.

### 4.5. Local Markdown links

Inspect links and link definitions represented by the shared Markdown parse.
Resolve a relative file path from the linking document's directory and an empty
path against the same document. Decode supported local URL escapes once before
normalizing path segments; reject encoded separators, NUL bytes, or paths that
escape the workspace. Reject absolute filesystem paths, drive prefixes, and
network-path references instead of treating them as relative files.

Use capability-relative no-follow opens and enforce the same symlink and nested
repository boundaries as discovery. A lexical `../` check alone is insufficient.
A missing file produces `missing_link_target`; a missing fragment produces
`missing_link_fragment`; escape or forbidden traversal produces
`unsafe_link_target`. These are error-severity document findings. An existing
target that cannot be read produces an I/O diagnostic instead.

Heading and fragment lookup must use one documented slugging implementation,
with golden fixtures for duplicate headings, punctuation, inline formatting,
Unicode, percent escapes, and explicit supported anchors. Pin the chosen
GitHub-compatible slug behaviour before 6.1.2 is complete; do not use a second,
approximate regular-expression slugger in the checker. Ambiguous explicit
anchors produce an error with all candidates, not first-match success.

Remote HTTP and HTTPS links produce `unchecked_remote_link` notes and are never
fetched. Other unsupported URI schemes produce `unchecked_link_scheme` notes;
active or remote schemes must never trigger a subprocess or network request.
Linked Markdown contributes a heading index, not a recursively validated
roadmap or an implicitly selected documentation-set member.

## 5. Findings and human diagnostics

The domain owns a typed `MapspliceDiagnostic` representation. Every finding has
a stable code, severity (`error`, `warning`, or `note`), path, short message,
and optional source range, help, related locations, and typed evidence. Omit
unavailable ranges rather than inventing line 1 for a filename collision.

Text ranges use one-based lines and Unicode-scalar columns, with an inclusive
start and exclusive end. Retain byte spans internally for Markdown positions
and miette; convert deliberately at the JSON boundary. A related location has a
path, optional range, and label. Evidence schemas are specific to finding codes:
cycle findings carry a closed vertex path, redundancy findings an alternate
path, and sequence collisions a namespace and all claimant paths.

Use the `mapsplice::validate::` prefix for the short validation codes named in
this design. Discovery additionally defines `empty_documentation_set`,
`invalid_record_filename`, `invalid_sequence_number`, and
`ambiguous_membership`; ADR 003 defines `duplicate_sequence_number`. Protocol
identifiers, code values, severity, evidence keys, and exit classes are not
localized. JSON messages and help are stable non-localized text; consumers must
branch on codes and evidence, not compare prose.

Sort findings by workspace-relative path, starting source position (path-only
findings first), code, and stable evidence key. Preserve all related locations
in deterministic order. Aggregate errors, warnings, and notes even when an error
exists; do not discard advice simply because the result uses the failure stream.

Implement miette's diagnostic metadata from the domain representation, including
codes, severity, labels, help, and related diagnostics. Human mode writes
findings to stderr and may print a short success summary to stdout:

```plaintext
docs/roadmap.md: ok
```

A cycle diagnostic must show the closed path and all participating clause
locations; a duplicate sequence diagnostic must name the set and every claimant.
Pin human snapshots with colour, hyperlink output, and terminal-width variance
disabled. Render through an isolated handler rather than mutable global state.

## 6. JSON and edit-output contracts

### 6.1. Stream ownership and early failures

Adopt `--json` as the global structured-output flag for every command:

- Success writes exactly one JSON document to stdout and nothing to stderr.
- Failure writes no stdout and exactly one JSON diagnostic document to stderr.
- Human mode may emit rich diagnostics on stderr.
- JSON mode must not leak tracing, subprocess output, progress messages, or
  localized log previews onto either protocol stream.

Decide JSON mode before clap emits an error using a lenient argv pre-scan for
exactly `--json`, respecting the `--` end-of-options delimiter. Do not invent
aliases or interpret a target named `--json` after that delimiter as a flag.
Route usage failures through the JSON renderer when the scan selected JSON.
The normal parser still owns argument validity.

The current binary's tracing plus terminal error output must become one output
adapter. Disable stderr tracing in JSON mode or route logs to an explicitly
configured non-protocol sink. Do not embed log previews in diagnostic JSON. Any
future sink-status field needs a versioned, ignorable metadata contract.

### 6.2. Validation schemas

The outer agent protocol uses `schema_version: "1"`. It is independent of the
numeric manifest schema and the export artefact's `format_version`. Version any
incompatible shape or semantic change; golden fixtures own the exact schema.
A clean single-roadmap result has this shape:

```json
{
  "schema_version": "1",
  "kind": "mapsplice.validation_result",
  "target": "docs/roadmap.md",
  "status": "ok",
  "summary": { "error": 0, "warning": 0, "note": 0 },
  "passes": [
    { "name": "syntax", "status": "completed" },
    { "name": "dependencies", "status": "completed" },
    { "name": "cycles", "status": "completed" },
    { "name": "redundancy", "status": "completed" },
    { "name": "completion_status", "status": "completed" },
    { "name": "links", "status": "completed" }
  ],
  "findings": []
}
```

With only warnings or notes, `status` becomes `findings` and exit code remains
0. A skipped pass includes `reason`; a failed prerequisite also produces its
own diagnostic, so skipping cannot turn an incomplete validation into success.
An error result uses this diagnostic envelope:

```json
{
  "schema_version": "1",
  "kind": "mapsplice.diagnostic",
  "exit_class": "validation",
  "scope": { "kind": "roadmap", "documents": ["docs/roadmap.md"] },
  "summary": { "error": 1, "warning": 0, "note": 0 },
  "passes": [
    {
      "document": "docs/roadmap.md",
      "checks": [
        { "name": "syntax", "status": "completed" },
        { "name": "dependencies", "status": "completed" },
        { "name": "cycles", "status": "completed" },
        {
          "name": "redundancy",
          "status": "skipped",
          "reason": "unresolved dependencies"
        },
        { "name": "completion_status", "status": "completed" },
        { "name": "links", "status": "completed" }
      ]
    }
  ],
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

When validation reached inspection, the failure envelope also includes the
available `scope`, `summary`, and per-document `passes`. Usage failures before
scope resolution can omit those fields. Complete command fixtures must include
available inspection metadata and every collected finding, not only the first
error. Set-validation pass entries use the same document-and-checks grouping.

Documentation-set success uses `kind: "mapsplice.documentation_set_result"`,
with `manifest`, `manifest_schema_version`, the ADR 002 inventory in `sets`,
aggregate `summary`, per-document `passes`, `status`, and `findings`. Failure
uses the same diagnostic envelope and retains the available inventory in
`scope`. No selected inventory or I/O failure may result in a misleading empty
`ok` result. Document paths are workspace-relative, not host-specific absolute
identifiers; the invocation identifies the workspace.

### 6.3. Structured edit summaries

JSON edit success reports operation kind, target path, in-place status,
inserted and removed anchors, renumber mappings, dependency rewrites, and the
emitted artefact's content hash. Hash the bytes actually written, not an earlier
model or an unrelated export. Preserve document identity on anchor mappings.

Human non-in-place edits continue writing Markdown to stdout. JSON edits use
one of two explicit artefact routes:

- In-place edits report `artifact.kind = "target_rewritten"` and the target
  path after the guarded write succeeds.
- Non-in-place JSON edits require `--output <path>` and report
  `artifact.kind = "written_file"`, the output path, and content hash.

For example:

```console
mapsplice insert --json --output rewritten.md target.md 2 fragment.md
```

This writes Markdown to `rewritten.md` and one compact success document to
stdout. Reject the missing-output case as usage before editing. Existing
fail-closed preflight and write guarantees still apply. A future
`--include-document` option may embed small documents, but it is not part of
this default contract.

## 7. Export, graph, and drift compatibility

### 7.1. One grammar, explicit export profiles

PR #153 introduces a deterministic export view and semantic drift checker. Its
[version 1 format](https://github.com/leynos/mapsplice/blob/ccf348837835b6d19b44696a3a9080fb9fcba936/docs/roadmap-export-format.md)
is the integration baseline, not a silently interchangeable definition of
roadmap validity. The branch is unmerged at this design's revision.

General validation accepts the roadmap grammar, including body tables and code
blocks. The draft export profile requires task checkboxes and rejects some body
structures. An accepted roadmap may therefore fail that profile. Such failure
must say `mapsplice::export::unsupported_structure`, with a source location and
the profile name, rather than claim that the document violates roadmap syntax.
Do not make successful export a prerequisite for general validation.

Both commands use the same token classification. General validation warns about
invalid/version-like dependency tokens; the strict export profile rejects them
as `mapsplice::export::invalid_dependency_token` because it cannot promise a
complete dependency view. Unresolved valid anchors are errors in both commands.
This is an explicit policy difference after shared classification, not licence
for two scanners. Require paired fixtures for these cases.

### 7.2. Artefact and agent-protocol versions

Preserve these distinct uses:

```console
mapsplice export --format json ROADMAP.md
mapsplice export --check EXPORT.json ROADMAP.md
mapsplice export --json --format json ROADMAP.md
mapsplice export --json --check EXPORT.json ROADMAP.md
```

The first emits the documented raw export artefact, with numeric
`format_version`, source-ordered arrays, and deterministic serialization. It
must not silently acquire an outer wrapper when global JSON support lands.
The third emits an agent envelope with `schema_version`,
`kind: "mapsplice.roadmap_export"`, `target`, `profile`, the versioned artefact
in `export`, and the resolved `graph`. The two flags are not synonyms:
`--format` selects the artefact representation; `--json` selects the command
protocol and its diagnostics.

The envelope must not change the meaning of version 1 artefact fields. Shape or
semantic changes to the artefact require its own format-version increment and
migration documentation. Keep raw and enveloped output fixtures separately.
Moving the documented export format onto `main` belongs to the export work,
not this documentation-only change.

### 7.3. Resolved graph contract

The graph has source-ordered nodes for phases, steps, tasks, structural
sub-tasks, and supported checkbox details. Each node carries its document
identity, kind, anchor where present, source-local owner key, title, optional
checkbox state, and source location. Unknown state stays unknown. Unnumbered
detail keys locate a node within that source snapshot; they are not durable IDs
or new `Requires` targets.

Edges carry source and target node keys plus all explicit dependency occurrence
locations. Their direction is dependent to prerequisite. Preserve first
occurrence order after deduplicating edges. Export must not omit an accepted
anchor level, reassign detail dependencies to parents, synthesize unresolved
nodes, or convert hierarchy into dependency edges.

Export graph construction reuses section 4.2. A structurally resolved graph may
contain cycles; export presents that graph for inspection rather than certify
it acyclic. Run `validate` for cycle and hygiene findings. Consumers can derive
ready tasks, critical paths, and consistency queries, but phase 6 does not add
dedicated scheduling commands or certify downstream traceability semantics.

### 7.4. Drift checking

The check form reads canonical Markdown and a committed export without writing
either. Compare parsed, supported-version artefacts according to the documented
normalization rules, not raw JSON whitespace. Report item-level differences
such as title, status, dependencies, ordering, additions, and removals.

On equality, JSON mode returns `kind: "mapsplice.export_check_result"` with
`status: "ok"`, target, export path, and format version. On drift, return
`mapsplice::export::drift` in the shared failure envelope with validation exit
class. Invalid export JSON or unsupported versions receive distinct typed
validation diagnostics; unreadable inputs receive I/O diagnostics. Do not
silently regenerate an export or treat drift detection as full validation of
roadmap syntax, cycles, links, documentation sets, or semantic evidence.

## 8. Agent context and maintenance skill

`mapsplice context --json` emits a compact, schema-versioned
`mapsplice.agent_context` document. Include all edit commands, both validation
forms, export and check forms, context itself, and implemented maintenance
commands. Describe mutation boundaries, whether `--in-place` applies, accepted
output modes, artefact destinations, exit classes, and safe examples.

The context must distinguish raw export JSON from global JSON protocol mode,
and single-document validation from explicit documentation-set validation. It
must advertise only implemented capabilities, even though this design describes
future ones. Generate or check the maintenance skill against that context.

The skill must require `validate --json` before and after roadmap edits, prefer
structural edits over hand editing, inspect diagnostics by `code` and
`exit_class`, and stop for maintainer judgement on ambiguous links or
identities, duplicate anchors, incomplete inspection, or failed postflight
checks. Run set validation only when an explicit manifest selects that scope.
A successful export or clean drift check must not substitute for validation.

## 9. Documentation-set extension

Roadmap 6.3 brings issue #150 into the same validation pipeline. ADR 002 owns
manifest selection, directory classification, recursion, exclusions, standalone
workspaces, filesystem boundaries, and reproducible inventory. ADR 003 owns
set-scoped ADR/RFC claims, document-scoped roadmap anchors, number
canonicalization, overlapping membership, and identity lifetime.

Do not infer a namespace from a directory or merge companion roadmap graphs.
One series can span current and archived directories; independent sets can reuse
a number. Leading-zero variants of a positive record number collide within one
set. Legal sequence gaps and unselected files do not produce findings. Filename
collisions include every claimant without fabricated content spans.

Aggregate findings and pass status across selected documents, preserving valid
results from independent documents when another fails. Discovery failure or
ambiguous membership prevents a complete success result; it cannot disappear
behind a clean roadmap result. No part of this extension authorizes record
renumbering, automatic gap filling, or cross-document dependency resolution.

## 10. Testing and acceptance strategy

Each implementation slice requires unit and behavioural coverage, with golden
fixtures for protocol and human output. In particular:

- Syntax recovery must collect independent findings without constructing invalid
  models, and fatal syntax must visibly skip dependent passes.
- Dependency fixtures must cover hard wraps, nested bullets and details, empty
  clauses, malformed/version-like tokens, unresolved targets, all anchor levels,
  duplicate occurrences, and unchanged exclusion rules.
- Graph fixtures must cover self-edges, two- and multi-node cycles, multiple
  cyclic components, cyclic components with self-edges, disconnected nodes,
  incomplete resolution, and deterministic full cycle witnesses.
- Hygiene fixtures must cover a redundant edge and its alternate path, diamonds
  without a redundant edge, repeated direct references, checked-to-unchecked
  dependencies, unknown status, detail ownership, and skipped redundancy on an
  invalid graph. Notes alone must retain exit code 0.
- Link fixtures must pin slugging and fragment behaviour, missing versus
  unreadable targets, workspace escape including encoded paths, symlinks,
  nested repositories, and a test adapter that rejects all network access.
- Paired validation/export fixtures must demonstrate accepted tables and code
  blocks that the strict export profile rejects, and shared token classification
  with deliberately different warning/error policy.
- Export fixtures must pin all node kinds, unknown checkbox state, resolved
  edges and provenance, raw versus enveloped versions, semantic drift,
  malformed/unsupported export inputs, and byte-identical source files.
- Discovery and namespace fixtures must cover ADRs 002/003, including archives,
  independent series, companion roadmaps, padding equivalence, gaps,
  three-claimant collisions, empty/missing selections, and overlapping
  membership.
- JSON fixtures for every command must assert exact stream ownership, early
  usage failures, `--` handling, tracing suppression, mixed severities, exit
  precedence, and inventory/pass evidence on partial failures.
- Agent-context tests must reject skill references to unimplemented commands
  and enforce validation before and after edits.

Use `proptest` for generated graphs, identifiers, spans, and path boundaries;
non-Rust ports should use the nearest equivalent such as Hypothesis. For small
bounded graphs, compare cycle existence and redundant-edge predicates against
an exhaustive reachability oracle. Every cycle witness must be closed and every
consecutive pair must be an actual edge; every redundancy witness must remain
valid after removing its reported direct edge. Namespace properties must be
invariant under enumeration order and independent-set extension.

Assert read-only behaviour across success and failure: selected documents,
exports, directory entries, and existing temporary-file inventories remain
unchanged. A proof obligation requires an exhaustive proof, bounded model check,
or documented finite-state argument for the production property, not an
unconnected demonstration model. Passing documentation checks does not mark any
of these implementation gates complete.

## 11. Rollout and review boundaries

1. Land the reviewed design and proposed ADRs without claiming implementation.
2. Implement 6.1.1's domain findings, recovery, and read-only command; complete
   shared scanner work in 1.2.2/1.2.3 before accepting full dependency coverage.
3. Deliver dependency/link checks, cycle witnesses, advisory hygiene, and human
   rendering through 6.1.2 to 6.1.5 with their independent acceptance fixtures.
4. Deliver global JSON and edit summaries through 6.2.1/6.2.2. Reconcile PR #153
   with 6.2.3's graph and output-profile requirements rather than replacing its
   in-flight implementation or marking the task done prematurely.
5. Add context and its checked skill through 6.2.4/6.2.5. Implement the accepted
   manifest and namespace decisions through 6.3.1 to 6.3.3.
6. Update users' and developers' guides only as commands become available,
   documenting the exit migration and distinct artefact/protocol versions.

Keep ExecPlan identity/apply work in phase 7. Cross-roadmap references, expanded
ADR/RFC semantic checks, and configurable rule policy require separate planning;
none is an implicit prerequisite for the bounded uniqueness extension.

## References

- [Agent-native CLI assistance design](https://github.com/leynos/ortho-config/blob/main/docs/agent-native-cli-design.md).
- [miette `Diagnostic` documentation](https://docs.rs/miette/latest/miette/trait.Diagnostic.html).
- [miette `GraphicalReportHandler` documentation](https://docs.rs/miette/latest/miette/struct.GraphicalReportHandler.html).
