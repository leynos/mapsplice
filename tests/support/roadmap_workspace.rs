//! Shared workspace fixture for roadmap integration tests.

#[path = "workspace.rs"]
mod shared_workspace;

use rstest::fixture;
pub use shared_workspace::{TestResult, Workspace};

#[fixture]
pub fn workspace() -> TestResult<Workspace> {
    let workspace = shared_workspace::create_workspace()?;
    Ok(workspace)
}
