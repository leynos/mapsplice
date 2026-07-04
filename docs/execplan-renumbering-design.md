# ExecPlan renumbering design

## Status and scope

- **Status:** design for roadmap phase 7.
- **Scope:** discovering ExecPlan references, producing dry-run renumber plans,
  and defining the preconditions for any future apply engine after a roadmap
  edit renumbers tasks.
- **Out of scope:** general Markdown migration, remote link checking, and
  semantic rewriting of historical transcripts. Automated apply is conditional
  follow-on work, not part of the first phase 7 delivery.

## Problem statement

Roadmap task numbers are not contained inside `docs/roadmap.md`. They are also
encoded in ExecPlan filenames, titles, links, and live references inside design
documents. A roadmap edit that renumbers tasks can therefore leave the project
in a split state: the roadmap is correct, but the supporting plans still point
at old task numbers.

Renumbering ExecPlans is risky because plans also contain history. A transcript
or audit note that says a task was once `4.2.2` may be true historical evidence
and must not be rewritten as if it were a live link. Phase 7 therefore ships
detection and dry-run planning first. An apply engine waits until dry-run
reports show that manual guided renumbering is frequent or costly enough to
justify the extra failure-mode surface.

## Goals

- Map live roadmap task anchors to ExecPlan files.
- Convert a roadmap renumber map into a dry-run ExecPlan renumber plan.
- Detect missing, duplicate, stale, manually named, and conflicting ExecPlans.
- Define safe rename and rewrite preconditions for a future apply engine.
- Preserve historical evidence sections unless a policy explicitly marks a
  reference as live.
- Emit JSON dry-run reports.
- Fail closed before partial writes when conflicts are present.

## Non-goals

- Rewrite arbitrary prose numbers.
- Infer meaning from old transcripts without structural markers.
- Delete old ExecPlans automatically.
- Follow or validate remote URLs.
- Batch unrelated roadmap edits.
- Build one-command apply before demand is proven by dry-run usage.

## Alternative considered

