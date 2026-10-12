"""Enforce CodeScene protection rules on complete typed domain values."""

from __future__ import annotations

from mapsplice_codescene_models import (
    BranchPolicy,
    EnvironmentProtection,
    VerificationError,
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


def policies(collection: tuple[BranchPolicy, ...]) -> tuple[int, str, str]:
    """Require one main branch from a complete verified policy collection.

    For example, ``policies((BranchPolicy(1, "main", "branch"),))`` returns
    ``(1, "main", "branch")``; extra branches and tag rules are rejected.
    """
    if len(collection) != 1:
        raise VerificationError(
            "policy-count", "codescene requires exactly one deployment branch policy"
        )
    policy = collection[0]
    if (policy.name, policy.kind) != ("main", "branch"):
        raise VerificationError(
            "policy-identity", "codescene requires only the main branch policy"
        )
    return policy.identifier, policy.name, policy.kind
