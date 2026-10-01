"""Exercise evaluated Make rustdoc routes and their failure contract."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

import pytest

from build_standard_routing_test import (
    ROOT,
    RouteInvocation,
    execute_make_route,
    parse_invocations,
    route_for,
)


REQUIRED_RUSTDOC_FLAGS = ("--cfg", "docsrs", "-D", "warnings")


def assert_rustdoc_route(
    route: RouteInvocation,
    expected_arguments: tuple[str, ...],
    extra_flags: tuple[str, ...] = (),
) -> None:
    """Require the workspace rustdoc invocation and ordered caller flags."""
    assert route.arguments == expected_arguments
    assert route.rustdoc_flags == (*REQUIRED_RUSTDOC_FLAGS, *extra_flags)


@pytest.mark.parametrize(
    ("goal", "expected_command", "expected_arguments"),
    [
        ("lint-clippy", ("doc",), ("doc", "--workspace", "--no-deps")),
        (
            "test",
            ("test", "--doc"),
            ("test", "--doc", "--workspace", "--all-features"),
        ),
    ],
)
@pytest.mark.parametrize(
    ("caller_flags", "extra_flags"),
    [
        (None, ()),
        ("--cfg caller_docs -Z unstable-options", (
            "--cfg", "caller_docs", "-Z", "unstable-options",
        )),
    ],
)
def test_rustdoc_routes_apply_required_flags_before_caller_flags(
    tmp_path: Path,
    goal: str,
    expected_command: tuple[str, ...],
    expected_arguments: tuple[str, ...],
    caller_flags: str | None,
    extra_flags: tuple[str, ...],
) -> None:
    """Docs and doctests keep required rustdoc policy and caller additions."""
    variables = {"RUSTDOC_FLAGS": caller_flags} if caller_flags else {}
    if goal == "test":
        variables.update({"DOC_TEST_TARGETS": "true", "TEST_CMD": "test"})
    invocations = execute_make_route(tmp_path, goal, variables=variables)
    assert_rustdoc_route(
        route_for(invocations, expected_command), expected_arguments, extra_flags,
    )


def makefile_route(
    tmp_path: Path,
    makefile: Path,
    *,
    fail_documentation: bool = False,
) -> tuple[subprocess.CompletedProcess[str], list[RouteInvocation]]:
    """Evaluate a copied Makefile against a recording Cargo executable."""
    commands = tmp_path / "commands"
    commands.mkdir()
    probe = commands / "probe"
    probe.write_text(
        "#!/bin/sh\n"
        "printf '%s\\035%s\\035%s\\035%s\\035%s\\035%s\\035%s\\036' "
        '"$*" "${CARGO_ENCODED_RUSTFLAGS-}" "${RUSTFLAGS-}" '
        '"${RUSTDOCFLAGS-}" "${CARGO_PROFILE_RELEASE_CODEGEN_BACKEND-}" '
        '"${CARGO_PROFILE_DEV_CODEGEN_BACKEND-}" '
        '"${CARGO_PROFILE_TEST_CODEGEN_BACKEND-}" >> "$ROUTE_PROBE"\n'
        + ("if [ \"$1\" = 'doc' ]; then exit 23; fi\n" if fail_documentation else ""),
        encoding="utf-8",
    )
    probe.chmod(0o755)
    shell = commands / "bash"
    shell.write_text(
        "#!/bin/sh\n"
        "if [ \"$1\" = 'scripts/check-build-tools.sh' ]; then exit 0; fi\n"
        "exec /bin/bash \"$@\"\n",
        encoding="utf-8",
    )
    shell.chmod(0o755)
    output = tmp_path / "route"
    cargo_home = tmp_path / "cargo-home"
    cargo_home.mkdir()
    environment = {
        **os.environ,
        "CARGO_ENCODED_RUSTFLAGS": "",
        "CARGO_HOME": str(cargo_home),
        "PATH": f"{commands}:{os.environ['PATH']}",
        "ROUTE_PROBE": str(output),
        "RUST_FLAGS": "",
        "RUSTDOC_FLAGS": "",
    }
    environment.pop("CARGO_BUILD_TARGET", None)
    for make_flags in ("MAKEFLAGS", "GNUMAKEFLAGS", "MFLAGS"):
        environment.pop(make_flags, None)
    result = subprocess.run(
        [
            "make", "--no-print-directory", "--always-make", "-f", str(makefile),
            "lint-clippy", f"CARGO={probe}", "DOC_TEST_TARGETS=false",
        ],
        cwd=ROOT,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )
    return result, parse_invocations(output.read_text(encoding="utf-8"))


def test_rustdoc_command_failure_propagates_through_make(tmp_path: Path) -> None:
    """A failing Cargo documentation command stops the lint-clippy gate."""
    makefile = tmp_path / "Makefile"
    makefile.write_text((ROOT / "Makefile").read_text(encoding="utf-8"), encoding="utf-8")
    result, invocations = makefile_route(tmp_path, makefile, fail_documentation=True)
    assert result.returncode != 0
    assert "Error 23" in result.stderr
    assert route_for(invocations, ("doc",)).arguments == (
        "doc", "--workspace", "--no-deps",
    )
    assert not any(invocation.arguments[0] == "clippy" for invocation in invocations)


@pytest.mark.parametrize(
    ("mutation", "expected_arguments"),
    [
        ("docsrs", ("doc", "--workspace", "--no-deps")),
        ("workspace", ("doc", "--workspace", "--no-deps")),
    ],
)
def test_rustdoc_contract_rejects_required_policy_mutations(
    tmp_path: Path,
    mutation: str,
    expected_arguments: tuple[str, ...],
) -> None:
    """Missing docsrs defaults or workspace scope fail the evaluated check."""
    makefile = tmp_path / "Makefile"
    source = (ROOT / "Makefile").read_text(encoding="utf-8")
    if mutation == "docsrs":
        mutated = source.replace("--cfg docsrs -D warnings", "-D warnings", 1)
    else:
        mutated = source.replace(
            "$(CARGO) doc --workspace --no-deps", "$(CARGO) doc --no-deps", 1,
        )
    assert mutated != source, f"{mutation} mutation did not alter Makefile"
    makefile.write_text(mutated, encoding="utf-8")
    result, invocations = makefile_route(tmp_path, makefile)
    assert result.returncode == 0, result.stderr
    documentation = route_for(invocations, ("doc",))
    with pytest.raises(AssertionError):
        assert_rustdoc_route(documentation, expected_arguments)
