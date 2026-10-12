"""Offline contracts for the read-only CodeScene environment verifier."""

from __future__ import annotations

import subprocess
import sys
from collections.abc import Mapping
from pathlib import Path
from types import SimpleNamespace

import pytest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

import verify_codescene_environment as verifier
from mapsplice_codescene_models import MAX_PAGES

REPOSITORY = "leynos/mapsplice"
REPOSITORY_ENDPOINT = f"repos/{REPOSITORY}"
ENVIRONMENT = f"repos/{REPOSITORY}/environments/codescene"
POLICIES = "repositories/1208786150/environments/codescene/deployment-branch-policies"
PAGE_ONE = f"{POLICIES}?per_page=100&page=1"
PAGE_TWO = f"{POLICIES}?per_page=100&page=2"
PAGE_THREE = f"{POLICIES}?per_page=100&page=3"
MAIN_POLICY = {"id": 1, "name": "main", "type": "branch"}
REPOSITORY_METADATA = {"id": 1208786150, "full_name": REPOSITORY}


def _environment(name: str = "codescene") -> dict[str, object]:
    return {
        "name": name,
        "deployment_branch_policy": {
            "protected_branches": False,
            "custom_branch_policies": True,
        },
    }


def _response(
    body: object,
    *,
    status: int = 200,
    headers: Mapping[str, str] | None = None,
) -> verifier.ApiResponse:
    return verifier.ApiResponse(status, headers or {}, body)


def _page(
    items: list[object],
    count: object = 1,
    *,
    headers: Mapping[str, str] | None = None,
) -> verifier.ApiResponse:
    return _response({"total_count": count, "branch_policies": items}, headers=headers)


def _policy(
    identifier: int, name: str = "main", kind: str = "branch"
) -> dict[str, object]:
    return {"id": identifier, "name": name, "type": kind}


class ScriptedFetch:
    """Return exact queued API replies and record every requested endpoint."""

    def __init__(self, replies: dict[str, list[verifier.ApiResponse]]) -> None:
        self.replies = {
            endpoint: list(responses) for endpoint, responses in replies.items()
        }
        self.calls: list[str] = []

    def __call__(self, endpoint: str) -> verifier.ApiResponse:
        self.calls.append(endpoint)
        queued = self.replies.get(endpoint)
        assert queued, f"unexpected or repeated API read: {endpoint}"
        return queued.pop(0)

    def assert_consumed(self) -> None:
        leftovers = [endpoint for endpoint, queued in self.replies.items() if queued]
        assert not leftovers, f"unconsumed API replies: {leftovers!r}"


def _valid_fetch(
    *,
    first_page: verifier.ApiResponse | None = None,
    repeated_page: verifier.ApiResponse | None = None,
    sentinel: verifier.ApiResponse | None = None,
    environment_reads: list[verifier.ApiResponse] | None = None,
) -> ScriptedFetch:
    first = first_page or _page([MAIN_POLICY])
    repeated = repeated_page or first
    count = first.body.get("total_count", 1) if isinstance(first.body, dict) else 1
    return ScriptedFetch(
        {
            ENVIRONMENT: environment_reads
            or [_response(_environment()), _response(_environment())],
            REPOSITORY_ENDPOINT: [_response(REPOSITORY_METADATA)],
            PAGE_ONE: [first, repeated],
            PAGE_TWO: [sentinel or _page([], count)],
        }
    )


def _assert_code(fetch: ScriptedFetch, code: str) -> None:
    with pytest.raises(verifier.VerificationError) as caught:
        verifier.verify_environment(REPOSITORY, fetch)
    assert caught.value.code == code, (
        f"expected verification code {code!r}, observed error {caught.value!r}"
    )


