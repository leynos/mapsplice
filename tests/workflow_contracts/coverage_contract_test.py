"""Mutation tests for CV-005 coverage ownership and token isolation."""

from __future__ import annotations

from copy import deepcopy

import pytest
from coverage_contract import parse_workflow, pr_reachable, read_workflows, validate


def _workflow_shape_fixture() -> dict[str, dict[object, object]]:
    """Model the intended YAML declaration, without proving live protection."""
    workflows = deepcopy(read_workflows())
    workflows["coverage-main.yml"]["jobs"]["coverage-publisher"]["environment"] = (
        "codescene"
    )
    return workflows


def _main_steps(workflows: dict[str, dict[object, object]]) -> list[dict[str, object]]:
    return workflows["coverage-main.yml"]["jobs"]["coverage-publisher"]["steps"]


def _pr_steps(workflows: dict[str, dict[object, object]]) -> list[dict[str, object]]:
    return workflows["ci.yml"]["jobs"]["build-test"]["steps"]


def test_workflow_shape_with_publisher_declaration() -> None:
    """The intended workflow shape passes without claiming API provision."""
    validate(_workflow_shape_fixture())


def test_live_coverage_contract() -> None:
    """The checked-out workflows must meet the complete CV-005 contract."""
    validate(read_workflows())


@pytest.mark.parametrize(
    ("source", "reason"),
    [
        ("name: A\non: push\non: pull_request\njobs: {x: {steps: []}}", "duplicate"),
        ("name: A\n'on': push\non: pull_request\njobs: {x: {steps: []}}", "ambiguous"),
        ("", "mapping"),
    ],
)
def test_strict_yaml_rejects_ambiguous_or_empty_input(source: str, reason: str) -> None:
    with pytest.raises((AssertionError, ValueError), match=reason):
        parse_workflow(source)


