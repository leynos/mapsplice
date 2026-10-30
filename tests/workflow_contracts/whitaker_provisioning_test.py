"""Consumer contracts for the pinned, binary-only Whitaker installation."""

from __future__ import annotations

import json
import os
import re
import shlex
import subprocess
import sys
from dataclasses import replace
from pathlib import Path

import pytest
import yaml

from build_route_probe import (
    assert_whitaker_route,
    execute_make_route,
    route_for,
)
from build_standard_test import ROOT

WORKFLOWS = ROOT / ".github" / "workflows"
ACTION = (
    "leynos/shared-actions/.github/actions/install-whitaker"
    "@5a99aed78ce68dcebd3052b75c31e6f173162b1e"
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
        for name, job in _workflow_jobs(filename, document):
            if _contains_whitaker_lint(filename, name, job):
                jobs.append((filename, name, job))
    assert jobs, "no reachable Whitaker suite job found"
    return jobs


def _workflow_jobs(
    filename: str, document: dict
) -> list[tuple[str, dict]]:
    declared_jobs = document.get("jobs")
    assert isinstance(declared_jobs, dict) and declared_jobs, f"{filename}: no jobs"
    jobs = []
    for name, job in declared_jobs.items():
        assert isinstance(job, dict), f"{filename}/{name}: invalid job"
        jobs.append((name, job))
    return jobs


def _contains_whitaker_lint(filename: str, name: str, job: dict) -> bool:
    steps = job.get("steps", [])
    if _contains_lint_step(steps):
        return True
    if _is_local_mapsplice_call(job):
        raise AssertionError(f"{filename}/{name}: unresolved local workflow call")
    return False


def _contains_lint_step(steps: object) -> bool:
    if not isinstance(steps, list):
        return False
    return any(
        _has_lint_command(step) for step in steps if isinstance(step, dict)
    )


def _is_local_mapsplice_call(job: dict) -> bool:
    target = job.get("uses")
    return isinstance(target, str) and "mapsplice" in target


def _installer_index(filename: str, name: str, steps: list[dict]) -> int:
    installers = [index for index, step in enumerate(steps) if step.get("uses") == ACTION]
    assert len(installers) == 1, f"{filename}/{name}: require one pinned action"
    index = installers[0]
    install = steps[index]
    assert "if" not in install, f"{filename}/{name}: installer is conditional"
    assert install.get("continue-on-error") is not True, (
        f"{filename}/{name}: installer may fail softly"
    )
    assert install.get("with", {}) == {}, (
        f"{filename}/{name}: unsupported Whitaker inputs"
    )
    return index


def _validate_lint_steps(
    filename: str, name: str, steps: list[dict], install_index: int
) -> None:
    lint_indices = [index for index, step in enumerate(steps) if _has_lint_command(step)]
    assert all(install_index < index for index in lint_indices), (
        f"{filename}/{name}: Whitaker is installed after lint"
    )
    for index in lint_indices:
        step = steps[index]
        assert "if" not in step, f"{filename}/{name}: lint is conditional"
        assert step.get("continue-on-error") is not True, (
            f"{filename}/{name}: lint may fail softly"
        )
        command = step["run"]
        assert not re.search(r"\|\|\s*true|continue-on-error|\|\s*tee\b", command), (
            f"{filename}/{name}: lint exit status may be masked"
        )


def _validate_no_ad_hoc_installation(filename: str, name: str, job: dict) -> None:
    for step in job["steps"]:
        run = str(step.get("run", ""))
        assert not re.search(
            r"cargo\s+(?:install|binstall).*whitaker|whitaker-installer", run
        ), f"{filename}/{name}: ad hoc Whitaker installation remains"
        uses = str(step.get("uses", ""))
        if "install-whitaker@" in uses:
            assert uses == ACTION, f"{filename}/{name}: wrong Whitaker action pin"
    assert "WHITAKER_INSTALLER_VERSION" not in str(job), (
        f"{filename}/{name}: obsolete installer override remains"
    )


def _validate_suite_job(filename: str, name: str, job: dict) -> None:
    steps = job["steps"]
    assert all(isinstance(step, dict) for step in steps), f"{filename}/{name}: bad step"
    install_index = _installer_index(filename, name, steps)
    _validate_lint_steps(filename, name, steps, install_index)
    _validate_no_ad_hoc_installation(filename, name, job)


def _validate(workflows: dict[str, dict]) -> None:
    """Require unconditional pinned provisioning before each reachable suite."""
    for filename, name, job in _suite_jobs(workflows):
        _validate_suite_job(filename, name, job)


def _run_whitaker_environment_probe(
    tmp_path: Path, *, make_variables: tuple[str, ...] = (),
) -> dict[str, str]:
    """Capture the environment seen by an injected Make Whitaker executable."""
    probe = tmp_path / "whitaker-environment-probe"
    output = tmp_path / "whitaker-environment.json"
    probe.write_text(
        f"#!{sys.executable}\n"
        "import json, os\n"
        "names = ('CARGO_UNSTABLE_CODEGEN_BACKEND', 'CARGO_PROFILE_DEV_CODEGEN_BACKEND', "
        "'CARGO_PROFILE_TEST_CODEGEN_BACKEND', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTFLAGS')\n"
        "with open(os.environ['WHITAKER_PROBE_OUTPUT'], 'w', encoding='utf-8') as stream:\n"
        "    json.dump({name: os.environ.get(name, '') for name in names}, stream)\n",
        encoding="utf-8",
    )
    probe.chmod(0o755)
    environment = os.environ.copy()
    for name in (
        "CARGO_UNSTABLE_CODEGEN_BACKEND",
        "CARGO_PROFILE_DEV_CODEGEN_BACKEND",
        "CARGO_PROFILE_TEST_CODEGEN_BACKEND",
        "RUSTFLAGS",
    ):
        environment.pop(name, None)
    environment["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(
        (
            "-Zthreads=8",
            "-Zcodegen-backend=cranelift",
            "-C",
            "link-arg=-fuse-ld=mold",
            "--cfg",
            "caller_policy",
        )
    )
    environment["WHITAKER_PROBE_OUTPUT"] = str(output)
    environment["RUST_FLAGS"] = ""
    subprocess.run(
        [
            "make",
            "--silent",
            "--always-make",
            "lint-whitaker",
            "CARGO=true",
            "CARGO_FLAGS=",
            "DOC_TEST_TARGETS=false",
            f"WHITAKER={probe}", *make_variables,
        ],
        cwd=ROOT,
        env=environment,
        text=True,
        capture_output=True,
        check=True,
    )
    return json.loads(output.read_text(encoding="utf-8"))


def _assert_whitaker_environment(environment: dict[str, str]) -> None:
    """Require the Whitaker invocation to receive Cargo's supported backend route."""
    assert environment["CARGO_UNSTABLE_CODEGEN_BACKEND"] == "true", (
        "Whitaker must opt in to Cargo's unstable codegen-backend setting"
    )
    assert environment["CARGO_PROFILE_DEV_CODEGEN_BACKEND"] == "llvm"
    assert environment["CARGO_PROFILE_TEST_CODEGEN_BACKEND"] == "llvm"
    encoded_flags = environment["CARGO_ENCODED_RUSTFLAGS"].split("\x1f")
    rust_flags = shlex.split(environment["RUSTFLAGS"])
    flags = [*encoded_flags, *rust_flags]
    forbidden = {
        "-Zthreads=8",
        "-Zcodegen-backend=cranelift",
        "-Clink-arg=-fuse-ld=mold",
    }
    assert not forbidden.intersection(flags), "Whitaker inherited development flags"
    forbidden_pairs = {
        ("-Z", "threads=8"),
        ("-Z", "codegen-backend=cranelift"),
        ("-C", "link-arg=-fuse-ld=mold"),
    }
    pairs = {tuple(flags[index:index + 2]) for index in range(len(flags) - 1)}
    assert not forbidden_pairs.intersection(pairs), "Whitaker inherited development flags"


def test_ci_provisions_whitaker_before_each_lint_route() -> None:
    """The real workflow keeps a binding binary-only provisioning path."""
    _validate(_workflows())


def test_workflow_reader_rejects_duplicate_keys() -> None:
    """A duplicate jobs key cannot hide an unprovisioned suite job."""
    with pytest.raises(AssertionError, match="duplicate mapping keys"):
        yaml.load("jobs:\n  lint: {}\njobs:\n  hidden: {}\n", Loader=UniqueKeyLoader)


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


def test_make_whitaker_invocation_carries_cargo_opt_in(tmp_path: Path) -> None:
    """The evaluated Whitaker recipe opts in while retaining explicit LLVM routing."""
    environment = _run_whitaker_environment_probe(tmp_path)
    _assert_whitaker_environment(environment)

    without_opt_in = _run_whitaker_environment_probe(
        tmp_path,
        make_variables=(
            "WHITAKER_ENV=CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm "
            "CARGO_PROFILE_TEST_CODEGEN_BACKEND=llvm",
        ),
    )
    with pytest.raises(AssertionError, match="unstable codegen-backend setting"):
        _assert_whitaker_environment(without_opt_in)


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


def test_make_whitaker_failure_reaches_make_exit_status(tmp_path: Path) -> None:
    """A failing executable must make the composite lint target fail."""
    shell_shim = tmp_path / "bash"
    shell_shim.write_text(
        "#!/bin/sh\n"
        "if [ \"$1\" = 'scripts/check-build-tools.sh' ]; then exit 0; fi\n"
        'exec /bin/bash "$@"\n',
        encoding="utf-8",
    )
    shell_shim.chmod(0o755)
    result = subprocess.run(
        ["make", "--silent", "lint", "WHITAKER=false", "CARGO=true"],
        cwd=ROOT,
        env={**os.environ, "PATH": f"{tmp_path}:{os.environ['PATH']}"},
        text=True,
        capture_output=True,
        check=False,
    )
    assert result.returncode != 0, "a failing Whitaker run must fail make lint"
    assert re.search(r"\[Makefile:\d+: lint-whitaker\] Error \d+", result.stderr), (
        f"make lint failed before or outside the Whitaker recipe: {result.stderr!r}"
    )
