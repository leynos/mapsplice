"""Typed values shared by CodeScene transport parsing and policy rules."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass

PAGE_SIZE = 100
MAX_PAGES = 100


class VerificationError(Exception):
    """Report one bounded reason why CodeScene protection cannot be proved."""

    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code


@dataclass(frozen=True)
class EnvironmentProtection:
    """Decoded GitHub environment protection settings."""

    name: str
    protected_branches: bool | None
    custom_branch_policies: bool | None


@dataclass(frozen=True)
class BranchPolicy:
    """One decoded GitHub deployment branch policy."""

    identifier: int
    name: str
    kind: str


@dataclass(frozen=True)
class DeploymentPolicyPage:
    """One decoded policy page and its validated pagination destinations."""

    number: int
    total: int
    policies: tuple[BranchPolicy, ...]
    links: Mapping[str, int]
