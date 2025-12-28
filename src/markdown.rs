use std::cmp::Reverse;

use chrono::NaiveDate;
use regex::Regex;

use crate::task_item::{Status, TaskItem};

const ACTIVE_TASKS_LABEL: &str = "Active Tasks";

pub fn tasks_as_markdown<F>(mut items: Vec<TaskItem>, format: F) -> String
where
    F: Fn(&TaskItem) -> String,
{
    let mut outputs: Vec<String> = vec![];
    let mut current_date: Option<NaiveDate> = None;

    // First sort by date (with dateless ones first) because we need to write the active tasks
    // first in the file and active tasks do not have a date.
    items.sort_by_key(|i| i.active_date);
    // Write dateless (active) tasks first
    outputs.push(format!("## {}", ACTIVE_TASKS_LABEL));
    for item in items {
        if item.active_date.is_none() && current_date.is_some() {
            // This should never happen due to the sorting.
            panic!(
                "⚠️ Found tasks without date AFTER tasks with date when writing! Sorting must have failed!"
            );
        }

        // Write tasks with dates together by date
        if let Some(date) = item.active_date {
            if let Some(current) = current_date {
                if date != current {
                    outputs.push(format!("\n## {}", date));
                }
            } else {
                outputs.push(format!("\n## {}", date));
            }
        }

        current_date = item.active_date;
        outputs.push(format(&item));
    }
    outputs.join("\n")
}

/// Transform archived tasks into archived format.
pub fn archived_tasks_as_markdown<F>(mut items: Vec<TaskItem>, format: F) -> String
where
    F: Fn(&TaskItem) -> String,
{
    let mut outputs: Vec<String> = vec![];
    items.sort_by_key(|i| i.active_date);
    let mut current_date: NaiveDate = Default::default();
    for item in items {
        let Some(date) = item.active_date else {
            panic!(
                "⚠️ Refusing to convert task to markdown because it has no date: {}",
                item
            );
        };
        if date != current_date {
            outputs.push(format!("\n## {}", date));
            current_date = date;
        }
        outputs.push(format!("- {}", format(&item)));
    }
    outputs.join("\n")
}

pub fn parse_markdown_tasks(
    content: &str,
    starting_id: usize,
) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
    let mut current_id: usize = starting_id;
    let mut current_date: Option<NaiveDate> = None;
    let mut tasks: Vec<TaskItem> = Vec::new();

    // Key for task status inside the `- [ ] Task`:
    // ~ is InProgress
    // x is Done
    // A is Archived
    // space is Todo
    let status_regex = Regex::new(r"^-\s+\[(\s|x|~|A)\] (\S.+)")?;

    // Links wrap the task title, like `- [ ] [Task](https://example.com)`
    let link_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)")?;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(format!("## {}", ACTIVE_TASKS_LABEL).as_str()) {
            continue;
        }
        if trimmed.starts_with("## ") {
            let date_string = trimmed.trim_start_matches("## ");
            current_date = Some(NaiveDate::parse_from_str(date_string, "%Y-%m-%d")?);
        }

        let Some(status_matches) = status_regex.captures(trimmed) else {
            // Ignore any line that's not a date or a task item.
            continue;
        };

        let status = match &status_matches[1] {
            "~" => Status::InProgress,
            "x" => Status::Done,
            "A" => Status::Archived,
            _ => Status::Todo,
        };

        let task_body = &status_matches[2];

        let star = match task_body.starts_with("⭐️ ") {
            true => Some(true),
            false => None,
        };

        let title_and_link = task_body.trim_start_matches("⭐️ ");

        // Handle task inside markdown link
        if let Some(matches) = link_regex.captures(title_and_link) {
            let title = &matches[1];
            let link = &matches[2];
            let task = TaskItem {
                id: current_id,
                title: title.to_string(),
                link: Some(link.to_string()),
                active_date: current_date,
                status,
                star,
            };
            tasks.push(task);
            current_id += 1;
            continue;
        }

        // Handle task without link
        let task = TaskItem {
            id: current_id,
            title: title_and_link.to_string(),
            link: None,
            active_date: current_date,
            status,
            star,
        };
        tasks.push(task);
        current_id += 1;
        continue;
    }

    // Starred tasks go first, checked tasks last
    tasks.sort_by_key(|t| {
        (
            t.status == Status::Done,
            Reverse(t.star),
            t.active_date,
            t.id,
        )
    });

    Ok(tasks)
}

pub fn parse_markdown_archive_with_start_id(
    content: &str,
    starting_id: usize,
) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
    let mut current_id: usize = starting_id;
    let mut current_date: Option<NaiveDate> = None;
    let mut tasks: Vec<TaskItem> = Vec::new();

    let link_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)")?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            let date_string = trimmed.trim_start_matches("## ");
            current_date = Some(NaiveDate::parse_from_str(date_string, "%Y-%m-%d")?);
        }

        if current_date.is_none() {
            // Require an explicit date for archived items.
            continue;
        }

        if trimmed.starts_with("- ") {
            if let Some(matches) = link_regex.captures(trimmed) {
                let title = &matches[1];
                let link = &matches[2];
                let id = current_id;
                current_id += 1;
                let task = TaskItem {
                    id,
                    title: title.to_string(),
                    link: Some(link.to_string()),
                    active_date: current_date,
                    status: Status::Archived,
                    star: None,
                };
                tasks.push(task);
                continue;
            }
            let title = trimmed.trim_start_matches("- ");
            let id = current_id;
            current_id += 1;
            let task = TaskItem {
                id,
                title: title.to_string(),
                link: None,
                active_date: current_date,
                status: Status::Archived,
                star: None,
            };
            tasks.push(task);
            continue;
        }
    }

    Ok(tasks)
}
