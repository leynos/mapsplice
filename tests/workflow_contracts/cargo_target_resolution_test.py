"""Property-test inherited Cargo target selection in isolated trees."""

from __future__ import annotations

import errno
import importlib.util
from pathlib import Path
from typing import NamedTuple

import pytest
from build_standard_test import ROOT
from hypothesis import example, given, settings
from hypothesis import strategies as st

SPEC = importlib.util.spec_from_file_location(
    "cargo_target_resolver", ROOT / "scripts" / "resolve-cargo-build-target.py"
)
assert SPEC is not None and SPEC.loader is not None
RESOLVER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RESOLVER)


class ConfigState(NamedTuple):
    """Files and selected target represented by one Cargo config location."""

    files: tuple[tuple[str, str], ...]
    target: str | None


TARGET = st.text(
    alphabet="abcdefghijklmnopqrstuvwxyz0123456789-_", min_size=1, max_size=32
)
CONFIG_NAME = st.sampled_from(("config", "config.toml"))
INVALID_CONFIG_KINDS = (
    "empty_target",
    "integer_target",
    "array_target",
    "malformed_toml",
    "invalid_build_table",
    "ambiguous_pair",
)
BASELINE_CHAIN = (
    1,
    [
        ConfigState(
            (("config.toml", '[build]\ntarget = "home-target"\n'),), "home-target"
        ),
        ConfigState((), None),
        ConfigState((("config", '[build]\ntarget = "near-target"\n'),), "near-target"),
    ],
)


def valid_config_states() -> st.SearchStrategy[ConfigState]:
    """Generate absent, target-free, and scalar-target configurations."""
    target_file = st.tuples(CONFIG_NAME, TARGET).map(
        lambda item: ConfigState(
            ((item[0], f'[build]\ntarget = "{item[1]}"\n'),), item[1]
        )
    )
    target_free_file = CONFIG_NAME.map(
        lambda name: ConfigState(((name, "[build]\nother = true\n"),), None)
    )
    return st.one_of(st.just(ConfigState((), None)), target_free_file, target_file)


def invalid_config_state(kind: str) -> ConfigState:
    """Build one malformed or unsupported configuration for fail-closed tests."""
    if kind == "empty_target":
        return ConfigState((("config.toml", '[build]\ntarget = ""\n'),), None)
    if kind == "integer_target":
        return ConfigState((("config.toml", "[build]\ntarget = 7\n"),), None)
    if kind == "array_target":
        return ConfigState((("config.toml", '[build]\ntarget = ["x86_64"]\n'),), None)
    if kind == "malformed_toml":
        return ConfigState((("config.toml", '[build\ntarget = "broken"\n'),), None)
    if kind == "invalid_build_table":
        return ConfigState((("config.toml", "build = 7\n"),), None)
    if kind == "ambiguous_pair":
        return ConfigState(
            (
                ("config", '[build]\ntarget = "legacy-target"\n'),
                ("config.toml", '[build]\ntarget = "modern-target"\n'),
            ),
            None,
        )
    raise AssertionError(f"unknown invalid config kind: {kind}")


@st.composite
def valid_chains(draw: st.DrawFn) -> tuple[int, list[ConfigState]]:
    """Generate Cargo-home and nested-ancestor configurations."""
    depth = draw(st.integers(min_value=0, max_value=4))
    count = depth + 2
    states = draw(st.lists(valid_config_states(), min_size=count, max_size=count))
    return depth, states


def write_chain(
    tmp_path_factory: pytest.TempPathFactory,
    depth: int,
    states: list[ConfigState],
) -> tuple[Path, Path, Path, Path]:
    """Create fresh Cargo-home and bounded project ancestor trees."""
    sandbox = tmp_path_factory.mktemp("cargo-target-chain")
    cargo_home = sandbox / "cargo-home"
    root = sandbox / "workspace"
    directories = [root]
    for index in range(depth):
        directories.append(directories[-1] / f"nested-{index}")
    locations = [cargo_home, *directories]
    for index, (location, state) in enumerate(zip(locations, states, strict=True)):
        if not state.files:
            continue
        cargo_directory = location if index == 0 else location / ".cargo"
        cargo_directory.mkdir(parents=True, exist_ok=True)
        for filename, contents in state.files:
            (cargo_directory / filename).write_text(contents, encoding="utf-8")
    directories[-1].mkdir(parents=True, exist_ok=True)
    return sandbox, cargo_home, root, directories[-1]


