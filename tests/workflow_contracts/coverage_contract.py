"""Validate the local coverage publisher and its pull-request trust boundary."""

from __future__ import annotations

import re
from collections.abc import Mapping
from pathlib import Path

import yaml

COVERAGE_GENERATE_PIN = "2ddf19b3fd52c0f1089db0d57b015f859b80ab67"
UPLOADER_PIN = "2ddf19b3fd52c0f1089db0d57b015f859b80ab67"
SETUP_RUST_PIN = "2ddf19b3fd52c0f1089db0d57b015f859b80ab67"
DEPENDABOT_WORKFLOW = (
    "leynos/shared-actions/.github/workflows/dependabot-automerge.yml"
)
GENERATE = (
    f"leynos/shared-actions/.github/actions/generate-coverage@{COVERAGE_GENERATE_PIN}"
)
UPLOAD = (
    f"leynos/shared-actions/.github/actions/upload-codescene-coverage@{UPLOADER_PIN}"
)
TOKEN_CHECK = (
    'echo "available=${{ secrets.CS_ACCESS_TOKEN != \'\' }}" >> "$GITHUB_OUTPUT"'
)
TOKEN_INPUT = "${{ secrets.CS_ACCESS_TOKEN }}"
SKIP_WARNING_IF = "steps.codescene-token.outputs.available != 'true'"
SKIP_WARNING_RUN = (
    'echo "::warning::CodeScene coverage was not published because the '
    'environment token is unavailable."'
)
UPLOAD_GUARD = (
    "steps.codescene-token.outputs.available == 'true' && "
    "github.ref == 'refs/heads/main'"
)
WORKFLOW_ROOT = Path(__file__).resolve().parents[2] / ".github" / "workflows"


class StrictLoader(yaml.SafeLoader):
    """Read YAML while rejecting duplicate mapping keys, including ``on``."""


def _mapping(loader: StrictLoader, node: yaml.MappingNode) -> dict[object, object]:
    loader.flatten_mapping(node)
    result: dict[object, object] = {}
    for key_node, value_node in node.value:
        key = loader.construct_object(key_node, deep=True)
        if key in result:
            raise ValueError(f"duplicate YAML key: {key!r}")
        result[key] = loader.construct_object(value_node, deep=True)
    return result


StrictLoader.add_constructor(yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, _mapping)


def parse_workflow(source: str) -> dict[object, object]:
    """Read one complete workflow, refusing ambiguous or empty input."""
    document = yaml.load(source, Loader=StrictLoader)
    assert isinstance(document, dict) and document, "workflow must be a mapping"
    assert not ({"on", True} <= set(document)), "ambiguous on/boolean trigger keys"
    return document


def read_workflows(root: Path = WORKFLOW_ROOT) -> dict[str, dict[object, object]]:
    """Read every workflow in the selected repository directory."""
    paths = sorted((*root.glob("*.yml"), *root.glob("*.yaml")))
    assert paths, "no workflows found"
    return {
        path.name: parse_workflow(path.read_text(encoding="utf-8")) for path in paths
    }


def _triggers(workflow: Mapping[object, object]) -> dict[str, object]:
    raw = workflow.get("on", workflow.get(True))
    if isinstance(raw, str):
        return {raw: None}
    if isinstance(raw, list):
        assert raw and all(isinstance(item, str) for item in raw), (
            "invalid trigger list"
        )
        return dict.fromkeys(raw)
    assert isinstance(raw, dict) and raw, "workflow triggers are missing"
    assert all(isinstance(key, str) for key in raw), "invalid trigger key"
    return raw


def _jobs(workflow: Mapping[object, object]) -> dict[str, dict[str, object]]:
    jobs = workflow.get("jobs")
    assert isinstance(jobs, dict) and jobs, "workflow jobs are missing"
    assert all(isinstance(job, dict) for job in jobs.values()), "invalid job"
    return jobs


