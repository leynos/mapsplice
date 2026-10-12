"""Compile-fail contracts for the repository's Rust lint policy."""

from __future__ import annotations

import json
import os
import subprocess
import tomllib
from dataclasses import dataclass
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
LINTS = {
    "clippy": {
        "disallowed_methods": "deny",
        "missing_assert_message": "deny",
        "missing_docs_in_private_items": "deny",
    },
    "rust": {
        "unknown_lints": "deny",
        "renamed_and_removed_lints": "deny",
        "unsafe_code": "forbid",
        "missing_docs": "deny",
    },
    "rustdoc": {
        "missing_crate_level_docs": "deny",
        "broken_intra_doc_links": "deny",
        "private_intra_doc_links": "deny",
        "bare_urls": "deny",
        "invalid_html_tags": "deny",
        "invalid_codeblock_attributes": "deny",
        "unescaped_backticks": "deny",
    },
}


@dataclass(frozen=True)
class LintCase:
    """One expected lint, source location, and optional Clippy config."""

    name: str
    lint: str
    source: str
    marker: str
    clippy_config: str = ""


def _repository_lints() -> dict[str, dict[str, str]]:
    """Return configured lint levels after checking required policies."""
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    configured = manifest.get("lints", {})
    missing = {
        f"{section}.{name}": (expected, configured.get(section, {}).get(name))
        for section, names in LINTS.items()
        for name, expected in names.items()
        if configured.get(section, {}).get(name) != expected
    }
    assert not missing, f"Cargo.toml lint policy differs from required levels: {missing!r}"
    return {
        section: {name: configured[section][name] for name in names}
        for section, names in LINTS.items()
    }


def _write_probe(
    directory: Path,
    source: str,
    clippy_config: str = "",
) -> Path:
    """Create a dependency-free crate using this repository's lint levels."""
    lints = _repository_lints()
    lines = [
        "[package]",
        f'name = "mapsplice-lint-probe-{directory.name}"',
        'version = "0.1.0"',
        'edition = "2024"',
        "",
        "[workspace]",
        "",
    ]
    for section, values in lints.items():
        lines.append(f"[lints.{section}]")
        lines.extend(f'{name} = "{level}"' for name, level in values.items())
        lines.append("")
    manifest = directory / "Cargo.toml"
    directory.mkdir(parents=True, exist_ok=True)
    manifest.write_text("\n".join(lines), encoding="utf-8")
    (directory / "src").mkdir()
    (directory / "src" / "lib.rs").write_text(source, encoding="utf-8")
    if clippy_config:
        (directory / "clippy.toml").write_text(clippy_config, encoding="utf-8")
    return manifest


def _cargo_environment() -> dict[str, str]:
    """Keep shared Cargo caches, pinned rustc, and compile admission active."""
    environment = os.environ.copy()
    toolchain = tomllib.loads(
        (ROOT / "rust-toolchain.toml").read_text(encoding="utf-8")
    )["toolchain"]["channel"]
    environment["RUSTUP_TOOLCHAIN"] = toolchain
    environment["CARGO_TARGET_DIR"] = environment.get(
        "CARGO_TARGET_DIR", str(ROOT / "target")
    )
    return environment


def _run_route(route: str, manifest: Path) -> subprocess.CompletedProcess[str]:
    """Run the requested compiler route against one isolated probe crate."""
    common = ["cargo", route, "--manifest-path", str(manifest), "--lib"]
    if route == "clippy":
        common.extend(["--message-format=json"])
    elif route == "doc":
        common.extend(["--no-deps", "--message-format=json"])
    else:
        common.extend(["--message-format=json"])
    return subprocess.run(
        common,
        cwd=manifest.parent,
        env=_cargo_environment(),
        text=True,
        capture_output=True,
        check=False,
    )


