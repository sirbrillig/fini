use std::cmp::Reverse;

use crate::{
    copier::Copier,
    storage::TaskStorage,
    task_item::{Status, TaskItem, TaskItemWithIndex},
};
use chrono::{Local, NaiveDate};
use colored::Colorize;
use edit::edit;
use inquire::{Confirm, MultiSelect, Select};

pub const SELECT_PAGE_SIZE: usize = 20;

pub fn archived_tasks_as_markdown(mut items: Vec<TaskItem>) -> String {
    // Markdown can't have color codes so we should always remove those if copying this text but I
    // would like them when printing in the terminal so I'm leaving that to the caller.
    let mut outputs: Vec<String> = vec![];
    items.sort_by_key(|i| i.active_date);
    let mut current_date: NaiveDate = Default::default();
    for item in items {
        if item.status == Status::Archived {
            let Some(date) = item.active_date else {
                continue;
            };
            if date != current_date {
                outputs.push(format!("\n## {}", date.to_string().green()));
                current_date = date;
            }
            outputs.push(format!("- {}", item));
        }
    }
    outputs.join("\n")
}

pub fn tasks_as_markdown_by_date<F>(mut items: Vec<TaskItem>, format: F) -> String
where
    F: Fn(&TaskItem) -> String,
{
    let mut outputs: Vec<String> = vec![];
    items.sort_by_key(|i| i.active_date);
    let mut current_date: NaiveDate = Default::default();
    for item in items {
        let Some(date) = item.active_date else {
            continue;
        };
        if date != current_date {
            outputs.push(format!("\n## {}", date));
            current_date = date;
        }
        outputs.push(format!("- {}", format(&item)));
    }
    outputs.join("\n")
}

pub fn sort_visible_items(items: &[TaskItem]) -> Vec<&TaskItem> {
    let mut visible: Vec<_> = items
        .iter()
        .filter(|i| matches!(i.status, Status::Todo | Status::InProgress | Status::Done))
        .collect();
    visible.sort_by_key(|i| Reverse(i.star));
    visible
}

pub fn get_task_ids_before_date(
    storage: &dyn TaskStorage,
    date: String,
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let tasks = get_all_items(storage)?;
    // TODO: parse the input date so it can be various formats like "yesterday"
    Ok(tasks
        .iter()
        .filter_map(|t| {
            let task_date = t.active_date?;
            if task_date.format("%Y-%m-%d").to_string() < date {
                Some(t.id)
            } else {
                None
            }
        })
        .collect())
}

/// Return all task IDs after the given date, inclusive
pub fn get_task_ids_after_date(
    storage: &dyn TaskStorage,
    date: String,
    statuses: &[Status],
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let tasks = get_all_items(storage)?;
    // TODO: parse the input date so it can be various formats like "yesterday"
    Ok(tasks
        .iter()
        .filter(|t| statuses.contains(&t.status))
        .filter_map(|t| {
            let task_date = t.active_date?;
            if task_date.format("%Y-%m-%d").to_string() >= date {
                Some(t.id)
            } else {
                None
            }
        })
        .collect())
}

pub fn get_tasks_for_ids(
    storage: &dyn TaskStorage,
    ids: &[usize],
) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
    let tasks = get_all_items(storage)?;
    Ok(tasks.into_iter().filter(|t| ids.contains(&t.id)).collect())
}

pub fn prompt_for_task_id(
    storage: &dyn TaskStorage,
    message: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let items = storage.read()?;
    let visible = sort_visible_items(&items);
    let val = Select::new(message, visible)
        .with_page_size(SELECT_PAGE_SIZE)
        .prompt()?;
    Ok(val.id)
}

pub fn prompt_for_task_ids(
    storage: &dyn TaskStorage,
    message: &str,
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let items = storage.read()?;
    let visible = sort_visible_items(&items);
    let val = MultiSelect::new(message, visible)
        .with_page_size(SELECT_PAGE_SIZE)
        .prompt()?;
    Ok(val.iter().map(|i| i.id).collect())
}

pub fn get_all_items(
    storage: &dyn TaskStorage,
) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
    storage.read()
}

pub fn get_visible_items(
    storage: &dyn TaskStorage,
) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
    let items = storage.read()?;
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
    title: String,
    link: Option<String>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
    let id = get_next_id(&tasks);
    let item = TaskItem {
        id,
        title,
        link,
        ..Default::default()
    };
    println!("Added task: {}", &item.title);
    tasks.push(item);
    storage.write(tasks)?;
    Ok(id)
}

pub fn list(storage: &dyn TaskStorage) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = storage.read()?;
    let visible = sort_visible_items(&tasks);
    if visible.is_empty() {
        println!("No tasks");
    } else {
        for (index, item) in visible.iter().enumerate() {
            println!("{}", TaskItemWithIndex(item, index + 1));
        }
    }
    Ok(())
}

