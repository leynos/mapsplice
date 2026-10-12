"""Mutation tests for CV-005 coverage ownership and token isolation."""

from __future__ import annotations

from collections.abc import Callable
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


WorkflowSet = dict[str, dict[object, object]]
Mutation = Callable[[WorkflowSet], None]


def _add_local_reusable_call(workflows: WorkflowSet) -> None:
    workflows["ci.yml"]["jobs"]["called"] = {
        "uses": "./.github/workflows/hidden.yml"
    }
    workflows["hidden.yml"] = parse_workflow(
        "name: Hidden\non: workflow_call\njobs:\n"
        "  hidden:\n    runs-on: ubuntu-latest\n"
        "    steps: [{run: 'echo cs-coverage upload'}]\n"
    )


def _change_skip_warning(workflows: WorkflowSet) -> None:
    next(
        step
        for step in _main_steps(workflows)
        if step.get("name") == "Report skipped CodeScene upload"
    )["run"] = "echo ${{ secrets.CS_ACCESS_TOKEN }}"


def _reorder_token_check(workflows: WorkflowSet) -> None:
    steps = _main_steps(workflows)
    token_index = next(
        index for index, step in enumerate(steps) if step.get("id") == "codescene-token"
    )
    warning_index = next(
        index
        for index, step in enumerate(steps)
        if step.get("name") == "Report skipped CodeScene upload"
    )
    steps[token_index], steps[warning_index] = steps[warning_index], steps[token_index]


def _change_coverage_format(workflows: WorkflowSet) -> None:
    coverage = next(
        step
        for step in _main_steps(workflows)
        if "generate-coverage@" in str(step.get("uses", ""))
    )
    coverage["with"]["format"] = "cobertura"


def _move_preflight_after_suite(workflows: WorkflowSet) -> None:
    steps = _main_steps(workflows)
    check = next(step for step in steps if step.get("run") == "make check-build-tools")
    steps.remove(check)
    coverage_index = next(
        index
        for index, step in enumerate(steps)
        if "generate-coverage@" in str(step.get("uses", ""))
    )
    steps.insert(coverage_index + 1, check)


def _add_duplicate_writer(workflows: WorkflowSet) -> None:
    workflows["second.yml"] = parse_workflow(
        "name: Second\non:\n  push:\n    branches: [main]\n"
        "jobs:\n  write:\n    runs-on: ubuntu-latest\n"
        "    steps:\n      - uses: leynos/shared-actions/.github/actions/generate-coverage@"
        "abf0dcf2686de1eaf79b6dc9a16662b631bed149\n"
        "        with: {with-ratchet: 'true'}\n"
    )


def _move_publisher_environment(workflows: WorkflowSet) -> None:
    workflows["coverage-main.yml"]["jobs"]["coverage-publisher"]["environment"] = (
        "other"
    )
    workflows["ci.yml"]["jobs"]["build-test"]["environment"] = "codescene"


MUTATIONS: dict[str, Mutation] = {
    "pr_token": lambda workflows: _pr_steps(workflows).append(
        {"run": "echo ${{ secrets.CS_ACCESS_TOKEN }}"}
    ),
    "pr_environment": lambda workflows: workflows["ci.yml"]["jobs"]["build-test"].__setitem__(
        "environment", "codescene"
    ),
    "pr_reusable": _add_local_reusable_call,
    "pr_secret_inherit": lambda workflows: workflows["dependabot-automerge.yml"][
        "jobs"
    ]["automerge"].__setitem__("secrets", "inherit"),
    "pr_external_pin": lambda workflows: workflows["dependabot-automerge.yml"][
        "jobs"
    ]["automerge"].__setitem__(
        "uses",
        "leynos/shared-actions/.github/workflows/dependabot-automerge.yml@main",
    ),
    "pr_computed_secret": lambda workflows: _pr_steps(workflows).append(
        {"run": "echo ${{ toJSON(secrets) }}"}
    ),
    "pr_baseline_override": lambda workflows: _pr_steps(workflows)[-1]["with"].__setitem__(
        "publish-baseline", "always"
    ),
    "pr_ratchet_off": lambda workflows: _pr_steps(workflows)[-1]["with"].__setitem__(
        "with-ratchet", "false"
    ),
    "pr_artefact_on": lambda workflows: _pr_steps(workflows)[-1]["with"].__setitem__(
        "publish-artefact", "true"
    ),
    "remove_token_check": lambda workflows: _main_steps(workflows).__setitem__(
        slice(None),
        [step for step in _main_steps(workflows) if step.get("id") != "codescene-token"],
    ),
    "remove_skip_warning": lambda workflows: _main_steps(workflows).__setitem__(
        slice(None),
        [
            step
            for step in _main_steps(workflows)
            if step.get("name") != "Report skipped CodeScene upload"
        ],
    ),
    "change_skip_warning": _change_skip_warning,
    "reorder_token_check": _reorder_token_check,
    "token_env": lambda workflows: _main_steps(workflows)[-1].__setitem__(
        "env", {"CS_ACCESS_TOKEN": "${{ secrets.CS_ACCESS_TOKEN }}"}
    ),
    "bypass_guard": lambda workflows: _main_steps(workflows)[-1].__setitem__(
        "if", _main_steps(workflows)[-1]["if"] + " || github.event_name == 'workflow_dispatch'"
    ),
    "cancel": lambda workflows: workflows["coverage-main.yml"]["concurrency"].__setitem__(
        "cancel-in-progress", True
    ),
    "coverage_parity": _change_coverage_format,
    "remove_preflight": lambda workflows: _main_steps(workflows).__setitem__(
        slice(None),
        [step for step in _main_steps(workflows) if step.get("run") != "make check-build-tools"],
    ),
    "preflight_after_suite": _move_preflight_after_suite,
    "secret_in_extra_step": lambda workflows: _main_steps(workflows).insert(
        -2, {"run": "echo ${{ secrets.CS_ACCESS_TOKEN }}"}
    ),
    "duplicate_writer": _add_duplicate_writer,
    "remove_environment": lambda workflows: workflows["coverage-main.yml"]["jobs"][
        "coverage-publisher"
    ].pop("environment"),
    "move_environment": _move_publisher_environment,
    "remove_upload": lambda workflows: _main_steps(workflows).pop(),
    "empty_workflows": lambda workflows: workflows.clear(),
}


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
        ("remove_skip_warning", "missing-token warning"),
        ("change_skip_warning", "missing-token warning"),
        ("reorder_token_check", "publisher steps are out of order"),
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
    MUTATIONS[mutation](workflows)
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
