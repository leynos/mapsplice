"""Read GitHub's CodeScene environment protection without changing it.

Example: ``python3 scripts/verify_codescene_environment.py --repo leynos/mapsplice``
returns success only after the environment and its sole main-branch policy have
been read back. The API reads are not an atomic snapshot; a second read detects
ordinary changes while the check runs.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from collections.abc import Callable, Mapping
from dataclasses import dataclass
from urllib.parse import parse_qs, urlsplit

PAGE_SIZE = 100
MAX_PAGES = 100
REPOSITORY = re.compile(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+\Z")
STATUS_LINE = re.compile(r"HTTP/\S+ (?P<status>[1-5][0-9]{2})(?: [^\r\n]*)?\Z")
LINK_PART = re.compile(r'<(?P<url>[^>]+)>;\s*rel="(?P<rel>[a-z]+)"\Z')


class VerificationError(Exception):
    """Report one bounded reason why protection cannot be proved."""

    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code


@dataclass(frozen=True)
class ApiResponse:
    """One GitHub API status, header set, and decoded JSON body."""

    status: int
    headers: Mapping[str, str]
    body: object


Fetch = Callable[[str], ApiResponse]


def _unique_json_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON key")
        result[key] = value
    return result


def _reject_json_constant(_: str) -> object:
    raise ValueError("non-finite JSON number")


def _parse_http_response(raw: str, returncode: int) -> ApiResponse:
    """Parse gh's include output while keeping response values out of errors."""
    parts = re.split(r"\r?\n\r?\n", raw, maxsplit=1)
    if len(parts) != 2:
        raise VerificationError("api-response", "GitHub API headers are missing")
    lines = parts[0].splitlines()
    status_match = STATUS_LINE.fullmatch(lines[0]) if lines else None
    if status_match is None:
        raise VerificationError("api-response", "GitHub API status is malformed")
    status = int(status_match.group("status"))
    if (returncode == 0) != (200 <= status < 300):
        raise VerificationError("api-response", "GitHub CLI and HTTP status disagree")
    headers: dict[str, str] = {}
    for line in lines[1:]:
        key, separator, value = line.partition(":")
        if not separator or not key:
            raise VerificationError("api-response", "GitHub API header is malformed")
        lower_key = key.lower()
        if lower_key in headers:
            raise VerificationError("api-response", "GitHub API repeats a header")
        headers[lower_key] = value.strip()
    if status != 200:
        return ApiResponse(status, headers, None)
    try:
        body = json.loads(
            parts[1],
            object_pairs_hook=_unique_json_object,
            parse_constant=_reject_json_constant,
        )
    except (json.JSONDecodeError, ValueError) as error:
        raise VerificationError(
            "api-response", "GitHub API JSON is malformed"
        ) from error
    return ApiResponse(status, headers, body)


