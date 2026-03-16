use crate::markdown::{
    archived_tasks_as_markdown, parse_markdown_archive_with_start_id, parse_markdown_tasks,
    tasks_as_markdown,
};
use crate::task_item::{TaskItem, TaskItemCopyableMarkdown, TaskItemFiniMarkdown};
use chrono::Datelike;
use directories::BaseDirs;
use glob::glob;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Get the path to the main data directory for fini
pub fn get_default_data_dir() -> PathBuf {
    let data_dir = BaseDirs::new()
        .map(|b| b.data_dir().to_path_buf())
        .unwrap_or(PathBuf::from("."));
    data_dir.join("fini")
}

#[derive(Eq, Hash, PartialEq)]
struct YearMonth {
    year: i32,
    month: u32,
}

impl YearMonth {
    pub fn get_archive_filename(&self) -> String {
        format!("archived-{:04}-{:02}.md", self.year, self.month)
    }
}

pub trait TaskStorage {
    fn read_tasks(&self) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>>;
    fn read_archived(&self) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>>;
    fn write_tasks(&mut self, tasks: Vec<TaskItem>) -> Result<(), Box<dyn std::error::Error>>;
    fn write_archived(&mut self, tasks: Vec<TaskItem>) -> Result<(), Box<dyn std::error::Error>>;

    fn active_board(&self) -> String;
    fn list_boards(&self) -> Result<Vec<String>, Box<dyn std::error::Error>>;
    fn switch_board(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct FileStorage {
    data_dir: PathBuf,
    board: String,
}

impl FileStorage {
    pub fn new(data_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let board = Self::read_active_board(&data_dir);
        fs::create_dir_all(data_dir.join(&board))?;
        Ok(Self { data_dir, board })
    }

    fn read_active_board(data_dir: &Path) -> String {
        let name = fs::read_to_string(data_dir.join("active_list"))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "tasks".to_string());
        if data_dir.join(&name).exists() {
            name
        } else {
            "tasks".to_string()
        }
    }

    fn board_path(&self) -> PathBuf {
        self.data_dir.join(&self.board)
    }

    fn find_all_archive_files(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let archived_file_glob = self.board_path().join("archived-*.md");
        let archived_file_pattern = Regex::new(r"^archived-\d{4}-\d{2}\.md$").unwrap();
        let paths: Vec<PathBuf> = glob(archived_file_glob.to_str().unwrap())?
            .filter_map(|p| p.ok())
            .filter(|p| {
                let file_name = p.file_name();
                let Some(file_name) = file_name else {
                    return false;
                };
                let Some(file_name) = file_name.to_str() else {
                    return false;
                };
                archived_file_pattern.is_match(file_name)
            })
            .collect();
        Ok(paths)
    }

    fn group_tasks_by_month(&self, tasks: Vec<TaskItem>) -> HashMap<YearMonth, Vec<TaskItem>> {
        // Note: i32 is the default type of date.year() and u32 is date.month()
        let mut grouped: HashMap<YearMonth, Vec<TaskItem>> = HashMap::new();
        for task in tasks {
            if let Some(date) = task.active_date {
                let key = YearMonth {
                    year: date.year(),
                    month: date.month(),
                };
                grouped.entry(key).or_default().push(task);
            } else {
                eprintln!(
                    "⚠️ Warning: archived task without active_date: {}",
                    task.title
                );
            }
        }
        grouped
    }
}

impl TaskStorage for FileStorage {
    fn read_tasks(&self) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
        let file_path = self.board_path().join("tasks.md");
        let Ok(data) = std::fs::read_to_string(file_path) else {
            return Ok(Vec::new());
        };
        let tasks = parse_markdown_tasks(&data, 1)?;
        Ok(tasks)
    }

    fn write_tasks(&mut self, tasks: Vec<TaskItem>) -> Result<(), Box<dyn std::error::Error>> {
        let board_path = self.board_path();
        let path = board_path.join("tasks.md");
        let mut tmp = NamedTempFile::new_in(&board_path)?;
        let markdown = tasks_as_markdown(tasks, |t| TaskItemFiniMarkdown(t).to_string());
        fs::write(&tmp, markdown)?;
        tmp.as_file_mut().flush()?;
        tmp.as_file().sync_all()?;
        tmp.persist(path)?;
        Ok(())
    }

