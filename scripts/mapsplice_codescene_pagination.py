"""Collect a complete stable GitHub deployment-policy snapshot.

Pagination and API consistency belong to this adapter; domain rules receive
only the resulting immutable branch-policy values.
"""

from __future__ import annotations

from collections.abc import Callable, Mapping
from dataclasses import dataclass, field

from mapsplice_codescene_models import (
    MAX_PAGES,
    PAGE_SIZE,
    BranchPolicy,
    DeploymentPolicyPage,
    VerificationError,
)

PolicyPageReader = Callable[[int], DeploymentPolicyPage]


@dataclass
class _PolicyPages:
    """Track completeness and identities while reading one API snapshot."""

    total: int | None = None
    expected_pages: int | None = None
    first_page: DeploymentPolicyPage | None = None
    identities: list[BranchPolicy] = field(default_factory=list)
    identifiers: set[int] = field(default_factory=set)


def _page_count(total: int) -> int:
    """Bound the declared count before deriving its required page count."""
    if total > MAX_PAGES * PAGE_SIZE:
        raise VerificationError(
            "policy-cap", "deployment branch policy count exceeds audit cap"
        )
    return max(1, (total + PAGE_SIZE - 1) // PAGE_SIZE)


def _remember_total(state: _PolicyPages, page: DeploymentPolicyPage) -> None:
    """Record the initial count and reject changes across subsequent pages."""
    if state.total is None:
        state.total = page.total
        state.expected_pages = _page_count(page.total)
        state.first_page = page
    elif page.total != state.total:
        raise VerificationError(
            "policy-drift", "deployment branch policy count changed during read"
        )


def _expected_link_page(relation: str, current_page: int, page_count: int) -> int:
    """Resolve each validated relation to its only permitted destination."""
    return {
        "first": 1,
        "prev": current_page - 1,
        "next": current_page + 1,
        "last": page_count,
    }[relation]


def _validate_link_order(
    page_links: Mapping[str, int],
    current_page: int,
    page_count: int,
) -> None:
    """Require all parsed pagination links to agree with sequential traversal."""
    for relation, linked_page in page_links.items():
        expected_page = _expected_link_page(relation, current_page, page_count)
        if linked_page != expected_page or linked_page < 1:
            raise VerificationError(
                "pagination-cycle",
                "deployment branch policy Link disagrees with page order",
            )


def _is_sentinel(page: DeploymentPolicyPage, page_count: int) -> bool:
    """Identify the extra page required to disprove hidden policies."""
    return page.number > page_count


def _validate_sentinel(page: DeploymentPolicyPage) -> None:
    """Require the sentinel to contain neither entries nor a continuation."""
    if page.policies or page.links.get("next") is not None:
        raise VerificationError(
            "policy-count", "deployment branch policy sentinel is not empty"
        )


def _validate_page_item_count(
    page: DeploymentPolicyPage,
    page_count: int,
    seen_count: int,
) -> None:
    """Require full intermediate pages and the declared remaining final entries."""
    expected_items = min(PAGE_SIZE, page.total - seen_count)
    if len(page.policies) != expected_items:
        raise VerificationError(
            "policy-count", "deployment branch policy page is incomplete"
        )
    if page.number < page_count:
        _validate_intermediate_page_links(page, page_count)
    else:
        _validate_final_page_links(page)


def _validate_intermediate_page_links(
    page: DeploymentPolicyPage,
    page_count: int,
) -> None:
    """Require next and last links to prove the remaining page traversal."""
    if page.links.get("next") != page.number + 1:
        raise VerificationError(
            "pagination-link", "deployment branch policy next/last page is unproved"
        )
    if page.links.get("last") != page_count:
        raise VerificationError(
            "pagination-link", "deployment branch policy next/last page is unproved"
        )


def _validate_final_page_links(page: DeploymentPolicyPage) -> None:
    """Reject continuation links after the declared final content page."""
    if page.links.get("next") is not None:
        raise VerificationError(
            "pagination-cycle", "deployment branch policy Link continues past total"
        )


def _append_policies(
    state: _PolicyPages,
    policies: tuple[BranchPolicy, ...],
) -> None:
    """Accumulate well-formed unique identities across the entire snapshot."""
    for policy in policies:
        if policy.identifier < 1:
            raise VerificationError(
                "policy-shape", "deployment branch policy identity is incomplete"
            )
        if policy.identifier in state.identifiers:
            raise VerificationError(
                "pagination-cycle", "deployment branch policy is repeated"
            )
        state.identifiers.add(policy.identifier)
        state.identities.append(policy)


def _collect_policy_pages(read_page: PolicyPageReader) -> _PolicyPages:
    """Read bounded sequential pages and stop only at a valid sentinel."""
    state = _PolicyPages()
    for number in range(1, MAX_PAGES + 2):
        current = read_page(number)
        _require_requested_page(current, number)
        _remember_total(state, current)
        page_count = state.expected_pages
        assert page_count is not None
        _validate_link_order(current.links, number, page_count)
        if _is_sentinel(current, page_count):
            _validate_sentinel(current)
            break
        _validate_page_item_count(current, page_count, len(state.identities))
        _append_policies(state, current.policies)
    else:
        raise VerificationError(
            "policy-cap", "deployment branch policy pagination exceeded audit cap"
        )
    return state


def _require_requested_page(page: DeploymentPolicyPage, requested: int) -> None:
    """Reject injected readers that return a page other than the request."""
    if page.number != requested:
        raise VerificationError(
            "policy-page", "deployment branch policy page number is inconsistent"
        )


def _verify_first_page_stable(
    read_page: PolicyPageReader,
    first_page: DeploymentPolicyPage,
) -> None:
    """Detect policy or pagination changes with a final first-page readback."""
    if read_page(1) != first_page:
        raise VerificationError(
            "policy-drift", "deployment branch policies changed during read"
        )


def collect_policies(read_page: PolicyPageReader) -> tuple[BranchPolicy, ...]:
    """Require complete bounded pages, an empty sentinel, and stable readback."""
    state = _collect_policy_pages(read_page)
    if state.total != len(state.identities):
        raise VerificationError(
            "policy-count", "deployment branch policy collection is incomplete"
        )
    if state.first_page is None:
        raise VerificationError(
            "policy-count", "deployment branch policy page is missing"
        )
    _verify_first_page_stable(read_page, state.first_page)
    return tuple(state.identities)
