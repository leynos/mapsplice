//! Shared ownership and ordering rule for structural sub-tasks.
//!
//! Both the Markdown parser and the export adapter must reject a numbered
//! detail that belongs to another task or skips an ordinal. Keeping the rule
//! here prevents the two boundaries from drifting apart; each boundary still
//! renders the returned error in its own diagnostic vocabulary.

use super::{SubTaskNumber, TaskNumber};

/// Why a numbered detail is not the next expected structural sub-task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SubTaskOrderError {
    /// The number names a different parent task.
    WrongParent {
        /// Parent task that owns the surrounding list.
        parent: TaskNumber,
        /// Number found in the document.
        actual: SubTaskNumber,
    },
    /// The number is not the next expected ordinal.
    OutOfOrder {
        /// Ordinal required at this position.
        expected: u32,
        /// Number found in the document.
        actual: SubTaskNumber,
    },
}

/// Check a sub-task's parent and expected position in document order.
///
/// # Example
///
/// ```ignore
/// let parent = TaskNumber::new(step, 1)?;
/// let first = SubTaskNumber::new(parent, 1)?;
/// assert!(validate_sub_task_position(parent, 1, first).is_ok());
/// ```
pub(crate) fn validate_sub_task_position(
    parent: TaskNumber,
    expected_ordinal: u32,
    actual: SubTaskNumber,
) -> Result<(), SubTaskOrderError> {
    if actual.task_number() != parent {
        return Err(SubTaskOrderError::WrongParent { parent, actual });
    }
    if actual.sub_task() != expected_ordinal {
        return Err(SubTaskOrderError::OutOfOrder {
            expected: expected_ordinal,
            actual,
        });
    }
    Ok(())
}
