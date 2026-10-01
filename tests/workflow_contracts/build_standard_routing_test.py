"""Exercise Make's evaluated target and encoded-rustflag routing with probes."""

from __future__ import annotations

from dataclasses import dataclass
import os
import platform
import subprocess
import sys
import tomllib
from pathlib import Path

import pytest

from build_standard_test import DEV_FLAGS, ROOT, make_plan


INHERITED_FLAGS = [
    "--cfg", "caller_policy", "-Z", "threads=8", "-C",
    "link-arg=-fuse-ld=mold", "-Z", "codegen-backend=cranelift",
]
COMPATIBLE_FLAGS = ["--cfg", "caller_policy"]
RESOLVER = ROOT / "scripts" / "resolve-cargo-build-target.py"


@dataclass(frozen=True)
class RouteInvocation:
    """One executable invocation and the compiler environment Make selected."""

    arguments: tuple[str, ...]
    encoded_flags: tuple[str, ...]
    rust_flags: tuple[str, ...]
    rustdoc_flags: tuple[str, ...]
    release_backend: str
    dev_backend: str
    test_backend: str


def execute_make_route(
    tmp_path: Path,
    goal: str,
    *,
    variables: dict[str, str] | None = None,
    environment: dict[str, str] | None = None,
) -> list[RouteInvocation]:
    """Run one Make goal against recording executables instead of Cargo."""
    commands = tmp_path / "commands"
    commands.mkdir()
    probe = commands / "probe"
    probe.write_text(
        "#!/bin/sh\nprintf '%s\\035%s\\035%s\\035%s\\035%s\\035%s\\035%s\\036' "
        "\"$*\" \"${CARGO_ENCODED_RUSTFLAGS-}\" \"${RUSTFLAGS-}\" "
        "\"${RUSTDOCFLAGS-}\" \"${CARGO_PROFILE_RELEASE_CODEGEN_BACKEND-}\" "
        "\"${CARGO_PROFILE_DEV_CODEGEN_BACKEND-}\" "
        "\"${CARGO_PROFILE_TEST_CODEGEN_BACKEND-}\" >> \"$ROUTE_PROBE\"\n",
        encoding="utf-8",
    )
    probe.chmod(0o755)
    # The configured Make wrapper uses env bash, so only bypass preflight.
    shell = commands / "bash"
    shell.write_text(
        "#!/bin/sh\n"
        "if [ \"$1\" = 'scripts/check-build-tools.sh' ]; then exit 0; fi\n"
        "exec /bin/bash \"$@\"\n",
        encoding="utf-8",
    )
    shell.chmod(0o755)
    output = tmp_path / "route"
    cargo_home = tmp_path / "cargo-home"
    cargo_home.mkdir()
    arguments = [
        "make", "--no-print-directory", "--always-make", goal,
        f"CARGO={probe}", f"WHITAKER={probe}", "DOC_TEST_TARGETS=false",
    ]
    arguments.extend(f"{key}={value}" for key, value in (variables or {}).items())
    command_environment = {
        **os.environ,
        "CARGO_ENCODED_RUSTFLAGS": "\x1f".join(INHERITED_FLAGS),
        "CARGO_HOME": str(cargo_home),
        "PATH": f"{commands}:{os.environ['PATH']}",
        "ROUTE_PROBE": str(output),
        "RUST_FLAGS": "",
        "RUSTDOC_FLAGS": "",
    }
    command_environment.pop("CARGO_BUILD_TARGET", None)
    command_environment.update(environment or {})
    # An outer Make may export dry-run flags; this child must execute its probes.
    for make_flags in ("MAKEFLAGS", "GNUMAKEFLAGS", "MFLAGS"):
        command_environment.pop(make_flags, None)
    subprocess.run(arguments, cwd=ROOT, env=command_environment, check=True)
    return parse_invocations(output.read_text(encoding="utf-8"))


def parse_invocations(recording: str) -> list[RouteInvocation]:
    """Decode the append-only probe recording without losing encoded flags."""
    invocations = []
    for record in recording.split("\x1e"):
        if not record:
            continue
        fields = record.split("\x1d")
        assert len(fields) == 7, f"malformed route probe record: {record!r}"
        invocations.append(RouteInvocation(
            arguments=tuple(fields[0].split()),
            encoded_flags=tuple(flag for flag in fields[1].split("\x1f") if flag),
            rust_flags=tuple(fields[2].split()),
            rustdoc_flags=tuple(fields[3].split()),
            release_backend=fields[4],
            dev_backend=fields[5],
            test_backend=fields[6],
        ))
    assert invocations, "Make did not execute a route probe"
    return invocations


def route_for(
    invocations: list[RouteInvocation], expected: tuple[str, ...],
) -> RouteInvocation:
    """Return exactly one recorded command with the requested leading words."""
    matches = [
        invocation for invocation in invocations
        if invocation.arguments[:len(expected)] == expected
    ]
    assert len(matches) == 1, f"expected one {' '.join(expected)} route, got {matches!r}"
    return matches[0]


