use std::cmp::Reverse;

use crate::{
    commands::LinkFormat,
    copier::Copier,
    markdown::archived_tasks_as_markdown,
    printer::Printer,
    prompter::Prompter,
    storage::TaskStorage,
    task_item::{
        Status, TaskItem, TaskItemAdjacentLink, TaskItemCopyable, TaskItemCopyableMarkdown,
        TaskItemHyperlinked, TaskItemNewlineLink, TaskItemWithIndex, TaskItemWithStatus,
    },
};
use chrono::{Local, NaiveDate};
use inquire::{MultiSelect, Select};

const SELECT_PAGE_SIZE: usize = 20;

pub fn get_task_link_for_format(task: &TaskItem, format: LinkFormat) -> String {
    match format {
        LinkFormat::None => task.to_string(),
        LinkFormat::Adjacent => TaskItemCopyable(task).to_string(),
        LinkFormat::AdjacentColor => TaskItemAdjacentLink(task).to_string(),
        LinkFormat::Hyperlink => TaskItemHyperlinked(task).to_string(),
        LinkFormat::Markdown => TaskItemCopyableMarkdown(task).to_string(),
        LinkFormat::Newline => TaskItemNewlineLink(task).to_string(),
    }
}

pub fn sort_visible_items(items: &[TaskItem]) -> Vec<&TaskItem> {
    let mut visible: Vec<_> = items
        .iter()
        .filter(|i| matches!(i.status, Status::Todo | Status::InProgress | Status::Done))
        .collect();
    visible.sort_by_key(|i| Reverse(i.star));
    visible.sort_by_key(|i| i.status == Status::Done);
    visible
}

/// Return all task IDs after the given date, inclusive
pub fn get_task_ids_after_date(
    tasks: &[TaskItem],
    date: &str,
    statuses: &[Status],
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let filter_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")?;
    Ok(tasks
        .iter()
        .filter(|t| statuses.contains(&t.status))
        .filter_map(|t| {
            let task_date = t.active_date?;
            if task_date >= filter_date {
                Some(t.id)
            } else {
                None
            }
        })
        .collect())
}

pub fn get_task_for_id(
    storage: &dyn TaskStorage,
    id: usize,
) -> Result<TaskItem, Box<dyn std::error::Error>> {
    let tasks = storage.read_tasks()?;
    let Some(item) = tasks.into_iter().find(|t| t.id == id) else {
        return Err("Task not found".into());
    };
    Ok(item)
}

pub fn get_tasks_for_ids(
    tasks: Vec<TaskItem>,
    ids: &[usize],
) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
    Ok(tasks.into_iter().filter(|t| ids.contains(&t.id)).collect())
}

pub fn prompt_for_task_id(
    storage: &dyn TaskStorage,
    message: &str,
) -> Result<Option<usize>, Box<dyn std::error::Error>> {
    let items = storage.read_tasks()?;
    let visible = sort_visible_items(&items);
    let formatted: Vec<_> = visible
        .iter()
        .map(|t| TaskItemWithStatus(t, LinkFormat::None))
        .collect();
    let val = match Select::new(message, formatted)
        .with_page_size(SELECT_PAGE_SIZE)
        .prompt()
    {
        Ok(val) => Some(val.0.id),
        Err(inquire::InquireError::OperationCanceled) => None,
        Err(err) => return Err(err.into()),
    };
    Ok(val)
}

pub fn prompt_for_task_ids(
    storage: &dyn TaskStorage,
    message: &str,
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let items = storage.read_tasks()?;
    let visible = sort_visible_items(&items);
    let formatted: Vec<_> = visible
        .iter()
        .map(|t| TaskItemWithStatus(t, LinkFormat::None))
        .collect();
    let val = MultiSelect::new(message, formatted)
        .with_page_size(SELECT_PAGE_SIZE)
        .with_select_on_empty_submit()
        .prompt()
        .or_else(|err| match err {
            inquire::InquireError::OperationCanceled => Ok(Vec::new()),
            err => Err(err),
        })?;
    Ok(val.iter().map(|i| i.0.id).collect())
}

pub fn get_visible_items(
    storage: &dyn TaskStorage,
) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
    let items = storage.read_tasks()?;
    Ok(items
        .into_iter()
        .filter(|i| matches!(i.status, Status::Todo | Status::InProgress | Status::Done))
        .collect())
}

pub fn get_task_id_by_index_from_list(index: usize, visible: &[TaskItem]) -> Option<usize> {
    visible.get(index - 1).map(|i| i.id)
}

pub fn get_next_id(items: &[TaskItem]) -> usize {
    items.iter().map(|t| t.id).max().unwrap_or(0) + 1
}