Stable task IDs would remove most of phase 7: ExecPlans could reference durable
IDs while roadmap numbers remained presentation order. The current design
rejects that alternative for now because numbered anchors are the normative
roadmap grammar and `Requires` clauses depend on those numbers. This rejection
should be revisited if phase 7.1 dry-run reports show frequent renumber churn.

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
  "roadmap": {
    "path": "docs/roadmap.md",
    "source_hash": "sha256:old-roadmap",
    "destination_hash": "sha256:new-roadmap",
    "destination_anchors": ["6.2.5"]
  },
  "actions": [
    {
      "kind": "rename",
      "from": "docs/execplans/roadmap-6-2-4.md",
      "to": "docs/execplans/roadmap-6-2-5.md",
      "anchor_from": "6.2.4",
      "anchor_to": "6.2.5",
      "source_hash": "sha256:old-execplan",
      "destination_exists": false
    }
  ],
  "diagnostics": []
}
```

The plan is valid for a future apply command only when it contains no `error`
diagnostics, every action's source file still has the same content hash
observed during planning, and the on-disk roadmap already contains the
destination anchors named by the plan.

## Rewrite policy

The design uses a conservative live-versus-history rule. It is anchored to the
ExecPlan heading template rather than prose categories alone.

Live references are rewritten:

- conventional ExecPlan filenames;
- the document title heading when it names the current roadmap item;
- explicit metadata fields such as `Status` and future roadmap anchor fields;
- links from `docs/roadmap.md` and design documents to conventional ExecPlan
  paths;
- live references in `Purpose / big picture`, `Context and orientation`,
  `Plan of work`, `Concrete steps`, `Validation and acceptance`, and
  `Interfaces and dependencies`.

Historical references are preserved:

- `Progress`;
- `Surprises & Discoveries` / `Surprises & discoveries`;
- `Decision Log` / `Decision log`;
- `Outcomes & Retrospective` / `Outcomes & retrospective`;
- `Revision note`;
- fenced code blocks, command output, and quoted review comments unless a
  block is explicitly marked as generated live metadata.

Unrecognised sections default to preserve. Ambiguous references produce
`review_needed` diagnostics. Any future apply command must skip those files
unless the user passes an explicit override that is itself recorded in the JSON
report.

## Apply contract

The apply command is conditional. It should not be implemented until phase 7.1
dry-run reports show enough repeated manual work to justify automation. If that
threshold is met, the command shape should be:

```console
mapsplice execplans renumber --apply --plan renumber.json --json <target>
```

Apply preflight must require:

- a clean git worktree, unless the caller passes an explicit `--allow-dirty`
  escape hatch;
- no `error` diagnostics in the dry-run plan;
- a target-state check proving the on-disk roadmap contains the plan's
  destination anchors;
- matching roadmap and ExecPlan content hashes from the dry-run plan;
- no destination path conflicts;
- no `review_needed` diagnostics unless an explicit override names the files.

Apply then runs in five stages:

1. **Preflight:** validate the roadmap, validate the plan schema, confirm file
   hashes, and reject conflicts.
2. **Prepare:** build temporary rewritten files beside their targets.
3. **Rename:** move files in an order that avoids destination collisions.
4. **Rewrite:** replace live content references and linked paths.
5. **Postflight:** run validation, link checks, and idempotence checks.

If preflight fails, nothing is written. If prepare fails, temporary files are
removed. If rename or rewrite fails after a mutation, the user should recover
from git because the clean-worktree precondition makes every mutation
reviewable and reversible. The JSON report should still name every completed
and pending action when the process survives long enough to write it.

## Integration with roadmap edits

The edit pipeline already builds a renumber plan for roadmap anchors. Phase 7
should expose that plan as a reusable artefact rather than reparsing diffs.

Recommended flow:

```console
mapsplice insert --in-place --json docs/roadmap.md 6 fragment.md > edit.json
mapsplice execplans renumber --dry-run --plan edit.json --json docs/roadmap.md > renumber.json
# Either apply the dry-run manually, or use a future gated apply command.
```

For non-in-place roadmap edits, the caller must materialize the rewritten
roadmap first, then re-run dry-run planning against that target state. The
planner must reject a renumber map whose destination anchors are absent from
the on-disk roadmap.

## Diagnostics

ExecPlan renumbering should reuse the phase 6 diagnostic model. Required
finding codes include:

- `mapsplice::execplans::missing_plan`;
- `mapsplice::execplans::duplicate_plan`;
- `mapsplice::execplans::destination_conflict`;
- `mapsplice::execplans::ambiguous_reference`;
- `mapsplice::execplans::stale_plan_hash`;
- `mapsplice::execplans::target_state_mismatch`;
- `mapsplice::execplans::dirty_worktree`;
- `mapsplice::execplans::postflight_failed`.

Human diagnostics should use miette spans when the finding has a source
location. JSON diagnostics should include stable codes, severity, path, range,
help, and related locations.

## Testing strategy

- Fixture sets cover matching, stale, missing, duplicate, manual, conflicting,
  and ambiguous ExecPlans.
- Dry-run golden fixtures pin action ordering and diagnostics.
- Conditional apply tests, if phase 7.2 proceeds, use temporary capability
  roots and injected failures for preflight, prepare, rename, rewrite, and
  postflight stages.
- Markdown tests prove rewritten ExecPlans pass scoped formatting and linting.
- Idempotence tests assert that applying the same plan twice returns a clear
  stale-plan diagnostic.
- Historical-preservation fixtures prove transcripts and audit evidence remain
  unchanged.

## Rollout

1. Add ExecPlan discovery and classification.
2. Expose edit renumber maps in JSON output.
3. Add dry-run planning with conflicts, destination anchors, and content
   hashes.
4. Document the guided manual rename workflow.
5. Measure dry-run reports to decide whether apply automation is justified.
6. If justified, add the clean-worktree, target-state, and hash preflight
   checks before any file mutation.
7. Update the users' guide, developers' guide, and roadmap-maintenance skill.

## Open questions

- Should addendum sub-task ExecPlans use four-number filenames, nested
  directories, or remain manual until there is evidence of demand?
- Which ExecPlan sections should be structurally marked as historical in the
  long term?

## References

- [Validation and agent output design](validation-and-agent-output-design.md).
- [Mapsplice design](mapsplice-design.md).
- [Agent-native CLI assistance design](https://github.com/leynos/ortho-config/blob/main/docs/agent-native-cli-design.md).
