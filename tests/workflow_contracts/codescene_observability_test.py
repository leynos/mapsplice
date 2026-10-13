"""Deterministic contracts for payload-free CodeScene audit observations."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
from types import SimpleNamespace

import pytest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

import verify_codescene_environment as verifier
from mapsplice_codescene_observability import AuditSession

SECRET = "credential-sentinel-do-not-log"
REPO = "private-owner/private-repository"


class Clock:
    """Advance an exact eighth of a second per observation."""

    def __init__(self) -> None:
        self.value = 0.0

    def __call__(self) -> float:
        self.value += 0.125
        return self.value


def _response(body: object, status: int = 200) -> verifier.ApiResponse:
    return verifier.ApiResponse(status, {"authorization": SECRET}, body)


def _environment(custom: bool = True) -> verifier.ApiResponse:
    return _response(
        {
            "name": "codescene",
            "sensitive": SECRET,
            "deployment_branch_policy": {
                "protected_branches": False,
                "custom_branch_policies": custom,
            },
        }
    )


def _page(*, total: int = 1, empty: bool = False, identifier: int = 1):
    return _response(
        {
            "total_count": total,
            "sensitive": SECRET,
            "branch_policies": []
            if empty
            else [
                {
                    "id": identifier,
                    "name": "main",
                    "type": "branch",
                    "sensitive": SECRET,
                }
            ],
        }
    )


def _replies() -> list[verifier.ApiResponse]:
    return [
        _environment(),
        _response({"id": 123, "full_name": REPO, "sensitive": SECRET}),
        _page(),
        _page(empty=True),
        _page(),
        _environment(),
    ]


@pytest.fixture
def observed(monkeypatch):
    """Expose the CLI's real observation session with deterministic time and sink."""
    events: list[dict[str, object]] = []
    session = AuditSession(clock=Clock(), emit=events.append)
    monkeypatch.setattr(verifier, "AuditSession", lambda: session)
    return events


def _run(replies: list[verifier.ApiResponse]) -> int:
    queued = iter(replies)
    return verifier.main(["--repo", REPO], lambda _: next(queued))


def _assert_safe(events: list[dict[str, object]]) -> None:
    text = json.dumps(events)
    for forbidden in (SECRET, REPO, "authorization", "api.github.com", "sensitive"):
        assert forbidden not in text
    for event in events:
        common = {"event", "error_code", "outcome", "elapsed_ms"}
        if event["event"] == "codescene_summary":
            expected = common | {
                "request_count",
                "page_count",
                "page_read_count",
                "request_latency_ms",
                "request_outcomes",
                "stage_outcomes",
            }
        else:
            expected = common | {"operation", "resource", "page"}
            if event["event"] == "codescene_request":
                expected.add("status")
            else:
                assert event["event"] == "codescene_stage"
        assert set(event) == expected
        assert isinstance(event["elapsed_ms"], (float, int))
        assert 0 <= event["elapsed_ms"] <= 86400000
        assert "message" not in event
        assert "endpoint" not in event
        assert "headers" not in event
        assert "body" not in event


def test_cli_observes_all_requests_and_both_readbacks(observed, capsys):
    """Every actual fetch contributes once, including sentinel and repeated page 1."""
    assert _run(_replies()) == 0
    requests = [event for event in observed if event["event"] == "codescene_request"]
    assert [(event["operation"], event["page"]) for event in requests] == [
        ("environment", None),
        ("repository", None),
        ("policy_page", 1),
        ("policy_page", 2),
        ("policy_readback", 1),
        ("environment_readback", None),
    ]
    assert all(event["status"] == 200 for event in requests)
    assert all(event["elapsed_ms"] == 125.0 for event in requests)
    summary = observed[-1]
    assert summary["event"] == "codescene_summary"
    assert summary["request_count"] == 6
    assert summary["page_count"] == 2
    assert summary["page_read_count"] == 3
    assert summary["request_latency_ms"] == 750.0
    assert summary["request_outcomes"]["success"] == 6
    assert summary["outcome"] == "success"
    _assert_safe(observed)
    assert "verified main-only deployment branch policy" in capsys.readouterr().out


