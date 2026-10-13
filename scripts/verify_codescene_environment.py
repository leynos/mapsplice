"""Read GitHub's CodeScene environment protection without changing it.

Example: ``python3 scripts/verify_codescene_environment.py --repo leynos/mapsplice``
returns success only after the environment and its sole main-branch policy have
been read back. The API reads are not an atomic snapshot; a second read detects
ordinary changes while the check runs.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from collections.abc import Iterator
from contextlib import contextmanager

from mapsplice_codescene_api import (
    ApiResponse,
    Fetch,
    deployment_policies,
    environment,
    parse_http_response,
    response_object,
)
from mapsplice_codescene_models import (
    AuditOperation,
    EnvironmentProtection,
    VerificationError,
    failure_context,
)
from mapsplice_codescene_observability import AuditSession, audit_stage
from mapsplice_codescene_policies import (
    CodeScenePolicyError,
    policies,
    verify_environment_policy,
)

REPOSITORY = re.compile(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+\Z")


@contextmanager
def _policy_context(
    operation: AuditOperation, page: int | None = None
) -> Iterator[None]:
    """Translate domain failures before attaching trusted verifier context."""
    with failure_context(operation, page=page):
        try:
            yield
        except CodeScenePolicyError as error:
            raise VerificationError(error.code, str(error)) from error


def gh_fetch(endpoint: str) -> ApiResponse:
    """Make exactly one read-only API request through the configured gh login."""
    try:
        result = subprocess.run(
            [
                "gh",
                "api",
                "--hostname",
                "github.com",
                "--include",
                "--method",
                "GET",
                endpoint,
            ],
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
    except subprocess.TimeoutExpired as error:
        raise VerificationError(
            "api-unavailable",
            "GitHub API request did not complete",
            failure_kind="timeout",
        ) from error
    except OSError as error:
        raise VerificationError(
            "api-unavailable",
            "GitHub API request did not complete",
            failure_kind="process_launch",
        ) from error
    except UnicodeDecodeError as error:
        raise VerificationError(
            "api-unavailable",
            "GitHub API request did not complete",
            failure_kind="decode",
        ) from error
    # In particular, never include stderr: gh can print account details or a
    # server-supplied message that is not suitable for an audit log.
    return parse_http_response(result.stdout, result.returncode)


def _validate_repository_name(repo: str) -> None:
    """Reject repository names outside the supported owner/name form."""
    if REPOSITORY.fullmatch(repo) is None or any(
        segment in {".", ".."} for segment in repo.split("/")
    ):
        raise VerificationError("repository", "repository must be OWNER/NAME")


def _repository_identifier(
    repo: str, fetch: Fetch, audit: AuditSession | None = None
) -> int:
    """Fetch and verify the immutable GitHub repository identifier."""
    metadata = response_object(
        _observed_fetch(fetch, f"repos/{repo}", "repository", audit), "repository"
    )
    identifier = metadata.get("id")
    if not _repository_metadata_matches(metadata, identifier, repo):
        raise VerificationError(
            "repository-identity", "repository identity is unproved"
        )
    return identifier


def _repository_metadata_matches(
    metadata: object, identifier: object, expected_name: str
) -> bool:
    return (
        isinstance(metadata, dict)
        and type(identifier) is int
        and identifier > 0
        and metadata.get("full_name") == expected_name
    )


def _verify_environment_unchanged(
    endpoint: str,
    expected: EnvironmentProtection,
    fetch: Fetch,
    audit: AuditSession | None = None,
) -> None:
    current = environment(
        _observed_fetch(fetch, endpoint, "environment_readback", audit)
    )
    verify_environment_policy(current)
    if current != expected:
        raise VerificationError(
            "environment-drift", "codescene protection changed during read"
        )


def _observed_fetch(
    fetch: Fetch, endpoint: str, operation: AuditOperation, audit: AuditSession | None
) -> ApiResponse:
    """Forward requests without deriving telemetry labels from their endpoint."""
    if audit is None:
        return fetch(endpoint)
    return audit.request(fetch, endpoint, operation)


def verify_environment(
    repo: str, fetch: Fetch, audit: AuditSession | None = None
) -> tuple[int, str, str]:
    """Prove one protected environment and exactly one main branch policy."""
    with audit_stage(audit, "repository"), failure_context("repository"):
        _validate_repository_name(repo)
    environment_endpoint = f"repos/{repo}/environments/codescene"
    with audit_stage(audit, "environment"), _policy_context("environment"):
        original = environment(
            _observed_fetch(fetch, environment_endpoint, "environment", audit)
        )
        verify_environment_policy(original)
    with audit_stage(audit, "repository"), failure_context("repository"):
        identifier = _repository_identifier(repo, fetch, audit)
    # GitHub canonicalizes pagination links to its immutable repository ID route.
    branch_endpoint = (
        f"repositories/{identifier}/environments/codescene/deployment-branch-policies"
    )
    collection = deployment_policies(branch_endpoint, fetch, audit)
    with (
        audit_stage(audit, "policy_page", page=1),
        _policy_context("policy_page", page=1),
    ):
        policy = policies(collection)
    with (
        audit_stage(audit, "environment_readback"),
        _policy_context("environment_readback"),
    ):
        _verify_environment_unchanged(environment_endpoint, original, fetch, audit)
    return policy


def main(argv: list[str] | None = None, fetch: Fetch = gh_fetch) -> int:
    """Run the read-only audit and emit only bounded non-secret diagnostics."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True, help="repository as OWNER/NAME")
    args = parser.parse_args(argv)
    audit = AuditSession()
    try:
        with audit.run():
            verify_environment(args.repo, fetch, audit)
    except VerificationError as error:
        context = [f"operation={error.operation}"] if error.operation else []
        if error.page is not None:
            context.append(f"page={error.page}")
        if error.failure_kind is not None:
            context.append(f"failure_kind={error.failure_kind}")
        suffix = f" ({' '.join(context)})" if context else ""
        print(f"codescene environment: {error.code}: {error}{suffix}", file=sys.stderr)
        return 1
    print("codescene environment: verified main-only deployment branch policy")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
