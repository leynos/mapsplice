//! Property-test fixture adapters.

#[path = "workspace.rs"]
mod workspace_support;

pub use workspace_support::create_workspace;

pub const PHASE_FRAGMENT: &str = concat!(
    "## 9. Inserted phase\n\n",
    "### 9.1. Added step\n\n",
    "- [ ] 9.1.1. Added task. Requires 9.1.1.\n",
);
