use crate::commands::{execute_command, Command};
use crate::prompter::Prompter;
use crate::storage::TaskStorage;
use crate::task_item::{Status, TaskItem, TaskItemCopyable, TaskItemCopyableMarkdown};
use crate::util::{
    archived_tasks_as_markdown, get_archived_tasks, get_next_id, get_tasks_for_ids,
    sort_visible_items, tasks_as_markdown_by_date,
};
use arboard::Clipboard;
use chrono::Local;
use colored::Colorize;
use edit::edit;
use inquire::{Confirm, Select};

pub fn interactive(
    storage: &mut dyn TaskStorage,
    prompter: &dyn Prompter,
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
            "copy-archived",
            "copy-date",
            "copy-checked",
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
            "list-archived" => execute_command(storage, prompter, Command::Archived)?,
            "clear" => execute_command(storage, prompter, Command::Clear)?,
            "add" => execute_command(
                storage,
                prompter,
                Command::Add {
                    title: None,
                    link: None,
                },
            )?,
            "copy" => execute_command(storage, prompter, Command::Copy { ids: None })?,
            "copy-markdown" => {
                execute_command(storage, prompter, Command::CopyMarkdown { ids: None })?
            }
            "copy-checked" => execute_command(storage, prompter, Command::CopyChecked)?,
            "copy-archived" => execute_command(storage, prompter, Command::CopyArchived)?,
            "copy-date" => execute_command(storage, prompter, Command::CopyDate { date: None })?,
            "copy-after" => {
                execute_command(storage, prompter, Command::CopyAfterDate { date: None })?
            }
            "edit" => execute_command(storage, prompter, Command::Edit { id: None })?,
            "check" => execute_command(storage, prompter, Command::Check { ids: None })?,
            "star" => execute_command(storage, prompter, Command::Star { ids: None })?,
            "delete-before" => {
                execute_command(storage, prompter, Command::DeleteBefore { date: None })?
            }
            "delete" => execute_command(storage, prompter, Command::Delete { ids: None })?,
            "begin" => execute_command(storage, prompter, Command::Begin { ids: None })?,
            _ => println!("Unknown command"),
        }
    }
    Ok(())
}

pub fn add(
    storage: &mut dyn TaskStorage,
    title: String,
    link: Option<String>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
    let id = get_next_id(&tasks);
    let item = TaskItem {
        id,
        title,
        link,
        ..Default::default()
    };
    println!("Added task: {}", &item.title);
    tasks.push(item);
    storage.write(tasks)?;
    Ok(id)
}

pub fn link(
    storage: &mut dyn TaskStorage,
    id: usize,
    link: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
    let Some(item) = tasks.iter_mut().find(|t| t.id == id) else {
        eprintln!("⚠️ No task found with id {id}");
        return Ok(());
    };
    item.link = Some(link);
    println!("Added link to task: {}", item.title);
    storage.write(tasks)?;
    Ok(())
}

pub fn list(storage: &dyn TaskStorage) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = storage.read()?;
    let visible = sort_visible_items(&tasks);
    if visible.is_empty() {
        println!("No tasks");
    } else {
        for (index, item) in visible.iter().enumerate() {
            item.print_with_index(index + 1);
        }
    }
    Ok(())
}

pub fn archived(storage: &dyn TaskStorage) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = storage.read()?;
    print!("{}", archived_tasks_as_markdown(tasks));
    Ok(())
}

pub fn edit_link(
    storage: &mut dyn TaskStorage,
    id: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
    let Some(item) = tasks.iter_mut().find(|t| t.id == id) else {
        eprintln!("⚠️ No task found with id {id}");
        return Ok(());
    };
    let current_link = item.link.clone().unwrap_or_default();

    let new_link = edit(&current_link)?;
    let new_link = new_link.trim().to_string();
    if new_link.is_empty() {
        eprintln!("⚠️ link cannot be empty");
        return Ok(());
    }

    if new_link == current_link {
        println!("No changes made");
        return Ok(());
    }

    // Update the task
    println!("Updated task: {}", new_link);
    item.link = Some(new_link);
    storage.write(tasks)?;
    Ok(())
}

pub fn edit_task(
    storage: &mut dyn TaskStorage,
    id: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
    let Some(item) = tasks.iter_mut().find(|t| t.id == id) else {
        return Err("No task found to edit".into());
    };

    let new_title = edit(&item.title)?;
    let new_title = new_title.trim().to_string();
    if new_title.is_empty() {
        eprintln!("⚠️ Title cannot be empty");
        return Ok(());
    }

    if new_title != item.title {
        item.title = new_title.clone();
        storage.write(tasks)?;
        println!("Updated task: {}", new_title);
    }

    let confirm_answer = Confirm::new("Do you want to edit the link?")
        .with_default(false)
        .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
        .prompt();
    if confirm_answer.is_ok_and(|x| x) {
        edit_link(storage, id)?;
    }
    Ok(())
}