def test_valid_protection_reads_environment_and_policies_back() -> None:
    """The successful proof reads the environment and first page twice."""
    fetch = _valid_fetch()
    assert verifier.verify_environment(REPOSITORY, fetch) == (1, "main", "branch"), (
        "verified main-branch protection must return policy tuple "
        "(1, 'main', 'branch')"
    )
    assert fetch.calls == [ENVIRONMENT, REPOSITORY_ENDPOINT] + [
        PAGE_ONE,
        PAGE_TWO,
        PAGE_ONE,
        ENVIRONMENT,
    ], f"successful verification made unexpected API calls: {fetch.calls!r}"
    fetch.assert_consumed()


@pytest.mark.parametrize(
    "repository",
    [{"id": 999, "full_name": "other/repo"}, {"id": True, "full_name": REPOSITORY}],
)
def test_repository_metadata_must_prove_identity(repository: dict[str, object]) -> None:
    fetch = _valid_fetch()
    fetch.replies[REPOSITORY_ENDPOINT] = [_response(repository)]
    _assert_code(fetch, "repository-identity")
    assert fetch.calls == [ENVIRONMENT, REPOSITORY_ENDPOINT], (
        f"repository identity rejection made unexpected API calls: {fetch.calls!r}"
    )


def test_canonical_numeric_sentinel_links_are_accepted() -> None:
    """GitHub sends prev/first/last links even on the empty second page."""
    url = f"https://api.github.com/{POLICIES}?per_page=100&page=1"
    links = ", ".join(f'<{url}>; rel="{rel}"' for rel in ("prev", "last", "first"))
    fetch = _valid_fetch(sentinel=_page([], headers={"Link": links}))
    assert verifier.verify_environment(REPOSITORY, fetch) == (1, "main", "branch")
    fetch.assert_consumed()


@pytest.mark.parametrize(
    ("status", "code"),
    [(403, "access-denied"), (404, "not-found")],
    ids=["forbidden", "absent"],
)
def test_environment_access_failures_remain_distinct(status: int, code: str) -> None:
    fetch = ScriptedFetch(
        {ENVIRONMENT: [_response({"token": "private"}, status=status)]}
    )
    _assert_code(fetch, code)
    assert fetch.calls == [ENVIRONMENT], (
        f"HTTP {status} environment rejection made unexpected API calls: "
        f"{fetch.calls!r}"
    )


@pytest.mark.parametrize(
    ("count", "items", "code"),
    [
        (True, [MAIN_POLICY], "policy-shape"),
        (2, [MAIN_POLICY], "policy-count"),
        (2, [MAIN_POLICY, _policy(2, "release")], "policy-count"),
        (1, [_policy(1, "*")], "policy-identity"),
        (1, [_policy(1, "main", "tag")], "policy-identity"),
        (1, [{"id": 1, "name": "main"}], "policy-shape"),
        (1, [{"id": 1, "name": "main", "type": 7}], "policy-shape"),
    ],
    ids=[
        "boolean-count",
        "short-page",
        "extra-branch",
        "wildcard",
        "tag",
        "missing-type",
        "invalid-type",
    ],
)
def test_only_one_main_branch_policy_is_accepted(
    count: object, items: list[object], code: str
) -> None:
    fetch = _valid_fetch(first_page=_page(items, count))
    _assert_code(fetch, code)


def test_pagination_reads_101_entries_and_an_empty_sentinel() -> None:
    """The second page and empty sentinel are required before rejecting extras."""
    many = [_policy(identifier, f"branch-{identifier}") for identifier in range(1, 101)]
    final = _policy(101, "main")
    next_url = f"https://api.github.com/{POLICIES}?per_page=100&page=2"
    page_one = _page(
        many,
        101,
        headers={"Link": f'<{next_url}>; rel="next", <{next_url}>; rel="last"'},
    )
    fetch = _valid_fetch(first_page=page_one)
    fetch.replies[PAGE_TWO] = [_page([final], 101)]
    fetch.replies[PAGE_THREE] = [_page([], 101)]
    _assert_code(fetch, "policy-count")
    expected_calls = [ENVIRONMENT, REPOSITORY_ENDPOINT] + [
        PAGE_ONE,
        PAGE_TWO,
        PAGE_THREE,
        PAGE_ONE,
    ]
    assert fetch.calls == expected_calls, (
        f"101-policy pagination calls were {fetch.calls!r}; "
        f"expected {expected_calls!r}"
    )


