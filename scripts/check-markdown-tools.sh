#!/usr/bin/env bash
# Check the external formatters used by the golden round-trip tests.
set -euo pipefail

fail() {
  printf '%s\n' "check-markdown-tools: $*" >&2
  exit 1
}

MARKDOWN_TEST_TOOLS=${MARKDOWN_TEST_TOOLS:-'mdtablefix markdownlint-cli2'}
MDTABLEFIX_REQUIRED_VERSION=${MDTABLEFIX_REQUIRED_VERSION:-}
MARKDOWNLINT_CLI2_REQUIRED_VERSION=${MARKDOWNLINT_CLI2_REQUIRED_VERSION:-}

read -r -a markdown_test_tools <<<"$MARKDOWN_TEST_TOOLS"
for tool in "${markdown_test_tools[@]}"; do
  command -v "$tool" >/dev/null 2>&1 || \
    fail "$tool not found on PATH; see .github/workflows/ci.yml for the pinned installation"
done

if [[ -n $MDTABLEFIX_REQUIRED_VERSION ]]; then
  if ! actual=$(mdtablefix --version); then
    fail 'could not read mdtablefix version (see .github/workflows/ci.yml)'
  fi
  expected="mdtablefix $MDTABLEFIX_REQUIRED_VERSION"
  if [[ $actual != "$expected" ]]; then
    fail "mdtablefix version mismatch; expected '$expected', got '$actual' (see .github/workflows/ci.yml)"
  fi
fi

if [[ -n $MARKDOWNLINT_CLI2_REQUIRED_VERSION ]]; then
  markdownlint_cli2_path=$(command -v markdownlint-cli2)
  if ! actual=$(node - "$markdownlint_cli2_path" 2>/dev/null <<'NODE'
const fs = require("node:fs");
const path = require("node:path");

let directory = path.dirname(fs.realpathSync(process.argv[2]));
for (;;) {
  try {
    const packageJson = JSON.parse(
      fs.readFileSync(path.join(directory, "package.json"), "utf8"),
    );
    if (
      packageJson.name === "markdownlint-cli2" &&
      typeof packageJson.version === "string"
    ) {
      process.stdout.write(packageJson.version);
      process.exit(0);
    }
  } catch (error) {
    if (error.code !== "ENOENT") process.exit(1);
  }

  const parent = path.dirname(directory);
  if (parent === directory) process.exit(1);
  directory = parent;
}
NODE
  ); then
    fail 'could not read markdownlint-cli2 package version (see .github/workflows/ci.yml)'
  fi
  expected=$MARKDOWNLINT_CLI2_REQUIRED_VERSION
  if [[ $actual != "$expected" ]]; then
    fail "markdownlint-cli2 version mismatch; expected $MARKDOWNLINT_CLI2_REQUIRED_VERSION," \
      "got '$actual' (see .github/workflows/ci.yml)"
  fi
fi
