#[cfg(test)]
mod tests {
    use fini::commands::{execute_command, Command};
    use fini::prompter::{MockPrompter, Prompter};
    use fini::storage::{InMemoryStorage, TaskStorage};
    use fini::task_item::Status;
    use fini::util::get_id_for_index;

    fn add_task(
        storage: &mut dyn TaskStorage,
        prompter: &dyn Prompter,
        title: &str,
        link: Option<String>,
    ) {
        let command = Command::Add {
            title: Some(title.to_string()),
            link,
        };
        let result = execute_command(storage, prompter, command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_command_with_link() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let title = "Test task";
        let link = "https://example.com";
        let command = Command::Add {
            title: Some(title.to_string()),
            link: Some(link.to_string()),
        };
        let result = execute_command(&mut storage, &prompter, command);

        assert!(result.is_ok());
        let tasks = storage.read().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].link, Some(link.to_string()));
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_add_command_with_prompted_title() {
        let mut storage = InMemoryStorage::new();
        let mut prompter = MockPrompter::new();
        let title = "Test task";
        let link = "https://example.com";
        prompter.next_text_response = title.to_string();
        let command = Command::Add {
            title: None,
            link: Some(link.to_string()),
        };
        let result = execute_command(&mut storage, &prompter, command);

        assert!(result.is_ok());
        let tasks = storage.read().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].link, Some(link.to_string()));
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_add_command_with_prompted_link() {
        let mut storage = InMemoryStorage::new();
        let mut prompter = MockPrompter::new();
        let title = "Test task";
        let link = "https://example.com";
        prompter.next_text_response = link.to_string();
        let command = Command::Add {
            title: Some(title.to_string()),
            link: None,
        };
        let result = execute_command(&mut storage, &prompter, command);

        assert!(result.is_ok());
        let tasks = storage.read().unwrap();
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
        let command = Command::Add {
            title: Some(title.to_string()),
            link: None,
        };
        let result = execute_command(&mut storage, &prompter, command);

        assert!(result.is_ok());
        let tasks = storage.read().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].link, None);
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_check_command_from_todo() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let title = "Test task";
        add_task(&mut storage, &prompter, title, None);

        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, command);

        assert!(result.is_ok());
        let tasks = storage.read().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Done);
    }

    #[test]
    fn test_check_command_from_done() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let title = "Test task";
        add_task(&mut storage, &prompter, title, None);

        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        execute_command(&mut storage, &prompter, command).unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, command);

        assert!(result.is_ok());
        let tasks = storage.read().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_begin_command_from_todo() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let title = "Test task";
        add_task(&mut storage, &prompter, title, None);

        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, command);

        assert!(result.is_ok());
        let tasks = storage.read().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::InProgress);
    }

    #[test]
    fn test_begin_command_from_in_progess() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let title = "Test task";
        add_task(&mut storage, &prompter, title, None);

        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        execute_command(&mut storage, &prompter, command).unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, command);

        assert!(result.is_ok());
        let tasks = storage.read().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_star_command_from_todo() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let title = "Test task";
        add_task(&mut storage, &prompter, title, None);

        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Star {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, command);

        assert!(result.is_ok());
        let tasks = storage.read().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Todo);
        assert_eq!(tasks[0].star, Some(true));
    }

    #[test]
    fn test_star_command_from_starred() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let title = "Test task";
        add_task(&mut storage, &prompter, title, None);

        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Star {
            ids: Some(vec![id]),
        };
        execute_command(&mut storage, &prompter, command).unwrap();
        let command = Command::Star {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, command);

        assert!(result.is_ok());
        let tasks = storage.read().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Todo);
        assert_eq!(tasks[0].star, None);
    }

    #[test]
    fn test_delete_command() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let title1 = "Test task 1";
        add_task(&mut storage, &prompter, title1, None);
        let title2 = "Test task 2";
        add_task(&mut storage, &prompter, title2, None);

        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Delete {
            ids: Some(vec![id]),
        };
        let result = execute_command(&mut storage, &prompter, command);

        assert!(result.is_ok());
        let tasks = storage.read().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title2);
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_clear_command() {
        let mut storage = InMemoryStorage::new();
        let prompter = MockPrompter::new();
        let title1 = "Test task 1";
        add_task(&mut storage, &prompter, title1, None);
        let title2 = "Test task 2";
        add_task(&mut storage, &prompter, title2, None);
        let title3 = "Test task 3";
        add_task(&mut storage, &prompter, title3, None);
        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        execute_command(&mut storage, &prompter, command).unwrap();
        let id = get_id_for_index(&storage, 2).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        execute_command(&mut storage, &prompter, command).unwrap();

        let command = Command::Clear {};
        let result = execute_command(&mut storage, &prompter, command);

        assert!(result.is_ok());
        let tasks = storage.read().unwrap();
        assert_eq!(tasks.len(), 4);
        // The first task (at done) was archived
        assert_eq!(tasks[0].title, title1);
        assert_eq!(tasks[0].status, Status::Archived);
        // The second task (at begin) was changed back to Todo
        assert_eq!(tasks[1].title, title2);
        assert_eq!(tasks[1].status, Status::Todo);
        // The third task (at todo) remains unchanged
        assert_eq!(tasks[2].title, title3);
        assert_eq!(tasks[2].status, Status::Todo);
        // The second task (since it was at begin) was duplicated and archived
        assert_eq!(tasks[3].title, title2);
        assert_eq!(tasks[3].status, Status::Archived);
    }
}
