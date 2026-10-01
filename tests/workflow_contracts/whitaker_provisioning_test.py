"""Consumer contracts for the pinned, binary-only Whitaker installation."""

from __future__ import annotations

import copy
from dataclasses import replace
import re
import subprocess
from pathlib import Path

import pytest
import yaml

from build_standard_routing_test import (
    assert_whitaker_route,
    execute_make_route,
    route_for,
)

ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = ROOT / ".github" / "workflows"
ACTION = (
    "leynos/shared-actions/.github/actions/install-whitaker"
    "@6dea5677a84fec60ca51b07202570e3af12ffdb4"
)
MAKE_CALL = re.compile(r"(?<![\w-])make\s+(?:--[\w-]+\s+)*([a-z][\w-]*)")
DIRECT_GATE = re.compile(r"(?<![\w-])whitaker\s+--all\b")


class UniqueKeyLoader(yaml.SafeLoader):
    """Reject duplicate YAML keys instead of silently losing a CI route."""


def _unique_mapping(loader: UniqueKeyLoader, node: yaml.MappingNode) -> dict:
    pairs = loader.construct_pairs(node, deep=True)
    keys = [key for key, _ in pairs]
    assert len(keys) == len(set(keys)), "workflow has duplicate mapping keys"
    return dict(pairs)


UniqueKeyLoader.add_constructor(
    yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, _unique_mapping
)


def _workflows() -> dict[str, dict]:
    """Read every workflow so a newly reachable lint job cannot go unnoticed."""
    paths = sorted(WORKFLOWS.glob("*.yml")) + sorted(WORKFLOWS.glob("*.yaml"))
    assert paths, "no workflows found"
    workflows = {}
    for path in paths:
        document = yaml.load(path.read_text(encoding="utf-8"), Loader=UniqueKeyLoader)
        assert isinstance(document, dict), f"{path}: workflow must be a mapping"
        workflows[path.name] = document
    return workflows


def _whitaker_make_targets() -> set[str]:
    """Resolve simple Make prerequisites instead of naming today's CI goal."""
    source = (ROOT / "Makefile").read_text(encoding="utf-8")
    dependencies = {
        target: set(prerequisites.split())
        for target, prerequisites in re.findall(
            r"^([a-z][\w-]*):\s*([^#\n]*)", source, flags=re.MULTILINE
        )
    }
    assert "lint-whitaker" in dependencies, "Whitaker Make leaf is missing"
    reachable = {"lint-whitaker"}
    while True:
        expanded = reachable | {
            target
            for target, prerequisites in dependencies.items()
            if prerequisites & reachable
        }
        if expanded == reachable:
            break
        reachable = expanded
    assert {"lint", "all"} <= reachable, "Whitaker is not binding through lint/all"
    return reachable


def _has_lint_command(step: dict) -> bool:
    run = str(step.get("run", ""))
    targets = _whitaker_make_targets()
    return bool(DIRECT_GATE.search(run)) or any(
        match.group(1) in targets for match in MAKE_CALL.finditer(run)
    )


def _suite_jobs(workflows: dict[str, dict]) -> list[tuple[str, str, dict]]:
    jobs = []
    for filename, document in workflows.items():
        declared_jobs = document.get("jobs")
        assert isinstance(declared_jobs, dict) and declared_jobs, f"{filename}: no jobs"
        for name, job in declared_jobs.items():
            assert isinstance(job, dict), f"{filename}/{name}: invalid job"
            steps = job.get("steps", [])
            if isinstance(steps, list) and any(
                isinstance(step, dict) and _has_lint_command(step) for step in steps
            ):
                jobs.append((filename, name, job))
            elif isinstance(job.get("uses"), str) and "mapsplice" in job["uses"]:
                raise AssertionError(f"{filename}/{name}: unresolved local workflow call")
    assert jobs, "no reachable Whitaker suite job found"
    return jobs


def _validate(workflows: dict[str, dict]) -> None:
    """Require unconditional pinned provisioning before each reachable suite."""
    for filename, name, job in _suite_jobs(workflows):
        steps = job["steps"]
        assert all(isinstance(step, dict) for step in steps), f"{filename}/{name}: bad step"
        installers = [index for index, step in enumerate(steps) if step.get("uses") == ACTION]
        assert len(installers) == 1, f"{filename}/{name}: require one pinned action"
        install_index = installers[0]
        install = steps[install_index]
        assert "if" not in install, f"{filename}/{name}: installer is conditional"
        assert install.get("continue-on-error") is not True, (
            f"{filename}/{name}: installer may fail softly"
        )
        assert install.get("with") == {"cranelift": "true"}, (
            f"{filename}/{name}: unsupported Whitaker inputs"
        )
        lint_indices = [index for index, step in enumerate(steps) if _has_lint_command(step)]
        assert all(install_index < index for index in lint_indices), (
            f"{filename}/{name}: Whitaker is installed after lint"
        )
        for index in lint_indices:
            assert "if" not in steps[index], f"{filename}/{name}: lint is conditional"
            assert steps[index].get("continue-on-error") is not True, (
                f"{filename}/{name}: lint may fail softly"
            )
            command = steps[index]["run"]
            assert not re.search(r"\|\|\s*true|continue-on-error|\|\s*tee\b", command), (
                f"{filename}/{name}: lint exit status may be masked"
            )
        for step in steps:
            run = str(step.get("run", ""))
            assert not re.search(r"cargo\s+(?:install|binstall).*whitaker|whitaker-installer", run), (
                f"{filename}/{name}: ad hoc Whitaker installation remains"
            )
            uses = str(step.get("uses", ""))
            if "install-whitaker@" in uses:
                assert uses == ACTION, f"{filename}/{name}: wrong Whitaker action pin"
        assert "WHITAKER_INSTALLER_VERSION" not in str(job), (
            f"{filename}/{name}: obsolete installer override remains"
        )


