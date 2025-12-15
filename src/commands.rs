use crate::copier::Copier;
use crate::interactive::interactive;
use crate::printer::Printer;
use crate::prompter::Prompter;
use crate::storage::TaskStorage;
use crate::task_item::{Status, TaskItemCopyable, TaskItemCopyableMarkdown};
use crate::util::{
    add, archive, archived, clear, copy, delete, done, edit_task, get_task_for_id,
    get_task_ids_after_date, get_tasks_for_ids, list, prompt_for_task_id, prompt_for_task_ids,
    star, tasks_as_markdown_by_date, work,
};
use inquire::DateSelect;

/// The way that links will be formatted by an action
#[derive(Clone, Copy, Debug)]
pub enum LinkFormat {
    /// Print the link after the task title
    Adjacent,
    /// Print the link as an OSC 8 hyperlink after the task title
    Hyperlink,
    /// Make the task title into a markdown link
    Markdown,
}

pub enum Command {
    /// Add a new task
    Add {
        /// The task title (will prompt if missing)
        title: Option<String>,
        /// The task link (will prompt if missing)
        link: Option<String>,
    },
    /// List all current tasks
    List {
        /// The format of the printed links
        format: LinkFormat,
    },
    /// Open a task link in the browser
    Open {
        /// The id of the task to open (will prompt if missing)
        id: Option<usize>,
    },
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
    /// Mark a task as archived
    Archive {
        /// The ids of the tasks to archive (will prompt if missing)
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
        /// The format of the copied links
        format: LinkFormat,
    },
    /// Copy all completed, begun, or archived tasks to the clipboard after the date (inclusive)
    CopyAfterDate {
        /// The date to start (will prompt if missing)
        date: Option<String>,
        /// The format of the copied links
        format: LinkFormat,
    },
    /// List all archived tasks
    Archived,
    /// Enter interactive mode
    Interactive,
}

pub fn execute_command(
    storage: &mut dyn TaskStorage,
    prompter: &dyn Prompter,
    copier: &mut dyn Copier,
    printer: &mut dyn Printer,
    command: Command,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Command::Add { title, link } => {
            let title = match title {
                Some(content) => content,
                None => prompter.text("Enter task title:")?,
            };
            if title.is_empty() {
                printer.print("The title of the task cannot be empty.");
                return Ok(());
            }
            let link = match link {
                Some(content) => Some(content),
                None => prompter
                    .text("(Optional) Enter link:")
                    .ok()
                    .filter(|l| !l.is_empty()),
            };
            add(storage, printer, title, link)?;
        }
        Command::Copy { ids, format } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids(storage, "Select tasks to copy")?,
            };
            copy(storage, copier, printer, &ids, |i| match format {
                LinkFormat::Adjacent => TaskItemCopyable(i).to_string(),
                LinkFormat::Hyperlink => i.to_string(),
                LinkFormat::Markdown => TaskItemCopyableMarkdown(i).to_string(),
            })?;
        }
        Command::CopyAfterDate { date, format } => {
            let date = match date {
                Some(content) => content,
                None => DateSelect::new("Select date to start copying begun and completed tasks:")
                    .prompt()?
                    .to_string(),
            };
            // FIXME: this needs to find tasks in the archive also
            let ids: Vec<usize> = get_task_ids_after_date(
                storage,
                &date,
                &[Status::Done, Status::InProgress, Status::Archived],
            )?;
            let tasks = get_tasks_for_ids(storage, &ids)?;
            let text = tasks_as_markdown_by_date(tasks, |i| match format {
                LinkFormat::Adjacent => TaskItemCopyable(i).to_string(),
                LinkFormat::Hyperlink => i.to_string(),
                LinkFormat::Markdown => TaskItemCopyableMarkdown(i).to_string(),
            });
            copier.copy(&text)?;
            printer
                .print(format!("Copied tasks as Markdown by date starting at {}", date).as_str());
        }
        Command::List { format } => list(storage, printer, format)?,
        Command::Archived => archived(storage, printer)?,
        Command::Open { id } => {
            let id = match id {
                Some(id) => id,
                None => prompt_for_task_id(storage, "Select task to open")?,
            };
            let task = get_task_for_id(storage, id)?;
            let Some(link) = task.link else {
                printer.print("That task does not have a link");
                return Ok(());
            };
            webbrowser::open(&link)?;
        }
        Command::Edit { id } => {
            let id = match id {
                Some(id) => id,
                None => prompt_for_task_id(storage, "Select task to edit")?,
            };
            edit_task(storage, printer, id)?;
        }
        Command::Begin { ids } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids(storage, "Select tasks to begin")?,
            };
            work(storage, printer, &ids)?;
        }
        Command::Star { ids } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids(storage, "Select tasks to star")?,
            };
            star(storage, printer, &ids)?;
        }
        Command::Archive { ids } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids(storage, "Select tasks to archive")?,
            };
            archive(storage, printer, &ids)?;
        }
        Command::Check { ids } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids(storage, "Select tasks to check")?,
            };
            done(storage, printer, &ids)?;
        }
        Command::Delete { ids } => {
            let ids = match ids {
                Some(ids) => ids,
                None => prompt_for_task_ids(storage, "Select tasks to delete")?,
            };
            // Print tasks that will be deleted
            let tasks_to_delete = get_tasks_for_ids(storage, &ids)?;
            if !tasks_to_delete.is_empty() {
                printer.print("Tasks to be deleted:");
                for task in &tasks_to_delete {
                    printer.print(format!("  - {}", task).as_str());
                }
                printer.print(""); // Blank line before confirmation
            }
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
                clear(storage, printer)?;
            }
        }
        Command::Interactive => {
            interactive(storage, prompter, copier, printer)?;
        }
    }
    Ok(())
}
