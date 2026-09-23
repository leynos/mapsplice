#!/usr/bin/env bash
# Verify that every claim in the verification ledger names a source symbol.
#
# Each row of the ledger's claim table names, in its "Executable function"
# column, a Rust function that the product calls. This check requires that
# function to exist below SCAN_DIR/src so the ledger cannot survive as prose
# after a rename or a deletion. A claim whose kernel has been renamed is a
# claim about nothing.
#
# A row whose executable-function cell reads `Pending` is exempt: it declares
# work that has not landed rather than asserting a result.
#
# The declaration is matched line-anchored, so a name that survives only in a
# doc comment or a commented-out line does not satisfy a claim. That is the
# realistic drift this guards: renaming a kernel leaves prose references behind.
# What it does not do is mask string literals, so a claim name occurring only
# inside a line-initial string literal would still match. The check is
# necessary rather than sufficient, and the ledger's policy section says why
# the proofs themselves, not this script, are the evidence.
#
# Usage: check-verification-ledger.sh [SCAN_DIR]
#
# Exit status:
#   0  every named ledger symbol exists below SCAN_DIR/src
#   1  a ledger claim names a malformed or missing symbol, or names none
#   *  a file, tool, or ripgrep failure is propagated
set -euo pipefail

read -r -a rg_cmd <<<"${RG:-rg}"
scan_dir="${1:-.}"
ledger_path="${scan_dir}/docs/verification.md"
source_dir="${scan_dir}/src"

# The table starts with a pipe, so awk's field one is the empty text before it
# and the executable-function cell is field three. The header and the `| --- |`
# separator occupy that cell too, so both are filtered out.
claim_symbols="$(awk -F '|' '
function trim(value) {
    sub(/^[[:space:]]+/, "", value)
    sub(/[[:space:]]+$/, "", value)
    return value
}

/^\|/ {
    symbol = trim($3)
    gsub(/`/, "", symbol)
    if (symbol != "" && symbol != "Executable function" && symbol !~ /^-+$/ && symbol != "Pending") {
        print symbol
    }
}
' "${ledger_path}")"

if [[ -z "${claim_symbols}" ]]; then
    echo "verification ledger states no verifiable claims: ${ledger_path}" >&2
    exit 1
fi

while IFS= read -r symbol; do
    [[ -z "${symbol}" ]] && continue
    if [[ ! "${symbol}" =~ ^[[:alpha:]_][[:alnum:]_]*$ ]]; then
        echo "verification ledger names missing symbol: ${symbol}"
        exit 1
    fi

    # A declaration line is indentation, an optional visibility and qualifier
    # run, then `fn <name>` followed by a generic list, a parameter list, or
    # end of line. Requiring the line to start there is what stops a name that
    # survives only in a doc comment from satisfying a claim.
    visibility='(pub(\((crate|self|super|in[[:space:]]+[[:alnum:]_:]+)\))?[[:space:]]+)?'
    qualifiers='(const[[:space:]]+)?(async[[:space:]]+)?(unsafe[[:space:]]+)?'
    declaration="^[[:space:]]*${visibility}${qualifiers}fn[[:space:]]+${symbol}([[:space:]<(]|$)"

    status=0
    "${rg_cmd[@]}" --quiet --glob '*.rs' "${declaration}" "${source_dir}" || status=$?
    case "${status}" in
        0) ;;
        1)
            echo "verification ledger names missing symbol: ${symbol}"
            exit 1
            ;;
        *)
            echo "failed to scan verification symbols (rg exit ${status})" >&2
            exit "${status}"
            ;;
    esac
done <<<"${claim_symbols}"
