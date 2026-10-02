"""Local contracts for the selected development build and its CI provisioning."""

from __future__ import annotations

import copy
import os
import platform
import re
import shutil
import subprocess
from pathlib import Path

import pytest
import yaml

ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = ROOT / ".github" / "workflows"
SETUP_RUST = (
    "leynos/shared-actions/.github/actions/setup-rust@"
    "ff1dd759dfffc0db3459e30e833f52437ee62b57"
)
DEV_FLAGS = ("-Zthreads=8", "-fuse-ld=mold")
LINKER_DIGESTS = {
    "x86_64": "a3696680d99e692970590a178bc3a33d78d60d1c6dc9db7a11b557b02b751f5d",
    "aarch64": "946de2774b06a71346bd59b55fddba610b65b8d93c3a4a1559cc84e103472710",
}


def make_plan(goal: str, **variables: str) -> list[str]:
    """Return evaluated recipe lines without executing a Cargo gate."""
    command = [
        "make", "--no-print-directory", "--dry-run", "--always-make",
        goal, "CARGO=probe-cargo",
    ]
    command.extend(f"{key}={value}" for key, value in variables.items())
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=True)
    return result.stdout.splitlines()


def workflow(name: str) -> dict[str, object]:
    """Load one local workflow for its executable step contract."""
    return yaml.safe_load((WORKFLOWS / name).read_text(encoding="utf-8"))


def linux_runner(runs_on: object) -> bool:
    """Recognize the scalar, list and group runner forms used by Actions."""
    if isinstance(runs_on, str):
        return "ubuntu" in runs_on.lower() or "linux" in runs_on.lower()
    if isinstance(runs_on, list):
        return any(linux_runner(item) for item in runs_on)
    if isinstance(runs_on, dict):
        return linux_runner(runs_on.get("labels", []))
    return False


def job_runs_on_linux(job: dict[str, object]) -> bool:
    """Resolve literal runner labels and simple matrix runner selections."""
    runs_on = job.get("runs-on")
    if isinstance(runs_on, str) and "${{" in runs_on:
        match = re.fullmatch(r"\$\{\{\s*matrix\.(\w+)\s*\}\}", runs_on)
        assert match, f"cannot resolve runner expression {runs_on!r}"
        matrix = job.get("strategy", {}).get("matrix", {})
        choices = matrix.get(match.group(1)) if isinstance(matrix, dict) else None
        assert isinstance(choices, list) and choices, (
            f"cannot resolve runner matrix {runs_on!r}"
        )
        return any(linux_runner(choice) for choice in choices)
    return linux_runner(runs_on)


def is_suite_step(step: dict[str, object]) -> bool:
    """Identify a test-bearing step, including coverage's hidden Cargo run."""
    command = str(step.get("run", ""))
    action = str(step.get("uses", ""))
    return bool(
        re.search(
            r"\bmake(?:\s+-\S+)*\s+(?:all|test|lint|typecheck|build|release)(?![\w-])",
            command,
        )
        or re.search(r"\bcargo\s+(?:test|nextest|build|check|clippy)\b", command)
        or "/generate-coverage@" in action
    )


def validate_suite_provisioning(document: dict[str, object]) -> None:
    """Reject every reachable Linux suite job without an earlier hard install."""
    jobs = document.get("jobs")
    assert isinstance(jobs, dict) and jobs, "workflow has no jobs"
    suites = 0
    for name, job in jobs.items():
        assert isinstance(job, dict), f"{name} is not a job mapping"
        if "uses" in job:
            continue  # Reusable callers have a separate setup-commands contract.
        steps = job.get("steps")
        if not isinstance(steps, list):
            continue
        suite_indices = [i for i, step in enumerate(steps) if is_suite_step(step)]
        if not suite_indices:
            continue
        if not job_runs_on_linux(job):
            continue
        suites += 1
        installs = [
            i
            for i, step in enumerate(steps)
            if step.get("uses") == SETUP_RUST
            and step.get("with", {}).get("install-mold") == "true"
        ]
        assert len(installs) == 1, f"{name} needs one pinned linker install"
        install = steps[installs[0]]
        assert "if" not in install and install.get("continue-on-error") is not True, (
            f"{name} installer must be unconditional and binding"
        )
        assert installs[0] < min(suite_indices), f"{name} installs after its suite"
        assert install.get("with", {}).get("rustflags") == "", (
            f"{name} setup-rust must leave Cargo config discoverable"
        )
        clang = [
            i for i, step in enumerate(steps)
            if "apt-get install" in str(step.get("run", ""))
            and "clang" in str(step.get("run", ""))
            and step.get("if", "runner.os == 'Linux'") == "runner.os == 'Linux'"
            and step.get("continue-on-error") is not True
        ]
        assert clang and min(clang) < min(suite_indices), (
            f"{name} needs clang before its suite"
        )
    assert suites, "no reachable Linux suite jobs were measured"


