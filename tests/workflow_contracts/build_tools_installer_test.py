"""Controlled installation tests for the pinned Linux linker pair."""

from __future__ import annotations

import shlex
import shutil
import subprocess
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
VERSION = "2.41.0"
DIGESTS = {
    "x86_64": "a3696680d99e692970590a178bc3a33d78d60d1c6dc9db7a11b557b02b751f5d",
    "aarch64": "946de2774b06a71346bd59b55fddba610b65b8d93c3a4a1559cc84e103472710",
}


def _write_command(directory: Path, name: str, contents: str) -> None:
    command = directory / name
    command.write_text(contents, encoding="utf-8")
    command.chmod(0o755)


@pytest.mark.parametrize(
    ("architecture", "companion_version", "checksum_matches", "expected_status"),
    [
        ("x86_64", "mold 2.41.0 (test)", True, 0),
        ("x86_64", "mold 2.40.4 (test)", True, 1),
        ("x86_64", "mold 2.41.0 (test)", False, 1),
        ("aarch64", "mold 2.41.0 (test)", True, 0),
        ("aarch64", "mold 2.40.4 (test)", True, 1),
        ("aarch64", "mold 2.41.0 (test)", False, 1),
    ],
    ids=[
        "x86_64-pinned-companion",
        "x86_64-wrong-companion-version",
        "x86_64-checksum-mismatch",
        "aarch64-pinned-companion",
        "aarch64-wrong-companion-version",
        "aarch64-checksum-mismatch",
    ],
)
def test_installer_checks_ld_mold_before_writing_success_marker(
    tmp_path: Path,
    architecture: str,
    companion_version: str,
    checksum_matches: bool,
    expected_status: int,
) -> None:
    """Fresh installs validate the archive and linker pair for each Linux target."""
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
        f"  -m) printf '%s\\n' {architecture} ;;\n"
        "  *) exit 1 ;;\nesac\n",
    )
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
    _write_command(
        commands,
        "tar",
        "#!/bin/sh\nprintf '%s\\n' called > \"$TAR_MARKER\"\n"
        "destination=\nwhile [ \"$#\" -gt 0 ]; do\n"
        "  if [ \"$1\" = -C ]; then destination=$2; shift 2; else shift; fi\n"
        "done\n"
        f"directory=\"$destination/mold-{VERSION}-{architecture}-linux/bin\"\n"
        "mkdir -p \"$directory\"\n"
        f"printf '%s\\n' '#!/bin/sh' \"printf '%s\\\\n' 'mold {VERSION} (test)'\" "
        "> \"$directory/mold\"\n"
        f"printf '%s\\n' '#!/bin/sh' \"printf '%s\\\\n' '{companion_version}'\" "
        "> \"$directory/ld.mold\"\n",
    )

    prefix = tmp_path / "installed-tools"
    scratch = tmp_path / "scratch"
    scratch.mkdir()
    archive = f"mold-{VERSION}-{architecture}-linux.tar.gz"
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
        "EXPECTED_DIGEST": DIGESTS[architecture],
        "EXPECTED_ARCHIVE": archive,
        "CHECKSUM_MODE": "accept" if checksum_matches else "mismatch",
    }
    result = subprocess.run(
        ["/bin/bash", "scripts/install-build-tools.sh"],
        cwd=ROOT,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )

    marker = prefix / "bin" / f".mold-{VERSION}-{DIGESTS[architecture]}"
    assert result.returncode == expected_status, result.stderr
    assert curl_url_log.read_text(encoding="utf-8").strip().endswith(archive)
    assert checksum_log.read_text(encoding="utf-8").splitlines() == [
        DIGESTS[architecture],
        archive,
    ]
    assert marker.exists() is (expected_status == 0)
    if checksum_matches:
        assert tar_marker.exists()
    else:
        assert "Pinned mold archive checksum failed" in result.stderr
        assert not tar_marker.exists(), "checksum failure must precede extraction"
        assert not (prefix / "bin" / "mold").exists()
        assert not (prefix / "bin" / "ld.mold").exists()
    if checksum_matches and expected_status != 0:
        assert "Installed ld.mold does not report the pinned version" in result.stderr
