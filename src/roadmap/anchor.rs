//! Typed identifiers for phases, steps, tasks, and CLI anchors.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, de::Error as SerdeError};

use crate::error::{MapspliceError, Result};

/// A phase number such as `8`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct PhaseNumber(u32);

/// A step number such as `8.2`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct StepNumber {
    phase: PhaseNumber,
    step: u32,
}

/// A task number such as `8.2.3`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TaskNumber {
    step: StepNumber,
    task: u32,
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

impl PhaseNumber {
    /// Construct a validated phase number.
    ///
    /// # Errors
    ///
    /// Returns [`MapspliceError::InvalidAnchor`] when `phase` is zero.
    pub fn new(phase: u32) -> Result<Self> { validate_positive("phase", phase).map(Self) }

    /// Return the numeric phase value.
    #[must_use]
    pub const fn get(self) -> u32 { self.0 }
}

impl StepNumber {
    /// Construct a validated step number.
    ///
    /// # Errors
    ///
    /// Returns [`MapspliceError::InvalidAnchor`] when `step` is zero.
    pub fn new(phase: PhaseNumber, step: u32) -> Result<Self> {
        let step_number = validate_positive("step", step)?;
        Ok(Self {
            phase,
            step: step_number,
        })
    }

    /// Return the parent phase number.
    #[must_use]
    pub const fn phase(self) -> PhaseNumber { self.phase }

    /// Return the step ordinal within the phase.
    #[must_use]
    pub const fn step(self) -> u32 { self.step }
}

impl TaskNumber {
    /// Construct a validated task number.
    ///
    /// # Errors
    ///
    /// Returns [`MapspliceError::InvalidAnchor`] when `task` is zero.
    pub fn new(step: StepNumber, task: u32) -> Result<Self> {
        let task_number = validate_positive("task", task)?;
        Ok(Self {
            step,
            task: task_number,
        })
    }

    /// Return the parent step number.
    #[must_use]
    pub const fn step_number(self) -> StepNumber { self.step }

    /// Return the task ordinal within the step.
    #[must_use]
    pub const fn task(self) -> u32 { self.task }
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
        [phase] => Ok(PhaseNumber::new(*phase)?.into()),
        [phase, step] => Ok(StepNumber::new(PhaseNumber::new(*phase)?, *step)?.into()),
        [phase, step, task] => {
            Ok(TaskNumber::new(StepNumber::new(PhaseNumber::new(*phase)?, *step)?, *task)?.into())
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

fn validate_positive(label: &str, number: u32) -> Result<u32> {
    if number == 0 {
        return Err(MapspliceError::InvalidAnchor {
            anchor: format!("{label} number 0"),
        });
    }
    Ok(number)
}

impl<'de> Deserialize<'de> for PhaseNumber {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(u32::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

#[derive(Deserialize)]
struct StepNumberWire {
    phase: PhaseNumber,
    step: u32,
}

impl<'de> Deserialize<'de> for StepNumber {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = StepNumberWire::deserialize(deserializer)?;
        Self::new(wire.phase, wire.step).map_err(D::Error::custom)
    }
}

#[derive(Deserialize)]
struct TaskNumberWire {
    step: StepNumber,
    task: u32,
}

impl<'de> Deserialize<'de> for TaskNumber {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = TaskNumberWire::deserialize(deserializer)?;
        Self::new(wire.step, wire.task).map_err(D::Error::custom)
    }
}
