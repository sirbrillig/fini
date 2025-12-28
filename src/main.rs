use clap::{Parser, Subcommand};
use fini::commands::{Command, LinkFormat, execute_command};
use fini::config::load_config;
use fini::copier::ClipboardCopier;
use fini::indices::{get_id_for_index, get_ids_for_indices};
use fini::printer::StdoutPrinter;
use fini::prompter::InquirePrompter;
use fini::storage::{FileStorage, get_default_storage_path};
use std::fs::create_dir_all;

#[derive(Parser)]
#[command(
    name = "fini",
    version,
    about = "An interactive CLI task list with links"
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
    /// Open a task in your web browser (alias: o)
    #[command(alias = "o")]
    Open {
        /// The index of the task to open
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
    /// List all archived tasks
    Archived,
    /// Enter interactive mode (alias: i)
    #[command(alias = "i")]
    Interactive,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage_path = get_default_storage_path();
    create_dir_all(&storage_path)?;
    let mut storage = FileStorage::new(storage_path);
    let prompter = InquirePrompter {};
    let mut copier = ClipboardCopier {};
    let mut printer = StdoutPrinter {};
    let config = load_config()?;
    let cli = Cli::parse();
    let command = match cli.command {
        CliCommands::Add { title } => Command::Add {
            title: Some(title.join(" ")).filter(|x| !x.is_empty()),
            link: None,
        },
        CliCommands::Copy { indices } => {
            let ids = get_ids_for_indices(&storage, indices)?;
            Command::Copy {
                ids: Some(ids).filter(|x| !x.is_empty()),
                format: LinkFormat::Adjacent,
            }
        }
        CliCommands::CopyMarkdown { indices } => {
            let ids = get_ids_for_indices(&storage, indices)?;
            Command::Copy {
                ids: Some(ids).filter(|x| !x.is_empty()),
                format: LinkFormat::Markdown,
            }
        }
        CliCommands::CopyAfterDate { date } => Command::CopyAfterDate {
            date: Some(date),
            format: LinkFormat::Adjacent,
        },
        CliCommands::List => Command::List {
            format: config.list_link_format,
        },
        CliCommands::Archived => Command::Archived,
        CliCommands::Edit { index } => {
            let id = get_id_for_index(&storage, index)?;
            Command::Edit { id }
        }
        CliCommands::Open { index } => {
            let id = get_id_for_index(&storage, index)?;
            Command::Open { id }
        }
        CliCommands::Begin { indices } => {
            let ids = get_ids_for_indices(&storage, indices)?;
            Command::Begin {
                ids: Some(ids).filter(|x| !x.is_empty()),
            }
        }
        CliCommands::Star { indices } => {
            let ids = get_ids_for_indices(&storage, indices)?;
            Command::Star {
                ids: Some(ids).filter(|x| !x.is_empty()),
            }
        }
        CliCommands::Check { indices } => {
            let ids = get_ids_for_indices(&storage, indices)?;
            Command::Check {
                ids: Some(ids).filter(|x| !x.is_empty()),
            }
        }
        CliCommands::Delete { indices } => {
            let ids = get_ids_for_indices(&storage, indices)?;
            Command::Delete {
                ids: Some(ids).filter(|x| !x.is_empty()),
            }
        }
        CliCommands::Clear => Command::Clear,
        CliCommands::Interactive => Command::Interactive,
    };
    execute_command(&mut storage, &prompter, &mut copier, &mut printer, command)?;
    Ok(())
}
