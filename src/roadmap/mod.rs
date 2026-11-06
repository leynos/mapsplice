//! Roadmap parsing, mutation, renumbering, and rendering.

mod anchor;
pub(crate) mod export;
mod model;
pub(crate) mod numbered_detail;
mod ops;
mod parse;
pub(crate) mod preservation_events;
mod render;
pub(crate) mod source_location;
pub(crate) mod source_preservation;
mod step_section;
pub(crate) mod sub_task_numbering;

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
