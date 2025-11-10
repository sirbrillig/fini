use clap::{Parser, Subcommand};
use fini::{get_task_id_by_index, get_visible_items, Actions};
use inquire::Text;

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
    /// Add a new task
    Add {
        /// The task title
        title: Vec<String>,
    },
    /// List all current tasks
    List,
    /// Toggle a task as in-progress
    Work {
        /// The index of the task to toggle
        index: usize,
    },
    /// Edit a task
    Edit {
        /// The index of the task to edit
        index: usize,
    },
    /// Toggle a task as done
    Done {
        /// The index of the task to toggle
        index: usize,
    },
    /// Delete a task entirely
    Delete {
        /// The index of the task to delete
        index: usize,
    },
    /// Archive done tasks (archives a copy of in-progress tasks)
    Clear,
    /// Copy tasks to the clipboard
    Copy {
        /// The indices of the tasks to copy
        indices: Vec<usize>,
    },
    /// List all archived tasks
    Archive,
    /// Delete archived tasks
    Cycle,
    /// Enter interactive mode
    Interactive,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Add { title } => {
            let id = Actions::add(title.join(" "));
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
        Commands::List => Actions::list(),
        Commands::Archive => Actions::archived(),
        Commands::Edit { index } => {
            let visible = get_visible_items();
            if let Some(id) = get_task_id_by_index(index, &visible) {
                Actions::edit(id);
            } else {
                eprintln!("⚠️ No task found with index {index}");
            }
        }
        Commands::Work { index } => {
            let visible = get_visible_items();
            if let Some(id) = get_task_id_by_index(index, &visible) {
                Actions::work(id);
            } else {
                eprintln!("⚠️ No task found with index {index}");
            }
        }
        Commands::Done { index } => {
            let visible = get_visible_items();
            if let Some(id) = get_task_id_by_index(index, &visible) {
                Actions::done(id);
            } else {
                eprintln!("⚠️ No task found with index {index}");
            }
        }
        Commands::Delete { index } => {
            let visible = get_visible_items();
            if let Some(id) = get_task_id_by_index(index, &visible) {
                Actions::delete(id);
            } else {
                eprintln!("⚠️ No task found with index {index}");
            }
        }
        Commands::Clear => Actions::clear(),
        Commands::Cycle => Actions::cycle(),
        Commands::Interactive => Actions::interactive(),
    }
}