def test_ci_provisions_whitaker_before_each_lint_route() -> None:
    """The real workflow keeps a binding binary-only provisioning path."""
    _validate(_workflows())


def test_workflow_reader_rejects_duplicate_keys() -> None:
    """A duplicate jobs key cannot hide an unprovisioned suite job."""
    with pytest.raises(AssertionError, match="duplicate mapping keys"):
        yaml.load("jobs:\n  lint: {}\njobs:\n  hidden: {}\n", Loader=UniqueKeyLoader)


@pytest.mark.parametrize(
    ("mutation", "expected"),
    [
        ("remove_action", "require one pinned action"),
        ("wrong_pin", "require one pinned action"),
        ("suite_pin", "unsupported Whitaker inputs"),
        ("installer_pin", "unsupported Whitaker inputs"),
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
    ci = workflows["ci.yml"]
    job = ci["jobs"]["build-test"]
    steps = job["steps"]
    install_index = next(i for i, step in enumerate(steps) if step.get("uses") == ACTION)
    lint_index = next(i for i, step in enumerate(steps) if _has_lint_command(step))
    install = steps[install_index]
    if mutation == "remove_action":
        steps.pop(install_index)
    elif mutation == "wrong_pin":
        install["uses"] = ACTION.replace("6dea5677", "00000000")
    elif mutation == "suite_pin":
        install["with"]["suite-version"] = "v1"
    elif mutation == "installer_pin":
        install["with"]["installer-version"] = "0.2.8"
    elif mutation == "after_suite":
        steps.insert(lint_index + 1, steps.pop(install_index))
    elif mutation == "conditional":
        install["if"] = "github.event_name == 'push'"
    elif mutation == "soft_install":
        install["continue-on-error"] = True
    elif mutation == "soft_lint":
        steps[lint_index]["continue-on-error"] = True
    elif mutation == "source_fallback":
        steps.insert(install_index, {"run": "cargo install whitaker-installer"})
    elif mutation == "masked_lint":
        steps[lint_index]["run"] = "make lint | tee lint.log"
    elif mutation == "new_suite_job":
        ci["jobs"]["extra"] = {"runs-on": "ubuntu-latest", "steps": [{"run": "make lint"}]}
    elif mutation == "new_direct_job":
        ci["jobs"]["extra"] = {"runs-on": "windows-latest", "steps": [{"run": "whitaker --all"}]}
    elif mutation == "empty_jobs":
        ci["jobs"] = {}
    with pytest.raises(AssertionError, match=expected):
        _validate(workflows)


def test_make_lint_order_and_whitaker_flags(tmp_path: Path) -> None:
    """Dry-run checks order; the probe checks Whitaker's evaluated environment."""
    result = subprocess.run(
        ["make", "--dry-run", "-j", "4", "lint", "CARGO=true", "WHITAKER=whitaker-probe"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=True,
    )
    commands = result.stdout.splitlines()
    clippy_index = next(i for i, line in enumerate(commands) if " clippy " in line)
    whitaker_index = next(
        i for i, line in enumerate(commands) if "whitaker-probe --all" in line
    )
    assert clippy_index < whitaker_index
    assert_whitaker_route(route_for(execute_make_route(tmp_path, "lint"), ("--all",)))


@pytest.mark.parametrize(
    ("attribute", "replacement"),
    [
        pytest.param("dev_backend", "cranelift", id="remove-dev-llvm"),
        pytest.param("test_backend", "cranelift", id="remove-test-llvm"),
        pytest.param("rust_flags", ("-D", "warnings", "-Zthreads=8"), id="leak-frontend"),
        pytest.param("rust_flags", ("-D", "warnings", "-C", "link-arg=-fuse-ld=mold"), id="leak-linker"),
        pytest.param(
            "encoded_flags",
            ("--cfg", "caller_policy", "-Z", "codegen-backend=cranelift", "-D", "warnings"),
            id="leak-encoded-backend",
        ),
    ],
)
def test_make_whitaker_command_mutations_are_detected(
    tmp_path: Path, attribute: str, replacement: str | tuple[str, ...],
) -> None:
    """A changed evaluated Whitaker environment must fail the route contract."""
    route = route_for(execute_make_route(tmp_path, "lint-whitaker"), ("--all",))
    mutated = replace(route, **{attribute: replacement})
    assert mutated != route
    with pytest.raises(AssertionError):
        assert_whitaker_route(mutated)


def test_make_whitaker_failure_reaches_make_exit_status() -> None:
    """A failing executable must make the composite lint target fail."""
    result = subprocess.run(
        ["make", "--silent", "lint", "WHITAKER=false", "CARGO=true"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert result.returncode != 0
