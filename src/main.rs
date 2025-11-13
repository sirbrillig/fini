use clap::{Parser, Subcommand};
use fini::{get_task_id_by_index, get_task_ids_for_date, get_visible_items, Actions};
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
    /// Toggle a task as in-progress (aliases: w, b, begin)
    #[command(aliases=["w", "begin", "b"])]
    Work {
        /// The indices of the tasks to toggle
        indices: Vec<usize>,
    },
    /// Edit a task (alias: e)
    #[command(alias = "e")]
    Edit {
        /// The index of the task to edit
        index: usize,
    },
    /// Toggle a task as done (aliases: c, check)
    #[command(aliases=["check", "c"])]
    Done {
        /// The indices of the tasks to toggle
        indices: Vec<usize>,
    },
    /// Delete a task entirely (alias: d)
    #[command(alias = "d")]
    Delete {
        /// The index of the task to delete
        index: usize,
    },
    /// Archive done tasks (archives a copy of in-progress tasks)
    Clear,
    /// Copy tasks to the clipboard (aliases: y, yank)
    #[command(aliases = ["y", "yank"])]
    Copy {
        /// The indices of the tasks to copy
        indices: Vec<usize>,
    },
    /// Copy archived tasks to the clipboard by date
    Date {
        /// The date of the tasks to copy
        date: String,
    },
    /// List all archived tasks
    Archived,
    /// Delete archived tasks
    Cycle,
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
        Commands::Date { date } => {
            Actions::copy(get_task_ids_for_date(date));
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
        Commands::Work { indices } => {
            let visible = get_visible_items();
            let mut ids: Vec<usize> = vec![];
            indices.iter().for_each(|index| {
                if let Some(id) = get_task_id_by_index(*index, &visible) {
                    ids.push(id);
                }
            });
            Actions::work(ids);
        }
        Commands::Done { indices } => {
            let visible = get_visible_items();
            let mut ids: Vec<usize> = vec![];
            indices.iter().for_each(|index| {
                if let Some(id) = get_task_id_by_index(*index, &visible) {
                    ids.push(id);
                }
            });
            Actions::done(ids);
        }
        Commands::Delete { index } => {
            let visible = get_visible_items();
            if let Some(id) = get_task_id_by_index(index, &visible) {
                Actions::delete(id);
            } else {
                eprintln!("⚠️ No task found with index {index}");
            }
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
        Commands::Cycle => {
            let confirm_answer =
                Confirm::new("Are you sure you want to delete all archived tasks?")
                    .with_default(false)
                    .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
                    .prompt();
            if confirm_answer.is_ok_and(|x| x) {
                Actions::cycle();
            }
        }
        Commands::Interactive => Actions::interactive(),
    }
}
