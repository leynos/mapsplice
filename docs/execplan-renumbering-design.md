# ExecPlan renumbering design

## Status and scope

- **Status:** design for roadmap phase 7.
- **Scope:** planning and applying ExecPlan filename, heading, link, and live
  citation rewrites after a roadmap edit renumbers tasks.
- **Out of scope:** general Markdown migration, remote link checking, and
  semantic rewriting of historical transcripts.

## Problem statement

Roadmap task numbers are not contained inside `docs/roadmap.md`. They are also
encoded in ExecPlan filenames, titles, links, and live references inside design
documents. A roadmap edit that renumbers tasks can therefore leave the project
in a split state: the roadmap is correct, but the supporting plans still point
at old task numbers.

Renumbering ExecPlans is risky because plans also contain history. A transcript
or audit note that says a task was once `4.2.2` may be true historical evidence
and must not be rewritten as if it were a live link. Phase 7 therefore needs a
two-part design: first produce a reviewable plan, then apply only the safe
subset with validation before and after.

## Goals

- Map live roadmap task anchors to ExecPlan files.
- Convert a roadmap renumber map into a dry-run ExecPlan renumber plan.
- Detect missing, duplicate, stale, manually named, and conflicting ExecPlans.
- Rename ExecPlan files safely.
- Rewrite live links, titles, metadata, and current-roadmap references.
- Preserve historical evidence sections unless a policy explicitly marks a
  reference as live.
- Emit JSON dry-run and apply reports.
- Fail closed before partial writes when conflicts are present.

## Non-goals

- Rewrite arbitrary prose numbers.
- Infer meaning from old transcripts without structural markers.
- Delete old ExecPlans automatically.
- Follow or validate remote URLs.
- Batch unrelated roadmap edits.

## Identity model

The primary identity is a roadmap task anchor. The conventional ExecPlan path
is:

```text
docs/execplans/roadmap-<phase>-<step>-<task>.md
```

For example, task `6.2.4` maps to:

```text
docs/execplans/roadmap-6-2-4.md
```

The initial implementation should treat addendum sub-tasks as review-required
unless a later roadmap item defines a fourth-level filename convention.

Each discovered ExecPlan is classified as one of:

| Class         | Meaning                                           |
| ------------- | ------------------------------------------------- |
| matching      | Filename and live title match a current task.     |
| stale         | Filename maps to an old task in the renumber map. |
| missing       | A task has no conventional ExecPlan.              |
| duplicate     | More than one file claims the same live task.     |
| manual        | The filename does not follow the convention.      |
| conflict      | The proposed destination path already exists.     |
| review_needed | The file contains ambiguous live references.      |

## Dry-run plan

The dry-run command shape should be:

```console
mapsplice execplans renumber --dry-run --json <target>
```

When paired with a preceding edit, implementation may also accept a renumber
map artefact:

```console
mapsplice execplans renumber --plan renumber.json --json <target>
```

The dry-run report lists proposed actions without changing files:

```json
{
  "schema_version": "1",
  "kind": "mapsplice.execplan_renumber_plan",
  "target": "docs/roadmap.md",
  "actions": [
    {
      "kind": "rename",
      "from": "docs/execplans/roadmap-6-2-4.md",
      "to": "docs/execplans/roadmap-6-2-5.md",
      "anchor_from": "6.2.4",
      "anchor_to": "6.2.5"
    }
  ],
  "diagnostics": []
}
```

The plan is valid for apply only when it contains no `error` diagnostics and
when every action's source file still has the same content hash observed during
planning.

## Rewrite policy

The design uses a conservative live-versus-history rule.

Live references are rewritten:

- conventional ExecPlan filenames;
- document title headings that name the current roadmap item;
- explicit metadata fields that name the roadmap item;
- links from `docs/roadmap.md` and design documents to conventional ExecPlan
  paths;
- current-work sections that are structurally marked as status, next steps, or
  roadmap references.

Historical references are preserved:

- progress logs;
- transcripts;
- audit evidence;
- command output;
- quoted review comments;
- dated observations;
- fenced code blocks unless a block is explicitly marked as generated live
  metadata.

Ambiguous references produce `review_needed` diagnostics. The apply command
must skip those files unless the user passes an explicit future override that
is itself recorded in the JSON report.

## Apply contract

The apply command shape should be:

```console
mapsplice execplans renumber --apply --plan renumber.json --json <target>
```

Apply runs in five stages:

1. **Preflight:** validate the roadmap, validate the plan schema, confirm file
   hashes, and reject conflicts.
2. **Prepare:** build temporary rewritten files beside their targets.
3. **Rename:** move files in an order that avoids destination collisions.
4. **Rewrite:** replace live content references and linked paths.
5. **Postflight:** run validation, link checks, and idempotence checks.

If preflight fails, nothing is written. If prepare fails, temporary files are
removed. If rename or rewrite fails after a mutation, the report must include a
recovery section naming every completed and pending action. The first
implementation may stop short of transactional rollback, but it must not hide a
partial apply.

## Integration with roadmap edits

The edit pipeline already builds a renumber plan for roadmap anchors. Phase 7
should expose that plan as a reusable artefact rather than reparsing diffs.

Recommended flow:

```console
mapsplice insert --json docs/roadmap.md 6 fragment.md > edit.json
mapsplice execplans renumber --dry-run --plan edit.json --json docs/roadmap.md > renumber.json
mapsplice execplans renumber --apply --plan renumber.json --json docs/roadmap.md
```

The same flow should work with `--in-place`. In that case the edit JSON report
names the rewritten target and includes the renumber mapping used by the
ExecPlan planner.

## Diagnostics

ExecPlan renumbering should reuse the phase 6 diagnostic model. Required
finding codes include:

- `mapsplice::execplans::missing_plan`;
- `mapsplice::execplans::duplicate_plan`;
- `mapsplice::execplans::destination_conflict`;
- `mapsplice::execplans::ambiguous_reference`;
- `mapsplice::execplans::stale_plan_hash`;
- `mapsplice::execplans::postflight_failed`.

Human diagnostics should use miette spans when the finding has a source
location. JSON diagnostics should include stable codes, severity, path, range,
help, and related locations.

## Testing strategy

- Fixture sets cover matching, stale, missing, duplicate, manual, conflicting,
  and ambiguous ExecPlans.
- Dry-run golden fixtures pin action ordering and diagnostics.
- Apply tests use temporary capability roots and injected failures for prepare,
  rename, rewrite, and postflight stages.
- Markdown tests prove rewritten ExecPlans pass scoped formatting and linting.
- Idempotence tests assert that applying the same plan twice is a no-op or a
  clear stale-plan diagnostic.
- Historical-preservation fixtures prove transcripts and audit evidence remain
  unchanged.

## Rollout

1. Add ExecPlan discovery and classification.
2. Expose edit renumber maps in JSON output.
3. Add dry-run planning with conflicts and content hashes.
4. Add safe filename renames.
5. Add live-reference rewrites.
6. Add postflight validation and recovery reports.
7. Update the users' guide, developers' guide, and roadmap-maintenance skill.

## Open questions

- Should addendum sub-task ExecPlans use four-number filenames, nested
  directories, or remain manual until there is evidence of demand?
- Should apply support automatic rollback, or is a recovery report sufficient
  for the first implementation?
- Which ExecPlan sections should be structurally marked as historical in the
  long term?

## References

- [Validation and agent output design](validation-and-agent-output-design.md).
- [Mapsplice design](mapsplice-design.md).
- [Agent-native CLI assistance design](../ortho-config/docs/agent-native-cli-design.md).
