"""Verify malformed Cargo config diagnostics at process boundaries."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

import pytest
from build_standard_test import ROOT

RESOLVER = ROOT / "scripts" / "resolve-cargo-build-target.py"


@pytest.mark.parametrize("boundary", ("resolver", "make"))
def test_invalid_utf8_configuration_is_reported_without_traceback(
    tmp_path: Path,
    boundary: str,
) -> None:
    """The resolver and Make surface controlled errors for undecodable config."""
    cargo_home = tmp_path / "cargo-home"
    cargo_home.mkdir()
    config = cargo_home / "config.toml"
    config.write_bytes(b"\xff")
    command = (
        [sys.executable, str(RESOLVER)]
        if boundary == "resolver"
        else ["make", "--dry-run", "check-fmt", "CARGO=echo"]
    )
    cwd = tmp_path if boundary == "resolver" else ROOT
    result = subprocess.run(
        command,
        cwd=cwd,
        env={
            **os.environ,
            "CARGO_HOME": str(cargo_home),
            "HOME": str(tmp_path / "home"),
            "LC_ALL": "C",
            "PYTHONDONTWRITEBYTECODE": "1",
        },
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode != 0, f"{boundary} accepted invalid UTF-8 config"
    assert "cannot read Cargo configuration" in result.stderr, (
        f"{boundary} stderr was {result.stderr!r}"
    )
    assert str(config) in result.stderr, f"{boundary} omitted the config path"
    assert "Traceback" not in result.stderr, (
        f"{boundary} exposed a Python traceback: {result.stderr!r}"
    )
