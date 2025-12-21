use std::cmp::Reverse;

use chrono::NaiveDate;
use regex::Regex;

use crate::task_item::{Status, TaskItem};

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
    let status_regex = Regex::new(r"^- \[(\s|x|~|A)\] (\S.+)")?;

    // Links wrap the task title, like `- [ ] [Task](https://example.com)`
    let link_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)")?;

    for line in content.lines() {
        let trimmed = line.trim();
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

        let star = match task_body.starts_with("⭐️") {
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

    // Starred tasks go first
    tasks.sort_by_key(|i| Reverse(i.star));
    // Checked tasks go last
    tasks.sort_by_key(|i| i.status == Status::Done);

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
