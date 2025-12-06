use crate::actions::{
    add, archived, clear, copy, copy_archived, copy_as_markdown_list_by_date,
    copy_with_markdown_links, delete, done, edit_task, list, star, work,
};
use crate::copier::Copier;
use crate::prompter::Prompter;
use crate::storage::TaskStorage;
use crate::task_item::Status;
use crate::util::{
    get_all_items, get_data_path, get_task_ids_after_date, get_task_ids_before_date,
    get_task_ids_for_date, prompt_for_task_id, prompt_for_task_ids,
};

pub enum Command {
    /// Add a new task
    Add {
        /// The task title (will prompt if missing)
        title: Option<String>,
        /// The task link (will prompt if missing)
        link: Option<String>,
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
        /// The id of the task to edit (will prompt if missing)
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
    /// Copy tasks to the clipboard with markdown links
    CopyMarkdown {
        /// The ids of the tasks to copy (will prompt if missing)
        ids: Option<Vec<usize>>,
    },
    /// Copy checked tasks to the clipboard
    CopyChecked,
    /// Copy archived tasks to the clipboard
    CopyArchived,
    /// Copy all completed, begun, or archived tasks to the clipboard after the date (inclusive)
    CopyAfterDate {
        /// The date to start (will prompt if missing)
        date: Option<String>,
    },
    /// Copy archived tasks to the clipboard by date
    CopyDate {
        /// The date of the tasks to copy (will prompt if missing)
        date: Option<String>,
    },
    /// Print the file path where the data is kept
    FilePath,
    /// List all archived tasks
    Archived,
    /// Delete archived tasks before date
    DeleteBefore {
        /// The date before which to delete tasks (will prompt if missing)
        date: Option<String>,
    },
}

pub fn execute_command(
    storage: &mut dyn TaskStorage,
    prompter: &dyn Prompter,
    copier: &mut dyn Copier,
    command: Command,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Command::Add { title, link } => {
            let title = match title {
                Some(content) => content,
                None => prompter.text("Enter task title:")?,
            };
            if title.is_empty() {
                println!("The title of the task cannot be empty.");
                return Ok(());
            }
            let link = match link {
                Some(content) => Some(content),
                None => prompter
                    .text("(Optional) Enter link:")
                    .ok()
                    .filter(|l| !l.is_empty()),
            };
            add(storage, title, link)?;
        }
        Command::FilePath => {
            if let Some(path) = get_data_path().to_str() {
                println!("{}", path);
            }
        }
        Command::Copy { ids } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids(storage, "Select tasks to copy")?,
            };
            copy(storage, copier, &ids)?;
        }
        Command::CopyMarkdown { ids } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids(storage, "Select tasks to copy")?,
            };
            copy_with_markdown_links(storage, copier, &ids)?;
        }
        Command::CopyChecked => {
            let ids: Vec<usize> = get_all_items(storage)?
                .iter()
                .filter(|i| matches!(i.status, Status::Done | Status::InProgress))
                .map(|i| i.id)
                .collect();
            copy(storage, copier, &ids)?;
        }
        Command::CopyAfterDate { date } => {
            let date = match date {
                Some(content) => content,
                None => prompter.text("Enter date (YYYY-MM-DD):")?,
            };
            let ids: Vec<usize> = get_task_ids_after_date(
                storage,
                date,
                &[Status::Done, Status::InProgress, Status::Archived],
            )?;
            copy_as_markdown_list_by_date(storage, copier, &ids)?;
        }
        Command::CopyDate { date } => {
            let date = match date {
                Some(content) => content,
                None => prompter.text("Enter date (YYYY-MM-DD):")?,
            };
            copy(storage, copier, &get_task_ids_for_date(storage, date)?)?;
        }
        Command::CopyArchived => copy_archived(storage, copier)?,
        Command::List => list(storage)?,
        Command::Archived => archived(storage)?,
        Command::Edit { id } => {
            let id = match id {
                Some(id) => id,
                None => prompt_for_task_id(storage, "Select task to edit")?,
            };
            edit_task(storage, id)?;
        }
        Command::Begin { ids } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids(storage, "Select tasks to begin")?,
            };
            work(storage, &ids)?;
        }
        Command::Star { ids } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids(storage, "Select tasks to star")?,
            };
            star(storage, &ids)?;
        }
        Command::Check { ids } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids(storage, "Select tasks to check")?,
            };
            done(storage, &ids)?;
        }
        Command::Delete { ids } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids(storage, "Select tasks to delete")?,
            };
            // TODO: print tasks that will be deleted
            let confirm_answer =
                prompter.confirm("Are you sure you want to delete the selected tasks?");
            if confirm_answer.is_ok_and(|x| x) {
                delete(storage, &ids)?;
            }
        }
        Command::Clear => {
            let confirm_answer =
                prompter.confirm("Are you sure you want to archive all complete tasks?");
            if confirm_answer.is_ok_and(|x| x) {
                clear(storage)?;
            }
        }
        Command::DeleteBefore { date } => {
            let date = match date {
                Some(content) => content,
                None => prompter.text("Enter date (YYYY-MM-DD):")?,
            };
            let confirm_answer = prompter.confirm(&format!(
                "Are you sure you want to delete all archived tasks before {}?",
                date
            ));
            if confirm_answer.is_ok_and(|x| x) {
                let tasks = &get_task_ids_before_date(storage, date)?;
                delete(storage, tasks)?;
            }
        }
    }
    Ok(())
}
