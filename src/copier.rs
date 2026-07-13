use arboard::Clipboard;

pub enum CopyPayload {
    Text(String),
    Html { html: String, alt: String },
}

pub trait Copier {
    fn copy(&mut self, payload: &CopyPayload) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct ClipboardCopier;

impl Copier for ClipboardCopier {
    fn copy(&mut self, payload: &CopyPayload) -> Result<(), Box<dyn std::error::Error>> {
        let mut clipboard = Clipboard::new()?;
        match payload {
            CopyPayload::Text(text) => clipboard.set_text(text)?,
            CopyPayload::Html { html, alt } => clipboard.set_html(html, Some(alt))?,
        };
        Ok(())
    }
}

#[derive(Default)]
pub struct MockCopier {
    pub text: String,
    pub html: String,
    pub alt: String,
}

impl MockCopier {
    pub fn new() -> Self {
        Self {
            text: "".to_string(),
            html: "".to_string(),
            alt: "".to_string(),
        }
    }
}

impl Copier for MockCopier {
    fn copy(&mut self, payload: &CopyPayload) -> Result<(), Box<dyn std::error::Error>> {
        match payload {
            CopyPayload::Text(text) => self.text = text.to_string(),
            CopyPayload::Html { html, alt } => {
                self.html = html.to_string();
                self.alt = alt.to_string();
            }
        };
        Ok(())
    }
}
