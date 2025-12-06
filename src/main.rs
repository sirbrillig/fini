use clap::{Parser, Subcommand};
use fini::commands::{Command, execute_command};
use fini::copier::ClipboardCopier;
use fini::prompter::InquirePrompter;
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
    /// Copy tasks to the clipboard as markdown links
    CopyMarkdown {
        /// The indices of the tasks to copy
        indices: Vec<usize>,
    },
    /// Copy all completed, begun, or archived tasks to the clipboard after the date (inclusive)
    CopyAfterDate {
        /// The date to start (will prompt if missing)
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
    let mut storage = FileStorage::new(get_data_path());
    let prompter = InquirePrompter {};
    let mut copier = ClipboardCopier {};
    let cli = Cli::parse();
    match cli.command {
        CliCommands::Add { title } => {
            execute_command(
                &mut storage,
                &prompter,
                &mut copier,
                Command::Add {
                    title: Some(title.join(" ")).filter(|x| !x.is_empty()),
                    link: None,
                },
            )?;
        }
        CliCommands::FilePath => {
            execute_command(&mut storage, &prompter, &mut copier, Command::FilePath)?
        }
        CliCommands::Copy { indices } => {
            let ids = get_ids_for_indices(&storage, indices)?;
            execute_command(
                &mut storage,
                &prompter,
                &mut copier,
                Command::Copy {
                    ids: Some(ids).filter(|x| !x.is_empty()),
                },
            )?;
        }
        CliCommands::CopyMarkdown { indices } => {
            let ids = get_ids_for_indices(&storage, indices)?;
            execute_command(
                &mut storage,
                &prompter,
                &mut copier,
                Command::CopyMarkdown {
                    ids: Some(ids).filter(|x| !x.is_empty()),
                },
            )?;
        }
        CliCommands::CopyAfterDate { date } => execute_command(
            &mut storage,
            &prompter,
            &mut copier,
            Command::CopyAfterDate { date: Some(date) },
        )?,
        CliCommands::List => execute_command(&mut storage, &prompter, &mut copier, Command::List)?,
        CliCommands::Archived => {
            execute_command(&mut storage, &prompter, &mut copier, Command::Archived)?
        }
        CliCommands::Edit { index } => {
            let id = get_id_for_index(&storage, index)?;
            execute_command(&mut storage, &prompter, &mut copier, Command::Edit { id })?;
        }
        CliCommands::Begin { indices } => {
            let ids = get_ids_for_indices(&storage, indices)?;
            execute_command(
                &mut storage,
                &prompter,
                &mut copier,
                Command::Begin {
                    ids: Some(ids).filter(|x| !x.is_empty()),
                },
            )?;
        }
        CliCommands::Star { indices } => {
            let ids = get_ids_for_indices(&storage, indices)?;
            execute_command(
                &mut storage,
                &prompter,
                &mut copier,
                Command::Star {
                    ids: Some(ids).filter(|x| !x.is_empty()),
                },
            )?;
        }
        CliCommands::Check { indices } => {
            let ids = get_ids_for_indices(&storage, indices)?;
            execute_command(
                &mut storage,
                &prompter,
                &mut copier,
                Command::Check {
                    ids: Some(ids).filter(|x| !x.is_empty()),
                },
            )?;
        }
        CliCommands::Delete { indices } => {
            let ids = get_ids_for_indices(&storage, indices)?;
            execute_command(
                &mut storage,
                &prompter,
                &mut copier,
                Command::Delete {
                    ids: Some(ids).filter(|x| !x.is_empty()),
                },
            )?;
        }
        CliCommands::Clear => {
            execute_command(&mut storage, &prompter, &mut copier, Command::Clear)?
        }
        CliCommands::DeleteBefore { date } => execute_command(
            &mut storage,
            &prompter,
            &mut copier,
            Command::DeleteBefore { date: Some(date) },
        )?,
        CliCommands::Interactive => {
            fini::actions::interactive(&mut storage, &prompter, &mut copier)?
        }
    }
    Ok(())
}
