use crate::commands::{execute_command, Command};
use crate::task_item::{Status, TaskItem, TaskItemCopyable};
use crate::util::{
    archived_tasks_as_markdown, get_archived_tasks, get_data_path, get_next_id, read_data,
    sort_visible_items, write_data,
};
use arboard::Clipboard;
use chrono::Local;
use colored::Colorize;
use edit::edit;
use inquire::{Confirm, Select};

pub fn interactive() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("{}", "-----------------------------------------".green());
        list();
        let commands = vec![
            "quit",
            "list",
            "add",
            "check",
            "begin",
            "star",
            "copy",
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
            "list-archived" => execute_command(Command::Archived)?,
            "clear" => execute_command(Command::Clear)?,
            "add" => execute_command(Command::Add { title: None })?,
            "copy" => execute_command(Command::Copy { ids: None })?,
            "copy-checked" => execute_command(Command::CopyChecked)?,
            "copy-archived" => execute_command(Command::CopyArchived)?,
            "copy-date" => execute_command(Command::CopyDate { date: None })?,
            "edit" => execute_command(Command::Edit { id: None })?,
            "check" => execute_command(Command::Check { ids: None })?,
            "star" => execute_command(Command::Star { ids: None })?,
            "delete-before" => execute_command(Command::DeleteBefore { date: None })?,
            "delete" => execute_command(Command::Delete { ids: None })?,
            "begin" => execute_command(Command::Begin { ids: None })?,
            _ => println!("Unknown command"),
        }
    }
    Ok(())
}

pub fn add(title: String, link: Option<String>) -> Result<usize, Box<dyn std::error::Error>> {
    let data_path = get_data_path();
    let mut items = read_data(&data_path);
    let id = get_next_id(&items);
    let item = TaskItem {
        id,
        title,
        link,
        ..Default::default()
    };
    println!("Added task: {}", &item.title);
    items.push(item);
    write_data(&data_path, items)?;
    Ok(id)
}

pub fn link(id: usize, link: String) -> Result<(), Box<dyn std::error::Error>> {
    let data_path = get_data_path();
    let mut items = read_data(&data_path);
    let Some(item) = items.iter_mut().find(|t| t.id == id) else {
        eprintln!("⚠️ No task found with id {id}");
        return Ok(());
    };
    item.link = Some(link);
    println!("Added link to task: {}", item.title);
    write_data(&data_path, items)?;
    Ok(())
}

pub fn list() {
    let data_path = get_data_path();
    let items = read_data(&data_path);
    let visible = sort_visible_items(&items);
    if visible.is_empty() {
        println!("No tasks");
    } else {
        for (index, item) in visible.iter().enumerate() {
            item.print_with_index(index + 1);
        }
    }
}

pub fn archived() {
    let data_path = get_data_path();
    let items = read_data(&data_path);
    print!("{}", archived_tasks_as_markdown(items));
}

pub fn edit_link(id: usize) -> Result<(), Box<dyn std::error::Error>> {
    let data_path = get_data_path();
    let mut items = read_data(&data_path);
    let Some(item) = items.iter_mut().find(|t| t.id == id) else {
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
    write_data(&data_path, items)?;
    Ok(())
}

pub fn edit_task(id: usize) -> Result<(), Box<dyn std::error::Error>> {
    let data_path = get_data_path();
    let mut items = read_data(&data_path);
    let Some(item) = items.iter_mut().find(|t| t.id == id) else {
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
        write_data(&data_path, items)?;
        println!("Updated task: {}", new_title);
    }

    let confirm_answer = Confirm::new("Do you want to edit the link?")
        .with_default(false)
        .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
        .prompt();
    if confirm_answer.is_ok_and(|x| x) {
        edit_link(id)?;
    }
    Ok(())
}

pub fn work(ids: &[usize]) -> Result<(), Box<dyn std::error::Error>> {
    let data_path = get_data_path();
    let mut items = read_data(&data_path);
    let mut did_change = false;
    for id in ids {
        let Some(item) = items.iter_mut().find(|t| &t.id == id) else {
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
        write_data(&data_path, items)?;
    }
    Ok(())
}

pub fn star(ids: &[usize]) -> Result<(), Box<dyn std::error::Error>> {
    let data_path = get_data_path();
    let mut items = read_data(&data_path);
    let mut did_change = false;
    for id in ids {
        let Some(item) = items.iter_mut().find(|t| &t.id == id) else {
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
        write_data(&data_path, items)?;
        println!("Starred the selected tasks");
    } else {
        println!("No tasks selected");
    }
    Ok(())
}

pub fn done(ids: &[usize]) -> Result<(), Box<dyn std::error::Error>> {
    let data_path = get_data_path();
    let mut items = read_data(&data_path);
    let mut did_change = false;
    for id in ids {
        let Some(item) = items.iter_mut().find(|t| &t.id == id) else {
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
        write_data(&data_path, items)?;
    }
    Ok(())
}

pub fn delete(ids: &[usize]) -> Result<(), Box<dyn std::error::Error>> {
    let data_path = get_data_path();
    let mut items = read_data(&data_path);
    items.retain(|i| !ids.contains(&i.id));
    write_data(&data_path, items)?;
    Ok(())
}

pub fn copy_archived() -> Result<(), Box<dyn std::error::Error>> {
    let items = get_archived_tasks();
    // We have to strip escape codes to remove the color.
    let text = strip_ansi_escapes::strip_str(archived_tasks_as_markdown(items));
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)?;
    println!("Copied archived tasks as Markdown");
    Ok(())
}

pub fn copy(ids: &[usize]) -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::new()?;
    let data_path = get_data_path();
    let items = read_data(&data_path);
    let text_lines: Vec<String> = items
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

pub fn clear() -> Result<(), Box<dyn std::error::Error>> {
    let data_path = get_data_path();
    let mut items = read_data(&data_path);
    let mut copies: Vec<TaskItem> = vec![];
    let mut next_id = get_next_id(&items);
    for item in items.iter_mut() {
        // Unstar all items.
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
    items.extend(copies);
    write_data(&data_path, items)?;
    println!("Archived completed tasks");
    Ok(())
}
