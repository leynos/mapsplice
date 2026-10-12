"""Audit bounded GitHub deployment-policy pagination for CodeScene."""

from __future__ import annotations

from dataclasses import dataclass, field

from mapsplice_codescene_api import (
    MAX_PAGES,
    PAGE_SIZE,
    ApiResponse,
    Fetch,
    VerificationError,
    links,
    page,
)


@dataclass
class _PolicyPages:
    total: int | None = None
    expected_pages: int | None = None
    first_page: tuple[int, list[object]] | None = None
    first_links: dict[str, int] | None = None
    identities: list[tuple[int, str, str]] = field(default_factory=list)
    identifiers: set[int] = field(default_factory=set)


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


def _page_count(total: int) -> int:
    if total > MAX_PAGES * PAGE_SIZE:
        raise VerificationError(
            "policy-cap", "deployment branch policy count exceeds audit cap"
        )
    return max(1, (total + PAGE_SIZE - 1) // PAGE_SIZE)


def _remember_total(state: _PolicyPages, count: int, items: list[object]) -> None:
    if state.total is None:
        state.total = count
        state.expected_pages = _page_count(count)
        state.first_page = (count, items)
    elif count != state.total:
        raise VerificationError(
            "policy-drift", "deployment branch policy count changed during read"
        )


def _expected_link_page(relation: str, current_page: int, page_count: int) -> int:
    return {
        "first": 1,
        "prev": current_page - 1,
        "next": current_page + 1,
        "last": page_count,
    }[relation]


def _validate_link_order(
    page_links: dict[str, int], current_page: int, page_count: int
) -> None:
    for relation, linked_page in page_links.items():
        expected_page = _expected_link_page(relation, current_page, page_count)
        if linked_page != expected_page or linked_page < 1:
            raise VerificationError(
                "pagination-cycle",
                "deployment branch policy Link disagrees with page order",
            )


def _validate_policy_page(
    current_page: int,
    page_count: int,
    total: int,
    seen_count: int,
    items: list[object],
    page_links: dict[str, int],
) -> bool:
    if current_page > page_count:
        if items or page_links.get("next") is not None:
            raise VerificationError(
                "policy-count", "deployment branch policy sentinel is not empty"
            )
        return True
    expected_items = min(PAGE_SIZE, total - seen_count)
    if len(items) != expected_items:
        raise VerificationError(
            "policy-count", "deployment branch policy page is incomplete"
        )
    if current_page < page_count:
        if (
            page_links.get("next") != current_page + 1
            or page_links.get("last") != page_count
        ):
            raise VerificationError(
                "pagination-link",
                "deployment branch policy next/last page is unproved",
            )
    elif page_links.get("next") is not None:
        raise VerificationError(
            "pagination-cycle", "deployment branch policy Link continues past total"
        )
    return False


def _append_identities(
    state: _PolicyPages, items: list[object]
) -> None:
    for item in items:
        identity = _policy_identity(item)
        if identity[0] in state.identifiers:
            raise VerificationError(
                "pagination-cycle", "deployment branch policy is repeated"
            )
        state.identifiers.add(identity[0])
        state.identities.append(identity)


def _collect_policy_pages(endpoint: str, fetch: Fetch) -> _PolicyPages:
    state = _PolicyPages()
    for number in range(1, MAX_PAGES + 2):
        response = fetch(f"{endpoint}?per_page={PAGE_SIZE}&page={number}")
        count, items = page(response)
        _remember_total(state, count, items)
        page_links = links(response, endpoint)
        if state.first_links is None:
            state.first_links = page_links
        page_count = state.expected_pages
        assert page_count is not None
        _validate_link_order(page_links, number, page_count)
        if _validate_policy_page(
            number,
            page_count,
            count,
            len(state.identities),
            items,
            page_links,
        ):
            break
        _append_identities(state, items)
    else:
        raise VerificationError(
            "policy-cap", "deployment branch policy pagination exceeded audit cap"
        )
    return state


def _require_main_policy(state: _PolicyPages) -> tuple[int, str, str]:
    if state.total != len(state.identities) or len(state.identities) != 1:
        raise VerificationError(
            "policy-count", "codescene requires exactly one deployment branch policy"
        )
    identity = state.identities[0]
    if identity[1:] != ("main", "branch"):
        raise VerificationError(
            "policy-identity", "codescene requires only the main branch policy"
        )
    return identity


def _verify_first_page_stable(
    endpoint: str,
    fetch: Fetch,
    first_page: tuple[int, list[object]],
    first_links: dict[str, int],
) -> None:
    repeated = fetch(f"{endpoint}?per_page={PAGE_SIZE}&page=1")
    if page(repeated) != first_page or links(repeated, endpoint) != first_links:
        raise VerificationError(
            "policy-drift", "deployment branch policies changed during read"
        )


def policies(endpoint: str, fetch: Fetch) -> tuple[int, str, str]:
    """Prove exactly one stable main branch deployment policy."""
    state = _collect_policy_pages(endpoint, fetch)
    identity = _require_main_policy(state)
    if state.first_page is None or state.first_links is None:
        raise VerificationError("policy-count", "deployment branch policy page is missing")
    _verify_first_page_stable(endpoint, fetch, state.first_page, state.first_links)
    return identity


_policy_identity = _policy_identity
_policies = policies
