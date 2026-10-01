"""Validate the local coverage publisher and its pull-request trust boundary."""

from __future__ import annotations

from collections.abc import Mapping
from pathlib import Path

import yaml

COVERAGE_GENERATE_PIN = "abf0dcf2686de1eaf79b6dc9a16662b631bed149"
UPLOADER_PIN = "d4d248bbbecdcf7b4f5bc79ffd4d6caee370bd79"
SETUP_RUST_PIN = "d4d248bbbecdcf7b4f5bc79ffd4d6caee370bd79"
DEPENDABOT_PIN = "ff1dd759dfffc0db3459e30e833f52437ee62b57"
GENERATE = (
    "leynos/shared-actions/.github/actions/generate-coverage@"
    f"{COVERAGE_GENERATE_PIN}"
)
UPLOAD = (
    "leynos/shared-actions/.github/actions/upload-codescene-coverage@"
    f"{UPLOADER_PIN}"
)
DEPENDABOT_AUTOMERGE = (
    "leynos/shared-actions/.github/workflows/dependabot-automerge.yml@"
    f"{DEPENDABOT_PIN}"
)
TOKEN_CHECK = 'echo "available=${{ secrets.CS_ACCESS_TOKEN != \'\' }}" >> "$GITHUB_OUTPUT"'
TOKEN_INPUT = "${{ secrets.CS_ACCESS_TOKEN }}"
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


StrictLoader.add_constructor(
    yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, _mapping
)


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
    return {path.name: parse_workflow(path.read_text(encoding="utf-8")) for path in paths}


def _triggers(workflow: Mapping[object, object]) -> dict[str, object]:
    raw = workflow.get("on", workflow.get(True))
    if isinstance(raw, str):
        return {raw: None}
    if isinstance(raw, list):
        assert raw and all(isinstance(item, str) for item in raw), "invalid trigger list"
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


def pr_reachable(workflows: Mapping[str, dict[object, object]]) -> set[str]:
    """Resolve local workflow calls and workflow-run chains from PR events."""
    names = {document.get("name"): path for path, document in workflows.items()}
    assert len(names) == len(workflows), "workflow names must be unique"
    reached = {
        path for path, workflow in workflows.items()
        if {"pull_request", "pull_request_target"} & _triggers(workflow).keys()
    }
    assert reached, "no pull-request workflow found"
    while True:
        previous = reached.copy()
        for path in previous:
            for job in _jobs(workflows[path]).values():
                called = job.get("uses")
                if called is None:
                    continue
                assert isinstance(called, str), "job uses must be a literal"
                if called.startswith("./.github/workflows/"):
                    target = called.removeprefix("./.github/workflows/")
                    assert target in workflows, f"local workflow call is missing: {target}"
                    reached.add(target)
                else:
                    assert not called.startswith("leynos/mapsplice/"), (
                        "qualified self-call cannot be proved local"
                    )
                    assert called == DEPENDABOT_AUTOMERGE, (
                        f"unproved external workflow call: {called}"
                    )
                    assert "secrets" not in job, "external PR call passes secrets"
        active_names = {workflows[path].get("name") for path in reached}
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
                reached.add(path)
        if reached == previous:
            return reached


def _coverage_steps(workflow: Mapping[object, object]) -> list[dict[str, object]]:
    return [
        step for job in _jobs(workflow).values() if "steps" in job
        for step in _steps(job)
        if "generate-coverage@" in str(step.get("uses", ""))
    ]