def _steps(job: Mapping[str, object]) -> list[dict[str, object]]:
    steps = job.get("steps")
    assert isinstance(steps, list) and all(isinstance(step, dict) for step in steps), (
        "job steps are missing or malformed"
    )
    return steps


def _serialized(value: object) -> str:
    if isinstance(value, Mapping):
        return " ".join(f"{key} {_serialized(item)}" for key, item in value.items())
    if isinstance(value, list):
        return " ".join(map(_serialized, value))
    return str(value)


def _validate_external_workflow_call(
    job: Mapping[str, object], called: str
) -> None:
    assert not called.startswith("leynos/mapsplice/"), (
        "qualified self-call cannot be proved local"
    )
    called_workflow, separator, ref = called.partition("@")
    assert (
        separator
        and called_workflow == DEPENDABOT_WORKFLOW
        and re.fullmatch(r"[0-9a-f]{40}", ref)
    ), f"unproved external workflow call: {called}"
    assert "secrets" not in job, "external PR call passes secrets"


def _local_workflow_calls(
    workflows: Mapping[str, dict[object, object]], path: str
) -> set[str]:
    targets = set()
    for job in _jobs(workflows[path]).values():
        called = job.get("uses")
        if called is None:
            continue
        assert isinstance(called, str), "job uses must be a literal"
        if called.startswith("./.github/workflows/"):
            target = called.removeprefix("./.github/workflows/")
            assert target in workflows, f"local workflow call is missing: {target}"
            targets.add(target)
        else:
            _validate_external_workflow_call(job, called)
    return targets


def _workflow_run_sources(
    workflows: Mapping[str, dict[object, object]],
    names: Mapping[object, str],
    active_names: set[object],
) -> set[str]:
    targets = set()
    for path, workflow in workflows.items():
        triggers = _triggers(workflow)
        if "workflow_run" not in triggers:
            continue
        run = triggers["workflow_run"]
        assert isinstance(run, dict), "workflow_run must list source workflows"
        source_names = run.get("workflows")
        assert isinstance(source_names, list) and source_names, (
            "workflow_run sources are indeterminate"
        )
        assert all(source in names for source in source_names), (
            "workflow_run references unknown workflow"
        )
        if active_names.intersection(source_names):
            targets.add(path)
    return targets


def pr_reachable(workflows: Mapping[str, dict[object, object]]) -> set[str]:
    """Resolve local workflow calls and workflow-run chains from PR events."""
    names = {document.get("name"): path for path, document in workflows.items()}
    assert len(names) == len(workflows), "workflow names must be unique"
    reached = {
        path
        for path, workflow in workflows.items()
        if {"pull_request", "pull_request_target"} & _triggers(workflow).keys()
    }
    assert reached, "no pull-request workflow found"
    while True:
        previous = reached.copy()
        for path in previous:
            reached.update(_local_workflow_calls(workflows, path))
        active_names = {workflows[path].get("name") for path in reached}
        reached.update(_workflow_run_sources(workflows, names, active_names))
        if reached == previous:
            return reached


def _coverage_steps(workflow: Mapping[object, object]) -> list[dict[str, object]]:
    return [
        step
        for job in _jobs(workflow).values()
        if "steps" in job
        for step in _steps(job)
        if "generate-coverage@" in str(step.get("uses", ""))
    ]


def _validate_pr_reachable_workflows(
    workflows: Mapping[str, dict[object, object]], reachable: set[str]
) -> None:
    for path in reachable:
        content = _serialized(workflows[path]).lower()
        assert not any(
            term in content
            for term in (
                "codescene",
                "cs-coverage",
                "cs_access_token",
                "upload-codescene",
            )
        ), f"PR-reachable CodeScene route in {path}"
        assert "secrets" not in content, f"unproved PR secret route in {path}"
        for step in _coverage_steps(workflows[path]):
            assert step.get("with", {}).get("publish-baseline") != "always", (
                f"PR workflow {path} could write a coverage baseline"
            )


