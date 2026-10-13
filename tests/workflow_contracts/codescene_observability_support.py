"""Assertions for bounded CodeScene events at the CLI process boundary."""

from __future__ import annotations

import json

import pytest

REPOSITORY = "leynos/mapsplice"
PRIVATE_MARKER = "ghp_private_process_marker"


def observations(stderr: str, count: int, code: str | None = None):
    """Validate bounded JSON events separately from the unchanged human diagnostic."""
    lines = stderr.splitlines(keepends=True)
    event_lines = [line for line in lines if line.startswith("{")]
    diagnostic = "".join(line for line in lines if not line.startswith("{"))
    assert len(diagnostic) < 200
    assert all(len(line) < 1800 for line in event_lines)
    events = [json.loads(line) for line in event_lines]
    assert 1 <= len(events) <= 2 * count + 4
    assert PRIVATE_MARKER not in stderr
    assert REPOSITORY not in stderr
    assert "Traceback" not in stderr
    requests = [event for event in events if event["event"] == "codescene_request"]
    assert len(requests) == count
    expected = [
        ("environment", None),
        ("repository", None),
        ("policy_page", 1),
        ("policy_page", 2),
        ("policy_readback", 1),
        ("environment_readback", None),
    ]
    assert [(event["operation"], event["page"]) for event in requests] == expected[
        :count
    ]
    for event in events:
        assert 0 <= event["elapsed_ms"] <= 86400000
        if event["event"] == "codescene_summary":
            continue
        schema = {
            "event",
            "operation",
            "resource",
            "page",
            "error_code",
            "outcome",
            "elapsed_ms",
        }
        if event["event"] == "codescene_request":
            schema.add("status")
        else:
            assert event["event"] == "codescene_stage"
        assert set(event) == schema
        assert event["resource"] in {"environment", "repository", "deployment_policies"}
        assert event["page"] is None or 1 <= event["page"] <= 101
    summary = events[-1]
    assert summary["event"] == "codescene_summary"
    assert set(summary) == {
        "event",
        "outcome",
        "error_code",
        "elapsed_ms",
        "request_count",
        "page_count",
        "page_read_count",
        "request_latency_ms",
        "request_outcomes",
        "stage_outcomes",
    }
    assert 0 <= summary["request_count"] <= 110
    assert 0 <= summary["page_count"] <= 101
    assert summary["error_code"] == code
    assert summary["request_count"] == count
    assert summary["page_read_count"] == sum(
        event["page"] is not None for event in requests
    )
    assert summary["page_count"] == len(
        {event["page"] for event in requests if event["page"] is not None}
    )
    assert summary["request_latency_ms"] == pytest.approx(
        sum(event["elapsed_ms"] for event in requests), abs=0.001
    )
    assert sum(summary["request_outcomes"].values()) == count
    if code is None:
        assert summary["outcome"] == "success"
        assert all(
            event["status"] == 200 and event["outcome"] == "success"
            for event in requests
        )
    else:
        assert requests[-1]["error_code"] == code
        assert requests[-1]["status"] == (403 if code == "access-denied" else None)
        assert requests[-1]["outcome"] == (
            "access-denied" if code == "access-denied" else "failure"
        )
    return diagnostic
