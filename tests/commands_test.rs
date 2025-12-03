#[cfg(test)]
mod tests {
    use fini::commands::{execute_command, Command};
    use fini::storage::{InMemoryStorage, TaskStorage};
    use fini::task_item::Status;
    use fini::util::get_id_for_index;

    #[test]
    fn test_add_command() {
        let mut storage = InMemoryStorage::new();
        let title = "Test task";

        let command = Command::Add {
            title: Some(title.to_string()),
            link: Some("https://example.com".to_string()),
        };
        let result = execute_command(&mut storage, command);

        assert!(result.is_ok());
        let tasks = storage.read().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_check_command() {
        let mut storage = InMemoryStorage::new();
        let title = "Test task";

        let command = Command::Add {
            title: Some(title.to_string()),
            link: Some("https://example.com".to_string()),
        };
        let result = execute_command(&mut storage, command);
        assert!(result.is_ok());

        let id = get_id_for_index(&storage, 1).unwrap().unwrap();
        let command = Command::Check { ids: Some( vec![ id ] ) };
        let result = execute_command(&mut storage, command);

        assert!(result.is_ok());
        let tasks = storage.read().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Done);
    }
}
