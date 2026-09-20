//! Heading recognition and anchor extraction for roadmap parsing.

use markdown::mdast::{Heading, Node, Text};

use super::{
    super::PhaseNumber,
    MapspliceError,
    Result,
    RoadmapAnchor,
    RoadmapItemLevel,
    StepNumber,
    split_numbered_prefix,
};

/// Return whether a heading is a valid phase heading.
pub(super) fn is_phase_heading(heading: &Heading) -> bool {
    heading.depth == 2 && parse_phase_heading(heading).is_ok()
}

/// Return whether a heading is a valid step heading.
pub(super) fn is_step_heading(heading: &Heading) -> bool {
    heading.depth == 3 && parse_step_heading(heading).is_ok()
}

/// Parse a phase heading into its number and title nodes.
///
/// Returns an error when the heading does not begin with a phase anchor.
pub(super) fn parse_phase_heading(heading: &Heading) -> Result<(PhaseNumber, Vec<Node>)> {
    let (anchor, title) = strip_heading_prefix(&heading.children, RoadmapItemLevel::Phase)?;
    match anchor {
        RoadmapAnchor::Phase(number) => Ok((number, title)),
        _ => Err(MapspliceError::InvalidRoadmap {
            message: "expected a phase heading".to_owned(),
        }),
    }
}

/// Parse a step heading into its number and title nodes.
///
/// Returns an error when the heading does not begin with a step anchor.
pub(super) fn parse_step_heading(heading: &Heading) -> Result<(StepNumber, Vec<Node>)> {
    let (anchor, title) = strip_heading_prefix(&heading.children, RoadmapItemLevel::Step)?;
    match anchor {
        RoadmapAnchor::Step(number) => Ok((number, title)),
        _ => Err(MapspliceError::InvalidRoadmap {
            message: "expected a step heading".to_owned(),
        }),
    }
}

/// Remove and validate the numeric anchor at the start of a heading.
///
/// Returns its anchor and title nodes, or an error when the opening text is
/// absent, unsupported, or uses the wrong roadmap level.
fn strip_heading_prefix(
    children: &[Node],
    level: RoadmapItemLevel,
) -> Result<(RoadmapAnchor, Vec<Node>)> {
    let Node::Text(Text { value, .. }) =
        children
            .first()
            .ok_or_else(|| MapspliceError::InvalidRoadmap {
                message: "roadmap headings must start with plain text".to_owned(),
            })?
    else {
        return Err(MapspliceError::InvalidRoadmap {
            message: "roadmap headings must start with plain text".to_owned(),
        });
    };
    let (anchor, remainder) = split_numbered_prefix(value, level)?;
    let mut title = children.to_vec();
    if let Some(Node::Text(text)) = title.first_mut() {
        text.value = remainder;
    }
    Ok((anchor, title))
}
