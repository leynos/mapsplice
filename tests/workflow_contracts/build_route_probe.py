"""Record Make's compiler routes using fake Cargo and Whitaker executables."""

from __future__ import annotations

import os
import subprocess
from dataclasses import dataclass
from pathlib import Path

from build_standard_test import ROOT

INHERITED_FLAGS = [
    "--cfg",
    "caller_policy",
    "-Z",
    "threads=8",
    "-C",
    "link-arg=-fuse-ld=mold",
    "-Z",
    "codegen-backend=cranelift",
]
COMPATIBLE_FLAGS = ["--cfg", "caller_policy"]


@dataclass(frozen=True)
class RouteInvocation:
    """One executable invocation and the compiler environment Make selected."""

    arguments: tuple[str, ...]
    encoded_flags: tuple[str, ...]
    rust_flags: tuple[str, ...]
    rustdoc_flags: tuple[str, ...]
    release_backend: str
    dev_backend: str
    test_backend: str


def execute_make_route(
    tmp_path: Path,
    goal: str,
    *,
    variables: dict[str, str] | None = None,
    environment: dict[str, str] | None = None,
) -> list[RouteInvocation]:
    """Run one Make goal against recording executables instead of Cargo."""
    commands = tmp_path / "commands"
    commands.mkdir()
    probe = commands / "probe"
    probe.write_text(
        "#!/bin/sh\nprintf '%s\\035%s\\035%s\\035%s\\035%s\\035%s\\035%s\\036' "
        '"$*" "${CARGO_ENCODED_RUSTFLAGS-}" "${RUSTFLAGS-}" '
        '"${RUSTDOCFLAGS-}" "${CARGO_PROFILE_RELEASE_CODEGEN_BACKEND-}" '
        '"${CARGO_PROFILE_DEV_CODEGEN_BACKEND-}" '
        '"${CARGO_PROFILE_TEST_CODEGEN_BACKEND-}" >> "$ROUTE_PROBE"\n',
        encoding="utf-8",
    )
    probe.chmod(0o755)
    # The configured Make wrapper uses env bash, so only bypass preflight.
    shell = commands / "bash"
    shell.write_text(
        "#!/bin/sh\n"
        "if [ \"$1\" = 'scripts/check-build-tools.sh' ]; then exit 0; fi\n"
        'exec /bin/bash "$@"\n',
        encoding="utf-8",
    )
    shell.chmod(0o755)
    output = tmp_path / "route"
    cargo_home = tmp_path / "cargo-home"
    cargo_home.mkdir()
    arguments = [
        "make",
        "--no-print-directory",
        "--always-make",
        goal,
        f"CARGO={probe}",
        f"WHITAKER={probe}",
        "DOC_TEST_TARGETS=false",
    ]
    arguments.extend(f"{key}={value}" for key, value in (variables or {}).items())
    command_environment = {
        **os.environ,
        "CARGO_ENCODED_RUSTFLAGS": "\x1f".join(INHERITED_FLAGS),
        "CARGO_HOME": str(cargo_home),
        "PATH": f"{commands}:{os.environ['PATH']}",
        "ROUTE_PROBE": str(output),
        "RUST_FLAGS": "",
        "RUSTDOC_FLAGS": "",
    }
    command_environment.pop("CARGO_BUILD_TARGET", None)
    command_environment.update(environment or {})
    # An outer Make may export dry-run flags; this child must execute its probes.
    for make_flags in ("MAKEFLAGS", "GNUMAKEFLAGS", "MFLAGS"):
        command_environment.pop(make_flags, None)
    subprocess.run(arguments, cwd=ROOT, env=command_environment, check=True)
    return parse_invocations(output.read_text(encoding="utf-8"))


def parse_invocations(recording: str) -> list[RouteInvocation]:
    """Decode the append-only probe recording without losing encoded flags."""
    invocations = []
    for record in recording.split("\x1e"):
        if not record:
            continue
        fields = record.split("\x1d")
        assert len(fields) == 7, f"malformed route probe record: {record!r}"
        invocations.append(
            RouteInvocation(
                arguments=tuple(fields[0].split()),
                encoded_flags=tuple(flag for flag in fields[1].split("\x1f") if flag),
                rust_flags=tuple(fields[2].split()),
                rustdoc_flags=tuple(fields[3].split()),
                release_backend=fields[4],
                dev_backend=fields[5],
                test_backend=fields[6],
            )
        )
    assert invocations, "Make did not execute a route probe"
    return invocations


def route_for(
    invocations: list[RouteInvocation],
    expected: tuple[str, ...],
) -> RouteInvocation:
    """Return exactly one recorded command with the requested leading words."""
    matches = [
        invocation
        for invocation in invocations
        if invocation.arguments[: len(expected)] == expected
    ]
    assert len(matches) == 1, (
        f"expected one {' '.join(expected)} route, got {matches!r}"
    )
    return matches[0]


def assert_development_route(
    invocation: RouteInvocation, *, uses_pinned_linker: bool
) -> None:
    """Require one evaluated development route to restore its selected defaults."""
    expected = [*COMPATIBLE_FLAGS, "-D", "warnings", "-Zthreads=8"]
    rust_flags = ["-D", "warnings", "-Zthreads=8"]
    if uses_pinned_linker:
        expected.extend(["-C", "link-arg=-fuse-ld=mold"])
        rust_flags.extend(["-C", "link-arg=-fuse-ld=mold"])
    assert list(invocation.encoded_flags) == expected
    assert list(invocation.rust_flags) == rust_flags
    assert (
        invocation.release_backend
        == invocation.dev_backend
        == invocation.test_backend
        == ""
    )


def assert_whitaker_route(invocation: RouteInvocation) -> None:
    """Require the installer-managed verification route to remain LLVM-only."""
    assert invocation.arguments == (
        "--all",
        "--package",
        "mapsplice",
        "--",
        "--workspace",
        "--all-targets",
        "--all-features",
    )
    assert invocation.encoded_flags == (*COMPATIBLE_FLAGS, "-D", "warnings")
    assert invocation.rust_flags == ("-D", "warnings")
    assert invocation.rustdoc_flags == ()
    assert invocation.release_backend == ""
    assert invocation.dev_backend == invocation.test_backend == "llvm"


def cargo_home_with_target(tmp_path: Path, target: str) -> dict[str, str]:
    """Create a CARGO_HOME whose only target selection is the named triple."""
    cargo_home = tmp_path / "configured-cargo-home"
    cargo_home.mkdir()
    (cargo_home / "config.toml").write_text(
        f'[build]\ntarget = "{target}"\n',
        encoding="utf-8",
    )
    return {"CARGO_HOME": str(cargo_home)}
