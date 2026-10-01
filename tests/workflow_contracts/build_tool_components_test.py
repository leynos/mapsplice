"""Contracts for local Rust toolchain provisioning without Cranelift."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

import pytest

from build_standard_test import ROOT

REQUIRED_RUST_COMPONENTS = ("rustfmt", "clippy", "rust-analyzer", "llvm-tools")


def mocked_build_tool_environment(
    tmp_path: Path, components: tuple[str, ...],
) -> dict[str, str]:
    """Provide Rust command stubs and skip platform linker checks."""
    commands = tmp_path / "commands"
    commands.mkdir()
    stubs = {
        "rustc": "exit 0\n",
        "rustup": (
            'if [ "$1" = component ]; then printf \'%s\\n\' "$MOCK_COMPONENTS"; '
            'else printf \'%s\\n\' "$@" > "$RUSTUP_ARGV_LOG"; fi\n'
        ),
        "uname": "echo Darwin\n",
    }
    for name, body in stubs.items():
        executable = commands / name
        executable.write_text(f"#!/bin/sh\n{body}", encoding="utf-8")
        executable.chmod(0o755)
    return {
        **os.environ,
        "MOCK_COMPONENTS": "\n".join(f"{component}-test" for component in components),
        "RUSTUP_ARGV_LOG": str(tmp_path / "rustup-argv"),
        "PATH": f"{commands}:{os.environ['PATH']}",
    }


@pytest.mark.parametrize(
    ("installed", "missing"),
    [
        pytest.param(REQUIRED_RUST_COMPONENTS, None, id="complete-without-cranelift"),
        *[
            pytest.param(
                tuple(component for component in REQUIRED_RUST_COMPONENTS if component != missing),
                missing,
                id=f"missing-{missing}",
            )
            for missing in REQUIRED_RUST_COMPONENTS
        ],
    ],
)
def test_build_tool_preflight_requires_each_component(
    tmp_path: Path, installed: tuple[str, ...], missing: str | None,
) -> None:
    """The required toolchain passes without Cranelift; each missing tool fails."""
    result = subprocess.run(
        ["/bin/bash", "scripts/check-build-tools.sh"],
        cwd=ROOT,
        env=mocked_build_tool_environment(tmp_path, installed),
        capture_output=True,
        text=True,
        check=False,
    )
    assert (result.returncode == 0) == (missing is None), result.stderr
    if missing is not None:
        assert f"Build prerequisite missing: {missing}" in result.stderr


def test_build_tool_installer_requests_required_components_without_cranelift(
    tmp_path: Path,
) -> None:
    """The pinned nightly install requests maintenance tools and LLVM tooling only."""
    environment = mocked_build_tool_environment(tmp_path, ())
    result = subprocess.run(
        ["/bin/bash", "scripts/install-build-tools.sh"],
        cwd=ROOT,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    arguments = Path(environment["RUSTUP_ARGV_LOG"]).read_text(encoding="utf-8").splitlines()
    components = [
        arguments[index + 1]
        for index, argument in enumerate(arguments[:-1])
        if argument == "--component"
    ]
    assert components == ["rustfmt", "clippy", "rust-analyzer", "llvm-tools-preview"]
    assert "rustc-codegen-cranelift-preview" not in components
