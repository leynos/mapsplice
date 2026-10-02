#!/usr/bin/env bash
# Select the target before the shared wrapper resolves the pinned linker.
set -euo pipefail

exec "$(dirname -- "$0")/clang-linker.sh" --target=aarch64-unknown-linux-gnu "$@"
