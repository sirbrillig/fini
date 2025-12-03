#[cfg(test)]
mod tests {
    use fini::commands::{execute_command, Command};
    use fini::prompter::{MockPrompter, Prompter};
    use fini::storage::{InMemoryStorage, TaskStorage};
    use fini::task_item::Status;
    use fini::util::get_id_for_index;

    fn add_task(storage: &mut dyn TaskStorage, prompter: &dyn Prompter, title: &str, link: Option<String>) {
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
        let command = Command::Check { ids: Some( vec![ id ] ) };
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
        let command = Command::Check { ids: Some( vec![ id ] ) };
        execute_command(&mut storage, &prompter, command).unwrap();
        let command = Command::Check { ids: Some( vec![ id ] ) };
        let result = execute_command(&mut storage, &prompter, command);

        assert!(result.is_ok());
        let tasks = storage.read().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Todo);
    }
}
