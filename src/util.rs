use crate::task_item::{Status, TaskItem};
use chrono::NaiveDate;
use colored::Colorize;
use directories::BaseDirs;
use inquire::MultiSelect;
use std::io::Write;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

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

pub fn read_data(path: &PathBuf) -> Vec<TaskItem> {
    if let Ok(data) = fs::read_to_string(path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    }
}

pub fn write_data(path: &PathBuf, data: Vec<TaskItem>) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = NamedTempFile::new_in(dir)?;
    serde_json::to_writer_pretty(&mut tmp, &data)?;
    tmp.as_file_mut().flush()?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)?;
    Ok(())
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

pub fn get_task_ids_for_date(date: String) -> Vec<usize> {
    let tasks = get_all_items();
    // TODO: parse the input date so it can be various formats like "yesterday"
    tasks
        .iter()
        .filter_map(|t| {
            let task_date = t.active_date?;
            if task_date.format("%Y-%m-%d").to_string() == date {
                Some(t.id)
            } else {
                None
            }
        })
        .collect()
}

pub fn get_task_ids_before_date(date: String) -> Vec<usize> {
    let tasks = get_all_items();
    // TODO: parse the input date so it can be various formats like "yesterday"
    tasks
        .iter()
        .filter_map(|t| {
            let task_date = t.active_date?;
            if task_date.format("%Y-%m-%d").to_string() < date {
                Some(t.id)
            } else {
                None
            }
        })
        .collect()
}

pub fn get_archived_tasks() -> Vec<TaskItem> {
    let tasks = get_all_items();
    tasks
        .into_iter()
        .filter(|t| t.status == Status::Archived)
        .collect()
}

pub fn get_archived_task_ids() -> Vec<usize> {
    let tasks = get_all_items();
    tasks
        .iter()
        .filter(|t| t.status == Status::Archived)
        .map(|t| t.id)
        .collect()
}

pub fn prompt_for_task_ids(message: &str) -> Option<Vec<usize>> {
    let data_path = get_data_path();
    let items = read_data(&data_path);
    let visible = sort_visible_items(&items);
    MultiSelect::new(message, visible)
        .with_page_size(SELECT_PAGE_SIZE)
        .prompt()
        .ok()
        .map(|s| s.iter().map(|i| i.id).collect())
}

pub fn get_all_items() -> Vec<TaskItem> {
    let data_path = get_data_path();
    read_data(&data_path)
}

pub fn get_visible_items() -> Vec<TaskItem> {
    let data_path = get_data_path();
    let items = read_data(&data_path);
    items
        .into_iter()
        .filter(|i| matches!(i.status, Status::Todo | Status::InProgress | Status::Done))
        .collect()
}

pub fn get_ids_for_indices(indices: Vec<usize>) -> Vec<usize> {
    let visible = get_visible_items();
    indices
        .iter()
        .filter_map(|index| get_task_id_by_index(*index, &visible))
        .collect()
}

pub fn get_task_id_by_index(index: usize, visible: &[TaskItem]) -> Option<usize> {
    visible.get(index - 1).map(|i| i.id)
}

pub fn get_next_id(items: &[TaskItem]) -> usize {
    items.iter().map(|t| t.id).max().unwrap_or(0) + 1
}
