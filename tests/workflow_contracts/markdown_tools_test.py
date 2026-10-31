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
    assert isinstance(jobs, dict) and jobs, f"CI must have jobs; observed {jobs!r}"
    build_test = jobs.get("build-test")
    assert isinstance(build_test, dict), (
        f"CI must have a build-test job; observed {build_test!r}"
    )
    steps = build_test.get("steps")
    assert isinstance(steps, list) and steps, (
        f"build-test must have steps; observed {steps!r}"
    )
    non_mapping_steps = [
        type(step).__name__ for step in steps if not isinstance(step, dict)
    ]
    assert not non_mapping_steps, (
        "every build-test step must be a mapping; "
        f"observed non-mapping step types {non_mapping_steps!r}"
    )
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
        f"{action} must have a full commit pin; observed {pin!r}"
    )
    assert "if" not in step, (
        f"{action} must run in every build-test job; observed if={step.get('if')!r}"
    )
    assert step.get("continue-on-error") is not True, (
        f"{action} must fail closed; "
        f"observed continue-on-error={step.get('continue-on-error')!r}"
    )
    return index, step


def _step_index(steps: list[dict[str, object]], name: str) -> int:
    matches = [index for index, step in enumerate(steps) if step.get("name") == name]
    assert len(matches) == 1, f"expected one {name!r} step, found {len(matches)}"
    return matches[0]


def _check_contract(document: str) -> None:
    steps = _steps(document)
    install_index, installer = _action_step(steps, MDTABLEFIX_ACTION)
    installer_config = installer.get("with")
    assert installer_config == {"version": "0.6.1"}, (
        "mdtablefix installer must pin version 0.6.1; "
        f"observed with={installer_config!r}"
    )
    format_index = _step_index(steps, "Format")
    assert install_index < format_index, (
        "mdtablefix must be installed before Format; "
        f"observed installer index {install_index}, Format index {format_index}"
    )
    coverage_index = _step_index(steps, "Test and Measure Coverage")
    assert install_index < coverage_index, (
        "mdtablefix must be installed before Test and Measure Coverage; "
        f"observed installer index {install_index}, coverage index {coverage_index}"
    )

    lint_index, markdownlint = _action_step(steps, MARKDOWNLINT_ACTION)
    markdownlint_config = markdownlint.get("with")
    assert markdownlint_config == {"globs": "**/*.md"}, (
        "markdownlint must check every Markdown file; "
        f"observed with={markdownlint_config!r}"
    )
    assert lint_index < coverage_index, (
        "markdownlint must run before Test and Measure Coverage; "
        f"observed markdownlint index {lint_index}, coverage index {coverage_index}"
    )

    golden_index = _step_index(steps, "Install the golden-test Markdown linter")
    golden_install = steps[golden_index]
    assert golden_index < coverage_index, (
        "golden-test Markdown linter must be installed before Test and Measure Coverage; "
        f"observed installer index {golden_index}, coverage index {coverage_index}"
    )
    assert "if" not in golden_install, (
        "golden-test Markdown linter must run in every build-test job; "
        f"observed if={golden_install.get('if')!r}"
    )
    assert golden_install.get("continue-on-error") is not True, (
        "golden-test Markdown linter must fail closed; "
        f"observed continue-on-error={golden_install.get('continue-on-error')!r}"
    )
    job = yaml.safe_load(document)["jobs"]["build-test"]
    version = job["env"]["MARKDOWNLINT_CLI2_VERSION"]
    assert isinstance(version, str) and re.fullmatch(r"\d+\.\d+\.\d+", version), (
        "MARKDOWNLINT_CLI2_VERSION must be a pinned semantic version; "
        f"observed {version!r}"
    )
    golden_run = str(golden_install.get("run", ""))
    assert "npm install --global --prefix" in golden_run, (
        "golden-test Markdown linter must use an isolated global npm prefix; "
        f"observed run={golden_run!r}"
    )
    assert '"markdownlint-cli2@$MARKDOWNLINT_CLI2_VERSION"' in golden_run, (
        "golden-test Markdown linter must install the configured version; "
        f"observed run={golden_run!r}"
    )


def test_markdown_tools_precede_formatter_and_golden_tests() -> None:
    """Install pinned binaries before format checks and coverage's test suite."""
    _check_contract(WORKFLOW.read_text(encoding="utf-8"))