def _pr_coverage_step(
    workflows: Mapping[str, dict[object, object]],
) -> tuple[dict[str, object], dict[str, object]]:
    coverage_steps = _coverage_steps(workflows["ci.yml"])
    assert len(coverage_steps) == 1, "PR lane needs exactly one coverage run"
    coverage = coverage_steps[0]
    assert coverage.get("uses") == GENERATE, "PR coverage pin is wrong"
    inputs = coverage.get("with")
    assert isinstance(inputs, dict)
    assert inputs.get("with-ratchet") == "true"
    assert inputs.get("publish-artefact") == "false"
    assert "publish-baseline" not in inputs, "PR could override baseline publication"
    return coverage, inputs


def _publisher_job(
    main: Mapping[object, object]
) -> tuple[dict[str, object], list[dict[str, object]]]:
    assert _triggers(main) == {"push": {"branches": ["main"]}}, (
        "publisher must run only on main pushes"
    )
    assert main.get("permissions") == {}, "publisher default permissions must be empty"
    assert main.get("concurrency") == {
        "group": "${{ github.workflow }}-${{ github.ref }}",
        "cancel-in-progress": False,
    }, "publisher concurrency must queue by workflow and ref"
    jobs = _jobs(main)
    assert list(jobs) == ["coverage-publisher"], "publisher must have one job"
    job = jobs["coverage-publisher"]
    assert job.get("environment") in ("codescene", {"name": "codescene"}), (
        ".github/workflows/coverage-main.yml job coverage-publisher must "
        "declare the protected codescene environment; its owner must provision "
        "and freshly verify main-only protection before readiness or merge "
        "(scripts/verify_codescene_environment.py)"
    )
    assert job.get("permissions") == {"contents": "read"}
    assert "CS_ACCESS_TOKEN" not in _serialized(job.get("env", {}))
    return job, _steps(job)


def _validate_exclusive_environment(
    workflows: Mapping[str, dict[object, object]],
) -> None:
    for path, workflow in workflows.items():
        for name, job in _jobs(workflow).items():
            if path != "coverage-main.yml" or name != "coverage-publisher":
                assert job.get("environment") not in (
                    "codescene",
                    {"name": "codescene"},
                ), f"unexpected codescene environment in {path}:{name}"


def _publisher_coverage_step(
    main: Mapping[object, object],
    pr_coverage: Mapping[str, object],
    pr_inputs: Mapping[str, object],
) -> dict[str, object]:
    coverage_steps = _coverage_steps(main)
    assert len(coverage_steps) == 1 and coverage_steps[0].get("uses") == GENERATE
    coverage = coverage_steps[0]
    inputs = coverage.get("with")
    assert isinstance(inputs, dict)
    assert inputs == pr_inputs, "PR and main coverage inputs differ"
    assert (
        coverage.get("env") == pr_coverage.get("env") == {"RUSTFLAGS": "-D warnings"}
    ), "PR and main coverage compiler routes differ"
    return coverage


def _validate_build_tool_order(
    main_steps: list[dict[str, object]], coverage: Mapping[str, object]
) -> None:
    setup = [step for step in main_steps if "setup-rust@" in str(step.get("uses", ""))]
    assert len(setup) == 1 and setup[0].get("uses") == (
        f"leynos/shared-actions/.github/actions/setup-rust@{SETUP_RUST_PIN}"
    )
    assert setup[0].get("with") == {"install-mold": "true", "rustflags": ""}
    preflight = [
        step for step in main_steps if step.get("run") == "make check-build-tools"
    ]
    assert len(preflight) == 1, "publisher suite needs build-tool preflight"
    assert not any(key in preflight[0] for key in ("if", "continue-on-error"))
    assert main_steps.index(setup[0]) < main_steps.index(preflight[0]) < main_steps.index(
        coverage
    )


def _named_step(
    steps: list[dict[str, object]], name: str, error: str
) -> dict[str, object]:
    selected = [step for step in steps if step.get("name") == name]
    assert len(selected) == 1, error
    return selected[0]