def validate_coverage_exclusion(ci: dict[str, object]) -> None:
    """Refuse a missing or development-flagged coverage override."""
    steps = ci["jobs"]["build-test"]["steps"]
    coverage = [step for step in steps if "/generate-coverage@" in step.get("uses", "")]
    assert len(coverage) == 1
    assert coverage[0]["env"] == {"RUSTFLAGS": "-D warnings"}


def test_ci_installs_before_every_linux_suite_and_excludes_coverage() -> None:
    """The pinned action precedes all local CI compilation routes."""
    ci = workflow("ci.yml")
    validate_suite_provisioning(ci)
    validate_coverage_exclusion(ci)


@pytest.mark.parametrize("replacement", ["", "-D warnings -Zthreads=8", "-D warnings -Clink-arg=-fuse-ld=mold"])
def test_coverage_exclusion_mutations_are_detected(replacement: str) -> None:
    """Removing the override or adding either development flag is invalid."""
    ci = copy.deepcopy(workflow("ci.yml"))
    steps = ci["jobs"]["build-test"]["steps"]
    coverage = next(step for step in steps if "/generate-coverage@" in step.get("uses", ""))
    coverage["env"]["RUSTFLAGS"] = replacement
    with pytest.raises(AssertionError):
        validate_coverage_exclusion(ci)


def test_coverage_rejects_encoded_development_flags() -> None:
    """The local coverage action must receive its isolated compiler route."""
    ci = copy.deepcopy(workflow("ci.yml"))
    steps = ci["jobs"]["build-test"]["steps"]
    coverage = next(step for step in steps if "/generate-coverage@" in step.get("uses", ""))
    coverage["env"]["CARGO_ENCODED_RUSTFLAGS"] = "-Zthreads=8"
    with pytest.raises(AssertionError):
        validate_coverage_exclusion(ci)


@pytest.mark.parametrize("runner", ["ubuntu-latest", ["self-hosted", "Linux"], {"group": "builders", "labels": ["Linux", "X64"]}])
def test_runner_shapes_are_recognized(runner: object) -> None:
    """A runner syntax change must not silently erase suite coverage."""
    assert linux_runner(runner)


def test_matrix_runner_is_resolved_or_refused() -> None:
    """A matrix cannot hide a Linux suite behind a runner expression."""
    job = {
        "runs-on": "${{ matrix.os }}",
        "strategy": {"matrix": {"os": ["windows-latest", "ubuntu-latest"]}},
    }
    assert job_runs_on_linux(job)
    job["strategy"] = {"matrix": {"arch": ["X64"]}}
    with pytest.raises(AssertionError):
        job_runs_on_linux(job)


@pytest.mark.parametrize("mutation", ["remove", "late", "conditional", "soft", "new_job"])
def test_ci_installer_mutations_are_detected(mutation: str) -> None:
    """Representative missing or weakened installer routes must fail closed."""
    ci = copy.deepcopy(workflow("ci.yml"))
    steps = ci["jobs"]["build-test"]["steps"]
    index = next(i for i, step in enumerate(steps) if step.get("uses") == SETUP_RUST)
    if mutation == "remove":
        steps.pop(index)
    elif mutation == "late":
        steps.append(steps.pop(index))
    elif mutation == "conditional":
        steps[index]["if"] = "github.event_name == 'push'"
    elif mutation == "soft":
        steps[index]["continue-on-error"] = True
    else:
        ci["jobs"]["new-linux-suite"] = {
            "runs-on": {"group": "builders", "labels": ["Linux"]},
            "steps": [{"run": "make test"}],
        }
    with pytest.raises(AssertionError):
        validate_suite_provisioning(ci)


@pytest.mark.parametrize(
    "runner",
    ["ubuntu-latest", ["self-hosted", "Linux"], {"group": "builders", "labels": ["Linux"]}],
)
def test_new_suite_job_in_each_runner_form_needs_an_installer(runner: object) -> None:
    """A new job cannot pass merely by changing the runs-on YAML shape."""
    ci = copy.deepcopy(workflow("ci.yml"))
    ci["jobs"]["new-suite"] = {"runs-on": runner, "steps": [{"run": "make all"}]}
    with pytest.raises(AssertionError):
        validate_suite_provisioning(ci)


def validate_mutation_setup(job: dict[str, object]) -> None:
    """Check the reusable workflow's supported pre-suite setup hook."""
    assert "/mutation-cargo.yml@" in job["uses"]
    commands = job["with"]["setup-commands"].splitlines()
    install = commands.index("make install-build-tools")
    check = commands.index("make check-build-tools")
    flags = next(i for i, line in enumerate(commands) if "GITHUB_ENV" in line)
    assert install < check < flags
    assert all(flag in commands[flags] for flag in DEV_FLAGS)
    assert "clang" in " ".join(commands[:install])
    assert "|| true" not in "\n".join(commands)
    assert "continue-on-error" not in job


