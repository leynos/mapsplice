"""Controlled installation tests for the pinned Linux linker pair."""

from __future__ import annotations

import hashlib
import io
import shlex
import shutil
import subprocess
import tarfile
import tomllib
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
    mold_version: str = "mold 2.41.0 (test)"


@dataclass(frozen=True)
class InstallerVersionCase:
    binary_name: str
    reported_version: str
    expected_status: int
    expected_diagnostic: str | None


@dataclass(frozen=True)
class InstallerRun:
    result: subprocess.CompletedProcess[str]
    archive: str
    marker: Path
    curl_url_log: Path
    checksum_log: Path
    tar_marker: Path
    prefix: Path
    rustup_args_log: Path


def _write_command(directory: Path, name: str, contents: str) -> None:
    command = directory / name
    command.write_text(contents, encoding="utf-8")
    command.chmod(0o755)


def _write_platform_tools(commands: Path, case: InstallerCase) -> None:
    _write_command(
        commands,
        "rustup",
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$RUSTUP_ARGS_LOG\"\nexit 0\n",
    )
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
        "cp \"$ARCHIVE_FIXTURE\" \"$archive\"\n",
    )


def _write_checksum_tool(commands: Path) -> None:
    real_sha256sum = shutil.which("sha256sum")
    assert real_sha256sum is not None, "test host must provide sha256sum"
    # Map the pinned release digest to this fixture's real hash only on valid cases.
    _write_command(
        commands,
        "sha256sum",
        "#!/bin/sh\nset -eu\n"
        "[ \"$#\" -eq 2 ] && [ \"$1\" = --check ] && [ \"$2\" = --status ] || exit 2\n"
        "read -r digest archive\n"
        "printf '%s\\n%s\\n%s\\n%s\\n' \"$1\" \"$2\" \"$digest\" "
        "\"${archive##*/}\" > \"$CHECKSUM_LOG\"\n"
        "[ \"$digest\" = \"$EXPECTED_DIGEST\" ] || exit 1\n"
        "[ \"${archive##*/}\" = \"$EXPECTED_ARCHIVE\" ] || exit 1\n"
        "if [ \"$CHECKSUM_MODE\" = accept ]; then\n"
        f"  actual=$({shlex.quote(real_sha256sum)} \"$archive\")\n"
        "  actual=${actual%% *}\n"
        "  digest=$actual\n"
        "fi\n"
        "printf '%s  %s\\n' \"$digest\" \"$archive\" | "
        f"{shlex.quote(real_sha256sum)} \"$@\"\n",
    )


def _write_archive_tool(commands: Path) -> None:
    real_tar = shutil.which("tar")
    assert real_tar is not None, "test host must provide tar"
    _write_command(
        commands,
        "tar",
        "#!/bin/sh\nprintf '%s\\n' called > \"$TAR_MARKER\"\n"
        f"exec {shlex.quote(real_tar)} \"$@\"\n",
    )


def _write_archive_fixture(path: Path, case: InstallerCase) -> None:
    directory = f"mold-{VERSION}-{case.architecture}-linux/bin"
    binaries = {
        f"{directory}/mold": (
            "#!/bin/sh\nprintf '%s\\n' "
            f"{shlex.quote(case.mold_version)}\n"
        ),
        f"{directory}/ld.mold": (
            "#!/bin/sh\nprintf '%s\\n' "
            f"{shlex.quote(case.companion_version)}\n"
        ),
    }
    with tarfile.open(path, "w:gz") as archive:
        for name, contents in binaries.items():
            payload = contents.encode("utf-8")
            entry = tarfile.TarInfo(name)
            entry.mode = 0o755
            entry.size = len(payload)
            archive.addfile(entry, io.BytesIO(payload))


def _installer_commands(tmp_path: Path, case: InstallerCase) -> Path:
    commands = tmp_path / "commands"
    commands.mkdir()
    for name in ("awk", "cp", "gzip", "install", "mkdir", "mktemp", "rm", "touch"):
        resolved = shutil.which(name)
        assert resolved is not None, f"test host must provide {name}"
        (commands / name).symlink_to(resolved)
    _write_platform_tools(commands, case)
    _write_download_tool(commands)
    _write_checksum_tool(commands)
    _write_archive_tool(commands)
    return commands


def _run_installer(tmp_path: Path, case: InstallerCase) -> InstallerRun:
    commands = _installer_commands(tmp_path, case)
    prefix = tmp_path / "installed-tools"
    scratch = tmp_path / "scratch"
    scratch.mkdir()
    archive = f"mold-{VERSION}-{case.architecture}-linux.tar.gz"
    archive_fixture = tmp_path / "archive-fixture.tar.gz"
    _write_archive_fixture(archive_fixture, case)
    fixture_digest = hashlib.sha256(archive_fixture.read_bytes()).hexdigest()
    assert fixture_digest != DIGESTS[case.architecture]
    curl_url_log = tmp_path / "curl-url.log"
    checksum_log = tmp_path / "checksum.log"
    tar_marker = tmp_path / "tar-called"
    rustup_args_log = tmp_path / "rustup-args.log"
    environment = {
        "PATH": str(commands),
        "HOME": str(tmp_path / "home"),
        "BUILD_TOOLS_PREFIX": str(prefix),
        "TMPDIR": str(scratch),
        "CURL_URL_LOG": str(curl_url_log),
        "CHECKSUM_LOG": str(checksum_log),
        "RUSTUP_ARGS_LOG": str(rustup_args_log),
        "TAR_MARKER": str(tar_marker),
        "ARCHIVE_FIXTURE": str(archive_fixture),
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
        result,
        archive,
        marker,
        curl_url_log,
        checksum_log,
        tar_marker,
        prefix,
        rustup_args_log,
    )


