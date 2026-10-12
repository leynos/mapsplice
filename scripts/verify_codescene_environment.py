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
from mapsplice_codescene_api import (
    MAX_PAGES,
    PAGE_SIZE,
    ApiResponse,
    Fetch,
    deployment_policy_page,
    environment,
    parse_http_response,
    response_object,
)
from mapsplice_codescene_models import EnvironmentProtection, VerificationError
from mapsplice_codescene_policies import policies, verify_environment_policy

REPOSITORY = re.compile(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+\Z")


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
    except (OSError, subprocess.TimeoutExpired) as error:
        raise VerificationError(
            "api-unavailable", "GitHub API request did not complete"
        ) from error
    # In particular, never include stderr: gh can print account details or a
    # server-supplied message that is not suitable for an audit log.
    return parse_http_response(result.stdout, result.returncode)


def _validate_repository_name(repo: str) -> None:
    if REPOSITORY.fullmatch(repo) is None or any(
        segment in {".", ".."} for segment in repo.split("/")
    ):
        raise VerificationError("repository", "repository must be OWNER/NAME")


def _repository_identifier(repo: str, fetch: Fetch) -> int:
    metadata = response_object(fetch(f"repos/{repo}"), "repository")
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
) -> None:
    current = environment(fetch(endpoint))
    verify_environment_policy(current)
    if current != expected:
        raise VerificationError(
            "environment-drift", "codescene protection changed during read"
        )


def verify_environment(repo: str, fetch: Fetch) -> tuple[int, str, str]:
    """Prove one protected environment and exactly one main branch policy."""
    _validate_repository_name(repo)
    environment_endpoint = f"repos/{repo}/environments/codescene"
    original = environment(fetch(environment_endpoint))
    verify_environment_policy(original)
    identifier = _repository_identifier(repo, fetch)
    # GitHub canonicalizes pagination links to its immutable repository ID route.
    branch_endpoint = (
        f"repositories/{identifier}/environments/codescene/deployment-branch-policies"
    )
    policy = policies(
        lambda number: deployment_policy_page(
            fetch(f"{branch_endpoint}?per_page={PAGE_SIZE}&page={number}"),
            branch_endpoint,
            number,
        )
    )
    _verify_environment_unchanged(environment_endpoint, original, fetch)
    return policy


def _policies(endpoint: str, fetch: Fetch) -> tuple[int, str, str]:
    """Adapt existing fixtures to the verifier's typed policy-page interface."""
    return policies(
        lambda number: deployment_policy_page(
            fetch(f"{endpoint}?per_page={PAGE_SIZE}&page={number}"),
            endpoint,
            number,
        )
    )


def main(argv: list[str] | None = None, fetch: Fetch = gh_fetch) -> int:
    """Run the read-only audit and emit only bounded non-secret diagnostics."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True, help="repository as OWNER/NAME")
    args = parser.parse_args(argv)
    try:
        verify_environment(args.repo, fetch)
    except VerificationError as error:
        print(f"codescene environment: {error.code}: {error}", file=sys.stderr)
        return 1
    print("codescene environment: verified main-only deployment branch policy")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
