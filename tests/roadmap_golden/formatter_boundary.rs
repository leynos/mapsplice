//! Tests for the F1/F4 preservation boundary around accepted Markdown.

use camino::Utf8Path;
use cap_std::{ambient_authority, fs_utf8::Dir};
use mapsplice::run_from_args;
use rstest::{fixture, rstest};

use super::golden::{GoldenWorkspace, TestResult, create_workspace};

#[fixture]
fn workspace() -> TestResult<GoldenWorkspace> {
    let workspace = create_workspace()?;
    Ok(workspace)
}

#[test]
fn golden_corpus_has_no_formatter_boundary_surprises() -> TestResult {
    let repository = Dir::open_ambient_dir(env!("CARGO_MANIFEST_DIR"), ambient_authority())?;
    let mut findings = Vec::new();
    scan_fixture_dir(
        &repository,
        Utf8Path::new("tests/fixtures/golden"),
        &mut findings,
    )?;

    if findings.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "golden corpus contains formatter-boundary surfaces:\n{}",
            findings.join("\n")
        )
        .into())
    }
}

#[rstest]
#[serial_test::serial(cli_env)]
fn gate_clean_loose_list_survives_noop_replace(
    workspace: TestResult<GoldenWorkspace>,
) -> TestResult {
    assert_noop_replace_preserves_target(&workspace?, gate_clean_loose_list_target())
}

#[rstest]
#[serial_test::serial(cli_env)]
fn gate_clean_indented_code_fence_survives_noop_replace(
    workspace: TestResult<GoldenWorkspace>,
) -> TestResult {
    assert_noop_replace_preserves_target(&workspace?, gate_clean_indented_code_fence_target())
}

#[rstest]
#[case::repeated_ordered_markers(
    repeated_ordered_markers_target(),
    repeated_ordered_markers_expected()
)]
#[case::overindented_nested_list(
    overindented_nested_list_target(),
    overindented_nested_list_expected()
)]
#[case::tilde_fence(tilde_fence_target(), canonical_fence_expected())]
#[case::oversized_backtick_fence(oversized_fence_target(), canonical_fence_expected())]
#[serial_test::serial(cli_env)]
fn formatter_unstable_input_normalizes_on_noop_replace(
    workspace: TestResult<GoldenWorkspace>,
    #[case] target: &str,
    #[case] expected: &str,
) -> TestResult {
    assert_noop_replace_output(&workspace?, target, expected)
}

fn scan_fixture_dir(
    repository: &Dir,
    directory: &Utf8Path,
    findings: &mut Vec<String>,
) -> TestResult {
    for directory_entry in repository.read_dir(directory)? {
        let fixture_entry = directory_entry?;
        let file_name = fixture_entry.file_name()?;
        let fixture_path = directory.join(file_name.as_str());
        let file_type = fixture_entry.file_type()?;
        if file_type.is_dir() {
            scan_fixture_dir(repository, &fixture_path, findings)?;
        } else if file_type.is_file() && fixture_path.extension() == Some("md") {
            scan_markdown_fixture(repository, &fixture_path, findings)?;
        }
    }
    Ok(())
}

fn scan_markdown_fixture(
    repository: &Dir,
    fixture_path: &Utf8Path,
    findings: &mut Vec<String>,
) -> TestResult {
    let contents = repository.read_to_string(fixture_path)?;
    let lines = contents.lines().collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        if has_noncanonical_fence(line) {
            report(fixture_path, index, "non-canonical code fence", findings);
        }
        if has_repeated_or_noncontiguous_ordered_marker(&lines, index) {
            report(
                fixture_path,
                index,
                "repeated or non-contiguous ordered marker",
                findings,
            );
        }
        if has_overindented_nested_marker(&lines, index) {
            report(
                fixture_path,
                index,
                "over-indented nested list marker",
                findings,
            );
        }
        if starts_loose_list_pair(&lines, index) {
            report(fixture_path, index, "top-level loose list pair", findings);
        }
    }
    Ok(())
}

fn report(path: &Utf8Path, index: usize, label: &str, findings: &mut Vec<String>) {
    findings.push(format!("{path}:{}: {label}", index + 1));
}

fn has_noncanonical_fence(line: &str) -> bool {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    if indent > 3 {
        return false;
    }
    trimmed.starts_with("~~~") || trimmed.starts_with("````")
}

