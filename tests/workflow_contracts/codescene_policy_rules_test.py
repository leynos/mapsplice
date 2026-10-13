"""CodeScene domain rules consume typed values independently of GitHub APIs."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

import verify_codescene_environment as verifier
from mapsplice_codescene_models import (
    AuditOperation,
    BranchPolicy,
    EnvironmentProtection,
    VerificationError,
)
from mapsplice_codescene_policies import (
    CodeScenePolicyError,
    policies,
    verify_environment_policy,
)


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
    with pytest.raises(CodeScenePolicyError) as caught:
        policies(collection)
    assert caught.value.code == code
    assert vars(caught.value) == {"code": code}


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
    with pytest.raises(CodeScenePolicyError) as caught:
        verify_environment_policy(protection)
    assert caught.value.code == code
    assert vars(caught.value) == {"code": code}


@pytest.mark.parametrize(
    "operation", ["environment", "environment_readback", "policy_page"]
)
def test_verifier_translates_domain_failure_with_trusted_context(
    operation: AuditOperation,
) -> None:
    """The verifier adds stage metadata while preserving the domain cause."""
    domain_error = CodeScenePolicyError("policy-count", "one rule required")
    with (
        pytest.raises(VerificationError) as caught,
        verifier._policy_context(operation, page=1),
    ):
        raise domain_error
    assert caught.value.code == domain_error.code
    assert str(caught.value) == str(domain_error)
    assert caught.value.__cause__ is domain_error
    assert caught.value.operation == operation
    assert caught.value.page == 1
    assert caught.value.failure_kind is None
    assert vars(domain_error) == {"code": "policy-count"}
