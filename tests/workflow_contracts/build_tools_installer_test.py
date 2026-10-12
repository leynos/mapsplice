"""Controlled installation tests for the pinned Linux linker pair."""

from __future__ import annotations

import shlex
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
VERSION = "2.41.0"
DIGESTS = {
    "x86_64": "a3696680d99e692970590a178bc3a33d78d60d1c6dc9db7a11b557b02b751f5d",
    "aarch64": "946de2774b06a71346bd59b55fddba610b65b8d93c3a4a1559cc84e103472710",
}


@dataclass(frozen=True)
class InstallerCase:
    architecture: str
    companion_version: str
    checksum_matches: bool
    expected_status: int


@dataclass(frozen=True)
class InstallerRun:
    result: subprocess.CompletedProcess[str]
    archive: str
    marker: Path
    curl_url_log: Path
    checksum_log: Path
    tar_marker: Path
    prefix: Path


def _write_command(directory: Path, name: str, contents: str) -> None:
    command = directory / name
    command.write_text(contents, encoding="utf-8")
    command.chmod(0o755)


def _write_platform_tools(commands: Path, case: InstallerCase) -> None:
    _write_command(commands, "rustup", "#!/bin/sh\nexit 0\n")
    _write_command(
        commands,
        "uname",
        "#!/bin/sh\ncase \"$1\" in\n"
        "  -s) printf '%s\\n' Linux ;;\n"
        f"  -m) printf '%s\\n' {case.architecture} ;;\n"
        "  *) exit 1 ;;\n"
        "esac\n",
    )


def _write_download_tool(commands: Path) -> None:
    _write_command(
        commands,
        "curl",
        "#!/bin/sh\nurl=\nwhile [ \"$#\" -gt 0 ]; do\n"
        "  case $1 in\n"
        "    --output) archive=$2; shift 2 ;;\n"
        "    https://*) url=$1; shift ;;\n"
        "    *) shift ;;\n"
        "  esac\n"
        "done\n"
        "printf '%s\\n' \"$url\" > \"$CURL_URL_LOG\"\n"
        "printf '%s\\n' fixture > \"$archive\"\n",
    )


def _write_checksum_tool(commands: Path) -> None:
    real_sha256sum = shutil.which("sha256sum")
    assert real_sha256sum is not None, "test host must provide sha256sum"
    _write_command(
        commands,
        "sha256sum",
        "#!/bin/sh\nset -eu\n"
        "read -r digest archive\n"
        "printf '%s\\n%s\\n' \"$digest\" \"${archive##*/}\" > \"$CHECKSUM_LOG\"\n"
        "[ \"$digest\" = \"$EXPECTED_DIGEST\" ] || exit 1\n"
        "[ \"${archive##*/}\" = \"$EXPECTED_ARCHIVE\" ] || exit 1\n"
        "if [ \"$CHECKSUM_MODE\" = mismatch ]; then\n"
        f"  printf '%s  %s\\n' \"$digest\" \"$archive\" | "
        f"{shlex.quote(real_sha256sum)} --check --status\n"
        "fi\n",
    )


def _write_archive_tool(commands: Path, case: InstallerCase) -> None:
    _write_command(
        commands,
        "tar",
        "#!/bin/sh\nprintf '%s\\n' called > \"$TAR_MARKER\"\n"
        "destination=\nwhile [ \"$#\" -gt 0 ]; do\n"
        "  if [ \"$1\" = -C ]; then destination=$2; shift 2; else shift; fi\n"
        "done\n"
        f"directory=\"$destination/mold-{VERSION}-{case.architecture}-linux/bin\"\n"
        "mkdir -p \"$directory\"\n"
        f"printf '%s\\n' '#!/bin/sh' \"printf '%s\\\\n' 'mold {VERSION} (test)'\" "
        "> \"$directory/mold\"\n"
        f"printf '%s\\n' '#!/bin/sh' \"printf '%s\\\\n' '{case.companion_version}'\" "
        "> \"$directory/ld.mold\"\n",
    )


