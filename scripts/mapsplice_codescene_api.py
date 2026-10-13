"""Validate bounded GitHub API responses used by the CodeScene audit."""

from __future__ import annotations

import json
import re
from collections.abc import Callable, Mapping
from dataclasses import dataclass
from urllib.parse import SplitResult, parse_qs, urlsplit

from mapsplice_codescene_models import (
    PAGE_SIZE,
    AuditOperation,
    BranchPolicy,
    DeploymentPolicyPage,
    EnvironmentProtection,
    VerificationError,
)
from mapsplice_codescene_observability import AuditSession
from mapsplice_codescene_pagination import collect_policies

STATUS_LINE = re.compile(r"HTTP/\S+ (?P<status>[1-5][0-9]{2})(?: [^\r\n]*)?\Z")
LINK_PART = re.compile(r'<(?P<url>[^>]+)>;\s*rel="(?P<rel>[a-z]+)"\Z')


@dataclass(frozen=True)
class ApiResponse:
    """One GitHub API status, header set, and decoded JSON body."""

    status: int
    headers: Mapping[str, str]
    body: object


Fetch = Callable[[str], ApiResponse]


def _unique_json_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
    """Build a JSON object while rejecting duplicate keys."""
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON key")
        result[key] = value
    return result


def _reject_json_constant(_: str) -> object:
    raise ValueError("non-finite JSON number")


def _response_parts(raw: str) -> tuple[list[str], str]:
    """Split gh's included HTTP headers from its response body."""
    parts = re.split(r"\r?\n\r?\n", raw, maxsplit=1)
    if len(parts) != 2:
        raise VerificationError("api-response", "GitHub API headers are missing")
    return parts[0].splitlines(), parts[1]


def _response_status(lines: list[str], returncode: int) -> int:
    status_line = lines[0] if lines else ""
    match = STATUS_LINE.fullmatch(status_line)
    if match is None:
        raise VerificationError("api-response", "GitHub API status is malformed")
    status = int(match.group("status"))
    if (returncode == 0) != (200 <= status < 300):
        raise VerificationError("api-response", "GitHub CLI and HTTP status disagree")
    return status


def _response_headers(lines: list[str]) -> dict[str, str]:
    headers: dict[str, str] = {}
    for line in lines[1:]:
        key, separator, value = line.partition(":")
        if not separator or not key:
            raise VerificationError("api-response", "GitHub API header is malformed")
        lower_key = key.lower()
        if lower_key in headers:
            raise VerificationError("api-response", "GitHub API repeats a header")
        headers[lower_key] = value.strip()
    return headers


def _response_body(raw: str) -> object:
    try:
        return json.loads(
            raw,
            object_pairs_hook=_unique_json_object,
            parse_constant=_reject_json_constant,
        )
    except (json.JSONDecodeError, ValueError) as error:
        raise VerificationError(
            "api-response", "GitHub API JSON is malformed"
        ) from error


def parse_http_response(raw: str, returncode: int) -> ApiResponse:
    """Parse gh's include output while keeping response values out of errors."""
    lines, body = _response_parts(raw)
    status = _response_status(lines, returncode)
    headers = _response_headers(lines)
    if status != 200:
        return ApiResponse(status, headers, None)
    return ApiResponse(status, headers, _response_body(body))


def _validate_response_headers(response: ApiResponse) -> None:
    headers = response.headers
    if not isinstance(headers, Mapping):
        raise VerificationError("api-response", "GitHub API headers are malformed")
    names = _validated_header_names(headers)
    _validate_header_values(headers)
    _validate_unique_header_names(names)


def _validated_header_names(headers: Mapping[object, object]) -> list[str]:
    names = []
    for key in headers:
        if not isinstance(key, str):
            raise VerificationError("api-response", "GitHub API headers are malformed")
        names.append(key.lower())
    return names


def _validate_header_values(headers: Mapping[object, object]) -> None:
    for value in headers.values():
        if not isinstance(value, str):
            raise VerificationError("api-response", "GitHub API headers are malformed")


def _validate_unique_header_names(names: list[str]) -> None:
    if len(set(names)) != len(names):
        raise VerificationError("api-response", "GitHub API repeats a header")


def _validate_response_status(response: ApiResponse) -> None:
    if type(response.status) is not int or not 100 <= response.status <= 599:
        raise VerificationError("api-response", "GitHub API status is malformed")


def _validate_resource_status(response: ApiResponse, resource: str) -> None:
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


def response_object(response: ApiResponse, resource: str) -> Mapping[str, object]:
    """Require a well-formed successful API response containing an object."""
    _validate_response_status(response)
    _validate_response_headers(response)
    _validate_resource_status(response, resource)
    if not isinstance(response.body, dict):
        raise VerificationError("api-shape", f"{resource} JSON must be an object")
    return response.body


def environment(response: ApiResponse) -> EnvironmentProtection:
    """Decode the environment protection fields from a successful response."""
    body = response_object(response, "codescene environment")
    policy = body.get("deployment_branch_policy")
    name = body.get("name")
    if not isinstance(name, str) or not isinstance(policy, dict):
        raise VerificationError(
            "environment-shape", "codescene environment policy is missing"
        )
    protected = policy.get("protected_branches")
    custom = policy.get("custom_branch_policies")
    return EnvironmentProtection(
        name,
        protected if type(protected) is bool else None,
        custom if type(custom) is bool else None,
    )


