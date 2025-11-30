use crate::task_item::{Status, TaskItem, TaskItemCopyable};
use crate::util::{
    archived_tasks_as_markdown, get_archived_tasks, get_data_path, get_next_id,
    get_task_ids_before_date, get_task_ids_for_date, prompt_for_task_ids, read_data,
    sort_visible_items, write_data, SELECT_PAGE_SIZE,
};
use arboard::Clipboard;
use chrono::Local;
use colored::Colorize;
use edit::edit;
use inquire::{Confirm, Select, Text};

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
            "list-archived" => archived(),
            "clear" => {
                let confirm_answer =
                    Confirm::new("Are you sure you want to archive all complete tasks?")
                        .with_default(false)
                        .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
                        .prompt();
                if confirm_answer.is_ok_and(|x| x) {
                    clear()?;
                }
            }
            "add" => {
                let title = Text::new("Enter task:").prompt();
                let Ok(title) = title else {
                    println!("An error happened when asking for the task.");
                    continue;
                };
                if title.is_empty() {
                    println!("The title of the task cannot be empty.");
                    continue;
                }
                let link = Text::new("(Optional) Enter link:").prompt();
                let Ok(link) = link else {
                    continue;
                };
                if link.is_empty() {
                    add(title)?;
                } else {
                    add_with_link(title, link)?;
                }
            }
            "copy" => {
                if let Some(ids) = prompt_for_task_ids("Select tasks to copy") {
                    copy(ids)?;
                }
            }
            "copy-checked" => {
                let data_path = get_data_path();
                let items = read_data(&data_path);
                let ids = items
                    .iter()
                    .filter(|i| matches!(i.status, Status::Done | Status::InProgress))
                    .map(|i| i.id)
                    .collect();
                copy(ids)?;
            }
            "copy-archived" => {
                let items = get_archived_tasks();
                // We have to strip escape codes to remove the color.
                let text = strip_ansi_escapes::strip_str(archived_tasks_as_markdown(items));
                let mut clipboard = Clipboard::new()?;
                clipboard.set_text(text)?;
                println!("Copied archived tasks as Markdown");
            }
            "copy-date" => {
                let date = Text::new("Enter date:").prompt();
                if let Ok(date) = date {
                    copy(get_task_ids_for_date(date))?;
                }
            }
            "edit" => {
                let data_path = get_data_path();
                let items = read_data(&data_path);
                let visible = sort_visible_items(&items);
                if let Ok(selection) = Select::new("Select task to edit", visible)
                    .with_page_size(SELECT_PAGE_SIZE)
                    .prompt()
                {
                    edit_task(selection.id)?;
                }
            }
            "check" => {
                if let Some(ids) = prompt_for_task_ids("Select tasks to complete") {
                    done(ids)?;
                }
            }
            "star" => {
                if let Some(ids) = prompt_for_task_ids("Select tasks to star") {
                    star(ids)?;
                }
            }
            "delete-before" => {
                let date = Text::new("Enter date:").prompt();
                if let Ok(date) = date {
                    let confirm_answer = Confirm::new(&format!(
                        "Are you sure you want to delete all archived tasks before {}?",
                        date
                    ))
                    .with_default(false)
                    .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
                    .prompt();
                    if confirm_answer.is_ok_and(|x| x) {
                        delete(get_task_ids_before_date(date))?;
                    }
                }
            }
            "delete" => {
                if let Some(ids) = prompt_for_task_ids("Select tasks to delete") {
                    delete(ids)?;
                }
            }
            "begin" => {
                if let Some(ids) = prompt_for_task_ids("Select tasks to start") {
                    work(ids)?;
                }
            }
            _ => println!("Unknown command"),
        }
    }
    Ok(())
}

pub fn add(title: String) -> Result<usize, Box<dyn std::error::Error>> {
    let data_path = get_data_path();
    let mut items = read_data(&data_path);
    let id = get_next_id(&items);
    let item = TaskItem {
        id,
        title,
        ..Default::default()
    };
    println!("Added task: {}", &item.title);
    items.push(item);
    write_data(&data_path, items)?;
    Ok(id)
}