def assert_development_route(invocation: RouteInvocation, *, uses_pinned_linker: bool) -> None:
    """Require one evaluated development route to restore its selected defaults."""
    expected = [*COMPATIBLE_FLAGS, "-D", "warnings", "-Zthreads=8"]
    rust_flags = ["-D", "warnings", "-Zthreads=8"]
    if uses_pinned_linker:
        expected.extend(["-C", "link-arg=-fuse-ld=mold"])
        rust_flags.extend(["-C", "link-arg=-fuse-ld=mold"])
    assert list(invocation.encoded_flags) == expected
    assert list(invocation.rust_flags) == rust_flags
    assert invocation.release_backend == invocation.dev_backend == invocation.test_backend == ""


def assert_whitaker_route(invocation: RouteInvocation) -> None:
    """Require the installer-managed verification route to remain LLVM-only."""
    assert invocation.arguments == (
        "--all", "--package", "mapsplice", "--", "--workspace", "--all-targets", "--all-features",
    )
    assert invocation.encoded_flags == (*COMPATIBLE_FLAGS, "-D", "warnings")
    assert invocation.rust_flags == ("-D", "warnings")
    assert invocation.rustdoc_flags == ()
    assert invocation.release_backend == ""
    assert invocation.dev_backend == invocation.test_backend == "llvm"


def assert_no_cranelift_cargo_default(
    config: dict[str, object], toolchain: dict[str, object],
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


def cargo_home_with_target(tmp_path: Path, target: str) -> dict[str, str]:
    """Create a CARGO_HOME whose only target selection is the named triple."""
    cargo_home = tmp_path / "configured-cargo-home"
    cargo_home.mkdir()
    (cargo_home / "config.toml").write_text(
        f'[build]\ntarget = "{target}"\n', encoding="utf-8",
    )
    return {"CARGO_HOME": str(cargo_home)}


def test_cargo_defaults_keep_the_frontend_without_cranelift() -> None:
    """Target flags retain the frontend while Cargo leaves development on LLVM."""
    config = tomllib.loads((ROOT / ".cargo" / "config.toml").read_text())
    toolchain = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())
    assert config["build"]["rustflags"] == ["-Zthreads=8"]
    assert_no_cranelift_cargo_default(config, toolchain)
    for triple in ("x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"):
        target = config["target"][triple]
        flags = " ".join(target["rustflags"])
        assert target["linker"] == "scripts/clang-linker.sh"
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
        toolchain["toolchain"]["components"].append(
            "rustc-codegen-cranelift-preview"
        )
    with pytest.raises(AssertionError):
        assert_no_cranelift_cargo_default(config, toolchain)


@pytest.mark.parametrize("goal", ["test", "lint", "typecheck"])
def test_development_make_plans_preflight_before_cargo(goal: str) -> None:
    """Dry runs prove only each development gate's ordering and reachability."""
    lines = make_plan(goal, DOC_TEST_TARGETS="true")
    cargo_indices = [index for index, line in enumerate(lines) if "probe-cargo " in line]
    assert cargo_indices, f"{goal} has no Cargo invocation"
    assert lines.index("bash scripts/check-build-tools.sh") < min(cargo_indices)


def test_nextest_and_doctest_record_distinct_development_routes(tmp_path: Path) -> None:
    """Nextest and doctests must each receive evaluated development flags."""
    lines = make_plan("test", DOC_TEST_TARGETS="true", TEST_CMD="nextest run --no-tests pass")
    assert next(i for i, line in enumerate(lines) if "probe-cargo nextest run" in line) < next(
        i for i, line in enumerate(lines) if "probe-cargo test --doc" in line
    )
    invocations = execute_make_route(
        tmp_path, "test", variables={
            "DOC_TEST_TARGETS": "true", "TEST_CMD": "nextest run --no-tests pass",
        },
    )
    uses_pinned_linker = platform.system() == "Linux" and platform.machine() in {"x86_64", "aarch64"}
    assert_development_route(route_for(invocations, ("nextest", "run")), uses_pinned_linker=uses_pinned_linker)
    doctest = route_for(invocations, ("test", "--doc"))
    assert_development_route(doctest, uses_pinned_linker=uses_pinned_linker)
    assert doctest.rustdoc_flags == ("-D", "warnings")


def test_rustdoc_and_clippy_record_distinct_development_routes(tmp_path: Path) -> None:
    """Documentation and Clippy each preserve their own evaluated route."""
    invocations = execute_make_route(tmp_path, "lint-clippy")
    uses_pinned_linker = platform.system() == "Linux" and platform.machine() in {"x86_64", "aarch64"}
    documentation = route_for(invocations, ("doc",))
    assert_development_route(documentation, uses_pinned_linker=uses_pinned_linker)
    assert documentation.rustdoc_flags == ("-D", "warnings")
    assert_development_route(route_for(invocations, ("clippy",)), uses_pinned_linker=uses_pinned_linker)