@pytest.mark.parametrize(
    ("link", "code"),
    [
        (None, "pagination-link"),
        ('<{url}&page=1>; rel="next", <{url}&page=2>; rel="last"', "pagination-cycle"),
        ('<{url}&page=2#extra>; rel="next"', "pagination-link"),
        (
            '<{url}&page=2>; rel="next", <{url}&page=999>; rel="last"',
            "pagination-cycle",
        ),
        ('<https://[ghp_private_marker]/route?page=2>; rel="next"', "pagination-link"),
        (
            (
                "<https://api.github.com/repositories/999/environments/codescene/"
                'deployment-branch-policies?per_page=100&page=2>; rel="next"'
            ),
            "pagination-link",
        ),
    ],
    ids=["no-link", "loop", "fragment", "last", "malformed", "unrelated-repository"],
)
def test_invalid_pagination_links_fail_closed(link: str | None, code: str) -> None:
    url = f"https://api.github.com/{POLICIES}?per_page=100"
    headers = {} if link is None else {"Link": link.format(url=url)}
    fetch = _valid_fetch(
        first_page=_page([_policy(i) for i in range(100)], 101, headers=headers)
    )

    with pytest.raises(verifier.VerificationError) as caught:
        verifier.verify_environment(REPOSITORY, fetch)
    assert caught.value.code == code, (
        f"invalid pagination link {link!r} produced code {caught.value.code!r}"
    )
    assert "ghp_private_marker" not in str(caught.value), (
        f"invalid pagination link exposed its private marker: {caught.value!r}"
    )


def test_count_change_between_pages_is_reported() -> None:
    first_items = [_policy(i, f"branch-{i}") for i in range(1, 101)]
    next_url = f"https://api.github.com/{POLICIES}?per_page=100&page=2"
    first = _page(
        first_items,
        101,
        headers={"Link": f'<{next_url}>; rel="next", <{next_url}>; rel="last"'},
    )
    fetch = _valid_fetch(first_page=first)
    fetch.replies[PAGE_TWO] = [_page([_policy(101)], 100)]
    _assert_code(fetch, "policy-drift")


def test_duplicate_policy_ids_are_rejected() -> None:
    fetch = _valid_fetch(first_page=_page([MAIN_POLICY, _policy(1, "release")], 2))
    _assert_code(fetch, "pagination-cycle")


def test_nonempty_sentinel_page_is_rejected() -> None:
    fetch = _valid_fetch(sentinel=_page([_policy(2, "release")], 1))
    _assert_code(fetch, "policy-count")


def test_first_page_change_between_reads_is_reported() -> None:
    fetch = _valid_fetch(repeated_page=_page([_policy(2)]))
    _assert_code(fetch, "policy-drift")


def test_environment_detail_change_between_reads_is_reported() -> None:
    fetch = _valid_fetch(
        environment_reads=[_response(_environment()), _response(_environment("other"))]
    )
    _assert_code(fetch, "environment-shape")
    assert fetch.calls[-1] == ENVIRONMENT, (
        f"environment readback ended at {fetch.calls[-1]!r}, expected {ENVIRONMENT!r}"
    )


def test_policy_count_above_the_audit_cap_stops_before_pagination() -> None:
    fetch = _valid_fetch(first_page=_page([], MAX_PAGES * 100 + 1))
    _assert_code(fetch, "policy-cap")
    assert fetch.calls == [ENVIRONMENT, REPOSITORY_ENDPOINT, PAGE_ONE], (
        f"policy-cap rejection made unexpected API calls: {fetch.calls!r}"
    )


