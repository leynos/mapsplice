# Architectural decision record (ADR) 003: Scope documentation identities explicitly

## Status

Proposed. This record defines the namespace contract for review alongside
[ADR 002](adr-002-documentation-set-discovery.md). Implementation remains
unchecked in roadmap phase 6.

## Date

2026-10-14.

## Context and problem statement

[Issue #150](https://github.com/leynos/mapsplice/issues/150) reports two files
claiming ADR 006 in one series. That is a collision, whereas task `1.1.1` in
both a primary and a companion roadmap is not. A repository-wide integer set
would conflate those cases. A per-directory integer set would miss collisions
when one series spans current and archived records.

Discovery determines which documents participate. Identity must separately say
which claims compete and how dependency lookup selects a target. This contract
extends the existing duplicate-anchor check without inventing global task
numbers or changing the meaning of an unqualified `Requires` clause.

## Decision drivers

- Detect duplicate record numbers across all directories of a declared series.
- Preserve independent ADR/RFC series and companion roadmap task numbers.
- Treat padding as presentation rather than a way to evade uniqueness checks.
- Keep diagnostics deterministic and identify every competing claimant.
- Avoid accidental promises of durable identity or cross-roadmap scheduling.

## Options considered

- Make every number unique repository-wide. This rejects legitimate companion
  roadmaps and independent ADR/RFC series.
- Use the containing directory as the namespace. This makes filesystem layout
  define identity and misses a series spread across multiple directories.
- Use manifest-owned record namespaces and document-owned roadmap namespaces.
  This expresses both cases directly and is the proposed direction.

## Proposed direction

### Workspace, set, and document identities

The workspace is the capability-rooted scope established by ADR 002. Its
host-specific absolute path is not a serialized identity. Every serialized
reference is relative to that workspace; consumers combining results from
different workspaces must supply their own workspace discriminator.

A set has a manifest-unique name and a declared kind. A document has its exact,
normalized workspace-relative UTF-8 path. Neither display titles, directory
basenames, filesystem inode numbers, nor enumeration order define identity.
Case and Unicode normalization remain unchanged. Two hard-linked files at
distinct paths are two document claims, not a deduplicated inode.

Repeated selection of the same path within one set produces one document.
Selection of one path by different sets is
`mapsplice::validate::ambiguous_membership` at error severity, even when the
sets have the same kind. Report every selecting set and its manifest location;
do not assign the file to whichever set happens to run first.

### Sequence-number namespaces

An ADR or RFC record claims the tuple `(set kind, set name, sequence number)`
within the workspace. Directories do not participate in that tuple. An archive
included in the same set therefore shares its number space with current files.
Independent sets may each contain record 006; ADR 006 and RFC 0006 also occupy
different namespaces.

Parse filename numbers as non-empty ASCII decimal digits, remove leading zeros,
and reject an all-zero value. Compare the remaining decimal strings without
machine-integer truncation or overflow. Thus `6`, `006`, and `0006` denote the
same sequence number. Width and padding remain style concerns, not distinct
identities. A malformed candidate reports `invalid_sequence_number` or
`invalid_record_filename`, as appropriate.

Two or more different documents claiming the same tuple produce one
`mapsplice::validate::duplicate_sequence_number` error per tuple. Choose the
first claimant by path order as the primary location and include every other
claimant as a related location. Filename-only findings have a path and no
fabricated text range. Structured evidence includes the set kind, set name,
canonical number, and ordered claimant paths.

Check uniqueness only, not contiguity, allocation order, or monotonicity. Gaps
are legal and produce no warning. A withdrawn or superseded record that remains
in the inventory still claims its number. A deleted or excluded record does not;
version 1 does not infer historical reservations from Git or maintain an
allocation ledger. It neither renumbers files nor recommends filling gaps.

### Roadmap-local anchors and dependencies

A roadmap anchor claims `(document path, anchor components)` within the
workspace. Existing roadmap grammar and canonical anchor parsing remain
authoritative, including phase, step, task, and structural sub-task levels.
Duplicate anchors within one document remain
`mapsplice::validate::duplicate_anchor` errors. A `roadmap` set groups files for
validation; it does not merge their anchor spaces.

The `Requires` scanner resolves bare anchors only against the containing
roadmap. A matching number in another selected roadmap cannot satisfy a missing
local dependency. Graph vertices and edges retain their document identity even
when several independent graphs appear in one validation result. Unnumbered
checkbox details have source-local owner identities, not referenceable anchors.
Their status and outgoing requirements must not be silently attributed to the
parent task.

These examples illustrate the intended boundaries:

| Claims | Result |
| --- | --- |
| `docs/adr-006-a.md` and `docs/archive/adr-6-b.md` in `architecture` | Duplicate record number. |
| Record 006 in `architecture` and in `payments` | Independent record identities. |
| ADR 006 and RFC 0006 in their respective sets | Independent record identities. |
| Task `1.1.1` in `docs/roadmap.md` and `docs/validation-roadmap.md` | Independent task identities. |
| Two occurrences of task `1.1.1` in `docs/roadmap.md` | Duplicate anchor. |
| A local `Requires 2.1.1` whose only match is in another roadmap | Missing local dependency. |

_Table 1: Record and roadmap namespace examples._

Markdown links to another file and fragment remain link references, not
cross-document dependency edges. This slice does not introduce qualified
`Requires` syntax. A future cross-roadmap graph requires an explicit grammar,
resolution, cycle, and compatibility decision rather than falling back to a
repository-wide number lookup.

### Identity lifetime and exclusions

Document identity changes when its workspace-relative path changes. Record
identity survives a filename-description change within the same declared set,
but changes when the set name or sequence number changes. Task identity changes
when its document path or anchor changes. These are snapshot identities, not
permanent identifiers; phase 7's renumber maps remain the mechanism for relating
old and new task anchors.

ADRs and RFCs must not inherit the roadmap editor's sequential renumbering
policy. Filename/title agreement, record indexes, supersession validity,
semantic traceability, and proof adequacy remain outside this uniqueness slice.
Excluding a document changes the checked set, not its historical meaning.

## Consequences

One shared uniqueness kernel can group claims by a typed namespace and
canonical identifier while preserving domain-specific diagnostics. Discovery,
parsing, dependency analysis, and JSON rendering consume those same identities;
none may flatten them to an unqualified string key. Keep the production kernel
pure and directly testable rather than verifying a separate surrogate model.

Namespace declarations require review. The tool can detect overlapping
membership but cannot infer that a maintainer accidentally split a real series
into two non-overlapping sets. Reports must therefore include scope as well as
findings. Local validation detects the observed state; it does not prevent two
concurrent branches from allocating the same next number.

## Migration and acceptance

Implement through roadmap 6.3.2 and 6.3.3 after ADR 002's inventory boundary.
Fixtures must cover every row of Table 1, leading-zero equivalence, all-zero and
malformed numbers, long decimal identifiers without overflow, legal gaps,
withdrawn records, archive membership, repeated paths, ambiguous membership,
three-way collisions, and deterministic primary and related locations.

Property tests must show that changing enumeration order never changes
findings, adding an independent namespace never creates a collision in an
existing namespace, and every reported collision contains distinct documents
with the same canonical key. Cross-roadmap fixtures must prove that another
document cannot repair a missing local dependency.

## Outstanding decisions

Maintainer review must accept or amend the proposed namespace keys before
implementation. Durable identifiers, historical number reservation, and
cross-roadmap dependencies require separate decisions and are not prerequisites
for this bounded uniqueness check.
