use clap::{Parser, Subcommand};
use fini::{
    get_archived_task_ids, get_data_path, get_task_id_by_index, get_task_ids_before_date, get_task_ids_for_date, get_visible_items, Actions, Status
};
use inquire::{Confirm, Text};

#[derive(Parser)]
#[command(
    name = "fini",
    version,
    about = "An interactive CLI todo list tool with links"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task (alias: a)
    #[command(alias = "a")]
    Add {
        /// The task title
        title: Vec<String>,
    },
    /// List all current tasks (alias: l)
    #[command(alias = "l")]
    List,
    /// Toggle a task as in-progress (aliases: b)
    #[command(aliases=["b"])]
    Begin {
        /// The indices of the tasks to toggle
        indices: Vec<usize>,
    },
    /// Star or un-star a task (alias: s)
    #[command(aliases=["s"])]
    Star {
        /// The indices of the tasks to toggle
        indices: Vec<usize>,
    },
    /// Edit a task (alias: e)
    #[command(alias = "e")]
    Edit {
        /// The index of the task to edit
        index: usize,
    },
    /// Toggle a task as done (aliases: c)
    #[command(aliases=["c"])]
    Check {
        /// The indices of the tasks to toggle
        indices: Vec<usize>,
    },
    /// Delete tasks entirely (alias: d)
    #[command(alias = "d")]
    Delete {
        /// The indices of the tasks to delete
        indices: Vec<usize>,
    },
    /// Archive done tasks (archives a copy of in-progress tasks)
    Clear,
    /// Copy tasks to the clipboard (aliases: y, yank)
    #[command(aliases = ["y", "yank"])]
    Copy {
        /// The indices of the tasks to copy
        indices: Vec<usize>,
    },
    /// Copy checked tasks to the clipboard
    CopyChecked,
    /// Copy archived tasks to the clipboard
    CopyArchived,
    /// Copy archived tasks to the clipboard by date
    CopyDate {
        /// The date of the tasks to copy
        date: String,
    },
    /// Print the file path where the data is kept
    FilePath,
    /// List all archived tasks
    Archived,
    /// Delete archived tasks before date
    DeleteBefore {
        /// The date before which to delete tasks
        date: String,
    },
    /// Enter interactive mode (alias: i)
    #[command(alias = "i")]
    Interactive,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Add { title } => {
            let title_joined = title.join(" ");
            if title_joined.is_empty() {
                return;
            }
            let id = Actions::add(title_joined);
            let link = Text::new("(Optional) Enter link:").prompt();
            if let Ok(link) = link {
                Actions::link(id, link);
            }
        }
        Commands::FilePath => {
            if let Some(path) = get_data_path().to_str() {
                println!("{}", path);
            }
        }
        Commands::Copy { indices } => {
            let visible = get_visible_items();
            let mut ids: Vec<usize> = vec![];
            indices.iter().for_each(|index| {
                if let Some(id) = get_task_id_by_index(*index, &visible) {
                    ids.push(id);
                }
            });
            Actions::copy(ids);
        }
        Commands::CopyChecked => {
            let visible = get_visible_items();
            let mut ids: Vec<usize> = vec![];
            visible
                .iter()
                .filter(|i| matches!(i.status, Status::InProgress | Status::Done))
                .for_each(|task| {
                    ids.push(task.id);
                });
            Actions::copy(ids);
        }
        Commands::CopyDate { date } => {
            Actions::copy(get_task_ids_for_date(date));
        }
        Commands::CopyArchived => {
            Actions::copy(get_archived_task_ids());
        }
        Commands::List => Actions::list(),
        Commands::Archived => Actions::archived(),
        Commands::Edit { index } => {
            let visible = get_visible_items();
            if let Some(id) = get_task_id_by_index(index, &visible) {
                Actions::edit(id);
            } else {
                eprintln!("⚠️ No task found with index {index}");
            }
        }
        Commands::Begin { indices } => {
            let visible = get_visible_items();
            let mut ids: Vec<usize> = vec![];
            indices.iter().for_each(|index| {
                if let Some(id) = get_task_id_by_index(*index, &visible) {
                    ids.push(id);
                }
            });
            Actions::work(ids);
        }
        Commands::Star { indices } => {
            let visible = get_visible_items();
            let mut ids: Vec<usize> = vec![];
            indices.iter().for_each(|index| {
                if let Some(id) = get_task_id_by_index(*index, &visible) {
                    ids.push(id);
                }
            });
            Actions::star(ids);
        }
        Commands::Check { indices } => {
            let visible = get_visible_items();
            let mut ids: Vec<usize> = vec![];
            indices.iter().for_each(|index| {
                if let Some(id) = get_task_id_by_index(*index, &visible) {
                    ids.push(id);
                }
            });
            Actions::done(ids);
        }
        Commands::Delete { indices } => {
            let visible = get_visible_items();
            let mut ids: Vec<usize> = vec![];
            indices.iter().for_each(|index| {
                if let Some(id) = get_task_id_by_index(*index, &visible) {
                    ids.push(id);
                }
            });
            Actions::delete(ids);
        }
        Commands::Clear => {
            let confirm_answer =
                Confirm::new("Are you sure you want to archive all complete tasks?")
                    .with_default(false)
                    .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
                    .prompt();
            if confirm_answer.is_ok_and(|x| x) {
                Actions::clear();
            }
        }
        Commands::DeleteBefore { date } => {
            let confirm_answer =
                Confirm::new("Are you sure you want to delete all archived tasks before {date}?")
                    .with_default(false)
                    .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
                    .prompt();
            if confirm_answer.is_ok_and(|x| x) {
                Actions::delete(get_task_ids_before_date(date));
            }
        }
        Commands::Interactive => Actions::interactive(),
    }
}