def test_cli_uses_only_github_gets_and_never_echoes_api_secrets(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """A raw API error body and gh stderr stay out of the audit output."""
    secret = "ghp_private_response_marker"
    invocation: dict[str, object] = {}

    def fake_run(arguments: list[str], **kwargs: object) -> SimpleNamespace:
        invocation["arguments"] = arguments
        invocation["kwargs"] = kwargs
        return SimpleNamespace(
            stdout=f"HTTP/1.1 403 Forbidden\r\nX-Trace: safe\r\n\r\n{secret}",
            stderr=secret,
            returncode=1,
        )

    monkeypatch.setattr(verifier.subprocess, "run", fake_run)
    result = verifier.main(["--repo", REPOSITORY], fetch=verifier.gh_fetch)
    output = capsys.readouterr()

    assert result == 1, f"CLI returned status {result} for the simulated HTTP 403"
    assert secret not in output.out + output.err, (
        f"CLI exposed the simulated API marker in stdout/stderr: "
        f"{output.out!r} {output.err!r}"
    )
    expected_arguments = [
        "gh",
        "api",
        "--hostname",
        "github.com",
        "--include",
        "--method",
        "GET",
        ENVIRONMENT,
    ]
    assert invocation["arguments"] == expected_arguments, (
        f"GitHub request arguments were {invocation['arguments']!r}; "
        f"expected GET-only arguments {expected_arguments!r}"
    )
    expected_kwargs = {
        "capture_output": True,
        "text": True,
        "timeout": 30,
        "check": False,
    }
    assert invocation["kwargs"] == expected_kwargs, (
        f"GitHub subprocess options were {invocation['kwargs']!r}; "
        f"expected {expected_kwargs!r}"
    )


def test_duplicate_json_keys_are_rejected_without_echoing_values() -> None:
    secret = "ghp_private_json_marker"
    raw = (
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n"
        f'{{"name":"codescene","name":"{secret}"}}'
    )

    with pytest.raises(verifier.VerificationError) as caught:
        verifier.parse_http_response(raw, 0)

    assert caught.value.code == "api-response", (
        f"duplicate JSON key produced code {caught.value.code!r}"
    )
    assert secret not in str(caught.value), (
        f"duplicate JSON key error exposed its second value: {caught.value!r}"
    )


@pytest.mark.parametrize(
    "response",
    [
        verifier.ApiResponse("ghp_secret", {}, {}),
        verifier.ApiResponse(200, {"Link": "safe", "link": "other"}, {}),
        verifier.ApiResponse(200, {"X-Trace": 123}, {}),
    ],
    ids=["invalid-status", "case-insensitive-duplicate-header", "invalid-header-value"],
)
def test_injected_malformed_api_response_cannot_leak_values(
    response: verifier.ApiResponse,
) -> None:
    fetch = ScriptedFetch({ENVIRONMENT: [response]})
    with pytest.raises(verifier.VerificationError) as caught:
        verifier.verify_environment(REPOSITORY, fetch)

    assert caught.value.code == "api-response", (
        f"malformed API response {response!r} produced code {caught.value.code!r}"
    )
    assert "ghp_secret" not in str(caught.value), (
        f"malformed API response leaked its private marker: {caught.value!r}"
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
    """Timeout and launch failures retain one code but distinguish safe causes."""

    def fail_run(*args: object, **kwargs: object) -> None:
        raise failure

    monkeypatch.setattr(verifier.subprocess, "run", fail_run)
    assert verifier.main(["--repo", REPOSITORY], fetch=verifier.gh_fetch) == 1
    output = capsys.readouterr()
    assert output.out == ""
    assert output.err == (
        "codescene environment: api-unavailable: GitHub API request did not complete"
        f" (operation=environment failure_kind={kind})\n"
    )
    assert "ghp_private_marker" not in output.err


def test_invalid_repository_has_bounded_repository_context(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Invalid input reports a stable repository error before making API requests."""
    fetch = ScriptedFetch({})
    assert verifier.main(["--repo", "ghp_private_marker/../repo"], fetch=fetch) == 1
    output = capsys.readouterr()
    assert output.out == ""
    assert output.err == (
        "codescene environment: repository: repository must be OWNER/NAME"
        " (operation=repository)\n"
    )
    assert fetch.calls == []
