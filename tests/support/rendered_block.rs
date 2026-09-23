//! Locate one rendered item's own block inside a rendered roadmap.
//!
//! A property assertion written as `rendered.contains("Requires 1.1.2.")` is
//! weaker than it looks. The generator emits a fenced code example spelling
//! `Requires 1.1.1.` into **every** task body, so a clause assertion for an
//! anchor that happens to be `1.1.1` is satisfied by that example alone, and a
//! clause misdirected to any other anchor would still pass. Scoping each
//! assertion to the owning item's block is what makes it bite.
//!
//! Block boundaries come from the item summary lines the generator writes, not
//! from the production parser: a block runs from its own summary line up to,
//! but not including, the next summary line of any level. A summary line is a
//! heading or a checkbox list item; a nested `Requires` bullet carries no
//! checkbox, so a clause never ends the block it belongs to.

/// Return whether `line` begins a new roadmap item's summary.
///
/// Headings and checkbox list items start an item. The check deliberately does
/// not match a bare list item, because both nested clause forms render as
/// `- Requires ...` bullets inside the body of the item they belong to.
fn starts_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("## ")
        || trimmed.starts_with("### ")
        || trimmed.starts_with("- [ ] ")
        || trimmed.starts_with("- [x] ")
}

/// Return the block belonging to the item whose summary is headed by `summary`.
///
/// The summary must appear on exactly one line of `rendered` as a whole stem;
/// the first match wins. Returns `None` when no line matches. Lines are
/// collected rather than sliced by byte offset, so no index can fall inside a
/// multi-byte character.
#[must_use]
pub(crate) fn item_block<'a>(rendered: &'a str, summary: &str) -> Option<&'a str> {
    let match_line = rendered
        .lines()
        .position(|line| carries_stem(line, summary))?;
    Some(block_starting_at(rendered, match_line))
}

/// Return whether `line` carries `stem` as a whole summary stem.
///
/// A plain `contains` would accept a stem that is merely a prefix of a longer
/// item's summary, so looking for `Generated task 1` could land on
/// `Generated task 12` and scope an assertion to the wrong block. The stem
/// therefore has to end at a boundary: end of line, or a character that cannot
/// continue a number. The generator appends a clause and incidental prose to
/// the summary line, so the boundary is normally a space.
///
/// This is not currently reachable with the generator's labels — no summary is
/// a prefix of another, and a scan of the generated document order finds no
/// pair where the longer stem is rendered first — but a label change could make
/// it reachable, and a silently mis-scoped assertion is exactly the kind of
/// failure this module exists to prevent.
fn carries_stem(line: &str, stem: &str) -> bool {
    let mut search_from = 0;
    while let Some(offset) = line.get(search_from..).and_then(|rest| rest.find(stem)) {
        let end = search_from + offset + stem.len();
        let boundary = line
            .get(end..)
            .and_then(|rest| rest.chars().next())
            .is_none_or(|next| !next.is_ascii_alphanumeric());
        if boundary {
            return true;
        }
        search_from = end;
    }
    false
}

/// Return the block whose first line is `rendered`'s `start`-th line.
///
/// The block runs from that line to the line before the next item summary. A
/// line's own length is bounded, so the block is built by accumulating line
/// lengths rather than by searching for a terminator; the original text is
/// sliced in one step at the end.
///
/// Every offset here lands on a line boundary and so is genuinely safe, but
/// `str::get` is used rather than index slicing because `clippy::string_slice`
/// is denied, and because `get` degrades to an empty block rather than panicking
/// if the line-boundary invariant is ever broken.
fn block_starting_at(rendered: &str, start: usize) -> &str {
    let begin = line_prefix_len(rendered, start);
    let rest = rendered.get(begin..).unwrap_or_default();
    let mut end = 0;
    for (index, line) in rest.split_inclusive('\n').enumerate() {
        // `index == 0` is the block's own first line, which must not end it even
        // when it is itself an item summary.
        if index > 0 && starts_item(line) {
            break;
        }
        end += line.len();
    }
    rest.get(..end).unwrap_or_default()
}

/// Return the byte length of `text` up to the start of its `count`-th line.
fn line_prefix_len(text: &str, count: usize) -> usize {
    text.split_inclusive('\n').take(count).map(str::len).sum()
}

/// Return `text` with every fenced code region removed.
///
/// Fences are tracked by toggling on each delimiter line, so a fence inside the
/// removed region cannot re-open one.
#[must_use]
pub(crate) fn strip_fenced_regions(text: &str) -> String {
    let mut kept = String::new();
    let mut in_fence = false;
    for line in text.split_inclusive('\n') {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            kept.push_str(line);
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    //! Stem-boundary coverage.
    //!
    //! The property suite reaches `carries_stem` only through `item_block`, and
    //! every stem it passes is well-formed against the current generator labels,
    //! so a regression to plain `contains` would leave those properties green
    //! while silently mis-scoping assertions to a longer item's block. These
    //! cases fail if the boundary check is dropped.

    use rstest::rstest;

    use super::carries_stem;

    #[rstest]
    // The hazard this boundary exists for: a stem must not match a longer
    // summary that merely begins with it.
    #[case::does_not_match_a_longer_number(
        "- [ ] 1.1.1. Generated task 12. Requires 1.1.1.",
        "Generated task 1",
        false
    )]
    #[case::matches_the_exact_stem(
        "- [ ] 1.1.1. Generated task 1. Requires 1.1.1.",
        "Generated task 1",
        true
    )]
    // End of line is a boundary; the generator writes bare summaries for
    // headings.
    #[case::end_of_line_is_a_boundary("## 1. Generated phase 0", "Generated phase 0", true)]
    #[case::does_not_match_at_end_of_line_either(
        "## 1. Generated phase 01",
        "Generated phase 0",
        false
    )]
    #[case::sub_task_stems_are_guarded(
        "- [ ] 1.1.1.1. Generated sub-task 2-0",
        "Generated sub-task 2-0",
        true
    )]
    #[case::sub_task_stems_do_not_match_a_longer_tail(
        "- [ ] 1.1.1.1. Generated sub-task 2-01",
        "Generated sub-task 2-0",
        false
    )]
    // Prose and punctuation around the stem do not defeat the match.
    #[case::punctuation_is_a_boundary(
        "- [ ] 1.1.1. Generated task 1, see §2.1.",
        "Generated task 1",
        true
    )]
    #[case::an_absent_stem_does_not_match(
        "- [ ] 1.1.1. Something else.",
        "Generated task 1",
        false
    )]
    // A non-boundary occurrence must not hide a later valid one, which is what
    // makes the search loop necessary rather than a single `find`.
    #[case::a_later_valid_occurrence_still_matches(
        "Generated task 12 and Generated task 1.",
        "Generated task 1",
        true
    )]
    fn stems_match_only_at_a_boundary(
        #[case] line: &str,
        #[case] stem: &str,
        #[case] expected: bool,
    ) {
        assert_eq!(
            carries_stem(line, stem),
            expected,
            "stem {stem:?} against line {line:?}"
        );
    }
}
