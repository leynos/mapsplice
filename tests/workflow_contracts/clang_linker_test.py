"""Check the selected Clang target and the binary preflight actually validates."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

import pytest
from build_standard_test import ROOT


def _executable(path: Path, body: str) -> None:
    path.write_text(f"#!/bin/sh\n{body}", encoding="utf-8")
    path.chmod(0o755)


@pytest.mark.parametrize("target", ["native", "aarch64"])
@pytest.mark.parametrize("uses_pinned_linker", [False, True])
def test_clang_route_forwards_target_and_linker(
    tmp_path: Path,
    target: str,
    uses_pinned_linker: bool,
) -> None:
    """The AArch64 route always passes its triple through the shared wrapper."""
    commands = tmp_path / "commands"
    commands.mkdir()
    _executable(
        commands / "clang",
        'printf \'%s\\n\' "$@" > "$CLANG_ARGUMENTS"\n',
    )
    _executable(commands / "ld.mold", "echo 'mold 2.41.0 (test)'\n")
    wrapper = "clang-aarch64-linker.sh" if target == "aarch64" else "clang-linker.sh"
    arguments = ["-o", "out"]
    if uses_pinned_linker:
        arguments.append("-fuse-ld=mold")
    recording = tmp_path / "clang-arguments"
    result = subprocess.run(
        [str(ROOT / "scripts" / wrapper), *arguments],
        cwd=ROOT,
        env={
            **os.environ,
            "PATH": f"{commands}:{os.environ['PATH']}",
            "CLANG_ARGUMENTS": str(recording),
        },
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    observed = recording.read_text(encoding="utf-8").splitlines()
    expected = ["-B", str(commands)] if uses_pinned_linker else []
    if target == "aarch64":
        expected.append("--target=aarch64-unknown-linux-gnu")
    expected.extend(arguments)
    assert observed == expected


@pytest.mark.parametrize("linker_version", [None, "mold 2.40.4", "mold 2.41.0 (test)"])
def test_preflight_checks_linker_even_when_driver_is_correct(
    tmp_path: Path,
    linker_version: str | None,
) -> None:
    """An absent or stale linker companion cannot pass the preflight."""
    commands = tmp_path / "commands"
    commands.mkdir()
    _executable(commands / "rustc", "exit 0\n")
    _executable(
        commands / "rustup",
        "printf '%s\\n' rustfmt-test clippy-test rust-analyzer-test llvm-tools-test\n",
    )
    _executable(
        commands / "uname",
        'if [ "$1" = -s ]; then echo Linux; else echo x86_64; fi\n',
    )
    _executable(commands / "clang", "exit 0\n")
    _executable(commands / "mold", "echo 'mold 2.41.0 (test)'\n")
    (commands / "grep").symlink_to("/usr/bin/grep")
    if linker_version is not None:
        _executable(commands / "ld.mold", f"echo '{linker_version}'\n")
    result = subprocess.run(
        ["/bin/bash", "scripts/check-build-tools.sh"],
        cwd=ROOT,
        env={"PATH": str(commands)},
        capture_output=True,
        text=True,
        check=False,
    )
    if linker_version == "mold 2.41.0 (test)":
        assert result.returncode == 0, result.stderr
    else:
        assert result.returncode != 0
        assert "Build prerequisite missing: ld.mold 2.41.0" in result.stderr
        assert "make install-build-tools" in result.stderr
