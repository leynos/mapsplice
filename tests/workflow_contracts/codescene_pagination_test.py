"""Generated pagination histories for the read-only CodeScene adapter."""

from __future__ import annotations

import sys
from collections.abc import Callable, Mapping
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

import pytest
from hypothesis import example, given
from hypothesis import strategies as st
from hypothesis.strategies import SearchStrategy

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

import mapsplice_codescene_api as adapter
from mapsplice_codescene_models import (
    BranchPolicy,
    DeploymentPolicyPage,
    VerificationError,
)
from mapsplice_codescene_pagination import collect_policies

POLICIES = "repositories/1208786150/environments/codescene/deployment-branch-policies"
PAGE_SIZE = adapter.PAGE_SIZE
MAX_GENERATED_POLICIES = 500
FAULT_CODES = {
    "short-page": "policy-count",
    "missing-next": "pagination-link",
    "malformed-link": "pagination-link",
    "wrong-link": "pagination-cycle",
    "count-drift": "policy-drift",
    "cross-page-duplicate": "pagination-cycle",
    "sentinel-next": "policy-count",
    "nonempty-sentinel": "policy-count",
    "malformed-entry": "policy-shape",
}


def _page_url(page: int) -> str:
    return f"https://api.github.com/{POLICIES}?per_page={PAGE_SIZE}&page={page}"


def _link(*relations: tuple[int, str]) -> str:
    return ", ".join(
        f'<{_page_url(page)}>; rel="{relation}"' for page, relation in relations
    )


def _response(
    items: list[object],
    total: int,
    links: str = "",
) -> adapter.ApiResponse:
    headers: Mapping[str, str] = {"Link": links} if links else {}
    return adapter.ApiResponse(
        200,
        headers,
        {"total_count": total, "branch_policies": items},
    )


