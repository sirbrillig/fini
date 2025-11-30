use clap::{Parser, Subcommand};
use fini::actions;
use fini::task_item::Status;
use fini::util::{
    get_data_path, get_ids_for_indices, get_task_id_by_index, get_task_ids_before_date,
    get_task_ids_for_date, get_visible_items,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Add { title } => {
            let title_joined = title.join(" ");
            if title_joined.is_empty() {
                return Ok(());
            }
            let id = actions::add(title_joined)?;
            let link = Text::new("(Optional) Enter link:").prompt();
            if let Ok(link) = link {
                actions::link(id, link)?;
            }
        }
        Commands::FilePath => {
            if let Some(path) = get_data_path().to_str() {
                println!("{}", path);
            }
        }
        Commands::Copy { indices } => {
            actions::copy(get_ids_for_indices(indices))?;
        }
        Commands::CopyChecked => {
            let visible = get_visible_items();
            let ids = visible
                .iter()
                .filter(|i| matches!(i.status, Status::Done | Status::InProgress))
                .map(|i| i.id)
                .collect();
            actions::copy(ids)?;
        }
        Commands::CopyDate { date } => {
            actions::copy(get_task_ids_for_date(date))?;
        }
        Commands::CopyArchived => {
            actions::copy_archived()?;
        }
        Commands::List => actions::list(),
        Commands::Archived => actions::archived(),
        Commands::Edit { index } => {
            let visible = get_visible_items();
            if let Some(id) = get_task_id_by_index(index, &visible) {
                actions::edit_task(id)?;
            } else {
                eprintln!("⚠️ No task found with index {index}");
            }
        }
        Commands::Begin { indices } => {
            actions::work(get_ids_for_indices(indices))?;
        }
        Commands::Star { indices } => {
            actions::star(get_ids_for_indices(indices))?;
        }
        Commands::Check { indices } => {
            actions::done(get_ids_for_indices(indices))?;
        }
        Commands::Delete { indices } => {
            actions::delete(get_ids_for_indices(indices))?;
        }
        Commands::Clear => {
            let confirm_answer =
                Confirm::new("Are you sure you want to archive all complete tasks?")
                    .with_default(false)
                    .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
                    .prompt();
            if confirm_answer.is_ok_and(|x| x) {
                actions::clear()?;
            }
        }
        Commands::DeleteBefore { date } => {
            let confirm_answer = Confirm::new(&format!(
                "Are you sure you want to delete all archived tasks before {}?",
                date
            ))
            .with_default(false)
            .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
            .prompt();
            if confirm_answer.is_ok_and(|x| x) {
                actions::delete(get_task_ids_before_date(date))?;
            }
        }
        Commands::Interactive => actions::interactive()?,
    }
    Ok(())
}
