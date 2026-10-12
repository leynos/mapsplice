"""Exercise Make's spelling and Markdown lint routes with recording tools."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def _write_recorder(directory: Path, name: str) -> tuple[Path, Path, str]:
    """Create an executable stub that records its arguments as JSON."""
    executable = directory / name
    arguments_file = directory / f"{name}.json"
    arguments_variable = f"{name.upper()}_ARGUMENTS_FILE"
    executable.write_text(
        f"#!{sys.executable}\n"
        "import json\n"
        "import os\n"
        "import sys\n"
        "from pathlib import Path\n"
        f"Path(os.environ[{arguments_variable!r}]).write_text("
        "json.dumps(sys.argv[1:]), encoding='utf-8')\n",
        encoding="utf-8",
    )
    executable.chmod(0o755)
    return executable, arguments_file, arguments_variable


def _make_environment() -> dict[str, str]:
    """Return a predictable Make environment without inherited Make options."""
    environment = os.environ.copy()
    for name in (
        "MAKEFLAGS",
        "MFLAGS",
        "GNUMAKEFLAGS",
        "TYPOS_CONFIG_BUILDER_VERSION",
    ):
        environment.pop(name, None)
    environment["CARGO"] = "/bin/true"
    environment["CARGO_BUILD_TARGET"] = ""
    return environment


def _run_make(target: str, environment: dict[str, str]) -> None:
    """Run one Make target and include its output if the recipe fails."""
    result = subprocess.run(
        ["make", "--no-print-directory", target],
        cwd=ROOT,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, (
        f"make {target} failed with {result.returncode}:\n"
        f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
    )


def test_spelling_uses_pinned_builder_and_all_repository_scope(tmp_path: Path) -> None:
    """Run Make's spelling recipe through a stub uv executable."""
    uv, arguments_file, arguments_variable = _write_recorder(tmp_path, "uv")
    environment = _make_environment()
    environment["UV"] = str(uv)
    environment[arguments_variable] = str(arguments_file)

    _run_make("spelling", environment)

    assert json.loads(arguments_file.read_text(encoding="utf-8")) == [
        "tool",
        "run",
        "--python",
        "3.14",
        "--from",
        "git+https://github.com/leynos/typos-config-builder.git@v0.1.3",
        "typos-config-builder",
        "gate",
        "--repository",
        ".",
        "--scope",
        "all",
    ]


def test_markdownlint_runs_without_invoking_spelling_tool(tmp_path: Path) -> None:
    """Keep Markdown lint separate from the spelling-builder route."""
    uv, uv_arguments_file, uv_arguments_variable = _write_recorder(tmp_path, "uv")
    (
        markdownlint,
        markdownlint_arguments_file,
        markdownlint_arguments_variable,
    ) = _write_recorder(tmp_path, "markdownlint")
    environment = _make_environment()
    environment.update(
        {
            "UV": str(uv),
            "MDLINT": str(markdownlint),
            uv_arguments_variable: str(uv_arguments_file),
            markdownlint_arguments_variable: str(markdownlint_arguments_file),
        }
    )

    _run_make("markdownlint", environment)

    assert json.loads(markdownlint_arguments_file.read_text(encoding="utf-8")) == [
        "**/*.md"
    ]
    assert not uv_arguments_file.exists(), (
        "make markdownlint must not invoke the spelling tool"
    )
