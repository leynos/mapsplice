//! Serde validation of wire-format roadmap identifiers.

use serde::{Deserialize, Deserializer, de::Error as SerdeError};

use super::{PhaseNumber, StepNumber, SubTaskNumber, TaskNumber};

impl<'de> Deserialize<'de> for PhaseNumber {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(u32::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

/// Wire fields used to validate a step number on deserialization.
#[derive(Deserialize)]
struct StepNumberWire {
    /// Parent phase supplied by the serialized value.
    phase: PhaseNumber,
    /// Step ordinal supplied by the serialized value.
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

/// Wire fields used to validate a task number on deserialization.
#[derive(Deserialize)]
struct TaskNumberWire {
    /// Parent step supplied by the serialized value.
    step: StepNumber,
    /// Task ordinal supplied by the serialized value.
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

/// Wire fields used to validate a sub-task number on deserialization.
#[derive(Deserialize)]
struct SubTaskNumberWire {
    /// Parent task supplied by the serialized value.
    task: TaskNumber,
    /// Sub-task ordinal supplied by the serialized value.
    sub_task: u32,
}

impl<'de> Deserialize<'de> for SubTaskNumber {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = SubTaskNumberWire::deserialize(deserializer)?;
        Self::new(wire.task, wire.sub_task).map_err(D::Error::custom)
    }
}
