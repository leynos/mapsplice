//! GitHub-flavoured Markdown table rendering.

use markdown::mdast::{AlignKind, Node, Table};

use super::{indent_block, render_inline};
use crate::error::{MapspliceError, Result};

/// Render a GitHub-flavoured Markdown table.
pub(super) fn render_table(table: &Table, indent: usize) -> Result<String> {
    let rows = table
        .children
        .iter()
        .map(render_table_row)
        .collect::<Result<Vec<_>>>()?;
    let header = rows.first().ok_or_else(|| MapspliceError::InvalidRoadmap {
        message: "tables must contain at least one row".to_owned(),
    })?;
    let mut lines = vec![
        render_table_line(header),
        render_table_line(&render_table_delimiter(&table.align, header.len())),
    ];
    lines.extend(rows.iter().skip(1).map(|row| render_table_line(row)));
    Ok(indent_block(&lines.join("\n"), indent))
}

/// Render one mdast table row into cell text.
fn render_table_row(node: &Node) -> Result<Vec<String>> {
    let Node::TableRow(row) = node else {
        return Err(MapspliceError::InvalidRoadmap {
            message: "tables must contain only table rows".to_owned(),
        });
    };
    row.children
        .iter()
        .map(render_table_cell)
        .collect::<Result<Vec<_>>>()
}

/// Render one mdast table cell.
fn render_table_cell(node: &Node) -> Result<String> {
    let Node::TableCell(cell) = node else {
        return Err(MapspliceError::InvalidRoadmap {
            message: "table rows must contain only table cells".to_owned(),
        });
    };
    render_inline(&cell.children)
}

/// Render table delimiters for the declared alignments.
fn render_table_delimiter(align: &[AlignKind], width: usize) -> Vec<String> {
    (0..width)
        .map(
            |index| match align.get(index).copied().unwrap_or(AlignKind::None) {
                AlignKind::Left => ":---".to_owned(),
                AlignKind::Right => "---:".to_owned(),
                AlignKind::Center => ":---:".to_owned(),
                AlignKind::None => "---".to_owned(),
            },
        )
        .collect()
}

/// Render a full table row line.
fn render_table_line(cells: &[String]) -> String { format!("| {} |", cells.join(" | ")) }