fn has_repeated_or_noncontiguous_ordered_marker(lines: &[&str], index: usize) -> bool {
    let Some(line) = lines.get(index) else {
        return false;
    };
    let Some((indent, ordinal)) = ordered_marker(line) else {
        return false;
    };
    let Some(next) = lines
        .iter()
        .skip(index + 1)
        .find_map(|candidate| ordered_marker_at_indent(candidate, indent))
    else {
        return false;
    };
    next != ordinal + 1
}

fn ordered_marker_at_indent(line: &str, indent: usize) -> Option<u32> {
    let (marker_indent, ordinal) = ordered_marker(line)?;
    (marker_indent == indent).then_some(ordinal)
}

fn ordered_marker(line: &str) -> Option<(usize, u32)> {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    let (digits, rest) = trimmed.split_once('.')?;
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    rest.starts_with(' ')
        .then(|| digits.parse().ok().map(|ordinal| (indent, ordinal)))
        .flatten()
}

fn has_overindented_nested_marker(lines: &[&str], index: usize) -> bool {
    let Some(line) = lines.get(index) else {
        return false;
    };
    let Some(parent_indent) = unordered_marker(line) else {
        return false;
    };
    lines
        .iter()
        .skip(index + 1)
        .find_map(|candidate| unordered_marker(candidate))
        .is_some_and(|child_indent| child_indent > parent_indent + 2)
}

fn unordered_marker(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    trimmed
        .starts_with("- ")
        .then_some(indent)
        .filter(|_| !trimmed.starts_with("- ["))
}

fn starts_loose_list_pair(lines: &[&str], index: usize) -> bool {
    let Some(line) = lines.get(index) else {
        return false;
    };
    let is_top_level_unordered = line.starts_with("- ") && !line.starts_with("- [");
    let is_top_level_ordered = ordered_marker(line).is_some_and(|(indent, _)| indent == 0);
    if !(is_top_level_unordered || is_top_level_ordered) {
        return false;
    }
    lines
        .get(index + 1)
        .is_some_and(|next_line| next_line.is_empty())
        && lines.get(index + 2).is_some_and(|next_line| {
            next_line.starts_with("- ") || ordered_marker(next_line).is_some()
        })
}

fn assert_noop_replace_preserves_target(workspace: &GoldenWorkspace, target: &str) -> TestResult {
    assert_noop_replace_output(workspace, target, target)
}

fn assert_noop_replace_output(
    workspace: &GoldenWorkspace,
    target: &str,
    expected: &str,
) -> TestResult {
    workspace.write_target(target)?;
    workspace.write_fragment("- [ ] 1.1.1. Existing task.\n")?;

    let outcome = run_from_args([
        "mapsplice",
        "replace",
        workspace.target.as_str(),
        "1.1.1",
        workspace.fragment.as_str(),
    ])?;
    let actual = outcome.stdout.unwrap_or_default();
    if actual == expected {
        Ok(())
    } else {
        Err(
            format!("no-op replace output differed\nexpected:\n{expected}\nactual:\n{actual}")
                .into(),
        )
    }
}

const fn gate_clean_loose_list_target() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. Phase one\n\n",
        "- first untouched item\n\n",
        "- second untouched item\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Existing task.\n",
    )
}

const fn gate_clean_indented_code_fence_target() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. Phase one\n\n",
        " ```rust\n",
        " let answer = 42;\n",
        " ```\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Existing task.\n",
    )
}

const fn repeated_ordered_markers_target() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. Phase one\n\n",
        "1. first item\n",
        "1. second item\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Existing task.\n",
    )
}

const fn repeated_ordered_markers_expected() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. Phase one\n\n",
        "1. first item\n",
        "2. second item\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Existing task.\n",
    )
}

const fn overindented_nested_list_target() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. Phase one\n\n",
        "- parent item\n",
        "    - child item\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Existing task.\n",
    )
}

const fn overindented_nested_list_expected() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. Phase one\n\n",
        "- parent item\n\n",
        "  - child item\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Existing task.\n",
    )
}

const fn tilde_fence_target() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. Phase one\n\n",
        "~~~rust\n",
        "let answer = 42;\n",
        "~~~\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Existing task.\n",
    )
}

const fn oversized_fence_target() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. Phase one\n\n",
        "````rust\n",
        "let answer = 42;\n",
        "````\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Existing task.\n",
    )
}

const fn canonical_fence_expected() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. Phase one\n\n",
        "```rust\n",
        "let answer = 42;\n",
        "```\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Existing task.\n",
    )
}
