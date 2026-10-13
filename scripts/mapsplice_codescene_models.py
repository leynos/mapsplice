"""Typed values shared by CodeScene transport parsing and policy rules."""

from __future__ import annotations

from collections.abc import Iterator, Mapping
from contextlib import contextmanager
from dataclasses import dataclass
from typing import Literal

PAGE_SIZE = 100
MAX_PAGES = 100


AuditOperation = Literal[
    "environment", "repository", "policy_page", "policy_readback", "environment_readback"
]
FailureKind = Literal["timeout", "process_launch", "decode"]


class VerificationError(Exception):
    """Report one bounded reason why CodeScene protection cannot be proved."""

    def __init__(
        self, code: str, message: str, *, failure_kind: FailureKind | None = None
    ) -> None:
        super().__init__(message)
        self.code = code
        self.failure_kind = failure_kind
        self.operation: AuditOperation | None = None
        self.page: int | None = None


@contextmanager
def failure_context(operation: AuditOperation, page: int | None = None) -> Iterator[None]:
    """Attach trusted stage metadata to an audit failure without copying API values."""
    try:
        yield
    except VerificationError as error:
        if error.operation is None:
            error.operation = operation
            error.page = page
        raise


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