pub fn add_with_link(title: String, link: String) -> Result<usize, Box<dyn std::error::Error>> {
    let data_path = get_data_path();
    let mut items = read_data(&data_path);
    let id = get_next_id(&items);
    let item = TaskItem {
        id,
        title,
        link: Some(link),
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
    if let Some(item) = items.iter_mut().find(|t| t.id == id) {
        item.link = Some(link);
        println!("Added link to task: {}", item.title);
    }
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
    if let Some(item) = items.iter_mut().find(|t| t.id == id) {
        let item_id = item.id;
        let current_link = item.link.clone().unwrap_or_default();

        match edit(&current_link) {
            Ok(new_link) => {
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
                if let Some(item) = items.iter_mut().find(|t| t.id == item_id) {
                    item.link = Some(new_link.clone());
                    write_data(&data_path, items)?;
                    println!("Updated task: {}", new_link);
                    return Ok(());
                }
            }
            Err(e) => {
                eprintln!("⚠️ Error editing task: {}", e);
                return Ok(());
            }
        }
    }
    eprintln!("⚠️ No task found with id {id}");
    Ok(())
}

pub fn edit_task(id: usize) -> Result<(), Box<dyn std::error::Error>> {
    let data_path = get_data_path();
    let mut items = read_data(&data_path);
    if let Some(item) = items.iter_mut().find(|t| t.id == id) {
        let item_id = item.id;
        let current_title = item.title.clone();

        // Open editor with current title
        match edit(&current_title) {
            Ok(new_title) => {
                let new_title = new_title.trim().to_string();
                if new_title.is_empty() {
                    eprintln!("⚠️ Title cannot be empty");
                    return Ok(());
                }

                if new_title != current_title {
                    if let Some(item) = items.iter_mut().find(|t| t.id == item_id) {
                        item.title = new_title.clone();
                        write_data(&data_path, items)?;
                        println!("Updated task: {}", new_title);
                    }
                }

                let confirm_answer = Confirm::new("Do you want to edit the link?")
                    .with_default(false)
                    .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
                    .prompt();
                if confirm_answer.is_ok_and(|x| x) {
                    edit_link(id)?;
                }
                return Ok(());
            }
            Err(e) => {
                eprintln!("⚠️ Error editing task: {}", e);
                return Ok(());
            }
        }
    }
    eprintln!("⚠️ No task found with id {id}");
    Ok(())
}

pub fn work(ids: Vec<usize>) -> Result<(), Box<dyn std::error::Error>> {
    let data_path = get_data_path();
    let mut items = read_data(&data_path);
    let mut did_change = false;
    for id in ids {
        if let Some(item) = items.iter_mut().find(|t| t.id == id) {
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
        } else {
            eprintln!("⚠️ No task found with id {id}");
        }
    }
    if did_change {
        write_data(&data_path, items)?;
    }
    Ok(())
}

pub fn star(ids: Vec<usize>) -> Result<(), Box<dyn std::error::Error>> {
    let data_path = get_data_path();
    let mut items = read_data(&data_path);
    let mut did_change = false;
    for id in ids {
        if let Some(item) = items.iter_mut().find(|t| t.id == id) {
            if item.star.is_some_and(|v| v) {
                item.star = None;
            } else {
                item.star = Some(true);
            }
            did_change = true;
        } else {
            eprintln!("⚠️ No task found with id {id}");
        }
    }
    if did_change {
        write_data(&data_path, items)?;
        println!("Starred the selected tasks");
    } else {
        println!("No tasks selected");
    }
    Ok(())
}

pub fn done(ids: Vec<usize>) -> Result<(), Box<dyn std::error::Error>> {
    let data_path = get_data_path();
    let mut items = read_data(&data_path);
    let mut did_change = false;
    for id in ids {
        if let Some(item) = items.iter_mut().find(|t| t.id == id) {
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
        } else {
            eprintln!("⚠️ No task found with id {id}");
        }
    }
    if did_change {
        write_data(&data_path, items)?;
    }
    Ok(())
}

pub fn delete(ids: Vec<usize>) -> Result<(), Box<dyn std::error::Error>> {
    let data_path = get_data_path();
    let mut items = read_data(&data_path);
    items.retain(|i| !ids.contains(&i.id));
    write_data(&data_path, items)?;
    Ok(())
}

pub fn copy(ids: Vec<usize>) -> Result<(), Box<dyn std::error::Error>> {
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
