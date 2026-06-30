//! Text helpers for roadmap rendering.

/// Render a fenced code block using a fence longer than its contents.
pub(super) fn render_code_block(
    value: &str,
    lang: Option<&str>,
    meta: Option<&str>,
    indent: usize,
) -> String {
    let fence = safe_code_fence(value);
    let info = code_fence_info(lang, meta);
    let opener = format!("{fence}{info}");
    indent_block(&format!("{opener}\n{value}\n{fence}"), indent)
}

/// Escape Markdown metacharacters in plain text.
pub(super) fn escape_markdown(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if is_markdown_metacharacter(character) {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

/// Indent every non-empty line in a rendered block.
pub(super) fn indent_block(block: &str, spaces: usize) -> String {
    if spaces == 0 {
        return block.to_owned();
    }
    let padding = " ".repeat(spaces);
    block
        .lines()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{padding}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn code_fence_info(lang: Option<&str>, meta: Option<&str>) -> String {
    match (lang, meta) {
        (Some(language), Some(metadata)) => format!("{language} {metadata}"),
        (Some(language), None) => language.to_owned(),
        (None, Some(metadata)) => metadata.to_owned(),
        (None, None) => String::new(),
    }
}

fn safe_code_fence(value: &str) -> String {
    "`".repeat(longest_backtick_run(value).saturating_add(1).max(3))
}

fn longest_backtick_run(value: &str) -> usize {
    let mut longest = 0;
    let mut current = 0;
    for character in value.chars() {
        if character == '`' {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    longest
}

const fn is_markdown_metacharacter(character: char) -> bool {
    matches!(
        character,
        '*' | '_' | '`' | '[' | ']' | '(' | ')' | '~' | '>' | '#' | '+' | '=' | '|' | '{' | '}'
    )
}
