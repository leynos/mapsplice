# Architectural decision record (ADR) 002: Explicit documentation-set discovery

## Status

Proposed. This record specifies the phase 6 documentation-set extension for
review; it does not describe an implemented command or authorize automatic
repository-wide scans.

## Date

2026-10-14.

## Context and problem statement

[Issue #150](https://github.com/leynos/mapsplice/issues/150) extends validation
from anchors in one roadmap to sequence numbers in a documentation set. The
existing single-file command does not say which other documents belong to that
set. Guessing from the current directory, every Markdown filename, or a user's
global configuration would make findings depend on where and who runs the tool.
A monorepository may contain independent decision-record series, generated
copies, archives, and companion roadmaps.

Discovery must therefore establish a bounded, reproducible inventory before
the shared uniqueness checks run. [ADR 003](adr-003-documentation-namespaces.md)
defines identity and uniqueness within that inventory. Neither decision turns
ordinary documentation into roadmap grammar.

## Decision drivers

- Preserve the scope and read-only behaviour of single-roadmap validation.
- Make the complete checked inventory visible to humans and agents.
- Support repositories and standalone documentation without requiring Git.
- Prevent filesystem traversal from silently widening the declared scope.
- Keep discovery separate from domain checks and free of ambient user policy.

## Options considered

- Recursively infer sets from all repository Markdown. This requires almost no
  setup, but mixes independent series and includes generated or vendored files.
- Infer a set from each directory. This makes moving an archive change its
  namespace and cannot express one series spread over several directories.
- Use an explicit, versioned manifest with conventional filename classifiers.
  This adds a small configuration file but makes membership reviewable and
  supports both split and independent series. This is the proposed direction.

## Proposed direction

### Command selection and manifest ownership

Keep `mapsplice validate [--json] <target>` confined to the named roadmap. Add
an alternative, mutually exclusive command form:

```console
mapsplice validate --documentation-set mapsplice-docs.toml --json
```

The flag takes a manifest path, not a directory or glob. `mapsplice-docs.toml`
is the recommended filename, not an implicitly loaded configuration source.
Providing both a target and the flag, or neither, is a usage error. Existing
OrthoConfig defaults, environment variables, home-directory configuration, and
working-directory changes must not add sets or alter their membership. The
explicit manifest owns this contract.

The manifest's parent directory is the workspace root. Resolve the manifest
argument from the invocation directory once, open a capability for its parent,
and resolve all inventory paths relative to that capability. A manifest placed
below a Git root intentionally selects a narrower workspace. Standalone
folders use exactly the same rule; no Git executable is required.

The following manifest declares two numbered record series and two roadmaps.
The companion roadmap is an illustrative path, not a Mapsplice repository file.

```toml
schema_version = 1

[[sets]]
name = "architecture"
kind = "adr"
directories = ["docs"]
recursive = false
exclude = []

[[sets]]
name = "proposals"
kind = "rfc"
directories = ["docs/rfcs"]
recursive = false
exclude = []

[[sets]]
name = "plans"
kind = "roadmap"
files = ["docs/roadmap.md", "docs/validation-roadmap.md"]
```

Reject unknown schema versions, unknown keys or kinds, duplicate set names,
empty names, and an empty `sets` array as usage errors. Set names must match
`[a-z][a-z0-9_-]*`. ADR and RFC sets require a non-empty `directories` array;
`recursive` defaults to `false`, and `exclude` defaults to an empty array.
Roadmap sets require a non-empty `files` array and do not accept directory,
recursion, or exclusion fields. Version 1 has no globs, arbitrary regular
expressions, include files, or configuration inheritance.

### Path selection and containment

Inventory paths use UTF-8, workspace-relative `/` separators. Reject absolute
paths, drive-prefixed paths, backslashes, empty components, `.` components,
`..` components, and NUL bytes; the entire directory path `.` alone denotes
the workspace root. Do not case-fold or Unicode-normalize paths.

`exclude` contains exact file or directory paths relative to the workspace.
A directory exclusion also excludes its descendants. Exclusions apply before
classification and recursion. Duplicate paths in one set collapse to one
inventory entry. Reject exclusions outside that set's declared directories or
exclusions that do not identify an existing entry, so misspellings cannot
quietly change intended coverage.

Enumerate the working tree, including untracked and Git-ignored regular files;
`.gitignore` does not define membership. Exclude generated copies explicitly
in the manifest. Do not read only the Git index or require tracked files.
Inspect each declared directory; descend into children only when
`recursive = true`; never traverse a symlink, a nested Git worktree or
submodule, or a `.git` entry. A `.git` file and a `.git` directory both mark a
nested repository boundary. Excluded paths may deliberately omit such a
boundary. Otherwise, a requested traversal through a symlink or nested
repository is a validation error, not a successful partial scan. Explicit
roadmap files and every component of selected paths obey the same rules.
Reject a symlink manifest rather than silently relocating its workspace.

Use capability-relative, no-follow opens for enumeration and reading; lexical
prefix checks alone are insufficient. Do not execute repository hooks or
follow links to discover additional set members. A non-UTF-8 selected path
produces a located validation error rather than a lossy identity. A missing
manifest, declared directory, or explicit roadmap, or an unreadable selected
entry, produces an I/O failure. No failure may be reported as an empty,
successfully checked set.

### Filename classifiers and inventory evidence

The ADR classifier selects regular `.md` files beginning with `adr-`; valid
record names have the form `adr-<digits>-<non-empty-description>.md`. The RFC
classifier selects regular `.md` files whose first character is an ASCII digit;
valid names have the form `<digits>-<non-empty-description>.md`. Classification
is case-sensitive. A candidate with an invalid number or filename shape is a
validation finding, not an ignored document. Other files, including `README.md`
and templates without those prefixes, do not claim sequence numbers. This
slice does not enforce title style or zero-padding width.

An existing selected directory with no record candidates is an
`empty_documentation_set` validation error. An empty recursive child directory
is harmless if the set as a whole contains candidates. Explicitly listed
roadmaps must parse as roadmaps; filename heuristics do not determine their
identity. ADR/RFC contents do not pass through the roadmap parser.

Return the manifest schema version, selected set names and kinds, sorted
workspace-relative document paths, and explicit exclusions with the validation
result. Also report counts of visited regular files and non-candidate files per
set; an empty selected inventory must remain visible. Sort sets by name and
paths by UTF-8 byte order, independent of directory enumeration order. Keep
source order for findings inside a document. A physical path selected by two
different sets is an ambiguous-membership error under ADR 003, not two checks.

The inventory describes files observed during this invocation, not an atomic
filesystem snapshot. Read each selected file once into an immutable source
buffer. A detected disappearance, replacement, or read failure aborts complete
success; do not claim that a mutable worktree is a merge-time uniqueness lock.

## Consequences and limitations

A maintainer must declare unusual layouts rather than rely on guesses. Archives
can remain in the same series through multiple directories or explicit
recursion, while generated copies can be excluded deliberately. Broad discovery
will not cross a nested repository without a separate invocation and manifest.
The no-symlink rule also excludes otherwise harmless aliases in exchange for a
smaller, portable containment contract.

The inventory adapter supplies plain document identities and source buffers to
the existing validation domain. It must not create a second parser or duplicate
uniqueness semantics. Local link checks may inspect targets within the workspace
but do not enlarge its inventory or the scope of sequence-number claims.

## Migration and acceptance

Implement this contract through roadmap 6.3.1, then combine it with the
namespace checks in 6.3.2 and the reporting tests in 6.3.3. Until those tasks
land, examples here are proposed interfaces, not instructions for the released
binary.

Acceptance fixtures must cover invocation from different directories,
standalone folders, multiple series, recursive archives, explicit exclusions,
unknown configuration, malformed filenames, empty and missing sets, overlapping
sets, unreadable files, symlink escape and alias attempts, nested `.git` files
and directories, and deterministic inventory order. Tests must also prove that
single-roadmap validation does not discover neighbouring documents.

## Outstanding decisions

Maintainer review must accept or amend this proposed command spelling and
manifest schema before implementation. Convenience auto-discovery is deferred;
it must not change the meaning of either explicit command form.
