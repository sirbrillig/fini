use crate::copier::{Copier, CopyPayload, HtmlPayload, TextPayload};
use crate::interactive::interactive;
use crate::markdown::archived_tasks_as_markdown;
use crate::printer::Printer;
use crate::prompter::Prompter;
use crate::storage::TaskStorage;
use crate::task_item::{Status, TaskItemRichText};
use crate::util::{
    add, archive, archived, clear, copy, delete, done, edit_task, get_checked_task_ids,
    get_task_for_id, get_task_ids_after_date, get_task_ids_between, get_task_link_for_format,
    get_tasks_for_ids, list, prompt_for_task_id, prompt_for_task_ids, star, work,
};
use inquire::{DateSelect, Select};
use serde::Deserialize;

/// The way that links will be formatted by an action
#[derive(Clone, Copy, Debug, Deserialize, clap::ValueEnum)]
pub enum LinkFormat {
    /// Do not print the link
    None,
    /// Print the link after the task title
    Adjacent,
    /// Print the link after the task title with colors
    AdjacentColor,
    /// Print the link as an OSC 8 hyperlink after the task title
    Hyperlink,
    /// Make the task title into a markdown link
    Markdown,
    /// Print the link after the task title on a new line
    Newline,
    /// Make the task title into an HTML link
    RichText,
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
    /// Copy all completed or begun tasks to the clipboard
    CopyChecked {
        /// The format of the copied links
        format: LinkFormat,
    },
    /// Copy all completed, begun, or archived tasks to the clipboard starting on the date
    CopyAfterDate {
        /// The date to start (will prompt if missing)
        date: Option<String>,
        /// The format of the copied links
        format: LinkFormat,
    },
    /// List all completed, begun, or archived tasks between dates, inclusive
    ListBetween {
        /// The date to start (will prompt if missing)
        date_a: Option<String>,
        /// The date to end (will prompt if missing)
        date_b: Option<String>,
        /// The format of the copied links
        format: LinkFormat,
    },
    /// List all completed, begun, or archived tasks starting on the date
    ListAfterDate {
        /// The date to start (will prompt if missing)
        date: Option<String>,
        /// The format of the copied links
        format: LinkFormat,
    },
    /// List all archived tasks
    Archived,
    /// Enter interactive mode
    Interactive,
    /// List all available boards
    Boards,
    /// Print the filesystem path to the current active board
    BoardPath,
    /// Switch to (or create) a board; prompts interactively when name is None
    Use {
        /// The board name (will prompt if missing)
        name: Option<String>,
    },
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
            match format {
                LinkFormat::RichText =>  
                    copy(storage, copier, printer, &ids, |i| {
                        HtmlPayload { html: TaskItemRichText(i).to_string(), alt: get_task_link_for_format(i, LinkFormat::Adjacent) }
                    })?,
                _ => copy(storage, copier, printer, &ids, |i| {
                    TextPayload(get_task_link_for_format(i, format))
                })?,
            };
        }
        Command::CopyChecked { format } => {
            let ids = get_checked_task_ids(storage)?;
            match format {
                LinkFormat::RichText =>  
                    copy(storage, copier, printer, &ids, |i| {
                        HtmlPayload { html: TaskItemRichText(i).to_string(), alt: get_task_link_for_format(i, LinkFormat::Adjacent) }
                    })?,
                _ => copy(storage, copier, printer, &ids, |i| {
                    TextPayload(get_task_link_for_format(i, format))
                })?,
            };
        }
        Command::CopyAfterDate { date, format } => {
            let date = match date {
                Some(content) => content,
                None => DateSelect::new("Select date to start copying begun and completed tasks:")
                    .prompt()?
                    .to_string(),
            };
            let mut tasks = storage.read_tasks()?;
            let mut archived = storage.read_archived()?;
            tasks.append(&mut archived);
            let ids: Vec<usize> = get_task_ids_after_date(
                &tasks,
                &date,
                &[Status::Done, Status::InProgress, Status::Archived],
            )?;
            let tasks = get_tasks_for_ids(tasks, &ids)?;
            let text = archived_tasks_as_markdown(tasks, |i| get_task_link_for_format(i, format));
            let payload = CopyPayload::Text(TextPayload(text));
            copier.copy(&payload)?;
            printer
                .print(format!("Copied tasks as Markdown by date starting at {}", date).as_str());
            }
        Command::ListBetween {
            date_a,
            date_b,
            format,
        } => {
            let date_a = match date_a {
                Some(content) => content,
                None => DateSelect::new("Select first date to show begun and completed tasks:")
                    .prompt()?
                    .to_string(),
            };
            let date_b = match date_b {
                Some(content) => content,
                None => DateSelect::new("Select last date to show begun and completed tasks:")
                    .prompt()?
                    .to_string(),
            };
            let mut tasks = storage.read_tasks()?;
            let mut archived = storage.read_archived()?;
            tasks.append(&mut archived);
            let ids: Vec<usize> = get_task_ids_between(
                &tasks,
                &date_a,
                &date_b,
                &[Status::Done, Status::InProgress, Status::Archived],
            )?;
            let tasks = get_tasks_for_ids(tasks, &ids)?;
            let text = archived_tasks_as_markdown(tasks, |i| get_task_link_for_format(i, format));
            printer.print(&text);
        }
        Command::ListAfterDate { date, format } => {
            let date = match date {
                Some(content) => content,
                None => {
                    DateSelect::new("Select first date to start listing begun and completed tasks:")
                        .prompt()?
                        .to_string()
                }
            };
            let mut tasks = storage.read_tasks()?;
            let mut archived = storage.read_archived()?;
            tasks.append(&mut archived);
            let ids: Vec<usize> = get_task_ids_after_date(
                &tasks,
                &date,
                &[Status::Done, Status::InProgress, Status::Archived],
            )?;
            let tasks = get_tasks_for_ids(tasks, &ids)?;
            let text = archived_tasks_as_markdown(tasks, |i| get_task_link_for_format(i, format));
            printer.print(&text);
        }
        Command::List { format } => list(storage, printer, format)?,
        Command::Archived => archived(storage, printer)?,
        Command::Open { id } => {
            let id_option = match id {
                Some(id) => Some(id),
                None => prompt_for_task_id(storage, "Select task to open")?,
            };
            let Some(id) = id_option else {
                printer.print("No task selected");
                return Ok(());
            };
            let task = get_task_for_id(storage, id)?;
            let Some(link) = task.link else {
                printer.print("That task does not have a link");
                return Ok(());
            };
            webbrowser::open(&link)?;
        }
        Command::Edit { id } => {
            let id_option = match id {
                Some(id) => Some(id),
                None => prompt_for_task_id(storage, "Select task to edit")?,
            };
            let Some(id) = id_option else {
                printer.print("No task selected");
                return Ok(());
            };
            edit_task(storage, prompter, printer, id)?;
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
            let tasks = storage.read_tasks()?;
            let tasks_to_delete = get_tasks_for_ids(tasks, &ids)?;
            if tasks_to_delete.is_empty() {
                return Ok(());
            }
            printer.print("Tasks to be deleted:");
            for task in &tasks_to_delete {
                printer.print(format!("  - {}", task).as_str());
            }
            printer.print(""); // Blank line before confirmation
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
        Command::Boards => {
            let active = storage.active_board();
            for board in storage.list_boards()? {
                if board == active {
                    printer.print(&format!("* {board}"));
                } else {
                    printer.print(&format!("  {board}"));
                }
            }
        }
        Command::BoardPath => match storage.active_board_path() {
            None => printer.print("No path available"),
            Some(board_path) => printer.print(&board_path.display().to_string()),
        },
        Command::Use { name } => {
            let name = match name {
                Some(n) => {
                    let n = n.trim().to_string();
                    if n.is_empty() || n.contains('/') || n.contains('\\') {
                        printer.print(&format!("Invalid board name: '{n}'"));
                        return Ok(());
                    }
                    n
                }
                None => {
                    let mut boards = storage.list_boards()?;
                    boards.push("+ create new board".to_string());

                    let Ok(selection) = Select::new("Select a board:", boards).prompt() else {
                        return Ok(());
                    };

                    if selection == "+ create new board" {
                        let new_name = match prompter.text("Board name:") {
                            Ok(n) => n.trim().to_string(),
                            Err(_) => return Ok(()),
                        };
                        if new_name.is_empty() || new_name.contains('/') || new_name.contains('\\')
                        {
                            printer.print("Invalid board name.");
                            return Ok(());
                        }
                        new_name
                    } else {
                        selection
                    }
                }
            };

            if name != storage.active_board() {
                storage.switch_board(&name)?;
                printer.print(&format!("Switched to board '{name}'."));
            }
        }
    }
    Ok(())
}
