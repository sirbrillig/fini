use crate::commands::{Command, execute_command};
use crate::copier::Copier;
use crate::prompter::Prompter;
use crate::storage::TaskStorage;
use crate::util::list;
use colored::Colorize;
use inquire::Select;

pub fn interactive(
    storage: &mut dyn TaskStorage,
    prompter: &dyn Prompter,
    copier: &mut dyn Copier,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("{}", "-----------------------------------------".green());
        list(storage)?;
        let commands = vec![
            "quit",
            "list",
            "add",
            "check",
            "begin",
            "star",
            "copy",
            "copy-markdown",
            "copy-after",
            "clear",
            "delete",
            "delete-before",
            "edit",
            "list-archived",
        ];
        println!("{}", "-----------------------------------------".dimmed());
        let answer = Select::new("Select a command:", commands)
            .with_page_size(4)
            .prompt();

        let answer = match answer {
            Ok(cmd) => cmd,
            Err(_) => break, // Handle ctrl-c by quitting
        };

        match answer {
            "quit" => break,
            "list" => {
                // Do nothing as the list will be printed when we loop.
            }
            "list-archived" => execute_command(storage, prompter, copier, Command::Archived)?,
            "clear" => execute_command(storage, prompter, copier, Command::Clear)?,
            "add" => execute_command(
                storage,
                prompter,
                copier,
                Command::Add {
                    title: None,
                    link: None,
                },
            )?,
            "copy" => execute_command(
                storage,
                prompter,
                copier,
                Command::Copy {
                    ids: None,
                    format: crate::commands::LinkFormat::Adjacent,
                },
            )?,
            "copy-markdown" => execute_command(
                storage,
                prompter,
                copier,
                Command::Copy {
                    ids: None,
                    format: crate::commands::LinkFormat::Markdown,
                },
            )?,
            "copy-after" => execute_command(
                storage,
                prompter,
                copier,
                Command::CopyAfterDate {
                    date: None,
                    format: crate::commands::LinkFormat::Adjacent,
                },
            )?,
            "edit" => execute_command(storage, prompter, copier, Command::Edit { id: None })?,
            "check" => execute_command(storage, prompter, copier, Command::Check { ids: None })?,
            "star" => execute_command(storage, prompter, copier, Command::Star { ids: None })?,
            "delete-before" => execute_command(
                storage,
                prompter,
                copier,
                Command::DeleteBefore { date: None },
            )?,
            "delete" => execute_command(storage, prompter, copier, Command::Delete { ids: None })?,
            "begin" => execute_command(storage, prompter, copier, Command::Begin { ids: None })?,
            _ => println!("Unknown command"),
        }
    }
    Ok(())
}
