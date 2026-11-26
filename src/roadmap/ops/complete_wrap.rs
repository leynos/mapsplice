//! Wrapping of inserted done suffixes at the repository's column limit.
//!
//! Repository Markdown is wrapped at 80 columns, and `mdtablefix --wrap`
//! reflows an overflowing line using a hanging indent equal to the paragraph
//! content column: six spaces beneath `- [ ] `, eight beneath `  - [ ] `. It
//! leaves every other line alone. A suffix inserted by `mapsplice complete`
//! therefore refills only the item's final title line, greedily, at the same
//! 80-column limit and with the same hanging indent and document line ending.
//! Formatter-clean input then stays formatter-clean without any other line
//! being rewritten.
//!
//! The layout helpers are deliberately total over a validated span: they
//! return `None` rather than construct a diagnostic when a break cannot be
//! made safe, leaving the caller to report fail-closed completion errors.

/// Column limit for Markdown paragraphs and bullets in this repository.
pub(super) const WRAP_COLUMNS: usize = 80;

/// Layout inputs for wrapping an inserted suffix.
pub(super) struct SuffixWrap<'source> {
    /// Display width of the title line before the insertion point.
    pub(super) width: usize,
    /// Whether a space must separate the title from the first suffix word.
    pub(super) leading_space: bool,
    /// Title punctuation that stays in the source after the inserted words.
    ///
    /// It contributes to the last line's width but is not re-emitted, because
    /// the original bytes already follow the insertion point.
    pub(super) terminal: &'source str,
    /// Indentation for continuation lines.
    pub(super) hanging: String,
    /// Line ending used by the document.
    pub(super) line_ending: &'source str,
}

impl SuffixWrap<'_> {
    /// Display width of the title punctuation that follows the suffix.
    pub(super) fn terminal_width(&self) -> usize {
        self.terminal.trim_end_matches([' ', '\t']).chars().count()
    }
}

/// Append the suffix words greedily, wrapping at [`WRAP_COLUMNS`].
///
/// Returns `None` when a generated break cannot be made safe, so the caller
/// fails closed rather than changing the rendered title.
pub(super) fn wrap_suffix(suffix: &str, wrap: &SuffixWrap<'_>) -> Option<String> {
    let terminal_width = wrap.terminal_width();
    let hanging_width = wrap.hanging.chars().count();
    let words = suffix.split_whitespace().collect::<Vec<_>>();
    let mut rendered = String::new();
    let mut width = wrap.width;
    for (index, word) in words.iter().enumerate() {
        let is_last = index + 1 == words.len();
        let tail_width = if is_last { terminal_width } else { 0 };
        let word_width = word.chars().count();
        let separator = if index == 0 {
            usize::from(wrap.leading_space)
        } else {
            1
        };
        if width + separator + word_width + tail_width <= WRAP_COLUMNS {
            push_word(&mut rendered, word, separator);
            width += separator + word_width + tail_width;
        } else {
            ensure_word_fits_continuation(hanging_width, word_width, tail_width)?;
            start_continuation_line(&mut rendered, width, wrap, word)?;
            width = hanging_width + word_width + tail_width;
        }
    }
    Some(rendered)
}

/// Require one unbreakable word to fit a fresh continuation line.
///
/// The wrapping loop only reaches here when the word cannot stay on the
/// current line, and a word has no internal break point. A word too wide for
/// the hanging indent, the terminal punctuation, and the column limit
/// together would therefore leave the completed title over the limit, so the
/// caller fails closed instead.
fn ensure_word_fits_continuation(
    hanging_width: usize,
    word_width: usize,
    tail_width: usize,
) -> Option<()> {
    (hanging_width + word_width + tail_width <= WRAP_COLUMNS).then_some(())
}

/// Append a word to the current line, preceded by `separator` spaces.
fn push_word(rendered: &mut String, word: &str, separator: usize) {
    if separator == 1 {
        rendered.push(' ');
    }
    rendered.push_str(word);
}

/// Start a continuation line at the hanging indent with `word` on it.
///
/// A line that would end with an unpaired backslash gets one more backslash so
/// the literal character survives the break. Returns `None` when the line is
/// already at the column limit and cannot take that character.
fn start_continuation_line(
    rendered: &mut String,
    width: usize,
    wrap: &SuffixWrap<'_>,
    word: &str,
) -> Option<()> {
    if !rendered.is_empty() && ends_with_unpaired_backslash(rendered) {
        if width >= WRAP_COLUMNS {
            return None;
        }
        rendered.push('\\');
    }
    rendered.push_str(wrap.line_ending);
    rendered.push_str(&wrap.hanging);
    rendered.push_str(word);
    Some(())
}

/// Return the display width of the last line in `rendered`.
///
/// The first line follows the title text already on it, so its width adds
/// `first_line_width`; continuation lines carry their own hanging indent.
pub(super) fn last_line_width(rendered: &str, first_line_width: usize) -> usize {
    rendered.rfind(['\n', '\r']).map_or_else(
        || first_line_width + rendered.chars().count(),
        |index| {
            rendered
                .get(index + 1..)
                .map_or(0, |line| line.chars().count())
        },
    )
}

/// Report whether `text` ends with a backslash that is not itself escaped.
pub(super) fn ends_with_unpaired_backslash(text: &str) -> bool {
    let backslashes = text
        .chars()
        .rev()
        .take_while(|character| *character == '\\')
        .count();
    !backslashes.is_multiple_of(2)
}
