//! Typed identifiers for phases, steps, tasks, and CLI anchors.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::error::{MapspliceError, Result};

/// A phase number such as `8`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PhaseNumber(pub u32);

/// A step number such as `8.2`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct StepNumber {
    /// Parent phase number.
    pub phase: PhaseNumber,
    /// Step ordinal within the phase.
    pub step: u32,
}

/// A task number such as `8.2.3`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TaskNumber {
    /// Parent step number.
    pub step: StepNumber,
    /// Task ordinal within the step.
    pub task: u32,
}

/// Structural level in a roadmap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoadmapItemLevel {
    /// Phase-level operation.
    Phase,
    /// Step-level operation.
    Step,
    /// Task-level operation.
    Task,
}

impl fmt::Display for RoadmapItemLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Phase => formatter.write_str("phase"),
            Self::Step => formatter.write_str("step"),
            Self::Task => formatter.write_str("task"),
        }
    }
}

/// A CLI anchor such as `8`, `8.2`, or `8.2.3`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum RoadmapAnchor {
    /// Phase anchor.
    Phase(PhaseNumber),
    /// Step anchor.
    Step(StepNumber),
    /// Task anchor.
    Task(TaskNumber),
}

impl RoadmapAnchor {
    /// Return the structural level.
    #[must_use]
    pub const fn level(self) -> RoadmapItemLevel {
        match self {
            Self::Phase(_) => RoadmapItemLevel::Phase,
            Self::Step(_) => RoadmapItemLevel::Step,
            Self::Task(_) => RoadmapItemLevel::Task,
        }
    }
}

impl fmt::Display for PhaseNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl fmt::Display for StepNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.phase, self.step)
    }
}

impl fmt::Display for TaskNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.step, self.task)
    }
}

impl fmt::Display for RoadmapAnchor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Phase(number) => number.fmt(formatter),
            Self::Step(number) => number.fmt(formatter),
            Self::Task(number) => number.fmt(formatter),
        }
    }
}

impl From<PhaseNumber> for RoadmapAnchor {
    fn from(value: PhaseNumber) -> Self { Self::Phase(value) }
}

impl From<StepNumber> for RoadmapAnchor {
    fn from(value: StepNumber) -> Self { Self::Step(value) }
}

impl From<TaskNumber> for RoadmapAnchor {
    fn from(value: TaskNumber) -> Self { Self::Task(value) }
}

impl StepNumber {
    /// Construct a new step number.
    #[must_use]
    pub const fn new(phase: PhaseNumber, step: u32) -> Self { Self { phase, step } }
}

impl TaskNumber {
    /// Construct a new task number.
    #[must_use]
    pub const fn new(step: StepNumber, task: u32) -> Self { Self { step, task } }
}

impl FromStr for RoadmapAnchor {
    type Err = MapspliceError;

    fn from_str(value: &str) -> Result<Self> { parse_anchor(value) }
}

/// Parse a CLI anchor such as `8`, `8.2`, or `8.2.3`.
///
/// # Errors
///
/// Returns [`MapspliceError::InvalidAnchor`] when the value is empty, contains
/// empty path segments, includes non-numeric components, or has more than
/// three numeric parts.
pub fn parse_anchor(value: &str) -> Result<RoadmapAnchor> {
    let parts = value.split('.').collect::<Vec<_>>();
    if parts.is_empty() || parts.iter().any(|part| part.is_empty()) {
        return Err(MapspliceError::InvalidAnchor {
            anchor: value.to_owned(),
        });
    }

    let numbers = parts
        .iter()
        .map(|part| parse_canonical_positive_integer(part, value))
        .collect::<Result<Vec<_>>>()?;

    match numbers.as_slice() {
        [phase] => Ok(PhaseNumber(*phase).into()),
        [phase, step] => Ok(StepNumber::new(PhaseNumber(*phase), *step).into()),
        [phase, step, task] => {
            Ok(TaskNumber::new(StepNumber::new(PhaseNumber(*phase), *step), *task).into())
        }
        _ => Err(MapspliceError::InvalidAnchor {
            anchor: value.to_owned(),
        }),
    }
}

fn parse_canonical_positive_integer(part: &str, anchor: &str) -> Result<u32> {
    let number = part
        .parse::<u32>()
        .map_err(|_| MapspliceError::InvalidAnchor {
            anchor: anchor.to_owned(),
        })?;
    if number == 0 || part != number.to_string() {
        return Err(MapspliceError::InvalidAnchor {
            anchor: anchor.to_owned(),
        });
    }
    Ok(number)
}