pub fn add(
    storage: &mut dyn TaskStorage,
    printer: &mut dyn Printer,
    title: String,
    link: Option<String>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut tasks = storage.read_tasks()?;
    let id = get_next_id(&tasks);
    let item = TaskItem {
        id,
        title,
        link,
        ..Default::default()
    };
    printer.print(format!("Added task: {}", &item.title).as_str());
    tasks.push(item);
    storage.write_tasks(tasks)?;
    Ok(id)
}

pub fn list(
    storage: &dyn TaskStorage,
    printer: &mut dyn Printer,
    format: LinkFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = storage.read_tasks()?;
    let visible = sort_visible_items(&tasks);
    if visible.is_empty() {
        printer.print("No tasks");
    } else {
        for (index, item) in visible.iter().enumerate() {
            printer.print(format!("{}", TaskItemWithIndex(item, index + 1, format)).as_str());
        }
    }
    Ok(())
}

pub fn archived(
    storage: &dyn TaskStorage,
    printer: &mut dyn Printer,
) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = storage.read_archived()?;
    printer.print(&archived_tasks_as_markdown(tasks, |t| {
        TaskItemCopyable(t).to_string()
    }));
    Ok(())
}

pub fn work(
    storage: &mut dyn TaskStorage,
    printer: &mut dyn Printer,
    ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read_tasks()?;
    let mut did_change = false;
    for id in ids {
        let Some(item) = tasks.iter_mut().find(|t| &t.id == id) else {
            eprintln!("⚠️ No task found with id {id}");
            return Ok(());
        };
        if item.status == Status::InProgress {
            item.status = Status::Todo;
            item.active_date = None;
            did_change = true;
            printer.print(format!("Moved task back to todo: {}", item.title).as_str());
        } else {
            item.status = Status::InProgress;
            item.active_date = Some(Local::now().date_naive());
            did_change = true;
            printer.print(format!("Started task: {}", item.title).as_str());
        }
    }
    if did_change {
        storage.write_tasks(tasks)?;
    }
    Ok(())
}

pub fn star(
    storage: &mut dyn TaskStorage,
    printer: &mut dyn Printer,
    ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read_tasks()?;
    let mut did_change = false;
    for id in ids {
        let Some(item) = tasks.iter_mut().find(|t| &t.id == id) else {
            eprintln!("⚠️ No task found with id {id}");
            return Ok(());
        };
        if item.star.is_some_and(|v| v) {
            item.star = None;
        } else {
            item.star = Some(true);
        }
        did_change = true;
    }
    if did_change {
        storage.write_tasks(tasks)?;
        printer.print("Starred the selected tasks");
    } else {
        printer.print("No tasks selected");
    }
    Ok(())
}

pub fn done(
    storage: &mut dyn TaskStorage,
    printer: &mut dyn Printer,
    ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read_tasks()?;
    let mut did_change = false;
    for id in ids {
        let Some(item) = tasks.iter_mut().find(|t| &t.id == id) else {
            eprintln!("⚠️ No task found with id {id}");
            return Ok(());
        };
        if item.status == Status::Done {
            item.status = Status::Todo;
            item.active_date = None;
            did_change = true;
            printer.print(format!("Moved task back to todo: {}", item.title).as_str());
        } else {
            item.status = Status::Done;
            item.active_date = Some(Local::now().date_naive());
            did_change = true;
            printer.print(format!("Completed task: {}", item.title).as_str());
        }
    }
    if did_change {
        storage.write_tasks(tasks)?;
    }
    Ok(())
}

pub fn delete(
    storage: &mut dyn TaskStorage,
    ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read_tasks()?;
    tasks.retain(|i| !ids.contains(&i.id));
    storage.write_tasks(tasks)?;
    Ok(())
}

pub fn archive(
    storage: &mut dyn TaskStorage,
    printer: &mut dyn Printer,
    ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read_tasks()?;
    let mut archived: Vec<TaskItem> = storage.read_archived()?;
    let mut did_change = false;
    for id in ids {
        let Some(item) = tasks.iter_mut().find(|t| &t.id == id) else {
            eprintln!("⚠️ No task found with id {id}");
            return Ok(());
        };
        if item.status == Status::Archived {
            eprintln!("Task already archived");
            return Ok(());
        }
        item.status = Status::Archived;
        if item.active_date.is_none() {
            item.active_date = Some(Local::now().date_naive());
        }
        let copy = TaskItem {
            id: item.id,
            title: item.title.clone(),
            link: item.link.clone(),
            status: Status::Archived,
            active_date: item.active_date,
            ..Default::default()
        };
        archived.push(copy);
        did_change = true;
        printer.print(format!("Archived task: {}", item.title).as_str());
    }
    if did_change {
        storage.write_tasks(
            tasks
                .into_iter()
                .filter(|t| !matches!(t.status, Status::Archived))
                .collect(),
        )?;
        storage.write_archived(archived)?;
    }
    Ok(())
}

