#!/usr/bin/env bash
# Confirm that a regression in the shared source-span body cannot pass.
#
# The kernel body is shared between the compiled product and the Verus proof.
# A proof that never reaches that body would still report success, so this
# control mutates only the executable text in a temporary copy of the tracked
# tree, leaves the specification alone, and requires the verifier to attribute
# a failure to `line_start_offset`. It then mutates the same body again and
# requires the two runtime preservation tests to fail, so the proof and the
# product are both shown to be sensitive to the defect.
#
# Two mutations are used because they exercise different obligations:
#
#   cr      removes the CR test from the loop condition. The scan's own
#           progress invariant rejects it, which is the earliest attributable
#           obligation and the one that guards the CR-only source policy.
#   return  changes the returned index. The loop invariant still holds, so the
#           function's postcondition is the obligation that rejects it.
#
# A loop-based scan cannot produce a postcondition failure from the `cr`
# mutation: any invariant strong enough to prove the postcondition also
# describes the skipped region, so the defect is caught at the end of the loop
# body first. The `return` mutation is therefore the postcondition control,
# and both outcomes are attributed to the same function.
#
# Usage: check-source-span-mutation.sh [--repo-root DIR] [--verus-only]
#
# `--verus-only` stops after the verifier controls. `make verus-mutation` runs
# the whole script; the harness tests use the shorter form so they can exercise
# the attributed, accepted, unattributed, startup-failure and zero-match paths
# without compiling a Cargo tree for every case.
#
# Environment:
#   VERUS_RUN   Required. The command prefix that runs Verus, without
#               `--proof-file`; `make verus-mutation` supplies it.
#   CARGO       Cargo executable for the runtime controls (default: cargo).
#   CARGO_TARGET_DIR
#               Target directory for the runtime controls. Defaults to a
#               directory inside the temporary copy.
#
# Exit status:
#   0  both mutations were rejected for the expected, attributed reasons
#   1  a mutation was accepted, misattributed, or failed in an unexpected way
#   2  the mutation's exact source text was not found exactly once
#   *  a file, tool, or git failure is propagated
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
cargo="${CARGO:-cargo}"

verus_only=false
while (($# > 0)); do
    case "$1" in
        --repo-root)
            repo_root="$(cd "$2" && pwd)"
            shift 2
            ;;
        --verus-only)
            verus_only=true
            shift
            ;;
        *)
            echo "unknown argument: $1" >&2
            exit 2
            ;;
    esac
done

if [[ -z "${VERUS_RUN:-}" ]]; then
    echo "VERUS_RUN must name the configured Verus invocation" >&2
    exit 1
fi

kernel_rel="src/roadmap/source_span_kernel.macro.rs"
proof_rel="verus/lib.rs"
kernel="$repo_root/$kernel_rel"

copy="$(mktemp -d)"
trap 'rm -rf "$copy"' EXIT

# Copy the tracked tree, so untracked build output and local edits cannot make
# the control pass or fail for the wrong reason.
git -C "$repo_root" archive --format=tar HEAD | tar -x -C "$copy"

copied_kernel="$copy/$kernel_rel"
if [[ ! -f "$copied_kernel" ]]; then
    echo "the tracked tree has no kernel at $kernel_rel" >&2
    exit 1
fi

pristine="$(mktemp)"
cp "$copied_kernel" "$pristine"
restore_kernel() {
    cp "$pristine" "$copied_kernel"
}
trap 'rm -rf "$copy"; rm -f "$pristine"' EXIT

# Replace exactly one occurrence of OLD with NEW in FILE.
#
# A substitution that matches zero or several times is exit 2: a control that
# silently mutates nothing would report a false failure, and one that mutates
# more than intended would no longer be attributable to the shared scan.
mutate_once() {
    local file=$1 old=$2 new=$3
    python3 -I - "$file" "$old" "$new" <<'PY'
import sys

path, old, new = sys.argv[1:4]
with open(path, encoding="utf-8") as handle:
    text = handle.read()
if text.count(old) != 1:
    sys.exit(2)
with open(path, "w", encoding="utf-8") as handle:
    handle.write(text.replace(old, new))
PY
}

