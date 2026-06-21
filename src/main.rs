use clap::{Parser, Subcommand};
use directories::BaseDirs;
use fini::commands::{Command, LinkFormat, execute_command};
use fini::config::{PrompterType, load_config};
use fini::copier::ClipboardCopier;
use fini::indices::{get_id_for_index, get_ids_for_indices};
use fini::printer::StdoutPrinter;
use fini::prompter::{InquirePrompter, Prompter, VimPrompter};
use fini::storage::{FileStorage, get_default_data_dir};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "fini",
    version,
    about = "An interactive CLI task list with links"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<CliCommands>,
}

#[derive(Subcommand)]
enum CliCommands {
    /// Add a new task (alias: a)
    #[command(alias = "a")]
    Add {
        /// The task title
        title: Vec<String>,
        /// The optional task link
        #[arg(long)]
        link: Option<String>,
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
    /// Archive specific tasks
    Archive {
        /// The indices of the tasks to archive
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
    /// Copy completed or begun tasks to the clipboard as markdown links
    CopyChecked,
    /// Copy all completed, begun, or archived tasks to the clipboard after the date (inclusive)
    CopyAfterDate {
        /// The date to start (will prompt if missing)
        date: String,
    },
    /// List all completed, begun, or archived tasks after the date (inclusive)
    ListAfterDate {
        /// The date to start (will prompt if missing)
        date: String,
    },
    /// List all archived tasks
    Archived,
    /// Enter interactive mode (alias: i)
    #[command(alias = "i")]
    Interactive,
    /// Switch to (or create) a board
    Use {
        /// The name of the board
        name: String,
    },
    /// List all available boards
    Boards,
    /// Print the filesystem path to the current active board
    BoardPath,
}

fn expand_tilde(path: String) -> PathBuf {
    if let Some(suffix) = path.strip_prefix("~/") {
        if let Some(home) = BaseDirs::new().map(|b| b.home_dir().to_path_buf()) {
            return home.join(suffix);
        }
    } else if path == "~" {
        if let Some(home) = BaseDirs::new().map(|b| b.home_dir().to_path_buf()) {
            return home;
        }
    }
    PathBuf::from(path)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config = load_config()?;
    let data_dir = match config.data_dir {
        Some(dir) => expand_tilde(dir),
        None => get_default_data_dir(),
    };
    fs::create_dir_all(&data_dir)?;
    let mut storage = FileStorage::new(data_dir)?;
    let prompter: Box<dyn Prompter> = match config.prompter {
        PrompterType::Vim => Box::new(VimPrompter::new()?),
        PrompterType::Inquire => Box::new(InquirePrompter),
    };
    let mut copier = ClipboardCopier {};
    let mut printer = StdoutPrinter {};
    let requested_command = match cli.command {
        Some(requested_command) => requested_command,
        // Default to interactive.
        None => CliCommands::Interactive,
    };
    let command = match requested_command {
        CliCommands::Add { title, link } => Command::Add {
            title: Some(title.join(" ")).filter(|x| !x.is_empty()),
            link,
        },
        CliCommands::Copy { indices } => {
            let ids = get_ids_for_indices(&storage, indices)?;
            Command::Copy {
                ids: Some(ids).filter(|x| !x.is_empty()),
                format: LinkFormat::Adjacent,
            }
        }
        CliCommands::CopyChecked => Command::CopyChecked {
            format: LinkFormat::Markdown,
        },
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
        CliCommands::ListAfterDate { date } => Command::ListAfterDate {
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
        CliCommands::Archive { indices } => {
            let ids = get_ids_for_indices(&storage, indices)?;
            Command::Archive {
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
        CliCommands::Use { name } => Command::Use { name: Some(name) },
        CliCommands::Boards => Command::Boards,
        CliCommands::BoardPath => Command::BoardPath,
    };
    execute_command(
        &mut storage,
        prompter.as_ref(),
        &mut copier,
        &mut printer,
        command,
    )?;
    Ok(())
}
