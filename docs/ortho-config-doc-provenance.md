# ortho_config documentation provenance

This document records the origin of the upstream `ortho_config` reference
material stored in `docs/`. It exists so a reviewer can reproduce each import
and so that relative links inside the imported text remain interpretable even
though the destination filenames differ from the upstream names.

## Imported revision

| Field               | Value                                      |
| ------------------- | ------------------------------------------ |
| Upstream repository | `https://github.com/leynos/ortho-config`   |
| Tag                 | `v0.10.0`                                  |
| Tag object          | `996e5a1c9f22ace4f55bf5dcc196d2c0524262bf` |
| Peeled commit       | `38812c563e3953a490e3efc202609f37e21a0517` |
| Import date         | 2026-11-14                                 |

The tag is annotated. `git rev-parse 'v0.10.0^{tag}'` returns the tag object and
`git rev-parse 'v0.10.0^{commit}'` returns the commit shown above. No `main`,
`HEAD`, or latest-release URL was used.

## Imported files

| Upstream path                     | Destination path                               | SHA-256                                                            |
| --------------------------------- | ---------------------------------------------- | ------------------------------------------------------------------ |
| `docs/users-guide.md`             | `docs/ortho-config-users-guide.md`             | `84d1c97b77024224dc9220a12d4b749205d9bb2db54849185da21c969dda5575` |
| `docs/v0-10-0-migration-guide.md` | `docs/ortho-config-v0-10-0-migration-guide.md` | `b5331aa6cfcc07b3dcd188c620db98d0a1a1f13b0c8ef96d88913d701bca25bf` |
| `docs/v0-9-0-migration-guide.md`  | `docs/ortho-config-v0-9-0-migration-guide.md`  | `c174c424f91c055201d6a8ec644b0a3b2c7aa681ffbd052d9d6152d78eb2f49d` |

The imported text is byte-for-byte identical to the upstream files. The
destination names add an `ortho-config-` prefix so that the guides are grouped
together and cannot be mistaken for repository-owned user documentation.

## Reproduction

From a checkout of `leynos/ortho-config` at the pinned commit:

```console
$ git rev-parse HEAD
38812c563e3953a490e3efc202609f37e21a0517
$ sha256sum docs/users-guide.md docs/v0-10-0-migration-guide.md docs/v0-9-0-migration-guide.md
84d1c97b77024224dc9220a12d4b749205d9bb2db54849185da21c969dda5575  docs/users-guide.md
b5331aa6cfcc07b3dcd188c620db98d0a1a1f13b0c8ef96d88913d701bca25bf  docs/v0-10-0-migration-guide.md
c174c424f91c055201d6a8ec644b0a3b2c7aa681ffbd052d9d6152d78eb2f49d  docs/v0-9-0-migration-guide.md
```

Copy the three files to their destination paths and confirm equality with
`cmp`. Re-import from the same commit if any checksum changes.

## Relative link mapping

The imported text keeps its original links. Links beginning with `#` resolve
within the same document and need no mapping. The remaining relative links name
files that were not imported; resolve them against the pinned upstream commit.

| Link text in imported docs                               | Destination in this repository                                                                                                                     |
| -------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `users-guide.md` and `users-guide.md#...`                | `docs/ortho-config-users-guide.md` (same anchor)                                                                                                   |
| `v0-10-0-migration-guide.md` and its anchors             | `docs/ortho-config-v0-10-0-migration-guide.md` (same anchor)                                                                                       |
| `v0-9-0-migration-guide.md` and its anchors              | `docs/ortho-config-v0-9-0-migration-guide.md` (same anchor)                                                                                        |
| `../CHANGELOG.md`                                        | `https://github.com/leynos/ortho-config/blob/38812c563e3953a490e3efc202609f37e21a0517/CHANGELOG.md`                                                |
| `../README.md`                                           | `https://github.com/leynos/ortho-config/blob/38812c563e3953a490e3efc202609f37e21a0517/README.md`                                                   |
| `../ortho_l10n/README.md`                                | `https://github.com/leynos/ortho-config/blob/38812c563e3953a490e3efc202609f37e21a0517/ortho_l10n/README.md`                                        |
| `../examples/hello_world/`                               | `https://github.com/leynos/ortho-config/tree/38812c563e3953a490e3efc202609f37e21a0517/examples/hello_world`                                        |
| `cargo-orthohelp-design.md`                              | `https://github.com/leynos/ortho-config/blob/38812c563e3953a490e3efc202609f37e21a0517/docs/cargo-orthohelp-design.md`                              |
| `cargo-orthohelp-api-guide.md`                           | `https://github.com/leynos/ortho-config/blob/38812c563e3953a490e3efc202609f37e21a0517/docs/cargo-orthohelp-api-guide.md`                           |
| `adrs/adr-0004-cargo-external-subcommand-entry-point.md` | `https://github.com/leynos/ortho-config/blob/38812c563e3953a490e3efc202609f37e21a0517/docs/adrs/adr-0004-cargo-external-subcommand-entry-point.md` |

`docs/ortho-config-users-guide.md` replaces an earlier, smaller import that had
no recorded provenance. The v0.9.0 migration guide is retained as a historical
record because this repository migrated from v0.8.0, not directly from v0.9.0.

## Local migration notes

Repository-owned notes about how this codebase applied the guides live in
`docs/execplans/issue-182-migrate-from-ortho-config-v0-8-0-to-v0-10-0.md` and
`docs/developers-guide.md`. They are deliberately kept outside the imported
files.
