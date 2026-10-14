"""Contract tests for the mutation-testing caller workflow.

The executable logic lives in the ``leynos/shared-actions`` reusable
workflow, which carries its own unit and integration tests; mapsplice's
caller is declarative configuration. These tests parse the caller with
PyYAML and assert that it references the correct reusable workflow at a
commit SHA, and that other structural properties (permissions, triggers,
feature-flag configuration) hold, so drift (repointing the pin at a
branch, widening permissions, or losing the feature-flag configuration)
fails CI on the pull request rather than surfacing in a scheduled or
manual run. Dependabot owns the pinned SHA value; these tests do not
assert which SHA is pinned.

Run via ``make test-workflow-contracts``.
"""

from __future__ import annotations

from pathlib import Path

import yaml

WORKFLOW_PATH = (
    Path(__file__).resolve().parents[2]
    / ".github"
    / "workflows"
    / "mutation-testing.yml"
)
REPOSITORY_ROOT = WORKFLOW_PATH.parents[2]
CI_WORKFLOW_PATH = REPOSITORY_ROOT / ".github" / "workflows" / "ci.yml"

#: mapsplice is a single root crate with all mutable source under
#: ``src/`` (the shared workflow's default paths) and no scaffolding
#: modules outside ``#[cfg(test)]``, so the caller mirrors the CI baseline's
#: feature flags and provisions the development linker before mutant builds.
EXPECTED_WITH_KEYS = {
    "extra-args",
    "install-mold",
    "install-mdtablefix",
    "mdtablefix-version",
    "setup-commands",
}


def _load() -> dict[str, object]:
    """Parse the workflow file."""
    return yaml.safe_load(WORKFLOW_PATH.read_text(encoding="utf-8"))


def _ci_markdown_tool_versions() -> tuple[str, str]:
    """Return formatter versions pinned by the standard CI workflow."""
    ci_workflow = yaml.safe_load(CI_WORKFLOW_PATH.read_text(encoding="utf-8"))
    match ci_workflow:
        case {"jobs": {"build-test": dict() as build_test}}:
            pass
        case _:
            raise AssertionError("ci.yml must declare the build-test job")

    match build_test:
        case {"env": {"MARKDOWNLINT_CLI2_VERSION": str(markdownlint_version)}} if (
            markdownlint_version
        ):
            pass
        case _:
            raise AssertionError(
                "ci.yml build-test must declare MARKDOWNLINT_CLI2_VERSION"
            )

    match build_test:
        case {"steps": list(steps)}:
            pass
        case _:
            raise AssertionError("ci.yml build-test must declare a steps list")

    mdtablefix_version = None
    for step in steps:
        match step:
            case {
                "name": "Install mdtablefix",
                "with": {"version": str(version)},
            } if version:
                mdtablefix_version = version
                break
            case {"name": "Install mdtablefix"}:
                raise AssertionError(
                    "ci.yml Install mdtablefix step must declare its version"
                )

    if mdtablefix_version is None:
        raise AssertionError("ci.yml must contain an 'Install mdtablefix' step")
    return mdtablefix_version, markdownlint_version


def _setup_command_index(
    commands: list[str], expected: str, *, exact: bool = False
) -> int:
    """Return the setup-command index or fail with the missing contract."""
    for index, command in enumerate(commands):
        matches = command == expected if exact else expected in command
        if matches:
            return index
    raise AssertionError(f"mutation setup-commands must include {expected!r}")


def _triggers(workflow: dict[str, object]) -> dict[str, object]:
    """Return the ``on:`` mapping (PyYAML parses the bare key as True)."""
    triggers = workflow.get("on", workflow.get(True))
    assert isinstance(triggers, dict), "the workflow must declare an on: mapping"
    return triggers


def _mutation_job(workflow: dict[str, object]) -> dict[str, object]:
    """Return the single calling job."""
    jobs = workflow.get("jobs")
    assert isinstance(jobs, dict), "the workflow must declare a jobs mapping"
    assert jobs, "the workflow must declare at least one job"
    assert list(jobs) == ["mutation"], (
        f"expected a single job named 'mutation', found {sorted(jobs)}"
    )
    return jobs["mutation"]


def test_uses_reference_is_pinned_to_a_commit_sha() -> None:
    """The job must call the shared workflow pinned to a commit SHA.

    Dependabot owns the pinned SHA value, so this test asserts the shape
    of the pin (correct reusable workflow path, full 40-character
    lowercase hex commit SHA) without asserting which SHA is pinned.
    """
    uses = _mutation_job(_load()).get("uses")
    assert uses is not None, "jobs.mutation.uses is missing"
    path, _, ref = uses.partition("@")
    assert path == "leynos/shared-actions/.github/workflows/mutation-cargo.yml", (
        f"jobs.mutation.uses must reference mutation-cargo.yml, got {path!r}"
    )
    assert len(ref) == 40, (
        f"jobs.mutation.uses must pin a full 40-character commit SHA, "
        f"not a branch or tag: {ref!r}"
    )
    assert all(c in "0123456789abcdef" for c in ref), (
        f"jobs.mutation.uses must pin a lowercase hex commit SHA, "
        f"not a branch or tag: {ref!r}"
    )


