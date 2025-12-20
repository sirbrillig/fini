use crate::markdown::parse_markdown_archive;
use crate::task_item::{TaskItem, TaskItemCopyableMarkdown};
use crate::util::tasks_as_markdown_by_date;
use chrono::Datelike;
use directories::BaseDirs;
use glob::glob;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

pub fn get_default_storage_path() -> PathBuf {
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
}

pub struct FileStorage {
    path: PathBuf,
    data_filename: String,
}

impl FileStorage {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            data_filename: "fini_data.json".to_string(),
        }
    }

    fn find_all_archive_files(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let archived_file_glob = self.path.join("archived-*.md");
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
                eprintln!("Warning: archived task without active_date: {}", task.title);
            }
        }
        grouped
    }
}

impl TaskStorage for FileStorage {
    fn read_tasks(&self) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
        let path = &self.path.join(&self.data_filename);
        let Ok(data) = std::fs::read_to_string(path) else {
            return Ok(Vec::new());
        };
        Ok(serde_json::from_str(&data)?)
    }

    fn write_tasks(&mut self, tasks: Vec<TaskItem>) -> Result<(), Box<dyn std::error::Error>> {
        let path = &self.path.join(&self.data_filename);
        let dir = &self.path;
        let mut tmp = NamedTempFile::new_in(dir)?;
        serde_json::to_writer_pretty(&mut tmp, &tasks)?;
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
            let mut tasks = crate::markdown::parse_markdown_archive_with_start_id(&data, next_id)?;
            // Update next_id to be after the last ID used in this file
            if let Some(last_task) = tasks.last() {
                next_id = last_task.id + 1;
            }
            all_tasks.append(&mut tasks);
        }
        Ok(all_tasks)
    }

    fn write_archived(&mut self, tasks: Vec<TaskItem>) -> Result<(), Box<dyn std::error::Error>> {
        let grouped = self.group_tasks_by_month(tasks);
        // Write each month's tasks to its own file
        for (key, month_tasks) in grouped {
            let filename = key.get_archive_filename();
            let path = self.path.join(&filename);
            let markdown =
                tasks_as_markdown_by_date(month_tasks, |t| TaskItemCopyableMarkdown(t).to_string());
            let mut tmp = NamedTempFile::new_in(&self.path)?;
            fs::write(&tmp, markdown)?;
            tmp.as_file_mut().flush()?;
            tmp.as_file().sync_all()?;
            tmp.persist(&path)?;
        }
        Ok(())
    }
}

pub struct InMemoryStorage {
    tasks: Vec<TaskItem>,
    archived: String,
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            archived: "".to_string(),
        }
    }
}

impl TaskStorage for InMemoryStorage {
    fn read_tasks(&self) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
        Ok(self.tasks.clone())
    }

    fn write_tasks(&mut self, tasks: Vec<TaskItem>) -> Result<(), Box<dyn std::error::Error>> {
        self.tasks = tasks;
        Ok(())
    }

    fn read_archived(&self) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
        parse_markdown_archive(&self.archived)
    }

    fn write_archived(&mut self, tasks: Vec<TaskItem>) -> Result<(), Box<dyn std::error::Error>> {
        self.archived =
            tasks_as_markdown_by_date(tasks, |t| TaskItemCopyableMarkdown(t).to_string());
        Ok(())
    }
}
