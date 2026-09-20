//! Roadmap parsing, mutation, renumbering, and rendering.

mod anchor;
mod model;
mod ops;
mod parse;
pub(crate) mod preservation_events;
mod render;
pub(crate) mod source_preservation;
mod step_section;

pub use anchor::{
    PhaseNumber,
    RoadmapAnchor,
    RoadmapItemLevel,
    StepNumber,
    SubTaskNumber,
    TaskNumber,
    parse_anchor,
};
pub use model::{RoadmapDocument, RoadmapFragment, SubTaskEntry, fragment_level};
pub(crate) use ops::apply_command_with_report;
pub use ops::{RoadmapOperation, apply_command};
pub use parse::{parse_fragment, parse_roadmap};
pub use render::render_roadmap;
pub(crate) use render::render_roadmap_with_report;
