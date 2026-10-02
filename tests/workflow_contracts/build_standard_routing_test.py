"""Exercise Make's evaluated target and encoded-rustflag routing with probes."""

from __future__ import annotations

import os
import platform
import subprocess
import sys
from pathlib import Path

import pytest
import tomllib
from build_route_probe import (
    COMPATIBLE_FLAGS,
    RouteInvocation,
    assert_development_route,
    assert_whitaker_route,
    cargo_home_with_target,
    execute_make_route,
    parse_invocations,
    route_for,
)
from build_standard_test import DEV_FLAGS, ROOT, make_plan

__all__ = ["RouteInvocation", "parse_invocations"]

RESOLVER = ROOT / "scripts" / "resolve-cargo-build-target.py"


def assert_no_cranelift_cargo_default(
    config: dict[str, object],
    toolchain: dict[str, object],
) -> None:
    """Reject Cranelift selection in development profiles or toolchain components."""
    profiles = config.get("profile", {})
    assert isinstance(profiles, dict)
    for name in ("dev", "test"):
        profile = profiles.get(name, {})
        assert isinstance(profile, dict)
        assert profile.get("codegen-backend") != "cranelift"

    toolchain_config = toolchain.get("toolchain", {})
    assert isinstance(toolchain_config, dict)
    components = toolchain_config.get("components", [])
    assert isinstance(components, list)
    assert "rustc-codegen-cranelift-preview" not in components


def test_cargo_defaults_keep_the_frontend_without_cranelift() -> None:
    """Target flags retain the frontend while Cargo leaves development on LLVM."""
    config = tomllib.loads((ROOT / ".cargo" / "config.toml").read_text())
    toolchain = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())
    assert config["build"]["rustflags"] == ["-Zthreads=8"]
    assert_no_cranelift_cargo_default(config, toolchain)
    for triple, expected_linker in (
        ("x86_64-unknown-linux-gnu", "scripts/clang-linker.sh"),
        ("aarch64-unknown-linux-gnu", "scripts/clang-aarch64-linker.sh"),
    ):
        target = config["target"][triple]
        flags = " ".join(target["rustflags"])
        assert target["linker"] == expected_linker
        assert "codegen-backend=cranelift" not in flags
        assert all(flag in flags for flag in DEV_FLAGS)


@pytest.mark.parametrize(
    "mutation", ["dev_profile", "test_profile", "toolchain_component"]
)
def test_cranelift_cargo_default_mutations_are_detected(mutation: str) -> None:
    """A Cranelift development profile or component must fail the contract."""
    config = tomllib.loads((ROOT / ".cargo" / "config.toml").read_text())
    toolchain = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())
    if mutation in {"dev_profile", "test_profile"}:
        profile_name = "dev" if mutation == "dev_profile" else "test"
        config.setdefault("profile", {}).setdefault(profile_name, {})[
            "codegen-backend"
        ] = "cranelift"
    else:
        toolchain["toolchain"]["components"].append("rustc-codegen-cranelift-preview")
    with pytest.raises(AssertionError):
        assert_no_cranelift_cargo_default(config, toolchain)


@pytest.mark.parametrize("goal", ["test", "lint", "typecheck"])
def test_development_make_plans_preflight_before_cargo(goal: str) -> None:
    """Dry runs prove only each development gate's ordering and reachability."""
    lines = make_plan(goal, DOC_TEST_TARGETS="true")
    cargo_indices = [
        index for index, line in enumerate(lines) if "probe-cargo " in line
    ]
    assert cargo_indices, f"{goal} has no Cargo invocation"
    assert lines.index("bash scripts/check-build-tools.sh") < min(cargo_indices)


