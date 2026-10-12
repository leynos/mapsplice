"""Process-boundary contracts for the CodeScene environment verifier."""

from __future__ import annotations

import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
REPOSITORY = "leynos/mapsplice"
ENVIRONMENT = f"repos/{REPOSITORY}/environments/codescene"
REPOSITORY_ENDPOINT = f"repos/{REPOSITORY}"
POLICIES = "repositories/1208786150/environments/codescene/deployment-branch-policies"
COMMON_ARGS = ["api", "--hostname", "github.com", "--include", "--method", "GET"]
PRIVATE_MARKER = "ghp_private_process_marker"


def _fake_gh_failures() -> str:
    """Supply fake transport failures before any successful response is emitted."""
    return r"""if os.environ["FAKE_GH_MODE"] == "forbidden":
    sys.stdout.write(
        "HTTP/1.1 403 Forbidden\r\n"
        "X-Private: ghp_private_process_marker\r\n\r\n"
        "ghp_private_process_marker"
    )
    sys.stderr.write("ghp_private_process_marker")
    raise SystemExit(1)

if os.environ["FAKE_GH_MODE"].startswith("invalid-"):
    stream = (
        sys.stdout if os.environ["FAKE_GH_MODE"] == "invalid-stdout" else sys.stderr
    )
    stream.buffer.write(b"ghp_private_process_marker\xff")
    raise SystemExit(0)

request_count = len(open(os.environ["FAKE_GH_CALLS"], encoding="utf-8").read().splitlines())
failure_requests = {
    "environment-error": 1, "repository-error": 2, "page-error": 4,
    "page-malformed": 4, "policy-readback-error": 5, "environment-readback-error": 6,
}
if failure_requests.get(os.environ["FAKE_GH_MODE"]) == request_count:
    if os.environ["FAKE_GH_MODE"] == "page-malformed":
        sys.stdout.write("HTTP/1.1 200 OK\r\n\r\nghp_private_process_marker")
        raise SystemExit(0)
    sys.stdout.write("HTTP/1.1 403 Forbidden\r\n\r\nghp_private_process_marker")
    sys.stderr.write("ghp_private_process_marker")
    raise SystemExit(1)

"""


def _fake_gh_responses() -> str:
    """Supply the normal endpoint-specific bodies used by the process contract."""
    return r"""endpoint = args[6]
if endpoint == "repos/leynos/mapsplice/environments/codescene":
    body = {
        "name": "codescene",
        "deployment_branch_policy": {
            "protected_branches": False,
            "custom_branch_policies": True,
        },
    }
elif endpoint == "repos/leynos/mapsplice":
    body = {"id": 1208786150, "full_name": "leynos/mapsplice"}
elif endpoint == (
    "repositories/1208786150/environments/codescene/"
    "deployment-branch-policies?per_page=100&page=1"
):
    body = {
        "total_count": 1,
        "branch_policies": [{"id": 1, "name": "main", "type": "branch"}],
    }
elif endpoint == (
    "repositories/1208786150/environments/codescene/"
    "deployment-branch-policies?per_page=100&page=2"
):
    body = {"total_count": 1, "branch_policies": []}
else:
    sys.stderr.write("unexpected endpoint")
    raise SystemExit(3)

sys.stdout.write(
    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n"
    + json.dumps(body)
)
"""


def _fake_gh(bin_directory: Path) -> Path:
    """Write an executable gh probe with request recording and scripted outcomes."""
    executable = bin_directory / "gh"
    script = (
        "#!"
        + sys.executable
        + "\n"
        + r"""import json
import os
import sys

args = sys.argv[1:]
with open(os.environ["FAKE_GH_CALLS"], "a", encoding="utf-8") as calls:
    calls.write(json.dumps(args) + "\n")
expected = ["api", "--hostname", "github.com", "--include", "--method", "GET"]
if args[:6] != expected or len(args) != 7:
    sys.stderr.write("invalid request " + " ".join(args))
    raise SystemExit(2)

"""
        + _fake_gh_failures()
        + _fake_gh_responses()
    )
    executable.write_text(script, encoding="utf-8")
    executable.chmod(0o755)
    return executable


def _run_verifier(
    tmp_path: Path, mode: str
) -> tuple[subprocess.CompletedProcess[str], list[list[str]]]:
    bin_directory = tmp_path / "bin"
    bin_directory.mkdir()
    _fake_gh(bin_directory)
    calls_path = tmp_path / "calls.jsonl"
    child_environment = {
        "PATH": str(bin_directory),
        "HOME": str(tmp_path),
        "LC_ALL": "C",
        "LANG": "C",
        "NO_COLOR": "1",
        "FAKE_GH_CALLS": str(calls_path),
        "FAKE_GH_MODE": mode,
    }
    result = subprocess.run(
        [
            sys.executable,
            str(ROOT / "scripts/verify_codescene_environment.py"),
            "--repo",
            REPOSITORY,
        ],
        cwd=tmp_path,
        env=child_environment,
        capture_output=True,
        text=True,
        timeout=10,
        check=False,
    )
    calls = [
        json.loads(line) for line in calls_path.read_text(encoding="utf-8").splitlines()
    ]
    return result, calls


