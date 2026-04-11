use crate::commands::{Command, LinkFormat, execute_command};
use crate::config::load_config;
use crate::copier::Copier;
use crate::printer::Printer;
use crate::prompter::Prompter;
use crate::storage::TaskStorage;
use crate::util::list;
use colored::Colorize;
use inquire::Select;

pub fn interactive(
    storage: &mut dyn TaskStorage,
    prompter: &dyn Prompter,
    copier: &mut dyn Copier,
    printer: &mut dyn Printer,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    loop {
        let active_board = storage.active_board();
        println!("{}", format!("board: {active_board}").dimmed());
        println!("{}", "-----------------------------------------".green());
        list(storage, printer, config.list_link_format)?;
        let commands = vec![
            "quit",
            "list",
            "add",
            "check",
            "begin",
            "star",
            "open",
            "copy",
            "copy-markdown",
            "copy-checked",
            "copy-after",
            "clear",
            "delete",
            "edit",
            "list-archived",
            "list-after",
            "boards",
        ];
        println!("{}", "-----------------------------------------".dimmed());
        let answer = Select::new("Select a command:", commands)
            .with_page_size(4)
            .prompt()
            .or_else(|err| match err {
                inquire::InquireError::OperationCanceled => Ok("list"),
                err => Err(err),
            });

        let answer = match answer {
            Ok(cmd) => cmd,
            Err(_) => break, // Handle ctrl-c by quitting
        };

        if answer == "quit" {
            break;
        }

        let command: Option<Command> = match answer {
            // Do nothing as the list will be printed when we loop.
            "list" => None,
            "list-archived" => Some(Command::Archived),
            "list-after" => Some(Command::ListAfterDate {
                date: None,
                format: LinkFormat::Adjacent,
            }),
            "clear" => Some(Command::Clear),
            "add" => Some(Command::Add {
                title: None,
                link: None,
            }),
            "copy" => Some(Command::Copy {
                ids: None,
                format: LinkFormat::Adjacent,
            }),
            "copy-markdown" => Some(Command::Copy {
                ids: None,
                format: crate::commands::LinkFormat::Markdown,
            }),
            "copy-checked" => Some(Command::CopyChecked {
                format: LinkFormat::Adjacent,
            }),
            "copy-after" => Some(Command::CopyAfterDate {
                date: None,
                format: LinkFormat::Adjacent,
            }),
            "edit" => Some(Command::Edit { id: None }),
            "open" => Some(Command::Open { id: None }),
            "check" => Some(Command::Check { ids: None }),
            "star" => Some(Command::Star { ids: None }),
            "delete" => Some(Command::Delete { ids: None }),
            "begin" => Some(Command::Begin { ids: None }),
            "boards" => Some(Command::Use { name: None }),
            _ => {
                println!("Unknown command");
                None
            }
        };
        if let Some(command) = command {
            execute_command(storage, prompter, copier, printer, command)?;
        }
    }
    Ok(())
}
