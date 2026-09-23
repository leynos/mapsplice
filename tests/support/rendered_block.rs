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

/// Return the block belonging to the item whose summary contains `summary`.
///
/// `summary` must appear on exactly one line of `rendered`; the first match
/// wins. Returns `None` when no line matches. Lines are collected rather than
/// sliced by byte offset, so no index can fall inside a multi-byte character.
#[must_use]
pub(crate) fn item_block<'a>(rendered: &'a str, summary: &str) -> Option<&'a str> {
    let match_line = rendered.lines().position(|line| line.contains(summary))?;
    Some(block_starting_at(rendered, match_line))
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
