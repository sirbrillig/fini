use crate::{commands::LinkFormat, util::format_hyperlink};
use chrono::NaiveDate;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct TaskItem {
    pub id: usize,
    pub title: String,
    pub status: Status,
    pub active_date: Option<NaiveDate>,
    pub link: Option<String>,
    pub star: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Clone)]
pub enum Status {
    #[default]
    Todo,
    InProgress,
    Done,
    Archived,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Status::Todo => "☐".purple().to_string(),
                Status::InProgress => "…".yellow().to_string(),
                Status::Done => "✔".green().to_string(),
                Status::Archived => "-".green().to_string(),
            }
        )?;
        Ok(())
    }
}

impl fmt::Display for TaskItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.title)?;
        if let Some(link) = &self.link {
            write!(f, " {}", format_hyperlink(link, "[link]").dimmed())?;
        }
        Ok(())
    }
}

pub struct TaskItemWithIndex<'a>(pub &'a TaskItem, pub usize, pub LinkFormat);

impl fmt::Display for TaskItemWithIndex<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let task = self.0;
        let index = self.1;
        let format = self.2;
        write!(f, "{:>2}.{}", index, TaskItemWithStatus(task, format))?;
        Ok(())
    }
}

pub struct TaskItemWithStatus<'a>(pub &'a TaskItem, pub LinkFormat);

impl fmt::Display for TaskItemWithStatus<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let task = self.0;
        let format = self.1;
        let text = match format {
            LinkFormat::Adjacent => TaskItemCopyable(task).to_string(),
            LinkFormat::Hyperlink => task.to_string(),
            LinkFormat::Markdown => TaskItemCopyableMarkdown(task).to_string(),
        };
        match task.status {
            // Archived status doesn't require any extra formatting because it will never be mixed
            // with other statuses and will never be starred.
            Status::Archived => {
                write!(f, "{}", task.title)?;
            }
            _ => {
                if task.star.is_some_and(|v| v) {
                    write!(f, "{} ", "★".yellow())?;
                } else {
                    write!(f, "  ")?;
                }
                write!(f, "{}  {}", task.status, text)?;
            }
        }
        Ok(())
    }
}

pub struct TaskItemCopyable<'a>(pub &'a TaskItem);

impl fmt::Display for TaskItemCopyable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.title)?;
        if let Some(link) = &self.0.link {
            write!(f, " {}", link)?;
        }
        Ok(())
    }
}

pub struct TaskItemCopyableMarkdown<'a>(pub &'a TaskItem);

impl fmt::Display for TaskItemCopyableMarkdown<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(link) = &self.0.link {
            write!(f, "[{}]({})", self.0.title, link)?;
        } else {
            write!(f, "{}", self.0.title)?;
        }
        Ok(())
    }
}
