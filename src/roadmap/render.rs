//! Deterministic renderer for the supported roadmap subset.

#[path = "render_table.rs"]
mod table;

use markdown::mdast::{Code, Heading, Link, List, ListItem, Node};
use table::render_table;

use super::{
    RoadmapDocument,
    model::{MarkdownNodes, SubTaskEntry, TaskEntry},
};
use crate::error::{MapspliceError, Result};

/// Render a parsed roadmap back to Markdown.
#[tracing::instrument(skip_all, fields(phases = roadmap.phases.len()))]
pub fn render_roadmap(roadmap: &RoadmapDocument) -> Result<String> {
    let mut blocks = Vec::new();
    blocks.extend(render_markdown_nodes(&roadmap.preamble, 0)?);
    for phase in &roadmap.phases {
        blocks.push(format!(
            "## {}. {}",
            phase.number,
            render_inline(phase.title.nodes())?
        ));
        blocks.extend(render_markdown_nodes(&phase.body, 0)?);
        for step in &phase.steps {
            blocks.push(format!(
                "### {}. {}",
                step.number,
                render_inline(step.title.nodes())?
            ));
            blocks.extend(render_markdown_nodes(&step.body, 0)?);
            if !step.tasks.is_empty() {
                blocks.push(render_tasks(
                    step.tasks.iter().collect::<Vec<_>>().as_slice(),
                )?);
            }
            blocks.extend(render_markdown_nodes(&step.trailing, 0)?);
        }
        blocks.extend(render_markdown_nodes(&phase.trailing, 0)?);
    }
    Ok(blocks.join("\n\n"))
}

/// Render a list of roadmap tasks as Markdown checklist lines.
fn render_tasks(tasks: &[&TaskEntry]) -> Result<String> {
    tasks
        .iter()
        .map(|task| render_task(task))
        .collect::<Result<Vec<_>>>()
        .map(|lines| lines.join("\n"))
}

/// Render one roadmap task and any nested body blocks.
fn render_task(task: &TaskEntry) -> Result<String> {
    let checkbox = match task.checked {
        Some(true) => "[x] ",
        Some(false) => "[ ] ",
        None => "",
    };
    let mut parts = vec![format!(
        "- {checkbox}{}. {}",
        task.number,
        render_inline(task.summary.nodes())?
    )];
    for block in render_markdown_nodes(&task.body, 4)? {
        parts.push(block);
    }
    if !task.sub_tasks.is_empty() {
        parts.push(indent_block(&render_sub_tasks(&task.sub_tasks)?, 4));
    }
    Ok(parts.join("\n\n"))
}

fn render_sub_tasks(sub_tasks: &[SubTaskEntry]) -> Result<String> {
    sub_tasks
        .iter()
        .map(render_sub_task)
        .collect::<Result<Vec<_>>>()
        .map(|lines| lines.join("\n"))
}

fn render_sub_task(sub_task: &SubTaskEntry) -> Result<String> {
    let checkbox = match sub_task.checked {
        Some(true) => "[x] ",
        Some(false) => "[ ] ",
        None => "",
    };
    let mut parts = vec![format!(
        "- {checkbox}{}. {}",
        sub_task.number,
        render_inline(sub_task.summary.nodes())?
    )];
    for block in render_markdown_nodes(&sub_task.body, 4)? {
        parts.push(block);
    }
    Ok(parts.join("\n\n"))
}

fn render_markdown_nodes(markdown: &MarkdownNodes, indent: usize) -> Result<Vec<String>> {
    markdown
        .nodes()
        .iter()
        .zip(markdown.original_blocks())
        .map(|(node, original)| {
            original
                .as_ref()
                .map_or_else(|| render_block(node, indent), |block| Ok(block.clone()))
        })
        .collect()
}

fn render_blocks(nodes: &[Node], indent: usize) -> Result<Vec<String>> {
    nodes
        .iter()
        .map(|node| render_block(node, indent))
        .collect()
}

fn render_block(node: &Node, indent: usize) -> Result<String> {
    match node {
        Node::Paragraph(paragraph) => {
            Ok(indent_block(&render_inline(&paragraph.children)?, indent))
        }
        Node::Heading(Heading {
            depth, children, ..
        }) => Ok(indent_block(
            &format!(
                "{} {}",
                "#".repeat((*depth).into()),
                render_inline(children)?
            ),
            indent,
        )),
        Node::List(list) => render_list(list, indent),
        Node::Blockquote(blockquote) => render_blocks(&blockquote.children, 0)
            .map(|parts| indent_block(&render_blockquote_parts(parts), indent)),
        Node::Code(Code {
            value, lang, meta, ..
        }) => Ok(render_code_block(
            value,
            lang.as_deref(),
            meta.as_deref(),
            indent,
        )),
        Node::Html(html) => Ok(indent_block(&html.value, indent)),
        Node::Table(table) => render_table(table, indent),
        Node::ThematicBreak(_) => Ok(indent_block("---", indent)),
        other => Err(MapspliceError::InvalidRoadmap {
            message: format!(
                "unsupported block node `{}` in rendered roadmap",
                node_name(other)
            ),
        }),
    }
}

/// Render a fenced code block using a fence longer than its contents.
fn render_code_block(value: &str, lang: Option<&str>, meta: Option<&str>, indent: usize) -> String {
    let fence = safe_code_fence(value);
    let info = code_fence_info(lang, meta);
    let opener = format!("{fence}{info}");
    indent_block(&format!("{opener}\n{value}\n{fence}"), indent)
}