def reference_target(states: list[ConfigState]) -> str | None:
    """Model Cargo precedence without calling the resolver implementation."""
    selected = None
    for state in states:
        if state.target is not None:
            selected = state.target
    return selected


@settings(max_examples=60, deadline=None)
@example(
    chain=(
        1,
        [
            ConfigState(
                (("config.toml", '[build]\ntarget = "home-target"\n'),), "home-target"
            ),
            ConfigState((), None),
            ConfigState(
                (("config", '[build]\ntarget = "near-target"\n'),), "near-target"
            ),
        ],
    )
)
@given(chain=valid_chains())
def test_generated_valid_chains_match_reference_model(
    tmp_path_factory: pytest.TempPathFactory,
    chain: tuple[int, list[ConfigState]],
) -> None:
    """Nearest targets override; missing files and targets leave selection intact."""
    depth, states = chain
    _, cargo_home, root, start = write_chain(tmp_path_factory, depth, states)

    actual = RESOLVER.resolve_target(
        start,
        cargo_home_directory=cargo_home,
        root_directory=root,
    )

    assert actual == reference_target(states), (
        f"generated source targets were {[state.target for state in states]!r}; "
        f"selected {actual!r}"
    )


@settings(max_examples=36, deadline=None)
@example(invalid_kind="empty_target", chain=BASELINE_CHAIN)
@example(invalid_kind="integer_target", chain=BASELINE_CHAIN)
@example(invalid_kind="array_target", chain=BASELINE_CHAIN)
@example(invalid_kind="malformed_toml", chain=BASELINE_CHAIN)
@example(invalid_kind="invalid_build_table", chain=BASELINE_CHAIN)
@example(invalid_kind="ambiguous_pair", chain=BASELINE_CHAIN)
@given(
    invalid_kind=st.sampled_from(INVALID_CONFIG_KINDS),
    chain=valid_chains(),
)
def test_generated_invalid_configurations_fail_closed(
    tmp_path_factory: pytest.TempPathFactory,
    invalid_kind: str,
    chain: tuple[int, list[ConfigState]],
) -> None:
    """An invalid encountered config fails even when a nearer target is valid."""
    depth, generated_states = chain
    states = list(generated_states)
    if len(states) < 2:
        pytest.fail("generated chain must include Cargo home and a workspace root")
    states[depth] = invalid_config_state(invalid_kind)
    states[-1] = ConfigState(
        (("config.toml", '[build]\ntarget = "near-target"\n'),), "near-target"
    )
    _, cargo_home, root, start = write_chain(tmp_path_factory, depth, states)

    with pytest.raises(RESOLVER.ConfigurationError):
        RESOLVER.resolve_target(
            start,
            cargo_home_directory=cargo_home,
            root_directory=root,
        )


def test_legacy_config_and_missing_target_preserve_home_selection(
    tmp_path_factory: pytest.TempPathFactory,
) -> None:
    """Legacy Cargo home targets survive nearer modern configs without a target."""
    states = [
        ConfigState((("config", '[build]\ntarget = "home-target"\n'),), "home-target"),
        ConfigState((("config.toml", "[build]\nother = true\n"),), None),
        ConfigState(
            (("config", '[build]\ntarget = "nested-target"\n'),), "nested-target"
        ),
    ]
    _, cargo_home, root, start = write_chain(tmp_path_factory, 1, states)

    actual = RESOLVER.resolve_target(
        start,
        cargo_home_directory=cargo_home,
        root_directory=root,
    )

    assert actual == "nested-target"


