use arboard::Clipboard;

pub trait Copier {
    fn copy(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct ClipboardCopier;

impl Copier for ClipboardCopier {
    fn copy(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut clipboard = Clipboard::new()?;
        Ok(clipboard.set_text(text)?)
    }
}

#[derive(Default)]
pub struct MockCopier {
    pub text: String,
}

impl MockCopier {
    pub fn new() -> Self {
        Self {
            text: "".to_string(),
        }
    }
}

impl Copier for MockCopier {
    fn copy(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.text = text.to_string();
        Ok(())
    }
}
