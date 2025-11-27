use arboard::Clipboard;
use chrono::{Local, NaiveDate};
use colored::Colorize;
use directories::BaseDirs;
use edit::edit;
use inquire::{Confirm, MultiSelect, Select, Text};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

const SELECT_PAGE_SIZE: usize = 20;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TaskItem {
    pub id: usize,
    pub title: String,
    pub status: Status,
    pub active_date: Option<NaiveDate>,
    pub link: Option<String>,
    pub star: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
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
        match self.status {
            // Archived status doesn't require any extra formatting because it will never be mixed
            // with other statuses and will never be starred.
            Status::Archived => {
                write!(f, "{}", self.title)?;
            }
            _ => {
                if self.star.is_some_and(|v| v) {
                    write!(f, "{} ", "★".yellow())?;
                } else {
                    write!(f, "  ")?;
                }
                write!(f, "{}  {}", self.status, self.title)?;
            }
        }
        if let Some(link) = &self.link {
            write!(f, " {}", link.dimmed())?;
        }
        Ok(())
    }
}

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

impl TaskItem {
    pub fn get_copy_text(&self) -> String {
        let mut text = self.title.clone();
        if let Some(link) = &self.link {
            text.push_str(&format!(" {}", link));
        }
        text
    }

    pub fn print_with_index(&self, index: usize) {
        println!("{:>2}.{}", index, self);
    }
}