def _installer_commands(tmp_path: Path, case: InstallerCase) -> Path:
    commands = tmp_path / "commands"
    commands.mkdir()
    for name in ("awk", "install", "mkdir", "mktemp", "rm", "touch"):
        resolved = shutil.which(name)
        assert resolved is not None, f"test host must provide {name}"
        (commands / name).symlink_to(resolved)
    _write_platform_tools(commands, case)
    _write_download_tool(commands)
    _write_checksum_tool(commands)
    _write_archive_tool(commands, case)
    return commands


def _run_installer(tmp_path: Path, case: InstallerCase) -> InstallerRun:
    commands = _installer_commands(tmp_path, case)
    prefix = tmp_path / "installed-tools"
    scratch = tmp_path / "scratch"
    scratch.mkdir()
    archive = f"mold-{VERSION}-{case.architecture}-linux.tar.gz"
    curl_url_log = tmp_path / "curl-url.log"
    checksum_log = tmp_path / "checksum.log"
    tar_marker = tmp_path / "tar-called"
    environment = {
        "PATH": str(commands),
        "HOME": str(tmp_path / "home"),
        "BUILD_TOOLS_PREFIX": str(prefix),
        "TMPDIR": str(scratch),
        "CURL_URL_LOG": str(curl_url_log),
        "CHECKSUM_LOG": str(checksum_log),
        "TAR_MARKER": str(tar_marker),
        "EXPECTED_DIGEST": DIGESTS[case.architecture],
        "EXPECTED_ARCHIVE": archive,
        "CHECKSUM_MODE": "accept" if case.checksum_matches else "mismatch",
    }
    result = subprocess.run(
        ["/bin/bash", "scripts/install-build-tools.sh"],
        cwd=ROOT,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )
    marker = prefix / "bin" / f".mold-{VERSION}-{DIGESTS[case.architecture]}"
    return InstallerRun(
        result, archive, marker, curl_url_log, checksum_log, tar_marker, prefix
    )


@pytest.mark.parametrize(
    "case",
    [
        pytest.param(InstallerCase("x86_64", "mold 2.41.0 (test)", True, 0), id="x86_64-pinned-companion"),
        pytest.param(InstallerCase("x86_64", "mold 2.40.4 (test)", True, 1), id="x86_64-wrong-companion-version"),
        pytest.param(InstallerCase("x86_64", "mold 2.41.0 (test)", False, 1), id="x86_64-checksum-mismatch"),
        pytest.param(InstallerCase("aarch64", "mold 2.41.0 (test)", True, 0), id="aarch64-pinned-companion"),
        pytest.param(InstallerCase("aarch64", "mold 2.40.4 (test)", True, 1), id="aarch64-wrong-companion-version"),
        pytest.param(InstallerCase("aarch64", "mold 2.41.0 (test)", False, 1), id="aarch64-checksum-mismatch"),
    ],
)
def test_installer_checks_ld_mold_before_writing_success_marker(
    tmp_path: Path,
    case: InstallerCase,
) -> None:
    """Fresh installs validate the archive and linker pair for each Linux target."""
    run = _run_installer(tmp_path, case)
    assert run.result.returncode == case.expected_status, run.result.stderr
    assert run.curl_url_log.read_text(encoding="utf-8").strip().endswith(run.archive)
    assert run.checksum_log.read_text(encoding="utf-8").splitlines() == [
        DIGESTS[case.architecture],
        run.archive,
    ]
    assert run.marker.exists() is (case.expected_status == 0)
    if case.checksum_matches:
        assert run.tar_marker.exists()
    else:
        assert "Pinned mold archive checksum failed" in run.result.stderr
        assert not run.tar_marker.exists(), "checksum failure must precede extraction"
        assert not (run.prefix / "bin" / "mold").exists()
        assert not (run.prefix / "bin" / "ld.mold").exists()
    if case.checksum_matches and case.expected_status != 0:
        assert "Installed ld.mold does not report the pinned version" in run.result.stderr
