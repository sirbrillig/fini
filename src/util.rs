use crate::{
    storage::TaskStorage,
    task_item::{Status, TaskItem},
};
use chrono::NaiveDate;
use colored::Colorize;
use directories::BaseDirs;
use inquire::{MultiSelect, Select};
use std::path::PathBuf;

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

pub fn get_data_path() -> PathBuf {
    let data_dir = BaseDirs::new()
        .map(|b| b.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    data_dir.join("fini_data.json")
}

pub fn sort_visible_items(items: &[TaskItem]) -> Vec<&TaskItem> {
    items
        .iter()
        .filter(|i| matches!(i.status, Status::Todo | Status::InProgress | Status::Done))
        .collect()
}

pub fn get_task_ids_for_date(
    storage: &dyn TaskStorage,
    date: String,
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let tasks = get_all_items(storage)?;
    // TODO: parse the input date so it can be various formats like "yesterday"
    Ok(tasks
        .iter()
        .filter_map(|t| {
            let task_date = t.active_date?;
            if task_date.format("%Y-%m-%d").to_string() == date {
                Some(t.id)
            } else {
                None
            }
        })
        .collect())
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

pub fn get_archived_tasks(
    storage: &dyn TaskStorage,
) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
    let tasks = get_all_items(storage)?;
    Ok(tasks
        .into_iter()
        .filter(|t| t.status == Status::Archived)
        .collect())
}

pub fn get_archived_task_ids(
    storage: &dyn TaskStorage,
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let tasks = get_all_items(storage)?;
    Ok(tasks
        .iter()
        .filter(|t| t.status == Status::Archived)
        .map(|t| t.id)
        .collect())
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

pub fn get_id_for_index(
    storage: &dyn TaskStorage,
    index: usize,
) -> Result<Option<usize>, Box<dyn std::error::Error>> {
    let visible = get_visible_items(storage)?;
    Ok(get_task_id_by_index(index, &visible))
}

pub fn get_ids_for_indices(
    storage: &dyn TaskStorage,
    indices: Vec<usize>,
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let visible = get_visible_items(storage)?;
    Ok(indices
        .iter()
        .filter_map(|index| get_task_id_by_index(*index, &visible))
        .collect())
}

fn get_task_id_by_index(index: usize, visible: &[TaskItem]) -> Option<usize> {
    visible.get(index - 1).map(|i| i.id)
}

pub fn get_next_id(items: &[TaskItem]) -> usize {
    items.iter().map(|t| t.id).max().unwrap_or(0) + 1
}