def test_nextest_and_doctest_record_distinct_development_routes(tmp_path: Path) -> None:
    """Nextest and doctests must each receive evaluated development flags."""
    lines = make_plan(
        "test", DOC_TEST_TARGETS="true", TEST_CMD="nextest run --no-tests pass"
    )
    assert next(
        i for i, line in enumerate(lines) if "probe-cargo nextest run" in line
    ) < next(i for i, line in enumerate(lines) if "probe-cargo test --doc" in line)
    invocations = execute_make_route(
        tmp_path,
        "test",
        variables={
            "DOC_TEST_TARGETS": "true",
            "TEST_CMD": "nextest run --no-tests pass",
        },
    )
    uses_pinned_linker = platform.system() == "Linux" and platform.machine() in {
        "x86_64",
        "aarch64",
    }
    assert_development_route(
        route_for(invocations, ("nextest", "run")),
        uses_pinned_linker=uses_pinned_linker,
    )
    doctest = route_for(invocations, ("test", "--doc"))
    assert doctest.arguments == ("test", "--doc", "--workspace", "--all-features")
    assert_development_route(doctest, uses_pinned_linker=uses_pinned_linker)
    assert doctest.rustdoc_flags == ("--cfg", "docsrs", "-D", "warnings")


def test_rustdoc_and_clippy_record_distinct_development_routes(tmp_path: Path) -> None:
    """Documentation and Clippy each preserve their own evaluated route."""
    invocations = execute_make_route(tmp_path, "lint-clippy")
    uses_pinned_linker = platform.system() == "Linux" and platform.machine() in {
        "x86_64",
        "aarch64",
    }
    documentation = route_for(invocations, ("doc",))
    assert documentation.arguments == ("doc", "--workspace", "--no-deps")
    assert_development_route(documentation, uses_pinned_linker=uses_pinned_linker)
    assert documentation.rustdoc_flags == ("--cfg", "docsrs", "-D", "warnings")
    assert_development_route(
        route_for(invocations, ("clippy",)), uses_pinned_linker=uses_pinned_linker
    )


def test_build_preflights_and_restores_development_defaults(tmp_path: Path) -> None:
    """The ordinary build preflights before its executable development route."""
    lines = make_plan("build")
    build_index = next(
        index for index, line in enumerate(lines) if "probe-cargo build" in line
    )
    assert lines.index("bash scripts/check-build-tools.sh") < build_index
    uses_pinned_linker = platform.system() == "Linux" and platform.machine() in {
        "x86_64",
        "aarch64",
    }
    assert_development_route(
        route_for(execute_make_route(tmp_path, "build"), ("build",)),
        uses_pinned_linker=uses_pinned_linker,
    )


def test_inherited_make_dry_run_flags_do_not_skip_route_probe(tmp_path: Path) -> None:
    """The child Make must execute a Cargo probe despite outer dry-run flags."""
    invocations = execute_make_route(
        tmp_path,
        "typecheck",
        environment={"MAKEFLAGS": "-n", "GNUMAKEFLAGS": "-n", "MFLAGS": "-n"},
    )
    assert route_for(invocations, ("check",)).arguments == (
        "check",
        "--workspace",
        "--all-targets",
        "--all-features",
    )


@pytest.mark.parametrize(
    ("variables", "should_use_pinned_linker"),
    [
        (
            {
                "CARGO_FLAGS": "--workspace --config build.target=aarch64-unknown-linux-gnu"
            },
            True,
        ),
        ({"CARGO_FLAGS": "--config=build.target=aarch64-unknown-linux-gnu"}, True),
        ({"CARGO_BUILD_TARGET": "x86_64-pc-windows-gnu"}, False),
        ({"CARGO_FLAGS": "--config build.target=x86_64-pc-windows-gnu"}, False),
        (
            {
                "CARGO_FLAGS": "--target aarch64-unknown-linux-gnu --target x86_64-pc-windows-gnu"
            },
            False,
        ),
        (
            {
                "CARGO_FLAGS": "--target aarch64-unknown-linux-gnu",
                "CARGO_BUILD_TARGET": "x86_64-pc-windows-gnu",
            },
            False,
        ),
    ],
)
def test_target_selectors_choose_only_one_supported_linux_linker_route(
    tmp_path: Path,
    variables: dict[str, str],
    should_use_pinned_linker: bool,
) -> None:
    """CLI, environment and --config selectors fail closed when they conflict."""
    route = route_for(
        execute_make_route(tmp_path, "typecheck", variables=variables), ("check",)
    )
    assert_development_route(route, uses_pinned_linker=should_use_pinned_linker)


