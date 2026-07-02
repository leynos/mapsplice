//! Deterministic renderer for the supported roadmap subset.

#[cfg(test)]
#[path = "render_tests.rs"]
mod render_tests;
#[path = "render_table.rs"]
mod table;
#[path = "render_text.rs"]
mod text;

use markdown::mdast::{Code, Heading, Link, List, ListItem, Node};
use table::render_table;
use text::{escape_markdown, indent_block, render_code_block};

use super::{
    RoadmapDocument,
    model::{MarkdownNodes, SubTaskEntry, TaskChild, TaskEntry},
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
    let rendered = blocks.join("\n\n");
    Ok(if rendered.is_empty() {
        rendered
    } else {
        format!("{rendered}\n")
    })
}

/// Render a list of roadmap tasks as Markdown checklist lines.
fn render_tasks(tasks: &[&TaskEntry]) -> Result<String> {
    tasks
        .iter()
        .map(|task| render_task(task))
        .collect::<Result<Vec<_>>>()
        .map(|lines| lines.join("\n").trim_end_matches('\n').to_owned())
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
        render_item_summary(&render_inline(task.summary.nodes())?, 4)
    )];
    for child in &task.children {
        match child {
            TaskChild::Body(body) => parts.extend(render_nested_body(body, 4)?),
            TaskChild::SubTask(identity) => {
                if let Some(sub_task) = task
                    .sub_tasks
                    .iter()
                    .find(|sub_task| sub_task.identity == *identity)
                {
                    parts.push(render_sub_task(sub_task, 2)?);
                }
            }
        }
    }
    Ok(parts.join("\n"))
}

fn render_sub_task(sub_task: &SubTaskEntry, indent: usize) -> Result<String> {
    let checkbox = match sub_task.checked {
        Some(true) => "[x] ",
        Some(false) => "[ ] ",
        None => "",
    };
    let prefix = " ".repeat(indent);
    let mut parts = vec![format!(
        "{prefix}- {checkbox}{}. {}",
        sub_task.number,
        render_item_summary(&render_inline(sub_task.summary.nodes())?, indent + 2)
    )];
    let body_blocks = render_nested_body(&sub_task.body, indent + 4)?;
    if !body_blocks.is_empty() {
        parts.push(String::new());
        parts.extend(body_blocks);
    }
    Ok(parts.join("\n"))
}

fn render_nested_body(markdown: &MarkdownNodes, indent: usize) -> Result<Vec<String>> {
    let rendered_blocks = render_markdown_nodes(markdown, indent)?;
    let paragraph_count = markdown
        .nodes()
        .iter()
        .filter(|node| matches!(node, Node::Paragraph(_)))
        .count();
    let mut nested_blocks = Vec::new();
    for (node, rendered_block) in markdown.nodes().iter().zip(rendered_blocks) {
        if matches!(node, Node::Code(_) | Node::List(_) | Node::Table(_)) {
            push_blank_separator(&mut nested_blocks);
            nested_blocks.push(rendered_block);
            if !nested_blocks
                .last()
                .is_some_and(|block| block.ends_with('\n'))
            {
                push_blank_separator(&mut nested_blocks);
            }
        } else if matches!(node, Node::Paragraph(_)) && paragraph_count > 1 {
            push_blank_separator(&mut nested_blocks);
            nested_blocks.push(rendered_block);
            push_blank_separator(&mut nested_blocks);
        } else {
            nested_blocks.push(rendered_block);
        }
    }
    Ok(nested_blocks)
}

fn push_blank_separator(blocks: &mut Vec<String>) {
    if !blocks.last().is_some_and(String::is_empty) {
        blocks.push(String::new());
    }
}

fn render_item_summary(summary: &str, continuation_indent: usize) -> String {
    let mut lines = summary.lines();
    let Some(first) = lines.next() else {
        return String::new();
    };
    let mut rendered = vec![first.to_owned()];
    rendered.extend(lines.map(|line| indent_block(line, continuation_indent)));
    rendered.join("\n")
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
