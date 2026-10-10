"""Process-boundary contracts for the CodeScene environment verifier."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REPOSITORY = "leynos/mapsplice"
ENVIRONMENT = f"repos/{REPOSITORY}/environments/codescene"
REPOSITORY_ENDPOINT = f"repos/{REPOSITORY}"
POLICIES = "repositories/1208786150/environments/codescene/deployment-branch-policies"
COMMON_ARGS = ["api", "--hostname", "github.com", "--include", "--method", "GET"]
PRIVATE_MARKER = "ghp_private_process_marker"


def _fake_gh(bin_directory: Path) -> Path:
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

if os.environ["FAKE_GH_MODE"] == "forbidden":
    sys.stdout.write(
        "HTTP/1.1 403 Forbidden\r\n"
        "X-Private: ghp_private_process_marker\r\n\r\n"
        "ghp_private_process_marker"
    )
    sys.stderr.write("ghp_private_process_marker")
    raise SystemExit(1)

endpoint = args[6]
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

    assert result.returncode == 0, result.stderr
    assert (
        result.stdout
        == "codescene environment: verified main-only deployment branch policy\n"
    )
    assert result.stderr == ""
    assert calls == [COMMON_ARGS + [endpoint] for endpoint in expected_endpoints]


def test_cli_process_redacts_api_body_and_gh_stderr(
    tmp_path: Path,
) -> None:
    result, calls = _run_verifier(tmp_path, "forbidden")

    assert result.returncode == 1
    assert result.stdout == ""
    assert "codescene environment: access-denied" in result.stderr
    assert "HTTP 403" in result.stderr
    assert PRIVATE_MARKER not in result.stderr
    assert len(result.stderr) < 200, f"diagnostic was not bounded: {result.stderr!r}"
    assert calls == [COMMON_ARGS + [ENVIRONMENT]]
