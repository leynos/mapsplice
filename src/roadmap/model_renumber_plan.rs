//! Re-export of the renumbering plan from its dedicated roadmap module.
//!
//! The plan lives in `roadmap::renumber_plan` because the continuity aliases
//! added for in-place replacement need the operations module's helpers. This
//! module keeps the model's historical import path working without duplicating
//! the type.

pub(crate) use super::super::renumber_plan::RenumberPlan;
