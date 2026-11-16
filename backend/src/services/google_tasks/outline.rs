//! How a Google Tasks list's shape maps to intake (issue #86).
//!
//! - A task with subtasks is a heading ("BBQ" over "sausages", "buns"): a
//!   group name, not a grocery. It is never recorded, and it is deleted once
//!   every subtask under it is stored and gone from Google.
//! - Every other task, subtask or not, is one item.
//! - A task's notes are kept for a person to read, never parsed.
//!
//! Pure, so the poller only does the calls.

use std::collections::HashSet;

use super::api::RemoteTask;

/// Longest note kept, in characters; the column's CHECK matches.
pub const MAX_NOTE_CHARS: usize = 2000;

/// A list split into the tasks to record and the headings above them.
#[derive(Debug, PartialEq, Eq)]
pub struct Outline<'a> {
    /// Tasks that are items, in the list's order.
    pub items: Vec<&'a RemoteTask>,
    /// Tasks that have subtasks on this list.
    pub headings: Vec<&'a RemoteTask>,
}

/// Splits `tasks` into items and headings.
pub fn outline(tasks: &[RemoteTask]) -> Outline<'_> {
    let parents: HashSet<&str> = tasks
        .iter()
        .filter_map(|task| task.parent.as_deref())
        .collect();
    let (headings, items) = tasks
        .iter()
        .partition(|task| parents.contains(task.id.as_str()));
    Outline { items, headings }
}

/// The ids of `heading`'s subtasks.
pub fn subtasks_of<'a>(heading: &RemoteTask, tasks: &'a [RemoteTask]) -> Vec<&'a str> {
    tasks
        .iter()
        .filter(|task| task.parent.as_deref() == Some(heading.id.as_str()))
        .map(|task| task.id.as_str())
        .collect()
}

/// The note to keep: trimmed, `None` when blank, cut to [`MAX_NOTE_CHARS`].
pub fn note_of(task: &RemoteTask) -> Option<String> {
    let trimmed = task.notes.as_deref()?.trim();
    (!trimmed.is_empty()).then(|| trimmed.chars().take(MAX_NOTE_CHARS).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(id: &str, parent: Option<&str>, notes: Option<&str>) -> RemoteTask {
        RemoteTask {
            id: id.to_string(),
            title: id.to_string(),
            notes: notes.map(str::to_string),
            parent: parent.map(str::to_string),
        }
    }

    #[test]
    fn a_task_with_subtasks_is_a_heading_and_its_subtasks_are_items() {
        let tasks = [
            task("bbq", None, None),
            task("sausages", Some("bbq"), None),
            task("milk", None, None),
            task("buns", Some("bbq"), None),
        ];
        let split = outline(&tasks);
        let ids = |list: &[&RemoteTask]| list.iter().map(|t| t.id.clone()).collect::<Vec<_>>();
        assert_eq!(ids(&split.items), ["sausages", "milk", "buns"]);
        assert_eq!(ids(&split.headings), ["bbq"]);
        assert_eq!(subtasks_of(&tasks[0], &tasks), ["sausages", "buns"]);
    }

    #[test]
    fn a_subtask_whose_heading_is_gone_is_still_an_item() {
        let tasks = [task("sausages", Some("deleted-bbq"), None)];
        let split = outline(&tasks);
        assert_eq!(split.items.len(), 1);
        assert!(split.headings.is_empty());
    }

    #[test]
    fn notes_are_trimmed_blank_is_none_and_long_ones_are_cut() {
        assert_eq!(
            note_of(&task("a", None, Some("  lactose free \n"))),
            Some("lactose free".to_string())
        );
        assert_eq!(note_of(&task("a", None, Some(" \n "))), None);
        assert_eq!(note_of(&task("a", None, None)), None);
        let long = "x".repeat(MAX_NOTE_CHARS + 5);
        assert_eq!(
            note_of(&task("a", None, Some(&long)))
                .unwrap()
                .chars()
                .count(),
            MAX_NOTE_CHARS
        );
    }
}