@pytest.mark.parametrize(
    ("mutation", "reason"),
    [
        ("pr_token", "PR-reachable CodeScene"),
        ("pr_environment", "PR-reachable CodeScene"),
        ("pr_reusable", "PR-reachable CodeScene"),
        ("pr_secret_inherit", "external PR call passes secrets"),
        ("pr_external_pin", "unproved external workflow call"),
        ("pr_computed_secret", "unproved PR secret route"),
        ("pr_baseline_override", "could write a coverage baseline"),
        ("pr_ratchet_off", "assert"),
        ("pr_artefact_on", "assert"),
        ("remove_token_check", "Check CodeScene token"),
        ("reorder_token_check", "assert"),
        ("token_env", "CS_ACCESS_TOKEN"),
        ("bypass_guard", "assert"),
        ("cancel", "concurrency"),
        ("coverage_parity", "PR and main coverage inputs differ"),
        ("remove_preflight", "build-tool preflight"),
        ("preflight_after_suite", "assert"),
        ("secret_in_extra_step", "only the token check"),
        ("duplicate_writer", "baseline writers"),
        ("remove_environment", "protected codescene environment"),
        ("move_environment", "PR-reachable CodeScene"),
        ("remove_upload", "assert"),
        ("empty_workflows", "assert"),
    ],
)
def test_contract_rejects_mutations(mutation: str, reason: str) -> None:
    workflows = _workflow_shape_fixture()
    main = workflows["coverage-main.yml"]
    steps = _main_steps(workflows)
    if mutation == "pr_token":
        _pr_steps(workflows).append({"run": "echo ${{ secrets.CS_ACCESS_TOKEN }}"})
    elif mutation == "pr_environment":
        workflows["ci.yml"]["jobs"]["build-test"]["environment"] = "codescene"
    elif mutation == "pr_reusable":
        workflows["ci.yml"]["jobs"]["called"] = {
            "uses": "./.github/workflows/hidden.yml"
        }
        workflows["hidden.yml"] = parse_workflow(
            "name: Hidden\non: workflow_call\njobs:\n"
            "  hidden:\n    runs-on: ubuntu-latest\n"
            "    steps: [{run: 'echo cs-coverage upload'}]\n"
        )
    elif mutation == "pr_secret_inherit":
        workflows["dependabot-automerge.yml"]["jobs"]["automerge"]["secrets"] = (
            "inherit"
        )
    elif mutation == "pr_external_pin":
        workflows["dependabot-automerge.yml"]["jobs"]["automerge"]["uses"] = (
            "leynos/shared-actions/.github/workflows/dependabot-automerge.yml@"
            "0000000000000000000000000000000000000000"
        )
    elif mutation == "pr_computed_secret":
        _pr_steps(workflows).append({"run": "echo ${{ toJSON(secrets) }}"})
    elif mutation == "pr_baseline_override":
        _pr_steps(workflows)[-1]["with"]["publish-baseline"] = "always"
    elif mutation == "pr_ratchet_off":
        _pr_steps(workflows)[-1]["with"]["with-ratchet"] = "false"
    elif mutation == "pr_artefact_on":
        _pr_steps(workflows)[-1]["with"]["publish-artefact"] = "true"
    elif mutation == "remove_token_check":
        steps[:] = [step for step in steps if step.get("id") != "codescene-token"]
    elif mutation == "reorder_token_check":
        steps[-2], steps[-1] = steps[-1], steps[-2]
    elif mutation == "token_env":
        steps[-1]["env"] = {"CS_ACCESS_TOKEN": "${{ secrets.CS_ACCESS_TOKEN }}"}
    elif mutation == "bypass_guard":
        steps[-1]["if"] += " || github.event_name == 'workflow_dispatch'"
    elif mutation == "cancel":
        main["concurrency"]["cancel-in-progress"] = True
    elif mutation == "coverage_parity":
        steps[-3]["with"]["format"] = "cobertura"
    elif mutation == "remove_preflight":
        steps[:] = [
            step for step in steps if step.get("run") != "make check-build-tools"
        ]
    elif mutation == "preflight_after_suite":
        check = next(
            step for step in steps if step.get("run") == "make check-build-tools"
        )
        steps.remove(check)
        steps.insert(
            steps.index(
                next(
                    step
                    for step in steps
                    if "generate-coverage@" in str(step.get("uses", ""))
                )
            )
            + 1,
            check,
        )
    elif mutation == "secret_in_extra_step":
        steps.insert(-2, {"run": "echo ${{ secrets.CS_ACCESS_TOKEN }}"})
    elif mutation == "duplicate_writer":
        workflows["second.yml"] = parse_workflow(
            "name: Second\non:\n  push:\n    branches: [main]\n"
            "jobs:\n  write:\n    runs-on: ubuntu-latest\n"
            "    steps:\n      - uses: leynos/shared-actions/.github/actions/generate-coverage@"
            "abf0dcf2686de1eaf79b6dc9a16662b631bed149\n"
            "        with: {with-ratchet: 'true'}\n"
        )
    elif mutation == "remove_environment":
        del main["jobs"]["coverage-publisher"]["environment"]
    elif mutation == "move_environment":
        main["jobs"]["coverage-publisher"]["environment"] = "other"
        workflows["ci.yml"]["jobs"]["build-test"]["environment"] = "codescene"
    elif mutation == "remove_upload":
        steps.pop()
    elif mutation == "empty_workflows":
        workflows.clear()
    with pytest.raises((AssertionError, ValueError)) as caught:
        validate(workflows)
    assert reason == "assert" or reason in str(caught.value)


def test_pr_closure_follows_workflow_run_chain() -> None:
    workflows = _workflow_shape_fixture()
    workflows["after-pr.yml"] = parse_workflow(
        "name: After PR\non:\n  workflow_run:\n"
        "    workflows: [CI]\n    types: [completed]\n"
        "jobs:\n  example:\n    runs-on: ubuntu-latest\n"
        "    steps: [{run: 'echo safe'}]\n"
    )
    assert "after-pr.yml" in pr_reachable(workflows)
    workflows["after-pr.yml"]["jobs"]["example"]["steps"][0]["run"] = (
        "echo ${{ secrets.CS_ACCESS_TOKEN }}"
    )
    with pytest.raises(AssertionError, match="PR-reachable CodeScene"):
        validate(workflows)