def test_build_preflights_and_restores_development_defaults(tmp_path: Path) -> None:
    """The ordinary build preflights before its executable development route."""
    lines = make_plan("build")
    build_index = next(index for index, line in enumerate(lines) if "probe-cargo build" in line)
    assert lines.index("bash scripts/check-build-tools.sh") < build_index
    uses_pinned_linker = platform.system() == "Linux" and platform.machine() in {"x86_64", "aarch64"}
    assert_development_route(route_for(execute_make_route(tmp_path, "build"), ("build",)), uses_pinned_linker=uses_pinned_linker)


def test_inherited_make_dry_run_flags_do_not_skip_route_probe(tmp_path: Path) -> None:
    """The child Make must execute a Cargo probe despite outer dry-run flags."""
    invocations = execute_make_route(
        tmp_path,
        "typecheck",
        environment={"MAKEFLAGS": "-n", "GNUMAKEFLAGS": "-n", "MFLAGS": "-n"},
    )
    assert route_for(invocations, ("check",)).arguments == (
        "check", "--workspace", "--all-targets", "--all-features",
    )


@pytest.mark.parametrize(
    ("variables", "should_use_pinned_linker"),
    [
        ({"CARGO_FLAGS": "--workspace --config build.target=aarch64-unknown-linux-gnu"}, True),
        ({"CARGO_FLAGS": "--config=build.target=aarch64-unknown-linux-gnu"}, True),
        ({"CARGO_BUILD_TARGET": "x86_64-pc-windows-gnu"}, False),
        ({"CARGO_FLAGS": "--config build.target=x86_64-pc-windows-gnu"}, False),
        ({"CARGO_FLAGS": "--target aarch64-unknown-linux-gnu --target x86_64-pc-windows-gnu"}, False),
        ({"CARGO_FLAGS": "--target aarch64-unknown-linux-gnu", "CARGO_BUILD_TARGET": "x86_64-pc-windows-gnu"}, False),
    ],
)
def test_target_selectors_choose_only_one_supported_linux_linker_route(
    tmp_path: Path, variables: dict[str, str], should_use_pinned_linker: bool,
) -> None:
    """CLI, environment and --config selectors fail closed when they conflict."""
    route = route_for(execute_make_route(tmp_path, "typecheck", variables=variables), ("check",))
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
    tmp_path: Path, goal: str, variables: dict[str, str], command: tuple[str, ...],
) -> None:
    """Test, typecheck and Clippy must not retain the Linux linker for a Windows target."""
    route = route_for(execute_make_route(
        tmp_path, goal, variables=variables,
    ), command)
    assert_development_route(route, uses_pinned_linker=False)


@pytest.mark.parametrize(
    ("target", "should_use_pinned_linker"),
    [
        ("x86_64-pc-windows-gnu", False),
        ("aarch64-unknown-linux-gnu", True),
    ],
)
def test_cargo_home_target_controls_the_evaluated_route(
    tmp_path: Path, target: str, should_use_pinned_linker: bool,
) -> None:
    """A config-only effective target cannot silently inherit the native linker."""
    environment = cargo_home_with_target(tmp_path, target)
    route = route_for(execute_make_route(
        tmp_path, "typecheck", environment=environment,
    ), ("check",))
    assert_development_route(route, uses_pinned_linker=should_use_pinned_linker)


def test_cargo_home_target_conflict_fails_closed_for_linker(tmp_path: Path) -> None:
    """An inherited config target and a conflicting environment target lose the Linux linker."""
    route = route_for(execute_make_route(
        tmp_path,
        "typecheck",
        variables={"CARGO_BUILD_TARGET": "x86_64-pc-windows-gnu"},
        environment=cargo_home_with_target(tmp_path, "aarch64-unknown-linux-gnu"),
    ), ("check",))
    assert_development_route(route, uses_pinned_linker=False)


def test_config_resolver_prefers_nearer_ancestor_over_cargo_home(tmp_path: Path) -> None:
    """Cargo-home config is lowest precedence beneath ancestor Cargo configs."""
    cargo_home = tmp_path / "cargo-home"
    project = tmp_path / "project"
    child = project / "nested" / "crate"
    cargo_home.mkdir()
    (project / ".cargo").mkdir(parents=True)
    (project / "nested" / ".cargo").mkdir(parents=True)
    child.mkdir(parents=True)
    (cargo_home / "config.toml").write_text(
        '[build]\ntarget = "x86_64-pc-windows-gnu"\n', encoding="utf-8",
    )
    (project / ".cargo" / "config.toml").write_text(
        '[build]\ntarget = "aarch64-unknown-linux-gnu"\n', encoding="utf-8",
    )
    (project / "nested" / ".cargo" / "config.toml").write_text(
        '[build]\ntarget = "x86_64-unknown-linux-gnu"\n', encoding="utf-8",
    )
    result = subprocess.run(
        [sys.executable, str(RESOLVER)], cwd=child,
        env={**os.environ, "CARGO_HOME": str(cargo_home)},
        capture_output=True, text=True, check=True,
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
    assert_whitaker_route(route_for(execute_make_route(tmp_path, "lint-whitaker"), ("--all",)))
