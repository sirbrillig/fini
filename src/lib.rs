use chrono::{Local, NaiveDate};
use directories::BaseDirs;
use edit::edit;
use inquire::Select;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct TodoItem {
    id: usize,
    title: String,
    status: Status,
    active_date: Option<NaiveDate>,
    link: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Status {
    Todo,
    InProgress,
    Done,
    Archived,
}

impl TodoItem {
    pub fn print_with_date(&self) {
        if let Some(date) = &self.active_date {
            println!(
                "{} {} {}",
                self.get_status(),
                date.format("%Y-%m-%d"),
                self.title
            );
        } else {
            println!("{} {}", self.get_status(), self.title);
        }
        self.print_link();
    }

    pub fn print_with_index(&self, index: usize) {
        println!("{:>3}. {} {}", index, self.get_status(), &self.title);
        self.print_link();
    }

    fn print_link(&self) {
        if let Some(link) = &&self.link {
            println!("     🔗 {link}");
        }
    }

    fn get_status(&self) -> &str {
        match &self.status {
            Status::Todo => "☐",
            Status::InProgress => "…",
            Status::Done => "✔",
            Status::Archived => "✔",
        }
    }
}

fn read_data(path: &PathBuf) -> Vec<TodoItem> {
    if let Ok(data) = fs::read_to_string(path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn write_data(path: &PathBuf, data: Vec<TodoItem>) {
    // TODO: write this in a way that is extra safe in case the write fails
    let json = serde_json::to_string_pretty(&data).unwrap();
    fs::write(path, json).expect("Failed to write data file");
}

fn get_data_path() -> PathBuf {
    let data_dir = BaseDirs::new()
        .map(|b| b.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    data_dir.join("fini_data.json")
}

fn get_visible_items(items: &[TodoItem]) -> Vec<&TodoItem> {
    items
        .iter()
        .filter(|i| matches!(i.status, Status::Todo | Status::InProgress | Status::Done))
        .collect()
}

fn get_task_by_index<'a>(index: usize, visible: &[&'a TodoItem]) -> Option<&'a TodoItem> {
    visible.get(index - 1).copied()
}

fn get_next_id(items: &[TodoItem]) -> usize {
    items.iter().map(|t| t.id).max().unwrap_or(0) + 1
}

pub struct Actions {}

impl Actions {
    pub fn interactive() {
        loop {
            let commands = vec!["quit", "list"];
            let answer = Select::new("Select a command:", commands)
                .prompt()
                .expect("Failed to get user input");
            match answer {
                "quit" => break,
                "list" => Actions::list(),
                _ => eprintln!("Unknown command"),
            }
        }
    }

    pub fn add(title: String) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        let id = get_next_id(&items);
        let item = TodoItem {
            id,
            title: title.clone(),
            link: None,
            status: Status::Todo,
            active_date: None,
        };
        items.push(item);
        write_data(&data_path, items);
        println!("Added task: {}", title);
    }

    pub fn list() {
        let data_path = get_data_path();
        let items = read_data(&data_path);
        let visible = get_visible_items(&items);
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
        for item in items {
            if item.status == Status::Archived {
                item.print_with_date();
            }
        }
    }

    pub fn edit(index: usize) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        let visible = get_visible_items(&items);
        if let Some(item) = get_task_by_index(index, &visible) {
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

                    if new_title == current_title {
                        println!("No changes made");
                        return;
                    }

                    // Update the task
                    if let Some(item) = items.iter_mut().find(|t| t.id == item_id) {
                        item.title = new_title.clone();
                        write_data(&data_path, items);
                        println!("Updated task: {}", new_title);
                        return;
                    }
                }
                Err(e) => {
                    eprintln!("⚠️ Error editing task: {}", e);
                    return;
                }
            }
        }
        eprintln!("⚠️ No task found with index {index}");
    }

    pub fn work(index: usize) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        let visible = get_visible_items(&items);
        if let Some(item) = get_task_by_index(index, &visible) {
            let item_id = item.id;
            let title = item.title.clone();
            if let Some(item) = items.iter_mut().find(|t| t.id == item_id) {
                if item.status == Status::InProgress {
                    item.status = Status::Todo;
                    item.active_date = None;
                    write_data(&data_path, items);
                    println!("Moved task back to todo: {}", title);
                } else {
                    item.status = Status::InProgress;
                    item.active_date = Some(Local::now().date_naive());
                    write_data(&data_path, items);
                    println!("Started task: {}", title);
                }
                return;
            }
        }
        eprintln!("⚠️ No task found with index {index}");
    }

    pub fn done(index: usize) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        let visible = get_visible_items(&items);
        if let Some(item) = get_task_by_index(index, &visible) {
            let item_id = item.id;
            let title = item.title.clone();
            if let Some(item) = items.iter_mut().find(|t| t.id == item_id) {
                if item.status == Status::Done {
                    item.status = Status::Todo;
                    item.active_date = None;
                    write_data(&data_path, items);
                    println!("Moved task back to todo: {}", title);
                } else {
                    item.status = Status::Done;
                    item.active_date = Some(Local::now().date_naive());
                    write_data(&data_path, items);
                    println!("Completed task: {}", title);
                }
                return;
            }
        }
        eprintln!("⚠️ No task found with index {index}");
    }

    pub fn delete(index: usize) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        let visible = get_visible_items(&items);
        if let Some(item) = get_task_by_index(index, &visible) {
            let item_id = item.id;
            let title = item.title.clone();
            items.retain(|i| i.id != item_id);
            write_data(&data_path, items);
            println!("Deleted task: {}", title);
            return;
        }
        eprintln!("⚠️ No task found with index {index}");
    }

    pub fn clear() {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        let mut copies: Vec<TodoItem> = vec![];
        let mut next_id = get_next_id(&items);
        for item in items.iter_mut() {
            match item.status {
                Status::Done => item.status = Status::Archived,
                Status::InProgress => {
                    let copy = TodoItem {
                        id: next_id,
                        title: item.title.clone(),
                        link: item.link.clone(),
                        status: Status::Archived,
                        active_date: item.active_date,
                    };
                    next_id += 1;
                    copies.push(copy);
                }
                _ => {}
            }
        }
        items.extend(copies);
        write_data(&data_path, items);
        println!("Archived completed tasks");
    }

    pub fn cycle() {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        items.retain(|i| i.status != Status::Archived);
        write_data(&data_path, items);
        println!("Deleted archived tasks");
    }
}