def _page_count(total: int) -> int:
    return max(1, (total + PAGE_SIZE - 1) // PAGE_SIZE)


def _page_relations(page: int, pages: int) -> list[tuple[int, str]]:
    if page > pages:
        return [(1, "first"), (pages, "prev"), (pages, "last")]
    relations = []
    if page > 1:
        relations.extend(((1, "first"), (page - 1, "prev")))
    if page < pages:
        relations.append((page + 1, "next"))
    if pages > 1:
        relations.append((pages, "last"))
    return relations


def _policy(identifier: int) -> dict[str, object]:
    name = "main" if identifier == 1 else f"release-{identifier}"
    return {"id": identifier, "name": name, "type": "branch"}


def _page_items(total: int, page: int) -> list[object]:
    start = (page - 1) * PAGE_SIZE
    stop = min(page * PAGE_SIZE, total)
    return [_policy(identifier) for identifier in range(start + 1, stop + 1)]


def _complete_history(total: int) -> dict[int, adapter.ApiResponse]:
    pages = _page_count(total)
    responses = {
        page: _response(
            _page_items(total, page),
            total,
            _link(*_page_relations(page, pages)),
        )
        for page in range(1, pages + 1)
    }
    responses[pages + 1] = _response(
        [], total, _link(*_page_relations(pages + 1, pages))
    )
    return responses


def _paged_fetch(
    responses: dict[int, adapter.ApiResponse],
) -> tuple[adapter.Fetch, list[int]]:
    queued = {page: [response] for page, response in responses.items()}
    queued[1].append(responses[1])
    calls: list[int] = []

    def fetch(endpoint: str) -> adapter.ApiResponse:
        parsed = urlsplit(endpoint)
        assert parsed.path == POLICIES, f"unexpected policy endpoint: {endpoint}"
        query = parse_qs(parsed.query, strict_parsing=True)
        assert query["per_page"] == [str(PAGE_SIZE)]
        page = int(query["page"][0])
        calls.append(page)
        assert queued.get(page), f"unexpected or repeated page request: {page}"
        return queued[page].pop(0)

    return fetch, calls


@dataclass(frozen=True)
class MalformedHistory:
    total: int
    fault: str
    fault_page: int
    fault_relation: str | None
    count_delta: int


@dataclass
class PageState:
    count: int
    items: list[object]
    relations: list[tuple[int, str]]
    links: str
    page_number: int


def _malformed_history_strategy() -> SearchStrategy[MalformedHistory]:
    @st.composite
    def histories(
        draw: st.DrawFn,
    ) -> MalformedHistory:
        total = draw(
            st.integers(min_value=PAGE_SIZE + 1, max_value=MAX_GENERATED_POLICIES)
        )
        pages = _page_count(total)
        fault = draw(st.sampled_from(tuple(FAULT_CODES)))
        relation = None
        delta = 0
        if fault == "missing-next":
            page = draw(st.integers(min_value=1, max_value=pages - 1))
        elif fault in {"count-drift", "cross-page-duplicate"}:
            page = draw(st.integers(min_value=2, max_value=pages))
            if fault == "count-drift":
                delta = draw(st.sampled_from((-2, -1, 1, 2)))
        elif fault in {"sentinel-next", "nonempty-sentinel"}:
            page = pages + 1
        elif fault == "wrong-link":
            page = draw(st.integers(min_value=1, max_value=pages + 1))
            valid_relations = _page_relations(page, pages)
            relation = draw(st.sampled_from([name for _, name in valid_relations]))
        else:
            page = draw(st.integers(min_value=1, max_value=pages))
        return MalformedHistory(total, fault, page, relation, delta)

    return histories()


def _short_page(state: PageState, _: MalformedHistory) -> None:
    state.items.pop()


def _missing_next(state: PageState, _: MalformedHistory) -> None:
    state.links = _link(*(pair for pair in state.relations if pair[1] != "next"))


def _malformed_link(state: PageState, _: MalformedHistory) -> None:
    state.links = "not a GitHub Link header"


def _wrong_link(state: PageState, case: MalformedHistory) -> None:
    state.links = _link(
        *(
            (target + 1, name) if name == case.fault_relation else (target, name)
            for target, name in state.relations
        )
    )


def _count_drift(state: PageState, case: MalformedHistory) -> None:
    state.count += case.count_delta


def _cross_page_duplicate(state: PageState, _: MalformedHistory) -> None:
    state.items[0] = _policy((state.page_number - 1) * PAGE_SIZE)


def _malformed_entry(state: PageState, _: MalformedHistory) -> None:
    state.items[0] = {"id": "not-an-integer", "name": "branch"}


PAGE_FAULTS: dict[str, Callable[[PageState, MalformedHistory], None]] = {
    "short-page": _short_page,
    "missing-next": _missing_next,
    "malformed-link": _malformed_link,
    "wrong-link": _wrong_link,
    "count-drift": _count_drift,
    "cross-page-duplicate": _cross_page_duplicate,
    "malformed-entry": _malformed_entry,
}


def _apply_page_fault(state: PageState, case: MalformedHistory, page: int) -> None:
    if page != case.fault_page:
        return
    mutation = PAGE_FAULTS.get(case.fault)
    if mutation is not None:
        mutation(state, case)


def _apply_sentinel_fault(
    state: PageState, case: MalformedHistory, sentinel: int
) -> None:
    if case.fault_page != sentinel:
        return
    if case.fault == "sentinel-next":
        state.links = _link(*state.relations, (sentinel + 1, "next"))
    elif case.fault == "nonempty-sentinel":
        state.items.append(_policy(case.total + 1))
    elif case.fault == "malformed-link":
        _malformed_link(state, case)
    elif case.fault == "wrong-link":
        _wrong_link(state, case)


def _malformed_history(case: MalformedHistory) -> dict[int, adapter.ApiResponse]:
    pages = _page_count(case.total)
    responses: dict[int, adapter.ApiResponse] = {}
    for page in range(1, pages + 1):
        state = PageState(
            case.total,
            _page_items(case.total, page),
            _page_relations(page, pages),
            "",
            page,
        )
        state.links = _link(*state.relations)
        _apply_page_fault(state, case, page)
        responses[page] = _response(state.items, state.count, state.links)

    sentinel = pages + 1
    sentinel_state = PageState(
        case.total, [], _page_relations(sentinel, pages), "", sentinel
    )
    sentinel_state.links = _link(*sentinel_state.relations)
    _apply_sentinel_fault(sentinel_state, case, sentinel)
    responses[sentinel] = _response(
        sentinel_state.items, sentinel_state.count, sentinel_state.links
    )
    return responses


@example(total=0)
@example(total=1)
@example(total=100)
@example(total=101)
@example(total=200)
@example(total=201)
@given(total=st.integers(min_value=0, max_value=MAX_GENERATED_POLICIES))
def test_generated_complete_pagination_histories_are_read_in_order(
    total: int,
) -> None:
    """The adapter returns every typed entry after the sentinel and stable readback."""
    responses = _complete_history(total)
    fetch, calls = _paged_fetch(responses)
    pages = _page_count(total)

    result = adapter.deployment_policies(POLICIES, fetch)
    assert result == tuple(
        BranchPolicy(
            identifier, "main" if identifier == 1 else f"release-{identifier}", "branch"
        )
        for identifier in range(1, total + 1)
    )
    assert calls == [*range(1, pages + 2), 1]


@example(case=MalformedHistory(101, "short-page", 1, None, 0))
@example(case=MalformedHistory(200, "cross-page-duplicate", 2, None, 0))
@example(case=MalformedHistory(201, "count-drift", 3, None, -1))
@example(case=MalformedHistory(101, "sentinel-next", 3, None, 0))
@given(case=_malformed_history_strategy())
def test_generated_malformed_pagination_histories_fail_closed(
    case: MalformedHistory,
) -> None:
    """Generated page faults fail closed at the page where they appear."""
    responses = _malformed_history(case)
    fetch, calls = _paged_fetch(responses)

    with pytest.raises(VerificationError) as caught:
        adapter.deployment_policies(POLICIES, fetch)

    assert caught.value.code == FAULT_CODES[case.fault]
    assert caught.value.operation == "policy_page"
    assert caught.value.page == case.fault_page
    assert calls == list(range(1, case.fault_page + 1))


def test_page_reader_must_return_the_requested_page() -> None:
    """The pagination adapter rejects an inconsistent injected page reader."""
    with pytest.raises(VerificationError) as caught:
        collect_policies(lambda _: DeploymentPolicyPage(2, 0, (), {}))
    assert caught.value.code == "policy-page"


@pytest.mark.parametrize(
    "item",
    [
        None,
        {"id": True, "name": "main", "type": "branch"},
        {"id": 1, "name": "main"},
        {"id": 1, "name": "main", "type": 7},
    ],
    ids=["not-an-object", "boolean-identifier", "missing-kind", "non-string-kind"],
)
def test_adapter_rejects_malformed_policy_mapping(item: object) -> None:
    """Raw JSON identity fields must map to well-formed typed policy values."""
    with pytest.raises(VerificationError) as caught:
        adapter.deployment_policy_page(_response([item], 1), POLICIES, 1)
    assert caught.value.code == "policy-shape"


def test_page_two_failure_retains_the_requested_page_context() -> None:
    """Collector context must retain page two rather than replacing it with readback."""
    responses = _complete_history(101)
    responses[2] = _response([], 101)
    fetch, calls = _paged_fetch(responses)

    with pytest.raises(VerificationError) as caught:
        adapter.deployment_policies(POLICIES, fetch)

    assert caught.value.code == "policy-count"
    assert caught.value.operation == "policy_page"
    assert caught.value.page == 2
    assert calls == [1, 2]
