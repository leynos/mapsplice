//! Markdown gate checks for rendered golden-fixture output.

use std::{path::Path, process::Command};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs_utf8::Dir};

use super::TestResult;

const MDTABLEFIX_ARGS: &[&str] = &[
    "--wrap",
    "--renumber",
    "--breaks",
    "--ellipsis",
    "--fences",
    "--in-place",
];

/// Assert rendered Markdown is stable under the house formatter and linter.
pub(crate) fn assert_gate_clean_rendered_output(name: &str, rendered: &str) -> TestResult {
    let tempdir = tempfile::tempdir()?;
    let root = utf8_root(tempdir.path())?;
    let dir = Dir::open_ambient_dir(&root, ambient_authority())?;
    let rendered_path = root.join("rendered.md");
    dir.write("rendered.md", rendered)?;

    assert_formatters_leave_unchanged(name, &dir, &rendered_path)?;
    run_markdownlint(&rendered_path)?;
    Ok(())
}

/// Convert a temporary directory path to UTF-8, naming the path when it is not.
fn utf8_root(path: &Path) -> Result<Utf8PathBuf, String> {
    Utf8PathBuf::from_path_buf(path.to_path_buf()).map_err(|rejected| {
        format!(
            "temporary directory is not valid UTF-8: {}",
            rejected.display()
        )
    })
}

/// Run the formatters over `rendered.md` and fail if they change what a fixture rendered.
fn assert_formatters_leave_unchanged(
    name: &str,
    dir: &Dir,
    rendered_path: &Utf8Path,
) -> TestResult {
    let original = dir.read("rendered.md")?;
    run_mdtablefix(rendered_path)?;
    run_markdownlint_fix(rendered_path)?;
    let formatted = dir.read("rendered.md")?;
    if formatted != original {
        return Err(format!("golden fixture `{name}` formatter changed rendered output").into());
    }
    Ok(())
}

fn run_mdtablefix(path: &Utf8Path) -> TestResult {
    let mut command = Command::new("mdtablefix");
    command.args(MDTABLEFIX_ARGS).arg(path.as_std_path());
    run_command(command)
}

fn run_markdownlint_fix(path: &Utf8Path) -> TestResult {
    let mut command = Command::new("markdownlint-cli2");
    command.arg("--fix").arg(path.as_std_path());
    run_command(command)
}

fn run_markdownlint(path: &Utf8Path) -> TestResult {
    let mut command = Command::new("markdownlint-cli2");
    command.arg(path.as_std_path());
    run_command(command)
}

fn run_command(mut command: Command) -> TestResult {
    let description = format!("{command:?}");
    let output = command.output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "command `{description}` failed with status \
             {status}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            status = output.status,
            stdout = String::from_utf8_lossy(&output.stdout),
            stderr = String::from_utf8_lossy(&output.stderr),
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for rendered-output gate checks.

    use crate::golden::{TestResult, assert_gate_clean_rendered_output};

    #[test]
    fn gate_clean_rendered_output_accepts_stable_markdown() -> TestResult {
        assert_gate_clean_rendered_output(
            "stable_markdown",
            concat!(
                "# Roadmap\n\n",
                "| Name  | Value |\n",
                "| ----- | ----- |\n",
                "| Alpha | Done  |\n",
            ),
        )
    }

    #[test]
    fn gate_clean_rendered_output_rejects_formatter_drift() -> TestResult {
        let error = match assert_gate_clean_rendered_output(
            "unstable_markdown",
            concat!(
                "# Roadmap\n\n",
                "| Name | Value |\n",
                "| --- | --- |\n",
                "| Alpha | Done |\n",
            ),
        ) {
            Ok(()) => return Err("unaligned table should be changed by the formatter".into()),
            Err(error) => error,
        };

        if !error
            .to_string()
            .contains("formatter changed rendered output")
        {
            return Err(format!("unexpected formatter error: {error}").into());
        }
        Ok(())
    }
}
