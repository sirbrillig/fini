use chrono::{Local, NaiveDate};
use directories::BaseDirs;
use edit::edit;
use inquire::{Confirm, Select, Text};
use serde::{Deserialize, Serialize};
use std::{fmt, fs, path::PathBuf};

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

impl fmt::Display for TodoItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.get_status(), self.title)
    }
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

fn sort_visible_items(items: &[TodoItem]) -> Vec<&TodoItem> {
    items
        .iter()
        .filter(|i| matches!(i.status, Status::Todo | Status::InProgress | Status::Done))
        .collect()
}

pub fn get_visible_items() -> Vec<TodoItem> {
    let data_path = get_data_path();
    let items = read_data(&data_path);
    items
        .into_iter()
        .filter(|i| matches!(i.status, Status::Todo | Status::InProgress | Status::Done))
        .collect()
}

pub fn get_task_id_by_index(index: usize, visible: Vec<TodoItem>) -> Option<usize> {
    visible.get(index - 1).map(|i| i.id)
}

fn get_next_id(items: &[TodoItem]) -> usize {
    items.iter().map(|t| t.id).max().unwrap_or(0) + 1
}

pub struct Actions {}

impl Actions {
    pub fn interactive() {
        loop {
            Actions::list();
            let commands = vec![
                "quit", "list", "add", "check", "begin", "clear", "delete", "cycle", "archived",
            ];
            let answer = Select::new("Select a command:", commands)
                .prompt()
                .unwrap_or("");
            match answer {
                "quit" => break,
                "list" => Actions::list(),
                "archived" => Actions::archived(),
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
                "cycle" => {
                    let confirm_answer =
                        Confirm::new("Are you sure you want to delete all archived tasks?")
                            .with_default(false)
                            .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
                            .prompt();
                    if confirm_answer.is_ok_and(|x| x) {
                        Actions::cycle();
                    }
                }
                "add" => {
                    let title = Text::new("Enter task:").prompt();
                    let link = Text::new("(Optional) Enter link:").prompt();
                    if let (Ok(title), Ok(link)) = (title, link) {
                        if link.is_empty() {
                            Actions::add(title);
                        } else {
                            Actions::add_with_link(title, link);
                        }
                        continue;
                    }
                    println!("An error happened when asking for the task.");
                }
                "check" => {
                    let data_path = get_data_path();
                    let items = read_data(&data_path);
                    let visible = sort_visible_items(&items);
                    if let Ok(selection) = Select::new("Select task to complete", visible).prompt()
                    {
                        Actions::done(selection.id);
                    }
                }
                "delete" => {
                    let data_path = get_data_path();
                    let items = read_data(&data_path);
                    let visible = sort_visible_items(&items);
                    if let Ok(selection) = Select::new("Select task to delete", visible).prompt() {
                        Actions::delete(selection.id);
                    }
                }
                "begin" => {
                    let data_path = get_data_path();
                    let items = read_data(&data_path);
                    let visible = sort_visible_items(&items);
                    if let Ok(selection) = Select::new("Select task to start", visible).prompt() {
                        Actions::work(selection.id);
                    }
                }
                _ => println!("Unknown command"),
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

    pub fn add_with_link(title: String, link: String) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        let id = get_next_id(&items);
        let item = TodoItem {
            id,
            title: title.clone(),
            link: Some(link),
            status: Status::Todo,
            active_date: None,
        };
        items.push(item);
        write_data(&data_path, items);
        println!("Added task: {}", title);
    }

    pub fn link(id: usize, link: String) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        if let Some(item) = items.iter_mut().find(|t| t.id == id) {
            item.link = Some(link);
            println!("Added link to task: {}", item.title);
        }
        write_data(&data_path, items);
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
        for item in items {
            if item.status == Status::Archived {
                item.print_with_date();
            }
        }
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
        eprintln!("⚠️ No task found with id {id}");
    }

    pub fn work(id: usize) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        if let Some(item) = items.iter_mut().find(|t| t.id == id) {
            let title = item.title.clone();
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
        eprintln!("⚠️ No task found with id {id}");
    }

    pub fn done(id: usize) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        if let Some(item) = items.iter_mut().find(|t| t.id == id) {
            let title = item.title.clone();
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
        eprintln!("⚠️ No task found with id {id}");
    }

    pub fn delete(id: usize) {
        let data_path = get_data_path();
        let mut items = read_data(&data_path);
        if let Some(item) = items.iter_mut().find(|t| t.id == id) {
            let title = item.title.clone();
            items.retain(|i| i.id != id);
            write_data(&data_path, items);
            println!("Deleted task: {}", title);
            return;
        }
        eprintln!("⚠️ No task found with id {id}");
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
