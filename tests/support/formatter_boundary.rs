//! Property-test helpers for formatter-boundary roadmap bodies.

use std::process::Command;

use camino::Utf8PathBuf;
use cap_std::{ambient_authority, fs_utf8::Dir};
use proptest::prelude::*;

#[derive(Clone, Debug)]
pub(crate) enum GateCleanBoundaryShape {
    LooseList { first: String, second: String },
    IndentedCodeFence { value: String },
}

impl GateCleanBoundaryShape {
    pub(crate) fn target(&self) -> String {
        let body = match self {
            Self::LooseList { first, second } => format!("- {first}\n\n- {second}\n"),
            Self::IndentedCodeFence { value } => format!(" ```rust\n {value}\n ```\n"),
        };
        render_with_body(&body)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum FormatterUnstableBoundaryShape {
    RepeatedOrderedMarkers { first: String, second: String },
    OverindentedNestedList { parent: String, child: String },
    TildeFence { value: String },
    OversizedBacktickFence { value: String },
}

impl FormatterUnstableBoundaryShape {
    pub(crate) fn target(&self) -> String {
        let body = match self {
            Self::RepeatedOrderedMarkers { first, second } => {
                format!("1. {first}\n1. {second}\n")
            }
            Self::OverindentedNestedList { parent, child } => {
                format!("- {parent}\n    - {child}\n")
            }
            Self::TildeFence { value } => format!("~~~rust\n{value}\n~~~\n"),
            Self::OversizedBacktickFence { value } => format!("````rust\n{value}\n````\n"),
        };
        render_with_body(&body)
    }

    pub(crate) fn expected(&self) -> String {
        let body = match self {
            Self::RepeatedOrderedMarkers { first, second } => {
                format!("1. {first}\n2. {second}\n")
            }
            Self::OverindentedNestedList { parent, child } => {
                format!("- {parent}\n\n  - {child}\n")
            }
            Self::TildeFence { value } | Self::OversizedBacktickFence { value } => {
                format!("```rust\n{value}\n```\n")
            }
        };
        render_with_body(&body)
    }
}

pub(crate) fn gate_clean_boundary_shape() -> impl Strategy<Value = GateCleanBoundaryShape> {
    prop_oneof![
        ("[a-z]{1,12}", "[a-z]{1,12}")
            .prop_map(|(first, second)| GateCleanBoundaryShape::LooseList { first, second }),
        "[a-z]{1,12}".prop_map(|value| GateCleanBoundaryShape::IndentedCodeFence { value }),
    ]
}

pub(crate) fn formatter_unstable_boundary_shape()
-> impl Strategy<Value = FormatterUnstableBoundaryShape> {
    prop_oneof![
        ("[a-z]{1,12}", "[a-z]{1,12}").prop_map(|(first, second)| {
            FormatterUnstableBoundaryShape::RepeatedOrderedMarkers { first, second }
        }),
        ("[a-z]{1,12}", "[a-z]{1,12}").prop_map(|(parent, child)| {
            FormatterUnstableBoundaryShape::OverindentedNestedList { parent, child }
        }),
        "[a-z]{1,12}".prop_map(|value| FormatterUnstableBoundaryShape::TildeFence { value }),
        "[a-z]{1,12}"
            .prop_map(|value| FormatterUnstableBoundaryShape::OversizedBacktickFence { value }),
    ]
}

pub(crate) fn assert_house_format_noop(rendered: &str) -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(|error| format!("create temporary dir: {error}"))?;
    let root = Utf8PathBuf::from_path_buf(tempdir.path().to_path_buf())
        .map_err(|path| format!("temporary directory is not valid UTF-8: {}", path.display()))?;
    let dir = Dir::open_ambient_dir(&root, ambient_authority())
        .map_err(|error| format!("open temporary dir: {error}"))?;
    let rendered_path = root.join("rendered.md");
    dir.write("rendered.md", rendered)
        .map_err(|error| format!("write rendered Markdown: {error}"))?;
    let original = dir
        .read_to_string("rendered.md")
        .map_err(|error| format!("read rendered Markdown: {error}"))?;

    run_format_command("mdtablefix", &rendered_path, &MDTABLEFIX_ARGS)?;
    run_format_command("markdownlint-cli2", &rendered_path, &["--fix"])?;

    let formatted = dir
        .read_to_string("rendered.md")
        .map_err(|error| format!("read formatted Markdown: {error}"))?;
    if formatted == original {
        Ok(())
    } else {
        Err(format!(
            "house formatter changed rendered output\nexpected:\n{original}\nactual:\n{formatted}"
        ))
    }
}

fn render_with_body(body: &str) -> String {
    format!(
        "# Roadmap\n\n## 1. Phase one\n\n{body}\n### 1.1. Step one\n\n- [ ] 1.1.1. Existing \
         task.\n"
    )
}

const MDTABLEFIX_ARGS: [&str; 6] = [
    "--wrap",
    "--renumber",
    "--breaks",
    "--ellipsis",
    "--fences",
    "--in-place",
];

fn run_format_command(
    command: &str,
    rendered_path: &Utf8PathBuf,
    arguments: &[&str],
) -> Result<(), String> {
    let output = Command::new(command)
        .args(arguments)
        .arg(rendered_path.as_std_path())
        .output()
        .map_err(|error| format!("run {command}: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{command} failed with status {status}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            status = output.status,
            stdout = String::from_utf8_lossy(&output.stdout),
            stderr = String::from_utf8_lossy(&output.stderr),
        ))
    }
}
