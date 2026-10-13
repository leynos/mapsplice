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
//! What this deliberately does **not** support: block scalars (`|` and `>`),
//! multi-line flow collections, anchors and aliases, and tags. Given an
//! unsupported construct [`parse`] returns an error rather than something
//! wrong, so a future edit to `verus.yml` that this reader cannot read fails
//! loudly at the point of the edit instead of passing vacuously.
//!
//! Quoted keys _are_ supported, and are unquoted before lookup. The workflow
//! under test writes `'on':` because `on` is a YAML 1.1 boolean, so rejecting
//! the quoting would make the reader unable to read the very document it
//! exists for. Rejecting it was the earlier behaviour this file documented,
//! and it was wrong: [`Line::key`] returned the text with its quotes attached,
//! so a caller looking up `on` would have silently missed rather than failed.
//!
//! Flow sequences — `types: [opened, synchronize]` — are supported through
//! [`Line::sequence`], because the workflow under test uses one and a caller
//! that had to fall back to substring matching on it would defeat the point of
//! parsing the document at all.

/// One parsed YAML line: a mapping key, and the value or sequence item under it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Line {
    /// The key text for a mapping entry, or the item text for a sequence entry.
    text: String,
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

    /// Return the key of a `key: value` entry, unquoted.
    ///
    /// A single-quoted key — `'on':`, which the workflow under test uses
    /// because `on` is a YAML 1.1 boolean — yields the bare name. Returning
    /// the quotes would make a caller looking up `on` miss silently, which is
    /// exactly the failure a parser exists to prevent.
    #[must_use]
    pub fn key(&self) -> Option<&str> {
        let (key, _) = self.entry().split_once(':')?;
        Some(unquote(key))
    }

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
/// # Errors
///
/// Returns an error naming the offending line when the document uses a
/// construct this reader does not support: tab indentation, an indentation
/// column that is not a whole number of two-space levels, a block scalar
/// header, or a line that is neither a mapping entry nor a sequence item.
/// Reporting is deliberate — a reader that misread such a line instead would
/// let the workflow contract tests pass on a document they had not actually
/// parsed.
pub fn parse(yaml: &str) -> Result<Vec<Line>, String> {
    let mut lines = Vec::new();
    for raw in yaml.lines() {
        if raw.trim().is_empty() || raw.trim_start().starts_with('#') {
            continue;
        }
        let content = raw.trim_start();
        let indentation = raw
            .get(..raw.len().saturating_sub(content.len()))
            .unwrap_or_default();
        if indentation != " ".repeat(indentation.len()) {
            return Err(format!(
                "workflow_scan cannot read non-space indentation: {raw:?}"
            ));
        }
        if !indentation.len().is_multiple_of(2) {
            return Err(format!(
                "workflow_scan cannot read an indentation column that is not a whole number of \
                 two-space levels: {raw:?}"
            ));
        }
        // A block scalar header is a `key: |` or `key: >` entry, optionally
        // with a chomping or indentation indicator (`|-`, `>+`). Its value
        // lines are continuation text, which this reader would misread as
        // structure, so the header is rejected wherever it appears.
        if let Some(value) = content.split_once(':').map(|(_, value)| value.trim())
            && matches!(value.chars().next(), Some('|' | '>'))
        {
            return Err(format!(
                "workflow_scan cannot read a block scalar; its continuation lines would be \
                 misread as structure: {raw:?}"
            ));
        }
        let is_item = content.starts_with("- ");
        if !is_item && !content.contains(':') {
            return Err(format!(
                "workflow_scan found a line that is neither a mapping entry nor a sequence item: \
                 {raw:?}"
            ));
        }
        lines.push(Line {
            text: content.to_owned(),
        });
    }
    Ok(lines)
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

/// Strip a matching pair of surrounding quotes.
///
/// Both `'single'` and `"double"` forms are removed. Text that is not wrapped
/// in a matching pair is returned unchanged, so this is safe to apply to every
/// key rather than only to the ones that look quoted.
#[must_use]
fn unquote(text: &str) -> &str {
    for quote in ['\'', '"'] {
        if let Some(inner) = text
            .strip_prefix(quote)
            .and_then(|rest| rest.strip_suffix(quote))
        {
            return inner;
        }
    }
    text
}

#[cfg(test)]
mod tests {
    //! Reader-edge coverage.

    use rstest::rstest;

    use super::{parse, unquote};

    #[rstest]
    #[case::single("'on'", "on")]
    #[case::double("\"on\"", "on")]
    #[case::bare("on", "on")]
    #[case::single_quote_only("'on", "'on")]
    #[case::empty("", "")]
    fn unquote_strips_only_a_matching_pair(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(unquote(text), expected);
    }

    #[test]
    fn a_quoted_key_is_looked_up_without_its_quotes() {
        let lines = parse("'on':\n  workflow_dispatch:\n").unwrap_or_default();
        assert!(
            lines.iter().any(|line| line.key() == Some("on")),
            "a quoted key must be reachable by its bare name; parsed {lines:?}"
        );
    }

    #[rstest]
    #[case::tab("\ton: x\n")]
    #[case::two_and_a_half_levels("   on: x\n")]
    #[case::block_scalar_literal("run: |\n")]
    #[case::block_scalar_folded("run: >\n")]
    #[case::block_scalar_stripped("run: |-\n")]
    #[case::block_scalar_indented("run: |2\n")]
    fn parse_rejects_unsupported_constructs(#[case] yaml: &str) {
        assert!(
            parse(yaml).is_err(),
            "unsupported construct must fail loudly rather than be misread: {yaml:?}"
        );
    }

    #[test]
    fn even_indentation_is_accepted() {
        assert!(parse("on:\n  x:\n    y:\n").is_ok());
    }
}
