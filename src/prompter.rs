use inquire::{Confirm, Text};

pub trait Prompter {
    fn text(&self, message: &str) -> Result<String, Box<dyn std::error::Error>>;
    fn confirm(&self, message: &str) -> Result<bool, Box<dyn std::error::Error>>;
}

pub struct InquirePrompter;

impl Prompter for InquirePrompter {
    fn text(&self, message: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(Text::new(message).prompt()?)
    }

    fn confirm(&self, message: &str) -> Result<bool, Box<dyn std::error::Error>> {
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

    fn confirm(&self, _message: &str) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(true)
    }
}
