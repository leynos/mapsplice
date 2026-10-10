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
    """
    try:
        working_directory = Path.cwd()
    except (OSError, RuntimeError) as error:
        detail = getattr(error, "strerror", None) or type(error).__name__
        raise ConfigurationError(
            f"cannot determine current working directory: {detail}"
        ) from error
    return resolve_path(working_directory, "current working directory")


def configuration_paths(
    start: Path,
    *,
    cargo_home_directory: Path | None = None,
    root_directory: Path | None = None,
) -> list[Path]:
    """List Cargo configuration files from lowest to highest precedence."""
    start = resolve_path(start, "configuration search start")
    if root_directory is None:
        directories = [start, *start.parents]
    else:
        root = resolve_path(root_directory, "workspace root")
        if not start.is_relative_to(root):
            raise ConfigurationError(
                f"Cargo configuration search start {start} is outside root {root}"
            )
        directories = [start]
        while directories[-1] != root:
            directories.append(directories[-1].parent)

    cargo_directory = cargo_home_directory or cargo_home()
    cargo_directory = resolve_path(cargo_directory, "Cargo home")
    paths = [cargo_configuration_path(cargo_directory)]
    paths.extend(configuration_path(directory) for directory in reversed(directories))
    unique_paths: list[Path] = []
    for path in paths:
        if path is not None and path not in unique_paths:
            unique_paths.append(path)
    return unique_paths


def configured_target(path: Path) -> str | None:
    """Read one config's scalar build target or reject an unsafe form."""
    try:
        document = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise ConfigurationError(
            f"cannot read Cargo configuration {path}: {error}"
        ) from error
    build = document.get("build")
    if build is None:
        return None
    if not isinstance(build, dict):
        raise ConfigurationError(f"invalid [build] table in Cargo configuration {path}")
    target = build.get("target")
    if target is None:
        return None
    if not isinstance(target, str) or not target:
        raise ConfigurationError(
            f"unsupported build.target in {path}; set one non-empty target triple"
        )
    return target


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
