//! Markdown fixtures used by formatter-boundary integration tests.

/// Return a formatter-stable loose-list roadmap fixture.
pub(super) const fn gate_clean_loose_list_target() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. Phase one\n\n",
        "- first untouched item\n\n",
        "- second untouched item\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Existing task.\n",
    )
}

/// Return a formatter-stable indented-fence roadmap fixture.
pub(super) const fn gate_clean_indented_code_fence_target() -> &'static str {
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

/// Return a fixture with formatter-unstable repeated ordered markers.
pub(super) const fn repeated_ordered_markers_target() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. Phase one\n\n",
        "1. first item\n",
        "1. second item\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Existing task.\n",
    )
}

/// Return the canonical output for repeated ordered markers.
pub(super) const fn repeated_ordered_markers_expected() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. Phase one\n\n",
        "1. first item\n",
        "2. second item\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Existing task.\n",
    )
}

/// Return a fixture with a formatter-unstable nested list indentation.
pub(super) const fn overindented_nested_list_target() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. Phase one\n\n",
        "- parent item\n",
        "    - child item\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Existing task.\n",
    )
}

/// Return the canonical output for the nested list fixture.
pub(super) const fn overindented_nested_list_expected() -> &'static str {
    concat!(
        "# Roadmap\n\n",
        "## 1. Phase one\n\n",
        "- parent item\n\n",
        "  - child item\n\n",
        "### 1.1. Step one\n\n",
        "- [ ] 1.1.1. Existing task.\n",
    )
}

/// Return a fixture using a non-canonical tilde code fence.
pub(super) const fn tilde_fence_target() -> &'static str {
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

/// Return a fixture using an oversized backtick code fence.
pub(super) const fn oversized_fence_target() -> &'static str {
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

/// Return the canonical output shared by non-canonical fence fixtures.
pub(super) const fn canonical_fence_expected() -> &'static str {
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