def test_job_permissions_are_exactly_least_privilege() -> None:
    """The job grants contents: read and id-token: write, nothing broader."""
    permissions = _mutation_job(_load()).get("permissions")
    assert permissions == {"contents": "read", "id-token": "write"}, (
        "jobs.mutation.permissions must be exactly "
        f"{{'contents': 'read', 'id-token': 'write'}}, got {permissions!r}"
    )


def test_workflow_default_permissions_are_empty() -> None:
    """The workflow-level default token scope is empty."""
    workflow = _load()
    assert workflow.get("permissions") == {}, (
        f"top-level permissions must be an empty mapping, got "
        f"{workflow.get('permissions')!r}"
    )


def test_concurrency_serializes_per_ref_without_cancelling() -> None:
    """Runs queue per ref instead of cancelling one another."""
    concurrency = _load().get("concurrency")
    assert isinstance(concurrency, dict), "the workflow must declare concurrency"
    assert concurrency.get("group") == "mutation-testing-${{ github.ref }}", (
        f"concurrency.group must key on the triggering ref, got "
        f"{concurrency.get('group')!r}"
    )
    assert concurrency.get("cancel-in-progress") is False, (
        f"concurrency.cancel-in-progress must be false, got "
        f"{concurrency.get('cancel-in-progress')!r}"
    )


def test_triggers_keep_schedule_and_plain_dispatch() -> None:
    """The daily schedule stays; dispatch has no legacy branch input."""
    triggers = _triggers(_load())
    schedule = triggers.get("schedule")
    assert schedule == [{"cron": "50 3 * * *"}], (
        f"on.schedule must be the daily 03:50 UTC cron, got {schedule!r}"
    )
    assert "workflow_dispatch" in triggers, "on.workflow_dispatch is missing"
    dispatch = triggers.get("workflow_dispatch") or {}
    inputs = dispatch.get("inputs") or {}
    assert "branch" not in inputs, (
        "on.workflow_dispatch must not declare a branch input; the Actions "
        "run-workflow control selects the ref"
    )


def test_with_block_carries_the_caller_configuration() -> None:
    """The caller passes mutant arguments and provisions the development linker."""
    with_block = _mutation_job(_load()).get("with")
    assert isinstance(with_block, dict) and set(with_block) == EXPECTED_WITH_KEYS, (
        f"jobs.mutation.with must contain {EXPECTED_WITH_KEYS!r}, got {with_block!r}"
    )
    assert with_block["extra-args"] == "--all-features", (
        "mutation testing must retain CI's all-features selection"
    )
    assert with_block["install-mold"] == "true", (
        "mutation builds must provision the committed Linux linker"
    )
    assert with_block["install-mdtablefix"] == "true", (
        "the golden baseline must provision mdtablefix before setup commands"
    )
    mdtablefix_version, _ = _ci_markdown_tool_versions()
    assert with_block["mdtablefix-version"] == mdtablefix_version, (
        "mutation testing must use the mdtablefix version pinned by ci.yml"
    )
    setup_commands = with_block["setup-commands"]
    assert isinstance(setup_commands, str), "the build-tool setup hook must be text"
    commands = [line.strip() for line in setup_commands.splitlines()]

    required_commands = (
        "make install-build-tools",
        "make check-build-tools",
        (
            "echo 'RUSTFLAGS=-D warnings -Zthreads=8 -C link-arg=-fuse-ld=mold' "
            '>> "$GITHUB_ENV"'
        ),
    )
    positions = []
    for command in required_commands:
        assert command in commands, f"mutation setup is missing {command!r}"
        positions.append(commands.index(command))
    assert positions == sorted(positions), (
        "mutation setup must install and check build tools before assigning "
        "the development RUSTFLAGS"
    )


def test_setup_commands_install_markdown_tools_at_ci_pins() -> None:
    """Install both formatters and check them before mutation enumeration."""
    with_block = _mutation_job(_load()).get("with")
    assert isinstance(with_block, dict), "jobs.mutation.with must be a mapping"
    setup_commands = with_block.get("setup-commands")
    assert isinstance(setup_commands, str), "the build-tool setup hook must be text"
    mdtablefix_version, markdownlint_version = _ci_markdown_tool_versions()
    commands = [line.strip() for line in setup_commands.splitlines()]

    assert f"markdownlint-cli2@{markdownlint_version}" in setup_commands, (
        "mutation setup must install the markdownlint-cli2 version pinned by ci.yml"
    )
    assert f"MDTABLEFIX_REQUIRED_VERSION={mdtablefix_version}" in setup_commands, (
        "the markdown-tools preflight must check ci.yml's mdtablefix pin"
    )
    assert f"MARKDOWNLINT_CLI2_REQUIRED_VERSION={markdownlint_version}" in setup_commands, (
        "the markdown-tools preflight must check ci.yml's markdownlint-cli2 pin"
    )
    npm_install = _setup_command_index(
        commands, "npm install --global --prefix"
    )
    path_prepend = _setup_command_index(
        commands, 'export PATH="$npm_root/bin:$PATH"', exact=True
    )
    github_path_export = _setup_command_index(
        commands, 'echo "$npm_root/bin" >> "$GITHUB_PATH"', exact=True
    )
    markdown_preflight = _setup_command_index(
        commands, "make check-markdown-tools"
    )
    assert npm_install < path_prepend < github_path_export < markdown_preflight, (
        "install markdownlint-cli2, prepend and persist its bin directory, then run the preflight"
    )