def _branch_policy(item: object) -> BranchPolicy:
    if not isinstance(item, dict):
        raise VerificationError(
            "policy-shape", "deployment branch policy entry is malformed"
        )
    identifier = item.get("id")
    name = item.get("name")
    kind = item.get("type")
    if type(identifier) is not int:
        raise VerificationError(
            "policy-shape", "deployment branch policy identity is incomplete"
        )
    if not isinstance(name, str) or not isinstance(kind, str):
        raise VerificationError(
            "policy-shape", "deployment branch policy identity is incomplete"
        )
    return BranchPolicy(identifier, name, kind)


def deployment_policy_page(
    response: ApiResponse,
    endpoint: str,
    number: int,
) -> DeploymentPolicyPage:
    """Decode one bounded API page into typed policy values."""
    body = response_object(response, "deployment branch policies")
    count = body.get("total_count")
    items = body.get("branch_policies")
    if not _is_non_negative_integer(count):
        raise VerificationError(
            "policy-shape", "deployment branch policy count or list is malformed"
        )
    if not isinstance(items, list):
        raise VerificationError(
            "policy-shape", "deployment branch policy count or list is malformed"
        )
    if len(items) > PAGE_SIZE:
        raise VerificationError(
            "policy-cap", "deployment branch policy page exceeds 100 entries"
        )
    page_links = links(response, endpoint)
    policies = tuple(_branch_policy(item) for item in items)
    return DeploymentPolicyPage(
        number,
        count,
        policies,
        page_links,
    )


def _is_non_negative_integer(value: object) -> bool:
    if type(value) is not int:
        return False
    return value >= 0


def _link_relation(part: str) -> tuple[str, str]:
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
    return relation, match.group("url")


def _validated_link_query(url_text: str, endpoint: str) -> str:
    try:
        url = urlsplit(url_text)
    except ValueError:
        raise VerificationError(
            "pagination-link", "deployment branch policy Link URL is malformed"
        ) from None
    if not _is_github_api_url(url):
        _raise_link_outside_api_route()
    if not _is_expected_api_endpoint(url, endpoint):
        _raise_link_outside_api_route()
    return url.query


def _is_github_api_url(url: SplitResult) -> bool:
    return url.scheme == "https" and url.netloc == "api.github.com"


def _is_expected_api_endpoint(url: SplitResult, endpoint: str) -> bool:
    return url.path == f"/{endpoint}" and not url.fragment


def _raise_link_outside_api_route() -> None:
    raise VerificationError(
        "pagination-link", "deployment branch policy Link leaves the API route"
    )


def _link_target_page(url_text: str, endpoint: str) -> int:
    return _page_number(_validated_link_query(url_text, endpoint))


def _invalid_page_query() -> VerificationError:
    return VerificationError(
        "pagination-link", "deployment branch policy Link has invalid paging"
    )


def _page_number(query_text: str) -> int:
    try:
        # Empty values must remain visible to cardinality and exact-key checks.
        query = parse_qs(query_text, strict_parsing=True, keep_blank_values=True)
    except ValueError as error:
        raise _invalid_page_query() from error
    pages = _validated_page_values(query)
    return _positive_page_number(pages[0])


def _validated_page_values(query: dict[str, list[str]]) -> list[str]:
    if set(query) != {"per_page", "page"}:
        raise _invalid_page_query()
    if query["per_page"] != [str(PAGE_SIZE)]:
        raise _invalid_page_query()
    pages = query["page"]
    if len(pages) != 1:
        raise _invalid_page_query()
    return pages


def _positive_page_number(value: str) -> int:
    try:
        page_number = int(value)
    except ValueError as error:
        raise _invalid_page_query() from error
    if page_number < 1:
        raise _invalid_page_query()
    return page_number


def links(response: ApiResponse, endpoint: str) -> dict[str, int]:
    """Parse and constrain all pagination links to the expected API route."""
    raw = next(
        (value for key, value in response.headers.items() if key.lower() == "link"),
        "",
    )
    parsed: dict[str, int] = {}
    for part in raw.split(",") if raw else ():
        relation, target = _link_relation(part)
        if relation in parsed:
            raise VerificationError(
                "pagination-cycle", "deployment branch policy Link repeats a relation"
            )
        parsed[relation] = _link_target_page(target, endpoint)
    return parsed


def deployment_policies(
    endpoint: str, fetch: Fetch, audit: AuditSession | None = None
) -> tuple[BranchPolicy, ...]:
    """Read complete stable policies from one repository's GitHub API route.

    For example, ``deployment_policies(endpoint, gh_fetch)`` returns typed
    policies only after pagination completeness and readback are proved.
    """

    def read_page(
        number: int, operation: AuditOperation = "policy_page"
    ) -> DeploymentPolicyPage:
        page_endpoint = f"{endpoint}?per_page={PAGE_SIZE}&page={number}"
        response = (
            audit.request(fetch, page_endpoint, operation, number)
            if audit is not None
            else fetch(page_endpoint)
        )
        return deployment_policy_page(response, endpoint, number)

    return collect_policies(
        read_page, audit, readback=lambda number: read_page(number, "policy_readback")
    )