@pytest.mark.parametrize(
    ("scenario", "code", "operation", "count"),
    [
        ("access", "access-denied", "environment", 1),
        ("shape", "api-shape", "environment", 1),
        ("cap", "policy-cap", "policy_page", 3),
        ("policy_drift", "policy-drift", "policy_readback", 5),
        ("environment_policy", "environment-policy", "environment_readback", 6),
    ],
)
def test_cli_reports_semantic_failures_and_early_summary(
    scenario,
    code,
    operation,
    count,
    observed,
    capsys,
):
    """HTTP success and semantic success remain distinct observations."""
    replies = _replies()
    if scenario == "access":
        replies[0] = _response({"message": SECRET}, 403)
    elif scenario == "shape":
        replies[0] = _response([SECRET])
    elif scenario == "cap":
        replies[2] = _page(total=10001)
    elif scenario == "policy_drift":
        replies[4] = _page(identifier=2)
    else:
        replies[5] = _environment(False)
    assert _run(replies) == 1
    summary = observed[-1]
    assert summary["request_count"] == count
    assert summary["error_code"] == code
    if code in {"policy-cap", "policy-drift", "access-denied"}:
        assert summary["outcome"] == code
        assert summary["stage_outcomes"][code] >= 1
    failures = [
        event
        for event in observed
        if event["event"] == "codescene_stage" and event["error_code"] == code
    ]
    assert any(event["operation"] == operation for event in failures)
    assert code in capsys.readouterr().err
    _assert_safe(observed)


@pytest.mark.parametrize("failure", ["timeout", "malformed_json"])
def test_gh_transport_and_parser_failure_are_observed(
    failure,
    observed,
    monkeypatch,
    capsys,
):
    """Raw subprocess output and timeout attributes never enter JSON or diagnostics."""

    def run(*args, **kwargs):
        if failure == "timeout":
            raise subprocess.TimeoutExpired([SECRET], 30, output=SECRET, stderr=SECRET)
        return SimpleNamespace(stdout=f"HTTP/2.0 200 OK\n\n{SECRET}", returncode=0)

    monkeypatch.setattr(verifier.subprocess, "run", run)
    assert verifier.main(["--repo", REPO], verifier.gh_fetch) == 1
    summary = observed[-1]
    assert summary["request_count"] == 1
    assert summary["outcome"] == ("timeout" if failure == "timeout" else "failure")
    expected_code = "api-unavailable" if failure == "timeout" else "api-response"
    assert summary["error_code"] == expected_code
    requests = [event for event in observed if event["event"] == "codescene_request"]
    assert len(requests) == 1
    assert requests[0]["outcome"] == summary["outcome"]
    _assert_safe(observed)
    assert SECRET not in capsys.readouterr().err


def test_observations_sanitize_untrusted_error_codes():
    """Injected errors cannot expand the fixed telemetry label vocabulary."""
    events = []
    session = AuditSession(clock=Clock(), emit=events.append)
    error = verifier.VerificationError(SECRET, SECRET)
    with (
        pytest.raises(verifier.VerificationError),
        session.run(),
        session.stage("environment"),
    ):
        raise error
    assert events[-1]["error_code"] == "unexpected"
    assert events[-1]["outcome"] == "failure"
    _assert_safe(events)


def test_default_sink_emits_json_summary_on_early_rejection(capsys):
    """The production sink is stderr JSON and preserves the bounded old diagnostic."""
    code = verifier.main(
        ["--repo", "invalid"], lambda _: pytest.fail("unexpected fetch")
    )
    assert code == 1
    lines = capsys.readouterr().err.splitlines()
    events = [json.loads(line) for line in lines[:-1]]
    assert events[-1]["request_count"] == 0
    assert events[-1]["error_code"] == "repository"
    assert lines[-1] == (
        "codescene environment: repository: repository must be OWNER/NAME "
        "(operation=repository)"
    )


