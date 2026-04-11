use inquire::{Confirm, Text};
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{CompletionType, Config, EditMode, Editor};
use std::cell::RefCell;

pub trait Prompter {
    fn text(&self, message: &str) -> Result<String, Box<dyn std::error::Error>>;
    fn text_with_initial(
        &self,
        message: &str,
        initial: &str,
    ) -> Result<String, Box<dyn std::error::Error>>;
    fn confirm(&self, message: &str) -> Result<bool, Box<dyn std::error::Error>>;
}

pub struct InquirePrompter;

impl Prompter for InquirePrompter {
    fn text(&self, message: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(Text::new(message).prompt()?)
    }

    fn text_with_initial(
        &self,
        message: &str,
        initial: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Ok(Text::new(message).with_initial_value(initial).prompt()?)
    }

    fn confirm(&self, message: &str) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(Confirm::new(message)
            .with_default(false)
            .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
            .prompt()?)
    }
}

pub struct VimPrompter {
    editor: RefCell<Editor<(), DefaultHistory>>,
}

impl VimPrompter {
    pub fn new() -> Result<Self, ReadlineError> {
        let config = Config::builder()
            .edit_mode(EditMode::Vi)
            .completion_type(CompletionType::List)
            .build();
        let editor = Editor::with_config(config)?;
        Ok(Self {
            editor: RefCell::new(editor),
        })
    }
}

impl Default for VimPrompter {
    fn default() -> Self {
        Self::new().expect("Failed to create VimPrompter")
    }
}

impl Prompter for VimPrompter {
    fn text(&self, message: &str) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = format!("{} ", message);
        let input = self.editor.borrow_mut().readline(&prompt)?;
        Ok(input)
    }

    fn text_with_initial(
        &self,
        message: &str,
        initial: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = format!("{} ", message);
        let input = self
            .editor
            .borrow_mut()
            .readline_with_initial(&prompt, (initial, ""))?;
        Ok(input)
    }

    fn confirm(&self, message: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // Use inquire for confirm since it's more suitable for yes/no
        Ok(Confirm::new(message)
            .with_default(false)
            .with_help_message("Type 'yes' or 'no' or 'y'/'n'")
            .prompt()?)
    }
}

pub struct MockPrompter {
    pub next_text_response: String,
}

impl Default for MockPrompter {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPrompter {
    pub fn new() -> Self {
        Self {
            next_text_response: "".to_string(),
        }
    }
}

impl Prompter for MockPrompter {
    fn text(&self, _message: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.next_text_response.clone())
    }

    fn text_with_initial(
        &self,
        _message: &str,
        _initial: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.next_text_response.clone())
    }

    fn confirm(&self, _message: &str) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(true)
    }
}
