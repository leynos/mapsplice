#!/usr/bin/env bash
# Provision the pinned nightly and digest-verified Linux mold release.
set -euo pipefail

toolchain=$(awk -F '"' '/^[[:space:]]*channel[[:space:]]*=/ { print $2; exit }' rust-toolchain.toml)
[[ -n $toolchain ]] || { echo 'rust-toolchain.toml has no channel' >&2; exit 1; }
rustup toolchain install "$toolchain" --component rustfmt --component clippy \
  --component rust-analyzer --component llvm-tools-preview \
  --component rustc-codegen-cranelift-preview

if [[ $(uname -s) != Linux ]]; then
  exit 0
fi

case $(uname -m) in
  x86_64) digest=a3696680d99e692970590a178bc3a33d78d60d1c6dc9db7a11b557b02b751f5d ;;
  aarch64) digest=946de2774b06a71346bd59b55fddba610b65b8d93c3a4a1559cc84e103472710 ;;
  *) echo 'No mold default for this Linux architecture; installed Rust components only.'; exit 0 ;;
esac
arch=$(uname -m)
version=2.41.0
prefix=${BUILD_TOOLS_PREFIX:-"$HOME/.local"}
mkdir -p "$prefix/bin"
if [[ -f "$prefix/bin/.mold-$version-$digest" ]] &&
   [[ $("$prefix/bin/mold" --version) == "mold $version "* ]] &&
   [[ $("$prefix/bin/ld.mold" --version) == "mold $version "* ]]; then
  exit 0
fi

scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
archive="mold-$version-$arch-linux.tar.gz"
curl --fail --location --silent --show-error \
  "https://github.com/rui314/mold/releases/download/v$version/$archive" \
  --output "$scratch/$archive"
printf '%s  %s\n' "$digest" "$scratch/$archive" | sha256sum --check --status || {
  echo 'Pinned mold archive checksum failed' >&2
  exit 1
}
tar -xzf "$scratch/$archive" -C "$scratch" \
  "mold-$version-$arch-linux/bin/mold" \
  "mold-$version-$arch-linux/bin/ld.mold"
install -m 0755 "$scratch/mold-$version-$arch-linux/bin/mold" "$prefix/bin/mold"
install -m 0755 "$scratch/mold-$version-$arch-linux/bin/ld.mold" "$prefix/bin/ld.mold"
[[ $("$prefix/bin/mold" --version) == "mold $version "* ]] || {
  echo 'Installed mold does not report the pinned version' >&2
  exit 1
}
touch "$prefix/bin/.mold-$version-$digest"
printf 'Installed mold %s into %s/bin; put it before system tools on PATH.\n' "$version" "$prefix"