def gh_fetch(endpoint: str) -> ApiResponse:
    """Make exactly one read-only API request through the configured gh login."""
    try:
        result = subprocess.run(
            [
                "gh",
                "api",
                "--hostname",
                "github.com",
                "--include",
                "--method",
                "GET",
                endpoint,
            ],
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise VerificationError(
            "api-unavailable", "GitHub API request did not complete"
        ) from error
    # In particular, never include stderr: gh can print account details or a
    # server-supplied message that is not suitable for an audit log.
    return _parse_http_response(result.stdout, result.returncode)


def _object(response: ApiResponse, resource: str) -> Mapping[str, object]:
    if type(response.status) is not int or not 100 <= response.status <= 599:
        raise VerificationError("api-response", "GitHub API status is malformed")
    if not isinstance(response.headers, Mapping) or any(
        not isinstance(key, str) or not isinstance(value, str)
        for key, value in response.headers.items()
    ):
        raise VerificationError("api-response", "GitHub API headers are malformed")
    if len({key.lower() for key in response.headers}) != len(response.headers):
        raise VerificationError("api-response", "GitHub API repeats a header")
    if response.status == 403:
        raise VerificationError("access-denied", f"{resource} access denied (HTTP 403)")
    if response.status == 404:
        raise VerificationError(
            "not-found",
            f"{resource} not found or unavailable to this identity (HTTP 404)",
        )
    if response.status != 200:
        raise VerificationError(
            "api-status", f"{resource} returned HTTP {response.status}"
        )
    if not isinstance(response.body, dict):
        raise VerificationError("api-shape", f"{resource} JSON must be an object")
    return response.body


def _environment(response: ApiResponse) -> tuple[str, bool, bool]:
    body = _object(response, "codescene environment")
    policy = body.get("deployment_branch_policy")
    if body.get("name") != "codescene" or not isinstance(policy, dict):
        raise VerificationError(
            "environment-shape", "codescene environment policy is missing"
        )
    protected = policy.get("protected_branches")
    custom = policy.get("custom_branch_policies")
    if protected is not False or custom is not True:
        raise VerificationError(
            "environment-policy",
            "codescene requires protected_branches=false and custom_branch_policies=true",
        )
    return ("codescene", protected, custom)


def _page(response: ApiResponse) -> tuple[int, list[object]]:
    body = _object(response, "deployment branch policies")
    count = body.get("total_count")
    items = body.get("branch_policies")
    if type(count) is not int or count < 0 or not isinstance(items, list):
        raise VerificationError(
            "policy-shape", "deployment branch policy count or list is malformed"
        )
    if len(items) > PAGE_SIZE:
        raise VerificationError(
            "policy-cap", "deployment branch policy page exceeds 100 entries"
        )
    return count, items


def _links(response: ApiResponse, endpoint: str) -> dict[str, int]:
    raw = next(
        (value for key, value in response.headers.items() if key.lower() == "link"),
        "",
    )
    if not raw:
        return {}
    links: dict[str, int] = {}
    for part in raw.split(","):
        match = LINK_PART.fullmatch(part.strip())
        if match is None:
            raise VerificationError(
                "pagination-link", "deployment branch policy Link is malformed"
            )
        relation = match.group("rel")
        if relation not in {"first", "prev", "next", "last"}:
            raise VerificationError(
                "pagination-link", "deployment branch policy Link relation is unknown"
            )
        if relation in links:
            raise VerificationError(
                "pagination-cycle", "deployment branch policy Link repeats a relation"
            )
        try:
            url = urlsplit(match.group("url"))
        except ValueError:
            raise VerificationError(
                "pagination-link", "deployment branch policy Link URL is malformed"
            ) from None
        if (
            url.scheme != "https"
            or url.netloc != "api.github.com"
            or url.path != f"/{endpoint}"
            or url.fragment
        ):
            raise VerificationError(
                "pagination-link", "deployment branch policy Link leaves the API route"
            )
        try:
            query = parse_qs(url.query, strict_parsing=True)
            if set(query) != {"per_page", "page"} or query["per_page"] != [
                str(PAGE_SIZE)
            ]:
                raise ValueError
            page = int(query["page"][0])
            if len(query["page"]) != 1 or page < 1:
                raise ValueError
        except (ValueError, KeyError) as error:
            raise VerificationError(
                "pagination-link", "deployment branch policy Link has invalid paging"
            ) from error
        links[relation] = page
    return links


def _policy_identity(item: object) -> tuple[int, str, str]:
    if not isinstance(item, dict):
        raise VerificationError(
            "policy-shape", "deployment branch policy entry is malformed"
        )
    identifier, name, kind = item.get("id"), item.get("name"), item.get("type")
    if (
        type(identifier) is not int
        or identifier < 1
        or not isinstance(name, str)
        or not isinstance(kind, str)
    ):
        raise VerificationError(
            "policy-shape", "deployment branch policy identity is incomplete"
        )
    return identifier, name, kind


def _policies(endpoint: str, fetch: Fetch) -> tuple[int, str, str]:
    first: tuple[int, list[object]] | None = None
    first_links: dict[str, int] | None = None
    total: int | None = None
    identities: list[tuple[int, str, str]] = []
    for number in range(1, MAX_PAGES + 2):
        response = fetch(f"{endpoint}?per_page={PAGE_SIZE}&page={number}")
        count, items = _page(response)
        if total is None:
            total = count
            if total > MAX_PAGES * PAGE_SIZE:
                raise VerificationError(
                    "policy-cap", "deployment branch policy count exceeds audit cap"
                )
            first = (count, items)
        elif count != total:
            raise VerificationError(
                "policy-drift", "deployment branch policy count changed during read"
            )
        links = _links(response, endpoint)
        expected_pages = max(1, (total + PAGE_SIZE - 1) // PAGE_SIZE)
        if first_links is None:
            first_links = links
        for relation, linked_page in links.items():
            expected_page = {
                "first": 1,
                "prev": number - 1,
                "next": number + 1,
                "last": expected_pages,
            }[relation]
            if linked_page != expected_page or linked_page < 1:
                raise VerificationError(
                    "pagination-cycle",
                    "deployment branch policy Link disagrees with page order",
                )
        if number > expected_pages:
            if items or links.get("next") is not None:
                raise VerificationError(
                    "policy-count", "deployment branch policy sentinel is not empty"
                )
            break
        if len(items) != min(PAGE_SIZE, total - len(identities)):
            raise VerificationError(
                "policy-count", "deployment branch policy page is incomplete"
            )
        if number < expected_pages:
            if links.get("next") != number + 1 or links.get("last") != expected_pages:
                raise VerificationError(
                    "pagination-link",
                    "deployment branch policy next/last page is unproved",
                )
        elif links.get("next") is not None:
            raise VerificationError(
                "pagination-cycle", "deployment branch policy Link continues past total"
            )
        for item in items:
            identity = _policy_identity(item)
            if any(prior[0] == identity[0] for prior in identities):
                raise VerificationError(
                    "pagination-cycle", "deployment branch policy is repeated"
                )
            identities.append(identity)
    else:
        raise VerificationError(
            "policy-cap", "deployment branch policy pagination exceeded audit cap"
        )
    if total != len(identities) or len(identities) != 1:
        raise VerificationError(
            "policy-count", "codescene requires exactly one deployment branch policy"
        )
    identity = identities[0]
    if identity[1:] != ("main", "branch"):
        raise VerificationError(
            "policy-identity", "codescene requires only the main branch policy"
        )
    # Re-read the first page after enumeration to detect an ordinary concurrent
    # change. This is evidence of stability during the check, not a transaction.
    repeated = fetch(f"{endpoint}?per_page={PAGE_SIZE}&page=1")
    if _page(repeated) != first or _links(repeated, endpoint) != first_links:
        raise VerificationError(
            "policy-drift", "deployment branch policies changed during read"
        )
    return identity


def verify_environment(repo: str, fetch: Fetch) -> tuple[int, str, str]:
    """Prove one protected environment and exactly one main branch policy."""
    if REPOSITORY.fullmatch(repo) is None or any(
        segment in {".", ".."} for segment in repo.split("/")
    ):
        raise VerificationError("repository", "repository must be OWNER/NAME")
    environment_endpoint = f"repos/{repo}/environments/codescene"
    original = _environment(fetch(environment_endpoint))
    metadata = _object(fetch(f"repos/{repo}"), "repository")
    identifier = metadata.get("id")
    if (
        type(identifier) is not int
        or identifier < 1
        or metadata.get("full_name") != repo
    ):
        raise VerificationError(
            "repository-identity", "repository identity is unproved"
        )
    # GitHub canonicalizes pagination links to its immutable repository ID route.
    branch_endpoint = (
        f"repositories/{identifier}/environments/codescene/deployment-branch-policies"
    )
    policy = _policies(branch_endpoint, fetch)
    if _environment(fetch(environment_endpoint)) != original:
        raise VerificationError(
            "environment-drift", "codescene protection changed during read"
        )
    return policy


def main(argv: list[str] | None = None, fetch: Fetch = gh_fetch) -> int:
    """Run the read-only audit and emit only bounded non-secret diagnostics."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True, help="repository as OWNER/NAME")
    args = parser.parse_args(argv)
    try:
        verify_environment(args.repo, fetch)
    except VerificationError as error:
        print(f"codescene environment: {error.code}: {error}", file=sys.stderr)
        return 1
    print("codescene environment: verified main-only deployment branch policy")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