pub fn copy<F>(
    storage: &dyn TaskStorage,
    copier: &mut dyn Copier,
    printer: &mut dyn Printer,
    ids: &[usize],
    format: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(&TaskItem) -> String,
{
    let tasks = storage.read_tasks()?;
    let text_lines: Vec<String> = tasks
        .iter()
        .filter_map(|i| {
            if ids.contains(&i.id) {
                return Some(format(i));
            }
            None
        })
        .collect();
    let text = text_lines.join("\n");
    copier.copy(&text)?;
    match text_lines.len() {
        0 => printer.print("No tasks to copy"),
        1 => printer.print(format!("Copied text for task: {}", text_lines[0]).as_str()),
        _ => printer.print("Copied text for selected tasks"),
    };
    Ok(())
}

/// Return all checked or begun tasks
pub fn get_checked_task_ids(
    storage: &mut dyn TaskStorage,
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let tasks = storage.read_tasks()?;
    Ok(tasks
        .into_iter()
        .filter(|t| matches!(t.status, Status::Done | Status::InProgress))
        .map(|task| task.id)
        .collect())
}

pub fn clear(
    storage: &mut dyn TaskStorage,
    printer: &mut dyn Printer,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read_tasks()?;
    let mut archived: Vec<TaskItem> = storage.read_archived()?;
    let mut next_id = get_next_id(&tasks);

    for item in tasks.iter_mut() {
        match item.status {
            Status::Archived => {
                // In case there are any archived tasks still in task storage, move them to
                // archived storage.
                let copy = TaskItem {
                    id: item.id,
                    title: item.title.clone(),
                    link: item.link.clone(),
                    status: Status::Archived,
                    active_date: item.active_date,
                    ..Default::default()
                };
                archived.push(copy);
            }
            Status::Done => {
                item.status = Status::Archived;
                let copy = TaskItem {
                    id: item.id,
                    title: item.title.clone(),
                    link: item.link.clone(),
                    status: Status::Archived,
                    active_date: item.active_date,
                    ..Default::default()
                };
                archived.push(copy);
            }
            Status::InProgress => {
                // Duplicate in-progress tasks and complete one of them so you can see that this
                // task was worked on today.
                let copy = TaskItem {
                    id: next_id,
                    title: item.title.clone(),
                    link: item.link.clone(),
                    status: Status::Archived,
                    active_date: item.active_date,
                    ..Default::default()
                };
                next_id += 1;
                archived.push(copy);
                // Return in-progress tasks to To do
                item.status = Status::Todo;
            }
            _ => {}
        }
    }

    storage.write_tasks(
        tasks
            .into_iter()
            .filter(|t| !matches!(t.status, Status::Archived))
            .collect(),
    )?;
    storage.write_archived(archived)?;
    printer.print("Archived completed tasks");
    Ok(())
}

pub fn edit_link(
    storage: &mut dyn TaskStorage,
    prompter: &dyn Prompter,
    printer: &mut dyn Printer,
    id: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read_tasks()?;
    let Some(item) = tasks.iter_mut().find(|t| t.id == id) else {
        eprintln!("⚠️ No task found with id {id}");
        return Ok(());
    };
    let current_link = item.link.clone().unwrap_or_default();

    let new_link = prompter.text_with_initial("Edit link:", &current_link)?;
    let new_link = new_link.trim().to_string();
    if new_link.is_empty() {
        eprintln!("⚠️ link cannot be empty");
        return Ok(());
    }

    if new_link == current_link {
        printer.print("No changes made");
        return Ok(());
    }

    // Update the task
    printer.print(format!("Updated task: {}", new_link).as_str());
    item.link = Some(new_link);
    storage.write_tasks(tasks)?;
    Ok(())
}

pub fn edit_task(
    storage: &mut dyn TaskStorage,
    prompter: &dyn Prompter,
    printer: &mut dyn Printer,
    id: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read_tasks()?;
    let Some(item) = tasks.iter_mut().find(|t| t.id == id) else {
        return Err("No task found to edit".into());
    };

    let new_title = prompter.text_with_initial("Edit task:", &item.title)?;
    let new_title = new_title.trim().to_string();
    if new_title.is_empty() {
        eprintln!("⚠️ Title cannot be empty");
        return Ok(());
    }

    if new_title != item.title {
        item.title = new_title.clone();
        storage.write_tasks(tasks)?;
        printer.print(format!("Updated task: {}", new_title).as_str());
    }

    if prompter.confirm("Do you want to edit the link?")? {
        edit_link(storage, prompter, printer, id)?;
    }
    Ok(())
}

/// Format a URL as an OSC 8 hyperlink for terminal display.
pub fn format_hyperlink(url: &str, display_text: &str) -> String {
    format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, display_text)
}
