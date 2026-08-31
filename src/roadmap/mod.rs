//! Roadmap parsing, mutation, renumbering, and rendering.

mod anchor;
mod model;
mod ops;
mod parse;
mod render;
pub(crate) mod source_preservation;
mod step_section;
#[cfg(test)]
#[path = "task_source_tests.rs"]
mod task_source_tests;

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
pub use ops::{RoadmapOperation, apply_command};
pub use parse::{parse_fragment, parse_roadmap};
pub use render::render_roadmap;
