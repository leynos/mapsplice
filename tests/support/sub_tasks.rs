//! Shared fixtures for nested roadmap sub-task tests.

pub const TARGET_WITH_SUB_TASKS: &str = concat!(
    "# Example\n\n",
    "## 1. Phase one\n\n",
    "### 1.1. Step one\n\n",
    "- [ ] 1.1.1. Parent task. Requires 1.1.1.1.\n",
    "    - [ ] 1.1.1.1. First sub-task. Requires 1.1.1.\n",
    "    - [x] 1.1.1.2. Second sub-task.\n",
    "- [ ] 1.1.2. Sibling task. Requires 1.1.1.2.\n",
);