def validate(workflows: Mapping[str, dict[object, object]]) -> None:
    """Enforce the PR reader, sole main writer and protected upload shape."""
    assert "ci.yml" in workflows and "coverage-main.yml" in workflows
    reachable = pr_reachable(workflows)
    assert "ci.yml" in reachable and "coverage-main.yml" not in reachable
    for path in reachable:
        content = _serialized(workflows[path]).lower()
        assert not any(term in content for term in (
            "codescene", "cs-coverage", "cs_access_token", "upload-codescene"
        )), (
            f"PR-reachable CodeScene route in {path}"
        )
        assert "secrets" not in content, (
            f"unproved PR secret route in {path}"
        )
        for step in _coverage_steps(workflows[path]):
            assert step.get("with", {}).get("publish-baseline") != "always", (
                f"PR workflow {path} could write a coverage baseline"
            )
    pr_steps = _coverage_steps(workflows["ci.yml"])
    assert len(pr_steps) == 1, "PR lane needs exactly one coverage run"
    pr_coverage = pr_steps[0]
    assert pr_coverage.get("uses") == GENERATE, "PR coverage pin is wrong"
    pr_inputs = pr_coverage.get("with")
    assert isinstance(pr_inputs, dict)
    assert pr_inputs.get("with-ratchet") == "true"
    assert pr_inputs.get("publish-artefact") == "false"
    assert "publish-baseline" not in pr_inputs, "PR could override baseline publication"

    main = workflows["coverage-main.yml"]
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
        "publisher job needs the protected codescene environment"
    )
    assert job.get("permissions") == {"contents": "read"}
    assert "CS_ACCESS_TOKEN" not in _serialized(job.get("env", {}))
    for path, workflow in workflows.items():
        for name, other_job in _jobs(workflow).items():
            if path != "coverage-main.yml" or name != "coverage-publisher":
                assert other_job.get("environment") not in (
                    "codescene", {"name": "codescene"}
                ), f"unexpected codescene environment in {path}:{name}"
    main_steps = _steps(job)
    coverage = _coverage_steps(main)
    assert len(coverage) == 1 and coverage[0].get("uses") == GENERATE
    main_inputs = coverage[0].get("with")
    assert isinstance(main_inputs, dict)
    assert main_inputs == pr_inputs, "PR and main coverage inputs differ"
    assert coverage[0].get("env") == pr_coverage.get("env") == {
        "RUSTFLAGS": "-D warnings"
    }, "PR and main coverage compiler routes differ"
    setup = [step for step in main_steps if "setup-rust@" in str(step.get("uses", ""))]
    assert len(setup) == 1 and setup[0].get("uses") == (
        f"leynos/shared-actions/.github/actions/setup-rust@{SETUP_RUST_PIN}"
    )
    assert setup[0].get("with") == {"install-mold": "true", "rustflags": ""}
    preflight = [step for step in main_steps if step.get("run") == "make check-build-tools"]
    assert len(preflight) == 1, "publisher suite needs build-tool preflight"
    assert not any(key in preflight[0] for key in ("if", "continue-on-error"))
    assert main_steps.index(setup[0]) < main_steps.index(preflight[0]) < main_steps.index(coverage[0])
    assert [step.get("name") for step in main_steps].count("Check CodeScene token") == 1, (
        "Check CodeScene token step must be unique"
    )
    token = next(step for step in main_steps if step.get("name") == "Check CodeScene token")
    assert token.get("id") == "codescene-token" and token.get("run") == TOKEN_CHECK
    assert not any(key in token for key in ("env", "uses", "continue-on-error", "if"))
    uploads = [step for step in main_steps if "upload-codescene-coverage@" in str(step.get("uses", ""))]
    assert len(uploads) == 1 and uploads[0].get("uses") == UPLOAD
    upload = uploads[0]
    assert main_steps.index(coverage[0]) < main_steps.index(token) < main_steps.index(upload)
    assert upload.get("if") == UPLOAD_GUARD
    assert not any(key in upload for key in ("env", "run", "continue-on-error")), (
        "upload may receive CS_ACCESS_TOKEN only through its action input"
    )
    assert upload.get("with") == {
        "path": "lcov.info", "format": "lcov", "mode": "upload",
        "access-token": TOKEN_INPUT,
    }
    for step in main_steps:
        if step is not token and step is not upload:
            assert "CS_ACCESS_TOKEN" not in _serialized(step), (
                "only the token check and uploader may refer to the secret"
            )
    assert "CS_ACCESS_TOKEN" not in _serialized({
        key: value for key, value in main.items() if key != "jobs"
    })
    writers = [path for path, workflow in workflows.items()
               if path not in reachable and any(
                   step.get("with", {}).get("with-ratchet") == "true"
                   for step in _coverage_steps(workflow)
               )]
    assert writers == ["coverage-main.yml"], f"baseline writers: {writers}"
    all_uploads = [path for path, workflow in workflows.items()
                   for job in _jobs(workflow).values() if "steps" in job
                   for step in _steps(job)
                   if "upload-codescene-coverage@" in str(step.get("uses", ""))]
    assert all_uploads == ["coverage-main.yml"], f"uploaders: {all_uploads}"
