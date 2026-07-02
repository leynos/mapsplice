//! Addendum sub-task fragment parsing.

use markdown::mdast::{List, Node};

use super::{ParseContext, parse_sub_task_item_unchecked};
use crate::{
    error::{MapspliceError, Result},
    roadmap::{SubTaskEntry, model::SourceId},
};

pub(super) fn parse_sub_task_fragment_list(
    list: &List,
    source_text: &str,
) -> Result<Vec<SubTaskEntry>> {
    let context = ParseContext {
        source: SourceId::Fragment,
        source_text,
    };
    if list.ordered {
        return Err(MapspliceError::InvalidRoadmap {
            message: "roadmap sub-task fragments must be unordered checklist items".to_owned(),
        });
    }

    list.children
        .iter()
        .map(|node| match node {
            Node::ListItem(item) => parse_sub_task_item_unchecked(item, context),
            _ => Err(MapspliceError::InvalidRoadmap {
                message: "roadmap sub-task fragments must contain only list items".to_owned(),
            }),
        })
        .collect()
}
