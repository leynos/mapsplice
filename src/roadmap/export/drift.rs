//! Semantic comparison for committed roadmap export documents.

use super::schema::{
    ExportDetail,
    ExportDocument,
    ExportEntry,
    ExportPhase,
    ExportState,
    ExportStep,
    ExportTask,
};

/// Return stable item-oriented differences between two decoded exports.
pub(crate) fn differences(committed: &ExportDocument, fresh: &ExportDocument) -> Vec<String> {
    if committed.format_version != fresh.format_version {
        return vec![format!(
            "~ format_version: {} -> {}",
            committed.format_version, fresh.format_version
        )];
    }
    let mut output = Vec::new();
    let mut report = DriftReport::new(&mut output);
    let old_phase_numbers = old_numbers(&committed.phases, |phase| phase.number.clone());
    let new_phase_numbers = old_numbers(&fresh.phases, |phase| phase.number.clone());
    report.order("roadmap phases", &old_phase_numbers, &new_phase_numbers);
    compare_phases(&committed.phases, &fresh.phases, &mut report);
    output
}

/// Collect semantic drift lines without coupling comparisons to formatting.
struct DriftReport<'a> {
    /// Ordered human-readable difference lines.
    lines: &'a mut Vec<String>,
}

impl<'a> DriftReport<'a> {
    /// Start a report over the supplied output buffer.
    const fn new(lines: &'a mut Vec<String>) -> Self { Self { lines } }

    /// Report a changed sequence of identifiers.
    fn order(&mut self, field: &str, old: &[String], new: &[String]) {
        if old != new {
            self.lines
                .push(format!("~ {field} order: {old:?} -> {new:?}"));
        }
    }

    /// Report a changed string value.
    fn string(&mut self, field: &str, old: &str, new: &str) {
        if old != new {
            self.lines.push(format!("~ {field}: {old:?} -> {new:?}"));
        }
    }

    /// Report a changed string list.
    fn strings(&mut self, field: &str, old: &[String], new: &[String]) {
        if old != new {
            self.lines.push(format!("~ {field}: {old:?} -> {new:?}"));
        }
    }

    /// Report a changed optional string value.
    fn optional_string(&mut self, field: &str, old: Option<&str>, new: Option<&str>) {
        if old != new {
            self.lines.push(format!("~ {field}: {old:?} -> {new:?}"));
        }
    }

    /// Report a changed checkbox state.
    fn state(&mut self, item: &str, old: ExportState, new: ExportState) {
        self.string(&format!("{item} state"), old.as_str(), new.as_str());
    }
}

/// Compare phases by their dotted number.
fn compare_phases(old: &[ExportPhase], new: &[ExportPhase], report: &mut DriftReport<'_>) {
    for phase in new {
        let Some(previous) = old.iter().find(|item| item.number == phase.number) else {
            report.lines.push(format!("+ phase {} added", phase.number));
            continue;
        };
        let item = format!("phase {}", phase.number);
        report.string(&format!("{item} title"), &previous.title, &phase.title);
        report.strings(
            &format!("{item} requires"),
            &previous.requires,
            &phase.requires,
        );
        compare_entries(&item, &previous.entries, &phase.entries, report);
        compare_entries(
            &format!("{item} trailing"),
            &previous.trailing,
            &phase.trailing,
            report,
        );
        compare_steps(&phase.number, &previous.steps, &phase.steps, report);
    }
    for phase in old {
        if !new.iter().any(|item| item.number == phase.number) {
            report
                .lines
                .push(format!("- phase {} removed", phase.number));
        }
    }
}

/// Compare steps by their dotted number.
fn compare_steps(
    phase_number: &str,
    old: &[ExportStep],
    new: &[ExportStep],
    report: &mut DriftReport<'_>,
) {
    let old_step_numbers = old_numbers(old, |step| step.number.clone());
    let new_step_numbers = old_numbers(new, |step| step.number.clone());
    report.order(
        &format!("phase {phase_number} steps"),
        &old_step_numbers,
        &new_step_numbers,
    );
    for step in new {
        let item = format!("step {}", step.number);
        let Some(previous) = old.iter().find(|candidate| candidate.number == step.number) else {
            report.lines.push(format!("+ {item} added"));
            continue;
        };
        report.string(&format!("{item} title"), &previous.title, &step.title);
        report.optional_string(
            &format!("{item} question"),
            previous.question.as_deref(),
            step.question.as_deref(),
        );
        report.strings(
            &format!("{item} requires"),
            &previous.requires,
            &step.requires,
        );
        compare_entries(&item, &previous.entries, &step.entries, report);
        compare_entries(
            &format!("{item} trailing"),
            &previous.trailing,
            &step.trailing,
            report,
        );
        compare_tasks(&step.number, &previous.tasks, &step.tasks, report);
    }
    for step in old {
        if !new.iter().any(|candidate| candidate.number == step.number) {
            report.lines.push(format!("- step {} removed", step.number));
        }
    }
}

/// Compare tasks by their dotted number.
fn compare_tasks(
    step_number: &str,
    old: &[ExportTask],
    new: &[ExportTask],
    report: &mut DriftReport<'_>,
) {
    let old_task_numbers = old_numbers(old, |task| task.number.clone());
    let new_task_numbers = old_numbers(new, |task| task.number.clone());
    report.order(
        &format!("step {step_number} tasks"),
        &old_task_numbers,
        &new_task_numbers,
    );
    for task in new {
        let item = format!("task {}", task.number);
        let Some(previous) = old.iter().find(|candidate| candidate.number == task.number) else {
            report.lines.push(format!("+ {item} added"));
            continue;
        };
        report.string(&format!("{item} title"), &previous.title, &task.title);
        report.state(&item, previous.state, task.state);
        report.strings(
            &format!("{item} requires"),
            &previous.requires,
            &task.requires,
        );
        compare_entries(&item, &previous.entries, &task.entries, report);
        compare_details(&item, &previous.details, &task.details, report);
    }
    for task in old {
        if !new.iter().any(|candidate| candidate.number == task.number) {
            report.lines.push(format!("- task {} removed", task.number));
        }
    }
}

/// Compare numbered details by number and unnumbered details by source ordinal.
///
/// `owner` is already formatted, such as `task 1.1.1` or `sub-task 1.1.1.1`, so
/// recursive calls do not stack another level of prefix onto the label.
fn compare_details(
    owner: &str,
    old: &[ExportDetail],
    new: &[ExportDetail],
    report: &mut DriftReport<'_>,
) {
    let old_order = detail_order(old);
    let new_order = detail_order(new);
    report.order(&format!("{owner} details"), &old_order, &new_order);
    let old_unnumbered = old
        .iter()
        .filter(|detail| detail.number.is_none())
        .collect::<Vec<_>>();
    let mut new_unnumbered_ordinal = 0;

    for detail in new {
        let matched = detail.number.as_ref().map_or_else(
            || {
                let result = old_unnumbered.get(new_unnumbered_ordinal).copied();
                new_unnumbered_ordinal += 1;
                result
            },
            |number| {
                old.iter()
                    .find(|candidate| candidate.number.as_ref() == Some(number))
            },
        );
        let name = detail.number.as_ref().map_or_else(
            || format!("detail {new_unnumbered_ordinal} of {owner}"),
            |number| format!("sub-task {number}"),
        );
        let Some(previous) = matched else {
            report.lines.push(format!("+ {name} added"));
            continue;
        };
        compare_detail(&name, previous, detail, report);
    }

    let mut old_unnumbered_ordinal = 0;
    for detail in old {
        let removed = detail.number.as_ref().map_or_else(
            || {
                old_unnumbered_ordinal += 1;
                new_unnumbered_ordinal < old_unnumbered_ordinal
            },
            |number| {
                !new.iter()
                    .any(|candidate| candidate.number.as_ref() == Some(number))
            },
        );
        if removed {
            let name = detail.number.as_ref().map_or_else(
                || format!("detail {old_unnumbered_ordinal} of {owner}"),
                |number| format!("sub-task {number}"),
            );
            report.lines.push(format!("- {name} removed"));
        }
    }
}

/// Collect identifiers from an ordered collection.
fn old_numbers<T>(items: &[T], number: impl Fn(&T) -> String) -> Vec<String> {
    items.iter().map(number).collect()
}

/// Describe structural and unnumbered details in their source order.
fn detail_order(details: &[ExportDetail]) -> Vec<String> {
    details
        .iter()
        .enumerate()
        .map(|(index, detail)| {
            detail
                .number
                .clone()
                .unwrap_or_else(|| format!("detail {}", index + 1))
        })
        .collect()
}

/// Compare one matched detail and its nested content.
fn compare_detail(
    name: &str,
    old: &ExportDetail,
    new: &ExportDetail,
    report: &mut DriftReport<'_>,
) {
    report.string(&format!("{name} title"), &old.title, &new.title);
    report.state(name, old.state, new.state);
    report.strings(&format!("{name} requires"), &old.requires, &new.requires);
    compare_entries(name, &old.entries, &new.entries, report);
    compare_details(name, &old.details, &new.details, report);
}

/// Compare entries by source ordinal and report their changed fields.
fn compare_entries(
    item: &str,
    old: &[ExportEntry],
    new: &[ExportEntry],
    report: &mut DriftReport<'_>,
) {
    for (index, entry) in new.iter().enumerate() {
        let name = format!("{item} entry {}", index + 1);
        let Some(previous) = old.get(index) else {
            report.lines.push(format!("+ {name} added"));
            continue;
        };
        report.optional_string(
            &format!("{name} key"),
            previous.key.as_deref(),
            entry.key.as_deref(),
        );
        report.string(&format!("{name} value"), &previous.value, &entry.value);
        compare_entries(&name, &previous.entries, &entry.entries, report);
    }
    for index in new.len()..old.len() {
        report
            .lines
            .push(format!("- {item} entry {} removed", index + 1));
    }
}
