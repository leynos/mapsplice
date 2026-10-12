"""Mutation cases for the Whitaker workflow provisioning contract."""

from __future__ import annotations

import copy
from collections.abc import Callable
from dataclasses import dataclass

import pytest

from whitaker_provisioning_test import ACTION, _has_lint_command, _validate, _workflows


@dataclass
class MutationContext:
    ci: dict
    steps: list[dict]
    install_index: int
    lint_index: int
    install: dict


def _mutation_context(workflows: dict[str, dict]) -> MutationContext:
    ci = workflows["ci.yml"]
    steps = ci["jobs"]["build-test"]["steps"]
    install_index = next(i for i, step in enumerate(steps) if step.get("uses") == ACTION)
    lint_index = next(i for i, step in enumerate(steps) if _has_lint_command(step))
    return MutationContext(ci, steps, install_index, lint_index, steps[install_index])


MutationApplier = Callable[[MutationContext], object]
MUTATIONS: dict[str, MutationApplier] = {
    "remove_action": lambda context: context.steps.pop(context.install_index),
    "wrong_pin": lambda context: context.install.__setitem__(
        "uses", ACTION.replace("2ddf19b3", "00000000")
    ),
    "suite_pin": lambda context: context.install.setdefault("with", {}).__setitem__(
        "suite-version", "v1"
    ),
    "installer_pin": lambda context: context.install.setdefault("with", {}).__setitem__(
        "installer-version", "0.2.8"
    ),
    "cranelift_input": lambda context: context.install.setdefault("with", {}).__setitem__(
        "cranelift", "true"
    ),
    "after_suite": lambda context: context.steps.insert(
        context.lint_index + 1, context.steps.pop(context.install_index)
    ),
    "conditional": lambda context: context.install.__setitem__(
        "if", "github.event_name == 'push'"
    ),
    "soft_install": lambda context: context.install.__setitem__(
        "continue-on-error", True
    ),
    "soft_lint": lambda context: context.steps[context.lint_index].__setitem__(
        "continue-on-error", True
    ),
    "source_fallback": lambda context: context.steps.insert(
        context.install_index, {"run": "cargo install whitaker-installer"}
    ),
    "masked_lint": lambda context: context.steps[context.lint_index].__setitem__(
        "run", "make lint | tee lint.log"
    ),
    "new_suite_job": lambda context: context.ci["jobs"].__setitem__(
        "extra", {"runs-on": "ubuntu-latest", "steps": [{"run": "make lint"}]}
    ),
    "new_direct_job": lambda context: context.ci["jobs"].__setitem__(
        "extra", {"runs-on": "windows-latest", "steps": [{"run": "whitaker --all"}]}
    ),
    "empty_jobs": lambda context: context.ci.__setitem__("jobs", {}),
}


@pytest.mark.parametrize(
    ("mutation", "expected"),
    [
        ("remove_action", "require one pinned action"),
        ("wrong_pin", "require one pinned action"),
        ("suite_pin", "unsupported Whitaker inputs"),
        ("installer_pin", "unsupported Whitaker inputs"),
        ("cranelift_input", "unsupported Whitaker inputs"),
        ("after_suite", "installed after lint"),
        ("conditional", "installer is conditional"),
        ("soft_install", "installer may fail softly"),
        ("soft_lint", "lint may fail softly"),
        ("source_fallback", "ad hoc Whitaker installation"),
        ("masked_lint", "lint exit status may be masked"),
        ("new_suite_job", "require one pinned action"),
        ("new_direct_job", "require one pinned action"),
        ("empty_jobs", "no jobs"),
    ],
)
def test_ci_contract_rejects_regressions(mutation: str, expected: str) -> None:
    """Each named regression must produce a failing contract verdict."""
    workflows = copy.deepcopy(_workflows())
    MUTATIONS[mutation](_mutation_context(workflows))
    with pytest.raises(AssertionError, match=expected):
        _validate(workflows)