@pytest.mark.parametrize(
    "target, mode", [("fmt", "--in-place"), ("check-fmt", "--check")]
)
def test_make_uses_git_selected_markdown(target: str, mode: str) -> None:
    """The evaluated Make recipe reaches mdtablefix with Git selection."""
    completed = subprocess.run(
        ["make", "--dry-run", target, "CARGO=echo"],
        cwd=REPOSITORY,
        capture_output=True,
        text=True,
        check=True,
    )
    commands = [
        line for line in completed.stdout.splitlines() if line.startswith("mdtablefix ")
    ]
    expected_commands = [
        f"mdtablefix {mode} --git --include-untracked --wrap --renumber --breaks --ellipsis --fences"
    ]
    assert commands == expected_commands, (
        f"make {target} must pass Git selection and {mode} to mdtablefix; "
        f"observed commands={commands!r}"
    )


@pytest.mark.parametrize(
    "mutation",
    [
        "installer_after_format",
        "installer_soft_fail",
        "installer_guarded",
        "narrow_glob",
    ],
)
def test_markdown_tool_contract_rejects_mutations(mutation: str) -> None:
    """Catch drift that leaves required tooling unavailable during a suite."""
    workflow = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))
    steps = workflow["jobs"]["build-test"]["steps"]
    if mutation == "installer_after_format":
        install_index, _ = _action_step(steps, MDTABLEFIX_ACTION)
        format_index = _step_index(steps, "Format")
        steps[install_index], steps[format_index] = (
            steps[format_index],
            steps[install_index],
        )
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


def test_the_publisher_installs_the_markdown_tools_before_it_measures_coverage() -> None:
    """The publisher runs the suite, whose golden tests need both tools.

    The golden and formatter-boundary tests run mdtablefix and markdownlint-cli2
    as subprocesses, so the main-only publisher installs them as ci.yml does;
    without them every Coverage (main) run failed before reaching the upload.
    """
    publisher = WORKFLOW.with_name("coverage-main.yml")
    document = yaml.safe_load(publisher.read_text(encoding="utf-8"))
    job = document["jobs"]["coverage-publisher"]
    steps = job["steps"]
    install_index, installer = _action_step(steps, MDTABLEFIX_ACTION)
    assert installer.get("with") == {"version": "0.6.1"}, (
        "the publisher must pin mdtablefix 0.6.1; "
        f"observed with={installer.get('with')!r}"
    )
    coverage_index = next(
        index
        for index, step in enumerate(steps)
        if "generate-coverage@" in str(step.get("uses", ""))
    )
    assert install_index < coverage_index, (
        "mdtablefix must be installed before the publisher measures coverage; "
        f"observed installer index {install_index}, coverage index {coverage_index}"
    )
    linter_index = _step_index(steps, "Install the golden-test Markdown linter")
    assert linter_index < coverage_index, (
        "the golden-test Markdown linter must be installed before coverage; "
        f"observed linter index {linter_index}, coverage index {coverage_index}"
    )
    ci_version = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))["jobs"][
        "build-test"
    ]["env"]["MARKDOWNLINT_CLI2_VERSION"]
    assert job["env"]["MARKDOWNLINT_CLI2_VERSION"] == ci_version, (
        "the publisher must install the same markdownlint-cli2 as ci.yml; "
        f"observed {job['env']['MARKDOWNLINT_CLI2_VERSION']!r} against {ci_version!r}"
    )


def _publisher_linter_step() -> dict[str, object]:
    """Return the publisher's golden-test linter install step."""
    publisher = WORKFLOW.with_name("coverage-main.yml")
    document = yaml.safe_load(publisher.read_text(encoding="utf-8"))
    steps = document["jobs"]["coverage-publisher"]["steps"]
    return steps[_step_index(steps, "Install the golden-test Markdown linter")]


@pytest.mark.parametrize(
    ("needle", "reason"),
    [
        ("npm install --global --prefix", "an isolated global npm prefix"),
        ('"markdownlint-cli2@$MARKDOWNLINT_CLI2_VERSION"', "the pinned version"),
        ('>> "$GITHUB_PATH"', "an export to GITHUB_PATH"),
        ("markdownlint-cli2\" --version", "a version check of the installed binary"),
    ],
)
def test_the_publisher_linter_step_installs_and_exposes_the_binary(
    needle: str, reason: str
) -> None:
    """The publisher's linter step must install, expose and check the tool."""
    run = str(_publisher_linter_step().get("run", ""))
    assert needle in run, f"the publisher linter step must use {reason}; observed {run!r}"
    assert "set -euo pipefail" in run, f"the step must fail closed; observed {run!r}"
