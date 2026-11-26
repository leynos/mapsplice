#!/usr/bin/env bash
set -euo pipefail
if [[ -n "${FAKE_MUTATION_LOG:-}" ]]; then
    printf '%s\n' "$*" >> "$FAKE_MUTATION_LOG"
    printf 'VERUS_Z3_PATH=%s\n' "${VERUS_Z3_PATH:-}" >> "$FAKE_MUTATION_LOG"
fi
proof=''
expects_file=false
for argument in "$@"; do
    if [[ "$expects_file" == true ]]; then
        proof=$argument
        expects_file=false
    elif [[ "$argument" == "--proof-file" ]]; then
        expects_file=true
    fi
done
if [[ ! -f "$proof" ]]; then
    echo "fake runner received no proof file" >&2
    exit 3
fi
root=$(dirname "$(dirname "$proof")")
kernel="$root/src/roadmap/source_span_kernel.macro.rs"
if [[ ! -f "$kernel" ]]; then
    echo "fake runner found no kernel at $kernel" >&2
    exit 3
fi
line=$(grep -n -m1 'fn line_start_offset(' "$kernel" | cut -d: -f1)
mode="${FAKE_MUTATION_MODE:?}"
if grep -Fq "|| bytes[index - 1] == b'\r'" "$kernel"; then
    if [[ "$mode" == "faithful" ]] && grep -Fq "after { index + 1 }" "$kernel"; then
        echo "error: postcondition not satisfied"
        echo "  --> $kernel:$line:17"
        echo "Verus proofs failed (exit 1)."
        exit 1
    fi
    echo "verification results:: 21 verified, 0 errors"
    exit 0
fi
case "$mode" in
    accepted)
        echo "verification results:: 21 verified, 0 errors"
        exit 0
        ;;
    unattributed)
        echo "error: invariant not satisfied at end of loop body"
        echo "Verus proofs failed (exit 1)."
        exit 1
        ;;
    startup_failure)
        echo "runner unavailable"
        exit 1
        ;;
    faithful)
        echo "error: invariant not satisfied at end of loop body"
        echo "  --> $kernel:$line:17"
        echo "Verus proofs failed (exit 1)."
        exit 1
        ;;
esac
