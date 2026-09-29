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

#[derive(Deserialize)]
struct SubTaskNumberWire {
    task: TaskNumber,
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