fn read_data(path: &PathBuf) -> Vec<TaskItem> {
    if let Ok(data) = fs::read_to_string(path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn write_data(path: &PathBuf, data: Vec<TaskItem>) -> std::io::Result<()> {
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

fn sort_visible_items(items: &[TaskItem]) -> Vec<&TaskItem> {
    items
        .iter()
        .filter(|i| matches!(i.status, Status::Todo | Status::InProgress | Status::Done))
        .collect()
}

pub fn get_task_ids_for_date(date: String) -> Vec<usize> {
    let tasks = get_all_items();
    let mut ids: Vec<usize> = vec![];
    // TODO: parse the input date so it can be various formats like "yesterday"
    tasks.iter().for_each(|t| {
        if let Some(task_date) = t.active_date {
            if task_date.format("%Y-%m-%d").to_string() == date {
                ids.push(t.id);
            }
        }
    });
    ids
}

pub fn get_task_ids_before_date(date: String) -> Vec<usize> {
    let tasks = get_all_items();
    let mut ids: Vec<usize> = vec![];
    // TODO: parse the input date so it can be various formats like "yesterday"
    tasks.iter().for_each(|t| {
        if let Some(task_date) = t.active_date {
            if task_date.format("%Y-%m-%d").to_string() < date {
                ids.push(t.id);
            }
        }
    });
    ids
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
    let mut ids: Vec<usize> = vec![];
    tasks.iter().for_each(|t| {
        if t.status == Status::Archived {
            ids.push(t.id);
        }
    });
    ids
}

fn prompt_for_task_ids(message: &str) -> Option<Vec<usize>> {
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
    let mut ids: Vec<usize> = vec![];
    indices.iter().for_each(|index| {
        if let Some(id) = get_task_id_by_index(*index, &visible) {
            ids.push(id);
        }
    });
    ids
}

pub fn get_task_id_by_index(index: usize, visible: &[TaskItem]) -> Option<usize> {
    visible.get(index - 1).map(|i| i.id)
}

fn get_next_id(items: &[TaskItem]) -> usize {
    items.iter().map(|t| t.id).max().unwrap_or(0) + 1
}

pub struct Actions {}

impl Actions {
    pub fn interactive() {
        loop {
            println!("{}", "-----------------------------------------".green());
            Actions::list();
            let commands = vec![
                "quit",
                "list",
                "add",
                "check",
                "begin",
                "star",
                "copy",
                "copy-archived",
                "copy-date",
                "copy-checked",
                "clear",
                "delete",
                "delete-before",
                "edit",
                "list-archived",
            ];
            println!("{}", "-----------------------------------------".dimmed());
            let answer = Select::new("Select a command:", commands)
                .with_page_size(4)
                .prompt();

            let answer = match answer {
                Ok(cmd) => cmd,
                Err(_) => break, // Handle ctrl-c by quitting
            };

            match answer {
                "quit" => break,
                "list" => {
                    // Do nothing as the list will be printed when we loop.
                }
                "list-archived" => Actions::archived(),
                "clear" => {
                    let confirm_answer =
                        Confirm::new("Are you sure you want to archive all complete tasks?")
                            .with_default(false)
                            .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
                            .prompt();
                    if confirm_answer.is_ok_and(|x| x) {
                        Actions::clear();
                    }
                }
                "add" => {
                    let title = Text::new("Enter task:").prompt();
                    let Ok(title) = title else {
                        println!("An error happened when asking for the task.");
                        continue;
                    };
                    if title.is_empty() {
                        println!("The title of the task cannot be empty.");
                        continue;
                    }
                    let link = Text::new("(Optional) Enter link:").prompt();
                    let Ok(link) = link else {
                        continue;
                    };
                    if link.is_empty() {
                        Actions::add(title);
                    } else {
                        Actions::add_with_link(title, link);
                    }
                }
                "copy" => {
                    if let Some(ids) = prompt_for_task_ids("Select tasks to copy") {
                        Actions::copy(ids);
                    }
                }
                "copy-checked" => {
                    let data_path = get_data_path();
                    let items = read_data(&data_path);
                    let mut ids: Vec<usize> = vec![];
                    for item in items.iter() {
                        match item.status {
                            Status::Done | Status::InProgress => ids.push(item.id),
                            _ => {}
                        }
                    }
                    Actions::copy(ids);
                }
                "copy-archived" => {
                    let items = get_archived_tasks();
                    // We have to strip escape codes to remove the color.
                    let text = strip_ansi_escapes::strip_str(archived_tasks_as_markdown(items));
                    let mut clipboard = Clipboard::new().expect("Failed to access clipboard");
                    clipboard
                        .set_text(text)
                        .expect("Failed to save text to clipboard");
                    println!("Copied archived tasks as Markdown");
                }
                "copy-date" => {
                    let date = Text::new("Enter date:").prompt();
                    if let Ok(date) = date {
                        Actions::copy(get_task_ids_for_date(date));
                    }
                }
                "edit" => {
                    let data_path = get_data_path();
                    let items = read_data(&data_path);
                    let visible = sort_visible_items(&items);
                    if let Ok(selection) = Select::new("Select task to edit", visible)
                        .with_page_size(SELECT_PAGE_SIZE)
                        .prompt()
                    {
                        Actions::edit(selection.id);
                    }
                }
                "check" => {
                    if let Some(ids) = prompt_for_task_ids("Select tasks to complete") {
                        Actions::done(ids);
                    }
                }
                "star" => {
                    if let Some(ids) = prompt_for_task_ids("Select tasks to star") {
                        Actions::star(ids);
                    }
                }
                "delete-before" => {
                    let date = Text::new("Enter date:").prompt();
                    if let Ok(date) = date {
                        let confirm_answer = Confirm::new(&format!(
                            "Are you sure you want to delete all archived tasks before {}?",
                            date
                        ))
                        .with_default(false)
                        .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
                        .prompt();
                        if confirm_answer.is_ok_and(|x| x) {
                            Actions::delete(get_task_ids_before_date(date));
                        }
                    }
                }
                "delete" => {
                    if let Some(ids) = prompt_for_task_ids("Select tasks to delete") {
                        Actions::delete(ids);
                    }
                }
                "begin" => {
                    if let Some(ids) = prompt_for_task_ids("Select tasks to start") {
                        Actions::work(ids);
                    }
                }
                _ => println!("Unknown command"),
            }
        }
    }

    pub fn add(title: String) -> usize {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        let id = get_next_id(&items);
        let item = TaskItem {
            id,
            title: title.clone(),
            ..Default::default()
        };
        items.push(item);
        write_data(&data_path, items).expect("Failed to write file!");
        println!("Added task: {}", title);
        id
    }

    pub fn add_with_link(title: String, link: String) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        let id = get_next_id(&items);
        let item = TaskItem {
            id,
            title: title.clone(),
            link: Some(link),
            ..Default::default()
        };
        items.push(item);
        write_data(&data_path, items).expect("Failed to write file!");
        println!("Added task: {}", title);
    }

    pub fn link(id: usize, link: String) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        if let Some(item) = items.iter_mut().find(|t| t.id == id) {
            item.link = Some(link);
            println!("Added link to task: {}", item.title);
        }
        write_data(&data_path, items).expect("Failed to write file!");
    }

    pub fn list() {
        let data_path = get_data_path();
        let items = read_data(&data_path);
        let visible = sort_visible_items(&items);
        if visible.is_empty() {
            println!("No tasks");
        } else {
            for (index, item) in visible.iter().enumerate() {
                item.print_with_index(index + 1);
            }
        }
    }

    pub fn archived() {
        let data_path = get_data_path();
        let items = read_data(&data_path);
        print!("{}", archived_tasks_as_markdown(items));
    }

    pub fn edit_link(id: usize) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        if let Some(item) = items.iter_mut().find(|t| t.id == id) {
            let item_id = item.id;
            let current_link = item.link.clone().unwrap_or_default();

            match edit(&current_link) {
                Ok(new_link) => {
                    let new_link = new_link.trim().to_string();
                    if new_link.is_empty() {
                        eprintln!("⚠️ link cannot be empty");
                        return;
                    }

                    if new_link == current_link {
                        println!("No changes made");
                        return;
                    }

                    // Update the task
                    if let Some(item) = items.iter_mut().find(|t| t.id == item_id) {
                        item.link = Some(new_link.clone());
                        write_data(&data_path, items).expect("Failed to write file!");
                        println!("Updated task: {}", new_link);
                        return;
                    }
                }
                Err(e) => {
                    eprintln!("⚠️ Error editing task: {}", e);
                    return;
                }
            }
        }
        eprintln!("⚠️ No task found with id {id}");
    }

    pub fn edit(id: usize) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        if let Some(item) = items.iter_mut().find(|t| t.id == id) {
            let item_id = item.id;
            let current_title = item.title.clone();

            // Open editor with current title
            match edit(&current_title) {
                Ok(new_title) => {
                    let new_title = new_title.trim().to_string();
                    if new_title.is_empty() {
                        eprintln!("⚠️ Title cannot be empty");
                        return;
                    }

                    if new_title != current_title {
                        if let Some(item) = items.iter_mut().find(|t| t.id == item_id) {
                            item.title = new_title.clone();
                            write_data(&data_path, items).expect("Failed to write file!");
                            println!("Updated task: {}", new_title);
                        }
                    }

                    let confirm_answer = Confirm::new("Do you want to edit the link?")
                        .with_default(false)
                        .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
                        .prompt();
                    if confirm_answer.is_ok_and(|x| x) {
                        Actions::edit_link(id);
                    }
                    return;
                }
                Err(e) => {
                    eprintln!("⚠️ Error editing task: {}", e);
                    return;
                }
            }
        }
        eprintln!("⚠️ No task found with id {id}");
    }

    pub fn work(ids: Vec<usize>) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        let mut did_change = false;
        ids.iter().for_each(|id| {
            if let Some(item) = items.iter_mut().find(|t| t.id == *id) {
                let title = item.title.clone();
                if item.status == Status::InProgress {
                    item.status = Status::Todo;
                    item.active_date = None;
                    did_change = true;
                    println!("Moved task back to todo: {}", title);
                } else {
                    item.status = Status::InProgress;
                    item.active_date = Some(Local::now().date_naive());
                    did_change = true;
                    println!("Started task: {}", title);
                }
            } else {
                eprintln!("⚠️ No task found with id {id}");
            }
        });
        if did_change {
            write_data(&data_path, items).expect("Failed to write file!");
        }
    }

    pub fn star(ids: Vec<usize>) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        let mut did_change = false;
        ids.iter().for_each(|id| {
            if let Some(item) = items.iter_mut().find(|t| t.id == *id) {
                if item.star.is_some_and(|v| v) {
                    item.star = None;
                } else {
                    item.star = Some(true);
                }
                did_change = true;
            } else {
                eprintln!("⚠️ No task found with id {id}");
            }
        });
        if did_change {
            write_data(&data_path, items).expect("Failed to write file!");
            println!("Starred the selected tasks");
        } else {
            println!("No tasks selected");
        }
    }

    pub fn done(ids: Vec<usize>) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        let mut did_change = false;
        ids.iter().for_each(|id| {
            if let Some(item) = items.iter_mut().find(|t| t.id == *id) {
                let title = item.title.clone();
                if item.status == Status::Done {
                    item.status = Status::Todo;
                    item.active_date = None;
                    did_change = true;
                    println!("Moved task back to todo: {}", title);
                } else {
                    item.status = Status::Done;
                    item.active_date = Some(Local::now().date_naive());
                    did_change = true;
                    println!("Completed task: {}", title);
                }
            } else {
                eprintln!("⚠️ No task found with id {id}");
            }
        });
        if did_change {
            write_data(&data_path, items).expect("Failed to write file!");
        }
    }

    pub fn delete(ids: Vec<usize>) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        items.retain(|i| !ids.contains(&i.id));
        write_data(&data_path, items).expect("Failed to write file!");
    }

    pub fn copy(ids: Vec<usize>) {
        let mut clipboard = Clipboard::new().expect("Failed to access clipboard");
        let data_path = get_data_path();
        let items = read_data(&data_path);
        let text_lines: Vec<String> = items
            .iter()
            .filter_map(|i| {
                if ids.contains(&i.id) {
                    return Some(i.get_copy_text());
                }
                None
            })
            .collect();
        let text = text_lines.join("\n");
        clipboard
            .set_text(text)
            .expect("Failed to save text to clipboard");
        if ids.len() > 1 {
            println!("Copied text for selected tasks");
        } else if let Some(first_text) = text_lines.first() {
            println!("Copied text for task: {}", first_text);
        } else {
            println!("Copied text for selected tasks");
        }
    }

    pub fn clear() {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        let mut copies: Vec<TaskItem> = vec![];
        let mut next_id = get_next_id(&items);
        for item in items.iter_mut() {
            item.star = None;
            match item.status {
                Status::Done => item.status = Status::Archived,
                Status::InProgress => {
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
        items.extend(copies);
        write_data(&data_path, items).expect("Failed to write file!");
        println!("Archived completed tasks");
    }
}