pub fn archived(storage: &dyn TaskStorage) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = storage.read()?;
    print!("{}", archived_tasks_as_markdown(tasks));
    Ok(())
}

pub fn work(
    storage: &mut dyn TaskStorage,
    ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
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
            println!("Moved task back to todo: {}", item.title);
        } else {
            item.status = Status::InProgress;
            item.active_date = Some(Local::now().date_naive());
            did_change = true;
            println!("Started task: {}", item.title);
        }
    }
    if did_change {
        storage.write(tasks)?;
    }
    Ok(())
}

pub fn star(
    storage: &mut dyn TaskStorage,
    ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
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
        storage.write(tasks)?;
        println!("Starred the selected tasks");
    } else {
        println!("No tasks selected");
    }
    Ok(())
}

pub fn archive(
    storage: &mut dyn TaskStorage,
    ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
    let mut did_change = false;
    for id in ids {
        let Some(item) = tasks.iter_mut().find(|t| &t.id == id) else {
            eprintln!("⚠️ No task found with id {id}");
            return Ok(());
        };
        if item.status == Status::Archived {
            item.status = Status::Todo;
            item.active_date = None;
            did_change = true;
            println!("Moved task back to todo: {}", item.title);
        } else {
            item.status = Status::Archived;
            item.active_date = Some(Local::now().date_naive());
            did_change = true;
            println!("Archived task: {}", item.title);
        }
    }
    if did_change {
        storage.write(tasks)?;
    }
    Ok(())
}

pub fn done(
    storage: &mut dyn TaskStorage,
    ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
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
            println!("Moved task back to todo: {}", item.title);
        } else {
            item.status = Status::Done;
            item.active_date = Some(Local::now().date_naive());
            did_change = true;
            println!("Completed task: {}", item.title);
        }
    }
    if did_change {
        storage.write(tasks)?;
    }
    Ok(())
}

pub fn delete(
    storage: &mut dyn TaskStorage,
    ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
    tasks.retain(|i| !ids.contains(&i.id));
    storage.write(tasks)?;
    Ok(())
}

pub fn copy<F>(
    storage: &dyn TaskStorage,
    copier: &mut dyn Copier,
    ids: &[usize],
    format: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(&TaskItem) -> String,
{
    let tasks = storage.read()?;
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
        0 => println!("No tasks to copy"),
        1 => println!("Copied text for task: {}", text_lines[0]),
        _ => println!("Copied text for selected tasks"),
    };
    Ok(())
}

pub fn clear(storage: &mut dyn TaskStorage) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
    let mut copies: Vec<TaskItem> = vec![];
    let mut next_id = get_next_id(&tasks);
    for item in tasks.iter_mut() {
        // Unstar all tasks.
        item.star = None;
        match item.status {
            Status::Done => item.status = Status::Archived,
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
                copies.push(copy);
                // Return in-progress tasks to To do
                item.status = Status::Todo;
            }
            _ => {}
        }
    }
    tasks.extend(copies);
    storage.write(tasks)?;
    println!("Archived completed tasks");
    Ok(())
}

pub fn edit_link(
    storage: &mut dyn TaskStorage,
    id: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
    let Some(item) = tasks.iter_mut().find(|t| t.id == id) else {
        eprintln!("⚠️ No task found with id {id}");
        return Ok(());
    };
    let current_link = item.link.clone().unwrap_or_default();

    let new_link = edit(&current_link)?;
    let new_link = new_link.trim().to_string();
    if new_link.is_empty() {
        eprintln!("⚠️ link cannot be empty");
        return Ok(());
    }

    if new_link == current_link {
        println!("No changes made");
        return Ok(());
    }

    // Update the task
    println!("Updated task: {}", new_link);
    item.link = Some(new_link);
    storage.write(tasks)?;
    Ok(())
}

pub fn edit_task(
    storage: &mut dyn TaskStorage,
    id: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
    let Some(item) = tasks.iter_mut().find(|t| t.id == id) else {
        return Err("No task found to edit".into());
    };

    let new_title = edit(&item.title)?;
    let new_title = new_title.trim().to_string();
    if new_title.is_empty() {
        eprintln!("⚠️ Title cannot be empty");
        return Ok(());
    }

    if new_title != item.title {
        item.title = new_title.clone();
        storage.write(tasks)?;
        println!("Updated task: {}", new_title);
    }

    let confirm_answer = Confirm::new("Do you want to edit the link?")
        .with_default(false)
        .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
        .prompt();
    if confirm_answer.is_ok_and(|x| x) {
        edit_link(storage, id)?;
    }
    Ok(())
}