def test_installer_uses_rust_toolchain_channel_and_components(tmp_path: Path) -> None:
    """The rustup invocation follows the repository's pinned toolchain manifest."""
    case = InstallerCase("x86_64", "mold 2.41.0 (test)", True, 0)
    run = _run_installer(tmp_path, case)
    assert run.result.returncode == 0, (
        f"installer returned status {run.result.returncode}: {run.result.stderr!r}"
    )
    toolchain = tomllib.loads(
        (ROOT / "rust-toolchain.toml").read_text(encoding="utf-8")
    )["toolchain"]
    expected_arguments = ["toolchain", "install", toolchain["channel"]]
    for component in toolchain["components"]:
        expected_arguments.extend(["--component", component])
    observed_arguments = run.rustup_args_log.read_text(encoding="utf-8").splitlines()

    assert observed_arguments == expected_arguments, (
        "installer rustup arguments must use the manifest channel and components; "
        f"observed {observed_arguments!r}, expected {expected_arguments!r}"
    )


@pytest.mark.parametrize(
    "case",
    [
        pytest.param(
            InstallerVersionCase("mold", "mold 2.41.0 (test)", 0, None),
            id="mold-pinned",
        ),
        pytest.param(
            InstallerVersionCase(
                "mold",
                "mold 2.40.4 (test)",
                1,
                "Installed mold does not report the pinned version",
            ),
            id="mold-wrong-version",
        ),
        pytest.param(
            InstallerVersionCase("ld.mold", "mold 2.41.0 (test)", 0, None),
            id="ld-linker-pinned",
        ),
        pytest.param(
            InstallerVersionCase(
                "ld.mold",
                "mold 2.40.4 (test)",
                1,
                "Installed ld.mold does not report the pinned version",
            ),
            id="ld-linker-wrong-version",
        ),
    ],
)
def test_installer_requires_each_binary_to_report_the_pinned_version(
    tmp_path: Path,
    case: InstallerVersionCase,
) -> None:
    """Each installed linker is checked independently before success is marked."""
    mold_version = (
        case.reported_version
        if case.binary_name == "mold"
        else "mold 2.41.0 (test)"
    )
    companion_version = (
        case.reported_version
        if case.binary_name == "ld.mold"
        else "mold 2.41.0 (test)"
    )
    installer_case = InstallerCase(
        "x86_64",
        companion_version,
        True,
        case.expected_status,
        mold_version=mold_version,
    )
    run = _run_installer(tmp_path, installer_case)

    assert run.result.returncode == case.expected_status, (
        f"{case.binary_name} reporting {case.reported_version!r} returned "
        f"{run.result.returncode}; stderr was {run.result.stderr!r}"
    )
    if case.expected_diagnostic is not None:
        assert case.expected_diagnostic in run.result.stderr, (
            f"{case.binary_name} version rejection omitted "
            f"{case.expected_diagnostic!r}: "
            f"{run.result.stderr!r}"
        )
    assert run.marker.exists() is (case.expected_status == 0), (
        f"success marker state for {case.binary_name} reporting "
        f"{case.reported_version!r} "
        f"was {run.marker.exists()} with stderr {run.result.stderr!r}"
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
    assert run.result.returncode == case.expected_status, (
        f"installer exit status was {run.result.returncode}; stderr was "
        f"{run.result.stderr!r}"
    )
    download_url = run.curl_url_log.read_text(encoding="utf-8").strip()
    assert download_url.endswith(run.archive), (
        f"download URL {download_url!r} did not end with archive {run.archive!r}"
    )
    checksum_arguments = run.checksum_log.read_text(encoding="utf-8").splitlines()
    assert checksum_arguments == [
        "--check",
        "--status",
        DIGESTS[case.architecture],
        run.archive,
    ], f"checksum tool arguments were {checksum_arguments!r}"
    assert run.marker.exists() is (case.expected_status == 0), (
        f"success marker state was {run.marker.exists()} for exit status "
        f"{case.expected_status}; stderr was {run.result.stderr!r}"
    )
    if case.checksum_matches:
        assert run.tar_marker.exists(), "valid checksum must reach archive extraction"
    else:
        assert "Pinned mold archive checksum failed" in run.result.stderr, (
            f"checksum failure diagnostic was {run.result.stderr!r}"
        )
        assert not run.tar_marker.exists(), "checksum failure must precede extraction"
        assert not (run.prefix / "bin" / "mold").exists(), (
            "checksum failure must stop before linker extraction"
        )
        assert not (run.prefix / "bin" / "ld.mold").exists(), (
            "checksum failure must not install ld.mold"
        )
    if case.checksum_matches and case.expected_status != 0:
        assert "Installed ld.mold does not report the pinned version" in run.result.stderr, (
            f"wrong ld.mold version diagnostic was {run.result.stderr!r}"
        )