@pytest.mark.parametrize(
    ("goal", "variables", "command"),
    [
        (
            "test",
            {
                "TEST_CMD": "test",
                "TEST_FLAGS": "--workspace --target x86_64-pc-windows-gnu",
            },
            ("test",),
        ),
        (
            "typecheck",
            {"CARGO_FLAGS": "--workspace --target x86_64-pc-windows-gnu"},
            ("check",),
        ),
        (
            "lint-clippy",
            {
                "CLIPPY_FLAGS": "--workspace --target x86_64-pc-windows-gnu -- -D warnings",
            },
            ("clippy",),
        ),
    ],
)
def test_windows_development_routes_exclude_inherited_linker(
    tmp_path: Path,
    goal: str,
    variables: dict[str, str],
    command: tuple[str, ...],
) -> None:
    """Test, typecheck and Clippy must not retain the Linux linker for a Windows target."""
    route = route_for(
        execute_make_route(
            tmp_path,
            goal,
            variables=variables,
        ),
        command,
    )
    assert_development_route(route, uses_pinned_linker=False)


@pytest.mark.parametrize(
    ("target", "should_use_pinned_linker"),
    [
        ("x86_64-pc-windows-gnu", False),
        ("aarch64-unknown-linux-gnu", True),
    ],
)
def test_cargo_home_target_controls_the_evaluated_route(
    tmp_path: Path,
    target: str,
    should_use_pinned_linker: bool,
) -> None:
    """A config-only effective target cannot silently inherit the native linker."""
    environment = cargo_home_with_target(tmp_path, target)
    route = route_for(
        execute_make_route(
            tmp_path,
            "typecheck",
            environment=environment,
        ),
        ("check",),
    )
    assert_development_route(route, uses_pinned_linker=should_use_pinned_linker)


def test_cargo_home_target_conflict_fails_closed_for_linker(tmp_path: Path) -> None:
    """An inherited config target and a conflicting environment target lose the Linux linker."""
    route = route_for(
        execute_make_route(
            tmp_path,
            "typecheck",
            variables={"CARGO_BUILD_TARGET": "x86_64-pc-windows-gnu"},
            environment=cargo_home_with_target(tmp_path, "aarch64-unknown-linux-gnu"),
        ),
        ("check",),
    )
    assert_development_route(route, uses_pinned_linker=False)


def test_config_resolver_prefers_nearer_ancestor_over_cargo_home(
    tmp_path: Path,
) -> None:
    """Cargo-home config is lowest precedence beneath ancestor Cargo configs."""
    cargo_home = tmp_path / "cargo-home"
    project = tmp_path / "project"
    child = project / "nested" / "crate"
    cargo_home.mkdir()
    (project / ".cargo").mkdir(parents=True)
    (project / "nested" / ".cargo").mkdir(parents=True)
    child.mkdir(parents=True)
    (cargo_home / "config.toml").write_text(
        '[build]\ntarget = "x86_64-pc-windows-gnu"\n',
        encoding="utf-8",
    )
    (project / ".cargo" / "config.toml").write_text(
        '[build]\ntarget = "aarch64-unknown-linux-gnu"\n',
        encoding="utf-8",
    )
    (project / "nested" / ".cargo" / "config.toml").write_text(
        '[build]\ntarget = "x86_64-unknown-linux-gnu"\n',
        encoding="utf-8",
    )
    result = subprocess.run(
        [sys.executable, str(RESOLVER)],
        cwd=child,
        env={**os.environ, "CARGO_HOME": str(cargo_home)},
        capture_output=True,
        text=True,
        check=True,
    )
    assert result.stdout == "x86_64-unknown-linux-gnu\n"


def test_release_filters_development_flags_and_selects_llvm(tmp_path: Path) -> None:
    """Release keeps compatible policy flags while selecting its LLVM backend."""
    lines = make_plan("release")
    assert any("probe-cargo build" in line for line in lines)
    release = route_for(execute_make_route(tmp_path, "release"), ("build",))
    assert release.encoded_flags == (*COMPATIBLE_FLAGS, "-D", "warnings")
    assert release.rust_flags == ("-D", "warnings")
    assert release.rustdoc_flags == ()
    assert release.release_backend == "llvm"
    assert release.dev_backend == release.test_backend == ""


def test_whitaker_filters_development_flags_and_selects_llvm(tmp_path: Path) -> None:
    """Whitaker keeps policy flags while enforcing its installer-owned route."""
    assert_whitaker_route(
        route_for(execute_make_route(tmp_path, "lint-whitaker"), ("--all",))
    )