pub fn work(
    storage: &mut dyn TaskStorage,
    ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
    let mut did_change = false;
    for id in ids {
        let Some(item) = tasks.iter_mut().find(|t| &t.id == id) else {
            eprintln!("⚠️ No task found with id {id}");
            return Ok(());
        };
        if item.status == Status::InProgress {
            item.status = Status::Todo;
            item.active_date = None;
            did_change = true;
            println!("Moved task back to todo: {}", item.title);
        } else {
            item.status = Status::InProgress;
            item.active_date = Some(Local::now().date_naive());
            did_change = true;
            println!("Started task: {}", item.title);
        }
    }
    if did_change {
        storage.write(tasks)?;
    }
    Ok(())
}

pub fn star(
    storage: &mut dyn TaskStorage,
    ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
    let mut did_change = false;
    for id in ids {
        let Some(item) = tasks.iter_mut().find(|t| &t.id == id) else {
            eprintln!("⚠️ No task found with id {id}");
            return Ok(());
        };
        if item.star.is_some_and(|v| v) {
            item.star = None;
        } else {
            item.star = Some(true);
        }
        did_change = true;
    }
    if did_change {
        storage.write(tasks)?;
        println!("Starred the selected tasks");
    } else {
        println!("No tasks selected");
    }
    Ok(())
}

pub fn done(
    storage: &mut dyn TaskStorage,
    ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
    let mut did_change = false;
    for id in ids {
        let Some(item) = tasks.iter_mut().find(|t| &t.id == id) else {
            eprintln!("⚠️ No task found with id {id}");
            return Ok(());
        };
        if item.status == Status::Done {
            item.status = Status::Todo;
            item.active_date = None;
            did_change = true;
            println!("Moved task back to todo: {}", item.title);
        } else {
            item.status = Status::Done;
            item.active_date = Some(Local::now().date_naive());
            did_change = true;
            println!("Completed task: {}", item.title);
        }
    }
    if did_change {
        storage.write(tasks)?;
    }
    Ok(())
}

pub fn delete(
    storage: &mut dyn TaskStorage,
    ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
    tasks.retain(|i| !ids.contains(&i.id));
    storage.write(tasks)?;
    Ok(())
}

pub fn copy_archived(storage: &dyn TaskStorage) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = get_archived_tasks(storage)?;
    // We have to strip escape codes to remove the color.
    let text = strip_ansi_escapes::strip_str(archived_tasks_as_markdown(tasks));
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)?;
    println!("Copied archived tasks as Markdown");
    Ok(())
}

pub fn copy_as_markdown_list_by_date(
    storage: &dyn TaskStorage,
    ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = get_tasks_for_ids(storage, ids)?;
    let text = tasks_as_markdown_by_date(tasks);
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)?;
    println!("Copied tasks as Markdown by date");
    Ok(())
}

pub fn copy_with_markdown_links(
    storage: &dyn TaskStorage,
    ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::new()?;
    let tasks = storage.read()?;
    let text_lines: Vec<String> = tasks
        .iter()
        .filter_map(|i| {
            if ids.contains(&i.id) {
                return Some(TaskItemCopyableMarkdown(i).to_string());
            }
            None
        })
        .collect();
    let text = text_lines.join("\n");
    clipboard.set_text(text)?;
    match text_lines.len() {
        0 => println!("No tasks to copy"),
        1 => println!("Copied text for task: {}", text_lines[0]),
        _ => println!("Copied text for selected tasks"),
    };
    Ok(())
}

pub fn copy(storage: &dyn TaskStorage, ids: &[usize]) -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::new()?;
    let tasks = storage.read()?;
    let text_lines: Vec<String> = tasks
        .iter()
        .filter_map(|i| {
            if ids.contains(&i.id) {
                return Some(TaskItemCopyable(i).to_string());
            }
            None
        })
        .collect();
    let text = text_lines.join("\n");
    clipboard.set_text(text)?;
    match text_lines.len() {
        0 => println!("No tasks to copy"),
        1 => println!("Copied text for task: {}", text_lines[0]),
        _ => println!("Copied text for selected tasks"),
    };
    Ok(())
}

pub fn clear(storage: &mut dyn TaskStorage) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = storage.read()?;
    let mut copies: Vec<TaskItem> = vec![];
    let mut next_id = get_next_id(&tasks);
    for item in tasks.iter_mut() {
        // Unstar all tasks.
        item.star = None;
        match item.status {
            Status::Done => item.status = Status::Archived,
            Status::InProgress => {
                // Duplicate in-progress tasks and complete one of them so you can see that this
                // task was worked on today.
                let copy = TaskItem {
                    id: next_id,
                    title: item.title.clone(),
                    link: item.link.clone(),
                    status: Status::Archived,
                    active_date: item.active_date,
                    ..Default::default()
                };
                next_id += 1;
                copies.push(copy);
                // Return in-progress tasks to To do
                item.status = Status::Todo;
            }
            _ => {}
        }
    }
    tasks.extend(copies);
    storage.write(tasks)?;
    println!("Archived completed tasks");
    Ok(())
}
