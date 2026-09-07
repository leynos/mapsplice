//! Per-task source preservation accessors.

use super::{TaskEntry, TaskEntryParts, validate_task_children};
use crate::error::Result;

impl TaskEntry {
    /// Build a parsed task entry from parser-owned parts.
    ///
    /// A valid parts value produces a task with the supplied identity and
    /// source-preservation state. Structural child references are validated
    /// before the task is returned.
    ///
    /// # Examples
    ///
    /// A parsed task round-trips its preserved source. Internally,
    /// `validate_task_children` accepts matching child identities and rejects
    /// an unmatched reference; `original_node_source` widens a positioned node
    /// to its line and returns `None` for an unpositioned node:
    ///
    /// ```rust
    /// use mapsplice::parse_roadmap;
    ///
    /// # fn main() -> mapsplice::Result<()> {
    /// let source = concat!(
    ///     "# Roadmap\n\n## 1. Phase one\n\n### 1.1. Step one\n\n",
    ///     "  - [ ] 1.1.1. Indented task\n",
    /// );
    /// let roadmap = parse_roadmap(source)?;
    /// assert_eq!(
    ///     roadmap.phases[0].steps[0].tasks[0].number.to_string(),
    ///     "1.1.1"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Within the roadmap module, parser-owned parts exercise the complete
    /// preservation and child-validation boundary:
    ///
    /// ```rust,ignore
    /// # use markdown::{ParseOptions, mdast::{Node, Text}, to_mdast};
    /// # use super::super::source_preservation::original_node_source;
    /// # use super::{ItemIdentity, MarkdownNodes, RoadmapAnchor, SourceId, SubTaskEntry,
    /// #     TaskChild, TaskEntry, TaskEntryParts};
    /// # fn main() -> crate::error::Result<()> {
    /// let anchor = "1.1.1".parse::<RoadmapAnchor>()?;
    /// let RoadmapAnchor::Task(number) = anchor else { return Ok(()) };
    /// let identity = ItemIdentity { source: SourceId::Target, anchor };
    /// let mut task = TaskEntry::from_parts(TaskEntryParts {
    ///     identity,
    ///     number,
    ///     checked: Some(false),
    ///     summary: MarkdownNodes::new(),
    ///     body: MarkdownNodes::new(),
    ///     task_source: Some("- [ ] 1.1.1. Original".to_owned()),
    ///     sub_tasks: Vec::new(),
    ///     children: Vec::new(),
    /// })?;
    /// assert_eq!(task.task_source(), Some("- [ ] 1.1.1. Original"));
    /// task.clear_task_source();
    /// assert_eq!(task.task_source(), None);
    ///
    /// let child_anchor = "1.1.1.1".parse::<RoadmapAnchor>()?;
    /// let RoadmapAnchor::SubTask(child_number) = child_anchor else { return Ok(()) };
    /// let child = ItemIdentity { source: SourceId::Target, anchor: child_anchor };
    /// let valid = TaskEntry::from_parts(TaskEntryParts {
    ///     identity,
    ///     number,
    ///     checked: None,
    ///     summary: MarkdownNodes::new(),
    ///     body: MarkdownNodes::new(),
    ///     task_source: None,
    ///     sub_tasks: vec![SubTaskEntry {
    ///         identity: child,
    ///         number: child_number,
    ///         checked: None,
    ///         summary: MarkdownNodes::new(),
    ///         body: MarkdownNodes::new(),
    ///     }],
    ///     children: vec![TaskChild::SubTask(child)],
    /// });
    /// assert!(valid.is_ok());
    /// let invalid = TaskEntry::from_parts(TaskEntryParts {
    ///     identity,
    ///     number,
    ///     checked: None,
    ///     summary: MarkdownNodes::new(),
    ///     body: MarkdownNodes::new(),
    ///     task_source: None,
    ///     sub_tasks: Vec::new(),
    ///     children: vec![TaskChild::SubTask(identity)],
    /// });
    /// assert!(matches!(invalid, Err(crate::error::MapspliceError::InvalidRoadmap { .. })));
    ///
    /// let source = "  - [ ] 1.1.1. Indented task\n";
    /// let tree = to_mdast(source, &ParseOptions::gfm())?;
    /// let Node::Root(root) = &tree else { return Ok(()) };
    /// let list = root.children.first().ok_or_else(|| {
    ///     crate::error::MapspliceError::InvalidRoadmap { message: "missing list".to_owned() }
    /// })?;
    /// assert_eq!(original_node_source(list, source), Some(source.trim_end().to_owned()));
    /// let missing = Node::Text(Text { value: "no position".to_owned(), position: None });
    /// assert_eq!(original_node_source(&missing, "no position"), None);
    /// # Ok(())
    /// # }
    /// ```
    pub(crate) fn from_parts(parts: TaskEntryParts) -> Result<Self> {
        validate_task_children(&parts)?;
        let mut task = Self {
            identity: parts.identity,
            number: parts.number,
            checked: parts.checked,
            summary: parts.summary,
            body: parts.body,
            task_source: None,
            sub_tasks: parts.sub_tasks,
            children: parts.children,
        };
        task.set_task_source(parts.task_source);
        Ok(task)
    }

    /// Return verbatim source captured for this task during parsing.
    ///
    /// # Examples
    ///
    /// A parsed task with preserved source returns `Some(source)`; a task
    /// without captured source returns `None`. See [`Self::from_parts`] for a
    /// complete construction example.
    ///
    /// ```rust,ignore
    /// assert_eq!(task.task_source(), Some("- [ ] 1.1.1. Original"));
    /// ```
    #[must_use]
    pub(crate) fn task_source(&self) -> Option<&str> { self.task_source.as_deref() }

    /// Set verbatim source captured for this task during parsing.
    ///
    /// Passing `Some` records the exact task source; passing `None` leaves the
    /// task without a preserved source, so rendering uses canonical output.
    ///
    /// # Examples
    ///
    /// Setting `Some(source)` makes [`Self::task_source`] return that source;
    /// setting `None` selects canonical rendering instead.
    pub(crate) fn set_task_source(&mut self, source: Option<String>) { self.task_source = source; }

    /// Clear verbatim source after a semantic task edit.
    ///
    /// # Examples
    ///
    /// After [`Self::clear_task_source`], [`Self::task_source`] returns `None`,
    /// so a semantic edit cannot emit stale source.
    ///
    /// ```rust,ignore
    /// task.clear_task_source();
    /// assert_eq!(task.task_source(), None);
    /// ```
    pub(crate) fn clear_task_source(&mut self) { self.task_source = None; }
}
