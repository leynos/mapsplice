//! Process contracts for stable, description-free subcommand help.

use std::process::Command;

const HELP_CASES: &[(&str, &str)] = &[
    (
        "append",
        concat!(
            "Usage: mapsplice append [OPTIONS] <TARGET> <FRAGMENT>\n\n",
            "Arguments:\n",
            "  <TARGET>    \n",
            "  <FRAGMENT>  \n\n",
            "Options:\n",
            "  -h, --help  Print help\n\n",
            "Global options:\n",
            "  -i, --in-place  Rewrite the target file instead of printing to stdout\n",
        ),
    ),
    (
        "insert",
        concat!(
            "Usage: mapsplice insert [OPTIONS] <TARGET> <ANCHOR> <FRAGMENT>\n\n",
            "Arguments:\n",
            "  <TARGET>    \n",
            "  <ANCHOR>    \n",
            "  <FRAGMENT>  \n\n",
            "Options:\n",
            "      --after  \n",
            "  -h, --help   Print help\n\n",
            "Global options:\n",
            "  -i, --in-place  Rewrite the target file instead of printing to stdout\n",
        ),
    ),
    (
        "delete",
        concat!(
            "Usage: mapsplice delete [OPTIONS] <TARGET> <ANCHOR>\n\n",
            "Arguments:\n",
            "  <TARGET>  \n",
            "  <ANCHOR>  \n\n",
            "Options:\n",
            "  -h, --help  Print help\n\n",
            "Global options:\n",
            "  -i, --in-place  Rewrite the target file instead of printing to stdout\n",
        ),
    ),
    (
        "replace",
        concat!(
            "Usage: mapsplice replace [OPTIONS] <TARGET> <ANCHOR> <FRAGMENT>\n\n",
            "Arguments:\n",
            "  <TARGET>    \n",
            "  <ANCHOR>    \n",
            "  <FRAGMENT>  \n\n",
            "Options:\n",
            "  -h, --help  Print help\n\n",
            "Global options:\n",
            "  -i, --in-place  Rewrite the target file instead of printing to stdout\n",
        ),
    ),
];

#[test]
fn subcommand_help_preserves_raw_structure_without_inferred_descriptions() {
    for (subcommand, expected) in HELP_CASES {
        let output = Command::new(env!("CARGO_BIN_EXE_mapsplice"))
            .args([*subcommand, "--help"])
            .env("NO_COLOR", "1")
            .output()
            .expect("subcommand help should run");

        assert!(
            output.status.success(),
            "{subcommand} help failed: {:?}",
            output.status
        );
        assert!(
            output.stderr.is_empty(),
            "{subcommand} help wrote stderr: {:?}",
            output.stderr
        );
        assert_eq!(
            output.stdout,
            expected.as_bytes(),
            "{subcommand} raw help structure changed"
        );
    }
}
