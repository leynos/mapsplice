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


@dataclass(frozen=True)
class _PolicyPage:
    number: int
    page_count: int
    total: int
    seen_count: int
    items: list[object]
    links: dict[str, int]


def _policy_mapping(item: object) -> dict[object, object]:
    if not isinstance(item, dict):
        raise VerificationError(
            "policy-shape", "deployment branch policy entry is malformed"
        )
    return item


def _policy_identifier(value: object) -> int:
    if type(value) is not int:
        raise VerificationError(
            "policy-shape", "deployment branch policy identity is incomplete"
        )
    if value < 1:
        raise VerificationError(
            "policy-shape", "deployment branch policy identity is incomplete"
        )
    return value


def _policy_text(value: object) -> str:
    if not isinstance(value, str):
        raise VerificationError(
            "policy-shape", "deployment branch policy identity is incomplete"
        )
    return value


def _policy_identity(item: object) -> tuple[int, str, str]:
    policy = _policy_mapping(item)
    identifier = _policy_identifier(policy.get("id"))
    name = _policy_text(policy.get("name"))
    kind = _policy_text(policy.get("type"))
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


def _is_sentinel(page_state: _PolicyPage) -> bool:
    return page_state.number > page_state.page_count


def _validate_sentinel(page_state: _PolicyPage) -> None:
    if page_state.items:
        raise VerificationError(
            "policy-count", "deployment branch policy sentinel is not empty"
        )
    if page_state.links.get("next") is not None:
        raise VerificationError(
            "policy-count", "deployment branch policy sentinel is not empty"
        )


def _validate_page_item_count(page_state: _PolicyPage) -> None:
    expected_items = min(PAGE_SIZE, page_state.total - page_state.seen_count)
    if len(page_state.items) != expected_items:
        raise VerificationError(
            "policy-count", "deployment branch policy page is incomplete"
        )


def _validate_intermediate_page_links(page_state: _PolicyPage) -> None:
    if page_state.links.get("next") != page_state.number + 1:
        raise VerificationError(
            "pagination-link", "deployment branch policy next/last page is unproved"
        )
    if page_state.links.get("last") != page_state.page_count:
        raise VerificationError(
            "pagination-link", "deployment branch policy next/last page is unproved"
        )


def _validate_final_page_links(page_state: _PolicyPage) -> None:
    if page_state.links.get("next") is not None:
        raise VerificationError(
            "pagination-cycle", "deployment branch policy Link continues past total"
        )


def _validate_policy_page(page_state: _PolicyPage) -> bool:
    if _is_sentinel(page_state):
        _validate_sentinel(page_state)
        return True
    _validate_page_item_count(page_state)
    if page_state.number < page_state.page_count:
        _validate_intermediate_page_links(page_state)
    else:
        _validate_final_page_links(page_state)
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
        policy_page = _PolicyPage(
            number, page_count, count, len(state.identities), items, page_links
        )
        if _validate_policy_page(policy_page):
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
