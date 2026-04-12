//! Behavioural coverage for the `mapsplice` binary.

mod steps;

use rstest_bdd_macros::scenario;
use steps::{CliFixture, cli_state};

#[scenario(
    path = "tests/features/mapsplice.feature",
    name = "Append emits the rewritten roadmap to stdout"
)]
fn append_emits_stdout(cli_state: CliFixture) -> Result<(), Box<dyn std::error::Error>> {
    let _ = cli_state?;
    Ok(())
}

#[scenario(
    path = "tests/features/mapsplice.feature",
    name = "Insert before a phase renumbers later phases and dependencies"
)]
fn insert_before_phase(cli_state: CliFixture) -> Result<(), Box<dyn std::error::Error>> {
    let _ = cli_state?;
    Ok(())
}

#[scenario(
    path = "tests/features/mapsplice.feature",
    name = "Insert after a task renumbers later tasks within the step"
)]
fn insert_after_task(cli_state: CliFixture) -> Result<(), Box<dyn std::error::Error>> {
    let _ = cli_state?;
    Ok(())
}

#[scenario(
    path = "tests/features/mapsplice.feature",
    name = "Delete removes an addressed phase and rewrites downstream identifiers"
)]
fn delete_phase(cli_state: CliFixture) -> Result<(), Box<dyn std::error::Error>> {
    let _ = cli_state?;
    Ok(())
}

#[scenario(
    path = "tests/features/mapsplice.feature",
    name = "Replace swaps a phase with multiple phases from a fragment file"
)]
fn replace_phase(cli_state: CliFixture) -> Result<(), Box<dyn std::error::Error>> {
    let _ = cli_state?;
    Ok(())
}

#[scenario(
    path = "tests/features/mapsplice.feature",
    name = "In-place mode rewrites the target file and emits no roadmap body"
)]
fn in_place_mode(cli_state: CliFixture) -> Result<(), Box<dyn std::error::Error>> {
    let _ = cli_state?;
    Ok(())
}

#[scenario(
    path = "tests/features/mapsplice.feature",
    name = "Level mismatch returns a clear failure"
)]
fn level_mismatch(cli_state: CliFixture) -> Result<(), Box<dyn std::error::Error>> {
    let _ = cli_state?;
    Ok(())
}