def _lint_diagnostics(
    result: subprocess.CompletedProcess[str],
) -> list[tuple[str, str, int]]:
    """Return lint identifiers and primary source locations from Cargo JSON."""
    diagnostics = []
    for line in f"{result.stdout}\n{result.stderr}".splitlines():
        try:
            message = json.loads(line).get("message")
        except (json.JSONDecodeError, AttributeError):
            continue
        if not isinstance(message, dict):
            continue
        code = message.get("code")
        lint = code.get("code") if isinstance(code, dict) else None
        if not isinstance(lint, str):
            continue
        for span in message.get("spans", []):
            if span.get("is_primary"):
                diagnostics.append(
                    (lint.rsplit("::", maxsplit=1)[-1], span.get("file_name", ""), span.get("line_start", 0))
                )
    return diagnostics


def _assert_lint_failure(
    result: subprocess.CompletedProcess[str],
    case: LintCase,
) -> None:
    """Require the intended lint and fixture line, not an unrelated failure."""
    output = f"{result.stdout}\n{result.stderr}"
    assert result.returncode != 0, (
        f"route accepted {case.name} fixture; output was {output!r}"
    )
    expected_line = case.source.splitlines().index(case.marker) + 1
    diagnostics = _lint_diagnostics(result)
    assert any(
        lint == case.lint
        and Path(file_name).as_posix().endswith("src/lib.rs")
        and line == expected_line
        for lint, file_name, line in diagnostics
    ), (
        f"route omitted {case.lint!r} at src/lib.rs:{expected_line} for "
        f"{case.name}; observed {diagnostics!r}; output was {output!r}"
    )


CLIPPY_CASES = (
    LintCase(
        "missing-assert-message",
        "missing_assert_message",
        '#![doc = "Clippy lint fixture."]\n'
        "/// Assert readiness without a custom message.\n"
        "pub fn lint_probe(ready: bool) {\n"
        "    assert!(ready);\n"
        "}\n",
        "    assert!(ready);",
    ),
    LintCase(
        "disallowed-method",
        "disallowed_methods",
        '#![doc = "Clippy lint fixture."]\n'
        "/// Call the method listed by the test-only Clippy configuration.\n"
        "pub fn lint_probe() {\n"
        "    let _instant = std::time::Instant::now();\n"
        "}\n",
        "    let _instant = std::time::Instant::now();",
        'disallowed-methods = ["std::time::Instant::now"]\n',
    ),
    LintCase(
        "missing-private-item-docs",
        "missing_docs_in_private_items",
        '#![doc = "Clippy lint fixture."]\n'
        "/// Call the private helper.\n"
        "pub fn lint_probe() { private_helper(); }\n"
        "fn private_helper() {}\n",
        "fn private_helper() {}",
    ),
)

COMPILER_CASES = (
    LintCase(
        "unsafe-code",
        "unsafe_code",
        '#![doc = "Compiler lint fixture."]\n'
        "/// Read a value through an unsafe operation.\n"
        "pub fn lint_probe(value: &u8) -> u8 {\n"
        "    unsafe { core::ptr::read_volatile(value) }\n"
        "}\n",
        "    unsafe { core::ptr::read_volatile(value) }",
    ),
    LintCase(
        "unknown-lint",
        "unknown_lints",
        '#![doc = "Compiler lint fixture."]\n'
        "#![warn(mapsplice_nonexistent_policy_lint)]\n"
        "/// Return a value.\n"
        "pub fn lint_probe() -> u8 { 1 }\n",
        "#![warn(mapsplice_nonexistent_policy_lint)]",
    ),
    LintCase(
        "removed-lint",
        "renamed_and_removed_lints",
        '#![doc = "Compiler lint fixture."]\n'
        "#![warn(private_in_public)]\n"
        "/// Return a value.\n"
        "pub fn lint_probe() -> u8 { 1 }\n",
        "#![warn(private_in_public)]",
    ),
)