def test_cli_process_parses_responses_and_uses_only_api_gets(
    tmp_path: Path,
) -> None:
    result, calls = _run_verifier(tmp_path, "verified")
    expected_endpoints = [
        ENVIRONMENT,
        REPOSITORY_ENDPOINT,
        f"{POLICIES}?per_page=100&page=1",
        f"{POLICIES}?per_page=100&page=2",
        f"{POLICIES}?per_page=100&page=1",
        ENVIRONMENT,
    ]

    assert result.returncode == 0, (
        f"verified audit returned exit status {result.returncode}: {result.stderr!r}"
    )
    assert (
        result.stdout
        == "codescene environment: verified main-only deployment branch policy\n"
    ), f"verified audit stdout was {result.stdout!r}"
    assert result.stderr == "", f"verified audit wrote stderr: {result.stderr!r}"
    expected_calls = [COMMON_ARGS + [endpoint] for endpoint in expected_endpoints]
    assert calls == expected_calls, f"verified audit API calls were {calls!r}"


def test_cli_process_redacts_api_body_and_gh_stderr(
    tmp_path: Path,
) -> None:
    result, calls = _run_verifier(tmp_path, "forbidden")

    assert result.returncode == 1, (
        f"403 response returned exit status {result.returncode}: {result.stderr!r}"
    )
    assert result.stdout == "", f"403 response wrote stdout: {result.stdout!r}"
    assert "codescene environment: access-denied" in result.stderr, (
        f"403 diagnostic omitted access-denied classification: {result.stderr!r}"
    )
    assert "HTTP 403" in result.stderr, (
        f"403 diagnostic omitted HTTP status: {result.stderr!r}"
    )
    assert PRIVATE_MARKER not in result.stderr, (
        f"403 response leaked its private marker in stderr: {result.stderr!r}"
    )
    assert len(result.stderr) < 200, f"diagnostic was not bounded: {result.stderr!r}"
    assert calls == [COMMON_ARGS + [ENVIRONMENT]], (
        f"403 response made unexpected API calls: {calls!r}"
    )


@pytest.mark.parametrize("mode", ["invalid-stdout", "invalid-stderr"])
def test_cli_process_bounds_invalid_utf8_output(tmp_path: Path, mode: str) -> None:
    """Undecodable output from either pipe must fail without exposing its bytes."""
    result, calls = _run_verifier(tmp_path, mode)

    assert result.returncode == 1, (
        f"{mode} returned exit status {result.returncode}: {result.stderr!r}"
    )
    assert result.stdout == "", f"{mode} wrote stdout: {result.stdout!r}"
    assert result.stderr == (
        "codescene environment: api-unavailable: GitHub API request did not complete"
        " (operation=environment failure_kind=decode)\n"
    ), f"{mode} produced an unexpected diagnostic: {result.stderr!r}"
    assert "Traceback" not in result.stderr, (
        f"{mode} exposed a traceback: {result.stderr!r}"
    )
    assert PRIVATE_MARKER not in result.stderr, (
        f"{mode} leaked the invalid bytes' marker: {result.stderr!r}"
    )
    assert calls == [COMMON_ARGS + [ENVIRONMENT]], (
        f"{mode} made unexpected API calls: {calls!r}"
    )


@dataclass(frozen=True)
class FailureStageCase:
    """One transport failure and its expected bounded location metadata."""

    mode: str
    operation: str
    page: int | None
    error_code: str
    request_count: int


@pytest.mark.parametrize(
    "case",
    [
        FailureStageCase("environment-error", "environment", None, "access-denied", 1),
        FailureStageCase("repository-error", "repository", None, "access-denied", 2),
        FailureStageCase("page-error", "policy_page", 2, "access-denied", 4),
        FailureStageCase("page-malformed", "policy_page", 2, "api-response", 4),
        FailureStageCase("policy-readback-error", "policy_readback", 1, "access-denied", 5),
        FailureStageCase(
            "environment-readback-error", "environment_readback", None, "access-denied", 6
        ),
    ],
)
def test_cli_process_identifies_bounded_failure_stage(
    tmp_path: Path, case: FailureStageCase
) -> None:
    """Failed requests expose only a trusted stage and requested policy-page number."""
    result, calls = _run_verifier(tmp_path, case.mode)

    assert result.returncode == 1, (
        f"{case.mode} returned exit status {result.returncode}: {result.stderr!r}"
    )
    assert result.stdout == "", f"{case.mode} wrote stdout: {result.stdout!r}"
    expected_prefix = f"codescene environment: {case.error_code}:"
    assert expected_prefix in result.stderr, (
        f"{case.mode} stderr omitted {expected_prefix!r}: {result.stderr!r}"
    )
    expected_context = f"operation={case.operation}"
    if case.page is not None:
        expected_context += f" page={case.page}"
    else:
        assert "page=" not in result.stderr, (
            f"{case.mode} added an unrequested page number: {result.stderr!r}"
        )
    assert f"({expected_context})" in result.stderr, (
        f"{case.mode} stderr omitted bounded context {expected_context!r}: "
        f"{result.stderr!r}"
    )
    assert PRIVATE_MARKER not in result.stderr, (
        f"{case.mode} leaked its private marker: {result.stderr!r}"
    )
    assert REPOSITORY not in result.stderr, (
        f"{case.mode} leaked the repository name: {result.stderr!r}"
    )
    assert "Traceback" not in result.stderr, (
        f"{case.mode} exposed a traceback: {result.stderr!r}"
    )
    assert len(result.stderr) < 200, (
        f"{case.mode} diagnostic was not bounded: {result.stderr!r}"
    )
    assert len(calls) == case.request_count, (
        f"{case.mode} made {len(calls)} API calls: {calls!r}"
    )
