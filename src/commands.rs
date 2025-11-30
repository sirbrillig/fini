use crate::actions::{
    add, archived, clear, copy, copy_archived, delete, done, edit_task, list, star, work,
};
use crate::task_item::Status;
use crate::util::{
    get_all_items, get_data_path, get_task_ids_before_date, get_task_ids_for_date,
    prompt_for_task_id, prompt_for_task_ids,
};
use inquire::{Confirm, Text};

pub enum Command {
    /// Add a new task
    Add {
        /// The task title (will prompt if missing)
        title: Option<String>,
    },
    /// List all current tasks
    List,
    /// Toggle a task as in-progress
    Begin {
        /// The ids of the tasks to toggle (will prompt if missing)
        ids: Option<Vec<usize>>,
    },
    /// Star or un-star a task
    Star {
        /// The ids of the tasks to toggle (will prompt if missing)
        ids: Option<Vec<usize>>,
    },
    /// Edit a task
    Edit {
        /// The id of the task to edit (will prompty if missing)
        id: Option<usize>,
    },
    /// Toggle a task as done
    Check {
        /// The ids of the tasks to toggle (will prompt if missing)
        ids: Option<Vec<usize>>,
    },
    /// Delete tasks entirely
    Delete {
        /// The ids of the tasks to delete (will prompt if missing)
        ids: Option<Vec<usize>>,
    },
    /// Archive done tasks (archives a copy of in-progress tasks)
    Clear,
    /// Copy tasks to the clipboard
    Copy {
        /// The ids of the tasks to copy (will prompt if missing)
        ids: Option<Vec<usize>>,
    },
    /// Copy checked tasks to the clipboard
    CopyChecked,
    /// Copy archived tasks to the clipboard
    CopyArchived,
    /// Copy archived tasks to the clipboard by date
    CopyDate {
        /// The date of the tasks to copy (will prompty if missing)
        date: Option<String>,
    },
    /// Print the file path where the data is kept
    FilePath,
    /// List all archived tasks
    Archived,
    /// Delete archived tasks before date
    DeleteBefore {
        /// The date before which to delete tasks (will prompty if missing)
        date: Option<String>,
    },
}

pub fn execute_command(command: Command) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Command::Add { title } => {
            let title = match title {
                Some(content) => content,
                None => Text::new("Enter task title:").prompt()?,
            };
            if title.is_empty() {
                println!("The title of the task cannot be empty.");
                return Ok(());
            }
            let link_input = Text::new("(Optional) Enter link:").prompt();
            add(title, link_input.ok())?;
        }
        Command::FilePath => {
            if let Some(path) = get_data_path().to_str() {
                println!("{}", path);
            }
        }
        Command::Copy { ids } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids("Select tasks to copy")?,
            };
            copy(ids)?;
        }
        Command::CopyChecked => {
            let ids = get_all_items()
                .iter()
                .filter(|i| matches!(i.status, Status::Done | Status::InProgress))
                .map(|i| i.id)
                .collect();
            copy(ids)?;
        }
        Command::CopyDate { date } => {
            let date = match date {
                Some(content) => content,
                None => Text::new("Enter date (YYYY-MM-DD):").prompt()?,
            };
            copy(get_task_ids_for_date(date))?;
        }
        Command::CopyArchived => copy_archived()?,
        Command::List => list(),
        Command::Archived => archived(),
        Command::Edit { id } => {
            let id = match id {
                Some(id) => id,
                None => prompt_for_task_id("Select task to edit")?,
            };
            edit_task(id)?;
        }
        Command::Begin { ids } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids("Select tasks to begin")?,
            };
            work(ids)?;
        }
        Command::Star { ids } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids("Select tasks to star")?,
            };
            star(ids)?;
        }
        Command::Check { ids } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids("Select tasks to check")?,
            };
            done(ids)?;
        }
        Command::Delete { ids } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids("Select tasks to delete")?,
            };
            let confirm_answer =
                Confirm::new("Are you sure you want to delete the selected tasks?")
                    .with_default(false)
                    .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
                    .prompt();
            if confirm_answer.is_ok_and(|x| x) {
                delete(ids)?;
            }
        }
        Command::Clear => {
            let confirm_answer =
                Confirm::new("Are you sure you want to archive all complete tasks?")
                    .with_default(false)
                    .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
                    .prompt();
            if confirm_answer.is_ok_and(|x| x) {
                clear()?;
            }
        }
        Command::DeleteBefore { date } => {
            let date = match date {
                Some(content) => content,
                None => Text::new("Enter date (YYYY-MM-DD):").prompt()?,
            };
            let confirm_answer = Confirm::new(&format!(
                "Are you sure you want to delete all archived tasks before {}?",
                date
            ))
            .with_default(false)
            .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
            .prompt();
            if confirm_answer.is_ok_and(|x| x) {
                delete(get_task_ids_before_date(date))?;
            }
        }
    }
    Ok(())
}
