use crate::markdown::parse_markdown_archive;
use crate::task_item::{TaskItem, TaskItemCopyableMarkdown};
use crate::util::tasks_as_markdown_by_date;
use directories::BaseDirs;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

pub fn get_default_storage_path() -> PathBuf {
    let data_dir = BaseDirs::new()
        .map(|b| b.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    data_dir.join("fini")
}

pub trait TaskStorage {
    fn read_tasks(&self) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>>;
    fn read_archived(&self) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>>;
    fn write_tasks(&mut self, tasks: Vec<TaskItem>) -> Result<(), Box<dyn std::error::Error>>;
    fn write_archived(&mut self, tasks: Vec<TaskItem>) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct FileStorage {
    path: PathBuf,
}

impl FileStorage {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl TaskStorage for FileStorage {
    fn read_tasks(&self) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
        let path = &self.path.join("fini_data.json");
        if let Ok(data) = std::fs::read_to_string(path) {
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Vec::new())
        }
    }

    fn write_tasks(&mut self, tasks: Vec<TaskItem>) -> Result<(), Box<dyn std::error::Error>> {
        let path = &self.path.join("fini_data.json");
        let dir = &self.path;
        let mut tmp = NamedTempFile::new_in(dir)?;
        serde_json::to_writer_pretty(&mut tmp, &tasks)?;
        tmp.as_file_mut().flush()?;
        tmp.as_file().sync_all()?;
        tmp.persist(path)?;
        Ok(())
    }

    fn read_archived(&self) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
        // TODO: implement
        Ok(Vec::new())
    }

    fn write_archived(&mut self, tasks: Vec<TaskItem>) -> Result<(), Box<dyn std::error::Error>> {
        let markdown =
            tasks_as_markdown_by_date(tasks, |t| TaskItemCopyableMarkdown(t).to_string());
        let dir = &self.path;
        let path = &self.path.join("archived.md");
        let mut tmp = NamedTempFile::new_in(dir)?;
        fs::write(path, markdown)?;
        tmp.as_file_mut().flush()?;
        tmp.as_file().sync_all()?;
        tmp.persist(path)?;
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
