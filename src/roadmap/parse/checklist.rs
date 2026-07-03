//! Shared checklist item head parsing for roadmap structures.

use markdown::mdast::{ListItem, Node, Paragraph};

use crate::error::{MapspliceError, Result};

#[derive(Debug)]
pub(super) struct ChecklistItemHead<'item> {
    pub(super) checked: Option<bool>,
    pub(super) paragraph: &'item Paragraph,
    pub(super) child_body: &'item [Node],
}

#[derive(Clone, Copy)]
pub(super) enum ChecklistKind {
    Task,
    SubTask,
}

impl ChecklistKind {
    const fn required_message(self) -> &'static str {
        match self {
            Self::Task => "roadmap task lists must be unordered checklist items",
            Self::SubTask => "roadmap sub-task lists must be unordered checklist items",
        }
    }

    const fn paragraph_message(self) -> &'static str {
        match self {
            Self::Task => "task list items must start with a paragraph",
            Self::SubTask => "sub-task list items must start with a paragraph",
        }
    }
}

pub(super) fn parse_checklist_item_head(
    item: &ListItem,
    kind: ChecklistKind,
) -> Result<ChecklistItemHead<'_>> {
    if item.checked.is_none() {
        return Err(invalid_roadmap(kind.required_message()));
    }
    let first = item
        .children
        .first()
        .ok_or_else(|| invalid_roadmap(kind.paragraph_message()))?;
    let Node::Paragraph(paragraph) = first else {
        return Err(invalid_roadmap(kind.paragraph_message()));
    };
    Ok(ChecklistItemHead {
        checked: item.checked,
        paragraph,
        child_body: item.children.get(1..).unwrap_or(&[]),
    })
}

fn invalid_roadmap(message: &str) -> MapspliceError {
    MapspliceError::InvalidRoadmap {
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    //! Tests for shared checklist item head parsing.

    use markdown::mdast::{Html, ListItem, Node};
    use rstest::rstest;

    use super::{ChecklistKind, parse_checklist_item_head};
    use crate::MapspliceError;

    #[rstest]
    #[case::task(ChecklistKind::Task, "task list items must start with a paragraph")]
    #[case::sub_task(
        ChecklistKind::SubTask,
        "sub-task list items must start with a paragraph"
    )]
    fn checklist_item_head_rejects_checked_items_without_paragraph(
        #[case] kind: ChecklistKind,
        #[case] expected: &str,
    ) {
        let item = ListItem {
            children: vec![Node::Html(Html {
                value: "<div></div>".to_owned(),
                position: None,
            })],
            position: None,
            spread: false,
            checked: Some(false),
        };

        let error = parse_checklist_item_head(&item, kind)
            .expect_err("checked item without paragraph should fail");

        assert_eq!(invalid_roadmap_message(&error), expected);
    }

    fn invalid_roadmap_message(error: &MapspliceError) -> &str {
        match error {
            MapspliceError::InvalidRoadmap { message } => message,
            other => panic!("expected InvalidRoadmap error, got {other:?}"),
        }
    }
}
