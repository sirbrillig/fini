use clap::{Parser, Subcommand};
use fini::Actions;

#[derive(Parser)]
#[command(name = "fini", version, about = "A CLI todo list tool with links")]
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
        // TODO: figure out a good UX for adding a link
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
    /// List all archived tasks
    Archive,
    /// Delete archived tasks
    Cycle,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Add { title } => {
            Actions::add(title.join(" "));
        }
        Commands::List => {
            Actions::list();
        }
        Commands::Archive => {
            Actions::archived();
        }
        Commands::Edit { index } => {
            Actions::edit(index);
        }
        Commands::Work { index } => {
            Actions::work(index);
        }
        Commands::Done { index } => {
            Actions::done(index);
        }
        Commands::Delete { index } => {
            Actions::delete(index);
        }
        Commands::Clear => {
            Actions::clear();
        }
        Commands::Cycle => {
            Actions::cycle();
        }
    }
}
