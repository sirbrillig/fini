#[cfg(test)]
mod tests {
    use chrono::{Duration, Local};
    use fini::commands::{Command, LinkFormat, execute_command};
    use fini::copier::{Copier, MockCopier};
    use fini::indices::get_id_for_index;
    use fini::prompter::{MockPrompter, Prompter};
    use fini::storage::{InMemoryStorage, TaskStorage};
    use fini::task_item::Status;

    fn add_task(
        storage: &mut dyn TaskStorage,
        prompter: &dyn Prompter,
        copier: &mut dyn Copier,
        title: &str,
        link: Option<String>,
    ) {
        let command = Command::Add {
            title: Some(title.to_string()),
            link,
        };
        let result = execute_command(storage, prompter, copier, command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_command_with_link() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let mut copier = MockCopier::new();
        let title = "Test task";
        let link = "https://example.com";
        let command = Command::Add {
            title: Some(title.to_string()),
            link: Some(link.to_string()),
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);

        assert!(result.is_ok());
        let tasks = storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].link, Some(link.to_string()));
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_add_command_with_prompted_title() {
        let mut storage = InMemoryStorage::new();
        let mut prompter = MockPrompter::new();
        let mut copier = MockCopier::new();
        let title = "Test task";
        let link = "https://example.com";
        prompter.next_text_response = title.to_string();
        let command = Command::Add {
            title: None,
            link: Some(link.to_string()),
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);

        assert!(result.is_ok());
        let tasks = storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].link, Some(link.to_string()));
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_add_command_with_prompted_link() {
        let mut storage = InMemoryStorage::new();
        let mut prompter = MockPrompter::new();
        let mut copier = MockCopier::new();
        let title = "Test task";
        let link = "https://example.com";
        prompter.next_text_response = link.to_string();
        let command = Command::Add {
            title: Some(title.to_string()),
            link: None,
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);

        assert!(result.is_ok());
        let tasks = storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].link, Some(link.to_string()));
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_add_command_without_link() {
        let mut storage = InMemoryStorage::new();
        let title = "Test task";
        let prompter = MockPrompter::new();
        let mut copier = MockCopier::new();
        let command = Command::Add {
            title: Some(title.to_string()),
            link: None,
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);

        assert!(result.is_ok());
        let tasks = storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].link, None);
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_check_command_from_todo() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let mut copier = MockCopier::new();
        let title = "Test task";
        add_task(&mut storage, &prompter, &mut copier, title, None);

        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);

        assert!(result.is_ok());
        let tasks = storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Done);
    }

    #[test]
    fn test_check_command_from_done() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let mut copier = MockCopier::new();
        let title = "Test task";
        add_task(&mut storage, &prompter, &mut copier, title, None);

        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        execute_command(&mut storage, &prompter, &mut copier, command).unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);

        assert!(result.is_ok());
        let tasks = storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_begin_command_from_todo() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let mut copier = MockCopier::new();
        let title = "Test task";
        add_task(&mut storage, &prompter, &mut copier, title, None);

        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);

        assert!(result.is_ok());
        let tasks = storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::InProgress);
    }

    #[test]
    fn test_begin_command_from_in_progess() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let mut copier = MockCopier::new();
        let title = "Test task";
        add_task(&mut storage, &prompter, &mut copier, title, None);

        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        execute_command(&mut storage, &prompter, &mut copier, command).unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);

        assert!(result.is_ok());
        let tasks = storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_star_command_from_todo() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let mut copier = MockCopier::new();
        let title = "Test task";
        add_task(&mut storage, &prompter, &mut copier, title, None);

        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Star {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);

        assert!(result.is_ok());
        let tasks = storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Todo);
        assert_eq!(tasks[0].star, Some(true));
    }

    #[test]
    fn test_star_command_from_starred() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let mut copier = MockCopier::new();
        let title = "Test task";
        add_task(&mut storage, &prompter, &mut copier, title, None);

        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Star {
            ids: Some(vec![id]),
        };
        execute_command(&mut storage, &prompter, &mut copier, command).unwrap();
        let command = Command::Star {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);

        assert!(result.is_ok());
        let tasks = storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Todo);
        assert_eq!(tasks[0].star, None);
    }

    #[test]
    fn test_delete_command() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let mut copier = MockCopier::new();
        let title1 = "Test task 1";
        add_task(&mut storage, &prompter, &mut copier, title1, None);
        let title2 = "Test task 2";
        add_task(&mut storage, &prompter, &mut copier, title2, None);

        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Delete {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);

        assert!(result.is_ok());
        let tasks = storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title2);
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_clear_command() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let mut copier = MockCopier::new();

        // Add tasks
        let title1 = "Test task 1";
        add_task(&mut storage, &prompter, &mut copier, title1, None);

        let title2 = "Test task 2";
        add_task(&mut storage, &prompter, &mut copier, title2, None);

        let title3 = "Test task 3";
        add_task(&mut storage, &prompter, &mut copier, title3, None);

        let title4 = "Test task 4";
        let link4 = Some("https://example4.com".to_string());
        add_task(&mut storage, &prompter, &mut copier, title4, link4.clone());

        let title5 = "Test task 5";
        let link5 = Some("https://example5.com".to_string());
        add_task(&mut storage, &prompter, &mut copier, title5, link5.clone());

        // Check first task
        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        execute_command(&mut storage, &prompter, &mut copier, command).unwrap();

        // Begin second task
        let id = get_id_for_index(&storage, 2).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        execute_command(&mut storage, &prompter, &mut copier, command).unwrap();

        // Check fourth task
        let id = get_id_for_index(&storage, 4).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        execute_command(&mut storage, &prompter, &mut copier, command).unwrap();

        // Begin fifth task
        let id = get_id_for_index(&storage, 5).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        execute_command(&mut storage, &prompter, &mut copier, command).unwrap();

        let command = Command::Clear {};
        let result = execute_command(&mut storage, &prompter, &mut copier, command);

        assert!(result.is_ok());
        let tasks = storage.read_tasks().unwrap();
        let archived = storage.read_archived().unwrap();
        assert_eq!(tasks.len(), 3);
        assert_eq!(archived.len(), 4);

        // The first task (at done) was archived
        assert_eq!(archived[0].title, title1);
        assert_eq!(archived[0].status, Status::Archived);
        assert_eq!(archived[0].link, None);

        // The second task (at begin) was changed back to Todo
        assert_eq!(tasks[0].title, title2);
        assert_eq!(tasks[0].status, Status::Todo);
        assert_eq!(tasks[0].link, None);

        // The third task (at todo) remains unchanged
        assert_eq!(tasks[1].title, title3);
        assert_eq!(tasks[1].status, Status::Todo);
        assert_eq!(tasks[1].link, None);

        // The fifth task (at begin) was changed back to Todo
        assert_eq!(tasks[2].title, title5);
        assert_eq!(tasks[2].status, Status::Todo);
        assert_eq!(tasks[2].link, link5);

        // The second task (since it was at begin) was duplicated and archived
        assert_eq!(archived[1].title, title2);
        assert_eq!(archived[1].status, Status::Archived);
        assert_eq!(archived[1].link, None);

        // The fourth task (at done) was archived
        assert_eq!(archived[2].title, title4);
        assert_eq!(archived[2].status, Status::Archived);
        assert_eq!(archived[2].link, link4);

        // The fifth task (since it was at begin) was duplicated and archived
        assert_eq!(archived[3].title, title5);
        assert_eq!(archived[3].status, Status::Archived);
        assert_eq!(archived[3].link, link5);
    }

    #[test]
    fn test_clear_command_twice() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let mut copier = MockCopier::new();

        // Add tasks
        let title1 = "Test task 1";
        add_task(&mut storage, &prompter, &mut copier, title1, None);

        let title2 = "Test task 2";
        add_task(&mut storage, &prompter, &mut copier, title2, None);

        // Check first task
        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        execute_command(&mut storage, &prompter, &mut copier, command).unwrap();

        // Begin second task
        let id = get_id_for_index(&storage, 2).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        execute_command(&mut storage, &prompter, &mut copier, command).unwrap();

        let command = Command::Clear {};
        let result = execute_command(&mut storage, &prompter, &mut copier, command);

        assert!(result.is_ok());
        let tasks = storage.read_tasks().unwrap();
        let archived = storage.read_archived().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(archived.len(), 2);

        // Add more tasks
        let title3 = "Test task 3";
        add_task(&mut storage, &prompter, &mut copier, title3, None);

        // Check task
        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        execute_command(&mut storage, &prompter, &mut copier, command).unwrap();

        let command = Command::Clear {};
        let result = execute_command(&mut storage, &prompter, &mut copier, command);

        assert!(result.is_ok());
        let tasks = storage.read_tasks().unwrap();
        let archived = storage.read_archived().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(archived.len(), 3);
    }

    #[test]
    fn test_copy_command() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let mut copier = MockCopier::new();
        let title1 = "Test task 1";
        add_task(&mut storage, &prompter, &mut copier, title1, None);
        let title2 = "Test task 2";
        add_task(&mut storage, &prompter, &mut copier, title2, None);

        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Copy {
            ids: Some(vec![id]),
            format: LinkFormat::Adjacent,
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);

        assert!(result.is_ok());
        assert_eq!(copier.text, title1);
    }

    #[test]
    fn test_copy_after_command() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let mut copier = MockCopier::new();
        let title1 = "Test task 1";
        add_task(&mut storage, &prompter, &mut copier, title1, None);
        let title2 = "Test task 2";
        add_task(&mut storage, &prompter, &mut copier, title2, None);
        let title3 = "Test task 3";
        add_task(&mut storage, &prompter, &mut copier, title3, None);
        let title4 = "Test task 4";
        add_task(&mut storage, &prompter, &mut copier, title4, None);
        let title5 = "Test task 5";
        add_task(&mut storage, &prompter, &mut copier, title5, None);

        // First task is done
        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);
        assert!(result.is_ok());

        // Second task is started
        let id = get_id_for_index(&storage, 2).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);
        assert!(result.is_ok());

        // Third task is archived
        let id = get_id_for_index(&storage, 3).unwrap().unwrap();
        let command = Command::Archive {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);
        assert!(result.is_ok());

        // Foruth task is archived but before date
        let id = get_id_for_index(&storage, 3).unwrap().unwrap();
        let command = Command::Archive {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);
        assert!(result.is_ok());
        // Manually set the fourth task's active_date to 2 days ago
        let mut tasks = storage.read_tasks().unwrap();
        if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
            task.active_date = Some((Local::now() - Duration::days(2)).date_naive());
        }
        storage.write_tasks(tasks).unwrap();

        // Fifth task remains in Todo

        // Copy should ignore Todo task and task before date
        let command = Command::CopyAfterDate {
            date: Some(Local::now().format("%Y-%m-%d").to_string()),
            format: LinkFormat::Adjacent,
        };
        let result = execute_command(&mut storage, &prompter, &mut copier, command);

        assert!(result.is_ok());
        let expected = format!(
            "\n## {}\n- {}\n- {}\n- {}",
            Local::now().format("%Y-%m-%d"),
            title1,
            title2,
            title3
        );
        assert_eq!(copier.text, expected);
    }
}
