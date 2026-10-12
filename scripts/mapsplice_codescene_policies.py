"""Verify protected CodeScene settings from typed GitHub response values."""

from __future__ import annotations

from collections.abc import Callable, Mapping
from dataclasses import dataclass, field

from mapsplice_codescene_models import (
    MAX_PAGES,
    PAGE_SIZE,
    BranchPolicy,
    DeploymentPolicyPage,
    EnvironmentProtection,
    VerificationError,
)

PolicyPageReader = Callable[[int], DeploymentPolicyPage]


@dataclass
class _PolicyPages:
    total: int | None = None
    expected_pages: int | None = None
    first_page: DeploymentPolicyPage | None = None
    identities: list[BranchPolicy] = field(default_factory=list)
    identifiers: set[int] = field(default_factory=set)


def _page_count(total: int) -> int:
    if total > MAX_PAGES * PAGE_SIZE:
        raise VerificationError(
            "policy-cap", "deployment branch policy count exceeds audit cap"
        )
    return max(1, (total + PAGE_SIZE - 1) // PAGE_SIZE)


def _remember_total(state: _PolicyPages, page: DeploymentPolicyPage) -> None:
    if state.total is None:
        state.total = page.total
        state.expected_pages = _page_count(page.total)
        state.first_page = page
    elif page.total != state.total:
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
    page_links: Mapping[str, int],
    current_page: int,
    page_count: int,
) -> None:
    for relation, linked_page in page_links.items():
        expected_page = _expected_link_page(relation, current_page, page_count)
        if linked_page != expected_page or linked_page < 1:
            raise VerificationError(
                "pagination-cycle",
                "deployment branch policy Link disagrees with page order",
            )


def _is_sentinel(page: DeploymentPolicyPage, page_count: int) -> bool:
    return page.number > page_count


def _validate_sentinel(page: DeploymentPolicyPage) -> None:
    if page.policies or page.links.get("next") is not None:
        raise VerificationError(
            "policy-count", "deployment branch policy sentinel is not empty"
        )


def _validate_page_item_count(
    page: DeploymentPolicyPage,
    page_count: int,
    seen_count: int,
) -> None:
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
    if page.links.get("next") != page.number + 1:
        raise VerificationError(
            "pagination-link", "deployment branch policy next/last page is unproved"
        )
    if page.links.get("last") != page_count:
        raise VerificationError(
            "pagination-link", "deployment branch policy next/last page is unproved"
        )


def _validate_final_page_links(page: DeploymentPolicyPage) -> None:
    if page.links.get("next") is not None:
        raise VerificationError(
            "pagination-cycle", "deployment branch policy Link continues past total"
        )


def _append_policies(
    state: _PolicyPages,
    policies: tuple[BranchPolicy, ...],
) -> None:
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
    if page.number != requested:
        raise VerificationError(
            "policy-page", "deployment branch policy page number is inconsistent"
        )


def _require_main_policy(state: _PolicyPages) -> tuple[int, str, str]:
    if state.total != len(state.identities) or len(state.identities) != 1:
        raise VerificationError(
            "policy-count", "codescene requires exactly one deployment branch policy"
        )
    policy = state.identities[0]
    if (policy.name, policy.kind) != ("main", "branch"):
        raise VerificationError(
            "policy-identity", "codescene requires only the main branch policy"
        )
    return policy.identifier, policy.name, policy.kind


def _verify_first_page_stable(
    read_page: PolicyPageReader,
    first_page: DeploymentPolicyPage,
) -> None:
    if read_page(1) != first_page:
        raise VerificationError(
            "policy-drift", "deployment branch policies changed during read"
        )


def verify_environment_policy(environment: EnvironmentProtection) -> None:
    """Require the protected CodeScene environment's intended policy settings."""
    if environment.name != "codescene":
        raise VerificationError(
            "environment-shape", "codescene environment policy is missing"
        )
    if environment.protected_branches is not False or (
        environment.custom_branch_policies is not True
    ):
        raise VerificationError(
            "environment-policy",
            "codescene requires protected_branches=false and custom_branch_policies=true",
        )


def policies(read_page: PolicyPageReader) -> tuple[int, str, str]:
    """Prove exactly one stable main branch policy from typed pages."""
    state = _collect_policy_pages(read_page)
    identity = _require_main_policy(state)
    if state.first_page is None:
        raise VerificationError(
            "policy-count", "deployment branch policy page is missing"
        )
    _verify_first_page_stable(read_page, state.first_page)
    return identity
