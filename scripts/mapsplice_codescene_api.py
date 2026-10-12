"""Validate bounded GitHub API responses used by the CodeScene audit."""

from __future__ import annotations

import json
import re
from collections.abc import Callable, Mapping
from dataclasses import dataclass
from urllib.parse import parse_qs, urlsplit

PAGE_SIZE = 100
MAX_PAGES = 100
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


def _response_parts(raw: str) -> tuple[list[str], str]:
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
        raise VerificationError("api-response", "GitHub API JSON is malformed") from error


def parse_http_response(raw: str, returncode: int) -> ApiResponse:
    """Parse gh's include output while keeping response values out of errors."""
    lines, body = _response_parts(raw)
    status = _response_status(lines, returncode)
    headers = _response_headers(lines)
    if status != 200:
        return ApiResponse(status, headers, None)
    return ApiResponse(status, headers, _response_body(body))


def _validate_response_headers(response: ApiResponse) -> None:
    if not isinstance(response.headers, Mapping) or any(
        not isinstance(key, str) or not isinstance(value, str)
        for key, value in response.headers.items()
    ):
        raise VerificationError("api-response", "GitHub API headers are malformed")
    if len({key.lower() for key in response.headers}) != len(response.headers):
        raise VerificationError("api-response", "GitHub API repeats a header")


def _validate_response_status(response: ApiResponse) -> None:
    if type(response.status) is not int or not 100 <= response.status <= 599:
        raise VerificationError("api-response", "GitHub API status is malformed")


def _validate_resource_status(response: ApiResponse, resource: str) -> None:
    if response.status == 403:
        raise VerificationError("access-denied", f"{resource} access denied (HTTP 403)")
    if response.status == 404:
        raise VerificationError(
            "not-found", f"{resource} not found or unavailable to this identity (HTTP 404)"
        )
    if response.status != 200:
        raise VerificationError("api-status", f"{resource} returned HTTP {response.status}")


def response_object(response: ApiResponse, resource: str) -> Mapping[str, object]:
    """Require a well-formed successful API response containing an object."""
    _validate_response_status(response)
    _validate_response_headers(response)
    _validate_resource_status(response, resource)
    if not isinstance(response.body, dict):
        raise VerificationError("api-shape", f"{resource} JSON must be an object")
    return response.body


def environment(response: ApiResponse) -> tuple[str, bool, bool]:
    """Validate the CodeScene environment's protected-branch policy."""
    body = response_object(response, "codescene environment")
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


def page(response: ApiResponse) -> tuple[int, list[object]]:
    """Validate one bounded deployment-policy API page."""
    body = response_object(response, "deployment branch policies")
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


def _link_target_page(url_text: str, endpoint: str) -> int:
    try:
        url = urlsplit(url_text)
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
        if set(query) != {"per_page", "page"} or query["per_page"] != [str(PAGE_SIZE)]:
            raise ValueError
        pages = query["page"]
        page_number = int(pages[0])
        if len(pages) != 1 or page_number < 1:
            raise ValueError
    except (ValueError, KeyError) as error:
        raise VerificationError(
            "pagination-link", "deployment branch policy Link has invalid paging"
        ) from error
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


# Keep the verifier's existing private import names available to its tests and callers.
_parse_http_response = parse_http_response
_object = response_object
_environment = environment
_page = page
_links = links
