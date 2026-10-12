"""CodeScene domain rules consume typed values independently of GitHub APIs."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from mapsplice_codescene_models import (
    BranchPolicy,
    EnvironmentProtection,
    VerificationError,
)
from mapsplice_codescene_policies import policies, verify_environment_policy


def test_one_main_branch_is_the_permitted_domain_policy() -> None:
    """A complete typed main-only collection has the expected identity."""
    assert policies((BranchPolicy(7, "main", "branch"),)) == (7, "main", "branch")


@pytest.mark.parametrize(
    ("collection", "code"),
    [
        ((), "policy-count"),
        (
            (BranchPolicy(1, "main", "branch"), BranchPolicy(2, "release", "branch")),
            "policy-count",
        ),
        ((BranchPolicy(1, "*", "branch"),), "policy-identity"),
        ((BranchPolicy(1, "main", "tag"),), "policy-identity"),
    ],
    ids=["absent", "additional-branch", "wildcard", "tag"],
)
def test_complete_policy_collections_fail_closed(
    collection: tuple[BranchPolicy, ...], code: str
) -> None:
    """Collection cardinality and rule identity are domain decisions."""
    with pytest.raises(VerificationError) as caught:
        policies(collection)
    assert caught.value.code == code


def test_custom_branch_protection_is_permitted() -> None:
    """The intended typed environment settings pass without API fixtures."""
    verify_environment_policy(EnvironmentProtection("codescene", False, True))


@pytest.mark.parametrize(
    ("protection", "code"),
    [
        (EnvironmentProtection("other", False, True), "environment-shape"),
        (EnvironmentProtection("codescene", None, True), "environment-policy"),
        (EnvironmentProtection("codescene", True, True), "environment-policy"),
        (EnvironmentProtection("codescene", False, False), "environment-policy"),
        (EnvironmentProtection("codescene", False, None), "environment-policy"),
    ],
    ids=[
        "wrong-environment",
        "unknown-protected",
        "protected",
        "custom-off",
        "unknown-custom",
    ],
)
def test_environment_domain_rules_fail_closed(
    protection: EnvironmentProtection, code: str
) -> None:
    """Unknown or unsuitable typed settings cannot prove protection."""
    with pytest.raises(VerificationError) as caught:
        verify_environment_policy(protection)
    assert caught.value.code == code
