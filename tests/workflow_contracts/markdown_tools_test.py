"""Check the Markdown tools needed by mapsplice's formatter and golden tests."""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

import pytest
import yaml


WORKFLOW = Path(__file__).resolve().parents[2] / ".github/workflows/ci.yml"
REPOSITORY = WORKFLOW.parents[2]
MDTABLEFIX_ACTION = "leynos/shared-actions/.github/actions/install-mdtablefix"
MARKDOWNLINT_ACTION = "DavidAnson/markdownlint-cli2-action"


def _steps(document: str) -> list[dict[str, object]]:
    workflow = yaml.safe_load(document)
    jobs = workflow.get("jobs")
    assert isinstance(jobs, dict) and jobs, "CI must have jobs"
    build_test = jobs.get("build-test")
    assert isinstance(build_test, dict), "CI must have a build-test job"
    steps = build_test.get("steps")
    assert isinstance(steps, list) and steps, "build-test must have steps"
    assert all(isinstance(step, dict) for step in steps)
    return steps


def _action_step(steps: list[dict[str, object]], action: str) -> tuple[int, dict]:
    matches = [
        (index, step)
        for index, step in enumerate(steps)
        if str(step.get("uses", "")).partition("@")[0] == action
    ]
    assert len(matches) == 1, f"expected one {action} step, found {len(matches)}"
    index, step = matches[0]
    pin = str(step["uses"]).partition("@")[2]
    assert len(pin) == 40 and all(char in "0123456789abcdef" for char in pin), (
        f"{action} must have a full commit pin"
    )
    assert "if" not in step, f"{action} must run in every build-test job"
    assert step.get("continue-on-error") is not True, f"{action} must fail closed"
    return index, step


def _step_index(steps: list[dict[str, object]], name: str) -> int:
    matches = [index for index, step in enumerate(steps) if step.get("name") == name]
    assert len(matches) == 1, f"expected one {name!r} step, found {len(matches)}"
    return matches[0]


def _check_contract(document: str) -> None:
    steps = _steps(document)
    install_index, installer = _action_step(steps, MDTABLEFIX_ACTION)
    assert installer.get("with") == {"version": "0.6.0"}
    assert install_index < _step_index(steps, "Format")
    assert install_index < _step_index(steps, "Test and Measure Coverage")

    lint_index, markdownlint = _action_step(steps, MARKDOWNLINT_ACTION)
    assert markdownlint.get("with") == {"globs": "**/*.md"}
    assert lint_index < _step_index(steps, "Test and Measure Coverage")

    golden_index = _step_index(steps, "Install the golden-test Markdown linter")
    golden_install = steps[golden_index]
    assert golden_index < _step_index(steps, "Test and Measure Coverage")
    assert "if" not in golden_install
    assert golden_install.get("continue-on-error") is not True
    job = yaml.safe_load(document)["jobs"]["build-test"]
    version = job["env"]["MARKDOWNLINT_CLI2_VERSION"]
    assert isinstance(version, str) and re.fullmatch(r"\d+\.\d+\.\d+", version)
    assert "npm install --global --prefix" in str(golden_install.get("run", ""))
    assert '"markdownlint-cli2@$MARKDOWNLINT_CLI2_VERSION"' in str(
        golden_install.get("run", "")
    )


def test_markdown_tools_precede_formatter_and_golden_tests() -> None:
    """Install pinned binaries before format checks and coverage's test suite."""
    _check_contract(WORKFLOW.read_text(encoding="utf-8"))


@pytest.mark.parametrize("target, mode", [("fmt", "--in-place"), ("check-fmt", "--check")])
def test_make_uses_git_selected_markdown(target: str, mode: str) -> None:
    """The evaluated Make recipe reaches mdtablefix with Git selection."""
    completed = subprocess.run(
        ["make", "--dry-run", target, "CARGO=echo"],
        cwd=REPOSITORY,
        capture_output=True,
        text=True,
        check=True,
    )
    commands = [line for line in completed.stdout.splitlines() if line.startswith("mdtablefix ")]
    assert commands == [
        "mdtablefix "
        f"{mode} --git --include-untracked --wrap --renumber --breaks --ellipsis --fences"
    ]


@pytest.mark.parametrize(
    "mutation",
    ["installer_after_format", "installer_soft_fail", "installer_guarded", "narrow_glob"],
)
def test_markdown_tool_contract_rejects_mutations(mutation: str) -> None:
    """Catch drift that leaves required tooling unavailable during a suite."""
    workflow = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))
    steps = workflow["jobs"]["build-test"]["steps"]
    if mutation == "installer_after_format":
        install_index, _ = _action_step(steps, MDTABLEFIX_ACTION)
        format_index = _step_index(steps, "Format")
        steps[install_index], steps[format_index] = steps[format_index], steps[install_index]
    elif mutation == "installer_soft_fail":
        _, installer = _action_step(steps, MDTABLEFIX_ACTION)
        installer["continue-on-error"] = True
    elif mutation == "installer_guarded":
        _, installer = _action_step(steps, MDTABLEFIX_ACTION)
        installer["if"] = "github.event_name == 'workflow_dispatch'"
    else:
        _, markdownlint = _action_step(steps, MARKDOWNLINT_ACTION)
        markdownlint["with"]["globs"] = "docs/**/*.md"

    with pytest.raises(AssertionError):
        _check_contract(yaml.safe_dump(workflow))
