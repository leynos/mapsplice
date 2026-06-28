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
    insta::assert_snapshot!(stable_help_lines(&stdout), @r"
Usage: mapsplice [OPTIONS] <COMMAND>
Commands:
  append   
  insert   
  delete   
  replace  
  help     Print this message or the help of the given subcommand(s)
Global options:
  -i, --in-place  Rewrite the target file instead of printing to stdout
");
}

fn stable_help_lines(stdout: &str) -> String {
    stdout
        .trim()
        .lines()
        .filter(|line| {
            line.starts_with("Usage:")
                || line.starts_with("Commands:")
                || line.starts_with("Global options:")
                || line.starts_with("  append")
                || line.starts_with("  insert")
                || line.starts_with("  delete")
                || line.starts_with("  replace")
                || line.starts_with("  help")
                || line.starts_with("  -i, --in-place")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
