"""Generated pagination histories for the read-only CodeScene verifier."""

from __future__ import annotations

import sys
from collections.abc import Mapping
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

import pytest
from hypothesis import example, given
from hypothesis import strategies as st
from hypothesis.strategies import SearchStrategy

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

import verify_codescene_environment as verifier

POLICIES = "repositories/1208786150/environments/codescene/deployment-branch-policies"
PAGE_SIZE = verifier.PAGE_SIZE
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
) -> verifier.ApiResponse:
    headers: Mapping[str, str] = {"Link": links} if links else {}
    return verifier.ApiResponse(
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


def _complete_history(total: int) -> dict[int, verifier.ApiResponse]:
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
    responses: dict[int, verifier.ApiResponse],
) -> tuple[verifier.Fetch, list[int]]:
    queued = {page: [response] for page, response in responses.items()}
    queued[1].append(responses[1])
    calls: list[int] = []

    def fetch(endpoint: str) -> verifier.ApiResponse:
        parsed = urlsplit(endpoint)
        assert parsed.path == POLICIES, f"unexpected policy endpoint: {endpoint}"
        query = parse_qs(parsed.query, strict_parsing=True)
        assert query["per_page"] == [str(PAGE_SIZE)]
        page = int(query["page"][0])
        calls.append(page)
        assert queued.get(page), f"unexpected or repeated page request: {page}"
        return queued[page].pop(0)

    return fetch, calls


def _malformed_history_strategy() -> SearchStrategy[
    tuple[int, str, int, str | None, int]
]:
    @st.composite
    def histories(
        draw: st.DrawFn,
    ) -> tuple[int, str, int, str | None, int]:
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
        return total, fault, page, relation, delta

    return histories()


def _malformed_history(
    total: int,
    fault: str,
    fault_page: int,
    fault_relation: str | None,
    count_delta: int,
) -> dict[int, verifier.ApiResponse]:
    pages = _page_count(total)
    responses: dict[int, verifier.ApiResponse] = {}
    for page in range(1, pages + 1):
        items = _page_items(total, page)
        count = total
        relations = _page_relations(page, pages)
        links = _link(*relations)
        if page == fault_page:
            if fault == "short-page":
                items.pop()
            elif fault == "missing-next":
                links = _link(*(pair for pair in relations if pair[1] != "next"))
            elif fault == "malformed-link":
                links = "not a GitHub Link header"
            elif fault == "wrong-link":
                links = _link(
                    *(
                        (target + 1, name) if name == fault_relation else (target, name)
                        for target, name in relations
                    )
                )
            elif fault == "count-drift":
                count += count_delta
            elif fault == "cross-page-duplicate":
                items[0] = _policy((page - 1) * PAGE_SIZE)
            elif fault == "malformed-entry":
                items[0] = {"id": "not-an-integer", "name": "branch"}
        responses[page] = _response(items, count, links)

    sentinel = pages + 1
    sentinel_items: list[object] = []
    sentinel_relations = _page_relations(sentinel, pages)
    sentinel_links = _link(*sentinel_relations)
    if fault_page == sentinel:
        if fault == "malformed-link":
            sentinel_links = "not a GitHub Link header"
        elif fault == "wrong-link":
            sentinel_links = _link(
                *(
                    (target + 1, name) if name == fault_relation else (target, name)
                    for target, name in sentinel_relations
                )
            )
        elif fault == "sentinel-next":
            sentinel_links = _link(*sentinel_relations, (sentinel + 1, "next"))
        elif fault == "nonempty-sentinel":
            sentinel_items = [_policy(total + 1)]
    responses[sentinel] = _response(sentinel_items, total, sentinel_links)
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
    """Complete API pages reach the sentinel before policy cardinality is judged."""
    responses = _complete_history(total)
    fetch, calls = _paged_fetch(responses)
    pages = _page_count(total)

    if total == 1:
        assert verifier._policies(POLICIES, fetch) == (1, "main", "branch")
        assert calls == [1, 2, 1]
    else:
        with pytest.raises(verifier.VerificationError) as caught:
            verifier._policies(POLICIES, fetch)
        assert caught.value.code == "policy-count"
        assert calls == list(range(1, pages + 2))


@example(case=(101, "short-page", 1, None, 0))
@example(case=(200, "cross-page-duplicate", 2, None, 0))
@example(case=(201, "count-drift", 3, None, -1))
@example(case=(101, "sentinel-next", 3, None, 0))
@given(case=_malformed_history_strategy())
def test_generated_malformed_pagination_histories_fail_closed(
    case: tuple[int, str, int, str | None, int],
) -> None:
    """Generated page faults fail closed at the page where they appear."""
    total, fault, fault_page, relation, delta = case
    responses = _malformed_history(total, fault, fault_page, relation, delta)
    fetch, calls = _paged_fetch(responses)

    with pytest.raises(verifier.VerificationError) as caught:
        verifier._policies(POLICIES, fetch)

    assert caught.value.code == FAULT_CODES[fault]
    assert calls == list(range(1, fault_page + 1))