@pytest.mark.parametrize(
    ("failure", "kind"),
    [
        (subprocess.TimeoutExpired("ghp_private_marker", 30), "timeout"),
        (FileNotFoundError("ghp_private_marker"), "process_launch"),
    ],
)
def test_transport_failure_has_bounded_kind_and_operation(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
    failure: Exception,
    kind: str,
) -> None:
    """Timeout and launch failures retain their exact bounded human diagnostic."""

    def fail_run(*args: object, **kwargs: object) -> None:
        raise failure

    monkeypatch.setattr(verifier.subprocess, "run", fail_run)
    assert verifier.main(["--repo", "leynos/mapsplice"], fetch=verifier.gh_fetch) == 1
    output = capsys.readouterr()
    assert output.out == ""
    lines = output.err.splitlines(keepends=True)
    assert lines[-1] == (
        "codescene environment: api-unavailable: GitHub API request did not complete"
        f" (operation=environment failure_kind={kind})\n"
    )
    assert len(lines[-1]) < 200
    assert len(lines) <= 5
    assert all(len(line) < 1800 for line in lines[:-1])
    events = [json.loads(line) for line in lines[:-1]]
    _assert_safe(events)
    assert "ghp_private_marker" not in output.err
    assert "leynos/mapsplice" not in output.err
    requests = [event for event in events if event["event"] == "codescene_request"]
    assert len(requests) == 1
    request = requests[0]
    assert request["operation"] == "environment"
    assert request["resource"] == "environment"
    assert request["page"] is None
    assert request["status"] is None
    assert request["error_code"] == "api-unavailable"
    outcome = "timeout" if kind == "timeout" else "failure"
    assert request["outcome"] == outcome
    summary = events[-1]
    assert summary["event"] == "codescene_summary"
    assert summary["request_count"] == 1
    assert summary["page_count"] == summary["page_read_count"] == 0
    assert summary["error_code"] == "api-unavailable"
    assert summary["outcome"] == outcome
    assert summary["request_outcomes"][outcome] == 1
    assert summary["stage_outcomes"][outcome] == 1
    assert summary["request_latency_ms"] == request["elapsed_ms"]


def test_invalid_repository_has_bounded_repository_context(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Repository preflight emits zero-request metrics and the unchanged diagnostic."""
    calls = []

    def fetch(endpoint: str):
        calls.append(endpoint)
        pytest.fail("invalid repository must not make an API request")

    assert verifier.main(["--repo", "ghp_private_marker/../repo"], fetch=fetch) == 1
    output = capsys.readouterr()
    assert output.out == ""
    lines = output.err.splitlines(keepends=True)
    assert lines[-1] == (
        "codescene environment: repository: repository must be OWNER/NAME"
        " (operation=repository)\n"
    )
    assert calls == []
    assert len(lines[-1]) < 200
    assert all(len(line) < 1800 for line in lines[:-1])
    assert "ghp_private_marker" not in output.err
    events = [json.loads(line) for line in lines[:-1]]
    _assert_safe(events)
    assert len(events) == 2
    stage, summary = events
    assert stage["event"] == "codescene_stage"
    assert stage["operation"] == stage["resource"] == "repository"
    assert stage["page"] is None
    assert stage["error_code"] == "repository"
    assert stage["outcome"] == "failure"
    assert summary["event"] == "codescene_summary"
    assert summary["request_count"] == 0
    assert summary["page_count"] == summary["page_read_count"] == 0
    assert summary["request_latency_ms"] == 0
    assert summary["error_code"] == "repository"
    assert summary["outcome"] == "failure"
    assert sum(summary["request_outcomes"].values()) == 0
    assert summary["stage_outcomes"]["failure"] == 1
