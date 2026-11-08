//! Shared workspace fixture for roadmap integration tests.

#[path = "workspace.rs"]
mod shared_workspace;

use rstest::fixture;
pub use shared_workspace::{TestResult, Workspace, create_workspace};

#[fixture]
pub fn workspace() -> TestResult<Workspace> {
    let workspace = create_workspace()?;
    Ok(workspace)
}
