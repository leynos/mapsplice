#!/usr/bin/env bash
# Check the tools selected by the committed development Cargo configuration.
set -euo pipefail

fail() {
  printf 'Build prerequisite missing: %s. Run make install-build-tools; install clang with your system package manager if needed.\n' "$1" >&2
  exit 1
}

command -v rustc >/dev/null 2>&1 || fail 'rustc'
command -v rustup >/dev/null 2>&1 || fail 'rustup'
components=$(rustup component list --installed) || fail 'pinned Rust toolchain'
for component in rustfmt clippy rust-analyzer llvm-tools; do
  if ! grep -q "^${component}-" <<<"$components"; then
    fail "$component for the selected nightly"
  fi
done

if [[ $(uname -s) == Linux && $(uname -m) =~ ^(x86_64|aarch64)$ ]]; then
  command -v clang >/dev/null 2>&1 || fail 'clang'
  command -v mold >/dev/null 2>&1 || fail 'mold 2.41.0'
  command -v ld.mold >/dev/null 2>&1 || fail 'ld.mold 2.41.0'
  [[ $(mold --version) == 'mold 2.41.0 '* ]] || fail 'mold 2.41.0'
fi