# Return the line range of one literal kernel declaration in the copied macro.
declaration_range() {
    local name=$1
    python3 -I - "$copied_kernel" "$name" <<'PY'
import sys

import re

path, name = sys.argv[1:3]
declaration = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?fn\s+([A-Za-z0-9_]+)\s*\(")
with open(path, encoding="utf-8") as handle:
    lines = handle.readlines()
start = None
for index, line in enumerate(lines, start=1):
    match = declaration.match(line)
    if match and match.group(1) == name:
        start = index
        break
if start is None:
    sys.exit(2)
end = len(lines) + 1
for index in range(start + 1, len(lines) + 1):
    if declaration.match(lines[index - 1]):
        end = index
        break
print(f"{start} {end}")
PY
}

read -r function_start function_end < <(declaration_range line_start_offset)

# Run Verus against the copied proof, from the real repository root so the
# pinned Verus installation resolves. Capture output for the attribution
# checks and return the runner's status.
run_verus() {
    local log=$1
    local proof_rel_to_repo
    proof_rel_to_repo="$(realpath --relative-to="$repo_root" "$copy/$proof_rel")"
    ( cd "$repo_root" && eval "$VERUS_RUN --proof-file \"$proof_rel_to_repo\"" ) >"$log" 2>&1
}

# Require the diagnostic for `needle` to point inside `line_start_offset`.
require_attribution() {
    local log=$1 needle=$2
    if ! grep -Fq "$needle" "$log"; then
        echo "mutation was not rejected by the expected obligation: $needle" >&2
        cat "$log" >&2
        return 1
    fi
    local reported
    reported="$(sed -n -E "s|^ *--> .*${kernel_rel//./\\.}:([0-9]+):[0-9]+$|\\1|p" "$log" | head -n 1)"
    if [[ -z "$reported" ]]; then
        echo "mutation failure names no source location in $kernel_rel" >&2
        cat "$log" >&2
        return 1
    fi
    if (( reported < function_start || reported >= function_end )); then
        echo "mutation failure is not attributed to line_start_offset:" >&2
        echo "  reported line $reported, expected $function_start..$((function_end - 1))" >&2
        cat "$log" >&2
        return 1
    fi
}

# Require a nonzero Verus exit with the runner's own rejection banner.
require_rejected() {
    local log=$1 status=$2
    if (( status == 0 )); then
        echo "mutation was accepted by the verifier" >&2
        cat "$log" >&2
        return 1
    fi
    if ! grep -Fq "Verus proofs failed" "$log"; then
        echo "mutation run did not reach the verifier" >&2
        cat "$log" >&2
        return 1
    fi
}

cr_old="while (index > 0 && !(bytes[index - 1] == b'\\n' || bytes[index - 1] == b'\\r'))"
cr_new="while (index > 0 && !(bytes[index - 1] == b'\\n'))"
return_old="            after { index }"
return_new="            after { index + 1 }"

echo "control: the unmutated copy verifies"
if ! run_verus "$copy/control.log"; then
    echo "the unmutated copy does not verify; the control is not meaningful" >&2
    cat "$copy/control.log" >&2
    exit 1
fi

echo "mutation: remove the CR test from the scan condition"
if ! mutate_once "$copied_kernel" "$cr_old" "$cr_new"; then
    echo "the CR mutation matched no unique source text" >&2
    exit 2
fi
cr_status=0
run_verus "$copy/cr.log" || cr_status=$?
require_rejected "$copy/cr.log" "$cr_status"
require_attribution "$copy/cr.log" "invariant not satisfied at end of loop body"
restore_kernel

echo "mutation: return an index past the line start"
if ! mutate_once "$copied_kernel" "$return_old" "$return_new"; then
    echo "the return mutation matched no unique source text" >&2
    exit 2
fi
return_status=0
run_verus "$copy/return.log" || return_status=$?
require_rejected "$copy/return.log" "$return_status"
require_attribution "$copy/return.log" "postcondition not satisfied"
restore_kernel

if [[ "$verus_only" == true ]]; then
    echo "source-span mutation control passed: the verifier rejects both defects"
    exit 0
fi

target_dir="${CARGO_TARGET_DIR:-$copy/target}"

# Run one selected regression in the copy and capture its output.
run_regression() {
    local log=$1
    shift
    ( cd "$copy" && "$cargo" test "$@" --target-dir "$target_dir" ) >"$log" 2>&1
}

unit_filter="roadmap::source_preservation::tests::task_source_stops_before_the_next_marker_indentation"
append_filter="delete_cr_only_task_does_not_preserve_the_document_prefix"

# The unmutated copy is the runtime control. Both regressions must execute and
# pass here first; otherwise a startup or compilation failure below could be
# mistaken for the mutation being detected.
echo "runtime control: both regressions pass on the unmutated copy"
if ! run_regression "$copy/unit-control.log" --lib "$unit_filter"; then
    echo "the unmutated copy fails the CR unit regression" >&2
    cat "$copy/unit-control.log" >&2
    exit 1
fi
if ! run_regression "$copy/append-control.log" --test roadmap_append_preservation "$append_filter"; then
    echo "the unmutated copy fails the CR-only append regression" >&2
    cat "$copy/append-control.log" >&2
    exit 1
fi

echo "runtime: the CR mutation breaks the preservation regressions"
if ! mutate_once "$copied_kernel" "$cr_old" "$cr_new"; then
    echo "the CR mutation matched no unique source text" >&2
    exit 2
fi

# A mutated run must fail because its selected test failed, not because Cargo
# could not start or the crate did not compile. Both markers are required: the
# test runner's failure line and the regression's own name.
require_test_failure() {
    local log=$1 regression=$2
    if [[ ! -f "$log" ]]; then
        echo "mutation run produced no log for $regression" >&2
        return 1
    fi
    if ! grep -Fq "test result: FAILED" "$log"; then
        echo "mutation run did not execute a failing test for $regression" >&2
        cat "$log" >&2
        return 1
    fi
    if ! grep -Fq "$regression" "$log"; then
        echo "mutation run did not name $regression as the failing test" >&2
        cat "$log" >&2
        return 1
    fi
}

unit_status=0
run_regression "$copy/unit.log" --lib "$unit_filter" || unit_status=$?
if (( unit_status == 0 )); then
    echo "the CR mutation did not break the CR unit regression" >&2
    cat "$copy/unit.log" >&2
    exit 1
fi
require_test_failure "$copy/unit.log" "task_source_stops_before_the_next_marker_indentation"

append_status=0
run_regression "$copy/append.log" --test roadmap_append_preservation "$append_filter" || append_status=$?
if (( append_status == 0 )); then
    echo "the CR mutation did not break the CR-only append regression" >&2
    cat "$copy/append.log" >&2
    exit 1
fi
require_test_failure "$copy/append.log" "delete_cr_only_task_does_not_preserve_the_document_prefix"

echo "source-span mutation control passed: verifier and runtime both reject the defect"