fn render_blockquote_parts(parts: Vec<String>) -> String {
    parts
        .into_iter()
        .map(|part| {
            part.lines()
                .map(|line| format!("> {line}"))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n>\n")
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

/// Return the longest contiguous backtick run in a string.
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

/// Render an ordered or unordered Markdown list.
fn render_list(list: &List, indent: usize) -> Result<String> {
    let mut rendered = Vec::new();
    for (index, node) in list.children.iter().enumerate() {
        let Node::ListItem(item) = node else {
            return Err(MapspliceError::InvalidRoadmap {
                message: "lists must contain only list items".to_owned(),
            });
        };
        rendered.push(render_list_item(
            item,
            indent,
            list.ordered,
            list.start.unwrap_or(1) + list_offset(index)?,
        )?);
    }
    Ok(rendered.join("\n"))
}

/// Render one list item with the given marker state.
fn render_list_item(item: &ListItem, indent: usize, ordered: bool, ordinal: u32) -> Result<String> {
    let prefix = if ordered {
        format!("{ordinal}. ")
    } else {
        match item.checked {
            Some(true) => "- [x] ".to_owned(),
            Some(false) => "- [ ] ".to_owned(),
            None => "- ".to_owned(),
        }
    };

    let first = item
        .children
        .first()
        .ok_or_else(|| MapspliceError::InvalidRoadmap {
            message: "list items must contain at least one child block".to_owned(),
        })?;
    let mut lines = Vec::new();
    match first {
        Node::Paragraph(paragraph) => {
            lines.push(format!("{prefix}{}", render_inline(&paragraph.children)?));
        }
        other => {
            lines.push(format!("{prefix}{}", render_block(other, 0)?));
        }
    }

    for child in item.children.iter().skip(1) {
        lines.push(indent_block(&render_block(child, 0)?, prefix.len()));
    }

    Ok(indent_block(&lines.join("\n\n"), indent))
}

/// Render inline Markdown nodes into a single string.
fn render_inline(nodes: &[Node]) -> Result<String> {
    nodes
        .iter()
        .map(render_inline_node)
        .collect::<Result<Vec<_>>>()
        .map(|parts| parts.concat())
}

/// Render one supported inline Markdown node.
fn render_inline_node(node: &Node) -> Result<String> {
    match node {
        Node::Text(text) => Ok(escape_markdown(&text.value)),
        Node::Emphasis(emphasis) => Ok(format!("*{}*", render_inline(&emphasis.children)?)),
        Node::Strong(strong) => Ok(format!("**{}**", render_inline(&strong.children)?)),
        Node::Delete(delete) => Ok(format!("~~{}~~", render_inline(&delete.children)?)),
        Node::InlineCode(code) => Ok(format!("`{}`", code.value)),
        Node::Break(_) => Ok("\\\n".to_owned()),
        Node::Link(Link {
            children,
            url,
            title,
            ..
        }) => {
            let suffix = title
                .as_ref()
                .map_or_else(String::new, |link_title| format!(" \"{link_title}\""));
            Ok(format!("[{}]({url}{suffix})", render_inline(children)?))
        }
        other => Err(MapspliceError::InvalidRoadmap {
            message: format!(
                "unsupported inline node `{}` in rendered roadmap",
                node_name(other)
            ),
        }),
    }
}

/// Escape Markdown metacharacters in plain text.
fn escape_markdown(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if is_markdown_metacharacter(character) {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

/// Return whether a character should be escaped in plain text output.
const fn is_markdown_metacharacter(character: char) -> bool {
    matches!(
        character,
        '*' | '_'
            | '`'
            | '['
            | ']'
            | '('
            | ')'
            | '~'
            | '>'
            | '#'
            | '+'
            | '-'
            | '='
            | '|'
            | '{'
            | '}'
    )
}

/// Indent every non-empty line in a rendered block.
fn indent_block(block: &str, spaces: usize) -> String {
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

/// Convert a zero-based list index into a renderable ordinal offset.
fn list_offset(index: usize) -> Result<u32> {
    u32::try_from(index).map_err(|_| MapspliceError::InvalidRoadmap {
        message: "list item count exceeds supported numbering range".to_owned(),
    })
}

/// Return a stable mdast node name for renderer diagnostics.
const fn node_name(node: &Node) -> &'static str {
    match node {
        Node::Root(_) => "root",
        Node::Blockquote(_) => "blockquote",
        Node::FootnoteDefinition(_) => "footnoteDefinition",
        Node::MdxJsxFlowElement(_) => "mdxJsxFlowElement",
        Node::List(_) => "list",
        Node::MdxjsEsm(_) => "mdxjsEsm",
        Node::Toml(_) => "toml",
        Node::Yaml(_) => "yaml",
        Node::Break(_) => "break",
        Node::InlineCode(_) => "inlineCode",
        Node::InlineMath(_) => "inlineMath",
        Node::Delete(_) => "delete",
        Node::Emphasis(_) => "emphasis",
        Node::MdxTextExpression(_) => "mdxTextExpression",
        Node::FootnoteReference(_) => "footnoteReference",
        Node::Html(_) => "html",
        Node::Image(_) => "image",
        Node::ImageReference(_) => "imageReference",
        Node::MdxJsxTextElement(_) => "mdxJsxTextElement",
        Node::Link(_) => "link",
        Node::LinkReference(_) => "linkReference",
        Node::Strong(_) => "strong",
        Node::Text(_) => "text",
        Node::Code(_) => "code",
        Node::Math(_) => "math",
        Node::MdxFlowExpression(_) => "mdxFlowExpression",
        Node::Heading(_) => "heading",
        Node::Table(_) => "table",
        Node::ThematicBreak(_) => "thematicBreak",
        Node::TableRow(_) => "tableRow",
        Node::TableCell(_) => "tableCell",
        Node::ListItem(_) => "listItem",
        Node::Definition(_) => "definition",
        Node::Paragraph(_) => "paragraph",
    }
}
