#!/usr/bin/env bash
# Direct Clang to the pinned PATH linker; Clang otherwise prefers /usr/bin.
set -euo pipefail

uses_pinned_linker() {
  local argument response_argument
  for argument in "$@"; do
    [[ $argument != -fuse-ld=mold ]] || return 0
    # rustc's E2BIG retry writes one argument per line. Read literal tokens;
    # never expand shell syntax or recursively interpret response-file entries.
    if [[ $argument == @* && -f ${argument#@} && -r ${argument#@} ]]; then
      while IFS= read -r response_argument || [[ -n $response_argument ]]; do
        [[ $response_argument != -fuse-ld=mold ]] || return 0
      done < "${argument#@}"
    fi
  done
  return 1
}

if uses_pinned_linker "$@"; then
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

# Release and coverage assign RUSTFLAGS to remove the development linker flag.
exec clang "$@"
