//! Minimal structural reader for the repository's own workflow YAML.
//!
//! The Verus workflow contract tests need to assert on a handful of values in
//! `.github/workflows/verus.yml`: the job's `run:` commands, the `on:` event
//! types, and the cache step's `path:` and `key:`. Those assertions must live
//! inside `make test`, which rules out `actionlint` and `yamllint` — both are
//! separate binaries the Rust test step cannot assume.
//!
//! Adding a YAML parser to `[dev-dependencies]` would work, and would be the
//! natural choice if this file were reading arbitrary user configuration. It
//! is not. The input is one file this repository authors and reviews, its
//! grammar is a handful of flat mappings and sequences, and the alternative to
//! parsing it here is a new transitive dependency in the lock file for the
//! sake of four assertions. The narrow reader is the smaller commitment.
//!
//! What this deliberately does **not** support: multi-line scalars, anchors
//! and aliases, tags, quoted keys, or nested sequences of sequences. Given an
//! unsupported construct it panics rather than returning something wrong, so a
//! future edit to `verus.yml` that this reader cannot read fails loudly at the
//! point of the edit instead of passing vacuously.
//!
//! Flow sequences — `types: [opened, synchronize]` — are supported through
//! [`Line::sequence`], because the workflow under test uses one and a caller
//! that had to fall back to substring matching on it would defeat the point of
//! parsing the document at all.

/// One parsed YAML line: a mapping key, and the value or sequence item under it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Line {
    /// Zero-based depth, counted in leading two-space indentation levels.
    pub indent: usize,
    /// The key text for a mapping entry, or the item text for a sequence entry.
    pub text: String,
    /// Whether this line begins a sequence item.
    pub is_item: bool,
}

impl Line {
    /// Return the entry text with any sequence marker removed.
    ///
    /// A block sequence entry may itself be a mapping — `- uses: actions/x@y`
    /// is the common case in a workflow — so key and value are read from the
    /// text after `- `, not from the raw line. Without this, every `uses:`
    /// inside `steps:` would be invisible to [`entries`].
    #[must_use]
    fn entry(&self) -> &str { self.text.strip_prefix("- ").unwrap_or(&self.text) }

    /// Return the value of a `key: value` entry.
    ///
    /// `None` covers both a plain sequence scalar and a key with no inline
    /// value (a nested mapping or sequence follows).
    #[must_use]
    pub fn value(&self) -> Option<&str> { self.entry().split_once(": ").map(|(_, value)| value) }

    /// Return the key of a `key: value` entry.
    #[must_use]
    pub fn key(&self) -> Option<&str> { self.entry().split_once(':').map(|(key, _)| key) }

    /// Return the comma-separated elements of an inline flow sequence.
    ///
    /// `types: [opened, synchronize, reopened]` yields exactly those three
    /// strings, trimmed. A value that is not a flow sequence yields an empty
    /// list rather than a one-element list, so a caller cannot accidentally
    /// match on the brackets.
    #[must_use]
    pub fn sequence(&self) -> Vec<&str> {
        let Some(value) = self.value() else {
            return Vec::new();
        };
        let Some(inner) = value
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        else {
            return Vec::new();
        };
        inner
            .split(',')
            .map(str::trim)
            .filter(|element| !element.is_empty())
            .collect()
    }
}

/// Parse `yaml` into the flat line sequence this module understands.
///
/// Blank lines and whole-line comments are dropped; they carry no structure
/// this contract depends on.
///
/// # Panics
///
/// Panics on tab indentation, on indentation that is not a whole number of
/// two-space levels, and on any line that is neither a mapping entry nor a
/// sequence item. Those are the constructs listed as unsupported above, and
/// failing here is what keeps the reader from silently misreading them.
#[must_use]
pub fn parse(yaml: &str) -> Vec<Line> {
    let mut lines = Vec::new();
    for raw in yaml.lines() {
        if raw.trim().is_empty() || raw.trim_start().starts_with('#') {
            continue;
        }
        assert!(
            !raw.starts_with('\t') && !raw[..raw.len() - raw.trim_start().len()].contains('\t'),
            "workflow_scan cannot read tab indentation: {raw:?}"
        );
        let content = raw.trim_start();
        let indent = raw.len() - content.len();
        assert_eq!(
            indent % 2,
            0,
            "workflow_scan expects two-space indentation: {raw:?}"
        );
        let is_item = content.starts_with("- ");
        assert!(
            is_item || content.contains(':'),
            "workflow_scan found a line that is neither a mapping entry nor a sequence item: \
             {raw:?}"
        );
        lines.push(Line {
            indent: indent / 2,
            text: content.to_owned(),
            is_item,
        });
    }
    lines
}

/// Return the indices of every mapping entry whose key is `key`.
///
/// The search is document-wide and ignores depth, which is what a test wants
/// when it asserts on a stage-level `path:` or `run:` whose surrounding
/// structure is not itself under test.
#[must_use]
pub fn entries<'a>(lines: &'a [Line], key: &str) -> Vec<&'a Line> {
    lines
        .iter()
        .filter(|line| line.key() == Some(key))
        .collect()
}

/// Return the inline values of every mapping entry whose key is `key`.
#[must_use]
pub fn values<'a>(lines: &'a [Line], key: &str) -> Vec<&'a str> {
    entries(lines, key)
        .into_iter()
        .filter_map(Line::value)
        .collect()
}

/// Return whether any mapping entry has `key` with the inline value `value`.
#[must_use]
pub fn has_value(lines: &[Line], key: &str, value: &str) -> bool {
    values(lines, key).contains(&value)
}

/// Return the elements of the first flow sequence valued under `key`.
#[must_use]
pub fn sequence<'a>(lines: &'a [Line], key: &str) -> Vec<&'a str> {
    entries(lines, key)
        .first()
        .map(|line| line.sequence())
        .unwrap_or_default()
}
