#!/usr/bin/env bash
# Verify that the roadmap domain reaches no infrastructure.
#
# The roadmap domain owns the Markdown model, the splice, and the rendering.
# Whatever reads a file, reads an environment variable, or walks the filesystem
# belongs to an adapter that calls the domain, never to the domain itself. That
# is what keeps the dependency direction inward: a domain module naming
# `std::fs` or `std::env` would have to change when the host changed, and any
# proof about it would be a proof about the host.
#
# The gate exists because the rule is otherwise invisible. Nothing in the
# compiler stops a `use std::env;` from landing in `src/roadmap`, and the
# temptation is concrete: the resolved kernel is spliced with a macro, and the
# obvious way to locate a file for `include!` is `env!("CARGO_MANIFEST_DIR")`.
# An earlier revision of this repository did exactly that, which is why the
# check is a gate rather than a note in a style guide.
#
# Three shapes are matched. A `use` declaration is line-anchored and its root is
# spelled out, so `use crate::fs;` is caught alongside `use std::fs;` — the
# point is the dependency on infrastructure, not on which root names it. A
# grouped import is matched by the same principle across the brace, so
# `use std::{fs as files};` is caught even though neither the bare name nor the
# alias used at the call site appears in the shape a plain `use` has. A module
# path or call is matched wherever it appears, so a path qualified in place
# rather than imported is caught too.
#
# Like the verification-ledger check, this is necessary rather than sufficient.
# Matching is textual, so a name surviving only in a doc comment triggers it: a
# comment discussing an escape hatch is treated as an escape hatch. That is the
# conservative direction, and it is visible in review rather than silent.
# String literals are not masked either, so prose in a message would count.
# What the check does guarantee is that a genuine `use std::process;` cannot
# land in a production domain module without the gate reporting it.
#
# Test files are exempt by name, because `src/roadmap/render_tests.rs` drives
# the rendering path through the built binary and legitimately names
# `std::process`. The exemption is by file rather than by region so that no
# brace count has to be trusted; a file is test-only or it is production, with
# nothing in between for a scan to get wrong.
#
# Usage: check-domain-purity.sh [SCAN_DIR]
#
# Exit status:
#   0  no roadmap source reaches infrastructure
#   1  a roadmap source names infrastructure, or no source was scanned
#   *  a file, tool, or ripgrep failure is propagated
set -euo pipefail

read -r -a rg_cmd <<<"${RG:-rg}"
scan_dir="${1:-.}"
domain_dir="${scan_dir}/src/roadmap"

if [[ ! -d "${domain_dir}" ]]; then
    echo "roadmap domain directory is missing: ${domain_dir}" >&2
    exit 1
fi

# Infrastructure a domain module must not name: the crates that reach outside
# the process, and the build-time functions that read the build environment.
infrastructure='fs|path|io|process|env|net|thread'

visibility='(pub(\([^)]*\))?[[:space:]]+)?'
declaration="^[[:space:]]*${visibility}use[[:space:]]+(::)?((crate|std|alloc|core|self|super)::)*(${infrastructure})(::|[[:space:];]|$)"
# A grouped import names its paths inside braces, so the root is not adjacent to
# the `use`: `use std::{fs as files};` hides `fs` behind a brace and an alias,
# and the call site then reads `files::read(..)`, which `qualified` cannot see
# either. The leading group is optional so that the first brace member matches
# as `use std::{fs}`, and the trailing delimiter admits `,` for a member that is
# followed by others. Neither needs the alias to be recognised: importing the
# infrastructure at all is the violation, whatever the local name becomes.
grouped="^[[:space:]]*${visibility}use[[:space:]]+(::)?((crate|std|alloc|core|self|super)::)*[^;]*\{([^}]*[^[:alnum:]_])?(${infrastructure})([[:space:]]+as[[:space:]]+[[:alnum:]_]+)?[[:space:]]*[,}]"
qualified="(^|[^[:alnum:]_])(std::|crate::|self::|super::)?(${infrastructure})::"
build_time='include_str!|include_bytes!|option_env!|(^|[^[:alnum:]_])env!'
process_calls='(^|[^[:alnum:]_])(current_dir|current_exe|std::env::args)[[:space:]]*\('

pattern="${declaration}|${grouped}|${qualified}|${build_time}|${process_calls}"

production_scanned="$("${rg_cmd[@]}" --files \
    --glob '*.rs' --glob '!*_tests.rs' "${domain_dir}" | wc -l | tr -d '[:space:]')"
if [[ "${production_scanned}" -eq 0 ]]; then
    echo "roadmap domain states no production sources to scan: ${domain_dir}" >&2
    exit 1
fi

status=0
findings="$("${rg_cmd[@]}" --line-number --no-heading --color never \
    --glob '*.rs' --glob '!*_tests.rs' \
    -e "${pattern}" "${domain_dir}")" || status=$?
case "${status}" in
    0)
        printf '%s\n' "${findings}" >&2
        echo "roadmap domain must not reach infrastructure; move the access to an adapter" >&2
        exit 1
        ;;
    1) ;;
    *)
        echo "failed to scan the roadmap domain for infrastructure (rg exit ${status})" >&2
        exit "${status}"
        ;;
esac
