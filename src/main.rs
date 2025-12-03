use clap::{Parser, Subcommand};
use fini::commands::{execute_command, Command};
use fini::storage::FileStorage;
use fini::util::{get_data_path, get_id_for_index, get_ids_for_indices};

#[derive(Parser)]
#[command(
    name = "fini",
    version,
    about = "An interactive CLI todo list tool with links"
)]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,
}

#[derive(Subcommand)]
enum CliCommands {
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
    let storage = FileStorage::new(get_data_path());
    let cli = Cli::parse();
    match cli.command {
        CliCommands::Add { title } => {
            execute_command(&storage, Command::Add {
                title: Some(title.join(" ")).filter(|x| !x.is_empty())
            })?;
        }
        CliCommands::FilePath => execute_command(&storage, Command::FilePath)?,
        CliCommands::Copy { indices } => execute_command(&storage, Command::Copy {
            ids: Some(get_ids_for_indices(&storage, indices)?).filter(|x| !x.is_empty()),
        })?,
        CliCommands::CopyChecked => execute_command(&storage, Command::CopyChecked)?,
        CliCommands::CopyDate { date } => execute_command(&storage, Command::CopyDate { date: Some(date) })?,
        CliCommands::CopyArchived => execute_command(&storage, Command::CopyArchived)?,
        CliCommands::List => execute_command(&storage, Command::List)?,
        CliCommands::Archived => execute_command(&storage, Command::Archived)?,
        CliCommands::Edit { index } => execute_command(&storage, Command::Edit {
            id: get_id_for_index(&storage, index)?,
        })?,
        CliCommands::Begin { indices } => execute_command(&storage, Command::Begin {
            ids: Some(get_ids_for_indices(&storage, indices)?).filter(|x| !x.is_empty()),
        })?,
        CliCommands::Star { indices } => execute_command(&storage, Command::Star {
            ids: Some(get_ids_for_indices(&storage, indices)?).filter(|x| !x.is_empty()),
        })?,
        CliCommands::Check { indices } => execute_command(&storage, Command::Check {
            ids: Some(get_ids_for_indices(&storage, indices)?).filter(|x| !x.is_empty()),
        })?,
        CliCommands::Delete { indices } => execute_command(&storage, Command::Delete {
            ids: Some(get_ids_for_indices(&storage, indices)?).filter(|x| !x.is_empty()),
        })?,
        CliCommands::Clear => execute_command(&storage, Command::Clear)?,
        CliCommands::DeleteBefore { date } => {
            execute_command(&storage, Command::DeleteBefore { date: Some(date) })?
        }
        CliCommands::Interactive => fini::actions::interactive(&storage)?,
    }
    Ok(())
}