    fn read_archived(&self) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
        let archive_files = self.find_all_archive_files()?;
        let mut all_tasks: Vec<TaskItem> = Vec::new();
        let mut next_id = 100000;
        for file_path in archive_files {
            let data = fs::read_to_string(&file_path)?;
            let mut tasks = parse_markdown_archive_with_start_id(&data, next_id)?;
            // Update next_id to be after the last ID used in this file
            if let Some(last_task) = tasks.last() {
                next_id = last_task.id + 1;
            }
            all_tasks.append(&mut tasks);
        }
        Ok(all_tasks)
    }

    fn write_archived(&mut self, tasks: Vec<TaskItem>) -> Result<(), Box<dyn std::error::Error>> {
        let board_path = self.board_path();
        let grouped = self.group_tasks_by_month(tasks);
        // Write each month's tasks to its own file
        for (key, month_tasks) in grouped {
            let filename = key.get_archive_filename();
            let path = board_path.join(&filename);
            let markdown = archived_tasks_as_markdown(month_tasks, |t| {
                TaskItemCopyableMarkdown(t).to_string()
            });
            let mut tmp = NamedTempFile::new_in(&board_path)?;
            fs::write(&tmp, markdown)?;
            tmp.as_file_mut().flush()?;
            tmp.as_file().sync_all()?;
            tmp.persist(&path)?;
        }
        Ok(())
    }

    fn active_board(&self) -> String {
        self.board.clone()
    }

    fn list_boards(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        if !self.data_dir.exists() {
            return Ok(vec!["tasks".to_string()]);
        }
        let mut boards: Vec<String> = fs::read_dir(&self.data_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();
        boards.sort();
        if boards.is_empty() {
            boards.push("tasks".to_string());
        }
        Ok(boards)
    }

    fn switch_board(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(self.data_dir.join(name))?;
        fs::write(self.data_dir.join("active_list"), name)?;
        self.board = name.to_string();
        Ok(())
    }
}

pub struct InMemoryStorage {
    boards: HashMap<String, (String, String)>,
    active: String,
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryStorage {
    pub fn new() -> Self {
        let mut boards = HashMap::new();
        boards.insert("tasks".to_string(), (String::new(), String::new()));
        Self {
            boards,
            active: "tasks".to_string(),
        }
    }
}

impl TaskStorage for InMemoryStorage {
    fn read_tasks(&self) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
        let tasks_md = self.boards
            .get(&self.active)
            .map(|(t, _)| t.as_str())
            .unwrap_or("");
        parse_markdown_tasks(tasks_md, 1)
    }

    fn write_tasks(&mut self, tasks: Vec<TaskItem>) -> Result<(), Box<dyn std::error::Error>> {
        let md = tasks_as_markdown(tasks, |t| TaskItemFiniMarkdown(t).to_string());
        self.boards.entry(self.active.clone()).or_default().0 = md;
        Ok(())
    }

    fn read_archived(&self) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
        let archived_md = self.boards
            .get(&self.active)
            .map(|(_, a)| a.as_str())
            .unwrap_or("");
        parse_markdown_archive_with_start_id(archived_md, 100000)
    }

    fn write_archived(&mut self, tasks: Vec<TaskItem>) -> Result<(), Box<dyn std::error::Error>> {
        let md = archived_tasks_as_markdown(tasks, |t| TaskItemCopyableMarkdown(t).to_string());
        self.boards.entry(self.active.clone()).or_default().1 = md;
        Ok(())
    }

    fn active_board(&self) -> String {
        self.active.clone()
    }

    fn list_boards(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut boards: Vec<String> = self.boards.keys().cloned().collect();
        boards.sort();
        Ok(boards)
    }

    fn switch_board(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.boards.entry(name.to_string()).or_default();
        self.active = name.to_string();
        Ok(())
    }
}
