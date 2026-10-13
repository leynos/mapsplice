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
@pytest.mark.parametrize("response_file", [False, True])
def test_clang_route_forwards_target_and_linker(
    tmp_path: Path,
    target: str,
    uses_pinned_linker: bool,
    response_file: bool,
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
    if response_file:
        response = tmp_path / "linker arguments"
        response.write_text("\n".join(arguments) + "\n", encoding="utf-8")
        arguments = [f"@{response}"]
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
    assert observed == expected, (
        f"clang wrapper arguments differ for {target} with pinned linker="
        f"{uses_pinned_linker}: expected {expected!r}, observed {observed!r}"
    )


@pytest.mark.parametrize("architecture", ["x86_64", "aarch64"])
@pytest.mark.parametrize(
    ("missing_tool", "linker_version"),
    [
        (None, "mold 2.41.0 (test)"),
        ("clang", "mold 2.41.0 (test)"),
        ("mold", "mold 2.41.0 (test)"),
        ("ld.mold", None),
        ("ld.mold", "mold 2.40.4 (test)"),
    ],
)
def test_linux_preflight_requires_linker_tools_and_pinned_companion(
    tmp_path: Path,
    architecture: str,
    missing_tool: str | None,
    linker_version: str | None,
) -> None:
    """Linux tool checks run on both supported architectures in an isolated PATH."""
    commands = tmp_path / "commands"
    commands.mkdir()
    _executable(commands / "rustc", "exit 0\n")
    _executable(
        commands / "rustup",
        "printf '%s\\n' rustfmt-test clippy-test rust-analyzer-test llvm-tools-test\n",
    )
    _executable(
        commands / "uname",
        'if [ "$1" = -s ]; then echo Linux; else echo "$BUILD_ARCH"; fi\n',
    )
    if missing_tool != "clang":
        _executable(commands / "clang", "exit 0\n")
    if missing_tool != "mold":
        _executable(commands / "mold", "echo 'mold 2.41.0 (test)'\n")
    _executable(
        commands / "grep",
        """[ \"$1\" = -q ] || exit 2
pattern=${2#^}
while IFS= read -r component; do
  case \"$component\" in \"$pattern\"*) exit 0 ;; esac
done
exit 1
""",
    )
    if linker_version is not None:
        _executable(commands / "ld.mold", f"echo '{linker_version}'\n")
    result = subprocess.run(
        ["/bin/bash", "scripts/check-build-tools.sh"],
        cwd=ROOT,
        env={"PATH": str(commands), "BUILD_ARCH": architecture},
        capture_output=True,
        text=True,
        check=False,
    )
    if missing_tool is None:
        assert result.returncode == 0, result.stderr
        return

    prerequisite = "clang" if missing_tool == "clang" else f"{missing_tool} 2.41.0"
    if missing_tool == "ld.mold" and linker_version is not None:
        prerequisite = "ld.mold 2.41.0"
    assert result.returncode == 1, (
        f"missing {prerequisite} must exit 1; observed exit {result.returncode}, "
        f"stderr {result.stderr!r}"
    )
    assert f"Build prerequisite missing: {prerequisite}" in result.stderr, (
        f"preflight stderr must identify missing {prerequisite}; "
        f"observed {result.stderr!r}"
    )
    assert "make install-build-tools" in result.stderr, (
        f"preflight stderr must name the installer route for {prerequisite}; "
        f"observed {result.stderr!r}"
    )
    if missing_tool == "clang":
        assert "system package manager" in result.stderr, (
            f"missing clang diagnostic must name its system package route; "
            f"observed {result.stderr!r}"
        )


@pytest.mark.parametrize(
    "response_contents",
    [
        None,
        "",
        "-fuse-ld=mold-other\n",
        " -fuse-ld=mold\n",
        '"-fuse-ld=mold"\n',
        "$(touch SHOULD_NOT_EXIST)\n",
        "@nested-response\n",
    ],
)
def test_unrelated_response_arguments_do_not_select_pinned_linker(
    tmp_path: Path, response_contents: str | None
) -> None:
    """Only a literal complete response-file token selects the pinned linker."""
    commands = tmp_path / "commands"
    commands.mkdir()
    _executable(
        commands / "clang",
        'printf \'%s\\n\' "$@" > "$CLANG_ARGUMENTS"\n',
    )
    response = tmp_path / "response"
    if response_contents is not None:
        response.write_text(response_contents, encoding="utf-8")
    nested = tmp_path / "nested-response"
    nested.write_text("-fuse-ld=mold\n", encoding="utf-8")
    recording = tmp_path / "arguments"
    argument = f"@{response}"
    result = subprocess.run(
        [str(ROOT / "scripts/clang-linker.sh"), argument],
        cwd=tmp_path,
        env={
            **os.environ,
            "PATH": f"{commands}:{os.environ['PATH']}",
            "CLANG_ARGUMENTS": str(recording),
        },
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, (
        f"literal response-file argument should succeed; observed exit "
        f"{result.returncode}, stderr {result.stderr!r}"
    )
    assert recording.read_text(encoding="utf-8").splitlines() == [argument], (
        f"clang wrapper must pass the response-file argument unchanged; "
        f"expected {[argument]!r}, observed "
        f"{recording.read_text(encoding='utf-8').splitlines()!r}"
    )
    assert not (tmp_path / "SHOULD_NOT_EXIST").exists(), (
        "response-file contents must not be evaluated as shell commands"
    )