def test_mutation_reusable_route_provisions_and_restates_flags() -> None:
    """The reusable job's setup hook installs before its later mutant runs."""
    validate_mutation_setup(workflow("mutation-testing.yml")["jobs"]["mutation"])


@pytest.mark.parametrize("mutation", ["remove", "reorder", "soften", "lose_flag"])
def test_mutation_setup_regressions_are_detected(mutation: str) -> None:
    """The reusable caller cannot silently lose its pre-suite prerequisites."""
    job = copy.deepcopy(workflow("mutation-testing.yml")["jobs"]["mutation"])
    commands = job["with"]["setup-commands"].splitlines()
    if mutation == "remove":
        commands.remove("make install-build-tools")
    elif mutation == "reorder":
        commands.append(commands.pop(commands.index("make install-build-tools")))
    elif mutation == "soften":
        commands[commands.index("make install-build-tools")] += " || true"
    else:
        commands[-1] = commands[-1].replace("-Zthreads=8", "")
    job["with"]["setup-commands"] = "\n".join(commands)
    with pytest.raises((AssertionError, ValueError)):
        validate_mutation_setup(job)


def test_missing_tool_reports_an_installation_route() -> None:
    """The local preflight must fail before compilation if Rust is absent."""
    result = subprocess.run(
        ["/bin/bash", "scripts/check-build-tools.sh"],
        cwd=ROOT,
        env={"PATH": "/usr/bin:/bin"},
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode != 0
    assert "make install-build-tools" in result.stderr


@pytest.mark.skipif(
    platform.system() != "Linux" or platform.machine() not in LINKER_DIGESTS,
    reason="the pinned linker is only provisioned for supported Linux architectures",
)
def test_installer_repairs_a_missing_linker_companion(tmp_path: Path) -> None:
    """A marker and `mold` alone must not make the installer exit early."""
    prefix = tmp_path / "tools"
    bin_dir = prefix / "bin"
    bin_dir.mkdir(parents=True)
    version = "2.41.0"
    digest = LINKER_DIGESTS[platform.machine()]
    (bin_dir / f".mold-{version}-{digest}").touch()
    linker_binary = bin_dir / "mold"
    linker_binary.write_text(f"#!/bin/sh\necho 'mold {version} (test)'\n", encoding="utf-8")
    linker_binary.chmod(0o755)

    tools = tmp_path / "commands"
    tools.mkdir()
    (tools / "rustup").write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
    curl = tools / "curl"
    curl.write_text("#!/bin/sh\ntouch \"$INSTALLER_CURL_MARKER\"\nexit 22\n", encoding="utf-8")
    for command in tools.iterdir():
        command.chmod(0o755)

    curl_marker = tmp_path / "curl-ran"
    result = subprocess.run(
        ["/bin/bash", "scripts/install-build-tools.sh"],
        cwd=ROOT,
        env={
            **os.environ,
            "BUILD_TOOLS_PREFIX": str(prefix),
            "INSTALLER_CURL_MARKER": str(curl_marker),
            "PATH": f"{tools}:{os.environ['PATH']}",
        },
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode != 0
    assert curl_marker.exists(), "the missing ld.mold must trigger a reinstall"


@pytest.mark.skipif(platform.system() != "Linux", reason="the pinned linker is Linux-only")
def test_clang_wrapper_selects_pinned_linker_and_excludes_release(tmp_path: Path) -> None:
    """Clang must resolve the PATH linker, not a stale system ld.mold."""
    linker = shutil.which("ld.mold")
    assert linker is not None, "the build-tool installer must precede this test"
    wrapper = ROOT / "scripts" / "clang-linker.sh"
    common = [str(wrapper), "-###", "-x", "c", "/dev/null", "-o", str(tmp_path / "probe")]
    dev = subprocess.run(
        [*common, "-fuse-ld=mold"], capture_output=True, text=True, check=True
    )
    assert f'"{linker}"' in dev.stderr
    release = subprocess.run(common, capture_output=True, text=True, check=True)
    assert "ld.mold" not in release.stderr


@pytest.mark.skipif(platform.system() != "Linux", reason="the pinned linker is Linux-only")
def test_clang_wrapper_rejects_an_unpinned_linker(tmp_path: Path) -> None:
    """A PATH entry with an older linker cannot silently satisfy the wrapper."""
    fake = tmp_path / "ld.mold"
    fake.write_text("#!/bin/sh\necho 'mold 2.40.4 (untrusted)'\n", encoding="utf-8")
    fake.chmod(0o755)
    result = subprocess.run(
        [str(ROOT / "scripts" / "clang-linker.sh"), "-###", "-fuse-ld=mold"],
        cwd=ROOT,
        env={"PATH": f"{tmp_path}:{os.environ['PATH']}"},
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode != 0
    assert "Pinned ld.mold 2.41.0 is required" in result.stderr
