#!/usr/bin/env bash
# Direct Clang to the pinned PATH linker; Clang otherwise prefers /usr/bin.
set -euo pipefail

for argument in "$@"; do
  if [[ $argument == -fuse-ld=mold ]]; then
    linker=$(command -v ld.mold) || {
      echo 'Pinned ld.mold is missing; run make install-build-tools' >&2
      exit 1
    }
    [[ $("$linker" --version) == 'mold 2.41.0 '* ]] || {
      echo 'Pinned ld.mold 2.41.0 is required; run make install-build-tools' >&2
      exit 1
    }
    exec clang -B "$(dirname "$linker")" "$@"
  fi
done

# Release and coverage assign RUSTFLAGS to remove the development linker flag.
exec clang "$@"
