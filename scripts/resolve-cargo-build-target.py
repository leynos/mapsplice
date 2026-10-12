#!/usr/bin/env python3
"""Resolve Cargo's inherited ``build.target`` configuration conservatively."""

from __future__ import annotations

import os
import sys
from pathlib import Path

import tomllib


class ConfigurationError(Exception):
    """Describe a Cargo configuration that cannot safely select a linker."""


def configuration_file_exists(path: Path) -> bool:
    """Check a candidate config without treating inspection errors as absence.

    For example, a missing config returns ``False``, while a symlink loop
    raises ``ConfigurationError`` instead of silently selecting the default.

    Parameters
    ----------
    path : Path
        Candidate Cargo configuration path to inspect.

    Returns
    -------
    bool
        ``True`` if the path can be inspected, or ``False`` if it is missing
        or one of its parent directories does not exist.

    Raises
    ------
    ConfigurationError
        If inspecting the path fails for another operating-system error.
    """
    try:
        path.stat()
    except (FileNotFoundError, NotADirectoryError):
        return False
    except OSError as error:
        detail = error.strerror or type(error).__name__
        raise ConfigurationError(
            f"cannot inspect Cargo configuration {path}: {detail}"
        ) from error
    return True


def resolve_path(path: Path, description: str) -> Path:
    """Resolve one Cargo search path with a configuration diagnostic.

    For example, an inaccessible search root raises ``ConfigurationError``
    before any target can be selected from an incomplete chain.

    Parameters
    ----------
    path : Path
        Path to resolve.
    description : str
        Human-readable name included in a configuration error diagnostic.

    Returns
    -------
    Path
        The resolved path.

    Raises
    ------
    ConfigurationError
        If path resolution fails with an operating-system error or a symlink
        loop.
    """
    try:
        return path.resolve()
    except (OSError, RuntimeError) as error:
        detail = getattr(error, "strerror", None) or type(error).__name__
        raise ConfigurationError(
            f"cannot resolve {description} {path}: {detail}"
        ) from error


def cargo_configuration_path(cargo_directory: Path) -> Path | None:
    """Return one config from a Cargo directory, refusing ambiguous names."""
    legacy = cargo_directory / "config"
    modern = cargo_directory / "config.toml"
    legacy_exists = configuration_file_exists(legacy)
    modern_exists = configuration_file_exists(modern)
    if legacy_exists and modern_exists:
        raise ConfigurationError(
            f"ambiguous Cargo configuration in {cargo_directory}: both config and config.toml exist"
        )
    if modern_exists:
        return modern
    if legacy_exists:
        return legacy
    return None


def configuration_path(directory: Path) -> Path | None:
    """Return one ancestor Cargo config file from its enclosing directory."""
    return cargo_configuration_path(directory / ".cargo")


def cargo_home() -> Path:
    """Resolve Cargo's home directory without changing its cache location."""
    try:
        configured = os.environ.get("CARGO_HOME")
        if configured:
            return Path(configured).expanduser()
        return Path.home() / ".cargo"
    except (OSError, RuntimeError) as error:
        detail = getattr(error, "strerror", None) or type(error).__name__
        raise ConfigurationError(f"cannot determine Cargo home: {detail}") from error


def current_working_directory() -> Path:
    """Return the current directory or a safe configuration diagnostic.

    For example, a deleted or inaccessible working directory fails before
    target selection instead of leaking a traceback from the CLI entry point.

    Returns
    -------
    Path
        The resolved current working directory.

    Raises
    ------
    ConfigurationError
        If the current directory cannot be read or resolved.
    """
    try:
        working_directory = Path.cwd()
    except (OSError, RuntimeError) as error:
        detail = getattr(error, "strerror", None) or type(error).__name__
        raise ConfigurationError(
            f"cannot determine current working directory: {detail}"
        ) from error
    return resolve_path(working_directory, "current working directory")


def _search_directories(start: Path, root_directory: Path | None) -> list[Path]:
    start = resolve_path(start, "configuration search start")
    if root_directory is None:
        return [start, *start.parents]
    root = resolve_path(root_directory, "workspace root")
    if not start.is_relative_to(root):
        raise ConfigurationError(
            f"Cargo configuration search start {start} is outside root {root}"
        )
    directories = [start]
    while directories[-1] != root:
        directories.append(directories[-1].parent)
    return directories


def configuration_paths(
    start: Path,
    *,
    cargo_home_directory: Path | None = None,
    root_directory: Path | None = None,
) -> list[Path]:
    """List Cargo configuration files from lowest to highest precedence."""
    directories = _search_directories(start, root_directory)

    cargo_directory = cargo_home_directory or cargo_home()
    cargo_directory = resolve_path(cargo_directory, "Cargo home")
    paths = [cargo_configuration_path(cargo_directory)]
    paths.extend(configuration_path(directory) for directory in reversed(directories))
    unique_paths: list[Path] = []
    for path in paths:
        if path is not None and path not in unique_paths:
            unique_paths.append(path)
    return unique_paths


def _read_configuration(path: Path) -> dict[str, object]:
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError) as error:
        raise ConfigurationError(
            f"cannot read Cargo configuration {path}: {error}"
        ) from error


def _build_table(document: dict[str, object], path: Path) -> dict[str, object] | None:
    build = document.get("build")
    if build is None:
        return None
    if not isinstance(build, dict):
        raise ConfigurationError(f"invalid [build] table in Cargo configuration {path}")
    return build


def _target_value(build: dict[str, object] | None, path: Path) -> str | None:
    if build is None:
        return None
    target = build.get("target")
    if target is None:
        return None
    if not isinstance(target, str) or not target:
        raise ConfigurationError(
            f"unsupported build.target in {path}; set one non-empty target triple"
        )
    return target


def configured_target(path: Path) -> str | None:
    """Read one config's scalar build target or reject an unsafe form."""
    document = _read_configuration(path)
    return _target_value(_build_table(document, path), path)


def resolve_target(
    start: Path,
    *,
    cargo_home_directory: Path | None = None,
    root_directory: Path | None = None,
) -> str | None:
    """Apply Cargo's nearer-directory-over-home configuration precedence."""
    target = None
    for path in configuration_paths(
        start,
        cargo_home_directory=cargo_home_directory,
        root_directory=root_directory,
    ):
        selected = configured_target(path)
        if selected is not None:
            target = selected
    return target


def main() -> int:
    """Write the effective inherited target for Make's linker routing."""
    try:
        target = resolve_target(current_working_directory())
    except ConfigurationError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    if target is not None:
        print(target)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