RUSTDOC_CASES = (
    LintCase(
        "missing-crate-docs",
        "missing_crate_level_docs",
        "/// A documented public type.\n"
        "pub struct PublicType;\n",
        "/// A documented public type.",
    ),
    LintCase(
        "missing-public-docs",
        "missing_docs",
        '#![doc = "Rustdoc lint fixture."]\n'
        "pub struct UndocumentedType;\n",
        "pub struct UndocumentedType;",
    ),
    LintCase(
        "broken-intra-doc-link",
        "broken_intra_doc_links",
        '#![doc = "Rustdoc lint fixture."]\n'
        "/// References [`MissingType`].\n"
        "pub fn lint_probe() {}\n",
        "/// References [`MissingType`].",
    ),
    LintCase(
        "private-intra-doc-link",
        "private_intra_doc_links",
        '#![doc = "Rustdoc lint fixture."]\n'
        "struct PrivateType;\n"
        "/// References [`PrivateType`].\n"
        "pub fn lint_probe() {}\n",
        "/// References [`PrivateType`].",
    ),
    LintCase(
        "bare-url",
        "bare_urls",
        '#![doc = "Rustdoc lint fixture."]\n'
        "/// Visit https://example.com for information.\n"
        "pub fn lint_probe() {}\n",
        "/// Visit https://example.com for information.",
    ),
    LintCase(
        "invalid-html-tag",
        "invalid_html_tags",
        '#![doc = "Rustdoc lint fixture."]\n'
        "/// An unclosed <span> element.\n"
        "pub fn lint_probe() {}\n",
        "/// An unclosed <span> element.",
    ),
    LintCase(
        "invalid-codeblock-attribute",
        "invalid_codeblock_attributes",
        '#![doc = "Rustdoc lint fixture."]\n'
        "/// ```rust,should-panic\n"
        "/// panic!();\n"
        "/// ```\n"
        "pub fn lint_probe() {}\n",
        "/// ```rust,should-panic",
    ),
    LintCase(
        "unescaped-backtick",
        "unescaped_backticks",
        '#![doc = "Rustdoc lint fixture."]\n'
        "/// An unmatched ` backtick.\n"
        "pub fn lint_probe() {}\n",
        "/// An unmatched ` backtick.",
    ),
)


@pytest.mark.parametrize("case", CLIPPY_CASES, ids=lambda case: case.name)
def test_clippy_rejects_each_representative_policy_violation(
    tmp_path: Path,
    case: LintCase,
) -> None:
    """Clippy diagnostics must name the configured violation and source line."""
    manifest = _write_probe(tmp_path, case.source, case.clippy_config)
    _assert_lint_failure(_run_route("clippy", manifest), case)


@pytest.mark.parametrize("case", COMPILER_CASES, ids=lambda case: case.name)
def test_compiler_rejects_each_representative_policy_violation(
    tmp_path: Path,
    case: LintCase,
) -> None:
    """Compiler diagnostics must name the denied lint and source line."""
    manifest = _write_probe(tmp_path, case.source)
    _assert_lint_failure(_run_route("check", manifest), case)


@pytest.mark.parametrize("case", RUSTDOC_CASES, ids=lambda case: case.name)
def test_rustdoc_rejects_each_representative_policy_violation(
    tmp_path: Path,
    case: LintCase,
) -> None:
    """Rustdoc diagnostics must name each policy violation and source line."""
    manifest = _write_probe(tmp_path, case.source)
    _assert_lint_failure(_run_route("doc", manifest), case)


@pytest.mark.parametrize("route", ("clippy", "check", "doc"))
def test_each_lint_route_accepts_a_valid_fixture(tmp_path: Path, route: str) -> None:
    """The configured lint denials must accept one valid fixture per route."""
    source = (
        '#![doc = "A valid lint-policy fixture."]\n'
        "/// A public type used by the function docs.\n"
        "pub struct PublicType;\n"
        "/// Returns a [`PublicType`] without a policy violation.\n"
        "pub fn lint_probe() -> PublicType { PublicType }\n"
    )
    manifest = _write_probe(
        tmp_path,
        source,
        'disallowed-methods = ["std::time::Instant::now"]\n',
    )
    result = _run_route(route, manifest)
    output = f"{result.stdout}\n{result.stderr}"
    assert result.returncode == 0, (
        f"{route} rejected its valid policy fixture: {output!r}"
    )