def test_unreadable_configuration_fails_closed_with_fault_injection(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """A read error is covered deterministically without permission-bit tests."""
    cargo_home = tmp_path / "cargo-home"
    root = tmp_path / "workspace"
    start = root / "nested"
    selected = cargo_home / "config.toml"
    selected.parent.mkdir(parents=True)
    selected.write_text('[build]\ntarget = "home-target"\n', encoding="utf-8")
    start.mkdir(parents=True)
    original_read_text = Path.read_text

    def reject_selected_file(
        path: Path,
        *args: object,
        **kwargs: object,
    ) -> str:
        if path == selected:
            raise PermissionError("injected unreadable Cargo config")
        return original_read_text(path, *args, **kwargs)

    monkeypatch.setattr(Path, "read_text", reject_selected_file)

    with pytest.raises(
        RESOLVER.ConfigurationError, match="cannot read Cargo configuration"
    ):
        RESOLVER.resolve_target(
            start,
            cargo_home_directory=cargo_home,
            root_directory=root,
        )


def test_configuration_discovery_errors_fail_closed(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """A config stat error cannot be mistaken for a missing file."""
    cargo_home = tmp_path / "cargo-home"
    root = tmp_path / "workspace"
    start = root / "nested"
    selected = cargo_home / "config.toml"
    selected.parent.mkdir(parents=True)
    selected.write_text('[build]\ntarget = "home-target"\n', encoding="utf-8")
    start.mkdir(parents=True)
    original_stat = Path.stat

    def reject_selected_config(
        path: Path,
        *args: object,
        **kwargs: object,
    ) -> object:
        if path == selected:
            raise OSError(errno.ELOOP, "injected symlink loop", str(path))
        return original_stat(path, *args, **kwargs)

    monkeypatch.setattr(Path, "stat", reject_selected_config)

    with pytest.raises(
        RESOLVER.ConfigurationError, match="cannot inspect Cargo configuration"
    ):
        RESOLVER.resolve_target(
            start,
            cargo_home_directory=cargo_home,
            root_directory=root,
        )


@pytest.mark.parametrize(
    ("failed_location", "failure_type"),
    (
        ("start", OSError),
        ("root", OSError),
        ("cargo_home", OSError),
        ("start", RuntimeError),
    ),
)
def test_path_resolution_errors_fail_closed(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    failed_location: str,
    failure_type: type[Exception],
) -> None:
    """Every search-root resolution failure has a configuration diagnostic."""
    cargo_home = tmp_path / "cargo-home"
    root = tmp_path / "workspace"
    start = root / "nested"
    start.mkdir(parents=True)
    failed_path = {
        "start": start,
        "root": root,
        "cargo_home": cargo_home,
    }[failed_location]
    expected_description = {
        "start": "configuration search start",
        "root": "workspace root",
        "cargo_home": "Cargo home",
    }[failed_location]
    original_resolve = Path.resolve

    def reject_selected_path(path: Path, *args: object, **kwargs: object) -> Path:
        if path == failed_path:
            if failure_type is RuntimeError:
                raise RuntimeError("injected path resolution failure")
            raise OSError(errno.EACCES, "injected path resolution failure", str(path))
        return original_resolve(path, *args, **kwargs)

    monkeypatch.setattr(Path, "resolve", reject_selected_path)

    with pytest.raises(
        RESOLVER.ConfigurationError,
        match=f"cannot resolve {expected_description}",
    ):
        RESOLVER.resolve_target(
            start,
            cargo_home_directory=cargo_home,
            root_directory=root,
        )


def test_cli_converts_current_directory_failure_to_diagnostic(
    monkeypatch: pytest.MonkeyPatch,
    capfd: pytest.CaptureFixture[str],
) -> None:
    """A cwd lookup error exits cleanly without an unhandled traceback."""
    def reject_current_directory(cls: type[Path]) -> Path:
        del cls
        raise OSError(errno.ENOENT, "injected missing current directory")

    monkeypatch.setattr(Path, "cwd", classmethod(reject_current_directory))

    exit_code = RESOLVER.main()

    captured = capfd.readouterr()
    assert exit_code == 2, f"CLI exit was {exit_code}, expected configuration failure"
    assert captured.out == "", f"configuration failure wrote stdout: {captured.out!r}"
    assert "cannot determine current working directory" in captured.err, (
        f"cwd failure diagnostic was {captured.err!r}"
    )
    assert "Traceback" not in captured.err, (
        f"cwd failure leaked a traceback: {captured.err!r}"
    )
