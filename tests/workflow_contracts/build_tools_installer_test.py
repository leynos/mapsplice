"""Controlled installation tests for the pinned Linux linker pair."""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
VERSION = "2.41.0"
DIGEST = "a3696680d99e692970590a178bc3a33d78d60d1c6dc9db7a11b557b02b751f5d"


def _write_command(directory: Path, name: str, contents: str) -> None:
    command = directory / name
    command.write_text(contents, encoding="utf-8")
    command.chmod(0o755)


@pytest.mark.parametrize(
    ("companion_version", "expected_status", "marker_created"),
    [
        ("mold 2.41.0 (test)", 0, True),
        ("mold 2.40.4 (test)", 1, False),
    ],
    ids=["pinned-companion", "wrong-companion-version"],
)
def test_installer_checks_ld_mold_before_writing_success_marker(
    tmp_path: Path,
    companion_version: str,
    expected_status: int,
    marker_created: bool,
) -> None:
    """Fresh installation cannot mark an invalid linker pair as complete."""
    commands = tmp_path / "commands"
    commands.mkdir()
    for name in ("awk", "install", "mkdir", "mktemp", "rm", "touch"):
        resolved = shutil.which(name)
        assert resolved is not None, f"test host must provide {name}"
        (commands / name).symlink_to(resolved)

    _write_command(commands, "rustup", "#!/bin/sh\nexit 0\n")
    _write_command(
        commands,
        "uname",
        "#!/bin/sh\ncase \"$1\" in\n"
        "  -s) printf '%s\\n' Linux ;;\n"
        "  -m) printf '%s\\n' x86_64 ;;\n"
        "  *) exit 1 ;;\nesac\n",
    )
    _write_command(
        commands,
        "curl",
        "#!/bin/sh\nwhile [ \"$#\" -gt 0 ]; do\n"
        "  if [ \"$1\" = --output ]; then archive=$2; shift 2; else shift; fi\n"
        "done\nprintf '%s\\n' fixture > \"$archive\"\n",
    )
    _write_command(commands, "sha256sum", "#!/bin/sh\nexit 0\n")
    _write_command(
        commands,
        "tar",
        "#!/bin/sh\ndestination=\nwhile [ \"$#\" -gt 0 ]; do\n"
        "  if [ \"$1\" = -C ]; then destination=$2; shift 2; else shift; fi\n"
        "done\n"
        f"directory=\"$destination/mold-{VERSION}-x86_64-linux/bin\"\n"
        "mkdir -p \"$directory\"\n"
        f"printf '%s\\n' '#!/bin/sh' \"printf '%s\\\\n' 'mold {VERSION} (test)'\" "
        "> \"$directory/mold\"\n"
        f"printf '%s\\n' '#!/bin/sh' \"printf '%s\\\\n' '{companion_version}'\" "
        "> \"$directory/ld.mold\"\n",
    )

    prefix = tmp_path / "installed-tools"
    scratch = tmp_path / "scratch"
    scratch.mkdir()
    environment = {
        "PATH": str(commands),
        "HOME": str(tmp_path / "home"),
        "BUILD_TOOLS_PREFIX": str(prefix),
        "TMPDIR": str(scratch),
    }
    result = subprocess.run(
        ["/bin/bash", "scripts/install-build-tools.sh"],
        cwd=ROOT,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )

    marker = prefix / "bin" / f".mold-{VERSION}-{DIGEST}"
    assert result.returncode == expected_status, result.stderr
    assert marker.exists() is marker_created
    if not marker_created:
        assert "Installed ld.mold does not report the pinned version" in result.stderr
