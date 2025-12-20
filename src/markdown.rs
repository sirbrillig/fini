use chrono::NaiveDate;
use regex::Regex;

use crate::task_item::{Status, TaskItem};

pub fn parse_markdown_archive(content: &str) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
    parse_markdown_archive_with_start_id(content, 100000)
}

pub fn parse_markdown_archive_with_start_id(
    content: &str,
    starting_id: usize,
) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
    let mut current_id: usize = starting_id;
    let mut current_date: Option<NaiveDate> = None;
    let mut tasks: Vec<TaskItem> = Vec::new();

    let link_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
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
