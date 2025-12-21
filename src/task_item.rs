use crate::{
    commands::LinkFormat,
    util::{format_hyperlink, get_task_link_for_format},
};
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
        Ok(())
    }
}

// An intermediate format of a task with a number before it and status icons.
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

// An intermediate format of a task with the status and star icons included.
pub struct TaskItemWithStatus<'a>(pub &'a TaskItem, pub LinkFormat);
impl fmt::Display for TaskItemWithStatus<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let task = self.0;
        let format = self.1;
        let text = get_task_link_for_format(task, format);
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

// A version of the task where the link is adjacent to the text, corresponding to
// `LinkFormat::Adjacent`.
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

// A version of the task where the link is markdown linked to the text, corresponding to
// `LinkFormat::Markdown`.
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

// A version of the task where the link is OSC 8 linked to the text, corresponding to
// `LinkFormat::Hyperlink`.
pub struct TaskItemHyperlinked<'a>(pub &'a TaskItem);
impl fmt::Display for TaskItemHyperlinked<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0.title)?;
        if let Some(link) = &self.0.link {
            write!(f, " {}", format_hyperlink(link, "[link]").dimmed())?;
        }
        Ok(())
    }
}

// A version of the task in a custom Markdown-adjacent format which includes all metadata except
// date.
pub struct TaskItemFiniMarkdown<'a>(pub &'a TaskItem);
impl fmt::Display for TaskItemFiniMarkdown<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.status {
            Status::Todo => write!(f, "- [ ] ")?,
            Status::InProgress => write!(f, "- [~] ")?,
            Status::Done => write!(f, "- [x] ")?,
            Status::Archived => write!(f, "- [A] ")?,
        };
        match self.0.star {
            Some(_) => write!(f, "⭐️ ")?,
            None => write!(f, "")?,
        }
        if let Some(link) = &self.0.link {
            write!(f, "[{}]({})", self.0.title, link)?;
        } else {
            write!(f, "{}", self.0.title)?;
        }
        Ok(())
    }
}
