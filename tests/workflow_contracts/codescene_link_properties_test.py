"""Generated contracts for allowed pagination links and forbidden mutations."""

from __future__ import annotations

import sys
from pathlib import Path
from urllib.parse import urlencode

import pytest
from hypothesis import given
from hypothesis import strategies as st

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from mapsplice_codescene_api import ApiResponse, links
from mapsplice_codescene_models import VerificationError

ENDPOINT = "repositories/1208786150/environments/codescene/deployment-branch-policies"
RELATIONS = ("first", "prev", "next", "last")
PAGES = st.integers(min_value=1, max_value=10**30)
WORDS = st.text(alphabet="abcdefghijklmnopqrstuvwxyz", min_size=1, max_size=30)


def _header(url: str, relation: str) -> str:
    """Render a Link member without altering the generated URL or relation."""
    return f'<{url}>; rel="{relation}"'


def _url(page: int) -> str:
    """Render a valid URL for a generated positive page number."""
    return f"https://api.github.com/{ENDPOINT}?per_page=100&page={page}"


def _parse(header: str) -> dict[str, int]:
    """Exercise the public parser with an otherwise successful API response."""
    return links(ApiResponse(200, {"Link": header}, {}), ENDPOINT)


@given(destinations=st.dictionaries(st.sampled_from(RELATIONS), PAGES))
def test_valid_relation_subsets_preserve_destinations(
    destinations: dict[str, int],
) -> None:
    """Any subset of unique supported relations preserves its positive pages."""
    header = ", ".join(_header(_url(page), rel) for rel, page in destinations.items())
    assert _parse(header) == destinations


@pytest.mark.parametrize(
    "fault",
    [
        "relation",
        "scheme",
        "authority",
        "path",
        "fragment",
        "query-keys",
        "missing-page",
        "missing-size",
        "duplicate-page",
        "duplicate-size",
        "page-size",
        "page-number",
    ],
)
@given(
    relation=st.sampled_from(RELATIONS),
    page=PAGES,
    word=WORDS,
    blank_value=st.booleans(),
    invalid_page=st.one_of(
        st.integers(max_value=0).map(str),
        WORDS,
        st.sampled_from(["", "1.5", "1e3", "--1"]),
    ),
    invalid_size=st.one_of(
        st.integers(max_value=99).map(str),
        st.integers(min_value=101).map(str),
        WORDS,
        st.sampled_from(["", "0100", "+100"]),
    ),
    scheme=st.sampled_from(["http", "ftp", "file", "ssh"]),
    authority=st.sampled_from(
        [
            "github.com",
            "api.github.com:443",
            "api.github.com.evil.test",
            "user@api.github.com",
            "API.GITHUB.COM",
            "evil.test",
        ]
    ),
)
def test_each_forbidden_link_dimension_fails_closed(
    fault: str,
    relation: str,
    page: int,
    word: str,
    blank_value: bool,
    invalid_page: str,
    invalid_size: str,
    scheme: str,
    authority: str,
) -> None:
    """Changing any one required dimension rejects otherwise valid links."""
    route = f"/{ENDPOINT}"
    query = [("per_page", "100"), ("page", str(page))]
    fragment = ""
    actual_scheme, actual_authority = "https", "api.github.com"
    if fault == "relation":
        relation = f"unknown{word}"
    elif fault == "scheme":
        actual_scheme = scheme
    elif fault == "authority":
        actual_authority = authority
    elif fault == "path":
        route += f"/{word}"
    elif fault == "fragment":
        fragment = f"#{word}"
    elif fault == "query-keys":
        query.append((f"extra{word}", "" if blank_value else word))
    elif fault == "missing-page":
        query.pop()
    elif fault == "missing-size":
        query.pop(0)
    elif fault == "duplicate-page":
        query.append(("page", "" if blank_value else str(page)))
    elif fault == "duplicate-size":
        query.append(("per_page", "" if blank_value else "100"))
    elif fault == "page-size":
        query[0] = ("per_page", invalid_size)
    elif fault == "page-number":
        query[1] = ("page", invalid_page)
    url = f"{actual_scheme}://{actual_authority}{route}?{urlencode(query)}{fragment}"
    with pytest.raises(VerificationError) as caught:
        _parse(_header(url, relation))
    assert caught.value.code == "pagination-link"


@given(relation=st.sampled_from(RELATIONS), first=PAGES, second=PAGES)
def test_duplicate_relations_are_rejected(
    relation: str, first: int, second: int
) -> None:
    """Repeated relations cannot silently overwrite a previously parsed page."""
    header = ", ".join(_header(_url(page), relation) for page in (first, second))
    with pytest.raises(VerificationError) as caught:
        _parse(header)
    assert caught.value.code == "pagination-cycle"


@pytest.mark.parametrize("suffix", ["&page=", "&per_page=", "&extra="])
def test_empty_query_values_cannot_hide_duplicates_or_extra_keys(suffix: str) -> None:
    """Blank fields remain visible when validating the complete query shape."""
    with pytest.raises(VerificationError) as caught:
        _parse(_header(_url(1) + suffix, "next"))
    assert caught.value.code == "pagination-link"
