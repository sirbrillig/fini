use crate::task_item::TaskItem;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

pub trait TaskStorage {
    fn read(&self) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>>;
    fn write(&self, tasks: Vec<TaskItem>) -> Result<(), Box<dyn std::error::Error>>;
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
    fn read(&self) -> Result<Vec<TaskItem>, Box<dyn std::error::Error>> {
        if let Ok(data) = std::fs::read_to_string(&self.path) {
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Vec::new())
        }
    }

    fn write(&self, tasks: Vec<TaskItem>) -> Result<(), Box<dyn std::error::Error>> {
        let dir = self.path.parent().unwrap_or_else(|| Path::new("."));
        let mut tmp = NamedTempFile::new_in(dir)?;
        serde_json::to_writer_pretty(&mut tmp, &tasks)?;
        tmp.as_file_mut().flush()?;
        tmp.as_file().sync_all()?;
        tmp.persist(&self.path)?;
        Ok(())
    }
}
