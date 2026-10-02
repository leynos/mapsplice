#!/usr/bin/env python3
"""Resolve Cargo's inherited ``build.target`` configuration conservatively."""

from __future__ import annotations

import os
from pathlib import Path

import tomllib


class ConfigurationError(Exception):
    """Describe a Cargo configuration that cannot safely select a linker."""


def cargo_configuration_path(cargo_directory: Path) -> Path | None:
    """Return one config from a Cargo directory, refusing ambiguous names."""
    legacy = cargo_directory / "config"
    modern = cargo_directory / "config.toml"
    if legacy.exists() and modern.exists():
        raise ConfigurationError(
            f"ambiguous Cargo configuration in {cargo_directory}: both config and config.toml exist"
        )
    if modern.exists():
        return modern
    if legacy.exists():
        return legacy
    return None


def configuration_path(directory: Path) -> Path | None:
    """Return one ancestor Cargo config file from its enclosing directory."""
    return cargo_configuration_path(directory / ".cargo")


def cargo_home() -> Path:
    """Resolve Cargo's home directory without changing its cache location."""
    configured = os.environ.get("CARGO_HOME")
    if configured:
        return Path(configured).expanduser().resolve()
    return Path.home() / ".cargo"


def configuration_paths(
    start: Path,
    *,
    cargo_home_directory: Path | None = None,
    root_directory: Path | None = None,
) -> list[Path]:
    """List Cargo configuration files from lowest to highest precedence."""
    start = start.resolve()
    if root_directory is None:
        directories = [start, *start.parents]
    else:
        root = root_directory.resolve()
        if not start.is_relative_to(root):
            raise ConfigurationError(
                f"Cargo configuration search start {start} is outside root {root}"
            )
        directories = [start]
        while directories[-1] != root:
            directories.append(directories[-1].parent)

    cargo_directory = cargo_home_directory or cargo_home()
    paths = [cargo_configuration_path(cargo_directory.resolve())]
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
        target = resolve_target(Path.cwd().resolve())
    except ConfigurationError as error:
        print(f"error: {error}")
        return 2
    if target is not None:
        print(target)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
