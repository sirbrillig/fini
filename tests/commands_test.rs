#[cfg(test)]
mod tests {
    use chrono::{Duration, Local};
    use fini::commands::{Command, LinkFormat, execute_command};
    use fini::copier::MockCopier;
    use fini::indices::{get_id_for_index, get_id_for_title};
    use fini::printer::MockPrinter;
    use fini::prompter::MockPrompter;
    use fini::storage::{FileStorage, InMemoryStorage, TaskStorage};
    use fini::task_item::Status;
    use strip_ansi_escapes::strip_str;

    struct TestContext {
        storage: InMemoryStorage,
        prompter: MockPrompter,
        copier: MockCopier,
        printer: MockPrinter,
    }

    impl TestContext {
        fn new() -> Self {
            Self {
                storage: InMemoryStorage::new(),
                prompter: MockPrompter::new(),
                copier: MockCopier::new(),
                printer: MockPrinter::new(),
            }
        }

        // Convenience method that handles all the borrowing
        fn execute(&mut self, command: Command) -> Result<(), Box<dyn std::error::Error>> {
            execute_command(
                &mut self.storage,
                &self.prompter,
                &mut self.copier,
                &mut self.printer,
                command,
            )
        }

        fn add_task(&mut self, title: &str, link: Option<String>) {
            let command = Command::Add {
                title: Some(title.to_string()),
                link,
            };
            let result = self.execute(command);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_add_command_with_link() {
        let mut ctx = TestContext::new();
        let title = "Test task";
        let link = "https://example.com";
        let command = Command::Add {
            title: Some(title.to_string()),
            link: Some(link.to_string()),
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].link, Some(link.to_string()));
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_add_command_with_prompted_title() {
        let mut ctx = TestContext::new();
        let title = "Test task";
        let link = "https://example.com";
        ctx.prompter.next_text_response = title.to_string();
        let command = Command::Add {
            title: None,
            link: Some(link.to_string()),
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].link, Some(link.to_string()));
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_add_command_with_prompted_link() {
        let mut ctx = TestContext::new();
        let title = "Test task";
        let link = "https://example.com";
        ctx.prompter.next_text_response = link.to_string();
        let command = Command::Add {
            title: Some(title.to_string()),
            link: None,
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].link, Some(link.to_string()));
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_add_command_without_link() {
        let mut ctx = TestContext::new();
        let title = "Test task";
        let command = Command::Add {
            title: Some(title.to_string()),
            link: None,
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(
            tasks.len(),
            1,
            "Task was not found while reading after adding"
        );
        assert_eq!(
            tasks[0].title, title,
            "Title of task was incorrect after adding"
        );
        assert_eq!(tasks[0].link, None);
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_archive_command_from_todo() {
        let mut ctx = TestContext::new();
        let title = "Test task";
        ctx.add_task(title, None);

        let id = get_id_for_index(&ctx.storage, 1).unwrap().unwrap();
        let command = Command::Archive {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 0);

        let mut archived = ctx.storage.read_archived().unwrap();
        assert_eq!(archived.len(), 1);

        // Sort the archived tasks so it's easier to examine them (archived order doesn't matter)
        archived.sort_by_key(|t| t.title.clone());

        // The first task (at done) was archived
        assert_eq!(archived[0].title, title);
        assert_eq!(archived[0].status, Status::Archived);
        assert_eq!(archived[0].link, None);
    }

    #[test]
    fn test_check_command_from_todo() {
        let mut ctx = TestContext::new();
        let title = "Test task";
        ctx.add_task(title, None);

        let id = get_id_for_index(&ctx.storage, 1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Done);
    }

    #[test]
    fn test_check_command_from_done() {
        let mut ctx = TestContext::new();
        let title = "Test task";
        ctx.add_task(title, None);

        let id = get_id_for_index(&ctx.storage, 1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        ctx.execute(command).unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_check_command_for_starred_tasks() {
        let mut ctx = TestContext::new();
        let title1 = "Test task 1";
        ctx.add_task(title1, None);

        let title2 = "Test task 2";
        ctx.add_task(title2, None);

        let title3 = "Test task 3";
        ctx.add_task(title3, None);

        // Starring the second item should cause it to become the first one
        let id = get_id_for_index(&ctx.storage, 2).unwrap().unwrap();
        let command = Command::Star {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);
        assert!(result.is_ok());

        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].title, title2);
        assert_eq!(tasks[0].status, Status::Todo);
        assert_eq!(tasks[0].star, Some(true));

        // This should check the second item because it's now the first which should cause it to
        // become the last one
        let id = get_id_for_index(&ctx.storage, 1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].title, title1);
        assert_eq!(tasks[0].status, Status::Todo);
        assert_eq!(tasks[0].star, None);
        assert_eq!(tasks[1].title, title3);
        assert_eq!(tasks[1].status, Status::Todo);
        assert_eq!(tasks[1].star, None);
        assert_eq!(tasks[2].title, title2);
        assert_eq!(tasks[2].status, Status::Done);
        assert_eq!(tasks[2].star, Some(true));
    }

    #[test]
    fn test_begin_command_from_todo() {
        let mut ctx = TestContext::new();
        let title = "Test task";
        ctx.add_task(title, None);

        let id = get_id_for_index(&ctx.storage, 1).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::InProgress);
    }

    #[test]
    fn test_begin_command_from_in_progess() {
        let mut ctx = TestContext::new();
        let title = "Test task";
        ctx.add_task(title, None);

        let id = get_id_for_index(&ctx.storage, 1).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        ctx.execute(command).unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_star_command_from_todo() {
        let mut ctx = TestContext::new();
        let title = "Test task";
        ctx.add_task(title, None);

        let id = get_id_for_index(&ctx.storage, 1).unwrap().unwrap();
        let command = Command::Star {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Todo);
        assert_eq!(tasks[0].star, Some(true));
    }

    #[test]
    fn test_star_command_from_starred() {
        let mut ctx = TestContext::new();
        let title = "Test task";
        ctx.add_task(title, None);

        let id = get_id_for_index(&ctx.storage, 1).unwrap().unwrap();
        let command = Command::Star {
            ids: Some(vec![id]),
        };
        ctx.execute(command).unwrap();
        let command = Command::Star {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title);
        assert_eq!(tasks[0].status, Status::Todo);
        assert_eq!(tasks[0].star, None);
    }

    #[test]
    fn test_delete_command() {
        let mut ctx = TestContext::new();
        let title1 = "Test task 1";
        ctx.add_task(title1, None);
        let title2 = "Test task 2";
        ctx.add_task(title2, None);

        let id = get_id_for_index(&ctx.storage, 1).unwrap().unwrap();
        let command = Command::Delete {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, title2);
        assert_eq!(tasks[0].status, Status::Todo);
    }

    #[test]
    fn test_list_command() {
        let mut ctx = TestContext::new();
        let title1 = "Test task 1";
        ctx.add_task(title1, None);

        let title2 = "Test task 2";
        ctx.add_task(title2, None);

        let title3 = "Test task 3";
        ctx.add_task(title3, None);

        let id = get_id_for_index(&ctx.storage, 3).unwrap().unwrap();
        let command = Command::Archive {
            ids: Some(vec![id]),
        };
        ctx.execute(command).unwrap();
        let id = get_id_for_index(&ctx.storage, 1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        ctx.execute(command).unwrap();

        ctx.printer.clear();

        let command = Command::List {
            format: LinkFormat::Hyperlink,
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        // Checked tasks go last
        let expected = format!(" 1.  ☐  {}\n 2.  ✔  {}\n", title2, title1);
        assert_eq!(strip_str(ctx.printer.text), expected);
    }

    #[test]
    fn test_list_command_for_starred_tasks() {
        let mut ctx = TestContext::new();
        let title1 = "Test task 1";
        ctx.add_task(title1, None);

        let title2 = "Test task 2";
        ctx.add_task(title2, None);

        let title3 = "Test task 3";
        ctx.add_task(title3, None);

        // Starring the second item should cause it to become the first one
        let id = get_id_for_index(&ctx.storage, 2).unwrap().unwrap();
        let command = Command::Star {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);
        assert!(result.is_ok());

        ctx.printer.clear();

        let command = Command::List {
            format: LinkFormat::Hyperlink,
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let expected = format!(
            " 1.★ ☐  {}\n 2.  ☐  {}\n 3.  ☐  {}\n",
            title2, title1, title3
        );
        assert_eq!(strip_str(ctx.printer.text), expected);
    }

    #[test]
    fn test_archived_command() {
        let mut ctx = TestContext::new();
        let title1 = "Test task 1";
        ctx.add_task(title1, None);

        let title2 = "Test task 2";
        ctx.add_task(title2, None);

        let id = get_id_for_index(&ctx.storage, 1).unwrap().unwrap();
        let command = Command::Archive {
            ids: Some(vec![id]),
        };
        ctx.execute(command).unwrap();
        ctx.printer.clear();

        let command = Command::Archived {};
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let expected = format!("\n## {}\n- {}\n", Local::now().format("%Y-%m-%d"), title1);
        assert_eq!(strip_str(ctx.printer.text), expected);
    }

    #[test]
    fn test_clear_command() {
        let mut ctx = TestContext::new();

        // Add tasks
        let title1 = "Test task 1";
        ctx.add_task(title1, None);

        let title2 = "Test task 2";
        ctx.add_task(title2, None);

        let title3 = "Test task 3";
        ctx.add_task(title3, None);

        let title4 = "Test task 4";
        let link4 = Some("https://example4.com".to_string());
        ctx.add_task(title4, link4.clone());

        let title5 = "Test task 5";
        let link5 = Some("https://example5.com".to_string());
        ctx.add_task(title5, link5.clone());

        // Check first task (which makes it the last task)
        let id = get_id_for_title(&ctx.storage, title1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        ctx.execute(command).unwrap();

        // Begin second task (which is now the first task)
        let id = get_id_for_title(&ctx.storage, title2).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        ctx.execute(command).unwrap();

        // Check fourth task
        let id = get_id_for_title(&ctx.storage, title4).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        ctx.execute(command).unwrap();

        // Begin fifth task
        let id = get_id_for_title(&ctx.storage, title5).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        ctx.execute(command).unwrap();

        let command = Command::Clear {};
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let tasks = ctx.storage.read_tasks().unwrap();
        let mut archived = ctx.storage.read_archived().unwrap();
        assert_eq!(tasks.len(), 3);
        assert_eq!(archived.len(), 4);

        // Sort the archived tasks so it's easier to examine them (archived order doesn't matter)
        archived.sort_by_key(|t| t.title.clone());

        // The first task (at done) was archived
        assert_eq!(archived[0].title, title1);
        assert_eq!(archived[0].status, Status::Archived);
        assert_eq!(archived[0].link, None);

        // The third task (at todo) remains unchanged
        assert_eq!(tasks[0].title, title3);
        assert_eq!(tasks[0].status, Status::Todo);
        assert_eq!(tasks[0].link, None);

        // The fifth task (at begin) was changed back to Todo
        assert_eq!(tasks[1].title, title5);
        assert_eq!(tasks[1].status, Status::Todo);
        assert_eq!(tasks[1].link, link5);

        // The second task (at begin) was changed back to Todo
        assert_eq!(tasks[2].title, title2);
        assert_eq!(tasks[2].status, Status::Todo);
        assert_eq!(tasks[2].link, None);

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
        let mut ctx = TestContext::new();

        // Add tasks
        let title1 = "Test task 1";
        ctx.add_task(title1, None);

        let title2 = "Test task 2";
        ctx.add_task(title2, None);

        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(tasks[0].title, title1);
        assert_eq!(tasks[1].title, title2);

        // Check first task (this will make it the last task)
        let id = get_id_for_index(&ctx.storage, 1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        ctx.execute(command).unwrap();
        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(tasks[0].title, title2);
        assert_eq!(tasks[1].title, title1);

        // Begin second task (which is now the first task)
        let id = get_id_for_index(&ctx.storage, 1).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        ctx.execute(command).unwrap();
        let tasks = ctx.storage.read_tasks().unwrap();
        assert_eq!(tasks[0].title, title2);
        assert_eq!(tasks[1].title, title1);

        let command = Command::Clear {};
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let tasks = ctx.storage.read_tasks().unwrap();
        let archived = ctx.storage.read_archived().unwrap();
        assert_eq!(
            tasks.len(),
            1,
            "There should only be one task after clearing the checked task"
        );
        assert_eq!(
            archived.len(),
            2,
            "There should be two archived tasks: the checked one and a duplicate of the begun one"
        );

        // Add more tasks
        let title3 = "Test task 3";
        ctx.add_task(title3, None);

        // Check task
        let id = get_id_for_index(&ctx.storage, 1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        ctx.execute(command).unwrap();

        let command = Command::Clear {};
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let tasks = ctx.storage.read_tasks().unwrap();
        let archived = ctx.storage.read_archived().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(archived.len(), 3);
    }

    #[test]
    fn test_copy_command() {
        let mut ctx = TestContext::new();
        let title1 = "Test task 1";
        ctx.add_task(title1, None);
        let title2 = "Test task 2";
        ctx.add_task(title2, None);

        let id = get_id_for_index(&ctx.storage, 1).unwrap().unwrap();
        let command = Command::Copy {
            ids: Some(vec![id]),
            format: LinkFormat::Adjacent,
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        assert_eq!(ctx.copier.text, title1);
    }

    #[test]
    fn test_copy_checked_command() {
        let mut ctx = TestContext::new();
        let title1 = "Test task 1";
        ctx.add_task(title1, None);
        let title2 = "Test task 2";
        ctx.add_task(title2, None);
        let title3 = "Test task 3";
        ctx.add_task(title3, None);
        let title4 = "Test task 4";
        ctx.add_task(title4, None);

        // First task is done
        let id = get_id_for_title(&ctx.storage, title1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);
        assert!(result.is_ok());

        // Second task is started
        let id = get_id_for_title(&ctx.storage, title2).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);
        assert!(result.is_ok());

        let command = Command::CopyChecked {
            format: LinkFormat::Adjacent,
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let expected = format!("{}\n{}", title2, title1,);
        assert_eq!(ctx.copier.text, expected);
    }

    #[test]
    fn test_list_between_command() {
        let mut ctx = TestContext::new();
        let title1 = "Test task 1";
        ctx.add_task(title1, None);
        let title2 = "Test task 2";
        ctx.add_task(title2, None);
        let title3 = "Test task 3";
        ctx.add_task(title3, None);
        let title4 = "Test task 4";
        ctx.add_task(title4, None);
        let title5 = "Test task 5";
        ctx.add_task(title5, None);

        // First task is done
        let id = get_id_for_title(&ctx.storage, title1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);
        assert!(result.is_ok());

        // Second task is started
        let id = get_id_for_title(&ctx.storage, title2).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);
        assert!(result.is_ok());

        // Third task is archived
        let id = get_id_for_title(&ctx.storage, title3).unwrap().unwrap();
        let command = Command::Archive {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);
        assert!(result.is_ok());

        // Fourth task is archived but before date.
        // Manually set the fourth task's active_date to 2 days ago.
        let two_days_ago = Some((Local::now() - Duration::days(2)).date_naive());
        let id = get_id_for_title(&ctx.storage, title4).unwrap().unwrap();
        let mut tasks = ctx.storage.read_tasks().unwrap();
        let Some(task) = tasks.iter_mut().find(|t| t.id == id) else {
            panic!("Cannot find last task in archive");
        };
        task.active_date = two_days_ago;
        ctx.storage.write_tasks(tasks).unwrap();
        let id = get_id_for_title(&ctx.storage, title4).unwrap().unwrap();
        let command = Command::Archive {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);
        assert!(result.is_ok());
        let archived = ctx.storage.read_archived().unwrap();
        assert_eq!(archived.len(), 2);

        // Fifth task remains in Todo

        // List should ignore Todo task (task5) and task before date (task4)
        ctx.printer.clear();
        let command = Command::ListBetween {
            date_a: Some(
                (Local::now() - Duration::days(1))
                    .format("%Y-%m-%d")
                    .to_string(),
            ),
            date_b: Some(Local::now().format("%Y-%m-%d").to_string()),
            format: LinkFormat::Adjacent,
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let expected = format!(
            "\n## {}\n- {}\n- {}\n- {}\n",
            Local::now().format("%Y-%m-%d"),
            title2,
            title1,
            title3
        );
        assert_eq!(ctx.printer.text, expected);

        // List should include only task4
        ctx.printer.clear();
        let command = Command::ListBetween {
            date_a: Some(
                (Local::now() - Duration::days(2))
                    .format("%Y-%m-%d")
                    .to_string(),
            ),
            date_b: Some(
                (Local::now() - Duration::days(1))
                    .format("%Y-%m-%d")
                    .to_string(),
            ),
            format: LinkFormat::Adjacent,
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let expected = format!(
            "\n## {}\n- {}\n",
            (Local::now() - Duration::days(2)).format("%Y-%m-%d"),
            title4
        );
        assert_eq!(ctx.printer.text, expected);
    }

    #[test]
    fn test_copy_after_command() {
        let mut ctx = TestContext::new();
        let title1 = "Test task 1";
        ctx.add_task(title1, None);
        let title2 = "Test task 2";
        ctx.add_task(title2, None);
        let title3 = "Test task 3";
        ctx.add_task(title3, None);
        let title4 = "Test task 4";
        ctx.add_task(title4, None);
        let title5 = "Test task 5";
        ctx.add_task(title5, None);

        // First task is done
        let id = get_id_for_title(&ctx.storage, title1).unwrap().unwrap();
        let command = Command::Check {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);
        assert!(result.is_ok());

        // Second task is started
        let id = get_id_for_title(&ctx.storage, title2).unwrap().unwrap();
        let command = Command::Begin {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);
        assert!(result.is_ok());

        // Third task is archived
        let id = get_id_for_title(&ctx.storage, title3).unwrap().unwrap();
        let command = Command::Archive {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);
        assert!(result.is_ok());

        // Fourth task is archived but before date (note that index is 3 because the third task was
        // already archived).
        //
        // Manually set the fourth task's active_date to 2 days ago.
        let id = get_id_for_title(&ctx.storage, title4).unwrap().unwrap();
        let mut tasks = ctx.storage.read_tasks().unwrap();
        let Some(task) = tasks.iter_mut().find(|t| t.id == id) else {
            panic!("Cannot find last task in archive");
        };
        task.active_date = Some((Local::now() - Duration::days(2)).date_naive());
        ctx.storage.write_tasks(tasks).unwrap();
        let id = get_id_for_title(&ctx.storage, title4).unwrap().unwrap();
        let command = Command::Archive {
            ids: Some(vec![id]),
        };
        let result = ctx.execute(command);
        assert!(result.is_ok());
        let archived = ctx.storage.read_archived().unwrap();
        assert_eq!(archived.len(), 2);

        // Fifth task remains in Todo

        // Copy should ignore Todo task (task5) and task before date (task4)
        let command = Command::CopyAfterDate {
            date: Some(Local::now().format("%Y-%m-%d").to_string()),
            format: LinkFormat::Adjacent,
        };
        let result = ctx.execute(command);

        assert!(result.is_ok());
        let expected = format!(
            "\n## {}\n- {}\n- {}\n- {}",
            Local::now().format("%Y-%m-%d"),
            title2,
            title1,
            title3
        );
        assert_eq!(ctx.copier.text, expected);
    }

    #[test]
    fn test_boards_store_data_separately() -> Result<(), Box<dyn std::error::Error>> {
        let dir_a = tempfile::tempdir()?;
        let dir_b = tempfile::tempdir()?;

        let mut storage_a = FileStorage::new(dir_a.path().to_path_buf())?;
        let storage_b = FileStorage::new(dir_b.path().to_path_buf())?;

        let prompter = MockPrompter::new();
        let mut copier = MockCopier::new();
        let mut printer = MockPrinter::new();

        execute_command(
            &mut storage_a,
            &prompter,
            &mut copier,
            &mut printer,
            Command::Add {
                title: Some("Task on board A".to_string()),
                link: None,
            },
        )?;

        let tasks_a = storage_a.read_tasks()?;
        let tasks_b = storage_b.read_tasks()?;

        assert_eq!(tasks_a.len(), 1);
        assert_eq!(tasks_a[0].title, "Task on board A");
        assert!(
            tasks_b.is_empty(),
            "board B should be unaffected by writes to board A"
        );
        Ok(())
    }

    #[test]
    fn test_boards_can_hold_different_tasks() -> Result<(), Box<dyn std::error::Error>> {
        let dir_a = tempfile::tempdir()?;
        let dir_b = tempfile::tempdir()?;

        let mut storage_a = FileStorage::new(dir_a.path().to_path_buf())?;
        let mut storage_b = FileStorage::new(dir_b.path().to_path_buf())?;

        let prompter = MockPrompter::new();
        let mut copier = MockCopier::new();
        let mut printer = MockPrinter::new();

        execute_command(
            &mut storage_a,
            &prompter,
            &mut copier,
            &mut printer,
            Command::Add {
                title: Some("Board A task".to_string()),
                link: None,
            },
        )?;

        execute_command(
            &mut storage_b,
            &prompter,
            &mut copier,
            &mut printer,
            Command::Add {
                title: Some("Board B task".to_string()),
                link: None,
            },
        )?;

        let tasks_a = storage_a.read_tasks()?;
        let tasks_b = storage_b.read_tasks()?;

        assert_eq!(tasks_a.len(), 1);
        assert_eq!(tasks_b.len(), 1);
        assert_eq!(tasks_a[0].title, "Board A task");
        assert_eq!(tasks_b[0].title, "Board B task");
        Ok(())
    }
}
