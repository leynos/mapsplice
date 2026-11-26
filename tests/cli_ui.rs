//! Focused snapshots for stable CLI help text.

use std::process::Command;

#[test]
fn top_level_help_lists_supported_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_mapsplice"))
        .arg("--help")
        .output()
        .expect("help command should run");
    let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");

    assert!(output.status.success(), "help command should succeed");
    assert_eq!(
        stdout.trim_start().lines().next(),
        Some("Usage: mapsplice [OPTIONS] <COMMAND>"),
        "top-level help should start with usage, without an inferred description"
    );
    insta::assert_snapshot!(stable_help_lines(&stdout), @r"
Usage: mapsplice [OPTIONS] <COMMAND>
Commands:
  append
  insert
  delete
  replace
  complete
  export
  help      Print this message or the help of the given subcommand(s)
Global options:
  -i, --in-place  Rewrite the target file instead of printing to stdout
");
}

/// Help-line prefixes that stay stable when clap's wrapping or wording changes.
const STABLE_HELP_PREFIXES: [&str; 11] = [
    "Usage:",
    "Commands:",
    "Global options:",
    "  append",
    "  insert",
    "  delete",
    "  replace",
    "  complete",
    "  export",
    "  help",
    "  -i, --in-place",
];

fn stable_help_lines(stdout: &str) -> String {
    stdout
        .trim()
        .lines()
        .filter(|line| {
            STABLE_HELP_PREFIXES
                .iter()
                .any(|prefix| line.starts_with(prefix))
        })
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
}
