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
# Four shapes are matched, and the import shape is deliberately structural
# rather than a model of Rust's use-tree grammar. An earlier revision modelled
# that grammar — one alternative for a plain declaration, another for a brace
# group — and the model was defeated three times by forms it did not enumerate:
# first `use std::{fs};`, then `use std::{fs as files};` (the alias hid the call
# site from the module-path alternative too), and then
# `use std::{fmt::{self, Display}, fs as files};`, where an *inner* brace made
# the group alternative's single `\{[^}]*` run stop before `fs`. Each fix added
# an alternative and each was defeated again, because every brace group is a
# fresh opportunity to nest.
#
# So the import shape no longer models braces at all. It asks only whether the
# infrastructure name appears in a `use` statement as a whole path segment, and
# whether that statement is still open — the name is guarded on both sides
# (`(^|[^alnum_])` before, `(::|[,;}]|$)` after) so a longer identifier such as
# `fs_helpers`, `paths`, `processes`, or `env_vars` is not a segment and does
# not match. Nesting depth, aliasing, member order, and the root spelling all
# stop mattering, because none of them is represented. The shape is also
# root-agnostic, so `use crate::fs;` is caught alongside `use std::fs;`: the
# dependency is on infrastructure, not on which root names it.
#
# Matching is not anchored to the crate root, so `use foo::fs;` also matches. That
# is the same breadth the `qualified` alternative already has, where
# `crate::roadmap::path::Route` matches on the `path` segment without knowing
# where the road map keeps its own modules. The gate is therefore name-based
# rather than resolution-based, which is the conservative direction and the
# price of not modelling: `src/roadmap` defines no module named after an
# infrastructure crate today, so nothing legitimate is caught today, but a
# future roadmap module that shadowed one of those names would be. That is
# recorded as a known limit rather than left implicit.
#
# A module path or call is matched wherever it appears, so a path qualified in
# place rather than imported is caught too.
#
# The scan runs under `--multiline`. A real `use` may span lines —
# `src/roadmap/render.rs` has one — so a line-at-a-time scan cannot see the
# terminator that separates a member list from the next statement, and would
# miss the multi-line spelling of every shape above. `^` then anchors to the
# start of a line rather than of the file, which is the intent.
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
# The infrastructure name as a whole path segment anywhere inside an open `use`
# statement. The leading optional group consumes any prefix without reaching the
# name's own leading character, so the preceding guard sees a real boundary; the
# optional alias absorbs `fs as files`; the trailing group is the terminator,
# either a further segment (`fs::read`) or the end of the statement, where `$`
# covers a `use` whose terminator is not on the same line as the name.
import="^[[:space:]]*${visibility}use([^;]*[^[:alnum:]_])?(${infrastructure})([[:space:]]+as[[:space:]]+[[:alnum:]_]+)?[[:space:]]*(::([[:space:]]*[,;}])?|[,;}]|\$)"
qualified="(^|[^[:alnum:]_])(std::|crate::|self::|super::)?(${infrastructure})::"
build_time='include_str!|include_bytes!|option_env!|(^|[^[:alnum:]_])env!'
process_calls='(^|[^[:alnum:]_])(current_dir|current_exe|std::env::args)[[:space:]]*\('

pattern="${import}|${qualified}|${build_time}|${process_calls}"

production_scanned="$("${rg_cmd[@]}" --files \
    --glob '*.rs' --glob '!*_tests.rs' "${domain_dir}" | wc -l | tr -d '[:space:]')"
if [[ "${production_scanned}" -eq 0 ]]; then
    echo "roadmap domain states no production sources to scan: ${domain_dir}" >&2
    exit 1
fi

status=0
# `-U --multiline` lets the import shape see a `use` statement that spans lines.
# The scan is captured before any formatting so that the exit status read below
# is ripgrep's own, rather than whichever command a pipeline happened to end
# with.
matched="$("${rg_cmd[@]}" -U --multiline --line-number --no-heading --color never \
    --glob '*.rs' --glob '!*_tests.rs' \
    -e "${pattern}" "${domain_dir}")" || status=$?

# A multiline match may itself span lines, and ripgrep then prints only the
# first, so the remainder is trimmed rather than left as a half-printed
# fragment. A file and a line number are all a reader needs to open the finding.
findings="$(printf '%s' "${matched}" | sed -E 's/^([^:]+:[0-9]+):.*/\1/')"
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