def _publisher_token_steps(
    steps: list[dict[str, object]],
) -> tuple[dict[str, object], dict[str, object]]:
    token = _named_step(steps, "Check CodeScene token", "Check CodeScene token step must be unique")
    assert token.get("id") == "codescene-token" and token.get("run") == TOKEN_CHECK
    assert not any(key in token for key in ("env", "uses", "continue-on-error", "if"))
    warning = _named_step(
        steps, "Report skipped CodeScene upload", "missing-token warning must be unique"
    )
    assert warning == {
        "name": "Report skipped CodeScene upload",
        "if": SKIP_WARNING_IF,
        "run": SKIP_WARNING_RUN,
    }, "missing-token warning must be fixed and secret-free"
    return token, warning


def _publisher_upload_step(
    steps: list[dict[str, object]],
    coverage: Mapping[str, object],
    token: Mapping[str, object],
    warning: Mapping[str, object],
) -> dict[str, object]:
    uploads = [
        step
        for step in steps
        if "upload-codescene-coverage@" in str(step.get("uses", ""))
    ]
    assert len(uploads) == 1 and uploads[0].get("uses") == UPLOAD
    upload = uploads[0]
    assert (
        steps.index(coverage)
        < steps.index(token)
        < steps.index(warning)
        < steps.index(upload)
    ), "publisher steps are out of order"
    assert upload.get("if") == UPLOAD_GUARD
    assert not any(key in upload for key in ("env", "run", "continue-on-error")), (
        "upload may receive CS_ACCESS_TOKEN only through its action input"
    )
    assert upload.get("with") == {
        "path": "lcov.info",
        "format": "lcov",
        "mode": "upload",
        "access-token": TOKEN_INPUT,
    }
    return upload


def _validate_publisher_secret_routes(
    main: Mapping[object, object],
    steps: list[dict[str, object]],
    token: Mapping[str, object],
    upload: Mapping[str, object],
) -> None:
    for step in steps:
        if step is not token and step is not upload:
            assert "CS_ACCESS_TOKEN" not in _serialized(step), (
                "only the token check and uploader may refer to the secret"
            )
    metadata = {key: value for key, value in main.items() if key != "jobs"}
    assert "CS_ACCESS_TOKEN" not in _serialized(metadata)


def _validate_only_main_writer(
    workflows: Mapping[str, dict[object, object]], reachable: set[str]
) -> None:
    writers = [
        path
        for path, workflow in workflows.items()
        if path not in reachable
        and any(
            step.get("with", {}).get("with-ratchet") == "true"
            for step in _coverage_steps(workflow)
        )
    ]
    assert writers == ["coverage-main.yml"], f"baseline writers: {writers}"
    all_uploads = [
        path
        for path, workflow in workflows.items()
        for job in _jobs(workflow).values()
        if "steps" in job
        for step in _steps(job)
        if "upload-codescene-coverage@" in str(step.get("uses", ""))
    ]
    assert all_uploads == ["coverage-main.yml"], f"uploaders: {all_uploads}"


def validate(workflows: Mapping[str, dict[object, object]]) -> None:
    """Enforce the PR reader, sole main writer and protected upload shape."""
    assert "ci.yml" in workflows and "coverage-main.yml" in workflows
    reachable = pr_reachable(workflows)
    assert "ci.yml" in reachable and "coverage-main.yml" not in reachable
    _validate_pr_reachable_workflows(workflows, reachable)
    pr_coverage, pr_inputs = _pr_coverage_step(workflows)
    main = workflows["coverage-main.yml"]
    _, main_steps = _publisher_job(main)
    _validate_exclusive_environment(workflows)
    coverage = _publisher_coverage_step(main, pr_coverage, pr_inputs)
    _validate_build_tool_order(main_steps, coverage)
    token, warning = _publisher_token_steps(main_steps)
    upload = _publisher_upload_step(main_steps, coverage, token, warning)
    _validate_publisher_secret_routes(main, main_steps, token, upload)
    _validate_only_main_writer(workflows, reachable)
