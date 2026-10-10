"""Generated pagination histories for the read-only CodeScene verifier."""

from __future__ import annotations

import sys
from collections.abc import Mapping
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

import pytest
from hypothesis import example, given, settings
from hypothesis import strategies as st

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

import verify_codescene_environment as verifier

POLICIES = (
    "repositories/1208786150/environments/codescene/"
    "deployment-branch-policies"
)
PAGE_SIZE = verifier.PAGE_SIZE


def _page_url(page: int) -> str:
    return (
        f"https://api.github.com/{POLICIES}"
        f"?per_page={PAGE_SIZE}&page={page}"
    )


def _link(*relations: tuple[int, str]) -> str:
    return ", ".join(
        f'<{_page_url(page)}>; rel="{relation}"'
        for page, relation in relations
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


@pytest.mark.parametrize(
    ("history", "expected_code"),
    [
        ("cross-page-duplicate", "pagination-cycle"),
        ("missing-next", "pagination-link"),
        ("malformed-later-link", "pagination-link"),
        ("wrong-next", "pagination-cycle"),
        ("sentinel-next", "policy-count"),
        ("count-drift", "policy-drift"),
        ("short-page", "policy-count"),
        ("malformed-entry", "policy-shape"),
    ],
)
@settings(max_examples=8, deadline=None)
@example(tail_count=1)
@example(tail_count=100)
@given(tail_count=st.integers(min_value=1, max_value=100))
def test_generated_malformed_pagination_histories_fail_closed(
    history: str,
    expected_code: str,
    tail_count: int,
) -> None:
    """Adversarial page histories never produce a successful policy proof."""
    total = 200 + tail_count
    first_items: list[object] = [
        {"id": identifier, "name": f"branch-{identifier}", "type": "branch"}
        for identifier in range(1, PAGE_SIZE + 1)
    ]
    second_items: list[object] = [
        {
            "id": PAGE_SIZE + identifier,
            "name": f"branch-{PAGE_SIZE + identifier}",
            "type": "branch",
        }
        for identifier in range(1, PAGE_SIZE + 1)
    ]
    third_items: list[object] = [
        {
            "id": 2 * PAGE_SIZE + identifier,
            "name": f"branch-{2 * PAGE_SIZE + identifier}",
            "type": "branch",
        }
        for identifier in range(1, tail_count + 1)
    ]
    page_two_links = _link((1, "prev"), (3, "next"), (3, "last"))
    page_three_links = _link((1, "first"), (2, "prev"), (3, "last"))
    page_two_total = total
    sentinel_links = ""

    if history == "cross-page-duplicate":
        second_items[0] = first_items[-1]
    elif history == "missing-next":
        page_two_links = _link((1, "prev"), (3, "last"))
    elif history == "malformed-later-link":
        page_two_links = "not a GitHub Link header"
    elif history == "wrong-next":
        page_two_links = _link((1, "prev"), (4, "next"), (3, "last"))
    elif history == "sentinel-next":
        sentinel_links = _link((5, "next"))
    elif history == "count-drift":
        page_two_total += 1
    elif history == "short-page":
        second_items.pop()
    elif history == "malformed-entry":
        second_items[0] = {"id": "not-an-integer", "name": "branch"}

    responses = {
        1: _response(
            first_items,
            total,
            _link((2, "next"), (3, "last")),
        ),
        2: _response(second_items, page_two_total, page_two_links),
        3: _response(third_items, total, page_three_links),
        4: _response([], total, sentinel_links),
    }
    calls: list[int] = []

    def fetch(endpoint: str) -> verifier.ApiResponse:
        parsed = urlsplit(endpoint)
        assert parsed.path == POLICIES
        query = parse_qs(parsed.query, strict_parsing=True)
        page = int(query["page"][0])
        calls.append(page)
        return responses[page]

    with pytest.raises(verifier.VerificationError) as caught:
        verifier._policies(POLICIES, fetch)

    assert caught.value.code == expected_code
    assert calls[:2] == [1, 2]
