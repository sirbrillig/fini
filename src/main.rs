use clap::{Parser, Subcommand};
use fini::commands::{execute_command, Command};
use fini::util::{get_id_for_index, get_ids_for_indices};

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
    let cli = Cli::parse();
    match cli.command {
        CliCommands::Add { title } => {
            execute_command(Command::Add {
                // TODO: pass None if empty string
                title: Some(title.join(" ")),
            })?;
        }
        CliCommands::FilePath => execute_command(Command::FilePath)?,
        CliCommands::Copy { indices } => execute_command(Command::Copy {
            // TODO: pass None if empty list
            ids: Some(get_ids_for_indices(indices)),
        })?,
        CliCommands::CopyChecked => execute_command(Command::CopyChecked)?,
        CliCommands::CopyDate { date } => execute_command(Command::CopyDate { date: Some(date) })?,
        CliCommands::CopyArchived => execute_command(Command::CopyArchived)?,
        CliCommands::List => execute_command(Command::List)?,
        CliCommands::Archived => execute_command(Command::Archived)?,
        CliCommands::Edit { index } => execute_command(Command::Edit {
            id: get_id_for_index(index),
        })?,
        CliCommands::Begin { indices } => execute_command(Command::Begin {
            // TODO: pass None if empty list
            ids: Some(get_ids_for_indices(indices)),
        })?,
        CliCommands::Star { indices } => execute_command(Command::Star {
            // TODO: pass None if empty list
            ids: Some(get_ids_for_indices(indices)),
        })?,
        CliCommands::Check { indices } => execute_command(Command::Check {
            // TODO: pass None if empty list
            ids: Some(get_ids_for_indices(indices)),
        })?,
        CliCommands::Delete { indices } => execute_command(Command::Delete {
            // TODO: pass None if empty list
            ids: Some(get_ids_for_indices(indices)),
        })?,
        CliCommands::Clear => execute_command(Command::Clear)?,
        CliCommands::DeleteBefore { date } => {
            execute_command(Command::DeleteBefore { date: Some(date) })?
        }
        CliCommands::Interactive => fini::actions::interactive()?,
    }
    Ok(())
}
